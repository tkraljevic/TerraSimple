//! Nadogradnja programa iz izbornika Pomoć (Odluke (27. 11. 2026.)).
//!
//! Program pita GitHub za izdanja samo kad korisnik to zatraži. Paket instalira tek kad
//! potpis odgovara našem javnom ključu, a tajni ključ nikad ne ide na GitHub. Ime paketa
//! je dio potpisane poruke, pa se stariji paket ne može podmetnuti kao noviji.

#![forbid(unsafe_code)]

mod greska;
mod inacica;
mod izdanja;
mod mreza;
mod paket;
#[cfg(test)]
mod pomoc_testova;
mod potpis;
mod ugradnja;

pub use greska::Greska;
pub use inacica::Inacica;
pub use izdanja::{Datoteka, Ishod, Ponuda, Sustav};
pub use mreza::Izvor;
pub use paket::raspakiraj;
pub use potpis::{javni_kljuc_iz_tajnog, novi_tajni_kljuc, potpisi, provjeri_potpis};
pub use ugradnja::{Mjesto, naredba_ponovnog_pokretanja, pocisti_ostatke, ugradi};

/// Pita izvor za izdanja i kaže ima li novijega od `trenutna` za ovaj sustav.
///
/// # Errors
/// Kad se izvor ne može dohvatiti ili odgovor nije popis izdanja.
pub fn provjeri(izvor: &Izvor, trenutna: Inacica, sustav: Sustav) -> Result<Ishod, Greska> {
    let izdanja = izvor.izdanja()?;
    Ok(izdanja::odaberi(&izdanja, trenutna, sustav, &izvor.prefiks))
}

/// Preuzima ponuđeni paket, provjerava potpis, raspakirava ga i zamjenjuje program na `mjesto`.
///
/// # Errors
/// Kad preuzimanje, potpis, raspakiravanje ili zamjena ne uspiju. Program na disku tada
/// ostaje kakav je bio.
pub fn preuzmi_i_ugradi(izvor: &Izvor, ponuda: &Ponuda, mjesto: &Mjesto) -> Result<(), Greska> {
    let potpis = izvor.preuzmi(&ponuda.potpis, mreza::NAJVECI_POTPIS)?;
    let sadrzaj = izvor.preuzmi(&ponuda.paket, mreza::NAJVECI_PAKET)?;
    provjeri_potpis(&izvor.javni_kljuc, &ponuda.paket.ime, &sadrzaj, &potpis)?;
    ugradi(&sadrzaj, &ponuda.paket.ime, mjesto)
}

#[cfg(test)]
#[path = "lib_testovi.rs"]
mod testovi;
