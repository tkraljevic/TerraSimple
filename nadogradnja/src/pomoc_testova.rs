//! Pomoć za testove: privremena mapa i izrada paketa.

use std::fs;
use std::io::{Cursor, Write};
use std::path::PathBuf;

pub(crate) struct Mapa(pub PathBuf);

impl Mapa {
    pub(crate) fn nova(naziv: &str) -> Self {
        let staza = std::env::temp_dir().join(format!(
            "terrasimple-nadogradnja-{naziv}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&staza); // ostatak prijašnjeg prekinutog testa
        fs::create_dir_all(&staza).expect("mapa testa");
        Self(staza)
    }
}

impl Drop for Mapa {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0); // čišćenje nakon testa
    }
}

/// Zip sa stavkama (ime, sadržaj, mod); `None` sadržaj je mapa.
pub(crate) fn zip(stavke: &[(&str, Option<&[u8]>, u32)]) -> Vec<u8> {
    let mut pisac = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (ime, sadrzaj, mod_) in stavke {
        let opcije = zip::write::SimpleFileOptions::default().unix_permissions(*mod_);
        match sadrzaj {
            Some(s) => {
                pisac.start_file(*ime, opcije).expect("stavka");
                pisac.write_all(s).expect("upis");
            }
            None => pisac.add_directory(*ime, opcije).expect("mapa"),
        }
    }
    pisac.finish().expect("zip").into_inner()
}

pub(crate) fn tar_gz(stavke: &[(&str, &[u8], u32)]) -> Vec<u8> {
    let mut pisac = tar::Builder::new(flate2::write::GzEncoder::new(
        Vec::new(),
        flate2::Compression::fast(),
    ));
    for (ime, sadrzaj, mod_) in stavke {
        let mut zaglavlje = tar::Header::new_gnu();
        zaglavlje.set_size(sadrzaj.len() as u64);
        zaglavlje.set_mode(*mod_);
        zaglavlje.set_cksum();
        pisac
            .append_data(&mut zaglavlje, ime, *sadrzaj)
            .expect("stavka");
    }
    pisac.into_inner().expect("tar").finish().expect("gz")
}
