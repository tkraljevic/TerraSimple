//! Spremanje projekta u GeoPackage, bez ovisnosti o sučelju.
#![forbid(unsafe_code)]
mod citanje;
mod dimenzije;
mod fid;
mod geometrija;
mod pisanje;
mod qgis_projekt;
mod qgis_stari;
mod qgis_sustavi;
mod shema;
mod stil;
use rusqlite::Connection;
use slojevi::Projekt;
use std::path::Path;
pub type Greska = Box<dyn std::error::Error + Send + Sync>;
pub type Rezultat<T> = Result<T, Greska>;

/// Datoteku je od učitavanja promijenio drugi program, npr. QGIS. Spremanje se odbija, jer
/// bi pregazilo njegove izmjene. Prozor je prepoznaje po vrsti, ne po tekstu.
#[derive(Debug)]
pub struct DatotekaPromijenjena;

impl std::fmt::Display for DatotekaPromijenjena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Datoteku je u međuvremenu promijenio drugi program.")
    }
}

impl std::error::Error for DatotekaPromijenjena {}

/// Ime QGIS projekta koji se pri svakom spremanju zapiše u GeoPackage.
pub use qgis_projekt::NAZIV_PROJEKTA;
/// Isključivo za zasebne fuzz ciljeve; nije sučelje programa.
#[cfg(feature = "fuzz")]
#[doc(hidden)]
pub mod fuzz {
    pub use crate::geometrija::procitaj as geometrija;
    pub use crate::stil::procitaj as stil;
}

pub struct Datoteka {
    veza: Connection,
    prethodni: Projekt,
    verzija: i64,
    procitano: bool,
    pub upozorenja: Vec<String>,
    pub trajanje_upisa: std::time::Duration,
}
impl Datoteka {
    pub fn otvori(staza: impl AsRef<Path>) -> Result<Self, Greska> {
        let nova = !staza.as_ref().exists();
        let veza = otvori_vezu(staza)?;
        Self::iz_veze(veza).map(|mut d| {
            d.procitano = nova;
            d
        })
    }
    fn iz_veze(veza: Connection) -> Result<Self, Greska> {
        let verzija = veza.query_row("PRAGMA data_version", [], |r| r.get(0))?;
        Ok(Self {
            veza,
            prethodni: Projekt::default(),
            verzija,
            procitano: true,
            upozorenja: Vec::new(),
            trajanje_upisa: std::time::Duration::ZERO,
        })
    }
    pub fn ucitaj(&mut self) -> Result<Projekt, Greska> {
        self.ucitaj_iznad(0)
    }
    /// Objekti bez zapisanog broja dobiju brojeve veće od `najmanji`.
    fn ucitaj_iznad(&mut self, najmanji: u64) -> Result<Projekt, Greska> {
        let tx = self.veza.unchecked_transaction()?;
        let verzija = tx.query_row("PRAGMA data_version", [], |r| r.get(0))?;
        let (p, upozorenja) = citanje::projekt(&tx, najmanji)?;
        tx.commit()?;
        self.verzija = verzija;
        self.procitano = true;
        self.upozorenja = upozorenja;
        self.prethodni = p.clone();
        Ok(p)
    }
    pub fn spremi(&mut self, projekt: &Projekt) -> Result<(), Greska> {
        self.provjeri_spremanje(projekt)?;
        self.trajanje_upisa =
            pisanje::spremi(&mut self.veza, &self.prethodni, projekt, self.verzija)?;
        self.prethodni = projekt.clone();
        Ok(())
    }
    /// SQLite backup uključuje i spremljene WAL izmjene te nepoznate tablice i krivulje.
    ///
    /// Radi i nakon sukoba (Odluke (9. 11. 2026.)): nova datoteka dobije projekt kako ga vidi
    /// TerraSimple, a izvorna, koju je promijenio drugi program, ostaje netaknuta. Kopija tada
    /// nosi i tuđe izmjene, pa se objekti koje je drugi program dodao čitaju s brojevima iznad
    /// svih brojeva u projektu. Inače bi novi objekt iz TerraSimplea dobio isti broj kao tuđi
    /// i spremanje bi tuđi redak prepisalo umjesto da doda novi.
    pub fn spremi_kao(&mut self, staza: impl AsRef<Path>, projekt: &Projekt) -> Result<(), Greska> {
        self.provjeri_spremanje(projekt)?;
        if staza.as_ref().exists() {
            return Err("Odredišna datoteka već postoji.".into());
        }
        let mut druga = self.kopija(staza)?;
        druga.ucitaj_iznad(self.najmanji_novi_id(projekt))?;
        druga.spremi(projekt)
    }
    /// Druge greške pri provjeri izvorne datoteke ovdje se ne javljaju: kopija je već napravljena,
    /// a čitanje i spremanje kopije javit će svoju.
    fn najmanji_novi_id(&self, projekt: &Projekt) -> u64 {
        let promijenjena = pisanje::provjeri_verziju(&self.veza, self.verzija)
            .is_err_and(|e| e.is::<DatotekaPromijenjena>());
        if promijenjena {
            projekt.najveci_id()
        } else {
            0
        }
    }
    fn provjeri_spremanje(&self, p: &Projekt) -> Result<(), Greska> {
        if !self.procitano {
            return Err("Postojeću datoteku najprije učitajte prije spremanja.".into());
        }
        if !p.zapisi().slojevi.is_empty() && self.prethodni.zapisi().slojevi.is_empty() {
            return Err(
                "Projekt s izvornim zapisima spremite preko spremi_kao na izvornoj datoteci."
                    .into(),
            );
        }
        Ok(())
    }
    fn kopija(&self, staza: impl AsRef<Path>) -> Result<Self, Greska> {
        self.veza
            .backup("main", &staza, None::<fn(rusqlite::backup::Progress)>)
            .map_err(Into::into)
            .and_then(|()| Self::otvori(staza))
    }
}

fn otvori_vezu(staza: impl AsRef<Path>) -> Result<Connection, Greska> {
    let nova = !staza.as_ref().exists();
    let veza = Connection::open(staza)?;
    pripremi_vezu(&veza)
        .and_then(|()| pripremi_bazu(&veza, nova))
        .map(|()| veza)
}

fn pripremi_bazu(c: &Connection, nova: bool) -> Result<(), Greska> {
    if shema::postoji(c, "gpkg_contents")? {
        return Ok(());
    }
    if !nova {
        return Err("Postojeća datoteka nije GeoPackage.".into());
    }
    shema::stvori(c)
}

#[cfg(test)]
mod kvarovi;

fn pripremi_vezu(c: &Connection) -> Result<(), Greska> {
    c.busy_timeout(std::time::Duration::from_millis(250))
        .map_err(Into::into)
        .and_then(|()| geometrija::funkcije(c))
}
