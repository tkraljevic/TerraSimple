//! Karta sa stvarnim događajima miša u `egui`-ju, bez grafičke kartice i bez podloge.

use super::*;
use crate::Alat;
use eframe::egui::{Context, Event, RawInput, Rect, Vec2, pos2};
use slojevi::{Geometrija, Mjere, VrstaGeometrije};

const SEKUNDI_PO_OKVIRU: f64 = 0.016;
/// Dulje od `egui`-jeva razmaka za dvoklik, da dva klika ostanu dva klika.
const STANKA_S: f64 = 0.5;

fn vidljiv(_: IdSloja) -> bool {
    true
}
fn naziv(_: VrstaGeometrije) -> String {
    "Novi".into()
}
fn opis(_: &Mjere) -> String {
    "mjera".into()
}
fn naziv_objekta(_: VrstaGeometrije, redni: usize) -> String {
    format!("Objekt {redni}")
}
fn duljina(_: f64) -> String {
    "dio".into()
}

const OK: Okolina<'static> = Okolina {
    vidljiv: &vidljiv,
    naziv_sloja: &naziv,
    opis_mjere: &opis,
    opis_duljine: &duljina,
    naziv_objekta: &naziv_objekta,
};

/// `egui` bez slova ne može složiti natpis, a program svoja slova ugrađuje tek u `prozor`-u.
fn kontekst() -> Context {
    let ctx = Context::default();
    let mut slova = egui::FontDefinitions::empty();
    let inter =
        egui::FontData::from_static(include_bytes!("../../probe/izgled/fonti/Inter-Medium.otf"));
    slova.font_data.insert("inter".into(), inter.into());
    for obitelj in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        slova.families.insert(obitelj, vec!["inter".into()]);
    }
    ctx.set_fonts(slova);
    ctx
}

struct Proba {
    ctx: Context,
    karta: Karta,
    u: Uredjivac,
    vrijeme: f64,
    /// Oblici zadnjeg okvira, da se vidi je li išta nacrtano.
    oblika: usize,
    tekstovi: Vec<String>,
    /// Ispunjeni krugovi zadnjeg okvira: sredina i boja.
    krugovi: Vec<(Pos2, egui::Color32)>,
}

impl Proba {
    fn nova(alat: Alat) -> Self {
        let mut karta = Karta::bez_podloge(18.69, 45.55);
        karta.memorija().set_zoom(14.0).expect("zum");
        karta.crtanje().postavi_alat(alat);
        let mut p = Self {
            ctx: kontekst(),
            karta,
            u: Uredjivac::novi(),
            vrijeme: 0.0,
            oblika: 0,
            tekstovi: Vec::new(),
            krugovi: Vec::new(),
        };
        p.okvir(vec![Event::PointerMoved(pos2(1.0, 1.0))]);
        p
    }

    fn okvir(&mut self, dogadaji: Vec<Event>) {
        self.vrijeme += SEKUNDI_PO_OKVIRU;
        let ulaz = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(800.0))),
            time: Some(self.vrijeme),
            events: dogadaji,
            ..Default::default()
        };
        let (karta, u) = (&mut self.karta, &mut self.u);
        let mut izlaz = self.ctx.run_ui(ulaz, |ui| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| karta.prikazi(ui, u, &OK));
        });
        izlaz.textures_delta.clear();
        self.oblika = izlaz.shapes.len();
        self.krugovi = izlaz
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Circle(k) => Some((k.center, k.fill)),
                _ => None,
            })
            .collect();
        self.tekstovi = izlaz
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
                _ => None,
            })
            .collect();
    }

    fn tipka(&mut self, m: Pos2, gumb: PointerButton, pritisnuta: bool) {
        self.okvir(vec![Event::PointerButton {
            pos: m,
            button: gumb,
            pressed: pritisnuta,
            modifiers: Modifiers::NONE,
        }]);
    }

    fn klik_gumbom(&mut self, m: Pos2, gumb: PointerButton) {
        self.okvir(vec![Event::PointerMoved(m)]);
        self.tipka(m, gumb, true);
        self.tipka(m, gumb, false);
    }

    /// Klik, a zatim stanka, da sljedeći klik ne bude dvoklik.
    fn klik(&mut self, m: Pos2) {
        self.klik_gumbom(m, PointerButton::Primary);
        self.vrijeme += STANKA_S;
    }

    /// Zadnja točka pa desni klik, koji završava crtanje.
    fn klik_pa_desni(&mut self, m: Pos2) {
        self.klik(m);
        self.klik_gumbom(m, PointerButton::Secondary);
        self.vrijeme += STANKA_S;
    }

    fn povuci(&mut self, od: Pos2, koraci: usize, korak: Vec2) -> Pos2 {
        self.okvir(vec![Event::PointerMoved(od)]);
        self.tipka(od, PointerButton::Primary, true);
        let mut m = od;
        for _ in 0..koraci {
            m += korak;
            self.okvir(vec![Event::PointerMoved(m)]);
        }
        self.tipka(m, PointerButton::Primary, false);
        m
    }

    fn sredina(&mut self) -> [f64; 2] {
        let p = self.karta.memorija().detached();
        p.map_or([18.69, 45.55], |p| [p.x(), p.y()])
    }

    fn pomak_px(&mut self, od: [f64; 2]) -> f64 {
        let s = self.sredina();
        // Na zumu 14 svijet ima 256 * 2^14 točaka.
        let po_stupnju = 256.0 * 16384.0 / 360.0;
        ((s[0] - od[0]) * po_stupnju)
            .abs()
            .max(((s[1] - od[1]) * po_stupnju * 1.4).abs())
    }

    fn objekata(&self) -> usize {
        self.u
            .projekt()
            .slojevi()
            .map(|(_, s)| s.objekti.len())
            .sum()
    }

    fn zadnji_vrhova(&self) -> usize {
        let o = self
            .u
            .projekt()
            .slojevi()
            .flat_map(|(_, s)| s.objekti.values())
            .last();
        match o.map(|x| &x.geometrija) {
            Some(Geometrija::Linija(v)) => v.len(),
            Some(Geometrija::Poligon(p)) => p[0].len(),
            _ => 0,
        }
    }
}

const A: Pos2 = pos2(100.0, 100.0);
const B: Pos2 = pos2(300.0, 100.0);
const C: Pos2 = pos2(300.0, 300.0);

#[test]
fn klikovi_i_desni_klik_crtaju_poligon_na_pravoj_karti() {
    let mut p = Proba::nova(Alat::Poligon);
    p.klik(A);
    p.klik(B);
    p.klik(C);
    assert_eq!(p.objekata(), 0, "još se crta");
    p.klik_pa_desni(pos2(100.0, 300.0));
    assert_eq!(p.objekata(), 1);
    assert_eq!(
        p.zadnji_vrhova(),
        4,
        "dvoklik dodaje jednu točku, a ne dvije"
    );
    assert_eq!(p.karta.crtanje().alat(), Alat::Odaberi);
    // Pretvorba je povratna: točka je tamo gdje je kliknuta.
    let (sloj, o) = p.karta.crtanje().odabran().expect("odabran");
    let g =
        p.u.projekt()
            .objekt(sloj, o)
            .expect("objekt")
            .geometrija
            .clone();
    let Geometrija::Poligon(r) = g else {
        panic!("poligon");
    };
    assert!(r[0][0].duzina < r[0][1].duzina, "{:?}", r[0]);
    assert!(r[0][1].sirina > r[0][2].sirina, "y raste prema jugu");
}

#[test]
fn povlacenje_tocke_ne_pomice_kartu() {
    let mut p = Proba::nova(Alat::Linija);
    p.klik(A);
    p.klik(B);
    p.klik_pa_desni(C);
    let sredina = p.sredina();
    p.povuci(A, 6, Vec2::new(10.0, 5.0));
    assert!(p.pomak_px(sredina) < 1e-6, "karta se pomaknula");
    assert_eq!(p.u.ponisti(), Ok(true), "vrh je pomaknut naredbom");
}

#[test]
fn povlacenje_izvan_tocke_pomice_kartu() {
    let mut p = Proba::nova(Alat::Odaberi);
    let sredina = p.sredina();
    p.povuci(pos2(400.0, 400.0), 6, Vec2::new(3.0, 0.0));
    assert!(p.pomak_px(sredina) > 5.0);
    assert_eq!(p.objekata(), 0);
}

#[test]
fn klik_dok_karta_klizi_je_samo_zaustavlja_a_sljedeci_upisuje_tocku() {
    let mut p = Proba::nova(Alat::Linija);
    p.povuci(pos2(400.0, 400.0), 6, Vec2::new(25.0, -15.0));
    assert!(p.karta.memorija().animating(), "brzo povlačenje klizi");
    p.okvir(Vec::new());
    assert!(p.karta.memorija().animating());

    let m = pos2(200.0, 200.0);
    p.okvir(vec![Event::PointerMoved(m)]);
    p.tipka(m, PointerButton::Primary, true);
    assert!(!p.karta.memorija().animating(), "pritisak zaustavlja kartu");
    let stala = p.sredina();
    p.tipka(m, PointerButton::Primary, false);
    assert!(
        p.karta.crtanje().crtez.is_none(),
        "taj klik ne upisuje točku"
    );
    for _ in 0..10 {
        p.okvir(Vec::new());
    }
    assert!(p.pomak_px(stala) < 1e-6, "karta stoji");

    p.vrijeme += STANKA_S;
    p.klik(m);
    let c = p.karta.crtanje().crtez.as_ref().expect("crtež");
    assert_eq!(c.vrhovi.len(), 1, "sljedeći klik upisuje točku");
}

#[test]
fn povlacenje_dok_karta_klizi_samo_pomice_kartu() {
    let mut p = Proba::nova(Alat::Linija);
    p.povuci(pos2(400.0, 400.0), 6, Vec2::new(25.0, -15.0));
    let sredina = p.sredina();
    p.povuci(pos2(200.0, 200.0), 5, Vec2::new(4.0, 0.0));
    assert!(p.pomak_px(sredina) > 1.0);
    assert!(p.karta.crtanje().crtez.is_none());
    // I polagano povlačenje malo klizne, pa se klik tek nakon toga računa kao klik.
    for _ in 0..300 {
        p.okvir(Vec::new());
    }
    assert!(!p.karta.memorija().animating());
    p.vrijeme += STANKA_S;
    p.klik(pos2(200.0, 200.0));
    assert!(
        p.karta.crtanje().crtez.is_some(),
        "gutanje klika nije ostalo uključeno"
    );
}

#[test]
fn polagano_povlacenje_ne_ostavlja_gutanje_klika() {
    let mut p = Proba::nova(Alat::Linija);
    p.klik(A);
    assert!(p.karta.crtanje().crtez.is_some());
}

#[test]
fn backspace_brise_tocku_a_desni_klik_zavrsava_na_karti() {
    let mut p = Proba::nova(Alat::Linija);
    p.klik(A);
    p.klik(B);
    p.klik(C);
    let tipka = |key| Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    p.okvir(vec![tipka(Key::Backspace)]);
    let vrhova = |p: &mut Proba| {
        p.karta
            .crtanje()
            .crtez
            .as_ref()
            .map_or(0, |c| c.vrhovi.len())
    };
    assert_eq!(vrhova(&mut p), 2);
    p.klik_gumbom(B, PointerButton::Secondary);
    assert_eq!(p.objekata(), 1, "desni klik je završio liniju");
    assert_eq!(p.zadnji_vrhova(), 2);

    p.karta.crtanje().postavi_alat(Alat::Linija);
    p.klik(A);
    p.okvir(vec![tipka(Key::Escape)]);
    assert!(p.karta.crtanje().crtez.is_none());
}

#[test]
fn priprema_se_ponavlja_samo_kad_se_sloj_promijeni() {
    let mut p = Proba::nova(Alat::Linija);
    p.klik(A);
    p.klik_pa_desni(B);
    let (sloj, _) = p.karta.crtanje().odabran().expect("odabran");
    p.okvir(Vec::new());
    let razine = |p: &Proba| p.karta.pripremljeni[&sloj].razine.clone();
    let prva = razine(&p);
    p.okvir(Vec::new());
    assert!(
        Arc::ptr_eq(&prva, &razine(&p)),
        "isti sloj se ne priprema iznova"
    );
    assert_eq!(p.u.ponisti(), Ok(true));
    p.okvir(Vec::new());
    assert!(
        !p.karta.pripremljeni.contains_key(&sloj),
        "poništen je i sloj"
    );
}

#[test]
fn vuceni_objekt_nije_u_pripremi_dok_traje_vucenje() {
    let mut p = Proba::nova(Alat::Linija);
    p.klik(A);
    p.klik_pa_desni(B);
    let (sloj, id) = p.karta.crtanje().odabran().expect("odabran");
    p.okvir(vec![Event::PointerMoved(A)]);
    p.tipka(A, PointerButton::Primary, true);
    for i in 1..4 {
        p.okvir(vec![Event::PointerMoved(
            A + Vec2::new(10.0 * i as f32, 0.0),
        )]);
    }
    p.okvir(Vec::new());
    assert_eq!(p.karta.pripremljeni[&sloj].skriveni, Some(id));
    p.tipka(A, PointerButton::Primary, false);
    p.okvir(Vec::new());
    assert_eq!(p.karta.pripremljeni[&sloj].skriveni, None);
}

#[test]
fn skriveni_sloj_se_ne_priprema() {
    let mut p = Proba::nova(Alat::Linija);
    p.klik(A);
    p.klik_pa_desni(B);
    let nista = |_: IdSloja| false;
    let ok = Okolina {
        vidljiv: &nista,
        ..OK
    };
    assert!(p.karta.uskladi_slojeve(&p.u, &ok).is_empty());
    assert_eq!(p.karta.uskladi_slojeve(&p.u, &OK).len(), 1);
}

#[test]
fn karta_i_natpis_se_crtaju_i_bez_kartice() {
    let mut p = Proba::nova(Alat::Mjeri);
    p.okvir(Vec::new());
    let prazno = p.oblika;
    p.klik(A);
    p.okvir(vec![Event::PointerMoved(B)]);
    assert!(p.oblika > prazno, "crtež i natpis su nacrtani");
    p.karta.podloga = Some(Podloga::OpenFreeMap { tamna: false });
    let bez = p.oblika;
    p.okvir(Vec::new());
    assert!(p.oblika > bez, "navod izvora je nacrtan");
}

#[test]
fn kamera_ima_pravo_mjerilo_i_sredinu() {
    let mut p = Proba::nova(Alat::Odaberi);
    let ctx = p.ctx.clone();
    let mut izmjereno = None;
    let ulaz = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(800.0))),
        ..Default::default()
    };
    let memorija = p.karta.memorija();
    let mut izlaz = ctx.run_ui(ulaz, |ui| {
        Map::new(None, memorija, lon_lat(18.69, 45.55)).show(ui, |_, o, pr, _| {
            izmjereno = Some(kamera(o, pr));
        });
    });
    izlaz.textures_delta.clear();
    let k = izmjereno.expect("kamera");
    // Na zumu 14: opseg ekvatora kroz 256 * 2^14 točaka.
    let ocekivano = 2.0 * 20_037_508.342_789_244 / (256.0 * 16384.0);
    assert!(
        (k.metara_po_px - ocekivano).abs() / ocekivano < 1e-3,
        "{k:?}"
    );
    let s = u_mercator(18.69, 45.55);
    assert!((k.sredina[0] - s[0]).abs() < 10.0, "{k:?}");
    assert_eq!(k.pola, [400.0, 400.0]);
}

#[test]
fn promjena_podloge_mijenja_navod_i_izvor() {
    let mut p = Proba::nova(Alat::Odaberi);
    assert!(p.karta.izvor.is_none());
    let ctx = p.ctx.clone();
    p.karta
        .postavi_podlogu(Podloga::Dgu(crate::ListDgu::Tk25), ctx);
    assert_eq!(p.karta.podloga, Some(Podloga::Dgu(crate::ListDgu::Tk25)));
    assert!(p.karta.izvor.is_some());
}

#[test]
fn linija_se_nastavlja_pravim_klikovima_a_povlacenje_kraja_i_dalje_pomice_tocku() {
    let mut p = Proba::nova(Alat::Linija);
    p.klik(A);
    p.klik_pa_desni(B);
    assert_eq!(p.zadnji_vrhova(), 2);
    p.okvir(Vec::new());
    p.klik(B);
    assert_eq!(
        p.karta.crtanje().alat(),
        Alat::Linija,
        "klik na kraj nastavlja"
    );
    p.klik(C);
    p.klik_pa_desni(pos2(500.0, 300.0));
    assert_eq!(p.zadnji_vrhova(), 4);
    assert_eq!(p.objekata(), 1, "ista linija, a ne nova");
    assert_eq!(p.karta.crtanje().alat(), Alat::Odaberi);
    assert_eq!(p.u.ponisti(), Ok(true));
    assert_eq!(p.zadnji_vrhova(), 2, "jedan korak poništavanja");

    p.okvir(Vec::new());
    p.povuci(B, 5, Vec2::new(0.0, 10.0));
    assert_eq!(p.zadnji_vrhova(), 2, "povlačenje samo pomiče točku");
    assert_eq!(p.karta.crtanje().alat(), Alat::Odaberi);
}

#[test]
fn navod_podloge_karta_crta_samo_dok_ga_prozor_ne_preuzme() {
    let mut p = Proba::nova(Alat::Odaberi);
    let podloga = Podloga::OpenFreeMap { tamna: false };
    // Bez pločica s mreže: podloga je samo zapisana, a izvor ostaje prazan.
    p.karta.podloga = Some(podloga);
    p.okvir(Vec::new());
    assert!(
        p.tekstovi.iter().any(|t| t == podloga.navod()),
        "{:?}",
        p.tekstovi
    );
    p.karta.crtaj_navod(false);
    p.okvir(Vec::new());
    assert!(!p.tekstovi.iter().any(|t| t == podloga.navod()));
}

/// Crvene oznake presjeka u zadnjem okviru.
fn oznake_presjeka(p: &Proba) -> Vec<Pos2> {
    p.krugovi
        .iter()
        .filter(|(_, boja)| *boja == egui::Color32::from_rgb(0xD3, 0x2F, 0x2F))
        .map(|(m, _)| *m)
        .collect()
}

#[test]
fn osmica_dobiva_crvenu_oznaku_na_mjestu_presjeka() {
    let mut p = Proba::nova(Alat::Poligon);
    p.klik(A);
    p.klik(C);
    p.klik(B);
    assert!(oznake_presjeka(&p).is_empty(), "trokut ne siječe sam sebe");
    // Miš na četvrtom vrhu: lik koji se crta već se križa u sredini.
    let d = pos2(100.0, 300.0);
    p.okvir(vec![Event::PointerMoved(d)]);
    let oznake = oznake_presjeka(&p);
    assert_eq!(oznake.len(), 1, "{:?}", p.krugovi);
    assert!(oznake[0].distance(pos2(200.0, 200.0)) < 1.0, "{oznake:?}");
    p.klik_pa_desni(d);
    assert_eq!(p.objekata(), 1);
    // Gotov i odabran lik i dalje ima oznaku.
    p.okvir(Vec::new());
    let oznake = oznake_presjeka(&p);
    assert_eq!(oznake.len(), 1, "{:?}", p.krugovi);
    assert!(oznake[0].distance(pos2(200.0, 200.0)) < 1.0, "{oznake:?}");
}

#[test]
fn obican_lik_nema_oznake_presjeka() {
    let mut p = Proba::nova(Alat::Poligon);
    p.klik(A);
    p.klik(B);
    p.klik(C);
    p.klik_pa_desni(pos2(100.0, 300.0));
    p.okvir(Vec::new());
    assert_eq!(p.objekata(), 1);
    assert!(oznake_presjeka(&p).is_empty(), "{:?}", p.krugovi);
}
