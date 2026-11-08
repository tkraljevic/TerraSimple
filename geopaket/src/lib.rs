//! Spremanje projekta u GeoPackage, bez ovisnosti o sučelju.
#![forbid(unsafe_code)]
mod citanje;
mod dimenzije;
mod fid;
mod geometrija;
mod pisanje;
mod shema;
mod stil;
use rusqlite::Connection;
use slojevi::Projekt;
use std::path::Path;
pub type Greska = Box<dyn std::error::Error + Send + Sync>;
pub type Rezultat<T> = Result<T, Greska>;

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
        let tx = self.veza.unchecked_transaction()?;
        let verzija = tx.query_row("PRAGMA data_version", [], |r| r.get(0))?;
        let (p, upozorenja) = citanje::projekt(&tx)?;
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
    pub fn spremi_kao(&mut self, staza: impl AsRef<Path>, projekt: &Projekt) -> Result<(), Greska> {
        self.provjeri_spremanje(projekt)?;
        if staza.as_ref().exists() {
            return Err("Odredišna datoteka već postoji.".into());
        }
        pisanje::provjeri_verziju(&self.veza, self.verzija)?;
        let mut druga = self.kopija(staza)?;
        druga.ucitaj()?;
        druga.spremi(projekt)
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
            .and_then(|()| pisanje::provjeri_verziju(&self.veza, self.verzija))
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
