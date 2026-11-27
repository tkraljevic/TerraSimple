use crate::Inacica;
use serde::Deserialize;

/// Sustav za koji se traži paket. Imena paketa daje `skripte/izdanje.py`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sustav {
    MacOs,
    Windows,
    Linux,
}

impl Sustav {
    /// Sustav na kojem program radi.
    #[must_use]
    pub const fn trenutni() -> Self {
        TRENUTNI
    }

    const fn nastavak(self) -> &'static str {
        match self {
            Self::MacOs => "macos-universal.zip",
            Self::Windows => "windows-x86_64.zip",
            Self::Linux => "linux-x86_64.tar.gz",
        }
    }

    /// Ime paketa izdanja, npr. `TerraSimple-0.0.8-macos-universal.zip`.
    #[must_use]
    pub fn ime_paketa(self, inacica: Inacica) -> String {
        format!("TerraSimple-{inacica}-{}", self.nastavak())
    }
}

#[cfg(target_os = "macos")]
const TRENUTNI: Sustav = Sustav::MacOs;
#[cfg(windows)]
const TRENUTNI: Sustav = Sustav::Windows;
#[cfg(not(any(target_os = "macos", windows)))]
const TRENUTNI: Sustav = Sustav::Linux;

/// Datoteka priložena izdanju na GitHubu.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Datoteka {
    #[serde(rename = "name")]
    pub ime: String,
    #[serde(rename = "size")]
    pub velicina: u64,
    #[serde(rename = "browser_download_url")]
    pub adresa: String,
}

/// Izdanje kako ga vraća GitHub (`/repos/…/releases`).
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Izdanje {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    html_url: String,
    #[serde(default)]
    assets: Vec<Datoteka>,
}

/// Novije izdanje s potpisanim paketom za ovaj sustav.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ponuda {
    pub inacica: Inacica,
    pub stranica: String,
    pub paket: Datoteka,
    pub potpis: Datoteka,
}

/// Što je provjera našla.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ishod {
    /// Instalirano je najnovije izdanje.
    Najnovija(Inacica),
    /// Postoji novije izdanje koje se može instalirati iz programa.
    Dostupna(Ponuda),
    /// Postoji novije izdanje, ali bez potpisanog paketa za ovaj sustav: preuzima se ručno.
    BezPaketa { inacica: Inacica, stranica: String },
}

/// Bira najnovije objavljeno izdanje. Probna izdanja se ubrajaju, jer su sva dosad alfa.
/// Paket se nudi samo ako on i njegov potpis leže pod `prefiks`.
#[must_use]
pub(crate) fn odaberi(
    izdanja: &[Izdanje],
    trenutna: Inacica,
    sustav: Sustav,
    prefiks: &str,
) -> Ishod {
    let najnovije = izdanja
        .iter()
        .filter(|i| !i.draft)
        .filter_map(|i| Some((Inacica::procitaj(&i.tag_name)?, i)))
        .max_by_key(|(v, _)| *v);
    match najnovije {
        Some((v, izdanje)) if v > trenutna => ponuda(v, izdanje, sustav, prefiks),
        _ => Ishod::Najnovija(trenutna),
    }
}

fn ponuda(inacica: Inacica, izdanje: &Izdanje, sustav: Sustav, prefiks: &str) -> Ishod {
    let ime = sustav.ime_paketa(inacica);
    let nadji = |trazeno: &str| {
        izdanje
            .assets
            .iter()
            .find(|d| d.ime == trazeno && na_stranici_izdanja(&d.adresa, prefiks))
            .cloned()
    };
    match (nadji(&ime), nadji(&format!("{ime}.sig"))) {
        (Some(paket), Some(potpis)) => Ishod::Dostupna(Ponuda {
            inacica,
            stranica: izdanje.html_url.clone(),
            paket,
            potpis,
        }),
        _ => Ishod::BezPaketa {
            inacica,
            stranica: izdanje.html_url.clone(),
        },
    }
}

pub(crate) fn na_stranici_izdanja(adresa: &str, prefiks: &str) -> bool {
    adresa.starts_with(prefiks) && !adresa.contains("..")
}

#[cfg(test)]
#[path = "izdanja_testovi.rs"]
mod testovi;
