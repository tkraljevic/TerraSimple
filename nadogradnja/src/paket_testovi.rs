use super::*;
use crate::pomoc_testova::{Mapa, tar_gz, zip};
use std::io::Write;

#[test]
fn zip_se_raspakira_bez_mape_macosx_i_s_dozvolama() {
    let m = Mapa::nova("zip");
    let paket = zip(&[
        ("TerraSimple.app/", None, 0o755),
        (
            "TerraSimple.app/Contents/MacOS/TerraSimple",
            Some(b"program"),
            0o755,
        ),
        ("TerraSimple.app/Contents/Info.plist", Some(b"plist"), 0o644),
        ("__MACOSX/TerraSimple.app/._Info.plist", Some(b"x"), 0o644),
    ]);
    raspakiraj(&paket, "TerraSimple-0.0.9-macos-universal.zip", &m.0).expect("raspakirano");
    let program = m.0.join("TerraSimple.app/Contents/MacOS/TerraSimple");
    assert_eq!(fs::read(&program).expect("program"), b"program");
    assert!(!m.0.join("__MACOSX").exists());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mod_ = fs::metadata(&program)
            .expect("dozvole")
            .permissions()
            .mode();
        assert_eq!(mod_ & 0o111, 0o111, "program mora ostati izvršan");
    }
}

#[test]
fn tar_gz_se_raspakira() {
    let m = Mapa::nova("tar");
    let paket = tar_gz(&[
        (
            "TerraSimple-0.0.9-linux-x86_64/TerraSimple",
            b"program",
            0o755,
        ),
        (
            "TerraSimple-0.0.9-linux-x86_64/Dozvole/LICENSE",
            b"EUPL",
            0o644,
        ),
    ]);
    raspakiraj(&paket, "TerraSimple-0.0.9-linux-x86_64.tar.gz", &m.0).expect("raspakirano");
    let mapa = m.0.join("TerraSimple-0.0.9-linux-x86_64");
    assert_eq!(
        fs::read(mapa.join("Dozvole/LICENSE")).expect("licenca"),
        b"EUPL"
    );
}

#[test]
fn staza_izvan_odredista_se_odbija() {
    let m = Mapa::nova("staza");
    assert!(sigurna_staza(&m.0, Path::new("../van")).is_err());
    assert!(sigurna_staza(&m.0, Path::new("/etc/van")).is_err());
    assert!(sigurna_staza(&m.0, Path::new("")).is_err());
    assert_eq!(
        sigurna_staza(&m.0, Path::new("./a/b")).expect("staza"),
        m.0.join("./a/b")
    );
    let paket = zip(&[("../van.txt", Some(b"x"), 0o644)]);
    assert!(matches!(
        raspakiraj(&paket, "p.zip", &m.0),
        Err(Greska::OstecenPaket(_))
    ));
    assert!(!m.0.parent().expect("roditelj").join("van.txt").exists());
}

#[test]
fn poveznica_i_ostecen_paket_se_odbijaju() {
    let m = Mapa::nova("poveznica");
    let mut pisac = zip::ZipWriter::new(Cursor::new(Vec::new()));
    pisac
        .add_symlink(
            "veza",
            "/etc/passwd",
            zip::write::SimpleFileOptions::default(),
        )
        .expect("poveznica");
    let paket = pisac.finish().expect("zip").into_inner();
    assert!(raspakiraj(&paket, "p.zip", &m.0).is_err());
    assert!(raspakiraj(b"nije zip", "p.zip", &m.0).is_err());
    assert!(raspakiraj(b"nije tar", "p.tar.gz", &m.0).is_err());

    let mut zaglavlje = tar::Header::new_gnu();
    zaglavlje.set_entry_type(tar::EntryType::Symlink);
    zaglavlje.set_size(0);
    let mut pisac = tar::Builder::new(Vec::new());
    pisac
        .append_link(&mut zaglavlje, "veza", "/etc/passwd")
        .expect("poveznica");
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    gz.write_all(&pisac.into_inner().expect("tar")).expect("gz");
    assert!(raspakiraj(&gz.finish().expect("gz"), "p.tar.gz", &m.0).is_err());
}

#[test]
fn granica_broja_stavki_i_velicine() {
    let m = Mapa::nova("granica");
    let mut g = Granica {
        bajtova: 0,
        stavki: NAJVISE_STAVKI,
    };
    assert!(g.jos_jedna().is_err());
    let mut g = Granica {
        bajtova: NAJVISE_RASPAKIRANO - 2,
        stavki: 0,
    };
    assert!(g.upisi(&b"tri"[..], &m.0.join("a")).is_err());
    assert!(g.upisi(&b"dv"[..], &m.0.join("b")).is_ok());
    assert_eq!(g.bajtova, NAJVISE_RASPAKIRANO);
}
