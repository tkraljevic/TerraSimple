#![cfg(test)]
use super::*;
use std::{
    fs, io,
    sync::atomic::{AtomicU64, Ordering},
};
static BROJ: AtomicU64 = AtomicU64::new(0);
struct Mapa(PathBuf);
impl Mapa {
    fn nova() -> Self {
        let p = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../.quality")
            .join(format!(
                "qgis-test-{}-{}",
                std::process::id(),
                BROJ.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn datoteka(&self, n: &str) -> PathBuf {
        let p = self.0.join(n);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(&p, b"izmisljeno").unwrap();
        p
    }
    fn okolina(&self, platforma: Platforma) -> Okolina {
        Okolina {
            platforma,
            aplikacije: vec![self.0.join("Applications")],
            program_files: vec![self.0.join("Program Files")],
            osgeo4w: vec![self.0.join("OSGeo4W")],
            putanja: vec![self.0.join("bin")],
        }
    }
    fn mac(&self, naziv: &str, verzija: &str, binarno: bool) -> PathBuf {
        self.datoteka(&format!("Applications/{naziv}/Contents/MacOS/QGIS"));
        let p = self.0.join("Applications").join(naziv);
        let mut d = plist::Dictionary::new();
        d.insert(
            "CFBundleShortVersionString".into(),
            plist::Value::String(verzija.into()),
        );
        let v = plist::Value::Dictionary(d);
        let f = p.join("Contents/Info.plist");
        if binarno {
            v.to_file_binary(f).unwrap();
        } else {
            v.to_file_xml(f).unwrap();
        }
        p
    }
}
impl Drop for Mapa {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn izlaz(s: &str, uspjeh: bool) -> io::Result<Output> {
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;
    Ok(Output {
        status: std::process::ExitStatus::from_raw(if uspjeh { 0 } else { 256 }),
        stdout: s.as_bytes().to_vec(),
        stderr: Vec::new(),
    })
}
fn bez_upita(_: &Path) -> io::Result<Output> {
    panic!("Ne smije se pokrenuti upit inačice.")
}
#[test]
fn mac_usporedjuje_brojeve_i_cita_obje_vrste_plista() {
    let m = Mapa::nova();
    m.mac("QGIS 3.9.app", "3.9.9", false);
    let novo = m.mac("QGIS-LTR.app", "3.44.15", true);
    m.mac("Drugi.app", "99.0", false);
    m.datoteka("Applications/QGIS-nepotpun.app/Contents/Info.plist");
    let q = pronadji_u(&m.okolina(Platforma::Mac), None, bez_upita).unwrap();
    assert_eq!(q.staza, novo);
    assert_eq!(q.inacica, Some(Inacica(3, 44, 15)));
    assert!(q.paket_macos);
    let stara = m.0.join("Applications/QGIS 3.9.app");
    assert_eq!(
        pronadji_u(&m.okolina(Platforma::Mac), Some(&stara), bez_upita)
            .unwrap()
            .staza,
        stara.canonicalize().unwrap()
    );
}
#[test]
fn windows_usporedjuje_program_files_i_osgeo4w() {
    let m = Mapa::nova();
    m.datoteka("Program Files/QGIS 3.9.9/bin/qgis-bin.exe");
    m.datoteka("Program Files/QGIS 3.40.10/bin/qgis-bin.exe");
    let novo = m.datoteka("OSGeo4W/bin/qgis-ltr-bin.exe");
    let db = m.datoteka("OSGeo4W/etc/setup/installed.db");
    fs::write(
        db,
        "INSTALLED.DB 2\nqgis-ltr qgis-ltr-3.44.15-1.tar.bz2 0\n",
    )
    .unwrap();
    let q = pronadji_u(&m.okolina(Platforma::Windows), None, bez_upita).unwrap();
    assert_eq!(q.staza, novo);
    assert_eq!(q.inacica, Some(Inacica(3, 44, 15)));
}
#[test]
fn linux_usporedjuje_sve_staze_a_upit_je_zamjenjiv() {
    let m = Mapa::nova();
    let a = m.datoteka("a/qgis");
    let b = m.datoteka("b/qgis");
    let mut o = m.okolina(Platforma::Linux);
    o.putanja = vec![
        a.parent().unwrap().into(),
        b.parent().unwrap().into(),
        a.parent().unwrap().into(),
    ];
    let broj = std::cell::Cell::new(0);
    let q = pronadji_u(&o, None, |p| {
        broj.set(broj.get() + 1);
        izlaz(
            if p == a {
                "QGIS 3.40.9"
            } else {
                "QGIS 3.44.1 'Pokus'"
            },
            true,
        )
    })
    .unwrap();
    assert_eq!(broj.get(), 2);
    assert_eq!(q.staza, b);
    let q = pronadji_u(&o, Some(&a), bez_upita).unwrap();
    assert_eq!(q.staza, a.canonicalize().unwrap());
    assert!(
        pronadji_u(&o, None, |_| izlaz("nepoznato", true))
            .unwrap_err()
            .to_string()
            .contains("usporediti")
    );
    assert!(pronadji_u(&o, None, |_| izlaz("3.99.0", false)).is_err());
    assert!(pronadji_u(&o, None, |_| Err(io::ErrorKind::NotFound.into())).is_err());
}
#[test]
fn nedostajuci_program_i_nevaljana_zadana_staza_imaju_poruku() {
    let m = Mapa::nova();
    for p in [Platforma::Mac, Platforma::Windows, Platforma::Linux] {
        let o = m.okolina(p);
        let e = pronadji_u(&o, None, bez_upita).unwrap_err().to_string();
        assert!(e.contains("nije pronađen") && e.contains("stazu"));
        assert!(pronadji_u(&o, Some(&m.0.join("nema")), bez_upita).is_err());
        assert!(pronadji_u(&o, Some(&m.0), bez_upita).is_err());
    }
    let f = m.datoteka("Applications/QGIS-prazan.app/Contents/Info.plist");
    assert!(
        pronadji_u(
            &m.okolina(Platforma::Mac),
            Some(f.parent().unwrap().parent().unwrap()),
            bez_upita
        )
        .is_err()
    );
    let mut o = m.okolina(Platforma::Mac);
    o.aplikacije = vec![f];
    assert!(pronadji_u(&o, None, bez_upita).is_err());
    let mut o = m.okolina(Platforma::Windows);
    o.program_files = vec![m.datoteka("kriva-mapa")];
    assert!(pronadji_u(&o, None, bez_upita).is_err());
}
#[test]
fn pokvareni_metapodaci_uzimaju_odvojeni_upit_izvrsne_datoteke() {
    let m = Mapa::nova();
    let p = m.mac("QGIS.app", "nepoznato", false);
    let q = pronadji_u(&m.okolina(Platforma::Mac), None, |e| {
        assert_eq!(e, p.join("Contents/MacOS/QGIS"));
        izlaz("QGIS 3.44.15", true)
    })
    .unwrap();
    assert_eq!(q.inacica, Some(Inacica(3, 44, 15)));
    for v in [
        plist::Value::String("nije mapa".into()),
        plist::Value::Dictionary(plist::Dictionary::new()),
    ] {
        v.to_file_xml(p.join("Contents/Info.plist")).unwrap();
        assert!(verzija::mac(&p).is_none());
    }
    fs::write(p.join("Contents/Info.plist"), "krivo").unwrap();
    assert!(verzija::mac(&p).is_none());
    assert!(verzija::mac(&m.0).is_none());
    assert_eq!(
        pronadji_u(&m.okolina(Platforma::Mac), None, |_| Err(
            io::ErrorKind::NotFound.into()
        ))
        .unwrap()
        .inacica,
        None
    );
}
#[test]
fn naredba_cuva_jedan_argument_s_razmacima_i_posebnim_znakovima() {
    let m = Mapa::nova();
    let f = m.datoteka("--ime s razmacima & znakom.GPKG");
    for mac in [false, true] {
        let q = Qgis {
            staza: m.0.join("QGIS & program"),
            inacica: None,
            paket_macos: mac,
        };
        let n = q.naredba(&f).unwrap();
        // Na Windowsu canonicalize daje \\?\C:\…, a naredba taj prefiks skida.
        assert_eq!(
            n.argumenti.last().unwrap(),
            crate::bez_dugog_prefiksa(f.canonicalize().unwrap()).as_os_str()
        );
        if mac {
            assert_eq!(n.program, PathBuf::from("/usr/bin/open"));
            assert_eq!(n.argumenti[0], "-a");
            assert_eq!(n.argumenti[1], q.staza.as_os_str());
            assert_eq!(n.argumenti.len(), 3);
        } else {
            assert_eq!(n.program, q.staza);
            assert_eq!(n.argumenti.len(), 1);
        }
        assert!(q.naredba(&m.0).is_err());
        assert!(q.naredba(&m.datoteka("krivo.txt")).is_err());
    }
    let q = Qgis {
        staza: m.0.join("nepostojeci-program"),
        inacica: None,
        paket_macos: false,
    };
    assert!(q.otvori(&m.0.join("nema.gpkg")).is_err());
    // Ne postoji izvršna datoteka: provjera greške ne pokreće nikakav proces.
    assert!(q.otvori(&f).is_err());
    assert!(izlaz_verzije(&q.staza).is_err());
    assert!(pronadji(Some(&m.0.join("nepostojeci-program"))).is_err());
}
#[test]
fn sustav_i_varijable_ne_ovise_o_testnom_racunalu() {
    for (ime, p) in [
        ("macos", Platforma::Mac),
        ("windows", Platforma::Windows),
        ("linux", Platforma::Linux),
    ] {
        assert_eq!(Platforma::iz_naziva(ime), p);
    }
    let _trenutna = Okolina::trenutna();
    let o = Okolina::iz_varijabli(Platforma::Windows, |n| match n {
        "ProgramFiles" => Some("Program Files".into()),
        "OSGEO4W_ROOT" => Some("Drugi OSGeo".into()),
        "SystemDrive" => Some("D:".into()),
        "PATH" => Some(std::env::join_paths(["a", "b"]).unwrap()),
        _ => None,
    });
    assert_eq!(o.program_files, vec![PathBuf::from("Program Files")]);
    assert_eq!(o.osgeo4w.len(), 3);
    assert_eq!(o.putanja.len(), 2);
    let o = Okolina::iz_varijabli(Platforma::Linux, |_| None);
    assert!(o.program_files.is_empty());
    assert!(o.putanja.is_empty());
    assert_eq!(o.osgeo4w.len(), 2);
    for (s, v) in [
        ("QGIS 3.44.15-Solothurn", Some(Inacica(3, 44, 15))),
        ("3.9", Some(Inacica(3, 9, 0))),
        ("nema", None),
        ("3.", None),
        ("3.4.999999999999", None),
    ] {
        assert_eq!(verzija::procitaj(s), v);
    }
}

#[test]
fn dugi_prefiks_windowsa_se_skida_a_ostale_staze_ostaju() {
    use crate::bez_dugog_prefiksa as b;
    assert_eq!(
        b(PathBuf::from(r"\\?\C:\Teren\a.gpkg")),
        PathBuf::from(r"C:\Teren\a.gpkg")
    );
    assert_eq!(
        b(PathBuf::from(r"\\?\UNC\posluzitelj\dijeljeno\a.gpkg")),
        PathBuf::from(r"\\posluzitelj\dijeljeno\a.gpkg")
    );
    assert_eq!(
        b(PathBuf::from("/tmp/teren/a.gpkg")),
        PathBuf::from("/tmp/teren/a.gpkg")
    );
}

#[test]
fn zadana_staza_na_macu_smije_biti_program_a_na_linuxu_ne_paket() {
    let m = Mapa::nova();
    let program = m.datoteka("bin/qgis");
    let upit = |_: &Path| -> io::Result<std::process::Output> { panic!("zadana staza se ne pita") };
    let q = pronadji_u(&m.okolina(Platforma::Mac), Some(&program), upit).unwrap();
    assert!(!q.paket_macos);
    assert_eq!(q.staza, program.canonicalize().unwrap());
    m.datoteka("Paket.app/Contents/MacOS/QGIS");
    let paket = m.0.join("Paket.app");
    assert!(
        pronadji_u(&m.okolina(Platforma::Mac), Some(&paket), upit)
            .unwrap()
            .paket_macos
    );
    assert!(pronadji_u(&m.okolina(Platforma::Linux), Some(&paket), upit).is_err());
}

#[cfg(unix)]
#[test]
fn necitljiva_mapa_programa_javlja_gresku_a_ne_prazan_popis() {
    use std::os::unix::fs::PermissionsExt;
    let m = Mapa::nova();
    let mape = m.0.join("Applications");
    fs::create_dir_all(&mape).unwrap();
    fs::set_permissions(&mape, fs::Permissions::from_mode(0o000)).unwrap();
    let ishod = trazenje::kandidati(&m.okolina(Platforma::Mac));
    fs::set_permissions(&mape, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(ishod.is_err());
}
