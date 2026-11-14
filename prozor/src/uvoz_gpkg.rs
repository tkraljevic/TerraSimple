//! Uvoz slojeva iz tuđeg GeoPackagea kao kopije u projekt (zaslon G, Odluke (14. 11. 2026.)).
//! Izvorna datoteka se samo čita. Mape iz datoteke ostaju mape, a sve ide u novu mapu nazvanu po
//! datoteci, kao jedan korak poništavanja.

use crate::poruke::KatalogPoruka;
use koordinate::Sustav;
use slojevi::{
    Greska, IdMape, IdSloja, IzmjenaObjekta, Naredba, NaredbaObjekta, NaredbaStabla, Novo, Objekt,
    Projekt, Skupina, Sloj, Stavka, StilObjekta, Uredjivac, Vrijednost, VrstaGeometrije,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Stupci po kojima objekt bez stupca `naziv` dobiva naziv, redom.
const STUPCI_NAZIVA: [&str; 5] = ["naziv", "name", "ime", "title", "oznaka"];

/// Jedan sloj iz datoteke, s kućicom u prozorčiću.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlojIzvora {
    pub id: IdSloja,
    pub naziv: String,
    pub vrsta: VrstaGeometrije,
    pub objekata: usize,
    pub sustav: &'static str,
    /// Stupci koje projekt zasad ne pamti, pa se ne kopiraju. Stupac naziva se kopira kao naziv.
    pub polja: Vec<String>,
    pub odabran: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PripremaUvoza {
    pub staza: PathBuf,
    pub naziv: String,
    pub izvor: Arc<Projekt>,
    pub slojevi: Vec<SlojIzvora>,
}

impl PripremaUvoza {
    #[must_use]
    pub fn odabranih(&self) -> usize {
        self.slojevi.iter().filter(|s| s.odabran).count()
    }

    /// Stupci odabranih slojeva koji ostaju samo u izvornoj datoteci.
    #[must_use]
    pub fn izostavljena_polja(&self) -> Vec<String> {
        let polja: BTreeSet<&String> = self
            .slojevi
            .iter()
            .filter(|s| s.odabran)
            .flat_map(|s| &s.polja)
            .collect();
        polja.into_iter().cloned().collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SazetakUvozaGpkg {
    pub mapa: IdMape,
    pub slojeva: usize,
    pub objekata: usize,
}

#[must_use]
pub const fn naziv_sustava(s: Sustav) -> &'static str {
    match s {
        Sustav::Wgs84 => "WGS84",
        Sustav::WebMercator => "Web Mercator",
        Sustav::Htrs96Tm => "HTRS96/TM",
        Sustav::HdksZona5 => "HDKS, zona 5",
        Sustav::HdksZona6 => "HDKS, zona 6",
    }
}

fn sloj_izvora(p: &Projekt, id: IdSloja, s: &Sloj) -> SlojIzvora {
    let izvor = p.zapisi().slojevi.get(&id);
    let polja: BTreeSet<String> = izvor
        .iter()
        .flat_map(|z| z.objekti.values())
        .flat_map(|o| o.nepoznati.keys().cloned())
        .filter(|stupac| !STUPCI_NAZIVA.iter().any(|n| stupac.eq_ignore_ascii_case(n)))
        .collect();
    SlojIzvora {
        id,
        naziv: s.naziv.clone(),
        vrsta: s.vrsta,
        objekata: s.objekti.len(),
        sustav: naziv_sustava(izvor.map_or(Sustav::Wgs84, |z| z.sustav)),
        polja: polja.into_iter().collect(),
        odabran: true,
    }
}

/// Čita datoteku i priprema popis slojeva. Svi su zadano odabrani.
pub fn pripremi(staza: &Path, katalog: &KatalogPoruka) -> Result<PripremaUvoza, String> {
    let (projekt, _) =
        geopaket::procitaj_za_uvoz(staza).map_err(|_| katalog.tekst("uvoz-gpkg-greska"))?;
    let slojevi: Vec<SlojIzvora> = projekt
        .slojevi()
        .map(|(id, s)| sloj_izvora(&projekt, id, s))
        .collect();
    if slojevi.is_empty() {
        return Err(katalog.tekst("uvoz-gpkg-prazno"));
    }
    let naziv = staza
        .file_stem()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(PripremaUvoza {
        staza: staza.to_path_buf(),
        naziv,
        izvor: Arc::new(projekt),
        slojevi,
    })
}

/// Naziv objekta, a bez njega tekst iz stupca kao `name` ili `ime`, npr. iz QGIS-a.
fn naziv_objekta(p: &Projekt, sloj: IdSloja, o: &Objekt) -> String {
    if !o.naziv.trim().is_empty() {
        return o.naziv.clone();
    }
    p.zapisi()
        .slojevi
        .get(&sloj)
        .and_then(|z| z.objekti.get(&o.id))
        .and_then(|z| {
            STUPCI_NAZIVA.iter().find_map(|trazeni| {
                z.nepoznati.iter().find_map(|(stupac, v)| match v {
                    Vrijednost::Tekst(t) if stupac.eq_ignore_ascii_case(trazeni) => Some(t.clone()),
                    _ => None,
                })
            })
        })
        .unwrap_or_default()
}

fn broj(n: Option<Novo>) -> Option<u64> {
    match n {
        Some(Novo::Mapa(id)) => Some(id.0),
        Some(Novo::Sloj(id)) => Some(id.0),
        Some(Novo::Objekt(id)) => Some(id.0),
        None => None,
    }
}

/// Dodavanje u novu mapu i novi sloj ne može pasti, jer nisu zaključani i geometrija je već
/// provjerena pri čitanju. Ako ipak padne, objekt se preskače, a sažetak to pokazuje.
fn dodaj(g: &mut Skupina<'_>, n: impl Into<Naredba>) -> Option<u64> {
    g.izvrsi(n).ok().and_then(broj)
}

/// Uvoz u nekoliko koraka unutar jedne skupine poništavanja.
struct Kopiranje<'a, 'b> {
    g: &'a mut Skupina<'b>,
    izvor: &'a Projekt,
    odabrani: BTreeSet<IdSloja>,
    sazetak: SazetakUvozaGpkg,
}

impl Kopiranje<'_, '_> {
    /// Ima li stavka u sebi ijedan odabrani sloj.
    fn ima_odabrani(&self, s: Stavka) -> bool {
        match s {
            Stavka::Sloj(id) => self.odabrani.contains(&id),
            Stavka::Mapa(m) => self
                .izvor
                .mapa(m)
                .is_some_and(|m| m.djeca.iter().any(|d| self.ima_odabrani(*d))),
        }
    }

    fn stavka(&mut self, s: Stavka, u: IdMape) {
        if self.ima_odabrani(s) {
            match s {
                Stavka::Sloj(id) => self.sloj(id, u),
                Stavka::Mapa(m) => self.mapa(m, u),
            }
        }
    }

    fn mapa(&mut self, m: IdMape, u: IdMape) {
        let izvor = self.izvor;
        if let Some(mapa) = izvor.mapa(m)
            && let Some(nova) = dodaj(
                self.g,
                NaredbaStabla::DodajMapu {
                    u: Some(u),
                    naziv: mapa.naziv.clone(),
                },
            )
        {
            mapa.djeca
                .iter()
                .for_each(|d| self.stavka(*d, IdMape(nova)));
        }
    }

    fn sloj(&mut self, id: IdSloja, u: IdMape) {
        let izvor = self.izvor;
        if let Some(s) = izvor.sloj(id)
            && let Some(novi) = dodaj(
                self.g,
                NaredbaStabla::DodajSloj {
                    u: Some(u),
                    naziv: s.naziv.clone(),
                    vrsta: s.vrsta,
                },
            )
        {
            let novi = IdSloja(novi);
            let _ = self.g.izvrsi(NaredbaStabla::PostaviStil {
                sloj: novi,
                stil: s.stil,
            });
            self.sazetak.slojeva += 1;
            for o in s.objekti.values() {
                self.objekt(novi, o, naziv_objekta(izvor, id, o));
            }
        }
    }

    fn objekt(&mut self, sloj: IdSloja, o: &Objekt, naziv: String) {
        let geometrija = o.geometrija.clone();
        if let Some(objekt) = dodaj(
            self.g,
            NaredbaObjekta::Dodaj {
                sloj,
                naziv,
                geometrija,
            },
        ) {
            let objekt = slojevi::IdObjekta(objekt);
            let opis = (!o.opis.is_empty()).then(|| IzmjenaObjekta::Opis(o.opis.clone()));
            let stil = (o.stil != StilObjekta::default()).then_some(IzmjenaObjekta::Stil(o.stil));
            for izmjena in opis.into_iter().chain(stil) {
                let _ = self.g.izvrsi(NaredbaObjekta::Izmijeni {
                    sloj,
                    objekt,
                    izmjena,
                });
            }
            self.sazetak.objekata += 1;
        }
    }
}

/// Kopira odabrane slojeve u novu mapu na vrhu stabla, kao jedan korak poništavanja.
pub fn uvezi(u: &mut Uredjivac, p: &PripremaUvoza) -> Result<SazetakUvozaGpkg, Greska> {
    let odabrani: BTreeSet<IdSloja> = p
        .slojevi
        .iter()
        .filter(|s| s.odabran)
        .map(|s| s.id)
        .collect();
    let izvor = p.izvor.as_ref();
    u.skupina(|g| {
        let mapa = g.izvrsi(NaredbaStabla::DodajMapu {
            u: None,
            naziv: p.naziv.clone(),
        });
        mapa.map(|n| {
            let mapa = IdMape(broj(n).unwrap_or_default());
            let mut k = Kopiranje {
                g,
                izvor,
                odabrani,
                sazetak: SazetakUvozaGpkg {
                    mapa,
                    slojeva: 0,
                    objekata: 0,
                },
            };
            izvor.korijen().iter().for_each(|s| k.stavka(*s, mapa));
            k.sazetak
        })
    })
}

#[cfg(test)]
#[path = "uvoz_gpkg_testovi.rs"]
mod testovi;
