//! Zamjena programa na disku. Stari dio se prvo preimenuje (`… .stara`), pa novi dolazi na
//! njegovo mjesto; ako bilo što ne uspije, vraća se staro. Na Windowsu se program koji radi ne
//! može obrisati, ali se može preimenovati, pa ostatak briše idući pokretanje.

use crate::{Greska, Sustav, raspakiraj};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const RADNA_MAPA: &str = ".TerraSimple-nadogradnja";
const NASTAVAK_STAROGA: &str = ".stara";

/// Gdje je instaliran program koji radi: paket `.app` na Macu, a drugdje mapa s programom.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mjesto {
    sustav: Sustav,
    korijen: PathBuf,
}

const fn ime_programa(sustav: Sustav) -> &'static str {
    match sustav {
        Sustav::Windows => "TerraSimple.exe",
        Sustav::MacOs | Sustav::Linux => "TerraSimple",
    }
}

fn ime(staza: &Path) -> Option<&str> {
    staza.file_name().and_then(|n| n.to_str())
}

impl Mjesto {
    /// Mjesto programa iz staze izvršne datoteke. Program izgrađen iz izvornog koda nije u
    /// paketu izdanja, pa se ne nadograđuje sam.
    ///
    /// # Errors
    /// [`Greska::NijeIzPaketa`] kad staza ne izgleda kao raspakirani paket izdanja.
    pub fn za_program(exe: &Path, sustav: Sustav) -> Result<Self, Greska> {
        let korijen = match sustav {
            Sustav::MacOs => paket_maca(exe),
            Sustav::Windows | Sustav::Linux => mapa_programa(exe, sustav),
        };
        korijen
            .map(|korijen| Self { sustav, korijen })
            .ok_or_else(|| Greska::NijeIzPaketa(exe.to_path_buf()))
    }

    /// Paket `.app` ili mapa programa.
    #[must_use]
    pub fn korijen(&self) -> &Path {
        &self.korijen
    }

    /// Radna mapa mora biti na istom disku kao program, da se zamjena svodi na preimenovanje.
    fn radna_mapa(&self) -> PathBuf {
        match self.sustav {
            Sustav::MacOs => self.korijen.with_file_name(RADNA_MAPA),
            Sustav::Windows | Sustav::Linux => self.korijen.join(RADNA_MAPA),
        }
    }

    /// Parovi (novo, staro) koje treba zamijeniti, iz raspakiranog paketa.
    fn parovi(&self, raspakirano: &Path) -> Result<Vec<(PathBuf, PathBuf)>, Greska> {
        match self.sustav {
            Sustav::MacOs => {
                let novo = raspakirano.join("TerraSimple.app");
                provjeri_program(&novo.join("Contents/MacOS/TerraSimple"))?;
                Ok(vec![(novo, self.korijen.clone())])
            }
            Sustav::Windows | Sustav::Linux => {
                let mapa = jedina_mapa(raspakirano)?;
                provjeri_program(&mapa.join(ime_programa(self.sustav)))?;
                sadrzaj_mape(&mapa, &self.korijen)
            }
        }
    }
}

fn paket_maca(exe: &Path) -> Option<PathBuf> {
    let macos = exe.parent()?;
    let contents = macos.parent()?;
    let app = contents.parent()?;
    let valja = ime(macos) == Some("MacOS")
        && ime(contents) == Some("Contents")
        && app.extension().is_some_and(|e| e == "app");
    valja.then(|| app.to_path_buf())
}

fn mapa_programa(exe: &Path, sustav: Sustav) -> Option<PathBuf> {
    let mapa = exe.parent()?;
    let valja = ime(exe) == Some(ime_programa(sustav)) && mapa.join("Dozvole").is_dir();
    valja.then(|| mapa.to_path_buf())
}

fn provjeri_program(staza: &Path) -> Result<(), Greska> {
    if staza.is_file() {
        Ok(())
    } else {
        Err(crate::greska::ostecen(format!(
            "u paketu nema {}",
            staza.display()
        )))
    }
}

fn jedina_mapa(raspakirano: &Path) -> Result<PathBuf, Greska> {
    let stavke: Vec<PathBuf> = fs::read_dir(raspakirano)
        .map_err(crate::greska::ostecen)?
        .filter_map(|s| s.ok().map(|s| s.path()))
        .collect();
    match stavke.as_slice() {
        [mapa] if mapa.is_dir() => Ok(mapa.clone()),
        _ => Err(crate::greska::ostecen("paket nema jednu mapu programa")),
    }
}

fn sadrzaj_mape(mapa: &Path, korijen: &Path) -> Result<Vec<(PathBuf, PathBuf)>, Greska> {
    let mut parovi = Vec::new();
    for stavka in fs::read_dir(mapa).map_err(crate::greska::ostecen)? {
        let stavka = stavka.map_err(crate::greska::ostecen)?;
        parovi.push((stavka.path(), korijen.join(stavka.file_name())));
    }
    parovi.sort();
    Ok(parovi)
}

/// Raspakirava provjereni paket i njime zamjenjuje program na `mjesto`.
///
/// # Errors
/// Kad paket nije ispravan ili zamjena ne uspije; program tada ostaje kakav je bio.
pub fn ugradi(sadrzaj: &[u8], ime_paketa: &str, mjesto: &Mjesto) -> Result<(), Greska> {
    let rad = mjesto.radna_mapa();
    let ishod = pripremi_mapu(&rad, mjesto).and_then(|()| {
        raspakiraj(sadrzaj, ime_paketa, &rad)?;
        zamijeni(&mjesto.parovi(&rad)?, mjesto)
    });
    // Ostatak radne mape ne smeta: briše ga i idući pokretanje (`pocisti_ostatke`).
    let _ = obrisi(&rad); // greška brisanja ne mijenja ishod nadogradnje
    ishod
}

fn pripremi_mapu(rad: &Path, mjesto: &Mjesto) -> Result<(), Greska> {
    obrisi(rad)
        .and_then(|()| fs::create_dir_all(rad))
        .map_err(|e| zamjena(mjesto, &e))
}

fn zamjena(mjesto: &Mjesto, e: &io::Error) -> Greska {
    Greska::Zamjena {
        mjesto: mjesto.korijen.clone(),
        detalji: e.to_string(),
    }
}

fn stara(staza: &Path) -> PathBuf {
    let mut ime = staza.as_os_str().to_owned();
    ime.push(NASTAVAK_STAROGA);
    PathBuf::from(ime)
}

fn obrisi(staza: &Path) -> io::Result<()> {
    let ishod = match fs::symlink_metadata(staza) {
        Ok(m) if m.is_dir() => fs::remove_dir_all(staza),
        Ok(_) => fs::remove_file(staza),
        Err(e) => Err(e),
    };
    match ishod {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        ishod => ishod,
    }
}

/// Stavlja `novo` na mjesto `staro`. Vraća je li staro postojalo (i sad je `… .stara`).
fn zamijeni_jedno(novo: &Path, staro: &Path) -> io::Result<bool> {
    let kopija = stara(staro);
    obrisi(&kopija)?;
    let postoji = fs::symlink_metadata(staro).is_ok();
    if postoji {
        fs::rename(staro, &kopija)?;
    }
    if let Err(e) = fs::rename(novo, staro) {
        if postoji {
            let _ = fs::rename(&kopija, staro); // vraćanje; izvorna greška je važnija
        }
        return Err(e);
    }
    Ok(postoji)
}

fn vrati(gotovo: &[(PathBuf, bool)]) {
    for (staro, postojalo) in gotovo.iter().rev() {
        let _ = obrisi(staro); // novo se uklanja da staro može natrag
        if *postojalo {
            let _ = fs::rename(stara(staro), staro); // najbolje što se može; greška je već javljena
        }
    }
}

fn zamijeni(parovi: &[(PathBuf, PathBuf)], mjesto: &Mjesto) -> Result<(), Greska> {
    let mut gotovo = Vec::new();
    for (novo, staro) in parovi {
        match zamijeni_jedno(novo, staro) {
            Ok(postojalo) => gotovo.push((staro.clone(), postojalo)),
            Err(e) => {
                vrati(&gotovo);
                return Err(zamjena(mjesto, &e));
            }
        }
    }
    for (staro, _) in &gotovo {
        let _ = obrisi(&stara(staro)); // na Windowsu program koji radi ostaje do idućeg pokretanja
    }
    Ok(())
}

/// Briše ostatke prethodne nadogradnje: stare dijelove i radnu mapu.
pub fn pocisti_ostatke(mjesto: &Mjesto) {
    let _ = obrisi(&mjesto.radna_mapa()); // ostatak ne smeta radu programa
    match mjesto.sustav {
        Sustav::MacOs => {
            let _ = obrisi(&stara(&mjesto.korijen)); // isto
        }
        Sustav::Windows | Sustav::Linux => pocisti_mapu(&mjesto.korijen),
    }
}

fn pocisti_mapu(mapa: &Path) {
    let Ok(stavke) = fs::read_dir(mapa) else {
        return;
    };
    for stavka in stavke.flatten() {
        let staza = stavka.path();
        if ime(&staza).is_some_and(|i| i.ends_with(NASTAVAK_STAROGA)) {
            let _ = obrisi(&staza); // ostatak ne smeta radu programa
        }
    }
}

/// Naredba koja pokreće novoinstalirani program. Na Macu `open -n`, jer bi obični `open`
/// samo prebacio na program koji se još zatvara.
#[must_use]
pub fn naredba_ponovnog_pokretanja(mjesto: &Mjesto) -> Command {
    match mjesto.sustav {
        Sustav::MacOs => {
            let mut naredba = Command::new("open");
            naredba.arg("-n").arg(&mjesto.korijen);
            naredba
        }
        Sustav::Windows | Sustav::Linux => {
            Command::new(mjesto.korijen.join(ime_programa(mjesto.sustav)))
        }
    }
}

#[cfg(test)]
#[path = "ugradnja_testovi.rs"]
mod testovi;
