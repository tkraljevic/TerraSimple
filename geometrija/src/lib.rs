//! Geometrija i mjere. Paket ne zna ništa o sučelju, slojevima ni datotekama.

#![forbid(unsafe_code)]

mod mjere;

pub use mjere::{Greska, Metoda, MjereCrte, MjereLika, Tocka, mjeri_crtu, mjeri_lik};
