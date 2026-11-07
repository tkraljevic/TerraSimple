//! Podjela KML stabla mapa na slojeve s jednom vrstom geometrije (točke, linije, poligoni).
//!
//! Prema odluci od 6. 11. 2026., svaki sloj u TerraSimpleu ima jednu vrstu geometrije.
//! Mapa koja sadrži različite vrste objekata ili objekt s miješanom `MultiGeometry`
//! dijeli se na najviše tri sloja (točke, linije, poligoni) unutar iste mape.

use crate::model::{Geometrija, Mapa, StabloKml, VrstaMape};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VrstaSloja {
    Tocke,
    Linije,
    Poligoni,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sloj {
    pub dubina: usize,
    pub vrsta: VrstaSloja,
    pub broj_objekata: usize,
    pub broj_geometrija: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IzvjestajSlojeva {
    pub slojevi: Vec<Sloj>,
    pub ukupno_spremnika: usize,
    pub nepraznih_spremnika: usize,
    pub mijesanih_spremnika: usize,
    pub mijesanih_visegeometrija: usize,
    pub slojeva_tocaka: usize,
    pub slojeva_linija: usize,
    pub slojeva_poligona: usize,
    pub ukupno_slojeva: usize,
    pub objekata_u_slojevima_tocaka: usize,
    pub objekata_u_slojevima_linija: usize,
    pub objekata_u_slojevima_poligona: usize,
}

#[derive(Default)]
struct BrojacVrsta {
    tocke: usize,
    linije: usize,
    poligoni: usize,
}

impl BrojacVrsta {
    fn broj_razlicitih_vrsta(&self) -> usize {
        usize::from(self.tocke > 0) + usize::from(self.linije > 0) + usize::from(self.poligoni > 0)
    }
}

fn razvrstaj_geometriju(geom: &Geometrija, b: &mut BrojacVrsta) {
    match geom {
        Geometrija::Tocka { .. } => b.tocke += 1,
        Geometrija::Linija { vrhovi, .. } | Geometrija::Trag { vrhovi, .. } => {
            if !vrhovi.is_empty() {
                b.linije += 1;
            }
        }
        Geometrija::Poligon { .. } => b.poligoni += 1,
        Geometrija::ViseGeometrija(pod) => {
            for g in pod {
                razvrstaj_geometriju(g, b);
            }
        }
    }
}

fn obradi_mapu(mapa: &Mapa, dubina: usize, izvjestaj: &mut IzvjestajSlojeva) {
    let nova_dubina = if mapa.vrsta == VrstaMape::Korijen {
        dubina
    } else {
        izvjestaj.ukupno_spremnika += 1;
        dubina + 1
    };

    let (mut obj_tocke, mut geom_tocke) = (0usize, 0usize);
    let (mut obj_linije, mut geom_linije) = (0usize, 0usize);
    let (mut obj_poligoni, mut geom_poligoni) = (0usize, 0usize);

    for obj in &mapa.objekti {
        let Some(ref geom) = obj.geometrija else {
            continue;
        };
        let mut b = BrojacVrsta::default();
        razvrstaj_geometriju(geom, &mut b);
        if b.broj_razlicitih_vrsta() > 1 {
            izvjestaj.mijesanih_visegeometrija += 1;
        }
        if b.tocke > 0 {
            obj_tocke += 1;
            geom_tocke += b.tocke;
        }
        if b.linije > 0 {
            obj_linije += 1;
            geom_linije += b.linije;
        }
        if b.poligoni > 0 {
            obj_poligoni += 1;
            geom_poligoni += b.poligoni;
        }
    }

    let nastalo_u_mapi =
        usize::from(obj_tocke > 0) + usize::from(obj_linije > 0) + usize::from(obj_poligoni > 0);
    if nastalo_u_mapi > 0 {
        izvjestaj.nepraznih_spremnika += 1;
        if nastalo_u_mapi > 1 {
            izvjestaj.mijesanih_spremnika += 1;
        }
    }

    if obj_tocke > 0 {
        izvjestaj.slojeva_tocaka += 1;
        izvjestaj.objekata_u_slojevima_tocaka += obj_tocke;
        izvjestaj.slojevi.push(Sloj {
            dubina: nova_dubina,
            vrsta: VrstaSloja::Tocke,
            broj_objekata: obj_tocke,
            broj_geometrija: geom_tocke,
        });
    }
    if obj_linije > 0 {
        izvjestaj.slojeva_linija += 1;
        izvjestaj.objekata_u_slojevima_linija += obj_linije;
        izvjestaj.slojevi.push(Sloj {
            dubina: nova_dubina,
            vrsta: VrstaSloja::Linije,
            broj_objekata: obj_linije,
            broj_geometrija: geom_linije,
        });
    }
    if obj_poligoni > 0 {
        izvjestaj.slojeva_poligona += 1;
        izvjestaj.objekata_u_slojevima_poligona += obj_poligoni;
        izvjestaj.slojevi.push(Sloj {
            dubina: nova_dubina,
            vrsta: VrstaSloja::Poligoni,
            broj_objekata: obj_poligoni,
            broj_geometrija: geom_poligoni,
        });
    }

    izvjestaj.ukupno_slojeva =
        izvjestaj.slojeva_tocaka + izvjestaj.slojeva_linija + izvjestaj.slojeva_poligona;

    for pod in &mapa.podmape {
        obradi_mapu(pod, nova_dubina, izvjestaj);
    }
}

pub fn podijeli_na_slojeve(stablo: &StabloKml) -> IzvjestajSlojeva {
    let mut izvjestaj = IzvjestajSlojeva::default();
    obradi_mapu(&stablo.korijen, 0, &mut izvjestaj);
    izvjestaj
}
