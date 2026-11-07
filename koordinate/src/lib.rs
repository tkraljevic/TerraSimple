//! Koordinatni sustavi koje program poznaje i pretvorba među njima.
//!
//! Program zna samo sustave s ovog popisa. Nepoznat sustav nije greška koja se prešuti, nego
//! `None`, pa korisnik mora reći što je u datoteci. Tako je odlučeno nakon probe koordinata
//! (7. 11. 2026.): `proj4wkt` iz definicije u datoteci tiho griješi 410 m za HDKS i 21 km za
//! Web Mercator.
#![forbid(unsafe_code)]

mod pretvorba;
mod sustav;

pub use pretvorba::{Greska, Pretvornik};
pub use sustav::Sustav;
