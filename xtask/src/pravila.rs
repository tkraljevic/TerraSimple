use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Funkcija {
    pub kljuc: String,
    pub cc: u32,
    pub pokrivenost: f64,
    pub rizik: f64,
    pub kriticna: bool,
    pub podrucja: u64,
    pub pokrivena: u64,
}

impl Funkcija {
    pub fn nova(kljuc: &str, cc: u32, pokrivenost: f64, kriticna: bool) -> Self {
        Self {
            kljuc: kljuc.into(),
            cc,
            pokrivenost,
            rizik: rizik(cc, pokrivenost),
            kriticna,
            podrucja: 0,
            pokrivena: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mjerenje {
    pub cilj: String,
    pub metoda: String,
    pub pokrivenost: f64,
    pub funkcije: Vec<Funkcija>,
}

impl Mjerenje {
    pub fn novo(cilj: &str, funkcije: Vec<Funkcija>) -> Self {
        Self {
            cilj: cilj.into(),
            metoda: "syn-cc-v1/llvm-regions-v1".into(),
            pokrivenost: 1.0,
            funkcije,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Referenca {
    pub schema: u32,
    pub mjerenja: BTreeMap<String, Mjerenje>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Iznimka {
    pub kljuc: String,
    pub razlog: String,
    pub odgovorna_osoba: String,
    pub rok: chrono::NaiveDate,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Postavke {
    pub iznimke: Vec<Iznimka>,
}

#[derive(Debug)]
pub struct Nalaz {
    pub kljuc: String,
    pub poruka: String,
}

pub fn rizik(cc: u32, pokrivenost: f64) -> f64 {
    f64::from(cc).powi(2) * (1.0 - pokrivenost).powi(3) + f64::from(cc)
}

fn nalaz(kljuc: String, vrsta: &str, mjesto: &str, sada: f64, granica: f64) -> Nalaz {
    Nalaz {
        kljuc,
        poruka: crate::poruke::tekst(
            "granica",
            &[
                ("vrsta", vrsta.into()),
                ("mjesto", mjesto.into()),
                ("sada", crate::poruke::broj(sada, 4)),
                ("granica", crate::poruke::broj(granica, 4)),
            ],
        ),
    }
}

fn gornja(nalazi: &mut Vec<Nalaz>, vrsta: &str, f: &Funkcija, sada: f64, granica: f64) {
    if sada > granica + 1e-9 {
        nalazi.push(nalaz(
            format!("{vrsta}:{}", f.kljuc),
            vrsta,
            &f.kljuc,
            sada,
            granica,
        ));
    }
}

fn provjeri_funkciju(nalazi: &mut Vec<Nalaz>, f: &Funkcija, staro: Option<&Funkcija>) {
    let granica = if f.kriticna { 7.0 } else { 10.0 };
    let stara = staro.filter(|_| !f.kriticna);
    let cc = stara.map_or(granica, |s| f64::from(s.cc));
    let crap = stara.map_or(granica, |s| s.rizik + 0.5);
    gornja(nalazi, "slozenost", f, f64::from(f.cc), cc);
    gornja(nalazi, "rizik", f, f.rizik, crap);
    if f.kriticna && f.pokrivenost < 0.95 - 1e-9 {
        nalazi.push(nalaz(
            format!("pokrivenost:{}", f.kljuc),
            "pokrivenost",
            &f.kljuc,
            f.pokrivenost * 100.0,
            95.0,
        ));
    }
}

pub fn usporedi(novo: &Mjerenje, staro: Option<&Mjerenje>, postavke: &Postavke) -> Vec<Nalaz> {
    let mut nalazi = Vec::new();
    if let Some(staro) = staro
        && novo.pokrivenost < staro.pokrivenost - 0.001 - 1e-9
    {
        nalazi.push(nalaz(
            "pad-pokrivenosti:ukupno".into(),
            "pad-pokrivenosti",
            &novo.cilj,
            (staro.pokrivenost - novo.pokrivenost) * 100.0,
            0.1,
        ));
    }
    let cilj = staro.map_or(0.85, |s| (s.pokrivenost - 0.001).max(0.85));
    if novo.pokrivenost < cilj - 1e-9 {
        nalazi.push(nalaz(
            "pokrivenost:ukupno".into(),
            "pokrivenost",
            &novo.cilj,
            novo.pokrivenost * 100.0,
            cilj * 100.0,
        ));
    }
    for f in &novo.funkcije {
        let s = staro.and_then(|m| m.funkcije.iter().find(|s| s.kljuc == f.kljuc));
        provjeri_funkciju(&mut nalazi, f, s);
    }
    nalazi.retain(|n| !postavke.iznimke.iter().any(|i| i.kljuc == n.kljuc));
    nalazi
}

pub fn provjeri_iznimke(postavke: &Postavke, danas: chrono::NaiveDate) -> Result<(), String> {
    let mut kljucevi = std::collections::BTreeSet::new();
    for i in &postavke.iznimke {
        if i.kljuc.trim().is_empty()
            || i.razlog.trim().is_empty()
            || i.odgovorna_osoba.trim().is_empty()
            || !kljucevi.insert(&i.kljuc)
        {
            return Err(crate::poruke::tekst(
                "nevaljana-iznimka",
                &[("kljuc", i.kljuc.clone())],
            ));
        }
        if i.rok < danas {
            return Err(crate::poruke::tekst(
                "istekla-iznimka",
                &[("kljuc", i.kljuc.clone()), ("rok", i.rok.to_string())],
            ));
        }
    }
    Ok(())
}
