//! Pronalazak QGIS-a i priprema otvaranja jednog GeoPackagea.
#![forbid(unsafe_code)]
mod trazenje;
mod verzija;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Child, Command, Output},
};
pub use trazenje::{Okolina, Platforma};
pub use verzija::Inacica;
pub type Greska = Box<dyn std::error::Error + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Qgis {
    pub staza: PathBuf,
    pub inacica: Option<Inacica>,
    pub paket_macos: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Naredba {
    pub program: PathBuf,
    pub argumenti: Vec<OsString>,
}
impl Naredba {
    /// Vraća proces odmah nakon pokretanja. Pozivatelj može poslije koristiti try_wait.
    pub fn pokreni(&self) -> std::io::Result<Child> {
        Command::new(&self.program).args(&self.argumenti).spawn()
    }
}
impl Qgis {
    /// Samo priprema naredbu; ne pokreće program i ne otvara prozor.
    ///
    /// Staza je uvijek apsolutna, pa je ni `open` ni QGIS ne mogu shvatiti kao opciju. Zato nema
    /// `--`, koji `open` ne navodi u svojim uputama.
    pub fn naredba(&self, datoteka: &Path) -> Result<Naredba, Greska> {
        provjeri_datoteku(datoteka)?;
        let p = bez_dugog_prefiksa(datoteka.canonicalize()?);
        if self.paket_macos {
            Ok(Naredba {
                program: "/usr/bin/open".into(),
                argumenti: vec![
                    "-a".into(),
                    self.staza.as_os_str().into(),
                    p.into_os_string(),
                ],
            })
        } else {
            Ok(Naredba {
                program: self.staza.clone(),
                argumenti: vec![p.into_os_string()],
            })
        }
    }
    pub fn otvori(&self, datoteka: &Path) -> Result<Child, Greska> {
        Ok(self.naredba(datoteka)?.pokreni()?)
    }

    /// Samo priprema naredbu; ne pokreće program i ne otvara prozor.
    ///
    /// Projekt se otvara preko protokola `geopackage:<staza>?projectName=<projekt>`.
    /// Na macOS-u s paketom aplikacije koristi se `/usr/bin/open -n -a <staza> --args --project <uri>`,
    /// pri čemu `-n` osigurava otvaranje novog prozora čak i ako je QGIS već pokrenut.
    /// Na ostalim platformama ili bez paketa koristi se `<program> --project <uri>`.
    pub fn naredba_projekt(&self, datoteka: &Path, projekt: &str) -> Result<Naredba, Greska> {
        provjeri_datoteku(datoteka)?;
        if projekt.trim().is_empty() {
            return Err("Naziv QGIS projekta ne smije biti prazan.".into());
        }
        let p = bez_dugog_prefiksa(datoteka.canonicalize()?);
        let uri = uri_projekta(&p, projekt);
        if self.paket_macos {
            Ok(Naredba {
                program: "/usr/bin/open".into(),
                argumenti: vec![
                    "-n".into(),
                    "-a".into(),
                    self.staza.as_os_str().into(),
                    "--args".into(),
                    "--project".into(),
                    uri.into(),
                ],
            })
        } else {
            Ok(Naredba {
                program: self.staza.clone(),
                argumenti: vec!["--project".into(), uri.into()],
            })
        }
    }

    pub fn otvori_projekt(&self, datoteka: &Path, projekt: &str) -> Result<Child, Greska> {
        Ok(self.naredba_projekt(datoteka, projekt)?.pokreni()?)
    }
}

pub(crate) fn uri_projekta(p: &Path, projekt: &str) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    format!("geopackage:{s}?projectName={projekt}")
}
/// Windows uz `canonicalize` vraća stazu `\\?\C:\…`, koju programi poput QGIS-a ne očekuju.
/// Prefiks se skida, a mrežna staza `\\?\UNC\poslužitelj\…` postaje `\\poslužitelj\…`.
fn bez_dugog_prefiksa(p: PathBuf) -> PathBuf {
    let s = p.to_string_lossy();
    if let Some(ostatak) = s.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{ostatak}"))
    } else if let Some(ostatak) = s.strip_prefix(r"\\?\") {
        PathBuf::from(ostatak)
    } else {
        p
    }
}

fn provjeri_datoteku(p: &Path) -> Result<(), Greska> {
    if !p.is_file() {
        return Err(
            "QGIS nije otvoren: datoteka projekta ne postoji. Spremi projekt i pokušaj ponovno."
                .into(),
        );
    }
    if !p
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("gpkg"))
    {
        return Err("QGIS nije otvoren: projekt nije spremljen kao GeoPackage. Spremi ga s nastavkom .gpkg.".into());
    }
    Ok(())
}

/// Zadana staza ima prednost. Bez nje se uspoređuju pronađene inačice.
/// Za izvršne datoteke bez metapodataka upit --version odvojen je od otvaranja.
pub fn pronadji(zadana: Option<&Path>) -> Result<Qgis, Greska> {
    pronadji_u(&Okolina::trenutna(), zadana, izlaz_verzije)
}

/// Ulazne mape i čitanje inačice mogu se zamijeniti u testovima, bez pokretanja QGIS-a.
pub fn pronadji_u(
    okolina: &Okolina,
    zadana: Option<&Path>,
    upit: impl Fn(&Path) -> std::io::Result<Output>,
) -> Result<Qgis, Greska> {
    if let Some(p) = zadana {
        return trazenje::zadani(p, okolina.platforma);
    }
    let kandidati = trazenje::kandidati(okolina)?;
    let mut svi = Vec::new();
    for mut q in kandidati {
        if q.inacica.is_none() {
            let p = if q.paket_macos {
                q.staza.join("Contents/MacOS/QGIS")
            } else {
                q.staza.clone()
            };
            q.inacica = upit(&p).ok().and_then(verzija::iz_izlaza);
        }
        svi.push(q);
    }
    odaberi(svi)
}
fn odaberi(mut svi: Vec<Qgis>) -> Result<Qgis, Greska> {
    if svi.len() > 1 && svi.iter().any(|q| q.inacica.is_none()) {
        return Err("QGIS nije otvoren: na računalu je više QGIS-a, a ne zna se koji je noviji. Ostavi samo jedan pa pokušaj ponovno.".into());
    }
    svi.sort_by(|a, b| {
        a.inacica
            .cmp(&b.inacica)
            .then_with(|| b.staza.cmp(&a.staza))
    });
    svi.pop().ok_or_else(|| {
        "QGIS nije pronađen na ovom računalu. Instaliraj ga s qgis.org pa pokušaj ponovno.".into()
    })
}
fn izlaz_verzije(p: &Path) -> std::io::Result<Output> {
    Command::new(p)
        .arg("--version")
        .env("QT_QPA_PLATFORM", "offscreen")
        .output()
}

#[cfg(test)]
mod testovi;
