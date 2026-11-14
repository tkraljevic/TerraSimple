//! Model mapa, slojeva i objekata te naredbe kojima se mijenjaju.
//!
//! Sve izmjene idu kroz [`Uredjivac`], pa se svaka može poništiti, a isti put koristit će i MCP
//! (Odluke (7. 11. 2026.)). Paket ne zna ništa o sučelju ni datotekama.

#![forbid(unsafe_code)]

mod greska;
mod model;
mod naredba;
mod promjena;
mod stil;
mod uredjivac;

pub use greska::Greska;
pub use model::{
    Geometrija, IdMape, IdObjekta, IdSloja, Mapa, Mjere, Objekt, Projekt, Sloj, StanjeSloja,
    Stavka, Vrh, VrstaGeometrije,
};
pub use naredba::{IzmjenaObjekta, Naredba, NaredbaObjekta, NaredbaStabla, Novo};
pub use stil::{Boja, Natpis, OblikZnaka, Stil, StilObjekta, VrstaCrte, VrstaIspune};
pub use uredjivac::{Skupina, Uredjivac};

mod zapis;
pub use zapis::{IzmjeneSpremanja, IzvorSloja, Vrijednost, ZapisObjekta, Zapisi};
