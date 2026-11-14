use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::dijalozi::{AktivniDijalog, RedakPrecaca, retci_precaca};
use crate::poruke::KatalogPoruka;
use crate::tema::{Razmaci, font_obicni, font_srednji};
use eframe::egui::{self, Key, Modifiers, os::OperatingSystem, vec2};
use egui_kittest::{Harness, kittest::Queryable};

fn tipke(sustav: OperatingSystem) -> Vec<String> {
    let k = KatalogPoruka::hrvatski();
    retci_precaca(&k, sustav)
        .into_iter()
        .flatten()
        .filter_map(|r| match r {
            RedakPrecaca::Precac { tipke, .. } => Some(tipke),
            RedakPrecaca::Skupina(_) => None,
        })
        .collect()
}

#[test]
fn prečaci_kako_ih_pise_sustav() {
    let mac = tipke(OperatingSystem::Mac);
    let win = tipke(OperatingSystem::Windows);
    assert!(mac.contains(&"⌘Z".to_owned()));
    assert!(win.contains(&"Ctrl+Z".to_owned()));
    assert!(win.contains(&"Delete ili Ctrl+Backspace".to_owned()));
    assert!(mac.contains(&"desni klik, ⌃ + klik ili dva prsta".to_owned()));
    assert!(win.contains(&"desni klik".to_owned()));
    assert!(mac.contains(&"dva prsta gore-dolje ili štipanje".to_owned()));
    assert!(win.contains(&"kotačić miša".to_owned()));
    assert_eq!(mac.len(), win.len());
    let k = KatalogPoruka::hrvatski();
    let [lijevo, desno] = retci_precaca(&k, OperatingSystem::Mac);
    assert_eq!(lijevo[0], RedakPrecaca::Skupina("Datoteka".into()));
    assert_eq!(desno[0], RedakPrecaca::Skupina("Crtanje i točke".into()));
}

/// Svaki znak prečaca mora postojati u fontu programa, inače bi egui pao pri crtanju.
#[test]
fn font_ima_sve_znakove_precaca() {
    let ctx = egui::Context::default();
    Aplikacija::postavi_kontekst(&ctx);
    let mut izlaz = ctx.run_ui(egui::RawInput::default(), |_| {});
    izlaz.textures_delta.clear();
    let k = KatalogPoruka::hrvatski();
    for sustav in [OperatingSystem::Mac, OperatingSystem::Windows] {
        for redak in retci_precaca(&k, sustav).into_iter().flatten() {
            let (tekst, font) = match &redak {
                RedakPrecaca::Skupina(t) => (t.clone(), font_srednji(13.0)),
                RedakPrecaca::Precac { radnja, tipke } => {
                    (format!("{radnja}{tipke}"), font_obicni(12.5))
                }
            };
            assert!(
                ctx.fonts_mut(|f| f.has_glyphs(&font, &tekst)),
                "nema znaka u: {tekst}"
            );
        }
    }
}

fn prozor() -> Harness<'static, Aplikacija> {
    let app = Aplikacija::nova(PostavkeAplikacije::default());
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.run_steps(3);
    h
}

#[test]
fn f1_cmd_kosa_crta_i_gumb_pomoci_otvaraju_precace() {
    let mut h = prozor();
    h.key_press(Key::F1);
    h.run_steps(3);
    assert_eq!(h.state().aktivni_dijalog, Some(AktivniDijalog::Precaci));
    h.get_by_label("Zatvori").click();
    h.run_steps(3);
    assert!(h.state().aktivni_dijalog.is_none());

    h.key_press_modifiers(Modifiers::COMMAND, Key::Slash);
    h.run_steps(3);
    assert_eq!(h.state().aktivni_dijalog, Some(AktivniDijalog::Precaci));
    h.get_by_label("Zatvori").click();
    h.run_steps(3);

    let gumb = h.get_all_by_label("Pomoć").last().expect("gumb pomoći");
    gumb.click();
    h.run_steps(3);
    assert_eq!(h.state().aktivni_dijalog, Some(AktivniDijalog::Precaci));
}

/// Prozor sam prepoznaje sustav: na Macu ⌃ + klik postaje desni klik.
#[test]
fn prozor_na_macu_pretvara_ctrl_klik() {
    use eframe::App;
    let mut app = Aplikacija::nova(PostavkeAplikacije::default());
    let klik = || egui::Event::PointerButton {
        pos: egui::pos2(5.0, 5.0),
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: Modifiers::CTRL,
    };
    for (sustav, ocekivano) in [
        (OperatingSystem::Mac, egui::PointerButton::Secondary),
        (OperatingSystem::Windows, egui::PointerButton::Primary),
    ] {
        let ctx = egui::Context::default();
        ctx.set_os(sustav);
        let mut unos = egui::RawInput {
            events: vec![klik()],
            ..egui::RawInput::default()
        };
        app.raw_input_hook(&ctx, &mut unos);
        let egui::Event::PointerButton { button, .. } = unos.events[0] else {
            panic!("klik");
        };
        assert_eq!(button, ocekivano, "{sustav:?}");
    }
}
