use crate::{
    alati::{greska_datoteke, pokreni},
    poruke::tekst,
    pravila::{Funkcija, Mjerenje},
    slozenost::{Opis, analiziraj},
};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
pub struct Metapodaci {
    pub workspace_members: Vec<String>,
    pub packages: Vec<Paket>,
}
#[derive(Deserialize)]
pub struct Paket {
    pub id: String,
    pub name: String,
    pub manifest_path: PathBuf,
    pub targets: Vec<Cilj>,
}
#[derive(Deserialize)]
pub struct Cilj {
    pub kind: Vec<String>,
    pub src_path: PathBuf,
}

pub fn metapodaci(mapa: &Path, cargo: &str) -> Result<Metapodaci, String> {
    let json = pokreni(
        mapa,
        cargo,
        &["metadata", "--no-deps", "--locked", "--format-version", "1"],
        "rustup update",
    )?;
    serde_json::from_slice(&json).map_err(|e| greska_datoteke(&mapa.join("Cargo.toml"), e))
}

#[derive(Deserialize)]
pub struct Pokrivenost {
    pub data: Vec<Podaci>,
}
#[derive(Deserialize)]
pub struct Podaci {
    pub functions: Vec<LlvmFunkcija>,
}
#[derive(Deserialize)]
pub struct LlvmFunkcija {
    pub filenames: Vec<PathBuf>,
    pub regions: Vec<[u64; 8]>,
}

pub fn izvori(mapa: &Path, meta: &Metapodaci) -> Result<BTreeSet<PathBuf>, String> {
    let mut staze = BTreeSet::new();
    for paket in &meta.packages {
        if !meta.workspace_members.contains(&paket.id) {
            continue;
        }
        let korijen = paket
            .manifest_path
            .parent()
            .ok_or_else(|| greska_datoteke(mapa, "Cargo.toml"))?;
        dodaj_izvore(&korijen.join("src"), &mut staze)?;
        for cilj in &paket.targets {
            if cilj.kind.iter().any(|k| {
                [
                    "bin",
                    "lib",
                    "rlib",
                    "proc-macro",
                    "custom-build",
                    "cdylib",
                    "staticlib",
                    "dylib",
                ]
                .contains(&k.as_str())
            }) {
                staze.insert(cilj.src_path.clone());
            }
        }
    }
    Ok(staze)
}

fn dodaj_izvore(mapa: &Path, staze: &mut BTreeSet<PathBuf>) -> Result<(), String> {
    if !mapa.exists() {
        return Ok(());
    }
    for zapis in fs::read_dir(mapa).map_err(|e| greska_datoteke(mapa, e))? {
        let zapis = zapis.map_err(|e| greska_datoteke(mapa, e))?;
        obradi_zapis(&zapis, staze)?;
    }
    Ok(())
}

fn obradi_zapis(zapis: &fs::DirEntry, staze: &mut BTreeSet<PathBuf>) -> Result<(), String> {
    let tip = zapis
        .file_type()
        .map_err(|e| greska_datoteke(&zapis.path(), e))?;
    if tip.is_dir() {
        dodaj_izvore(&zapis.path(), staze)?;
    }
    if tip.is_file() && zapis.path().extension().is_some_and(|s| s == "rs") {
        staze.insert(zapis.path());
    }
    Ok(())
}

pub fn kriticna(staza: &str, cilj: &str) -> bool {
    let dijelovi = [
        "koordinate",
        "geometrija",
        "formati",
        "projekt",
        "atributi",
        "predlozak",
        "ispis",
        "geopaket",
    ];
    dijelovi.iter().any(|d| {
        cilj.split(['-', '_']).any(|p| p == *d) || staza.split(['/', '.', ':']).any(|p| p == *d)
    })
}

pub fn izmjeri(mapa: &Path, meta: &Metapodaci, llvm: &Pokrivenost) -> Result<Mjerenje, String> {
    let imena: BTreeSet<_> = meta
        .packages
        .iter()
        .filter(|p| meta.workspace_members.contains(&p.id))
        .map(|p| p.name.as_str())
        .collect();
    let cilj = imena.into_iter().collect::<Vec<_>>().join("+");
    let mut funkcije = Vec::new();
    for staza in izvori(mapa, meta)? {
        funkcije.extend(izmjeri_datoteku(mapa, &staza, &cilj, llvm)?);
    }
    if funkcije.is_empty() {
        return Err(tekst("nema-funkcija", &[]));
    }
    let ukupno: u64 = funkcije.iter().map(|f| f.podrucja.max(1)).sum();
    let pokriveno: u64 = funkcije.iter().map(|f| f.pokrivena).sum();
    let mut mjerenje = Mjerenje::novo(&cilj, funkcije);
    mjerenje.pokrivenost = pokriveno as f64 / ukupno as f64;
    Ok(mjerenje)
}

fn izmjeri_datoteku(
    mapa: &Path,
    staza: &Path,
    cilj: &str,
    llvm: &Pokrivenost,
) -> Result<Vec<Funkcija>, String> {
    let kod = fs::read_to_string(staza).map_err(|e| greska_datoteke(staza, e))?;
    let opisi = analiziraj(&kod).map_err(|e| {
        tekst(
            "sintaksa",
            &[("staza", staza.display().to_string()), ("greska", e)],
        )
    })?;
    let relativna = staza
        .strip_prefix(mapa)
        .map_err(|e| greska_datoteke(staza, e))?
        .to_string_lossy()
        .replace('\\', "/");
    let mut funkcije = Vec::new();
    for opis in &opisi {
        let kljuc = format!("{relativna}::{}", opis.ime);
        let regije = podrucja(staza, opis, &opisi, llvm);
        let broj = regije.len() as u64;
        let pokrivena = regije.values().filter(|v| **v).count() as u64;
        let udio = if broj == 0 {
            0.0
        } else {
            pokrivena as f64 / broj as f64
        };
        let mut f = Funkcija::nova(&kljuc, opis.cc, udio, kriticna(&kljuc, cilj));
        f.podrucja = broj;
        f.pokrivena = pokrivena;
        funkcije.push(f);
    }
    Ok(funkcije)
}

fn pripada(r: &[u64; 8], opis: &Opis) -> bool {
    (r[0] as usize, r[1] as usize) >= opis.pocetak && (r[2] as usize, r[3] as usize) <= opis.kraj
}

fn podrucja(
    staza: &Path,
    opis: &Opis,
    opisi: &[Opis],
    llvm: &Pokrivenost,
) -> BTreeMap<[u64; 4], bool> {
    let mut regije = BTreeMap::new();
    for podaci in &llvm.data {
        for f in &podaci.functions {
            for r in &f.regions {
                if valjano_podrucje(staza, opis, opisi, f, r) {
                    let kljuc = [r[0], r[1], r[2], r[3]];
                    *regije.entry(kljuc).or_insert(false) |= r[4] > 0;
                }
            }
        }
    }
    regije
}

fn valjano_podrucje(
    staza: &Path,
    opis: &Opis,
    opisi: &[Opis],
    f: &LlvmFunkcija,
    r: &[u64; 8],
) -> bool {
    if r[7] != 0 || f.filenames.get(r[5] as usize).is_none_or(|p| p != staza) || !pripada(r, opis) {
        return false;
    }
    !opisi
        .iter()
        .any(|u| u.pocetak > opis.pocetak && u.kraj <= opis.kraj && pripada(r, u))
}
