//! Izvoz u KML ili KMZ za Google Earth (zaslon K). Prije izvoza prozorčić kaže što ide u datoteku i
//! što se usput mijenja ili gubi, jer KML ne zna sve što zna projekt.

use crate::poruke::KatalogPoruka;
use formati::{OpsegIzvoza, PregledIzvoza};
use slojevi::{Projekt, Stavka};
use std::path::Path;

/// Kako stavka pregleda prolazi u KML.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Prolaz {
    /// Ide u datoteku kakvo jest.
    Da,
    /// Ide, ali se mijenja.
    Pazi,
    /// Ne ide u datoteku.
    Ne,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StavkaPregleda {
    pub prolaz: Prolaz,
    pub naslov: String,
    pub opis: String,
}

/// Prozorčić izvoza: što se izvozi, pod kojim nazivom i što pregled kaže.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PripremaIzvoza {
    pub opseg: OpsegIzvoza,
    pub naziv: String,
    pub stavke: Vec<StavkaPregleda>,
    /// Bez ijednog objekta nema što izvesti.
    pub moze: bool,
}

fn stavka(katalog: &KatalogPoruka, prolaz: Prolaz, kljuc: &str, opis: String) -> StavkaPregleda {
    StavkaPregleda {
        prolaz,
        naslov: katalog.tekst(kljuc),
        opis,
    }
}

fn gubici(katalog: &KatalogPoruka, p: &PregledIzvoza) -> Vec<StavkaPregleda> {
    let pazi = |kljuc: &str| {
        stavka(
            katalog,
            Prolaz::Pazi,
            kljuc,
            katalog.tekst(&format!("{kljuc}-opis")),
        )
    };
    let mut s = Vec::new();
    if p.drugi_sustav {
        s.push(pazi("izvoz-sustav"));
    }
    if p.isprekidane > 0 {
        s.push(pazi("izvoz-isprekidane"));
    }
    if p.srafure > 0 {
        s.push(pazi("izvoz-srafura"));
    }
    if p.natpisi_bez_tocaka > 0 {
        s.push(pazi("izvoz-natpisi"));
    }
    if p.kurziv > 0 {
        s.push(pazi("izvoz-kurziv"));
    }
    if p.izostavljene_vrijednosti > 0 {
        let broj = p.izostavljene_vrijednosti.to_string();
        let opis = katalog.s_tekstovima("izvoz-izostavljeno-opis", &[("broj", broj)]);
        s.push(stavka(katalog, Prolaz::Ne, "izvoz-izostavljeno", opis));
    }
    s
}

/// Stavke pregleda redom kao u nacrtu: prvo što ide, pa što se mijenja, pa što ne ide.
#[must_use]
pub fn stavke_pregleda(katalog: &KatalogPoruka, p: &PregledIzvoza) -> Vec<StavkaPregleda> {
    if p.objekata() == 0 {
        let opis = katalog.tekst("izvoz-prazno-opis");
        return vec![stavka(katalog, Prolaz::Ne, "izvoz-prazno", opis)];
    }
    let brojevi = [
        ("objekti", katalog.broj_objekata(p.objekata())),
        ("mape", katalog.broj_mapa(p.mapa)),
    ];
    let opis = katalog.s_tekstovima("izvoz-geometrije-opis", &brojevi);
    let mut s = vec![stavka(katalog, Prolaz::Da, "izvoz-geometrije", opis)];
    if !p.atributi.is_empty() {
        let popis = p.atributi.iter().cloned().collect::<Vec<_>>().join(", ");
        let opis = katalog.s_tekstovima("izvoz-atributi-opis", &[("popis", popis)]);
        s.push(stavka(katalog, Prolaz::Da, "izvoz-atributi", opis));
    }
    s.extend(gubici(katalog, p));
    s
}

/// Naziv dokumenta i predložene datoteke: naziv mape ili sloja, a za cijeli projekt naziv datoteke.
#[must_use]
pub fn naziv_izvoza(
    projekt: &Projekt,
    opseg: OpsegIzvoza,
    staza_projekta: Option<&Path>,
    katalog: &KatalogPoruka,
) -> String {
    let naziv = match opseg {
        OpsegIzvoza::Stavka(Stavka::Mapa(m)) => projekt.mapa(m).map(|m| m.naziv.clone()),
        OpsegIzvoza::Stavka(Stavka::Sloj(s)) => projekt.sloj(s).map(|s| s.naziv.clone()),
        OpsegIzvoza::Projekt => staza_projekta
            .and_then(Path::file_stem)
            .map(|s| s.to_string_lossy().into_owned()),
    };
    naziv
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| katalog.tekst("izvoz-zadani-naziv"))
}

#[must_use]
pub fn pripremi(
    projekt: &Projekt,
    opseg: OpsegIzvoza,
    staza_projekta: Option<&Path>,
    katalog: &KatalogPoruka,
) -> PripremaIzvoza {
    let pregled = formati::pregledaj(projekt, opseg, &crate::uvoz::nazivi(katalog));
    PripremaIzvoza {
        opseg,
        naziv: naziv_izvoza(projekt, opseg, staza_projekta, katalog),
        stavke: stavke_pregleda(katalog, &pregled),
        moze: pregled.objekata() > 0,
    }
}

/// Datoteka ide na mjesto koje je korisnik odabrao. Vraća poruku za prozorčić.
pub fn izvezi(
    projekt: &Projekt,
    priprema: &PripremaIzvoza,
    staza: &Path,
    katalog: &KatalogPoruka,
) -> Result<String, String> {
    let datoteka = staza
        .file_name()
        .map(|d| d.to_string_lossy().into_owned())
        .unwrap_or_default();
    let nazivi = crate::uvoz::nazivi(katalog);
    match formati::izvezi(projekt, priprema.opseg, &priprema.naziv, &nazivi, staza) {
        Ok(p) => Ok(katalog.s_tekstovima(
            "izvoz-gotov",
            &[
                ("datoteka", datoteka),
                ("objekti", katalog.broj_objekata(p.objekata())),
            ],
        )),
        Err(_) => Err(katalog.s_tekstovima("izvoz-greska", &[("datoteka", datoteka)])),
    }
}
