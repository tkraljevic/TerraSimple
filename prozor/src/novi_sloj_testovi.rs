use super::*;
use crate::aplikacija::Aplikacija;
use crate::ploca::otvori_novi_sloj;
use crate::stanje::{Alat, Odabir};
use crate::tema::Razmaci;
use eframe::egui::{Event, Modifiers};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use slojevi::{NaredbaStabla, Novo};

fn prozor(app: Aplikacija) -> Harness<'static, Aplikacija> {
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.run_steps(3);
    h
}

fn mapa(u: &mut Uredjivac, iznad: Option<IdMape>, naziv: &str) -> IdMape {
    match u.izvrsi(NaredbaStabla::DodajMapu {
        u: iznad,
        naziv: naziv.into(),
    }) {
        Ok(Some(Novo::Mapa(m))) => m,
        r => panic!("{r:?}"),
    }
}

fn oznaceno(h: &Harness<'_, Aplikacija>, labela: &str) -> bool {
    h.get_by_label(labela).accesskit_node().toggled() == Some(egui::accesskit::Toggled::True)
}

#[test]
fn mape_su_redom_kao_u_stablu_s_dubinom() {
    let mut u = Uredjivac::novi();
    let a = mapa(&mut u, None, "A");
    let a1 = mapa(&mut u, Some(a), "A1");
    let b = mapa(&mut u, None, "B");
    assert_eq!(mape_stabla(u.projekt()), vec![(a, 0), (a1, 1), (b, 0)]);
    let k = KatalogPoruka::hrvatski();
    assert_eq!(naziv_mape(u.projekt(), &k, Some(a1)), "A1");
    assert_eq!(naziv_mape(u.projekt(), &k, None), "Na vrh, bez mape");
}

#[test]
fn gumb_novi_sloj_otvara_prozorcic_a_napravi_stvara_sloj_odabrane_vrste() {
    let mut app = Aplikacija::default();
    app.stanje.alat = Alat::Linija;
    let mut h = prozor(app);
    h.get_by_label("Novi sloj").click();
    h.run_steps(3);
    let novi = h
        .state()
        .stanje
        .novi_sloj
        .clone()
        .expect("prozorčić je otvoren");
    assert_eq!(novi.vrsta, VrstaGeometrije::Linija, "vrsta prema alatu");
    assert_eq!(novi.naziv, "Sloj 1");
    assert!(oznaceno(&h, "Linije"));
    assert!(!oznaceno(&h, "Točke"));
    h.get_by_label("Točke").click();
    h.run_steps(2);
    assert!(oznaceno(&h, "Točke"));
    h.get_by_label("Napravi sloj").click();
    h.run_steps(3);
    assert!(h.state().stanje.novi_sloj.is_none(), "prozorčić se zatvara");
    let (id, sloj) = h
        .state()
        .uredjivac
        .projekt()
        .slojevi()
        .next()
        .expect("sloj");
    assert_eq!(sloj.vrsta, VrstaGeometrije::Tocka);
    assert_eq!(sloj.naziv, "Sloj 1");
    assert_eq!(h.state().stanje.odabrano, Some(Odabir::Sloj(id)));
    assert_eq!(h.state().stanje.redni_novog_sloja, 2);
}

#[test]
fn naziv_se_upisuje_backspace_ne_brise_sloj_a_prazan_naziv_se_odbija() {
    let mut app = Aplikacija::default();
    let Ok(Some(Novo::Sloj(postojeci))) = app.uredjivac.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Postojeći".into(),
        vrsta: VrstaGeometrije::Poligon,
    }) else {
        panic!("sloj");
    };
    app.stanje.odabrano = Some(Odabir::Sloj(postojeci));
    let mut h = prozor(app);
    h.get_by_label("Novi sloj").click();
    h.run_steps(3);
    // Polje za naziv dobiva fokus samo od sebe, pa Backspace briše slova, a ne odabrani sloj.
    for _ in 0..7 {
        h.key_press(egui::Key::Backspace);
    }
    h.run_steps(2);
    assert_eq!(
        h.state()
            .stanje
            .novi_sloj
            .as_ref()
            .map(|n| n.naziv.as_str()),
        Some("")
    );
    assert!(h.state().uredjivac.projekt().sloj(postojeci).is_some());
    h.get_by_label("Napravi sloj").click();
    h.run_steps(3);
    assert!(
        h.state()
            .stanje
            .novi_sloj
            .as_ref()
            .is_some_and(|n| n.prazan_naziv),
        "bez naziva sloj ne nastaje"
    );
    assert!(h.query_by_label("Upiši naziv sloja.").is_some());
    h.get_by_label("Naziv").click();
    h.run_steps(1);
    h.input_mut().events.push(Event::Text("Bunari".into()));
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    h.run_steps(3);
    let nazivi: Vec<_> = h
        .state()
        .uredjivac
        .projekt()
        .slojevi()
        .map(|(_, s)| s.naziv.clone())
        .collect();
    assert_eq!(
        nazivi,
        vec!["Postojeći", "Bunari"],
        "Enter u nazivu stvara sloj"
    );
}

#[test]
fn odustani_i_escape_zatvaraju_bez_novog_sloja() {
    let mut h = prozor(Aplikacija::default());
    h.get_by_label("Novi sloj").click();
    h.run_steps(3);
    h.get_by_label("Odustani").click();
    h.run_steps(3);
    assert!(h.state().stanje.novi_sloj.is_none());
    h.get_by_label("Novi sloj").click();
    h.run_steps(3);
    h.input_mut().events.push(Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    });
    h.run_steps(3);
    assert!(h.state().stanje.novi_sloj.is_none());
    assert_eq!(h.state().uredjivac.projekt().slojevi().count(), 0);
}

#[test]
fn sloj_ide_u_mapu_odabranu_u_prozorcicu() {
    let mut app = Aplikacija::default();
    let gore = mapa(&mut app.uredjivac, None, "Gore");
    let dolje = mapa(&mut app.uredjivac, Some(gore), "Dolje");
    app.stanje.odabrano = Some(Odabir::Mapa(gore));
    let katalog = KatalogPoruka::hrvatski();
    otvori_novi_sloj(app.uredjivac.projekt(), &mut app.stanje, &katalog);
    assert_eq!(
        app.stanje.novi_sloj.as_ref().and_then(|n| n.mapa),
        Some(gore)
    );
    let mut h = prozor(app);
    if let Some(n) = h.state_mut().stanje.novi_sloj.as_mut() {
        n.mapa = Some(dolje);
    }
    h.run_steps(2);
    h.get_by_label("Napravi sloj").click();
    h.run_steps(3);
    let (_, sloj) = h
        .state()
        .uredjivac
        .projekt()
        .slojevi()
        .next()
        .expect("sloj");
    assert_eq!(sloj.roditelj, Some(dolje));
}
