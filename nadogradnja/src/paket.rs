//! Raspakiravanje paketa izdanja. Paket je potpisan prije nego što stigne ovamo, ali se svejedno
//! čita oprezno: bez staza izvan odredišta, bez poveznica i uz gornju granicu veličine.

use crate::Greska;
use crate::greska::ostecen;
use std::fs;
use std::io::{Cursor, Read};
use std::path::{Component, Path, PathBuf};

/// Raspakirano smije imati najviše 1 GB (paketi su sada oko 60 MB).
const NAJVISE_RASPAKIRANO: u64 = 1 << 30;
const NAJVISE_STAVKI: usize = 20_000;

/// Raspakirava zip ili tar.gz (prema imenu paketa) u praznu mapu `odrediste`.
///
/// # Errors
/// [`Greska::OstecenPaket`] kad paket nije ispravan ili ima nedopuštene stavke.
pub fn raspakiraj(sadrzaj: &[u8], ime: &str, odrediste: &Path) -> Result<(), Greska> {
    let mut granica = Granica::default();
    if ime.ends_with(".tar.gz") {
        raspakiraj_tar(sadrzaj, odrediste, &mut granica)
    } else {
        raspakiraj_zip(sadrzaj, odrediste, &mut granica)
    }
}

#[derive(Default)]
struct Granica {
    bajtova: u64,
    stavki: usize,
}

impl Granica {
    fn jos_jedna(&mut self) -> Result<(), Greska> {
        self.stavki += 1;
        if self.stavki > NAJVISE_STAVKI {
            return Err(ostecen("previše stavki"));
        }
        Ok(())
    }

    fn upisi(&mut self, izvor: impl Read, cilj: &Path) -> Result<(), Greska> {
        if let Some(mapa) = cilj.parent() {
            fs::create_dir_all(mapa).map_err(ostecen)?;
        }
        let preostalo = NAJVISE_RASPAKIRANO - self.bajtova;
        let mut datoteka = fs::File::create(cilj).map_err(ostecen)?;
        let upisano =
            std::io::copy(&mut izvor.take(preostalo + 1), &mut datoteka).map_err(ostecen)?;
        if upisano > preostalo {
            return Err(ostecen("raspakirano je veće od 1 GB"));
        }
        self.bajtova += upisano;
        Ok(())
    }
}

/// Staza unutar odredišta, ili greška za apsolutnu stazu i `..`.
fn sigurna_staza(odrediste: &Path, staza: &Path) -> Result<PathBuf, Greska> {
    let sigurna = staza
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir));
    if !sigurna || staza.as_os_str().is_empty() {
        return Err(ostecen(format!("nedopuštena staza {}", staza.display())));
    }
    Ok(odrediste.join(staza))
}

#[cfg(unix)]
fn postavi_dozvole(staza: &Path, mod_: Option<u32>) -> Result<(), Greska> {
    use std::os::unix::fs::PermissionsExt;
    match mod_ {
        Some(m) => {
            fs::set_permissions(staza, fs::Permissions::from_mode(m & 0o755)).map_err(ostecen)
        }
        None => Ok(()),
    }
}

#[cfg(not(unix))]
#[allow(clippy::unnecessary_wraps)]
fn postavi_dozvole(_staza: &Path, _mod: Option<u32>) -> Result<(), Greska> {
    Ok(())
}

/// `ditto` na Macu uz program sprema i mapu `__MACOSX` s dodatnim podacima; ona ne treba.
fn preskoci_zip(ime: &str) -> bool {
    ime.starts_with("__MACOSX/")
}

fn napravi_mapu(cilj: &Path) -> Result<(), Greska> {
    fs::create_dir_all(cilj).map_err(ostecen)
}

fn raspakiraj_zip(sadrzaj: &[u8], odrediste: &Path, granica: &mut Granica) -> Result<(), Greska> {
    let mut arhiva = zip::ZipArchive::new(Cursor::new(sadrzaj)).map_err(ostecen)?;
    for i in 0..arhiva.len() {
        stavka_zipa(arhiva.by_index(i).map_err(ostecen)?, odrediste, granica)?;
    }
    Ok(())
}

/// Ime stavke, ili `None` za stavku koja se preskače. Poveznica se odbija.
fn ime_zipa<R: Read>(stavka: &zip::read::ZipFile<'_, R>) -> Result<Option<String>, Greska> {
    let ime = stavka.name().map_err(ostecen)?.into_owned();
    if stavka.is_symlink() {
        return Err(ostecen(format!("poveznica {ime}")));
    }
    Ok((!preskoci_zip(&ime)).then_some(ime))
}

fn stavka_zipa<R: Read>(
    stavka: zip::read::ZipFile<'_, R>,
    odrediste: &Path,
    granica: &mut Granica,
) -> Result<(), Greska> {
    let Some(ime) = ime_zipa(&stavka)? else {
        return Ok(());
    };
    granica.jos_jedna()?;
    let cilj = sigurna_staza(odrediste, Path::new(&ime))?;
    if stavka.is_dir() {
        return napravi_mapu(&cilj);
    }
    let mod_ = stavka.unix_mode();
    granica.upisi(stavka, &cilj)?;
    postavi_dozvole(&cilj, mod_)
}

fn raspakiraj_tar(sadrzaj: &[u8], odrediste: &Path, granica: &mut Granica) -> Result<(), Greska> {
    let mut arhiva = tar::Archive::new(flate2::read::GzDecoder::new(Cursor::new(sadrzaj)));
    for stavka in arhiva.entries().map_err(ostecen)? {
        stavka_tara(stavka.map_err(ostecen)?, odrediste, granica)?;
    }
    Ok(())
}

fn stavka_tara<R: Read>(
    stavka: tar::Entry<'_, R>,
    odrediste: &Path,
    granica: &mut Granica,
) -> Result<(), Greska> {
    granica.jos_jedna()?;
    let staza = PathBuf::from(String::from_utf8_lossy(&stavka.path_bytes()).into_owned());
    let cilj = sigurna_staza(odrediste, &staza)?;
    let vrsta = stavka.header().entry_type();
    if vrsta.is_dir() {
        return napravi_mapu(&cilj);
    }
    if !vrsta.is_file() {
        return Err(ostecen(format!("nedopuštena stavka {}", cilj.display())));
    }
    let mod_ = stavka.header().mode().ok();
    granica.upisi(stavka, &cilj)?;
    postavi_dozvole(&cilj, mod_)
}

#[cfg(test)]
#[path = "paket_testovi.rs"]
mod testovi;
