use super::*;
use crate::pomoc_testova::{Mapa, tar_gz, zip};

fn napravi(staza: &Path, sadrzaj: &[u8]) {
    fs::create_dir_all(staza.parent().expect("roditelj")).expect("mapa");
    fs::write(staza, sadrzaj).expect("datoteka");
}

fn mac_program(m: &Mapa) -> PathBuf {
    let exe =
        m.0.join("Programi/TerraSimple.app/Contents/MacOS/TerraSimple");
    napravi(&exe, b"stari");
    exe
}

fn mapa_programa_na(m: &Mapa, sustav: Sustav) -> PathBuf {
    let exe = m.0.join("TerraSimple-0.0.8").join(ime_programa(sustav));
    napravi(&exe, b"stari");
    napravi(
        &exe.with_file_name("Dozvole").join("LICENSE"),
        b"stara licenca",
    );
    exe
}

#[test]
fn mjesto_se_prepoznaje_samo_u_paketu_izdanja() {
    let m = Mapa::nova("mjesto");
    let exe = mac_program(&m);
    let mjesto = Mjesto::za_program(&exe, Sustav::MacOs).expect("paket");
    assert_eq!(mjesto.korijen(), m.0.join("Programi/TerraSimple.app"));
    for sustav in [Sustav::Windows, Sustav::Linux] {
        let exe = mapa_programa_na(&m, sustav);
        let mjesto = Mjesto::za_program(&exe, sustav).expect("mapa");
        assert_eq!(mjesto.korijen(), m.0.join("TerraSimple-0.0.8"));
    }
    // Program izgrađen iz izvornog koda (target/debug/prozor) ne nadograđuje se sam.
    let razvoj = m.0.join("target/debug/prozor");
    napravi(&razvoj, b"x");
    for sustav in [Sustav::MacOs, Sustav::Windows, Sustav::Linux] {
        assert!(matches!(
            Mjesto::za_program(&razvoj, sustav),
            Err(Greska::NijeIzPaketa(_))
        ));
    }
    assert!(Mjesto::za_program(Path::new("TerraSimple"), Sustav::MacOs).is_err());
}

#[test]
fn mac_paket_zamjenjuje_cijeli_app() {
    let m = Mapa::nova("mac");
    let exe = mac_program(&m);
    napravi(&exe.with_file_name("ostatak"), b"samo u starom");
    let mjesto = Mjesto::za_program(&exe, Sustav::MacOs).expect("paket");
    let paket = zip(&[
        (
            "TerraSimple.app/Contents/MacOS/TerraSimple",
            Some(b"novi"),
            0o755,
        ),
        ("__MACOSX/TerraSimple.app/._x", Some(b"x"), 0o644),
    ]);
    ugradi(&paket, "TerraSimple-0.0.9-macos-universal.zip", &mjesto).expect("ugrađeno");
    assert_eq!(fs::read(&exe).expect("program"), b"novi");
    assert!(!exe.with_file_name("ostatak").exists());
    let roditelj = m.0.join("Programi");
    let imena: Vec<_> = fs::read_dir(&roditelj)
        .expect("popis")
        .map(|s| s.expect("stavka").file_name())
        .collect();
    assert_eq!(imena, ["TerraSimple.app"], "bez radne mape i starog paketa");
}

#[test]
fn linux_paket_zamjenjuje_sadrzaj_mape_programa() {
    let m = Mapa::nova("linux");
    let exe = mapa_programa_na(&m, Sustav::Linux);
    napravi(&exe.with_file_name("moje.gpkg"), b"korisnikovo");
    let mjesto = Mjesto::za_program(&exe, Sustav::Linux).expect("mapa");
    let paket = tar_gz(&[
        ("TerraSimple-0.0.9-linux-x86_64/TerraSimple", b"novi", 0o755),
        (
            "TerraSimple-0.0.9-linux-x86_64/Dozvole/LICENSE",
            b"nova licenca",
            0o644,
        ),
        (
            "TerraSimple-0.0.9-linux-x86_64/pokreni.sh",
            b"#!/bin/sh",
            0o755,
        ),
    ]);
    ugradi(&paket, "TerraSimple-0.0.9-linux-x86_64.tar.gz", &mjesto).expect("ugrađeno");
    let mapa = exe.parent().expect("mapa");
    assert_eq!(fs::read(&exe).expect("program"), b"novi");
    assert_eq!(
        fs::read(mapa.join("Dozvole/LICENSE")).expect("licenca"),
        b"nova licenca"
    );
    assert!(mapa.join("pokreni.sh").is_file());
    // Ono što nije iz paketa ostaje.
    assert_eq!(
        fs::read(mapa.join("moje.gpkg")).expect("podaci"),
        b"korisnikovo"
    );
    assert!(!mapa.join("TerraSimple.stara").exists());
    assert!(!mapa.join(RADNA_MAPA).exists());
}

#[test]
fn paket_bez_programa_ne_dira_postojeci() {
    let m = Mapa::nova("bez-programa");
    let exe = mapa_programa_na(&m, Sustav::Windows);
    let mjesto = Mjesto::za_program(&exe, Sustav::Windows).expect("mapa");
    let bez_exe = zip(&[("TerraSimple-0.0.9/Dozvole/LICENSE", Some(b"x"), 0o644)]);
    let dvije_mape = zip(&[
        ("a/TerraSimple.exe", Some(b"x"), 0o644),
        ("b/TerraSimple.exe", Some(b"x"), 0o644),
    ]);
    for paket in [bez_exe, dvije_mape] {
        assert!(matches!(
            ugradi(&paket, "TerraSimple-0.0.9-windows-x86_64.zip", &mjesto),
            Err(Greska::OstecenPaket(_))
        ));
        assert_eq!(fs::read(&exe).expect("program"), b"stari");
    }
    let mac = Mjesto::za_program(&mac_program(&m), Sustav::MacOs).expect("paket");
    let prazan = zip(&[("Drugo.app/Contents/MacOS/Drugo", Some(b"x"), 0o755)]);
    assert!(ugradi(&prazan, "p.zip", &mac).is_err());
}

#[test]
fn neuspjela_zamjena_vraca_staro() {
    let m = Mapa::nova("vracanje");
    let exe = mapa_programa_na(&m, Sustav::Linux);
    let mjesto = Mjesto::za_program(&exe, Sustav::Linux).expect("mapa");
    let novi = m.0.join("novi-program");
    napravi(&novi, b"novi");
    let parovi = vec![
        (novi, exe.clone()),
        (m.0.join("ne-postoji"), exe.with_file_name("Dozvole")),
    ];
    assert!(matches!(
        zamijeni(&parovi, &mjesto),
        Err(Greska::Zamjena { .. })
    ));
    assert_eq!(fs::read(&exe).expect("program"), b"stari");
    let licenca = exe.with_file_name("Dozvole").join("LICENSE");
    assert_eq!(fs::read(licenca).expect("licenca"), b"stara licenca");
    assert!(!stara(&exe).exists());
}

#[test]
fn radna_mapa_koja_se_ne_moze_napraviti_javlja_gresku() {
    let m = Mapa::nova("radna");
    let exe = mapa_programa_na(&m, Sustav::Linux);
    let mjesto = Mjesto::za_program(&exe, Sustav::Linux).expect("mapa");
    // Datoteka na mjestu roditelja radne mape: mapa se ne može napraviti.
    let pokvareno = Mjesto {
        sustav: Sustav::Linux,
        korijen: exe.clone(),
    };
    assert!(matches!(
        ugradi(b"x", "p.tar.gz", &pokvareno),
        Err(Greska::Zamjena { .. })
    ));
    assert!(ugradi(b"nije paket", "p.tar.gz", &mjesto).is_err());
}

#[test]
fn ostaci_prethodne_nadogradnje_se_brisu() {
    let m = Mapa::nova("ostaci");
    let exe = mapa_programa_na(&m, Sustav::Windows);
    let mapa = exe.parent().expect("mapa").to_path_buf();
    napravi(&mapa.join("TerraSimple.exe.stara"), b"stari");
    napravi(&mapa.join("Dozvole.stara/LICENSE"), b"stara");
    napravi(&mapa.join(RADNA_MAPA).join("x"), b"x");
    pocisti_ostatke(&Mjesto::za_program(&exe, Sustav::Windows).expect("mapa"));
    let mut imena: Vec<_> = fs::read_dir(&mapa)
        .expect("popis")
        .map(|s| s.expect("stavka").file_name())
        .collect();
    imena.sort();
    assert_eq!(imena, ["Dozvole", "TerraSimple.exe"]);

    let mac = mac_program(&m);
    let app = m.0.join("Programi/TerraSimple.app");
    napravi(&stara(&app).join("x"), b"x");
    pocisti_ostatke(&Mjesto::za_program(&mac, Sustav::MacOs).expect("paket"));
    assert!(!stara(&app).exists());
    pocisti_mapu(&m.0.join("ne-postoji"));
}

#[test]
fn ponovno_pokretanje_otvara_novi_program() {
    let m = Mapa::nova("pokretanje");
    let mac = Mjesto::za_program(&mac_program(&m), Sustav::MacOs).expect("paket");
    let n = naredba_ponovnog_pokretanja(&mac);
    assert_eq!(n.get_program(), "open");
    let argumenti: Vec<_> = n.get_args().collect();
    assert_eq!(argumenti[0], "-n");
    assert_eq!(Path::new(argumenti[1]), mac.korijen());
    let exe = mapa_programa_na(&m, Sustav::Windows);
    let win = Mjesto::za_program(&exe, Sustav::Windows).expect("mapa");
    assert_eq!(
        Path::new(naredba_ponovnog_pokretanja(&win).get_program()),
        exe
    );
}
