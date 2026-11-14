use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::stanje::{Alat, Odabir};
use crate::tema::Razmaci;
use eframe::egui::{
    Event, Key, Modifiers, MouseWheelSource, MouseWheelUnit, PointerButton, Pos2, TouchPhase, pos2,
    vec2,
};
use egui_kittest::Harness;
use slojevi::{Geometrija, IdSloja, NaredbaObjekta, NaredbaStabla, Novo, Vrh, VrstaGeometrije};

fn prozor(app: Aplikacija) -> Harness<'static, Aplikacija> {
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.run_steps(3);
    h
}

fn bez_mreze() -> Aplikacija {
    Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    })
}

fn dodaj_sloj(app: &mut Aplikacija, naziv: &str) -> IdSloja {
    let Ok(Some(Novo::Sloj(s))) = app.uredjivac.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: naziv.to_owned(),
        vrsta: VrstaGeometrije::Tocka,
    }) else {
        panic!("sloj {naziv}");
    };
    s
}

/// Sredina ploče Slojevi, koja je zadano uz desni rub karte.
fn sredina_ploce() -> Pos2 {
    pos2(
        Razmaci::SIRINA_PROZORA - Razmaci::RUB - Razmaci::SIRINA_PLOCE / 2.0,
        Razmaci::VISINA_PROZORA / 2.0,
    )
}

fn kotacic(h: &mut Harness<'_, Aplikacija>, gdje: Pos2, dolje: f32) {
    h.input_mut().events.push(Event::PointerMoved(gdje));
    h.run_steps(1);
    h.input_mut().events.push(Event::MouseWheel {
        unit: MouseWheelUnit::Point,
        delta: vec2(0.0, -dolje),
        phase: TouchPhase::Move,
        source: MouseWheelSource::Unknown,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(20);
}

fn klik(h: &mut Harness<'_, Aplikacija>, m: Pos2) {
    let tipka = |pressed| Event::PointerButton {
        pos: m,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    h.input_mut().events.push(Event::PointerMoved(m));
    h.run_steps(1);
    h.input_mut().events.push(tipka(true));
    h.run_steps(1);
    h.input_mut().events.push(tipka(false));
    h.run_steps(5);
}

fn trazi(h: &mut Harness<'_, Aplikacija>, tekst: &str) {
    h.input_mut().events.push(Event::Key {
        key: Key::F,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::COMMAND,
    });
    h.run_steps(2);
    h.input_mut().events.push(Event::Text(tekst.to_owned()));
    h.run_steps(3);
}

#[test]
fn kotacic_iznad_ploce_pomice_stablo_a_karta_ostaje_na_istom_zumu() {
    let mut app = bez_mreze();
    for i in 0..40 {
        dodaj_sloj(&mut app, &format!("Sloj {i}"));
    }
    let mut h = prozor(app);
    let zum = h.state_mut().karta.karta().memorija().zoom();

    kotacic(&mut h, sredina_ploce(), 300.0);

    assert!(h.state().stanje.pomak_stabla > 0.0);
    let poslije = h.state_mut().karta.karta().memorija().zoom();
    assert!((poslije - zum).abs() < 1e-9, "{zum} → {poslije}");

    // Daleko dolje stablo staje na zadnjem retku, pa se vraća na vrh.
    kotacic(&mut h, sredina_ploce(), 100_000.0);
    let najvise = h.state().stanje.pomak_stabla;
    assert!(najvise < 40.0 * Razmaci::VISINA_RETKA_STABLA);
    kotacic(&mut h, sredina_ploce(), -100_000.0);
    assert_eq!(h.state().stanje.pomak_stabla, 0.0);
}

#[test]
fn bez_dovoljno_redaka_stablo_ne_klizi() {
    let mut app = bez_mreze();
    dodaj_sloj(&mut app, "Jedini");
    let mut h = prozor(app);
    kotacic(&mut h, sredina_ploce(), 300.0);
    assert_eq!(h.state().stanje.pomak_stabla, 0.0);
}

#[test]
fn sloj_nadjen_trazilicom_stablo_dovodi_u_vidno_polje() {
    let mut app = bez_mreze();
    for i in 0..40 {
        dodaj_sloj(&mut app, &format!("Sloj {i}"));
    }
    let zadnji = dodaj_sloj(&mut app, "Zadnji");
    let mut h = prozor(app);
    assert_eq!(h.state().stanje.pomak_stabla, 0.0);

    trazi(&mut h, "Zadnji");
    h.input_mut().events.push(Event::Key {
        key: Key::Enter,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(4);

    assert_eq!(h.state().stanje.odabrano, Some(Odabir::Sloj(zadnji)));
    assert!(h.state().stanje.pomak_stabla > 0.0);
    assert!(!h.state().stanje.pokazi_odabrano);
}

#[test]
fn popis_rezultata_lezi_iznad_alata() {
    let mut app = bez_mreze();
    let sloj = dodaj_sloj(&mut app, "Točke");
    for i in 0..6 {
        app.uredjivac
            .izvrsi(NaredbaObjekta::Dodaj {
                sloj,
                naziv: format!("Kanal {i}"),
                geometrija: Geometrija::Tocka(Vrh::novi(18.0 + f64::from(i) * 0.01, 45.5)),
            })
            .expect("objekt");
    }
    let mut h = prozor(app);
    trazi(&mut h, "kanal");

    // Gumb „Točka” u traci alata, koji leži ispod trećeg retka rezultata.
    let gumb_tocka = pos2(
        Razmaci::RUB + 34.0,
        Razmaci::VISINA_IZBORNIKA + 68.0 + 6.0 + 50.0 + 23.0,
    );
    klik(&mut h, gumb_tocka);

    assert_eq!(h.state().stanje.alat, Alat::Odaberi);
    assert!(matches!(
        h.state().stanje.odabrano,
        Some(Odabir::Objekt(s, _)) if s == sloj
    ));
}
