use crate::{Greska, Qgis, verzija};
use std::{
    collections::BTreeSet,
    ffi::OsString,
    path::{Path, PathBuf},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platforma {
    Mac,
    Windows,
    Linux,
}
impl Platforma {
    pub fn trenutna() -> Self {
        Self::iz_naziva(std::env::consts::OS)
    }
    pub fn iz_naziva(s: &str) -> Self {
        match s {
            "macos" => Self::Mac,
            "windows" => Self::Windows,
            _ => Self::Linux,
        }
    }
}
#[derive(Clone, Debug)]
pub struct Okolina {
    pub platforma: Platforma,
    pub aplikacije: Vec<PathBuf>,
    pub program_files: Vec<PathBuf>,
    pub osgeo4w: Vec<PathBuf>,
    pub putanja: Vec<PathBuf>,
}
impl Okolina {
    pub fn trenutna() -> Self {
        Self::iz_varijabli(Platforma::trenutna(), |n| std::env::var_os(n))
    }
    pub fn iz_varijabli(platforma: Platforma, v: impl Fn(&str) -> Option<OsString>) -> Self {
        let program_files = ["ProgramFiles", "ProgramFiles(x86)"]
            .into_iter()
            .filter_map(&v)
            .map(PathBuf::from)
            .collect();
        let mut osgeo4w: Vec<_> = v("OSGEO4W_ROOT").map(PathBuf::from).into_iter().collect();
        let disk = v("SystemDrive").unwrap_or_else(|| "C:".into());
        for n in ["OSGeo4W", "OSGeo4W64"] {
            osgeo4w.push(PathBuf::from(format!("{}\\{n}", disk.to_string_lossy())));
        }
        let putanja = v("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        Self {
            platforma,
            aplikacije: vec!["/Applications".into()],
            program_files,
            osgeo4w,
            putanja,
        }
    }
}
fn podmape(p: &Path) -> Result<Vec<PathBuf>, Greska> {
    match std::fs::read_dir(p) {
        Ok(r) => r.map(|e| e.map(|e| e.path()).map_err(Into::into)).collect(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e.into()),
    }
}
fn ime(p: &Path) -> String {
    p.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase()
}
fn mac(p: &Path) -> Option<Qgis> {
    if !p.join("Contents/MacOS/QGIS").is_file() {
        return None;
    }
    Some(Qgis {
        staza: p.to_path_buf(),
        inacica: verzija::mac(p),
        paket_macos: true,
    })
}
fn windows(p: &Path) -> Vec<Qgis> {
    ["qgis", "qgis-ltr"]
        .into_iter()
        .filter_map(|paket| {
            let exe = p.join("bin").join(format!("{paket}-bin.exe"));
            exe.is_file().then(|| Qgis {
                staza: exe,
                inacica: verzija::windows(p, paket),
                paket_macos: false,
            })
        })
        .collect()
}
pub(crate) fn zadani(p: &Path, platforma: Platforma) -> Result<Qgis, Greska> {
    let p = p
        .canonicalize()
        .map_err(|_| "Zadana staza QGIS-a ne postoji. Odaberite njegov program.")?;
    if platforma == Platforma::Mac && p.extension().is_some_and(|e| e == "app") {
        return mac(&p).ok_or_else(|| "Odabrani paket nema program QGIS.".into());
    }
    if !p.is_file() {
        return Err("Odaberite izvršnu datoteku QGIS-a, a ne mapu.".into());
    }
    Ok(Qgis {
        staza: p,
        inacica: None,
        paket_macos: false,
    })
}
fn macovi(o: &Okolina) -> Result<Vec<Qgis>, Greska> {
    let mut svi = Vec::new();
    for d in &o.aplikacije {
        svi.extend(
            podmape(d)?
                .iter()
                .filter(|p| ime(p).starts_with("qgis") && p.extension().is_some_and(|e| e == "app"))
                .filter_map(|p| mac(p)),
        );
    }
    Ok(svi)
}
fn windowsi(o: &Okolina) -> Result<Vec<Qgis>, Greska> {
    let mut korijeni = o.osgeo4w.clone();
    for d in &o.program_files {
        korijeni.extend(
            podmape(d)?
                .into_iter()
                .filter(|p| ime(p).starts_with("qgis")),
        );
    }
    Ok(korijeni.iter().flat_map(|p| windows(p)).collect())
}
fn linuxi(o: &Okolina) -> Vec<Qgis> {
    o.putanja
        .iter()
        .map(|p| p.join("qgis"))
        .filter(|p| p.is_file())
        .map(|p| Qgis {
            staza: p,
            inacica: None,
            paket_macos: false,
        })
        .collect()
}
pub(crate) fn kandidati(o: &Okolina) -> Result<Vec<Qgis>, Greska> {
    let svi = match o.platforma {
        Platforma::Mac => macovi(o)?,
        Platforma::Windows => windowsi(o)?,
        Platforma::Linux => linuxi(o),
    };
    let mut vidjeni = BTreeSet::new();
    Ok(svi
        .into_iter()
        .filter(|q| vidjeni.insert(q.staza.canonicalize().unwrap_or_else(|_| q.staza.clone())))
        .collect())
}
