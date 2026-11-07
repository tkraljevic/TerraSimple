#![forbid(unsafe_code)]
#[cfg(test)]
mod testovi_naredbe;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::ExitCode,
};
use xtask::{
    alati, mjerenje,
    poruke::tekst,
    pravila::{self, Mjerenje, Postavke, Referenca},
};

struct Zahtjev {
    naredba: String,
    mapa: PathBuf,
    korijen: PathBuf,
    cargo: String,
}

fn main() -> ExitCode {
    match zahtjev().and_then(izvrsi) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}

fn zahtjev() -> Result<Zahtjev, String> {
    let korijen = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| tekst("uporaba", &[]))?
        .to_path_buf();
    procitaj_argumente(korijen, env::args_os().skip(1))
}

fn procitaj_argumente(
    korijen: PathBuf,
    mut argumenti: impl Iterator<Item = std::ffi::OsString>,
) -> Result<Zahtjev, String> {
    let naredba = argumenti
        .next()
        .and_then(|a| a.into_string().ok())
        .ok_or_else(|| tekst("uporaba", &[]))?;
    let mapa = match argumenti.next() {
        None => korijen.clone(),
        Some(a) if a == "--mapa" => {
            PathBuf::from(argumenti.next().ok_or_else(|| tekst("uporaba", &[]))?)
        }
        _ => return Err(tekst("uporaba", &[])),
    };
    if argumenti.next().is_some() {
        return Err(tekst("uporaba", &[]));
    }
    let mapa = fs::canonicalize(&mapa).map_err(|e| alati::greska_datoteke(&mapa, e))?;
    Ok(Zahtjev {
        naredba,
        mapa,
        korijen,
        cargo: "cargo".into(),
    })
}

fn izvrsi(z: Zahtjev) -> Result<(), String> {
    provjeri_naredbu(&z.naredba)?;
    let postavke = ucitaj_postavke(&z.korijen)?;
    let izvjestaji = mapa_izvjestaja(&z.korijen)?;
    fs::create_dir_all(&izvjestaji).map_err(|e| alati::greska_datoteke(&izvjestaji, e))?;
    let meta = mjerenje::metapodaci(&z.mapa, &z.cargo)?;
    match z.naredba.as_str() {
        "provjera-duboka" => {
            xtask::duboka::pokreni(&z.mapa, &z.cargo, &izvjestaji, &meta, &postavke)
        }
        _ => redovna(&z, &izvjestaji, &meta, &postavke),
    }
}

fn redovna(
    z: &Zahtjev,
    izvjestaji: &Path,
    meta: &mjerenje::Metapodaci,
    postavke: &Postavke,
) -> Result<(), String> {
    alati::dostupnost(&z.mapa, &z.cargo)?;
    let mjerenje = prikupi(z, izvjestaji, meta)?;
    ocijeni(z, &mjerenje, postavke)
}

fn mapa_izvjestaja(korijen: &Path) -> Result<PathBuf, String> {
    let vrijeme = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| alati::greska_datoteke(korijen, e))?
        .as_nanos();
    Ok(korijen
        .join(".quality")
        .join(format!("{}-{vrijeme}", std::process::id())))
}

fn prikupi(
    z: &Zahtjev,
    izvjestaji: &Path,
    meta: &mjerenje::Metapodaci,
) -> Result<Mjerenje, String> {
    let (llvm, mut padovi) = provjere(z, izvjestaji);
    let rezultat = alati::citaj_json::<mjerenje::Pokrivenost>(&llvm)
        .and_then(|llvm| mjerenje::izmjeri(&z.mapa, meta, &llvm));
    let mjerenje = match rezultat {
        Ok(m) => m,
        Err(e) => {
            padovi.push(e);
            return Err(padovi.join("\n"));
        }
    };
    prikazi(&mjerenje, izvjestaji)?;
    if padovi.is_empty() {
        Ok(mjerenje)
    } else {
        Err(padovi.join("\n"))
    }
}

fn pokusaj(
    z: &Zahtjev,
    izvjestaji: &Path,
    ime: &str,
    argumenti: &[&str],
    padovi: &mut Vec<String>,
) {
    println!(
        "{}",
        tekst(
            "izvodi",
            &[("alat", ime.into()), ("mapa", z.mapa.display().to_string())]
        )
    );
    let rezultat = alati::pokreni(&z.mapa, &z.cargo, argumenti, "rustup update");
    let zapis = match rezultat {
        Ok(izlaz) => String::from_utf8_lossy(&izlaz).into_owned(),
        Err(e) => {
            padovi.push(e.clone());
            e
        }
    };
    let staza = izvjestaji.join(format!("{ime}.log"));
    if let Err(e) = fs::write(&staza, zapis) {
        padovi.push(alati::greska_datoteke(&staza, e));
    }
}

fn provjere(z: &Zahtjev, izvjestaji: &Path) -> (PathBuf, Vec<String>) {
    let mut padovi = Vec::new();
    let llvm = izvjestaji.join("coverage.json");
    // Odluke (7. 11. 2026.): stari JSON nikad ne smije prikriti pad novog mjerenja.
    if llvm.exists()
        && let Err(e) = fs::remove_file(&llvm)
    {
        padovi.push(alati::greska_datoteke(&llvm, e));
    }
    pokusaj(
        z,
        izvjestaji,
        "rustfmt",
        &["fmt", "--all", "--", "--check"],
        &mut padovi,
    );
    pokusaj(
        z,
        izvjestaji,
        "clippy",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        &mut padovi,
    );
    pokusaj(
        z,
        izvjestaji,
        "testovi",
        &["test", "--workspace", "--all-targets", "--locked"],
        &mut padovi,
    );
    pokusaj(
        z,
        izvjestaji,
        "pokrivenost",
        &[
            "llvm-cov",
            "--workspace",
            "--all-targets",
            "--locked",
            "--json",
            "--output-path",
            &llvm.to_string_lossy(),
        ],
        &mut padovi,
    );
    let deny = z.korijen.join("quality/deny.toml");
    pokusaj(
        z,
        izvjestaji,
        "ovisnosti",
        &[
            "deny",
            "--format",
            "json",
            "--config",
            &deny.to_string_lossy(),
            "--locked",
            "check",
        ],
        &mut padovi,
    );
    (llvm, padovi)
}

fn prikazi(m: &Mjerenje, izvjestaji: &Path) -> Result<(), String> {
    let staza = izvjestaji.join("mjerenje.json");
    alati::zapisi_json(&staza, m)?;
    println!(
        "{}",
        tekst(
            "rezultat",
            &[
                ("broj", m.funkcije.len().to_string()),
                ("udio", xtask::poruke::broj(m.pokrivenost * 100.0, 2)),
                ("staza", staza.display().to_string())
            ]
        )
    );
    for f in &m.funkcije {
        if f.podrucja == 0 {
            eprintln!("{}", tekst("nemjereno", &[("funkcija", f.kljuc.clone())]));
        }
    }
    Ok(())
}

fn valjana_referenca(r: &Referenca, m: &Mjerenje) -> bool {
    r.schema == 1
        && r.mjerenja.values().all(|s| {
            s.metoda == m.metoda
                && (0.0..=1.0).contains(&s.pokrivenost)
                && s.funkcije.iter().all(|f| {
                    f.cc > 0
                        && (0.0..=1.0).contains(&f.pokrivenost)
                        && (f.rizik - pravila::rizik(f.cc, f.pokrivenost)).abs() < 1e-9
                })
        })
}

fn ocijeni(z: &Zahtjev, m: &Mjerenje, p: &Postavke) -> Result<(), String> {
    let staza = z.korijen.join("quality/baseline.json");
    let mut r: Referenca = alati::citaj_json(&staza)?;
    if !valjana_referenca(&r, m) {
        return Err(tekst("metoda", &[]));
    }
    let staro = r.mjerenja.get(&m.cilj);
    let nalazi = pravila::usporedi(m, staro, p);
    for n in &nalazi {
        eprintln!("{} [{}]", n.poruka, n.kljuc);
    }
    if z.naredba == "zapamti" {
        r.mjerenja.insert(m.cilj.clone(), m.clone());
        alati::zapisi_json(&staza, &r)?;
        println!(
            "{}",
            tekst("zapamceno", &[("staza", staza.display().to_string())])
        );
        return Ok(());
    }
    if staro.is_none() {
        return Err(tekst("nedostaje-referenca", &[("cilj", m.cilj.clone())]));
    }
    if !nalazi.is_empty() {
        return Err(tekst("pala", &[]));
    }
    println!("{}", tekst("prosla", &[]));
    Ok(())
}

fn ucitaj_postavke(korijen: &Path) -> Result<Postavke, String> {
    let postavke = alati::citaj_json(&korijen.join("quality/config.json"))?;
    pravila::provjeri_iznimke(&postavke, chrono::Local::now().date_naive())?;
    Ok(postavke)
}

fn provjeri_naredbu(naredba: &str) -> Result<(), String> {
    match naredba {
        "provjera-duboka" => Ok(()),
        "provjera" | "zapamti" => Ok(()),
        _ => Err(tekst("uporaba", &[])),
    }
}
