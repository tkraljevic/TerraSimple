//! Ključ i potpisi paketa izdanja (vidi docs/izdanje.md).
//!
//! ```text
//! cargo run -p nadogradnja --example potpis -- novi-kljuc
//! cargo run -p nadogradnja --example potpis -- potpisi PAKET...
//! ```
//!
//! Tajni ključ je u `~/.config/terrasimple/nadogradnja-ed25519.pk8` i nikad ne ide u
//! repozitorij ni na GitHub. Bez njega instalirani programi ne prihvaćaju nadogradnju.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn staza_kljuca() -> Result<PathBuf, String> {
    let dom = std::env::var_os("HOME").ok_or("Nema mape HOME.")?;
    Ok(PathBuf::from(dom).join(".config/terrasimple/nadogradnja-ed25519.pk8"))
}

/// Postojeći ključ se nikad ne prepisuje: novi bi značio da stari programi odbiju nadogradnju.
fn novi_kljuc(staza: &Path) -> Result<(), String> {
    if staza.exists() {
        return Err(format!("Ključ već postoji: {}", staza.display()));
    }
    let tajni = nadogradnja::novi_tajni_kljuc().map_err(|e| e.to_string())?;
    let mapa = staza.parent().ok_or("Staza ključa nema mapu.")?;
    std::fs::create_dir_all(mapa).map_err(|e| e.to_string())?;
    upisi_tajno(staza, &tajni)?;
    let javni = nadogradnja::javni_kljuc_iz_tajnog(&tajni).map_err(|e| e.to_string())?;
    println!("Tajni ključ: {}", staza.display());
    println!("Javni ključ za nadogradnja/src/mreza.rs: {javni}");
    Ok(())
}

#[cfg(unix)]
fn upisi_tajno(staza: &Path, sadrzaj: &[u8]) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(staza)
        .and_then(|mut d| d.write_all(sadrzaj))
        .map_err(|e| e.to_string())
}

#[cfg(not(unix))]
fn upisi_tajno(staza: &Path, sadrzaj: &[u8]) -> Result<(), String> {
    std::fs::write(staza, sadrzaj).map_err(|e| e.to_string())
}

fn potpisi(staza: &Path, paketi: &[String]) -> Result<(), String> {
    let tajni = std::fs::read(staza).map_err(|e| format!("{}: {e}", staza.display()))?;
    for paket in paketi {
        let p = Path::new(paket);
        let ime = p
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Neispravno ime: {paket}"))?;
        let sadrzaj = std::fs::read(p).map_err(|e| format!("{paket}: {e}"))?;
        let potpis = nadogradnja::potpisi(&tajni, ime, &sadrzaj).map_err(|e| e.to_string())?;
        let izlaz = format!("{paket}.sig");
        std::fs::write(&izlaz, format!("{potpis}\n")).map_err(|e| format!("{izlaz}: {e}"))?;
        println!("Potpisano: {izlaz}");
    }
    Ok(())
}

fn main() -> ExitCode {
    let argumenti: Vec<String> = std::env::args().skip(1).collect();
    let ishod = staza_kljuca().and_then(|k| match argumenti.split_first() {
        Some((n, [])) if n == "novi-kljuc" => novi_kljuc(&k),
        Some((n, paketi)) if n == "potpisi" && !paketi.is_empty() => potpisi(&k, paketi),
        _ => Err("Uporaba: potpis novi-kljuc | potpis potpisi PAKET...".into()),
    });
    match ishod {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
