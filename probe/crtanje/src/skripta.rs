//! Program sam izvodi radnje mišem i provjerava ishod.
//!
//! Događaji ulaze ondje gdje ih inače predaje operacijski sustav, pa prolaze isti put kroz
//! egui i walkers kao pravi miš. Sam operacijski sustav pri tome ne sudjeluje.

use std::collections::VecDeque;
use std::sync::Arc;

use eframe::egui::{self, Event, Key, Modifiers, PointerButton, Pos2, ScreenshotCallback, ViewportCommand, pos2, vec2};

use crate::Proba;
use crate::crtanje::{Alat, Cilj, opseg_i_povrsina, povrsina_u_ravnini_ha};

type Polozaj = Arc<dyn Fn(&Proba) -> Pos2 + Send + Sync>;

pub enum Korak {
    Cekaj(u32),
    /// Čeka da karta nakon povlačenja prestane kliziti.
    KartaMirna,
    Mis(Polozaj),
    Tipka(PointerButton, bool),
    Taster(Key, Modifiers),
    Provjeri(&'static str, Box<dyn Fn(&Proba) -> bool>),
    Pamti(fn(&mut Proba)),
    Snimi(&'static str),
    Mjeri(&'static str),
    Kraj,
}
use Korak::{Cekaj, KartaMirna, Kraj, Mis, Mjeri, Pamti, Provjeri, Snimi, Taster, Tipka};

#[derive(Default)]
pub struct Mjerenje {
    faza: Option<&'static str>,
    pub zapisi: Vec<(&'static str, f32, f32)>,
}

impl Mjerenje {
    pub fn zabiljezi(&mut self, razmak: f32, crtanje: f32) {
        if let Some(f) = self.faza {
            self.zapisi.push((f, razmak, crtanje));
        }
    }
}

fn opis(mut t: Vec<f32>) -> String {
    if t.is_empty() {
        return "nema okvira".to_string();
    }
    t.sort_by(f32::total_cmp);
    let n = t.len();
    let u = |d: f64| t[((n - 1) as f64 * d) as usize];
    format!("{n} okvira, medijan {:.1} ms, 95 % {:.1} ms, 99 % {:.1} ms, najviše {:.1} ms", u(0.5), u(0.95), u(0.99), t[n - 1])
}

pub fn izvedi(s: &mut Proba, ctx: &egui::Context, ulaz: &mut egui::RawInput) {
    if s.skripta.is_empty() {
        return;
    }
    ulaz.focused = true;
    // Pravi miš na stolu ne smije upasti među glumljene događaje.
    ulaz.events.retain(|e| !matches!(e, Event::PointerMoved(_) | Event::PointerButton { .. } | Event::PointerGone | Event::MouseMoved(_) | Event::MouseWheel { .. }));
    let sad = ulaz.time.unwrap_or(0.0);
    loop {
        if sad < s.cekaj_do {
            return;
        }
        let Some(k) = s.skripta.pop_front() else { return };
        match k {
            // U sekundama, ne u okvirima: na 120 Hz bi 30 okvira bilo kraće od vremena za dvoklik.
            Cekaj(n) => s.cekaj_do = sad + f64::from(n) * 0.02,
            KartaMirna => {
                if s.karta_klizi() {
                    s.skripta.push_front(KartaMirna);
                    return;
                }
                s.cekaj_do = sad + 0.1;
            }
            Mis(p) => {
                s.zadnji_mis = p(s);
                ulaz.events.push(Event::PointerMoved(s.zadnji_mis));
                return;
            }
            Tipka(button, pressed) => {
                ulaz.events.push(Event::PointerButton { pos: s.zadnji_mis, button, pressed, modifiers: Modifiers::NONE });
                return;
            }
            Taster(key, modifiers) => {
                ulaz.events.push(Event::Key { key, physical_key: Some(key), pressed: true, repeat: false, modifiers });
                return;
            }
            Provjeri(ime, f) => {
                let u_redu = f(s);
                println!("{} {ime}", if u_redu { "PROŠLO" } else { "PALO  " });
                s.rezultati.push((ime, u_redu));
            }
            Pamti(f) => f(s),
            Snimi(ime) => {
                if let Some(m) = &s.mapa_snimki {
                    let put = format!("{m}/{ime}.png");
                    ctx.send_viewport_cmd(ViewportCommand::Screenshot(ScreenshotCallback::new(move |i: Arc<egui::ColorImage>| {
                        let [w, h] = i.size;
                        let b: Vec<u8> = i.pixels.iter().flat_map(|p| p.to_array()).collect();
                        if let Err(g) = image::save_buffer(&put, &b, w as u32, h as u32, image::ColorType::Rgba8) {
                            eprintln!("snimka nije spremljena: {g}");
                        }
                    })));
                    s.cekaj_do = sad + 0.2;
                    return;
                }
            }
            Mjeri(f) => s.mjerenje.faza = if f.is_empty() { None } else { Some(f) },
            Kraj => {
                let pala = s.rezultati.iter().filter(|(_, u)| !u).count();
                if !s.rezultati.is_empty() {
                    println!("UKUPNO: {} provjera, palo {pala}", s.rezultati.len());
                }
                let mut faze: Vec<&'static str> = s.mjerenje.zapisi.iter().map(|z| z.0).collect();
                faze.dedup();
                for f in faze {
                    let okviri = s.mjerenje.zapisi.iter().filter(|z| z.0 == f).map(|z| z.1).collect();
                    let crtanje = s.mjerenje.zapisi.iter().filter(|z| z.0 == f).map(|z| z.2).collect();
                    println!("{f}: okvir: {}", opis(okviri));
                    println!("{f}: naš kod i egui: {}", opis(crtanje));
                }
                ctx.send_viewport_cmd(ViewportCommand::Close);
                return;
            }
        }
    }
}

fn na(x: f32, y: f32) -> Polozaj {
    Arc::new(move |_| pos2(x, y))
}

fn klik(v: &mut VecDeque<Korak>, p: Polozaj, gumb: PointerButton) {
    v.extend([Mis(p), Cekaj(2), Tipka(gumb, true), Tipka(gumb, false), Cekaj(30)]);
}

fn dvoklik(v: &mut VecDeque<Korak>, p: Polozaj) {
    let l = PointerButton::Primary;
    v.extend([Mis(p), Cekaj(2), Tipka(l, true), Tipka(l, false), Cekaj(2), Tipka(l, true), Tipka(l, false), Cekaj(30)]);
}

fn povuci(v: &mut VecDeque<Korak>, od: Polozaj, pomak: (f32, f32), koraka: u32) {
    let l = PointerButton::Primary;
    let od2 = od.clone();
    v.extend([Mis(od), Cekaj(2), Tipka(l, true)]);
    for i in 1..=koraka {
        let u = i as f32 / koraka as f32;
        let o = od2.clone();
        // Polazište se pamti pri prvom koraku, jer se hvatište miče zajedno s mišem.
        v.push_back(Mis(Arc::new(move |s| {
            let _ = &o;
            if i == 1 {
                s.zadnji_mis + vec2(pomak.0 * u, pomak.1 * u)
            } else {
                s.zadnji_mis + vec2(pomak.0 / koraka as f32, pomak.1 / koraka as f32)
            }
        })));
    }
    // Miš prvo stane, pa onda pušta tipku. Inače karta klizi koliko brzo se povlačilo, a to
    // ovisi o osvježavanju monitora, pa bi se kasnije točke našle izvan prozora.
    v.extend([Cekaj(20), Tipka(l, false), Cekaj(30)]);
}

fn gumb_alata(a: Alat) -> Polozaj {
    Arc::new(move |s| s.gumbi.get(&a).map_or(pos2(0.0, 0.0), |r| r.center()))
}

fn hvatiste(i: usize, srednje: bool) -> Polozaj {
    Arc::new(move |s| s.c.hvatista.iter().filter(|h| h.srednje == srednje).nth(i).map_or(pos2(0.0, 0.0), |h| h.mjesto))
}

fn provjeri(v: &mut VecDeque<Korak>, ime: &'static str, f: impl Fn(&Proba) -> bool + 'static) {
    v.push_back(Provjeri(ime, Box::new(f)));
}

fn broj_tocaka_crteza(s: &Proba) -> usize {
    s.c.crtez.as_ref().map_or(0, |c| c.tocke.len())
}

fn tocke_objekta(s: &Proba, i: usize) -> usize {
    s.c.objekti.get(i).map_or(0, |o| o.tocke.len())
}

fn u_stablu(p: walkers::Position, q: walkers::Position) -> bool {
    (p.x() - q.x()).abs() < 1e-9 && (p.y() - q.y()).abs() < 1e-9
}

/// Pamćenje između koraka: gdje je bila karta i gdje je bila prva točka crteža.
static ZAPAMCENO: std::sync::Mutex<Option<(walkers::Position, Option<walkers::Position>, f64)>> = std::sync::Mutex::new(None);

fn zapamti_stanje(s: &mut Proba) {
    let prva = s.c.crtez.as_ref().and_then(|c| c.tocke.first().copied());
    if let Ok(mut z) = ZAPAMCENO.lock() {
        *z = Some((s.centar(), prva, s.zum()));
    }
}

fn zapamceno() -> Option<(walkers::Position, Option<walkers::Position>, f64)> {
    ZAPAMCENO.lock().ok().and_then(|z| *z)
}

pub fn samoprovjera() -> VecDeque<Korak> {
    let l = PointerButton::Primary;
    let d = PointerButton::Secondary;
    let mut v = VecDeque::new();
    v.push_back(Cekaj(150)); // da karta stigne

    klik(&mut v, gumb_alata(Alat::Poligon), l);
    provjeri(&mut v, "klik na alat Poligon bira alat i ne dira kartu", |s| s.c.alat == Alat::Poligon && s.c.crtez.is_none());

    for (x, y) in [(400.0, 300.0), (800.0, 280.0), (780.0, 620.0), (590.0, 420.0), (400.0, 620.0)] {
        klik(&mut v, na(x, y), l);
    }
    provjeri(&mut v, "pet lijevih klikova daje pet točaka udubljenog lika", |s| broj_tocaka_crteza(&s) == 5 && s.c.objekti.is_empty());
    v.push_back(Mis(na(520.0, 500.0)));
    v.push_back(Cekaj(5));
    v.push_back(Snimi("1-crtanje"));

    klik(&mut v, na(1000.0, 400.0), l);
    provjeri(&mut v, "šesti klik dodaje točku", |s| broj_tocaka_crteza(&s) == 6);
    klik(&mut v, na(1000.0, 400.0), d);
    provjeri(&mut v, "desni klik na točku je briše", |s| broj_tocaka_crteza(&s) == 5);
    klik(&mut v, na(1100.0, 750.0), d);
    provjeri(&mut v, "desni klik pokraj točke ništa ne briše", |s| broj_tocaka_crteza(&s) == 5);

    v.push_back(Pamti(zapamti_stanje));
    povuci(&mut v, na(800.0, 280.0), (60.0, -40.0), 10);
    provjeri(&mut v, "povlačenje točke pomiče točku", |s| s.c.hvatista.iter().any(|h| h.mjesto.distance(pos2(860.0, 240.0)) < 2.0));
    provjeri(&mut v, "povlačenje točke ne pomiče kartu", |s| zapamceno().is_some_and(|z| u_stablu(z.0, s.centar())));
    provjeri(&mut v, "povlačenje točke ne dodaje novu točku", |s| broj_tocaka_crteza(&s) == 5);

    v.push_back(Pamti(zapamti_stanje));
    povuci(&mut v, na(1100.0, 750.0), (-100.0, -60.0), 10);
    v.push_back(KartaMirna);
    provjeri(&mut v, "povlačenje po praznoj karti pomiče kartu", |s| zapamceno().is_some_and(|z| !u_stablu(z.0, s.centar())));
    provjeri(&mut v, "pomak karte ne dodaje točku", |s| broj_tocaka_crteza(&s) == 5);
    provjeri(&mut v, "pomak karte ne mijenja zemljopisni položaj točaka", |s| {
        zapamceno().is_some_and(|z| z.1.zip(s.c.crtez.as_ref().and_then(|c| c.tocke.first().copied())).is_some_and(|(a, b)| u_stablu(a, b)))
    });

    v.push_back(Pamti(zapamti_stanje));
    dvoklik(&mut v, na(640.0, 700.0));
    provjeri(&mut v, "dvoklik završava poligon, s točkom prvog klika", |s| s.c.crtez.is_none() && tocke_objekta(s, 0) == 6);
    provjeri(&mut v, "dvoklik ne približava kartu", |s| zapamceno().is_some_and(|z| (z.2 - s.zum()).abs() < 1e-9));
    provjeri(&mut v, "nakon završetka lik je odabran, a alat je Odaberi", |s| s.c.odabran == Some(0) && s.c.alat == Alat::Odaberi);
    provjeri(&mut v, "površina se slaže s neovisnim računom unutar 2 %", |s| s.c.objekti.first().is_some_and(|o| {
        let (_, p) = opseg_i_povrsina(&o.tocke);
        let r = povrsina_u_ravnini_ha(&o.tocke);
        println!("       površina {p:.3} ha, u ravnini {r:.3} ha");
        r > 0.0 && (p - r).abs() / r < 0.02
    }));
    v.push_back(Cekaj(5));
    v.push_back(Snimi("2-gotov-lik"));

    klik(&mut v, hvatiste(1, false), d);
    provjeri(&mut v, "desni klik briše točku gotovog lika", |s| tocke_objekta(s, 0) == 5);
    povuci(&mut v, hvatiste(0, true), (20.0, 20.0), 8);
    provjeri(&mut v, "povlačenje srednje točke dodaje vrh", |s| tocke_objekta(s, 0) == 6);
    v.push_back(Taster(Key::Z, Modifiers::COMMAND));
    v.push_back(Cekaj(5));
    provjeri(&mut v, "poništavanje vraća lik", |s| tocke_objekta(s, 0) == 5);

    klik(&mut v, gumb_alata(Alat::Linija), l);
    klik(&mut v, na(200.0, 300.0), l);
    klik(&mut v, na(300.0, 350.0), l);
    v.push_back(Taster(Key::Backspace, Modifiers::NONE));
    v.push_back(Cekaj(5));
    provjeri(&mut v, "Backspace miče zadnju točku", |s| broj_tocaka_crteza(&s) == 1);
    v.push_back(Taster(Key::Escape, Modifiers::NONE));
    v.push_back(Cekaj(5));
    provjeri(&mut v, "Esc odustaje, a gotovi lik ostaje", |s| s.c.crtez.is_none() && s.c.objekti.len() == 1);

    klik(&mut v, na(200.0, 200.0), l);
    dvoklik(&mut v, na(300.0, 260.0));
    provjeri(&mut v, "dvoklik završava liniju", |s| s.c.objekti.get(1).is_some_and(|o| o.vrsta == Alat::Linija && o.tocke.len() == 2));
    klik(&mut v, hvatiste(0, false), d);
    provjeri(&mut v, "linija ne može ostati bez dvije točke", |s| tocke_objekta(s, 1) == 2 && s.c.poruka().contains("barem"));

    klik(&mut v, gumb_alata(Alat::Tocka), l);
    klik(&mut v, na(1200.0, 250.0), l);
    provjeri(&mut v, "alat Točka dodaje točku jednim klikom", |s| s.c.objekti.get(2).is_some_and(|o| o.vrsta == Alat::Tocka));

    klik(&mut v, gumb_alata(Alat::Odaberi), l);
    klik(&mut v, na(1300.0, 800.0), l);
    provjeri(&mut v, "klik po praznom miče odabir", |s| s.c.odabran.is_none());
    klik(&mut v, na(1200.0, 250.0), l);
    provjeri(&mut v, "klik na objekt ga odabire", |s| s.c.odabran == Some(2));

    let _ = Cilj::Crtez;
    v.push_back(Snimi("3-kraj"));
    v.push_back(Kraj);
    v
}

pub fn mjerenje_brzine() -> VecDeque<Korak> {
    let mut v = VecDeque::new();
    v.push_back(Cekaj(60));
    v.push_back(Mjeri("mirno"));
    v.push_back(Cekaj(60));
    v.push_back(Mjeri("povlačenje točke"));
    povuci(&mut v, hvatiste(0, false), (-200.0, 80.0), 120);
    v.push_back(Mjeri("pomicanje karte"));
    povuci(&mut v, na(40.0, 860.0), (300.0, -150.0), 120);
    v.push_back(Mjeri(""));
    v.push_back(Kraj);
    v
}
