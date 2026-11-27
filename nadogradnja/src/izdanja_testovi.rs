use super::*;

const PREFIKS: &str = "https://github.com/tkraljevic/TerraSimple/releases/download/";

fn datoteka(ime: &str) -> serde_json::Value {
    serde_json::json!({
        "name": ime,
        "size": 1234,
        "browser_download_url": format!("{PREFIKS}x/{ime}"),
    })
}

fn izdanje(oznaka: &str, draft: bool, imena: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "tag_name": oznaka,
        "draft": draft,
        "prerelease": true,
        "html_url": format!("https://github.com/tkraljevic/TerraSimple/releases/tag/{oznaka}"),
        "assets": imena.iter().map(|i| datoteka(i)).collect::<Vec<_>>(),
    })
}

fn izdanja(vrijednost: serde_json::Value) -> Vec<Izdanje> {
    serde_json::from_value(vrijednost).expect("popis izdanja")
}

const TRENUTNA: Inacica = Inacica(0, 0, 8);

#[test]
fn imena_paketa_su_kao_u_skripti_izdanja() {
    let v = Inacica(0, 0, 9);
    assert_eq!(
        Sustav::MacOs.ime_paketa(v),
        "TerraSimple-0.0.9-macos-universal.zip"
    );
    assert_eq!(
        Sustav::Windows.ime_paketa(v),
        "TerraSimple-0.0.9-windows-x86_64.zip"
    );
    assert_eq!(
        Sustav::Linux.ime_paketa(v),
        "TerraSimple-0.0.9-linux-x86_64.tar.gz"
    );
    #[cfg(target_os = "macos")]
    assert_eq!(Sustav::trenutni(), Sustav::MacOs);
    #[cfg(windows)]
    assert_eq!(Sustav::trenutni(), Sustav::Windows);
    #[cfg(not(any(target_os = "macos", windows)))]
    assert_eq!(Sustav::trenutni(), Sustav::Linux);
}

#[test]
fn nudi_najnovije_izdanje_s_paketom_i_potpisom() {
    let popis = izdanja(serde_json::json!([
        izdanje(
            "v0.0.9",
            false,
            &[
                "TerraSimple-0.0.9-macos-universal.zip",
                "TerraSimple-0.0.9-macos-universal.zip.sig",
                "TerraSimple-0.0.9-windows-x86_64.zip",
            ]
        ),
        izdanje("v0.0.8", false, &[]),
    ]));
    let Ishod::Dostupna(p) = odaberi(&popis, TRENUTNA, Sustav::MacOs, PREFIKS) else {
        panic!("treba ponuda");
    };
    assert_eq!(p.inacica, Inacica(0, 0, 9));
    assert_eq!(p.paket.ime, "TerraSimple-0.0.9-macos-universal.zip");
    assert_eq!(p.potpis.ime, "TerraSimple-0.0.9-macos-universal.zip.sig");
    assert!(p.stranica.ends_with("/v0.0.9"));
    // Za Windows nema potpisa, pa se izdanje preuzima ručno.
    assert_eq!(
        odaberi(&popis, TRENUTNA, Sustav::Windows, PREFIKS),
        Ishod::BezPaketa {
            inacica: Inacica(0, 0, 9),
            stranica: p.stranica.clone()
        }
    );
}

#[test]
fn nacrt_starije_i_neispravne_oznake_se_ne_nude() {
    let popis = izdanja(serde_json::json!([
        izdanje("v0.1.0", true, &[]),
        izdanje("nesto", false, &[]),
        izdanje("v0.0.7", false, &[]),
        izdanje("v0.0.8", false, &[]),
    ]));
    assert_eq!(
        odaberi(&popis, TRENUTNA, Sustav::Linux, PREFIKS),
        Ishod::Najnovija(TRENUTNA)
    );
    assert_eq!(
        odaberi(&[], TRENUTNA, Sustav::Linux, PREFIKS),
        Ishod::Najnovija(TRENUTNA)
    );
}

#[test]
fn paket_s_tude_adrese_ili_pod_starim_imenom_se_ne_nudi() {
    let mut tudje = izdanje(
        "v0.0.9",
        false,
        &[
            "TerraSimple-0.0.9-linux-x86_64.tar.gz",
            "TerraSimple-0.0.9-linux-x86_64.tar.gz.sig",
        ],
    );
    tudje["assets"][0]["browser_download_url"] = "https://primjer.invalid/paket.tar.gz".into();
    // Paket 0.0.8 objavljen pod oznakom 0.0.9 nema ime koje se traži.
    let staro_ime = izdanje(
        "v0.0.9",
        false,
        &[
            "TerraSimple-0.0.8-linux-x86_64.tar.gz",
            "TerraSimple-0.0.8-linux-x86_64.tar.gz.sig",
        ],
    );
    for i in [tudje, staro_ime] {
        assert!(matches!(
            odaberi(
                &izdanja(serde_json::json!([i])),
                TRENUTNA,
                Sustav::Linux,
                PREFIKS
            ),
            Ishod::BezPaketa { .. }
        ));
    }
    assert!(!na_stranici_izdanja(&format!("{PREFIKS}../../x"), PREFIKS));
}
