//! Geometrija i mjere. Paket ne zna ništa o sučelju, slojevima ni datotekama.

#![forbid(unsafe_code)]

mod mjere;
mod presjek;

pub use mjere::{
    Greska, Metoda, MjereCrte, MjereLika, Tocka, mjeri_crtu, mjeri_lik, provjeri_tocku,
};
pub use presjek::presjek_lika;
