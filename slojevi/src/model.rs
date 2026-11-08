//! Što projekt sadrži: stablo mapa i slojeva, a u sloju objekte.
//!
//! Koordinate su uvijek u WGS84. Ono što se crta i mjeri radi s njima, a preračun u sustav datoteke
//! dolazi uz spremanje.

use crate::greska::Greska;
use crate::stil::{Stil, StilObjekta};
use geometrija::{MjereCrte, MjereLika, Tocka, mjeri_crtu, mjeri_lik, provjeri_tocku};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdMape(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdSloja(pub u64);

/// Redoslijed brojeva je redoslijed nastanka, pa `BTreeMap` drži objekte tim redom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdObjekta(pub u64);

/// Jedna stavka u stablu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stavka {
    Mapa(IdMape),
    Sloj(IdSloja),
}

/// Vrh u stupnjevima WGS84. Visina se čuva i kad je program ne koristi, jer je `geozero` i
/// `geo_types` tiho gube, a uređivanje 3D linije ne smije baciti visine (proba GeoPackagea).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vrh {
    pub duzina: f64,
    pub sirina: f64,
    pub visina: Option<f64>,
}

impl Vrh {
    pub fn novi(duzina: f64, sirina: f64) -> Self {
        Self {
            duzina,
            sirina,
            visina: None,
        }
    }

    fn tocka(&self) -> Tocka {
        [self.duzina, self.sirina]
    }

    fn provjeri(&self) -> Result<(), Greska> {
        provjeri_tocku(self.tocka())?;
        match self.visina {
            Some(v) if !v.is_finite() => Err(geometrija::Greska::NijeBroj.into()),
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VrstaGeometrije {
    Tocka,
    Linija,
    Poligon,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Geometrija {
    Tocka(Vrh),
    Linija(Vec<Vrh>),
    /// Prvi prsten je vanjski rub, a ostali su rupe.
    Poligon(Vec<Vec<Vrh>>),
}

/// Mjere ovise o vrsti, a točka ih nema.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mjere {
    Tocka,
    Linija(MjereCrte),
    Poligon(MjereLika),
}

fn tocke(vrhovi: &[Vrh]) -> Vec<Tocka> {
    vrhovi.iter().map(Vrh::tocka).collect()
}

/// GeoPackage i KML prsten zatvaraju ponovljenom prvom točkom, pa se ona ne broji.
fn broj_vrhova_prstena(prsten: &[Vrh]) -> usize {
    match prsten.split_last() {
        Some((zadnji, ostali)) if prsten.first() == Some(zadnji) => ostali.len(),
        _ => prsten.len(),
    }
}

fn provjeri_prsten(prsten: &[Vrh]) -> Result<(), Greska> {
    if broj_vrhova_prstena(prsten) < 3 {
        return Err(Greska::PremaloVrhova);
    }
    prsten.iter().try_for_each(Vrh::provjeri)
}

fn provjeri_liniju(vrhovi: &[Vrh]) -> Result<(), Greska> {
    if vrhovi.len() < 2 {
        return Err(Greska::PremaloVrhova);
    }
    vrhovi.iter().try_for_each(Vrh::provjeri)
}

impl Geometrija {
    pub fn vrsta(&self) -> VrstaGeometrije {
        match self {
            Self::Tocka(_) => VrstaGeometrije::Tocka,
            Self::Linija(_) => VrstaGeometrije::Linija,
            Self::Poligon(_) => VrstaGeometrije::Poligon,
        }
    }

    /// Nedovršeni crtež živi u sučelju, a u model ulazi tek gotov objekt.
    pub(crate) fn provjeri(&self) -> Result<(), Greska> {
        match self {
            Self::Tocka(v) => v.provjeri(),
            Self::Linija(vrhovi) => provjeri_liniju(vrhovi),
            Self::Poligon(prstenovi) if prstenovi.is_empty() => Err(Greska::PremaloVrhova),
            Self::Poligon(prstenovi) => prstenovi.iter().try_for_each(|p| provjeri_prsten(p)),
        }
    }

    /// Duljina, opseg i površina na elipsoidu, s metodom.
    pub fn mjere(&self) -> Result<Mjere, Greska> {
        Ok(match self {
            Self::Tocka(_) => Mjere::Tocka,
            Self::Linija(v) => Mjere::Linija(mjeri_crtu(&tocke(v))?),
            Self::Poligon(p) => {
                let prstenovi: Vec<Vec<Tocka>> = p.iter().map(|r| tocke(r)).collect();
                Mjere::Poligon(mjeri_lik(&prstenovi)?)
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Objekt {
    pub id: IdObjekta,
    pub naziv: String,
    pub opis: String,
    pub geometrija: Geometrija,
    pub stil: StilObjekta,
}

/// Odakle sloj dolazi i smije li se mijenjati (docs/gocop-plugin.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StanjeSloja {
    Uredivo,
    SamoCitanje,
    /// Izmjene su spremljene ovdje, ali izvor ih još nema.
    CekaSlanje,
    /// Izvor se promijenio mimo nas. Dok se to ne riješi, izmjene bi samo povećale sukob.
    USukobu,
}

impl StanjeSloja {
    pub fn smije_mijenjati(self) -> bool {
        matches!(self, Self::Uredivo | Self::CekaSlanje)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Mapa {
    pub naziv: String,
    pub roditelj: Option<IdMape>,
    pub djeca: Vec<Stavka>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sloj {
    pub naziv: String,
    pub roditelj: Option<IdMape>,
    pub vrsta: VrstaGeometrije,
    pub stanje: StanjeSloja,
    pub stil: Stil,
    pub objekti: BTreeMap<IdObjekta, Objekt>,
}

/// Cijeli sadržaj projekta. Mijenja se samo kroz `Uredjivac`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Projekt {
    pub(crate) zapisi: crate::zapis::Zapisi,
    pub(crate) korijen: Vec<Stavka>,
    pub(crate) mape: BTreeMap<IdMape, Mapa>,
    pub(crate) slojevi: BTreeMap<IdSloja, Sloj>,
}

impl Projekt {
    /// Stavke na vrhu stabla, redom kojim se prikazuju.
    pub fn korijen(&self) -> &[Stavka] {
        &self.korijen
    }

    pub fn mapa(&self, id: IdMape) -> Option<&Mapa> {
        self.mape.get(&id)
    }

    pub fn sloj(&self, id: IdSloja) -> Option<&Sloj> {
        self.slojevi.get(&id)
    }

    pub fn objekt(&self, sloj: IdSloja, id: IdObjekta) -> Option<&Objekt> {
        self.sloj(sloj)?.objekti.get(&id)
    }

    /// Svi slojevi, redom nastanka. Redoslijed crtanja daje stablo.
    pub fn slojevi(&self) -> impl Iterator<Item = (IdSloja, &Sloj)> {
        self.slojevi.iter().map(|(id, s)| (*id, s))
    }
}

#[cfg(test)]
mod testovi {
    use super::*;

    fn vrhovi(n: usize) -> Vec<Vrh> {
        (0..n)
            .map(|i| Vrh::novi(18.0 + i as f64 * 0.01, 45.0 + (i % 2) as f64 * 0.01))
            .collect()
    }

    #[test]
    fn linija_treba_dva_vrha() {
        assert_eq!(
            Geometrija::Linija(vrhovi(1)).provjeri(),
            Err(Greska::PremaloVrhova)
        );
        assert_eq!(Geometrija::Linija(vrhovi(2)).provjeri(), Ok(()));
    }

    #[test]
    fn prsten_treba_tri_razlicita_vrha_a_zatvaranje_se_ne_broji() {
        let mut dva_zatvorena = vrhovi(2);
        dva_zatvorena.push(dva_zatvorena[0]);
        let los = Geometrija::Poligon(vec![dva_zatvorena]);
        assert_eq!(los.provjeri(), Err(Greska::PremaloVrhova));
        assert_eq!(
            Geometrija::Poligon(vec![]).provjeri(),
            Err(Greska::PremaloVrhova)
        );
        let mut zatvoren = vrhovi(3);
        zatvoren.push(zatvoren[0]);
        assert_eq!(Geometrija::Poligon(vec![zatvoren]).provjeri(), Ok(()));
        assert_eq!(
            Geometrija::Poligon(vec![vrhovi(3), vrhovi(2)]).provjeri(),
            Err(Greska::PremaloVrhova),
            "i rupa je prsten"
        );
    }

    #[test]
    fn krive_koordinate_i_visina_se_odbijaju_u_svakoj_vrsti() {
        let u_metrima = Vrh::novi(655_120.0, 5_046_380.0);
        let losa_visina = Vrh {
            visina: Some(f64::NAN),
            ..Vrh::novi(18.0, 45.0)
        };
        let metri = Greska::Koordinata(geometrija::Greska::IzvanRaspona);
        let nije_broj = Greska::Koordinata(geometrija::Greska::NijeBroj);
        assert_eq!(Geometrija::Tocka(u_metrima).provjeri(), Err(metri));
        assert_eq!(Geometrija::Tocka(losa_visina).provjeri(), Err(nije_broj));
        let mut crta = vrhovi(2);
        crta[1] = u_metrima;
        assert_eq!(Geometrija::Linija(crta).provjeri(), Err(metri));
        let mut prsten = vrhovi(3);
        prsten[2] = losa_visina;
        assert_eq!(Geometrija::Poligon(vec![prsten]).provjeri(), Err(nije_broj));
    }

    #[test]
    fn mjere_dolaze_iz_paketa_geometrija() {
        let v = vrhovi(3);
        assert_eq!(Geometrija::Tocka(v[0]).mjere(), Ok(Mjere::Tocka));
        let crta = Geometrija::Linija(v.clone()).mjere().expect("mjere");
        let lik = Geometrija::Poligon(vec![v.clone()]).mjere().expect("mjere");
        let t = tocke(&v);
        assert_eq!(crta, Mjere::Linija(mjeri_crtu(&t).expect("crta")));
        assert_eq!(lik, Mjere::Poligon(mjeri_lik(&[t]).expect("lik")));
        assert!(
            Geometrija::Linija(vec![Vrh::novi(f64::NAN, 45.0)])
                .mjere()
                .is_err()
        );
    }

    #[test]
    fn samo_uredivo_i_ceka_slanje_smiju_mijenjati() {
        assert!(StanjeSloja::Uredivo.smije_mijenjati());
        assert!(StanjeSloja::CekaSlanje.smije_mijenjati());
        assert!(!StanjeSloja::SamoCitanje.smije_mijenjati());
        assert!(!StanjeSloja::USukobu.smije_mijenjati());
    }
}
