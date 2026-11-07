use std::{collections::BTreeMap, fs};
use xtask::{alati, mjerenje::*, pravila::*};

#[test]
fn novi_paket_ne_mijenja_ime_radnog_prostora() {
    assert_eq!(ime_cilja(vec!["proba_koordinate"]), "proba_koordinate");
    assert_eq!(ime_cilja(vec!["koordinate", "xtask"]), "radni-prostor");
    assert_eq!(
        ime_cilja(vec!["geometrija", "koordinate", "xtask"]),
        "radni-prostor"
    );
}

#[test]
fn spaja_podrucja_i_instanciranja_bez_testova() {
    let mapa = std::env::temp_dir().join(format!("terrasimple-mjerenje-{}", std::process::id()));
    fs::create_dir_all(mapa.join("src")).unwrap();
    fs::create_dir_all(mapa.join("src/podmapa")).unwrap();
    fs::write(mapa.join("src/podmapa/prazno.rs"), "").unwrap();
    fs::write(mapa.join("src/podmapa/tekst.txt"), "").unwrap();
    let staza = mapa.join("src/lib.rs");
    fs::write(
        &staza,
        "pub fn racun() { }\n#[cfg(test)] mod testovi { fn a() {} }\n",
    )
    .unwrap();
    let meta = Metapodaci {
        workspace_members: vec!["a".into()],
        packages: vec![Paket {
            id: "a".into(),
            name: "koordinate".into(),
            manifest_path: mapa.join("Cargo.toml"),
            targets: vec![Cilj {
                kind: vec!["lib".into()],
                src_path: staza.clone(),
            }],
        }],
    };
    let llvm = Pokrivenost {
        data: vec![Podaci {
            functions: vec![
                LlvmFunkcija {
                    filenames: vec![staza.clone()],
                    regions: vec![[1, 16, 1, 19, 0, 0, 0, 0]],
                },
                LlvmFunkcija {
                    filenames: vec![staza.clone()],
                    regions: vec![[1, 16, 1, 19, 1, 0, 0, 0], [2, 40, 2, 41, 1, 0, 0, 0]],
                },
            ],
        }],
    };
    let m = izmjeri(&mapa, &meta, &llvm).unwrap();
    assert_eq!(m.funkcije.len(), 1);
    assert_eq!(m.funkcije[0].podrucja, 1);
    assert_eq!(m.pokrivenost, 1.0);
    assert!(m.funkcije[0].kriticna);
    let prazno = izmjeri(&mapa, &meta, &Pokrivenost { data: vec![] }).unwrap();
    assert_eq!(prazno.pokrivenost, 0.0);
    let json = mapa.join("mjerenje.json");
    alati::zapisi_json(&json, &m).unwrap();
    let procitano: Mjerenje = alati::citaj_json(&json).unwrap();
    assert_eq!(procitano.funkcije[0].kljuc, m.funkcije[0].kljuc);
    fs::write(&staza, "fn {").unwrap();
    assert!(
        izmjeri(&mapa, &meta, &llvm)
            .unwrap_err()
            .contains("nije moguće analizirati")
    );
    fs::write(&staza, "").unwrap();
    assert!(izmjeri(&mapa, &meta, &llvm).is_err());
    fs::remove_file(&staza).unwrap();
    assert!(izmjeri(&mapa, &meta, &llvm).is_err());
    fs::write(&json, "{").unwrap();
    assert!(alati::citaj_json::<Mjerenje>(&json).is_err());
    fs::remove_dir_all(mapa).unwrap();
}

#[test]
fn datoteke_i_nedostupan_alat_daju_opis() {
    let mapa = std::env::temp_dir();
    assert!(
        alati::pokreni(&mapa, "terrasimple-nepostojeci-alat", &[], "instalacija")
            .unwrap_err()
            .contains("instalacija")
    );
    assert!(
        alati::pokreni(&mapa, "rustc", &["--nepostojeca-opcija"], "rustup")
            .unwrap_err()
            .contains("nije prošla")
    );
    assert!(alati::pokreni(&mapa, "rustc", &["--version"], "rustup").is_ok());
    assert!(alati::citaj_json::<Mjerenje>(&mapa.join("nepostoji-terrasimple.json")).is_err());
    assert!(
        alati::zapisi_json(
            &mapa.join("nema-terrasimple-mape/x.json"),
            &BTreeMap::<String, String>::new()
        )
        .is_err()
    );
}

#[test]
fn kriticni_moduli_ne_ovise_o_platformi() {
    assert!(kriticna("src/atributi.rs::spremi", "program"));
    assert!(kriticna("src/lib.rs::projekt::spremi", "program"));
    assert!(kriticna("src/main.rs::a", "proba_koordinate"));
    assert!(!kriticna("src/karta.rs::a", "program"));
}

#[test]
fn iznimke_su_obvezno_potpune_i_vremenski_ogranicene() {
    let danas = chrono::NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
    let valjana = r#"{"iznimke":[{"kljuc":"rizik:x","razlog":"Pregledano.","odgovorna_osoba":"Održavatelj","rok":"2026-11-07"}]}"#;
    let p: Postavke = serde_json::from_str(valjana).unwrap();
    assert!(provjeri_iznimke(&p, danas).is_ok());
    assert!(
        provjeri_iznimke(&p, danas.succ_opt().unwrap())
            .unwrap_err()
            .contains("istekla")
    );
    let losa = valjana.replace("Pregledano.", " ");
    assert!(provjeri_iznimke(&serde_json::from_str(&losa).unwrap(), danas).is_err());
    let losa = valjana.replace("2026-11-07", "2026-02-30");
    assert!(serde_json::from_str::<Postavke>(&losa).is_err());
    let mut f = Funkcija::nova("x", 11, 1.0, false);
    f.kljuc = "x".into();
    let m = Mjerenje::novo("a", vec![f]);
    let nalazi = usporedi(&m, None, &p);
    assert_eq!(nalazi.len(), 1);
    assert!(nalazi[0].kljuc.starts_with("slozenost:"));
}
