//! Karta i crtanje mišem. Ekran je u Web Mercatoru, a mjere i koordinate u WGS84.
//!
//! Logika crtanja ne zna za grafičku karticu, pa se testira pravim događajima miša. Tekstove koje
//! korisnik čita daje `prozor`.

#![forbid(unsafe_code)]

mod crtac;
mod crtanje;
mod crtez_na_karti;
mod kamera;
mod karta;
mod mercator;
mod odabir;
mod podloga;
mod priprema;

pub use crtac::{provjera, ugradi};
pub use crtanje::{Alat, Crtanje, Mjerenje, Okolina, Poruka, Pretvorba};
pub use karta::{Karta, Pogled};
pub use podloga::{ListDgu, Podloga};
