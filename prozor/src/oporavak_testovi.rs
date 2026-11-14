use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::dijalozi::AktivniDijalog;
use crate::oporavak::{Oporavak, RAZMAK, mapa_za, tekstovi_ostatka, zadana_mapa};
use crate::poruke::KatalogPoruka;
use crate::tema::Razmaci;
use eframe::egui::vec2;
use egui_kittest::{Harness, kittest::Queryable};
use slojevi::{Geometrija, NaredbaObjekta, NaredbaStabla, Uredjivac, Vrh, VrstaGeometrije};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn mapa(ime: &str) -> PathBuf {
    let m = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../.quality/testovi")
        .join(format!("oporavak-{ime}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&m);
    m
}

fn s_objektom() -> Uredjivac {
    let mut u = Uredjivac::novi();
    u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Bunari".into(),
        vrsta: VrstaGeometrije::Tocka,
    })
    .unwrap();
    let s = u.projekt().slojevi().next().unwrap().0;
    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: "Bunar 1".into(),
        geometrija: Geometrija::Tocka(Vrh::novi(18.6, 45.5)),
    })
    .unwrap();
    u
}

/// Program koji je napravio kopiju i srušio se.
fn srusen(m: &Path) {
    let mut o = Oporavak::novi(m.to_path_buf()).unwrap();
    let u = s_objektom();
    assert!(o.tik(&u, None, Instant::now() + RAZMAK));
    o.pricekaj();
    o.padni();
}

#[test]
fn kopija_nastaje_tek_nakon_razmaka_i_nestaje_nakon_spremanja() {
    let m = mapa("razmak");
    let mut o = Oporavak::novi(m.clone()).unwrap();
    let pocetak = Instant::now();
    let mut u = Uredjivac::novi();
    assert!(!o.tik(&u, None, pocetak + RAZMAK), "nema izmjena");
    u = s_objektom();
    assert!(!o.tik(&u, None, pocetak), "prerano");
    assert!(o.tik(&u, None, pocetak + RAZMAK));
    assert!(
        !o.tik(&u, None, pocetak + RAZMAK * 3),
        "kopija još traje ili je ista"
    );
    o.pricekaj();
    assert!(!o.tik(&u, None, pocetak + RAZMAK * 3), "isto stanje");
    assert!(o.kopija().exists());
    assert!(o.kopija().with_extension("json").exists());

    // Druga kopija ovog programa: ona se ne nudi, jer program još radi.
    let drugi = Oporavak::novi(m.clone()).unwrap();
    assert!(drugi.ostaci().is_empty());

    u.oznaci_spremljeno();
    assert!(!o.tik(&u, None, pocetak + RAZMAK * 5));
    assert!(!o.kopija().exists(), "spremljeno više ne treba kopiju");
    drop(o);
    drop(drugi);
    assert_eq!(
        std::fs::read_dir(&m).unwrap().count(),
        0,
        "uredno zatvaranje ne ostavlja ništa"
    );
}

#[test]
fn pad_ostavlja_kopiju_koja_se_vraca_ili_odbacuje() {
    let m = mapa("pad");
    srusen(&m);
    srusen(&m);
    let mut o = Oporavak::novi(m.clone()).unwrap();
    let ostaci = o.ostaci();
    assert_eq!(ostaci.len(), 2);
    assert!(ostaci[0].izvor.is_none() && !ostaci[0].vrijeme.is_empty());
    o.odbaci(&ostaci[1]);
    assert_eq!(o.ostaci().len(), 1);

    let vraceno = o.preuzmi(&ostaci[0]).unwrap();
    assert!(vraceno.exists() && o.ostaci().is_empty());
    assert_eq!(o.vraceno(), Some(vraceno.as_path()));
    assert!(o.preuzmi(&ostaci[0]).is_err(), "kopije više nema");

    // Vraćeni rad kopira se iz vraćene datoteke, pa nestaje kad projekt dobije svoju datoteku.
    let mut u = Uredjivac::iz_projekta(
        geopaket::Datoteka::otvori(&vraceno)
            .unwrap()
            .ucitaj()
            .unwrap(),
    );
    u.oznaci_nespremljeno();
    assert!(o.tik(&u, None, Instant::now() + RAZMAK));
    o.pricekaj();
    assert!(o.kopija().exists());
    let staza = m.join("spremljeno.gpkg");
    o.tik(&u, Some(&staza), Instant::now());
    assert!(!vraceno.exists() && o.vraceno().is_none());
    drop(o);
    std::fs::remove_dir_all(&m).unwrap();
}

#[test]
fn tekstovi_i_zadana_mapa() {
    let k = KatalogPoruka::hrvatski();
    let m = mapa("tekstovi");
    srusen(&m);
    let mut o = Oporavak::novi(m.clone()).unwrap().ostaci().remove(0);
    let [(_, izvor), (_, vrijeme)] = tekstovi_ostatka(&k, &o);
    assert_eq!(izvor, "novog projekta");
    assert!(vrijeme.contains(". "));
    o.izvor = Some(PathBuf::from("/neka/mapa/Teren 2026.gpkg"));
    assert_eq!(tekstovi_ostatka(&k, &o)[0].1, "Teren 2026.gpkg");
    let z = zadana_mapa().unwrap();
    assert!(z.ends_with("TerraSimple/oporavak"), "{z:?}");
    let var = |n: &str| (n != "XDG_DATA_HOME").then(|| PathBuf::from(format!("/{n}")));
    let mac = mapa_za("macos", var).unwrap();
    assert!(mac.starts_with("/HOME/Library/Application Support"));
    assert!(
        mapa_za("windows", var)
            .unwrap()
            .starts_with("/LOCALAPPDATA")
    );
    assert!(
        mapa_za("linux", var)
            .unwrap()
            .starts_with("/HOME/.local/share")
    );
    let xdg = |n: &str| Some(PathBuf::from(format!("/{n}")));
    assert!(mapa_za("linux", xdg).unwrap().starts_with("/XDG_DATA_HOME"));
    assert!(mapa_za("linux", |_| None).is_none());
    std::fs::remove_dir_all(&m).unwrap();
}

fn prozor(m: &Path) -> Harness<'static, Aplikacija> {
    let app = Aplikacija::nova(PostavkeAplikacije {
        oporavak: Some(m.to_path_buf()),
        ..Default::default()
    });
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.run_steps(3);
    h
}

#[test]
fn nakon_pada_program_nudi_rad_i_vraca_ga_nespremljenog() {
    let m = mapa("vrati");
    srusen(&m);
    let mut h = prozor(&m);
    assert!(matches!(
        h.state().aktivni_dijalog,
        Some(AktivniDijalog::Oporavak(_))
    ));
    h.get_by_label("Vrati").click();
    h.run_steps(3);
    let a = h.state();
    let Some(AktivniDijalog::Obavijest { naslov, poruka }) = &a.aktivni_dijalog else {
        panic!("{:?}", a.aktivni_dijalog);
    };
    assert_eq!(naslov, "Rad je vraćen");
    assert!(poruka.contains("novog projekta"), "{poruka}");
    assert!(a.uredjivac.je_li_izmijenjen());
    assert!(a.datoteka.staza.is_none());
    let objekti: usize = a
        .uredjivac
        .projekt()
        .slojevi()
        .map(|(_, s)| s.objekti.len())
        .sum();
    assert_eq!(objekti, 1);
    drop(h);
    std::fs::remove_dir_all(&m).unwrap();
}

#[test]
fn odbacivanje_nudi_sljedecu_kopiju_pa_nista() {
    let m = mapa("odbaci");
    srusen(&m);
    srusen(&m);
    let mut h = prozor(&m);
    h.get_by_label("Odbaci").click();
    h.run_steps(3);
    assert!(matches!(
        h.state().aktivni_dijalog,
        Some(AktivniDijalog::Oporavak(_))
    ));
    h.get_by_label("Odbaci").click();
    h.run_steps(3);
    assert!(h.state().aktivni_dijalog.is_none());
    assert!(h.state().uredjivac.projekt().korijen().is_empty());
    drop(h);
    assert_eq!(std::fs::read_dir(&m).unwrap().count(), 0);
}

#[test]
fn kopija_koja_se_ne_moze_otvoriti_javlja_gresku() {
    let m = mapa("pokvarena");
    srusen(&m);
    let o = Oporavak::novi(m.clone()).unwrap();
    let ostatak = o.ostaci().remove(0);
    drop(o);
    std::fs::write(&ostatak.kopija, b"nije GeoPackage").unwrap();
    let mut h = prozor(&m);
    h.get_by_label("Vrati").click();
    h.run_steps(3);
    let Some(AktivniDijalog::PorukaGreske { naslov, .. }) = &h.state().aktivni_dijalog else {
        panic!("greška");
    };
    assert_eq!(naslov, "Vraćanje nije uspjelo");
    drop(h);
    std::fs::remove_dir_all(&m).unwrap();
}
