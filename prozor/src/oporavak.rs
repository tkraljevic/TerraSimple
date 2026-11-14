//! Automatsko spremanje u kopiju i oporavak nakon pada (Odluke (14. 11. 2026.)).
//!
//! Dok ima nespremljenih izmjena, program najviše svakih 30 s sprema kopiju projekta u mapu programa.
//! Projekt se mijenja tek na Spremi, pa se kopija ne sudara s QGIS-om. Svaki pokrenuti program drži
//! zaključanu svoju datoteku `.lock`. Kopija čija brava nije zaključana ostala je iza programa koji
//! se srušio, pa se pri pokretanju nudi da se vrati. Kopija drugog prozora koji još radi se ne nudi.

use serde::{Deserialize, Serialize};
use slojevi::Uredjivac;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

/// Koliko najmanje prolazi između dvije kopije.
pub const RAZMAK: Duration = Duration::from_secs(30);

/// Kopija koju je ostavio program koji se srušio.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ostatak {
    pub kopija: PathBuf,
    /// Datoteka projekta, ako je projekt bio spremljen barem jednom.
    pub izvor: Option<PathBuf>,
    /// Kad je kopija nastala, npr. „14. 11. 2026. 07:30”.
    pub vrijeme: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Opis {
    izvor: Option<PathBuf>,
    vrijeme: String,
}

pub struct Oporavak {
    mapa: PathBuf,
    id: String,
    /// Prazno samo u testu koji glumi pad programa.
    brava: Option<File>,
    /// Stanje projekta koje je zadnje kopirano ili spremljeno.
    zadnja_oznaka: u64,
    zadnja_kopija: Instant,
    posao: Option<JoinHandle<()>>,
    /// Ovaj program je napravio kopiju koja još nije obrisana.
    ima_kopiju: bool,
    /// Vraćena kopija iz koje je projekt otvoren, dok se ne spremi drugdje.
    vraceno: Option<PathBuf>,
}

fn staza(mapa: &Path, id: &str, nastavak: &str) -> PathBuf {
    mapa.join(format!("{id}.{nastavak}"))
}

/// Brava je slobodna samo ako je program koji ju je držao završio.
fn napustena(mapa: &Path, id: &str) -> bool {
    File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(staza(mapa, id, "lock"))
        .is_ok_and(|b| b.try_lock().is_ok())
}

fn procitaj_ostatak(mapa: &Path, opis: &Path) -> Option<Ostatak> {
    let id = opis.file_stem()?.to_string_lossy().into_owned();
    let kopija = staza(mapa, &id, "gpkg");
    if !kopija.exists() || !napustena(mapa, &id) {
        return None;
    }
    let o: Opis = serde_json::from_slice(&std::fs::read(opis).ok()?).ok()?;
    Some(Ostatak {
        kopija,
        izvor: o.izvor,
        vrijeme: o.vrijeme,
        id,
    })
}

fn vrijeme_sada() -> String {
    chrono::Local::now()
        .format("%-d. %-m. %Y. %H:%M")
        .to_string()
}

impl Oporavak {
    /// Mapa se stvara ako ne postoji. Bez nje ili bez brave automatskog spremanja nema.
    pub fn novi(mapa: PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(&mapa)?;
        let nano = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let id = format!("{}-{nano}", std::process::id());
        let brava = File::create(staza(&mapa, &id, "lock"))?;
        brava.try_lock().map_err(std::io::Error::other)?;
        Ok(Self {
            mapa,
            id,
            brava: Some(brava),
            zadnja_oznaka: 0,
            zadnja_kopija: Instant::now(),
            posao: None,
            ima_kopiju: false,
            vraceno: None,
        })
    }

    #[must_use]
    pub fn kopija(&self) -> PathBuf {
        staza(&self.mapa, &self.id, "gpkg")
    }

    /// Kopije koje su ostale iza srušenog programa, najnovija prva.
    #[must_use]
    pub fn ostaci(&self) -> Vec<Ostatak> {
        let Ok(popis) = std::fs::read_dir(&self.mapa) else {
            return Vec::new();
        };
        let mut ostaci: Vec<(std::time::SystemTime, Ostatak)> = popis
            .filter_map(Result::ok)
            .map(|d| d.path())
            .filter(|p| p.extension().is_some_and(|n| n == "json"))
            .filter_map(|p| procitaj_ostatak(&self.mapa, &p))
            .filter(|o| o.id != self.id)
            .map(|o| (vrijeme_izmjene(&o.kopija), o))
            .collect();
        ostaci.sort_by_key(|o| std::cmp::Reverse(o.0));
        ostaci.into_iter().map(|(_, o)| o).collect()
    }

    fn obrisi(&self, id: &str) {
        for nastavak in ["gpkg", "json", "lock", "u-tijeku"] {
            let _ = std::fs::remove_file(staza(&self.mapa, id, nastavak));
        }
    }

    /// Korisnik ne želi vratiti kopiju, pa je nema.
    pub fn odbaci(&self, o: &Ostatak) {
        self.obrisi(&o.id);
    }

    /// Kopija postaje vraćena kopija ovog programa, a ostatak srušenog nestaje.
    pub fn preuzmi(&mut self, o: &Ostatak) -> std::io::Result<PathBuf> {
        let cilj = staza(&self.mapa, &self.id, "vraceno.gpkg");
        std::fs::rename(&o.kopija, &cilj)?;
        self.obrisi(&o.id);
        self.vraceno = Some(cilj.clone());
        // Vraćeni rad dobiva svoju kopiju čim prođe razmak, da ga ne izgubi ni sljedeći pad.
        self.zadnja_oznaka = u64::MAX;
        Ok(cilj)
    }

    #[must_use]
    pub fn vraceno(&self) -> Option<&Path> {
        self.vraceno.as_deref()
    }

    fn posao_traje(&mut self) -> bool {
        if self.posao.as_ref().is_some_and(JoinHandle::is_finished) {
            self.pricekaj();
        }
        self.posao.is_some()
    }

    /// Glumi pad: brava se otpušta, a kopija ostaje.
    #[cfg(test)]
    pub(crate) fn padni(mut self) {
        self.pricekaj();
        self.brava = None;
    }

    /// Čeka da kopija koja se upravo piše bude gotova.
    pub fn pricekaj(&mut self) {
        if let Some(p) = self.posao.take() {
            let _ = p.join();
        }
    }

    /// Projekt spremljen u svoju datoteku više ne treba vraćenu kopiju.
    fn pusti_vraceno(&mut self, staza_projekta: Option<&Path>) {
        if staza_projekta.is_some()
            && let Some(v) = self.vraceno.take()
        {
            let _ = std::fs::remove_file(v);
        }
    }

    /// Poziva se u svakom okviru. Vraća je li upravo počela nova kopija.
    pub fn tik(&mut self, u: &Uredjivac, staza_projekta: Option<&Path>, sada: Instant) -> bool {
        self.pusti_vraceno(staza_projekta);
        if self.posao_traje() {
            return false;
        }
        let oznaka = u.oznaka_stanja();
        if !u.je_li_izmijenjen() {
            if std::mem::take(&mut self.ima_kopiju) {
                self.obrisi(&self.id.clone());
            }
            self.zadnja_oznaka = oznaka;
            return false;
        }
        let rano = sada.saturating_duration_since(self.zadnja_kopija) < RAZMAK;
        if oznaka == self.zadnja_oznaka || rano {
            return false;
        }
        self.zadnja_oznaka = oznaka;
        self.zadnja_kopija = sada;
        self.ima_kopiju = true;
        let izvor = staza_projekta.or(self.vraceno()).map(Path::to_path_buf);
        let projekt = u.projekt().clone();
        let (kopija, opis) = (self.kopija(), staza(&self.mapa, &self.id, "json"));
        self.posao = Some(std::thread::spawn(move || {
            if geopaket::spremi_kopiju(izvor.as_deref(), &kopija, &projekt).is_ok() {
                let o = Opis {
                    izvor,
                    vrijeme: vrijeme_sada(),
                };
                let _ = std::fs::write(opis, serde_json::to_vec(&o).unwrap_or_default());
            }
        }));
        true
    }
}

fn vrijeme_izmjene(p: &Path) -> std::time::SystemTime {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .unwrap_or(std::time::UNIX_EPOCH)
}

/// Program koji se uredno zatvara ne ostavlja kopiju: što nije spremljeno, korisnik je odbacio.
impl Drop for Oporavak {
    fn drop(&mut self) {
        self.pricekaj();
        if self.brava.is_none() {
            return;
        }
        self.obrisi(&self.id.clone());
        if let Some(v) = self.vraceno.take() {
            let _ = std::fs::remove_file(v);
        }
    }
}

/// Mapa programa za kopije: na Macu u `Library/Application Support`, na Windowsu u
/// `%LOCALAPPDATA%`, a na Linuxu u `~/.local/share`.
#[must_use]
pub fn zadana_mapa() -> Option<PathBuf> {
    mapa_za(std::env::consts::OS, |n| {
        std::env::var_os(n).map(PathBuf::from)
    })
}

/// Mapa za kopije na sustavu `os`, iz varijabli okoline koje daje `var`.
pub fn mapa_za(os: &str, var: impl Fn(&str) -> Option<PathBuf>) -> Option<PathBuf> {
    let baza = match os {
        "macos" => var("HOME").map(|h| h.join("Library/Application Support")),
        "windows" => var("LOCALAPPDATA"),
        _ => var("XDG_DATA_HOME").or_else(|| var("HOME").map(|h| h.join(".local/share"))),
    };
    baza.map(|b| b.join("TerraSimple").join("oporavak"))
}

/// Tekstovi za poruku: datoteka iz koje je rad i kad je kopija nastala.
#[must_use]
pub fn tekstovi_ostatka(
    katalog: &crate::poruke::KatalogPoruka,
    o: &Ostatak,
) -> [(&'static str, String); 2] {
    let izvor = o.izvor.as_deref().and_then(Path::file_name).map_or_else(
        || katalog.tekst("oporavak-novi-projekt"),
        |n| n.to_string_lossy().into_owned(),
    );
    [("izvor", izvor), ("vrijeme", o.vrijeme.clone())]
}
