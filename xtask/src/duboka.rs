use crate::{alati, mjerenje, poruke::tekst, pravila::Postavke};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

#[derive(Deserialize)]
pub struct Ishod {
    pub outcomes: Vec<Pokus>,
    pub total_mutants: usize,
    pub end_time: Option<String>,
}
#[derive(Deserialize)]
pub struct Pokus {
    pub scenario: Scenarij,
    pub summary: String,
}
#[derive(Deserialize)]
pub enum Scenarij {
    Baseline,
    Mutant(Mutacija),
}
#[derive(Deserialize)]
pub struct Mutacija {
    pub name: String,
    pub file: String,
    pub function: Option<Funkcija>,
}
#[derive(Deserialize)]
pub struct Funkcija {
    pub function_name: String,
}

fn opis(m: &Mutacija) -> String {
    tekst(
        "mutacija",
        &[
            ("datoteka", m.file.clone()),
            (
                "funkcija",
                m.function.as_ref().map_or("—", |f| &f.function_name).into(),
            ),
            ("promjena", m.name.clone()),
        ],
    )
}

fn nalaz(p: &Pokus, postavke: &Postavke) -> Option<String> {
    match &p.scenario {
        Scenarij::Baseline if p.summary == "Success" => None,
        Scenarij::Mutant(_) if p.summary == "CaughtMutant" => None,
        Scenarij::Mutant(m) if p.summary == "Unviable" => {
            println!("{}", tekst("mutacija-neprevodiva", &[("opis", opis(m))]));
            None
        }
        Scenarij::Mutant(m) => {
            let kljuc = format!("mutacija:{}", m.name);
            println!("{} [{}]", opis(m), kljuc);
            if p.summary == "MissedMutant" && postavke.iznimke.iter().any(|i| i.kljuc == kljuc) {
                None
            } else {
                Some(tekst(
                    "mutacija-pad",
                    &[("ishod", p.summary.clone()), ("opis", opis(m))],
                ))
            }
        }
        Scenarij::Baseline => Some(tekst("mutacije-osnova", &[])),
    }
}

pub fn ocijeni(m: &Ishod, p: &Postavke) -> Result<(), String> {
    let mut padovi: Vec<_> = m.outcomes.iter().filter_map(|o| nalaz(o, p)).collect();
    if !potpun(m) {
        padovi.push(tekst("mutacije-prazno", &[]));
    }
    if padovi.is_empty() {
        Ok(())
    } else {
        Err(padovi.join("\n"))
    }
}

fn potpun(m: &Ishod) -> bool {
    let broj = m
        .outcomes
        .iter()
        .filter(|o| matches!(o.scenario, Scenarij::Mutant(_)))
        .count();
    m.end_time.is_some()
        && broj > 0
        && broj == m.total_mutants
        && m.outcomes
            .iter()
            .any(|o| matches!(o.scenario, Scenarij::Baseline))
}

fn paket(mapa: &Path, cargo: &str, ime: &str, izlaz: &Path, p: &Postavke) -> Result<(), String> {
    fs::create_dir_all(izlaz).map_err(|e| alati::greska_datoteke(izlaz, e))?;
    let rezultat = std::process::Command::new(cargo)
        .env("CARGO_TARGET_DIR", "target")
        .current_dir(mapa)
        .args([
            "mutants",
            "--no-config",
            "--gitignore",
            "true",
            "--package",
            ime,
            "--output",
            &izlaz.to_string_lossy(),
            "--timeout",
            "60",
            "--build-timeout",
            "180",
            "--jobs",
            "2",
            "--",
            "--locked",
        ])
        .output()
        .map_err(|e| alati::greska_datoteke(izlaz, e))?;
    let zapis = format!(
        "{}\n{}",
        String::from_utf8_lossy(&rezultat.stdout),
        String::from_utf8_lossy(&rezultat.stderr)
    );
    let log = izlaz.join("naredba.log");
    fs::write(&log, zapis).map_err(|e| alati::greska_datoteke(&log, e))?;
    let ishod: Ishod = alati::citaj_json(&izlaz.join("mutants.out/outcomes.json"))?;
    ocijeni(&ishod, p)?;
    // Odluke (7. 11. 2026.): iznimka smije odobriti samo preživjelu mutaciju,
    // nikad kvar alata, prekid ili prekoračenje vremena.
    let kod = if ishod.outcomes.iter().any(|o| o.summary == "MissedMutant") {
        2
    } else {
        0
    };
    if rezultat.status.code() != Some(kod) {
        return Err(tekst(
            "mutacije-alat",
            &[("kod", rezultat.status.to_string())],
        ));
    }
    Ok(())
}

pub fn pokreni(
    mapa: &Path,
    cargo: &str,
    izlaz: &Path,
    meta: &mjerenje::Metapodaci,
    postavke: &Postavke,
) -> Result<(), String> {
    crate::fuzz::kratko(mapa, cargo, izlaz)?;
    duplikacija(mapa, meta, izlaz)?;
    alati::pokreni(
        mapa,
        cargo,
        &["mutants", "--version"],
        "cargo install cargo-mutants --version 27.1.0 --locked",
    )?;
    let paketi: Vec<_> = meta
        .packages
        .iter()
        .filter(|p| meta.workspace_members.contains(&p.id) && mjerenje::kriticna("", &p.name))
        .collect();
    if paketi.is_empty() {
        return Err(tekst("mutacije-prazno", &[]));
    }
    let padovi: Vec<_> = paketi
        .iter()
        .filter_map(|p| paket(mapa, cargo, &p.name, &izlaz.join(&p.name), postavke).err())
        .collect();
    if padovi.is_empty() {
        Ok(())
    } else {
        Err(padovi.join("\n"))
    }
}

/// Odluke (7. 11. 2026.): prvi udio samo izvještava; nema skrivene promjene reference.
fn duplikacija(mapa: &Path, meta: &mjerenje::Metapodaci, izlaz: &Path) -> Result<(), String> {
    let izvori: Vec<_> = mjerenje::izvori(mapa, meta)?
        .iter()
        .map(|s| fs::read_to_string(s).map_err(|e| alati::greska_datoteke(s, e)))
        .collect::<Result<_, _>>()?;
    let (ukupno, ponovljeno) = prebroji(&izvori);
    let udio = if ukupno == 0 {
        0.0
    } else {
        ponovljeno as f64 / ukupno as f64 * 100.0
    };
    alati::zapisi_json(
        &izlaz.join("duplikacija.json"),
        &serde_json::json!({"metoda":"retci-10-v1", "ukupno":ukupno, "ponovljeno":ponovljeno, "postotak":udio}),
    )?;
    println!(
        "{}",
        tekst("duplikacija", &[("udio", crate::poruke::broj(udio, 2))])
    );
    Ok(())
}

pub fn prebroji(izvori: &[String]) -> (usize, usize) {
    let retci: Vec<Vec<_>> = izvori
        .iter()
        .map(|s| {
            s.lines()
                .map(str::trim)
                .filter(|r| !r.is_empty() && !r.starts_with("//"))
                .collect()
        })
        .collect();
    let mut blokovi: BTreeMap<Vec<&str>, Vec<(usize, usize)>> = BTreeMap::new();
    for (datoteka, redci) in retci.iter().enumerate() {
        for (pocetak, blok) in redci.windows(10).enumerate() {
            blokovi
                .entry(blok.to_vec())
                .or_default()
                .push((datoteka, pocetak));
        }
    }
    let mut ponovljeni = BTreeSet::new();
    for mjesta in blokovi.values().filter(|m| m.len() > 1) {
        for (datoteka, pocetak) in mjesta {
            for redak in *pocetak..pocetak + 10 {
                ponovljeni.insert((*datoteka, redak));
            }
        }
    }
    (retci.iter().map(Vec::len).sum(), ponovljeni.len())
}
