#![cfg(test)]
use super::*;

fn pripremi(korijen: &Path) -> String {
    fs::create_dir_all(korijen.join("quality")).unwrap();
    fs::create_dir_all(korijen.join("src")).unwrap();
    fs::write(korijen.join("quality/config.json"), "{\"iznimke\":[]}").unwrap();
    fs::write(korijen.join("quality/deny.toml"), "").unwrap();
    fs::write(
        korijen.join("quality/baseline.json"),
        "{\"schema\":1,\"mjerenja\":{}}",
    )
    .unwrap();
    fs::write(korijen.join("src/lib.rs"), "pub fn a() { 1; }\n").unwrap();
    let alat = korijen.join(format!("alat{}", env::consts::EXE_SUFFIX));
    let status = std::process::Command::new("rustc")
        .args(["--edition=2024", "--crate-name", "probni_alat"])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/pomocni/alat.rs"))
        .arg("-o")
        .arg(&alat)
        .status()
        .unwrap();
    assert!(status.success());
    alat.to_string_lossy().into_owned()
}

#[test]
fn cijeli_tijek_ne_skriva_pad_i_ne_pomice_referencu() {
    let korijen = env::temp_dir().join(format!("terrasimple-naredba-{}", std::process::id()));
    let cargo = pripremi(&korijen);
    let korijen = alati::bez_windows_predmetka(fs::canonicalize(korijen).unwrap());
    let zahtjev = |naredba: &str| Zahtjev {
        naredba: naredba.into(),
        mapa: korijen.clone(),
        korijen: korijen.clone(),
        cargo: cargo.clone(),
    };
    assert!(
        izvrsi(zahtjev("provjera-duboka"))
            .unwrap_err()
            .contains("kritičnih paketa")
    );
    assert!(izvrsi(zahtjev("krivo")).is_err());
    let greska = izvrsi(zahtjev("provjera")).unwrap_err();
    assert!(greska.contains("Nema referentnog"), "{greska}");
    izvrsi(zahtjev("zapamti")).unwrap();
    fs::write(korijen.join("bez-nightly"), "").unwrap();
    izvrsi(zahtjev("provjera")).unwrap();
    fs::remove_file(korijen.join("bez-nightly")).unwrap();
    let prije = fs::read(korijen.join("quality/baseline.json")).unwrap();
    fs::write(korijen.join("pad-testa"), "").unwrap();
    assert!(
        izvrsi(zahtjev("provjera"))
            .unwrap_err()
            .contains("Namjeran pad")
    );
    assert_eq!(
        prije,
        fs::read(korijen.join("quality/baseline.json")).unwrap()
    );
    fs::remove_file(korijen.join("pad-testa")).unwrap();
    fs::write(korijen.join("pad-alata"), "").unwrap();
    assert!(
        izvrsi(zahtjev("provjera"))
            .unwrap_err()
            .contains("Instalirajte")
    );
    fs::remove_file(korijen.join("pad-alata")).unwrap();
    fs::write(
        korijen.join("src/lib.rs"),
        "pub fn a() { }\npub fn b() {}\n",
    )
    .unwrap();
    assert!(
        izvrsi(zahtjev("provjera"))
            .unwrap_err()
            .contains("nije prošla")
    );
    fs::write(
        korijen.join("quality/baseline.json"),
        "{\"schema\":2,\"mjerenja\":{}}",
    )
    .unwrap();
    assert!(
        izvrsi(zahtjev("provjera"))
            .unwrap_err()
            .contains("drugu metodu")
    );
    fs::remove_dir_all(korijen).unwrap();
}

#[test]
fn referenca_se_provjerava_prije_usporedbe() {
    let m = Mjerenje::novo("a", vec![pravila::Funkcija::nova("a", 1, 1.0, false)]);
    let mut r = Referenca {
        schema: 1,
        ..Referenca::default()
    };
    r.mjerenja.insert("a".into(), m.clone());
    assert!(valjana_referenca(&r, &m));
    r.mjerenja.get_mut("a").unwrap().funkcije[0].rizik = 100.0;
    assert!(!valjana_referenca(&r, &m));
}

#[test]
fn argumenti_prihvacaju_samo_poznati_oblik() {
    let korijen = env::temp_dir();
    let procitaj = |args: &[&str]| {
        procitaj_argumente(korijen.clone(), args.iter().map(std::ffi::OsString::from))
    };
    assert!(procitaj(&[]).is_err());
    assert!(procitaj(&["provjera"]).is_ok());
    assert!(procitaj(&["provjera", "--mapa", &korijen.to_string_lossy()]).is_ok());
    assert!(procitaj(&["provjera", "--mapa"]).is_err());
    assert!(procitaj(&["provjera", "--krivo"]).is_err());
    assert!(
        procitaj(&[
            "provjera",
            "--mapa",
            &korijen.join("terrasimple-nepostoji").to_string_lossy()
        ])
        .is_err()
    );
    assert!(procitaj(&["provjera", "--mapa", &korijen.to_string_lossy(), "visak"]).is_err());
}

#[test]
fn duboka_povezuje_alat_iznimke_i_padove() {
    let korijen = env::temp_dir().join(format!("terrasimple-duboka-{}", std::process::id()));
    let cargo = pripremi(&korijen);
    fs::write(korijen.join("kritican"), "").unwrap();
    let zahtjev = || Zahtjev {
        naredba: "provjera-duboka".into(),
        mapa: korijen.clone(),
        korijen: korijen.clone(),
        cargo: cargo.clone(),
    };
    izvrsi(zahtjev()).unwrap();
    let fuzz_zahtjev = || Zahtjev {
        naredba: "fuzz".into(),
        mapa: korijen.clone(),
        korijen: korijen.clone(),
        cargo: cargo.clone(),
    };
    izvrsi(fuzz_zahtjev()).unwrap();
    for cilj in ["geometrija", "qml", "datoteka"] {
        assert!(korijen.join("fuzz/corpus").join(cilj).is_dir(), "{cilj}");
        assert!(korijen.join("fuzz/artifacts").join(cilj).is_dir(), "{cilj}");
    }
    fs::write(korijen.join("bez-nightly"), "").unwrap();
    izvrsi(zahtjev()).unwrap();
    assert!(izvrsi(fuzz_zahtjev()).is_err());
    fs::remove_file(korijen.join("bez-nightly")).unwrap();
    fs::write(korijen.join("pad-fuzza"), "").unwrap();
    assert!(izvrsi(fuzz_zahtjev()).unwrap_err().contains("Fuzz"));
    assert!(izvrsi(zahtjev()).unwrap_err().contains("Fuzz"));
    fs::remove_file(korijen.join("pad-fuzza")).unwrap();
    fs::write(korijen.join("ishod"), "MissedMutant").unwrap();
    fs::write(korijen.join("kod"), "2").unwrap();
    assert!(izvrsi(zahtjev()).unwrap_err().contains("replace a"));
    fs::write(korijen.join("quality/config.json"), r#"{"iznimke":[{"kljuc":"mutacija:src/lib.rs:1: replace a with true","razlog":"Pregledano.","odgovorna_osoba":"Održavatelj","rok":"2099-01-01"}]}"#).unwrap();
    izvrsi(zahtjev()).unwrap();
    fs::write(korijen.join("kod"), "1").unwrap();
    assert!(
        izvrsi(zahtjev())
            .unwrap_err()
            .contains("nije završio valjano")
    );
    fs::write(korijen.join("ishod"), "Timeout").unwrap();
    assert!(izvrsi(zahtjev()).is_err());
    fs::write(korijen.join("pad-alata"), "").unwrap();
    assert!(izvrsi(zahtjev()).is_err());
    fs::remove_dir_all(korijen).unwrap();
}

#[test]
fn windows_predmetak_se_mice_samo_sa_staze_na_disku() {
    let staza = |s: &str| alati::bez_windows_predmetka(PathBuf::from(s));
    assert_eq!(staza(r"\\?\D:\a\b"), PathBuf::from(r"D:\a\b"));
    assert_eq!(staza(r"D:\a\b"), PathBuf::from(r"D:\a\b"));
    assert_eq!(staza("/tmp/a"), PathBuf::from("/tmp/a"));
    let mrezna = r"\\?\UNC\posluzitelj\a";
    assert_eq!(staza(mrezna), PathBuf::from(mrezna));
}
