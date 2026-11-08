//! Ograničeni pokusi libFuzzera, odvojeni od redovne stabilne provjere.
use crate::{alati, poruke::tekst};
use serde::Serialize;
use std::{fs, path::Path, process::Command};

#[derive(Debug, Serialize)]
pub struct Pokus {
    pub cilj: String,
    pub sekunde: u64,
    pub ulaza: u64,
    pub kod: Option<i32>,
    pub prosao: bool,
}

/// Nedostajući nightly ne ruši stabilni CI; preskakanje nije prolazan fuzz.
pub fn kratko(mapa: &Path, cargo: &str, izlaz: &Path) -> Result<(), String> {
    if dostupan(mapa, cargo) {
        return pokreni(mapa, cargo, izlaz, 10);
    }
    fs::create_dir_all(izlaz).map_err(|e| alati::greska_datoteke(izlaz, e))?;
    alati::zapisi_json(
        &izlaz.join("fuzz.json"),
        &serde_json::json!({"stanje":"nije-pokrenuto","razlog":"Nightly ili cargo-fuzz nisu dostupni."}),
    )?;
    println!("{}", tekst("fuzz-nedostupan", &[]));
    Ok(())
}
fn dostupan(mapa: &Path, cargo: &str) -> bool {
    Command::new(cargo)
        .current_dir(mapa)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .args(["+nightly", "fuzz", "--version"])
        .output()
        .is_ok_and(|o| o.status.success())
}

pub fn trajanje(v: Option<&str>) -> Result<u64, String> {
    let s = v
        .unwrap_or("60")
        .parse::<u64>()
        .map_err(|_| tekst("fuzz-trajanje", &[]))?;
    if !(1..=86400).contains(&s) {
        return Err(tekst("fuzz-trajanje", &[]));
    }
    Ok(s)
}

pub fn iz_okoline(mapa: &Path, cargo: &str, izlaz: &Path) -> Result<(), String> {
    let sekunde = trajanje(std::env::var("TERRASIMPLE_FUZZ_SEKUNDE").ok().as_deref())?;
    pokreni(mapa, cargo, izlaz, sekunde)
}

pub fn pokreni(mapa: &Path, cargo: &str, izlaz: &Path, sekunde: u64) -> Result<(), String> {
    let mut padovi = Vec::new();
    let mut pokusi = Vec::new();
    for cilj in ["geometrija", "qml", "datoteka"] {
        match cilj_pokreni(mapa, cargo, izlaz, cilj, sekunde) {
            Ok(p) => {
                println!(
                    "{}",
                    tekst(
                        "fuzz-rezultat",
                        &[
                            ("cilj", cilj.into()),
                            ("broj", p.ulaza.to_string()),
                            (
                                "kod",
                                p.kod.map_or_else(|| "bez koda".into(), |k| k.to_string())
                            )
                        ]
                    )
                );
                if !p.prosao {
                    padovi.push(tekst("fuzz-pad", &[("cilj", cilj.into())]));
                }
                pokusi.push(p);
            }
            Err(e) => padovi.push(e),
        }
    }
    alati::zapisi_json(&izlaz.join("fuzz.json"), &pokusi)?;
    if padovi.is_empty() {
        Ok(())
    } else {
        Err(padovi.join("\n"))
    }
}

fn pripremi(mapa: &Path, izlaz: &Path, cilj: &str) -> Result<(), String> {
    for d in [
        mapa.join("fuzz/corpus").join(cilj),
        mapa.join("fuzz/artifacts").join(cilj),
        izlaz.to_path_buf(),
    ] {
        fs::create_dir_all(&d).map_err(|e| alati::greska_datoteke(&d, e))?;
    }
    Ok(())
}

fn naredba(mapa: &Path, cargo: &str, cilj: &str, sekunde: u64) -> Command {
    let mut c = Command::new(cargo);
    c.current_dir(mapa)
        .env("RUSTUP_AUTO_INSTALL", "0")
        .env("CARGO_TARGET_DIR", mapa.join("target/fuzz"))
        .env("TERRASIMPLE_FUZZ_RAD", mapa.join("fuzz/rad"))
        .args(["+nightly", "fuzz", "run", cilj])
        .arg(format!("fuzz/corpus/{cilj}"))
        .arg(format!("fuzz/pocetni/{cilj}"))
        .arg("--")
        .args([
            format!("-max_total_time={sekunde}"),
            "-timeout=5".into(),
            "-rss_limit_mb=1024".into(),
            "-max_len=262144".into(),
            "-print_final_stats=1".into(),
            format!("-artifact_prefix=fuzz/artifacts/{cilj}/"),
        ]);
    c
}

fn cilj_pokreni(
    mapa: &Path,
    cargo: &str,
    izlaz: &Path,
    cilj: &str,
    sekunde: u64,
) -> Result<Pokus, String> {
    pripremi(mapa, izlaz, cilj)?;
    let log = izlaz.join(format!("fuzz-{cilj}.log"));
    let f = fs::File::create(&log).map_err(|e| alati::greska_datoteke(&log, e))?;
    let err = f.try_clone().map_err(|e| alati::greska_datoteke(&log, e))?;
    let status = naredba(mapa, cargo, cilj, sekunde)
        .stdout(f)
        .stderr(err)
        .status()
        .map_err(|e| alati::greska_datoteke(&log, e))?;
    let s = fs::read_to_string(&log).map_err(|e| alati::greska_datoteke(&log, e))?;
    Ok(ocijeni(cilj, sekunde, status.code(), &s))
}

pub fn ocijeni(cilj: &str, sekunde: u64, kod: Option<i32>, zapis: &str) -> Pokus {
    let ulaza = zapis
        .lines()
        .filter_map(|r| {
            r.strip_prefix("stat::number_of_executed_units:")?
                .trim()
                .parse::<u64>()
                .ok()
        })
        .next_back()
        .unwrap_or(0);
    Pokus {
        cilj: cilj.into(),
        sekunde,
        ulaza,
        kod,
        prosao: kod == Some(0) && ulaza > 0,
    }
}

#[cfg(test)]
mod testovi {
    use super::*;
    #[test]
    fn trajanje_i_rezultat_ne_skrivaju_pad() {
        assert_eq!(trajanje(None).unwrap(), 60);
        assert_eq!(trajanje(Some("1800")).unwrap(), 1800);
        for s in ["0", "86401", "krivo"] {
            assert!(trajanje(Some(s)).is_err());
        }
        let z = "stat::number_of_executed_units: 123\n";
        assert!(ocijeni("qml", 1, Some(0), z).prosao);
        for kod in [None, Some(1), Some(77)] {
            assert!(!ocijeni("qml", 1, kod, z).prosao);
        }
        for z in [
            "",
            "stat::number_of_executed_units: loše",
            "stat::number_of_executed_units: 0",
        ] {
            assert!(!ocijeni("qml", 1, Some(0), z).prosao);
        }
    }
    #[test]
    fn naredba_ima_nightly_i_granice_a_ne_mijenja_zadani_rust() {
        let c = naredba(Path::new("rad"), "cargo", "qml", 30);
        let a: Vec<_> = c
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(&a[..4], &["+nightly", "fuzz", "run", "qml"]);
        for s in [
            "-max_total_time=30",
            "-timeout=5",
            "-rss_limit_mb=1024",
            "-max_len=262144",
        ] {
            assert!(a.iter().any(|a| a == s));
        }
    }
}
