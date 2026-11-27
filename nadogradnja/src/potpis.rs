use crate::Greska;
use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::{ED25519, Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// Početak potpisane poruke. Mijenja se samo uz novi oblik potpisa.
const OZNAKA: &[u8] = b"TerraSimple nadogradnja 1\n";

/// Potpisuje se ime paketa zajedno sa sadržajem. U imenu je inačica, pa potpis starijeg
/// paketa ne vrijedi pod imenom novijega.
fn poruka(ime: &str, sadrzaj: &[u8]) -> Vec<u8> {
    let mut p = Vec::with_capacity(OZNAKA.len() + ime.len() + 1 + sadrzaj.len());
    p.extend_from_slice(OZNAKA);
    p.extend_from_slice(ime.as_bytes());
    p.push(b'\n');
    p.extend_from_slice(sadrzaj);
    p
}

/// Provjerava potpis iz datoteke `.sig` (base64 u jednom retku).
///
/// # Errors
/// [`Greska::KrivPotpis`] kad potpis nije base64 ili ne odgovara paketu.
pub fn provjeri_potpis(
    javni_kljuc: &[u8],
    ime: &str,
    sadrzaj: &[u8],
    tekst_potpisa: &[u8],
) -> Result<(), Greska> {
    let tekst = std::str::from_utf8(tekst_potpisa).map_err(|_| Greska::KrivPotpis)?;
    let potpis = STANDARD
        .decode(tekst.trim())
        .map_err(|_| Greska::KrivPotpis)?;
    UnparsedPublicKey::new(&ED25519, javni_kljuc)
        .verify(&poruka(ime, sadrzaj), &potpis)
        .map_err(|_| Greska::KrivPotpis)
}

/// Novi tajni ključ u obliku PKCS#8. Čuva se izvan repozitorija.
///
/// # Errors
/// Kad sustav ne da slučajne brojeve.
pub fn novi_tajni_kljuc() -> Result<Vec<u8>, Greska> {
    Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
        .map(|k| k.as_ref().to_vec())
        .map_err(crate::greska::neocekivano)
}

fn par(tajni: &[u8]) -> Result<Ed25519KeyPair, Greska> {
    Ed25519KeyPair::from_pkcs8(tajni).map_err(crate::greska::neocekivano)
}

/// Javni ključ koji se upisuje u program, kao base64.
///
/// # Errors
/// Kad datoteka nije tajni ključ Ed25519.
pub fn javni_kljuc_iz_tajnog(tajni: &[u8]) -> Result<String, Greska> {
    Ok(STANDARD.encode(par(tajni)?.public_key().as_ref()))
}

/// Potpis paketa kakav ide u datoteku `.sig`.
///
/// # Errors
/// Kad datoteka nije tajni ključ Ed25519.
pub fn potpisi(tajni: &[u8], ime: &str, sadrzaj: &[u8]) -> Result<String, Greska> {
    let potpis = par(tajni)?.sign(&poruka(ime, sadrzaj));
    Ok(STANDARD.encode(potpis.as_ref()))
}

#[cfg(test)]
#[path = "potpis_testovi.rs"]
mod testovi;
