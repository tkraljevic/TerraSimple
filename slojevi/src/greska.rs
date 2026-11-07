//! Zašto naredba nije izvršena.

use crate::model::VrstaGeometrije;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Greska {
    NepoznataMapa,
    NepoznatSloj,
    NepoznatObjekt,
    /// Mapa i sloj moraju imati naziv.
    PrazanNaziv,
    /// Sloj je samo za čitanje ili u sukobu, pa se njegovi objekti ne smiju mijenjati.
    SlojZakljucan,
    /// Sloj ima jednu vrstu geometrije (Odluke (6. 11. 2026.)).
    KrivaVrsta {
        ocekivana: VrstaGeometrije,
        dobivena: VrstaGeometrije,
    },
    /// Linija treba dva vrha, a svaki prsten poligona tri različita.
    PremaloVrhova,
    Koordinata(geometrija::Greska),
    /// Mapa se ne može premjestiti u samu sebe ni u svoju podmapu.
    MapaUSebe,
}

impl std::fmt::Display for Greska {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NepoznataMapa => f.write_str("mapa ne postoji"),
            Self::NepoznatSloj => f.write_str("sloj ne postoji"),
            Self::NepoznatObjekt => f.write_str("objekt ne postoji"),
            Self::PrazanNaziv => f.write_str("naziv je prazan"),
            Self::SlojZakljucan => f.write_str("sloj se ne smije mijenjati"),
            Self::KrivaVrsta {
                ocekivana,
                dobivena,
            } => write!(f, "sloj ima vrstu {ocekivana:?}, a objekt {dobivena:?}"),
            Self::PremaloVrhova => f.write_str("objekt ima premalo vrhova"),
            Self::Koordinata(g) => g.fmt(f),
            Self::MapaUSebe => f.write_str("mapa se ne može premjestiti u samu sebe"),
        }
    }
}

impl std::error::Error for Greska {}

impl From<geometrija::Greska> for Greska {
    fn from(g: geometrija::Greska) -> Self {
        Self::Koordinata(g)
    }
}
