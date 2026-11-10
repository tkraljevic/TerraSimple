use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::stanje::{Odabir, UredjivanjeStila};
use crate::stil::{
    IzmjenaStila, PONUDJENE_BOJE, izvrsi, prikaz, primijeni, primijeni_na_objekt, prozirnost,
    vrati_zadano,
};
use crate::tema::Razmaci;
use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, pos2, vec2};
use egui_kittest::{Harness, kittest::Queryable};
use slojevi::{
    Boja, Geometrija, IdMape, IdObjekta, IdSloja, NaredbaObjekta, NaredbaStabla, Novo, Stavka,
    Stil, StilObjekta, Uredjivac, Vrh, VrstaGeometrije,
};

const PLAVA: Boja = PONUDJENE_BOJE[3].0;

fn sloj_s_objektom(vrsta: VrstaGeometrije) -> (Uredjivac, IdSloja, IdObjekta) {
    let mut u = Uredjivac::novi();
    let Ok(Some(Novo::Sloj(s))) = u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Kanali".into(),
        vrsta,
    }) else {
        panic!("sloj");
    };
    let v = |x: f64, y: f64| Vrh::novi(18.6 + x, 45.5 + y);
    let geometrija = match vrsta {
        VrstaGeometrije::Tocka => Geometrija::Tocka(v(0.0, 0.0)),
        VrstaGeometrije::Linija => Geometrija::Linija(vec![v(0.0, 0.0), v(0.01, 0.01)]),
        VrstaGeometrije::Poligon => {
            Geometrija::Poligon(vec![vec![v(0.0, 0.0), v(0.01, 0.0), v(0.01, 0.01)]])
        }
    };
    let Ok(Some(Novo::Objekt(o))) = u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: "Kanal 1".into(),
        geometrija,
    }) else {
        panic!("objekt");
    };
    (u, s, o)
}

#[test]
fn boja_lika_mijenja_crtu_i_ispunu_a_prozirnost_ostaje() {
    let s = primijeni(Stil::default(), IzmjenaStila::Boja(PLAVA));
    assert_eq!(
        (s.boja_crte.r, s.boja_crte.g, s.boja_crte.b),
        (0x15, 0x65, 0xC0)
    );
    assert_eq!(s.boja_crte.a, 255);
    assert_eq!(s.boja_ispune.b, 0xC0);
    assert_eq!(s.boja_ispune.a, Stil::default().boja_ispune.a);

    let s = primijeni(s, IzmjenaStila::ProzirnostIspune(0.5));
    assert_eq!(s.boja_ispune.a, 128);
    let s = primijeni(s, IzmjenaStila::Prozirnost(1.5));
    assert_eq!(s.boja_crte.a, 0, "prozirnost iznad 100 % je 100 %");
    assert!((prozirnost(s.boja_crte) - 1.0).abs() < 1e-9);
    let s = primijeni(s, IzmjenaStila::Debljina(2.5));
    assert_eq!(s.debljina_mm, 2.5);
    let s = primijeni(s, IzmjenaStila::Velicina(6.0));
    assert_eq!(s.velicina_znaka_mm, 6.0);
}

#[test]
fn objekt_pamti_samo_izmijenjeno_a_ostalo_uzima_od_sloja() {
    let sloj = Stil::default();
    let o = primijeni_na_objekt(StilObjekta::default(), &sloj, IzmjenaStila::Debljina(3.0));
    assert_eq!(o.debljina_mm, Some(3.0));
    assert_eq!(o.boja_crte, None);
    let o = primijeni_na_objekt(o, &sloj, IzmjenaStila::Boja(PLAVA));
    assert_eq!(o.boja_crte.map(|b| b.b), Some(0xC0));
    assert_eq!(o.boja_ispune.map(|b| b.a), Some(sloj.boja_ispune.a));
    let o = primijeni_na_objekt(o, &sloj, IzmjenaStila::Prozirnost(0.0));
    assert_eq!(o.boja_crte.map(|b| b.a), Some(255));
    let o = primijeni_na_objekt(o, &sloj, IzmjenaStila::ProzirnostIspune(1.0));
    assert_eq!(o.boja_ispune.map(|b| b.a), Some(0));
    let o = primijeni_na_objekt(o, &sloj, IzmjenaStila::Velicina(4.0));
    assert_eq!(o.velicina_znaka_mm, Some(4.0));
}

#[test]
fn izmjena_objekta_ne_dira_sloj_i_vraca_se_na_stil_sloja() {
    let (mut u, s, o) = sloj_s_objektom(VrstaGeometrije::Linija);
    let mut ured = UredjivanjeStila::za(Odabir::Objekt(s, o)).expect("objekt");
    assert!(ured.samo_objekt, "iz retka objekta mijenja se objekt");
    assert!(izvrsi(&mut u, &mut ured, IzmjenaStila::Debljina(2.0)));
    assert_eq!(u.projekt().sloj(s).expect("sloj").stil, Stil::default());
    let p = prikaz(u.projekt(), &ured).expect("prikaz");
    assert_eq!(p.naziv, "Kanal 1");
    assert_eq!(p.stil.debljina_mm, 2.0);

    assert!(vrati_zadano(&mut u, &mut ured));
    assert_eq!(
        u.projekt().objekt(s, o).expect("objekt").stil,
        StilObjekta::default()
    );

    ured.samo_objekt = false;
    ured.potez = false;
    assert!(izvrsi(&mut u, &mut ured, IzmjenaStila::Debljina(1.5)));
    assert_eq!(u.projekt().sloj(s).expect("sloj").stil.debljina_mm, 1.5);
    assert_eq!(prikaz(u.projekt(), &ured).expect("prikaz").naziv, "Kanali");
    assert!(vrati_zadano(&mut u, &mut ured));
    assert_eq!(u.projekt().sloj(s).expect("sloj").stil, Stil::default());
}

#[test]
fn potez_se_spaja_u_jedan_korak_a_nestali_sloj_ne_mijenja_nista() {
    let (mut u, s, o) = sloj_s_objektom(VrstaGeometrije::Poligon);
    let mut ured = UredjivanjeStila::za(Odabir::Sloj(s)).expect("sloj");
    assert!(!ured.samo_objekt);
    assert!(izvrsi(
        &mut u,
        &mut ured,
        IzmjenaStila::ProzirnostIspune(0.1)
    ));
    assert!(ured.potez);
    assert!(izvrsi(
        &mut u,
        &mut ured,
        IzmjenaStila::ProzirnostIspune(0.2)
    ));
    assert!(izvrsi(
        &mut u,
        &mut ured,
        IzmjenaStila::ProzirnostIspune(0.3)
    ));
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.projekt().sloj(s).expect("sloj").stil, Stil::default());
    assert!(u.projekt().objekt(s, o).is_some(), "poništen je samo potez");

    assert!(UredjivanjeStila::za(Odabir::Mapa(IdMape(1))).is_none());
    u.izvrsi(NaredbaObjekta::Obrisi { sloj: s, objekt: o })
        .expect("brisanje");
    let mut za_objekt = UredjivanjeStila::za(Odabir::Objekt(s, o)).expect("objekt");
    assert!(prikaz(u.projekt(), &za_objekt).is_none());
    assert!(!izvrsi(&mut u, &mut za_objekt, IzmjenaStila::Debljina(1.0)));
    u.izvrsi(NaredbaStabla::Obrisi {
        stavka: Stavka::Sloj(s),
    })
    .expect("brisanje sloja");
    assert!(!izvrsi(&mut u, &mut ured, IzmjenaStila::Debljina(1.0)));
    assert!(prikaz(u.projekt(), &ured).is_none());
}

// Ploča u prozoru.

fn prozor() -> Harness<'static, Aplikacija> {
    let app = Aplikacija::nova(PostavkeAplikacije {
        primjer: true,
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

fn izbornik(h: &mut Harness<'_, Aplikacija>, redak: &str, stavka: &str) {
    h.get_by_label(redak).click_secondary();
    h.run_steps(4);
    h.get_by_label(stavka).click();
    h.run_steps(4);
}

fn klik(h: &mut Harness<'_, Aplikacija>, labela: &str) {
    h.get_by_label(labela).click();
    h.run_steps(4);
}

fn sloj_po_nazivu(h: &Harness<'_, Aplikacija>, naziv: &str) -> IdSloja {
    h.state()
        .uredjivac
        .projekt()
        .slojevi()
        .find(|(_, s)| s.naziv == naziv)
        .map(|(id, _)| id)
        .expect("sloj")
}

fn stil_sloja(h: &Harness<'_, Aplikacija>, naziv: &str) -> Stil {
    let id = sloj_po_nazivu(h, naziv);
    h.state().uredjivac.projekt().sloj(id).expect("sloj").stil
}

/// Povlači miš po stazi klizača, u nekoliko koraka.
fn povuci(h: &mut Harness<'_, Aplikacija>, od: Pos2, do_: Pos2) {
    let tipka = |pos, pressed| Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    h.input_mut().events.push(Event::PointerMoved(od));
    h.run_steps(1);
    h.input_mut().events.push(tipka(od, true));
    h.run_steps(1);
    for i in 1..=5 {
        let p = od + (do_ - od) * (i as f32 / 5.0);
        h.input_mut().events.push(Event::PointerMoved(p));
        h.run_steps(1);
    }
    h.input_mut().events.push(tipka(do_, false));
    h.run_steps(3);
}

#[test]
fn stil_sloja_iz_izbornika_mijenja_boju_i_debljinu_a_potez_je_jedan_korak() {
    let mut h = prozor();
    izbornik(&mut h, "Trase", "Stil");
    assert!(h.state().stanje.stil.is_some());
    assert!(
        h.query_by_label("Cijeli sloj").is_none(),
        "iz sloja nema izbora"
    );
    assert!(h.query_by_label("Prozirnost").is_some());

    klik(&mut h, "Plava");
    assert_eq!(stil_sloja(&h, "Trase").boja_crte.b, 0xC0);

    let staza = h.get_by_label("Debljina").rect();
    let y = staza.center().y;
    let (od, do_) = (pos2(staza.min.x + 8.0, y), pos2(staza.max.x - 8.0, y));
    povuci(&mut h, od, do_);
    assert_eq!(stil_sloja(&h, "Trase").debljina_mm, 5.0);
    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run_steps(3);
    assert_eq!(
        stil_sloja(&h, "Trase").debljina_mm,
        Stil::default().debljina_mm,
        "cijelo povlačenje poništava se odjednom"
    );
    assert_eq!(stil_sloja(&h, "Trase").boja_crte.b, 0xC0, "boja ostaje");

    klik(&mut h, "Vrati zadano");
    assert_eq!(stil_sloja(&h, "Trase"), Stil::default());
    klik(&mut h, "Gotovo");
    assert!(h.state().stanje.stil.is_none());
    assert!(
        h.query_by_label("Novi sloj").is_some(),
        "opet ploča Slojevi"
    );
}

#[test]
fn stil_objekta_samo_za_njega_ili_za_cijeli_sloj() {
    let mut h = prozor();
    izbornik(&mut h, "Parcela 1126", "Stil");
    let s = sloj_po_nazivu(&h, "Parcele");
    assert!(h.query_by_label("Prozirnost ispune").is_some());
    assert!(h.query_by_label("Debljina ruba").is_some());

    klik(&mut h, "Zelena");
    let objekt = h
        .state()
        .uredjivac
        .projekt()
        .sloj(s)
        .expect("sloj")
        .objekti
        .values()
        .find(|o| o.naziv == "Parcela 1126")
        .expect("objekt")
        .stil;
    assert_eq!(objekt.boja_crte.map(|b| b.g), Some(0x7D));
    assert_eq!(stil_sloja(&h, "Parcele"), Stil::default(), "sloj ostaje");

    klik(&mut h, "Cijeli sloj");
    klik(&mut h, "Crvena");
    assert_eq!(stil_sloja(&h, "Parcele").boja_crte.r, 0xD3);

    klik(&mut h, "Samo ovaj objekt");
    assert!(h.state().stanje.stil.expect("stil").samo_objekt);
    klik(&mut h, "Natrag na slojeve");
    assert!(h.state().stanje.stil.is_none());
}

#[test]
fn tocka_ima_velicinu_druga_boja_otvara_birac_a_x_zatvara_plocu() {
    let mut h = prozor();
    izbornik(&mut h, "Točke", "Stil");
    assert!(h.query_by_label("Veličina").is_some());
    assert!(h.query_by_label("Debljina").is_none());

    // Strelica pomiče klizač za jedan korak.
    h.get_by_label("Veličina").focus();
    h.run_steps(2);
    h.key_press(Key::ArrowRight);
    h.run_steps(2);
    assert_eq!(stil_sloja(&h, "Točke").velicina_znaka_mm, 3.5);
    h.key_press(Key::ArrowLeft);
    h.run_steps(2);
    assert_eq!(stil_sloja(&h, "Točke").velicina_znaka_mm, 3.0);

    let klizaci_prije = h
        .query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::Slider))
        .count();
    klik(&mut h, "Druga boja");
    let klizaci_poslije = h
        .query_all(egui_kittest::kittest::by().role(egui::accesskit::Role::Slider))
        .count();
    assert_eq!(klizaci_poslije, klizaci_prije + 3, "birač boje je otvoren");
    let staza = h.get_by_label("Nijansa").rect();
    let y = staza.center().y;
    let srediste = pos2(staza.center().x, y);
    povuci(&mut h, srediste, srediste + vec2(20.0, 0.0));
    let boja = stil_sloja(&h, "Točke").boja_crte;
    assert_ne!(
        (boja.r, boja.g, boja.b),
        (0xE6, 0x51, 0x00),
        "boja iz birača"
    );
    assert_eq!(boja.a, 255);
    h.key_press(Key::Escape);
    h.run_steps(3);
    klik(&mut h, "Zatvori ploču Stil");
    assert!(h.state().stanje.stil.is_none());
    assert!(!h.state().stanje.ploca_otvorena);
}

#[test]
fn zakljucan_sloj_pokazuje_napomenu_a_mapa_nema_stil() {
    let mut h = prozor();
    h.get_by_label("Teren 2026").click_secondary();
    h.run_steps(4);
    assert!(h.query_by_label("Stil").is_none(), "mapa nema stil");
    h.key_press(Key::Escape);
    h.run_steps(3);

    let nasipi = sloj_po_nazivu(&h, "nasipi");
    h.state_mut().stanje.stil = UredjivanjeStila::za(Odabir::Sloj(nasipi));
    h.run_steps(3);
    assert!(h.query_by_label("Boja").is_none());
    assert!(h.query_by_label("Vrati zadano").is_none());
    assert!(h.query_by_label("Natrag na slojeve").is_some());

    // Obrisan sloj zatvara ploču.
    h.state_mut()
        .uredjivac
        .postavi_stanje_sloja(nasipi, slojevi::StanjeSloja::Uredivo)
        .expect("otključavanje");
    h.state_mut()
        .uredjivac
        .izvrsi(NaredbaStabla::Obrisi {
            stavka: Stavka::Sloj(nasipi),
        })
        .expect("brisanje");
    h.run_steps(2);
    assert!(h.state().stanje.stil.is_none());
}
