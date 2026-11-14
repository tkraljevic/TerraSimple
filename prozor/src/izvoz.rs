//! Izvoz u KML ili KMZ za Google Earth i u GeoJSON (zaslon K). Prije izvoza prozorčić kaže što ide
//! u datoteku i što se usput mijenja ili gubi, jer nijedan od tih formata ne zna sve što zna projekt.

use crate::poruke::KatalogPoruka;
use formati::{OpsegIzvoza, PregledIzvoza};
use slojevi::{Projekt, Stavka};
use std::path::Path;

/// Kako stavka pregleda prolazi u KML.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prolaz {
    /// Ide u datoteku kakvo jest.
    Da,
    /// Ide, ali se mijenja.
    Pazi,
    /// Ne ide u datoteku.
    Ne,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StavkaPregleda {
    pub prolaz: Prolaz,
    pub naslov: String,
    pub opis: String,
}

/// U što se izvozi. Bira se u prozorčiću, a o njemu ovise pregled i nastavak datoteke.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FormatIzvoza {
    #[default]
    Kml,
    Kmz,
    GeoJson,
}

impl FormatIzvoza {
    pub const SVI: [Self; 3] = [Self::Kml, Self::Kmz, Self::GeoJson];

    #[must_use]
    pub const fn nastavak(self) -> &'static str {
        match self {
            Self::Kml => "kml",
            Self::Kmz => "kmz",
            Self::GeoJson => "geojson",
        }
    }

    #[must_use]
    pub const fn naziv(self) -> &'static str {
        match self {
            Self::Kml => "KML",
            Self::Kmz => "KMZ",
            Self::GeoJson => "GeoJSON",
        }
    }
}

/// Prozorčić izvoza: što se izvozi, pod kojim nazivom, u kojem formatu i što pregled kaže.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PripremaIzvoza {
    pub opseg: OpsegIzvoza,
    pub naziv: String,
    pub pregled: PregledIzvoza,
    pub format: FormatIzvoza,
}

impl PripremaIzvoza {
    /// Bez ijednog objekta nema što izvesti.
    #[must_use]
    pub const fn moze(&self) -> bool {
        self.pregled.objekata() > 0
    }

    #[must_use]
    pub fn stavke(&self, katalog: &KatalogPoruka) -> Vec<StavkaPregleda> {
        match self.format {
            FormatIzvoza::GeoJson => stavke_geojson(katalog, &self.pregled),
            FormatIzvoza::Kml | FormatIzvoza::Kmz => stavke_pregleda(katalog, &self.pregled),
        }
    }
}

fn stavka(katalog: &KatalogPoruka, prolaz: Prolaz, kljuc: &str, opis: String) -> StavkaPregleda {
    StavkaPregleda {
        prolaz,
        naslov: katalog.tekst(kljuc),
        opis,
    }
}

fn gubici(katalog: &KatalogPoruka, p: &PregledIzvoza) -> Vec<StavkaPregleda> {
    let pazi = |kljuc: &str| pazi(katalog, kljuc);
    let mut s = Vec::new();
    if p.drugi_sustav {
        s.push(pazi("izvoz-sustav"));
    }
    if p.isprekidane > 0 {
        s.push(pazi("izvoz-isprekidane"));
    }
    if p.srafure > 0 {
        s.push(pazi("izvoz-srafura"));
    }
    if p.natpisi_bez_tocaka > 0 {
        s.push(pazi("izvoz-natpisi"));
    }
    if p.kurziv > 0 {
        s.push(pazi("izvoz-kurziv"));
    }
    if p.izostavljene_vrijednosti > 0 {
        let broj = p.izostavljene_vrijednosti.to_string();
        let opis = katalog.s_tekstovima("izvoz-izostavljeno-opis", &[("broj", broj)]);
        s.push(stavka(katalog, Prolaz::Ne, "izvoz-izostavljeno", opis));
    }
    s
}

/// Stavke pregleda redom kao u nacrtu: prvo što ide, pa što se mijenja, pa što ne ide.
#[must_use]
pub fn stavke_pregleda(katalog: &KatalogPoruka, p: &PregledIzvoza) -> Vec<StavkaPregleda> {
    if p.objekata() == 0 {
        let opis = katalog.tekst("izvoz-prazno-opis");
        return vec![stavka(katalog, Prolaz::Ne, "izvoz-prazno", opis)];
    }
    let brojevi = [
        ("objekti", katalog.broj_objekata(p.objekata())),
        ("mape", katalog.broj_mapa(p.mapa)),
    ];
    let opis = katalog.s_tekstovima("izvoz-geometrije-opis", &brojevi);
    let mut s = vec![stavka(katalog, Prolaz::Da, "izvoz-geometrije", opis)];
    if !p.atributi.is_empty() {
        let popis = p.atributi.iter().cloned().collect::<Vec<_>>().join(", ");
        let opis = katalog.s_tekstovima("izvoz-atributi-opis", &[("popis", popis)]);
        s.push(stavka(katalog, Prolaz::Da, "izvoz-atributi", opis));
    }
    s.extend(gubici(katalog, p));
    s
}

fn pazi(katalog: &KatalogPoruka, kljuc: &str) -> StavkaPregleda {
    stavka(
        katalog,
        Prolaz::Pazi,
        kljuc,
        katalog.tekst(&format!("{kljuc}-opis")),
    )
}

/// GeoJSON nema mapa ni stila kakav čita QGIS, ali ima svojstva za sve stupce.
#[must_use]
pub fn stavke_geojson(katalog: &KatalogPoruka, p: &PregledIzvoza) -> Vec<StavkaPregleda> {
    let mut s = stavke_pregleda(katalog, p);
    if p.objekata() == 0 {
        return s;
    }
    s.retain(|st| st.prolaz != Prolaz::Pazi);
    if p.mapa > 0 {
        s.push(pazi(katalog, "izvoz-geojson-mape"));
    }
    s.push(pazi(katalog, "izvoz-geojson-stil"));
    if p.isprekidane + p.srafure + p.natpisi_bez_tocaka + p.kurziv > 0 {
        s.push(pazi(katalog, "izvoz-geojson-crte"));
    }
    if p.drugi_sustav {
        s.push(pazi(katalog, "izvoz-sustav"));
    }
    s.sort_by_key(|st| st.prolaz != Prolaz::Da);
    s
}

/// Naziv dokumenta i predložene datoteke: naziv mape ili sloja, a za cijeli projekt naziv datoteke.
#[must_use]
pub fn naziv_izvoza(
    projekt: &Projekt,
    opseg: OpsegIzvoza,
    staza_projekta: Option<&Path>,
    katalog: &KatalogPoruka,
) -> String {
    let naziv = match opseg {
        OpsegIzvoza::Stavka(Stavka::Mapa(m)) => projekt.mapa(m).map(|m| m.naziv.clone()),
        OpsegIzvoza::Stavka(Stavka::Sloj(s)) => projekt.sloj(s).map(|s| s.naziv.clone()),
        OpsegIzvoza::Projekt => staza_projekta
            .and_then(Path::file_stem)
            .map(|s| s.to_string_lossy().into_owned()),
    };
    naziv
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| katalog.tekst("izvoz-zadani-naziv"))
}

#[must_use]
pub fn pripremi(
    projekt: &Projekt,
    opseg: OpsegIzvoza,
    staza_projekta: Option<&Path>,
    katalog: &KatalogPoruka,
) -> PripremaIzvoza {
    PripremaIzvoza {
        opseg,
        naziv: naziv_izvoza(projekt, opseg, staza_projekta, katalog),
        pregled: formati::pregledaj(projekt, opseg, &crate::uvoz::nazivi(katalog)),
        format: FormatIzvoza::default(),
    }
}

/// Datoteka ide na mjesto koje je korisnik odabrao. Vraća poruku za prozorčić.
pub fn izvezi(
    projekt: &Projekt,
    priprema: &PripremaIzvoza,
    staza: &Path,
    katalog: &KatalogPoruka,
) -> Result<String, String> {
    let datoteka = staza
        .file_name()
        .map(|d| d.to_string_lossy().into_owned())
        .unwrap_or_default();
    let nazivi = crate::uvoz::nazivi(katalog);
    match formati::izvezi(projekt, priprema.opseg, &priprema.naziv, &nazivi, staza) {
        Ok(p) => Ok(katalog.s_tekstovima(
            "izvoz-gotov",
            &[
                ("datoteka", datoteka),
                ("objekti", katalog.broj_objekata(p.objekata())),
            ],
        )),
        Err(_) => Err(katalog.s_tekstovima("izvoz-greska", &[("datoteka", datoteka)])),
    }
}
