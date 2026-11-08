//! Gumb „Otvori u QGIS-u”: pronalazak QGIS-a i otvaranje spremljenog projekta. Testovi zamjenjuju
//! pravi QGIS lažnim, pa nikad ne otvaraju njegov prozor.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Pokretanje QGIS-a s projektom iz spremljenog GeoPackagea, bez čekanja da se zatvori.
pub trait PokretacQgisa: Send + Sync {
    /// Vraća poruku za korisnika kad QGIS nije pronađen ili se nije dao pokrenuti.
    fn otvori(&self, datoteka: &Path) -> Result<(), String>;
}

/// Pravi QGIS na ovom računalu, preko paketa `qgis`.
#[derive(Debug, Default)]
pub struct SustavskiQgis;

impl PokretacQgisa for SustavskiQgis {
    fn otvori(&self, datoteka: &Path) -> Result<(), String> {
        let q = qgis::pronadji(None).map_err(|e| e.to_string())?;
        // Otvara se projekt iz datoteke, da QGIS pokaže nazive, mape i redoslijed slojeva.
        // Proces se ne čeka; QGIS ostaje otvoren i kad se TerraSimple zatvori.
        q.otvori_projekt(datoteka, geopaket::NAZIV_PROJEKTA)
            .map(drop)
            .map_err(|e| e.to_string())
    }
}

/// Lažni QGIS za testove: pamti tražene datoteke, a može i javiti grešku.
#[derive(Clone, Default)]
pub struct LazniQgis {
    pub otvoreno: Arc<Mutex<Vec<PathBuf>>>,
    pub greska: Arc<Mutex<Option<String>>>,
}

impl PokretacQgisa for LazniQgis {
    fn otvori(&self, datoteka: &Path) -> Result<(), String> {
        if let Some(g) = self.greska.lock().ok().and_then(|g| g.clone()) {
            return Err(g);
        }
        if let Ok(mut o) = self.otvoreno.lock() {
            o.push(datoteka.to_path_buf());
        }
        Ok(())
    }
}
