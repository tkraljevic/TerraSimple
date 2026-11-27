use super::*;

fn javni(tajni: &[u8]) -> Vec<u8> {
    STANDARD
        .decode(javni_kljuc_iz_tajnog(tajni).expect("javni ključ"))
        .expect("base64")
}

#[test]
fn ispravan_potpis_prolazi() {
    let tajni = novi_tajni_kljuc().expect("ključ");
    let potpis =
        potpisi(&tajni, "TerraSimple-0.0.9-macos-universal.zip", b"paket").expect("potpis");
    let s_retkom = format!("{potpis}\n");
    provjeri_potpis(
        &javni(&tajni),
        "TerraSimple-0.0.9-macos-universal.zip",
        b"paket",
        s_retkom.as_bytes(),
    )
    .expect("potpis vrijedi");
}

/// Javni ključ, ime paketa, sadržaj i potpis.
type Slucaj<'a> = (&'a [u8], &'a str, &'a [u8], &'a [u8]);

#[test]
fn promijenjen_paket_ime_ili_kljuc_ne_prolaze() {
    let tajni = novi_tajni_kljuc().expect("ključ");
    let drugi = novi_tajni_kljuc().expect("ključ");
    let ime = "TerraSimple-0.0.9-linux-x86_64.tar.gz";
    let potpis = potpisi(&tajni, ime, b"paket").expect("potpis");
    let slucajevi: [Slucaj; 5] = [
        (&javni(&tajni), ime, b"paket!", potpis.as_bytes()),
        (
            &javni(&tajni),
            "TerraSimple-0.0.10-linux-x86_64.tar.gz",
            b"paket",
            potpis.as_bytes(),
        ),
        (&javni(&drugi), ime, b"paket", potpis.as_bytes()),
        (&javni(&tajni), ime, b"paket", b"nije base64 !"),
        (&javni(&tajni), ime, b"paket", &[0xff, 0xfe]),
    ];
    for (kljuc, ime, sadrzaj, potpis) in slucajevi {
        assert!(matches!(
            provjeri_potpis(kljuc, ime, sadrzaj, potpis),
            Err(Greska::KrivPotpis)
        ));
    }
}

#[test]
fn neispravan_tajni_kljuc_javlja_gresku() {
    assert!(javni_kljuc_iz_tajnog(b"nije kljuc").is_err());
    assert!(potpisi(b"nije kljuc", "x", b"y").is_err());
}
