use crate::poruke::tekst;
use serde::de::DeserializeOwned;
use std::{fs, path::Path, process::Command};

pub fn citaj_json<T: DeserializeOwned>(staza: &Path) -> Result<T, String> {
    let sadrzaj = fs::read(staza).map_err(|e| greska_datoteke(staza, e))?;
    serde_json::from_slice(&sadrzaj).map_err(|e| {
        tekst(
            "json",
            &[
                ("staza", staza.display().to_string()),
                ("greska", e.to_string()),
            ],
        )
    })
}

pub fn greska_datoteke(staza: &Path, greska: impl std::fmt::Display) -> String {
    tekst(
        "datoteka",
        &[
            ("staza", staza.display().to_string()),
            ("greska", greska.to_string()),
        ],
    )
}

pub fn zapisi_json(staza: &Path, podaci: &impl serde::Serialize) -> Result<(), String> {
    let tekst = serde_json::to_vec_pretty(podaci).map_err(|e| greska_datoteke(staza, e))?;
    fs::write(staza, tekst).map_err(|e| greska_datoteke(staza, e))
}

pub fn pokreni(
    mapa: &Path,
    program: &str,
    argumenti: &[&str],
    uputa: &str,
) -> Result<Vec<u8>, String> {
    let izlaz = Command::new(program)
        .args(argumenti)
        .current_dir(mapa)
        .output()
        .map_err(|e| {
            tekst(
                "alat",
                &[
                    ("alat", program.into()),
                    ("uputa", uputa.into()),
                    ("greska", e.to_string()),
                ],
            )
        })?;
    if !izlaz.status.success() {
        return Err(tekst(
            "pad-alata",
            &[
                ("alat", format!("{program} {}", argumenti.join(" "))),
                ("mapa", mapa.display().to_string()),
                (
                    "izlaz",
                    format!(
                        "{}\n{}",
                        String::from_utf8_lossy(&izlaz.stdout),
                        String::from_utf8_lossy(&izlaz.stderr)
                    ),
                ),
            ],
        ));
    }
    Ok(izlaz.stdout)
}

pub fn dostupnost(mapa: &Path, cargo: &str) -> Result<(), String> {
    let komponente = pokreni(
        mapa,
        "rustup",
        &["component", "list", "--installed"],
        "rustup component add llvm-tools",
    )?;
    if !String::from_utf8_lossy(&komponente)
        .lines()
        .any(|r| r.starts_with("llvm-tools"))
    {
        return Err(tekst(
            "alat",
            &[
                ("alat", "llvm-tools".into()),
                ("uputa", "rustup component add llvm-tools".into()),
                ("greska", "llvm-tools".into()),
            ],
        ));
    }
    for (argumenti, uputa) in [
        (vec!["fmt", "--version"], "rustup component add rustfmt"),
        (vec!["clippy", "--version"], "rustup component add clippy"),
        (
            vec!["llvm-cov", "--version"],
            "cargo install cargo-llvm-cov --locked; rustup component add llvm-tools",
        ),
        (
            vec!["deny", "--version"],
            "cargo install cargo-deny --locked",
        ),
    ] {
        pokreni(mapa, cargo, &argumenti, uputa).map_err(|e| {
            tekst(
                "alat",
                &[
                    ("alat", argumenti.join(" ")),
                    ("uputa", uputa.into()),
                    ("greska", e),
                ],
            )
        })?;
    }
    Ok(())
}
