//! Testovi sučelja za izbornik Datoteka, spremanje, otvaranje i dijaloge.

#![cfg(test)]

use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::datoteka::LazniBirac;
use crate::dijalozi::{AktivniDijalog, NamjeraPotvrde};
use crate::stanje::Odabir;
use crate::tema::Razmaci;
use eframe::egui::{self, Key, Modifiers, vec2};
use egui_kittest::{Harness, kittest::Queryable};
use slojevi::{NaredbaStabla, Uredjivac, VrstaGeometrije};
use std::path::PathBuf;
use std::time::Duration;

fn cekaj_smirenje<S>(harness: &mut Harness<'_, S>, najvise: Duration) {
    let pocetak = std::time::Instant::now();
    while pocetak.elapsed() < najvise {
        if harness.try_run().is_ok() {
            return;
        }
    }
    panic!("sučelje se nije smirilo unutar {najvise:?}");
}

fn nacrtani_tekstovi<S>(harness: &Harness<'_, S>) -> Vec<String> {
    harness
        .output()
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
            _ => None,
        })
        .collect()
}

#[test]
fn novi_s_potvrdom() {
    let mapa = PathBuf::from(".quality/test_ui_novi");
    let _ = std::fs::create_dir_all(&mapa);
    let staza_spremanja = mapa.join("spremljeno_prije_novog.gpkg");
    let _ = std::fs::remove_file(&staza_spremanja);

    let app = Aplikacija::nova(PostavkeAplikacije {
        primjer: false,
        ..Default::default()
    });
    let birac = LazniBirac::novi();
    let mut harness = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

    harness.state_mut().postavi_birac(Box::new(birac.clone()));

    // Dodaj novi sloj da imamo nespremljenu izmjenu
    harness.get_by_label("Novi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    harness.get_by_label("Napravi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().uredjivac.je_li_izmijenjen());
    assert_eq!(harness.state().uredjivac.projekt().slojevi().count(), 1);

    // Otvori Datoteka izbornik i klikni Novi
    harness.get_by_label("Datoteka").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    harness.get_by_label("Novi").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));

    // Dijalog potvrde se pojavio
    assert!(matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::PotvrdaNespremljeno(NamjeraPotvrde::Novi))
    ));

    // 1. Odustani -> projekt ostaje, sloj ostaje, dijalog zatvoren
    harness.get_by_label("Odustani").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());
    assert!(harness.state().uredjivac.je_li_izmijenjen());
    assert_eq!(harness.state().uredjivac.projekt().slojevi().count(), 1);

    // 2. Prečac Cmd+N -> Ne spremaj -> resetira projekt
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::PotvrdaNespremljeno(NamjeraPotvrde::Novi))
    ));
    harness.get_by_label("Ne spremaj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());
    assert!(!harness.state().uredjivac.je_li_izmijenjen());
    assert_eq!(harness.state().uredjivac.projekt().slojevi().count(), 0);

    // 3. Dodaj sloj, postavi stazu za spremanje, Cmd+N -> Spremi -> sprema u .quality/... i resetira
    harness.get_by_label("Novi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    harness.get_by_label("Napravi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().uredjivac.je_li_izmijenjen());

    birac.postavi_spremanje(staza_spremanja.clone());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::N);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::PotvrdaNespremljeno(NamjeraPotvrde::Novi))
    ));
    harness.get_by_label("Spremi").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());
    assert!(!harness.state().uredjivac.je_li_izmijenjen());
    assert_eq!(harness.state().uredjivac.projekt().slojevi().count(), 0);
    assert!(staza_spremanja.exists());

    let _ = std::fs::remove_dir_all(&mapa);
}

#[test]
fn otvori_spremi_i_spremi_kao() {
    let mapa = PathBuf::from(".quality/test_ui_otvori");
    let _ = std::fs::create_dir_all(&mapa);
    let staza_izvorna = mapa.join("otvori_izvor.gpkg");
    let staza_druga = mapa.join("spremi_kao_cilj.gpkg");
    let _ = std::fs::remove_file(&staza_izvorna);
    let _ = std::fs::remove_file(&staza_druga);

    // Pripremi pravu .gpkg datoteku s jednim slojem
    {
        let mut u = Uredjivac::novi();
        u.izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "Izvorni sloj".into(),
            vrsta: VrstaGeometrije::Poligon,
        })
        .unwrap();
        let mut d = geopaket::Datoteka::otvori(&staza_izvorna).unwrap();
        d.spremi(u.projekt()).unwrap();
    }

    let app = Aplikacija::nova(PostavkeAplikacije {
        primjer: false,
        ..Default::default()
    });
    let birac = LazniBirac::novi();
    let mut harness = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

    harness.state_mut().postavi_birac(Box::new(birac.clone()));

    // Otvori preko prečaca Cmd+O
    birac.postavi_otvaranje(staza_izvorna.clone());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));

    // Provjera da se sloj nalazi u stablu
    assert!(harness.query_by_label("Izvorni sloj").is_some());
    assert!(!harness.state().uredjivac.je_li_izmijenjen());
    let tekstovi = nacrtani_tekstovi(&harness);
    assert!(tekstovi.iter().any(|t| t.contains("otvori_izvor.gpkg")));
    assert!(
        !tekstovi
            .iter()
            .any(|t| t.contains('*') || t.contains("nespremljeno"))
    );

    // Dodaj izmjenu
    harness.get_by_label("Novi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    harness.get_by_label("Napravi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().uredjivac.je_li_izmijenjen());
    let tekstovi_nakon_izmjene = nacrtani_tekstovi(&harness);
    assert!(
        tekstovi_nakon_izmjene
            .iter()
            .any(|t| t.contains('*') || t.contains("nespremljeno"))
    );

    // Spremi (Cmd+S)
    harness.key_press_modifiers(Modifiers::COMMAND, Key::S);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(!harness.state().uredjivac.je_li_izmijenjen());
    let tekstovi_nakon_spremanja = nacrtani_tekstovi(&harness);
    assert!(
        !tekstovi_nakon_spremanja
            .iter()
            .any(|t| t.contains('*') || t.contains("nespremljeno"))
    );

    // Spremi kao na staza_druga (Cmd+Shift+S)
    birac.postavi_spremanje(staza_druga.clone());
    harness.key_press_modifiers(Modifiers::COMMAND | Modifiers::SHIFT, Key::S);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(!harness.state().uredjivac.je_li_izmijenjen());
    assert!(staza_druga.exists());
    let tekstovi_nakon_druge = nacrtani_tekstovi(&harness);
    assert!(
        tekstovi_nakon_druge
            .iter()
            .any(|t| t.contains("spremi_kao_cilj.gpkg"))
    );

    let _ = std::fs::remove_dir_all(&mapa);
}

#[test]
fn greska_pri_spremanju() {
    let mapa = PathBuf::from(".quality/test_ui_greska");
    let _ = std::fs::create_dir_all(&mapa);
    let staza = mapa.join("sukob.gpkg");
    let _ = std::fs::remove_file(&staza);

    // Pripremi bazu
    {
        let mut u = Uredjivac::novi();
        u.izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "Sloj Sukoba".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
        .unwrap();
        let mut d = geopaket::Datoteka::otvori(&staza).unwrap();
        d.spremi(u.projekt()).unwrap();
    }

    let app = Aplikacija::nova(PostavkeAplikacije {
        primjer: false,
        ..Default::default()
    });
    let birac = LazniBirac::novi();
    let mut harness = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

    harness.state_mut().postavi_birac(Box::new(birac.clone()));

    // Otvori
    birac.postavi_otvaranje(staza.clone());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));

    // Vanjska izmjena u sqlite bazi (mijenja data_version)
    {
        let conn = rusqlite::Connection::open(&staza).unwrap();
        conn.execute("CREATE TABLE vanjski_unos (id INTEGER);", [])
            .unwrap();
    }

    // Izmijeni projekt u aplikaciji
    harness.get_by_label("Novi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    harness.get_by_label("Napravi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().uredjivac.je_li_izmijenjen());
    let broj_slojeva_prije = harness.state().uredjivac.projekt().slojevi().count();

    // Pokušaj spremiti preko Cmd+S
    harness.key_press_modifiers(Modifiers::COMMAND, Key::S);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));

    // Iskočio je prozorčić o sukobu, a ne opća poruka s tekstom iz geopaketa.
    assert_eq!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::SukobSpremanja)
    );
    let t = nacrtani_tekstovi(&harness);
    assert!(
        t.iter()
            .any(|t| t == "Datoteku je promijenio drugi program"),
        "{t:?}"
    );
    assert!(
        t.iter()
            .any(|t| t.contains("Spremi svoj crtež kao novu datoteku")),
        "{t:?}"
    );

    // Odustani zatvara prozorčić, a projekt NIJE izgubljen.
    harness.get_by_label("Odustani").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());
    assert_eq!(
        harness.state().uredjivac.projekt().slojevi().count(),
        broj_slojeva_prije
    );
    assert!(harness.state().uredjivac.je_li_izmijenjen());

    // Odluke (9. 11. 2026.): Spremi kao sprema crtež u novu datoteku, a izvorna ostaje kakva jest.
    let izvorna_prije = std::fs::read(&staza).unwrap();
    let nova = mapa.join("nakon-sukoba.gpkg");
    let _ = std::fs::remove_file(&nova);
    harness.key_press_modifiers(Modifiers::COMMAND, Key::S);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert_eq!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::SukobSpremanja)
    );
    birac.postavi_spremanje(nova.clone());
    harness.get_by_label("Spremi kao…").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());
    assert!(!harness.state().uredjivac.je_li_izmijenjen());
    assert_eq!(harness.state().datoteka.staza.as_ref(), Some(&nova));
    assert_eq!(std::fs::read(&staza).unwrap(), izvorna_prije);
    let mut procitana = geopaket::Datoteka::otvori(&nova).unwrap();
    assert_eq!(
        procitana.ucitaj().unwrap().slojevi().count(),
        broj_slojeva_prije
    );

    let _ = std::fs::remove_dir_all(&mapa);
}

#[test]
fn upozorenje_pri_brisanju() {
    let mapa = PathBuf::from(".quality/test_ui_upozorenje");
    let _ = std::fs::create_dir_all(&mapa);
    let staza = mapa.join("nepodrzani.gpkg");
    let _ = std::fs::remove_file(&staza);

    // Napravi bazu s dva sloja: jedan s nepodržanim zapisima, drugi čist
    let (id_nepodrzani, id_cisti) = {
        let mut u = Uredjivac::novi();
        let s1 = match u
            .izvrsi(NaredbaStabla::DodajSloj {
                u: None,
                naziv: "Problematični".into(),
                vrsta: VrstaGeometrije::Tocka,
            })
            .unwrap()
        {
            Some(slojevi::Novo::Sloj(s)) => s,
            _ => panic!(),
        };
        let s2 = match u
            .izvrsi(NaredbaStabla::DodajSloj {
                u: None,
                naziv: "Obični".into(),
                vrsta: VrstaGeometrije::Tocka,
            })
            .unwrap()
        {
            Some(slojevi::Novo::Sloj(s)) => s,
            _ => panic!(),
        };
        let mut d = geopaket::Datoteka::otvori(&staza).unwrap();
        d.spremi(u.projekt()).unwrap();

        // Ubaci nepodržane zapise u prvi sloj
        let conn = rusqlite::Connection::open(&staza).unwrap();
        let mut krivulja = vec![b'G', b'P', 0, 1, 230, 16, 0, 0, 1, 9, 0, 0, 0];
        krivulja.extend(0_u32.to_le_bytes());
        let tablica = format!("sloj_{}", s1.0);
        conn.execute(
            &format!("INSERT INTO {tablica}(fid,geom) VALUES(999,?1)"),
            [krivulja],
        )
        .unwrap();
        conn.execute(
            &format!("INSERT INTO {tablica}(fid,geom) VALUES(1000,NULL)"),
            [],
        )
        .unwrap();
        (s1, s2)
    };

    let app = Aplikacija::nova(PostavkeAplikacije {
        primjer: false,
        ..Default::default()
    });
    let birac = LazniBirac::novi();
    let mut harness = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

    harness.state_mut().postavi_birac(Box::new(birac.clone()));

    // Otvori datoteku
    birac.postavi_otvaranje(staza.clone());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));

    // Dijalog upozorenja o nepodržanim zapisima pri otvaranju
    if matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::UpozorenjaOtvaranja { .. })
    ) {
        harness.get_by_label("U redu").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
    }

    // Odaberi problematični sloj
    harness.state_mut().stanje.odabrano = Some(Odabir::Sloj(id_nepodrzani));
    harness.run();

    // Tipka Delete -> otvara se upozorenje prije brisanja!
    harness.key_press(Key::Delete);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::UpozorenjeBrisanja { .. })
    ));

    // 1. Klik na "Odustani" -> sloj ostaje
    harness.get_by_label("Odustani").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());
    assert!(
        harness
            .state()
            .uredjivac
            .projekt()
            .sloj(id_nepodrzani)
            .is_some()
    );

    // 2. Ponovni Delete -> klik na "Obriši sloj" -> sloj se briše
    harness.key_press(Key::Delete);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::UpozorenjeBrisanja { .. })
    ));
    harness.get_by_label("Obriši sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());
    assert!(
        harness
            .state()
            .uredjivac
            .projekt()
            .sloj(id_nepodrzani)
            .is_none()
    );

    // 3. Odaberi obični sloj (0 nepodržanih zapisa) -> Delete briše odmah bez pitanja
    harness.state_mut().stanje.odabrano = Some(Odabir::Sloj(id_cisti));
    harness.run();
    harness.key_press(Key::Delete);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());
    assert!(harness.state().uredjivac.projekt().sloj(id_cisti).is_none());

    let _ = std::fs::remove_dir_all(&mapa);
}

#[test]
fn zatvaranje_s_nespremljenim_izmjenama() {
    let app = Aplikacija::nova(PostavkeAplikacije {
        primjer: false,
        ..Default::default()
    });
    let mut harness = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

    // Dodaj sloj da imamo nespremljenu izmjenu
    harness.get_by_label("Novi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    harness.get_by_label("Napravi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().uredjivac.je_li_izmijenjen());

    // Simuliraj zatvaranje prozora
    harness
        .input_mut()
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));

    // Dijalog se pojavio
    assert!(matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::PotvrdaNespremljeno(
            NamjeraPotvrde::ZatvoriProzor
        ))
    ));

    // Odustani zatvara dijalog i drži aplikaciju otvorenom
    harness.get_by_label("Odustani").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());
    assert!(harness.state().uredjivac.je_li_izmijenjen());

    // Opet zatvaranje -> Ne spremaj
    harness
        .input_mut()
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::PotvrdaNespremljeno(
            NamjeraPotvrde::ZatvoriProzor
        ))
    ));
    harness.get_by_label("Ne spremaj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());

    // Pravi prozor nakon „Ne spremaj” ponovno javi zatvaranje; tad se više ne smije pitati.
    harness
        .input_mut()
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(
        harness.state().aktivni_dijalog.is_none(),
        "nakon „Ne spremaj” prozor se mora zatvoriti bez novog pitanja"
    );
}

#[test]
fn snimanje_s_nespremljenim_izmjenama_se_zatvara_bez_pitanja() {
    let app = Aplikacija::nova(PostavkeAplikacije {
        snimka: Some(PathBuf::from(".quality/snimka_test.png")),
        primjer: false,
        ..Default::default()
    });
    let mut harness = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

    // Dodaj sloj da imamo nespremljenu izmjenu
    harness.get_by_label("Novi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    harness.get_by_label("Napravi sloj").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().uredjivac.je_li_izmijenjen());

    // Simuliraj zatvaranje prozora kad je postavljena snimka
    harness
        .input_mut()
        .viewports
        .entry(egui::ViewportId::ROOT)
        .or_default()
        .events
        .push(egui::ViewportEvent::Close);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));

    // Nema otvorenog dijaloga potvrde – zatvaranje ide bez pitanja
    assert!(
        harness.state().aktivni_dijalog.is_none(),
        "uz --snimka zatvaranje ne smije otvarati dijalog potvrde"
    );
}

#[test]
fn upozorenja_pri_otvaranju() {
    let mapa = PathBuf::from(".quality/test_ui_upozorenja_otvaranja");
    let _ = std::fs::create_dir_all(&mapa);
    let staza = mapa.join("nepodrzani_otvaranje.gpkg");
    let _ = std::fs::remove_file(&staza);

    // Stvori datoteku s nepodržanim zapisima
    {
        let mut u = Uredjivac::novi();
        let s = match u
            .izvrsi(NaredbaStabla::DodajSloj {
                u: None,
                naziv: "Krivulje".into(),
                vrsta: VrstaGeometrije::Linija,
            })
            .unwrap()
        {
            Some(slojevi::Novo::Sloj(s)) => s,
            _ => panic!(),
        };
        let mut d = geopaket::Datoteka::otvori(&staza).unwrap();
        d.spremi(u.projekt()).unwrap();

        let conn = rusqlite::Connection::open(&staza).unwrap();
        let mut krivulja = vec![b'G', b'P', 0, 1, 230, 16, 0, 0, 1, 9, 0, 0, 0];
        krivulja.extend(0_u32.to_le_bytes());
        let tablica = format!("sloj_{}", s.0);
        conn.execute(
            &format!("INSERT INTO {tablica}(fid,geom) VALUES(999,?1)"),
            [krivulja],
        )
        .unwrap();
    }

    let app = Aplikacija::nova(PostavkeAplikacije {
        primjer: false,
        ..Default::default()
    });
    let birac = LazniBirac::novi();
    let mut harness = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

    harness.state_mut().postavi_birac(Box::new(birac.clone()));

    // Otvori datoteku
    birac.postavi_otvaranje(staza.clone());
    harness.key_press_modifiers(Modifiers::COMMAND, Key::O);
    cekaj_smirenje(&mut harness, Duration::from_millis(500));

    // Dijalog upozorenja se otvorio
    assert!(matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::UpozorenjaOtvaranja { .. })
    ));

    // Proširi detalje
    let gumb_popis = harness.get_by_label("Prikaži popis (1)");
    gumb_popis.click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));

    assert!(matches!(
        harness.state().aktivni_dijalog,
        Some(AktivniDijalog::UpozorenjaOtvaranja {
            otvoren_popis: true,
            ..
        })
    ));

    // Zatvori na "U redu"
    harness.get_by_label("U redu").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().aktivni_dijalog.is_none());

    let _ = std::fs::remove_dir_all(&mapa);
}

#[test]
fn klik_izvan_padajuceg_izbornika_zatvara_izbornik() {
    let app = Aplikacija::nova(PostavkeAplikacije {
        primjer: false,
        ..Default::default()
    });
    let mut harness = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

    // Otvori izbornik klikom na Datoteka
    harness.get_by_label("Datoteka").click();
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(harness.state().izbornik_datoteka_otvoren);

    // Klik na područje karte (izvan izbornika i ispod trake izbornika)
    harness.event(egui::Event::PointerMoved(egui::pos2(400.0, 250.0)));
    harness.event(egui::Event::PointerButton {
        pos: egui::pos2(400.0, 250.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
    harness.event(egui::Event::PointerButton {
        pos: egui::pos2(400.0, 250.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });
    cekaj_smirenje(&mut harness, Duration::from_millis(500));
    assert!(
        !harness.state().izbornik_datoteka_otvoren,
        "klik izvan izbornika mora zatvoriti padajući izbornik"
    );
}

#[test]
fn izbornik_pise_precace_kako_je_uobicajeno_na_sustavu() {
    for (sustav, ocekivano, krivo) in [
        (
            egui::os::OperatingSystem::Mac,
            ["⌘N", "⌘O", "⌘S", "⇧⌘S"],
            "Ctrl+S",
        ),
        (
            egui::os::OperatingSystem::Windows,
            ["Ctrl+N", "Ctrl+O", "Ctrl+S", "Ctrl+Shift+S"],
            "⌘S",
        ),
        (
            egui::os::OperatingSystem::Nix,
            ["Ctrl+N", "Ctrl+O", "Ctrl+S", "Ctrl+Shift+S"],
            "⌘S",
        ),
    ] {
        let app = Aplikacija::nova(PostavkeAplikacije {
            primjer: false,
            ..Default::default()
        });
        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_os(sustav)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
        harness.get_by_label("Datoteka").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        let tekstovi = nacrtani_tekstovi(&harness);
        for oznaka in ocekivano {
            assert!(
                tekstovi.iter().any(|t| t == oznaka),
                "{sustav:?}: nema {oznaka} u {tekstovi:?}"
            );
        }
        assert!(!tekstovi.iter().any(|t| t == krivo), "{sustav:?}: {krivo}");
    }
}

#[test]
fn spremi_kao_iz_novog_projekta_bez_postojeceg_gpkg() {
    let mapa = PathBuf::from(".quality/test_ui_spremi_kao_novi");
    let _ = std::fs::create_dir_all(&mapa);
    let staza = mapa.join("potpuno_novi.gpkg");
    let _ = std::fs::remove_file(&staza);

    let mut uredjivac = Uredjivac::novi();
    let _ = uredjivac.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Novi sloj".into(),
        vrsta: VrstaGeometrije::Tocka,
    });
    let mut datoteka = crate::datoteka::PodaciDatoteke::default();
    let katalog = crate::poruke::KatalogPoruka::hrvatski();

    assert!(datoteka.gpkg.is_none());
    crate::datoteka::radnja_spremi_kao(&mut uredjivac, &mut datoteka, &katalog, staza.clone())
        .expect("spremanje novog projekta mora uspjeti");

    assert!(staza.exists(), "stvorena datoteka mora postojati na disku");
    assert!(datoteka.gpkg.is_some());
    assert_eq!(datoteka.staza.as_deref(), Some(staza.as_path()));
    assert!(!uredjivac.je_li_izmijenjen());
}

fn prozor_s_qgisom(
    ime: &str,
) -> (
    Harness<'static, Aplikacija>,
    LazniBirac,
    crate::qgis_veza::LazniQgis,
    PathBuf,
) {
    let mapa = PathBuf::from(".quality/test_ui_qgis");
    let _ = std::fs::create_dir_all(&mapa);
    let staza = mapa.join(format!("{ime}-{}.gpkg", std::process::id()));
    let _ = std::fs::remove_file(&staza);
    let mut app = Aplikacija::default();
    let birac = LazniBirac::novi();
    let qgis = crate::qgis_veza::LazniQgis::default();
    app.postavi_birac(Box::new(birac.clone()));
    app.qgis = Box::new(qgis.clone());
    app.uredjivac
        .izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "Bunari".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
        .expect("sloj");
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.run_steps(3);
    (h, birac, qgis, staza)
}

fn otvoreno(q: &crate::qgis_veza::LazniQgis) -> Vec<PathBuf> {
    q.otvoreno.lock().map(|o| o.clone()).unwrap_or_default()
}

#[test]
fn otvori_u_qgisu_najprije_sprema_pa_otvara_spremljenu_datoteku() {
    let (mut h, birac, qgis, staza) = prozor_s_qgisom("sprema");
    birac.postavi_spremanje(&staza);
    h.get_by_label("Otvori u QGIS-u").click();
    cekaj_smirenje(&mut h, Duration::from_millis(500));
    assert!(staza.is_file(), "projekt je spremljen prije otvaranja");
    assert!(!h.state().uredjivac.je_li_izmijenjen());
    assert_eq!(otvoreno(&qgis), vec![staza.clone()]);
    // Drugi put datoteka već postoji, pa se ne pita kamo spremiti.
    h.get_by_label("Otvori u QGIS-u").click();
    cekaj_smirenje(&mut h, Duration::from_millis(500));
    assert_eq!(otvoreno(&qgis), vec![staza.clone(), staza.clone()]);
    let _ = std::fs::remove_file(&staza);
}

#[test]
fn odustajanje_od_spremanja_ne_otvara_qgis() {
    let (mut h, _birac, qgis, _staza) = prozor_s_qgisom("odustaje");
    h.get_by_label("Otvori u QGIS-u").click();
    cekaj_smirenje(&mut h, Duration::from_millis(500));
    assert!(otvoreno(&qgis).is_empty());
    assert!(h.state().aktivni_dijalog.is_none());
}

#[test]
fn qgis_koji_se_ne_da_otvoriti_javlja_poruku() {
    let (mut h, birac, qgis, staza) = prozor_s_qgisom("bez-qgisa");
    birac.postavi_spremanje(&staza);
    if let Ok(mut g) = qgis.greska.lock() {
        *g = Some("QGIS nije pronađen na ovom računalu.".into());
    }
    h.get_by_label("Otvori u QGIS-u").click();
    cekaj_smirenje(&mut h, Duration::from_millis(500));
    assert!(matches!(
        h.state().aktivni_dijalog,
        Some(AktivniDijalog::PorukaGreske { .. })
    ));
    let t = nacrtani_tekstovi(&h);
    assert!(t.iter().any(|t| t == "QGIS se nije otvorio"), "{t:?}");
    assert!(t.iter().any(|t| t.contains("QGIS nije pronađen")), "{t:?}");
    let _ = std::fs::remove_file(&staza);
}

/// `--dijalog <naziv>` otvara točno taj prozorčić, za snimke izgleda.
#[test]
fn pregled_dijaloga_po_nazivu() {
    use crate::aplikacija::dijalog_za_pregled;
    let k = crate::poruke::KatalogPoruka::hrvatski();
    assert_eq!(
        dijalog_za_pregled(Some("potvrda"), &k),
        Some(AktivniDijalog::PotvrdaNespremljeno(NamjeraPotvrde::Novi))
    );
    assert_eq!(
        dijalog_za_pregled(Some("sukob"), &k),
        Some(AktivniDijalog::SukobSpremanja)
    );
    assert!(matches!(
        dijalog_za_pregled(Some("greska"), &k),
        Some(AktivniDijalog::PorukaGreske { poruka, .. }) if poruka.contains("disk je pun")
    ));
    assert!(matches!(
        dijalog_za_pregled(Some("upozorenje"), &k),
        Some(AktivniDijalog::UpozorenjeBrisanja {
            broj_nepodrzanih: 5,
            ..
        })
    ));
    assert!(matches!(
        dijalog_za_pregled(Some("upozorenja-otvaranja"), &k),
        Some(AktivniDijalog::UpozorenjaOtvaranja { upozorenja, .. }) if upozorenja.len() == 3
    ));
    assert_eq!(
        dijalog_za_pregled(Some("o-programu"), &k),
        Some(AktivniDijalog::OProgramu)
    );
    assert_eq!(dijalog_za_pregled(Some("nepoznato"), &k), None);
    assert_eq!(dijalog_za_pregled(None, &k), None);
}

/// Izbornik Pomoć i gumb „?” otvaraju „O programu” s inačicom, licencom i izvorom karte.
#[test]
fn pomoc_otvara_o_programu() {
    let app = Aplikacija::nova(PostavkeAplikacije::default());
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.run_steps(3);
    let k = crate::poruke::KatalogPoruka::hrvatski();
    let pomoc: Vec<_> = h
        .get_all_by_label("Pomoć")
        .map(|n| n.rect().center())
        .collect();
    assert_eq!(pomoc.len(), 2, "stavka izbornika i gumb");
    for mjesto in pomoc {
        assert!(h.state().aktivni_dijalog.is_none());
        h.input_mut().events.push(egui::Event::PointerMoved(mjesto));
        h.run_steps(1);
        for pritisnut in [true, false] {
            h.input_mut().events.push(egui::Event::PointerButton {
                pos: mjesto,
                button: egui::PointerButton::Primary,
                pressed: pritisnut,
                modifiers: Modifiers::NONE,
            });
            h.run_steps(1);
        }
        cekaj_smirenje(&mut h, Duration::from_millis(500));
        assert_eq!(h.state().aktivni_dijalog, Some(AktivniDijalog::OProgramu));
        let t = nacrtani_tekstovi(&h);
        let naslov = format!("TerraSimple {} alfa", env!("CARGO_PKG_VERSION"));
        assert!(t.contains(&naslov), "{t:?}");
        for redak in crate::dijalozi::retci_o_programu(&k) {
            assert!(t.contains(&redak), "nema retka {redak:?} u {t:?}");
        }
        assert!(t.iter().any(|t| t.contains("EUPL-1.2")));
        assert!(t.iter().any(|t| t.contains("OpenStreetMap")));
        h.get_by_label("Zatvori").click();
        cekaj_smirenje(&mut h, Duration::from_millis(500));
    }
    assert!(h.state().aktivni_dijalog.is_none());
}

/// Datoteka i Slojevi u izborniku i dalje rade, a ostale stavke ne otvaraju ništa.
#[test]
fn stavke_izbornika_rade_svaka_svoje() {
    let mut app = Aplikacija::nova(PostavkeAplikacije::default());
    let ploca = app.stanje.ploca_otvorena;
    app.otvori_stavku_izbornika("izbornik-datoteka");
    assert!(app.izbornik_datoteka_otvoren);
    app.otvori_stavku_izbornika("izbornik-slojevi");
    assert_eq!(app.stanje.ploca_otvorena, !ploca);
    assert!(app.aktivni_dijalog.is_none());
    app.otvori_stavku_izbornika("izbornik-pomoc");
    assert_eq!(app.aktivni_dijalog, Some(AktivniDijalog::OProgramu));
}

/// Gumb za temu mijenja svijetlu u tamnu i natrag.
#[test]
fn gumb_teme_mijenja_temu() {
    let app = Aplikacija::nova(PostavkeAplikacije::default());
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.run_steps(3);
    let naziv = crate::poruke::KatalogPoruka::hrvatski().tekst("gumb-tema");
    assert!(!h.state().stanje.tema_tamna);
    h.get_by_label(&naziv).click();
    cekaj_smirenje(&mut h, Duration::from_millis(500));
    assert!(h.state().stanje.tema_tamna);
    h.get_by_label(&naziv).click();
    cekaj_smirenje(&mut h, Duration::from_millis(500));
    assert!(!h.state().stanje.tema_tamna);
}
