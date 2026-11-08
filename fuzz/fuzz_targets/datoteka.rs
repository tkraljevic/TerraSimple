#![no_main]
use std::{path::PathBuf, sync::OnceLock};
static STAZA: OnceLock<PathBuf> = OnceLock::new();
libfuzzer_sys::fuzz_target!(|b: &[u8]| {
    let p = STAZA.get_or_init(|| {
        let korijen = std::env::var_os("TERRASIMPLE_FUZZ_RAD")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("rad"));
        let d = korijen.join(std::process::id().to_string());
        std::fs::create_dir_all(&d).unwrap();
        d.join("ulaz.gpkg")
    });
    // Dnevnik SQLitea iz prošlog ulaza ne smije ući u sljedeći, inače pad nije ponovljiv.
    for nastavak in ["-wal", "-shm", "-journal"] {
        let mut s = p.clone().into_os_string();
        s.push(nastavak);
        let _ = std::fs::remove_file(s);
    }
    std::fs::write(p, b).unwrap();
    // Greške SQLitea i modela dopuštene su; panika, zastoj i rast memorije nisu.
    if let Ok(mut d) = geopaket::Datoteka::otvori(p) {
        drop(d.ucitaj());
    }
});
