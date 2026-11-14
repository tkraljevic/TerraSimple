//! Uvoz KML-a, KMZ-a i GeoJSON-a u otvoreni projekt: poruke, mapa za uvezeno i pogled na nju.

use crate::karta_prozora::obuhvat_vrhova;
use crate::poruke::KatalogPoruka;
use crate::stanje::{Odabir, StanjePrikaza};
use formati::{GreskaKml, NaziviSlojeva, SazetakUvoza};
use koordinate::Sustav;
use slojevi::{Geometrija, IdMape, Projekt, Stavka, Uredjivac};
use std::path::Path;

/// Oznake KML-a koje karta ne prikazuje, s nazivom za korisnika.
const NAZIVI_PRESKOCENOG: [(&str, &str); 6] = [
    ("NetworkLink", "uvoz-preskoceno-mrezne-veze"),
    ("GroundOverlay", "uvoz-preskoceno-slike"),
    ("ScreenOverlay", "uvoz-preskoceno-slike-zaslona"),
    ("PhotoOverlay", "uvoz-preskoceno-fotografije"),
    ("Tour", "uvoz-preskoceno-obilasci"),
    ("Model", "uvoz-preskoceno-modeli"),
];

#[must_use]
pub fn nazivi(katalog: &KatalogPoruka) -> NaziviSlojeva {
    NaziviSlojeva {
        tocke: katalog.tekst("uvoz-sloj-tocke"),
        linije: katalog.tekst("uvoz-sloj-linije"),
        poligoni: katalog.tekst("uvoz-sloj-poligoni"),
        bez_naziva: katalog.tekst("uvoz-mapa-bez-naziva"),
    }
}

#[must_use]
pub fn poruka_greske(katalog: &KatalogPoruka, greska: GreskaKml) -> String {
    let kljuc = match greska {
        GreskaKml::Citanje => "uvoz-greska-citanje",
        GreskaKml::NeispravanXml { .. } => "uvoz-greska-xml",
        GreskaKml::PreDuboko => "uvoz-greska-duboko",
        GreskaKml::NeispravnaArhiva => "uvoz-greska-arhiva",
        GreskaKml::NemaKmlUArhivi => "uvoz-greska-nema-kml",
        GreskaKml::NedopustenaPutanja => "uvoz-greska-putanja",
        GreskaKml::PrevelikaArhiva => "uvoz-greska-prevelika",
        GreskaKml::NeispravanJson => "uvoz-greska-json",
        GreskaKml::NepoznatSustav => "uvoz-greska-sustav",
        GreskaKml::BezSustava { .. } => "uvoz-greska-bez-sustava",
    };
    katalog.tekst(kljuc)
}

/// Koliko je uvezeno, a ispod toga ono što karta ne prikazuje.
#[must_use]
pub fn poruka_sazetka(katalog: &KatalogPoruka, s: &SazetakUvoza) -> String {
    let mut poruka = katalog.s_tekstovima(
        "uvoz-sazetak",
        &[
            ("objekata", katalog.broj_objekata(s.objekata)),
            ("slojeva", katalog.broj_slojeva(s.slojeva)),
            ("mapa", katalog.s_brojem("uvoz-broj-mapa", s.mapa_ukupno)),
        ],
    );
    let gubici = popis_gubitaka(katalog, s);
    if !gubici.is_empty() {
        poruka.push_str("\n\n");
        poruka
            .push_str(&katalog.s_tekstovima("uvoz-nije-uvezeno", &[("popis", gubici.join(", "))]));
    }
    poruka
}

fn popis_gubitaka(katalog: &KatalogPoruka, s: &SazetakUvoza) -> Vec<String> {
    let g = &s.gubici;
    let mut popis: Vec<String> = NAZIVI_PRESKOCENOG
        .iter()
        .filter_map(|(oznaka, kljuc)| g.preskoceno.get(*oznaka).map(|n| (*kljuc, *n)))
        .chain([
            ("uvoz-preskoceno-prazne", g.prazne_geometrije),
            ("uvoz-preskoceno-koordinate", g.neispravne_koordinate),
            ("uvoz-preskoceno-odbijeno", s.odbijeno),
        ])
        .filter(|(_, n)| *n > 0)
        .map(|(kljuc, n)| format!("{}: {n}", katalog.tekst(kljuc)))
        .collect();
    popis.dedup();
    if !g.izostavljena_svojstva.is_empty() {
        let svojstva = g.izostavljena_svojstva.iter().cloned().collect::<Vec<_>>();
        let tekst = katalog.s_tekstovima(
            "uvoz-izostavljena-svojstva",
            &[("popis", svojstva.join(", "))],
        );
        popis.push(tekst);
    }
    popis
}

/// Zašto uvoz datoteke nije uspio.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GreskaUvozaDatoteke {
    Poruka(String),
    /// Datoteka ne kaže koordinatni sustav, pa treba pitati. Uz to ide prijedlog po koordinatama.
    BezSustava(Sustav),
}

/// Čita datoteku i uvozi je u novu mapu, kao jedan korak poništavanja. Nova mapa je odabrana i
/// otvorena, a njezine podmape zatvorene, da veliko stablo ne zatrpa ploču. `sustav` vrijedi za
/// datoteku koja ga sama ne zapisuje.
pub fn uvezi_datoteku(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
    staza: &Path,
    sustav: Option<Sustav>,
) -> Result<SazetakUvoza, GreskaUvozaDatoteke> {
    let dokument = formati::procitaj_u_sustavu(staza, sustav).map_err(|g| match g {
        GreskaKml::BezSustava { istok, sjever } => {
            GreskaUvozaDatoteke::BezSustava(formati::prijedlog_sustava(istok, sjever))
        }
        g => GreskaUvozaDatoteke::Poruka(poruka_greske(katalog, g)),
    })?;
    let ime = staza
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let sazetak = formati::uvezi(uredjivac, &dokument, &ime, &nazivi(katalog)).map_err(|g| {
        let opis = katalog.s_tekstovima("uvoz-greska-projekt", &[("opis", g.to_string())]);
        GreskaUvozaDatoteke::Poruka(opis)
    })?;
    let projekt = uredjivac.projekt();
    for podmapa in podmape(projekt, sazetak.mapa) {
        stanje.postavi_otvorenost_mape(podmapa, false);
    }
    stanje.postavi_otvorenost_mape(sazetak.mapa, true);
    stanje.odabrano = Some(Odabir::Mapa(sazetak.mapa));
    Ok(sazetak)
}

/// Sve mape ispod zadane, bez nje same.
#[must_use]
pub fn podmape(projekt: &Projekt, mapa: IdMape) -> Vec<IdMape> {
    let mut izlaz = Vec::new();
    let mut stog = vec![mapa];
    while let Some(m) = stog.pop() {
        for stavka in projekt.mapa(m).map(|m| m.djeca.as_slice()).unwrap_or(&[]) {
            if let Stavka::Mapa(d) = stavka {
                izlaz.push(*d);
                stog.push(*d);
            }
        }
    }
    izlaz
}

/// Sredina i zum na kojem se vide svi objekti mape i njezinih podmapa.
#[must_use]
pub fn obuhvat_mape(projekt: &Projekt, mapa: IdMape) -> Option<(f64, f64, f64)> {
    let mut mape = podmape(projekt, mapa);
    mape.push(mapa);
    let vrhovi = mape
        .into_iter()
        .filter_map(|m| projekt.mapa(m))
        .flat_map(|m| m.djeca.iter())
        .filter_map(|s| match s {
            Stavka::Sloj(id) => projekt.sloj(*id),
            Stavka::Mapa(_) => None,
        })
        .flat_map(|s| s.objekti.values())
        .flat_map(|o| match &o.geometrija {
            Geometrija::Tocka(v) => vec![v],
            Geometrija::Linija(v) => v.iter().collect(),
            Geometrija::Poligon(p) => p.iter().flatten().collect(),
        });
    obuhvat_vrhova(vrhovi)
}
