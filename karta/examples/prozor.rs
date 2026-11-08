//! Karta u pravom prozoru, s izmišljenim objektima, da se vidi pravi izgled i isproba mišem.
//!
//! `cargo run -p karta --example prozor` otvara prozor. Uz put do PNG-a program sam snimi sliku i izađe.

use eframe::egui;
use karta::{Alat, Karta, Okolina, Podloga};
use slojevi::{
    Geometrija, Mjere, NaredbaObjekta, NaredbaStabla, Novo, Uredjivac, Vrh, VrstaGeometrije,
};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SREDINA: [f64; 2] = [18.69, 45.55];

struct Prozor {
    karta: Karta,
    u: Uredjivac,
    snimka: Option<String>,
    pocetak: Instant,
    slika: Arc<Mutex<Option<Arc<egui::ColorImage>>>>,
    trazena: bool,
    /// Od zadnjeg prema prvom.
    dogadaji: Vec<egui::Event>,
}

fn sloj(u: &mut Uredjivac, naziv: &str, vrsta: VrstaGeometrije) -> slojevi::IdSloja {
    let n = NaredbaStabla::DodajSloj {
        u: None,
        naziv: naziv.into(),
        vrsta,
    };
    match u.izvrsi(n) {
        Ok(Some(Novo::Sloj(s))) => s,
        r => panic!("{r:?}"),
    }
}

fn primjer() -> Uredjivac {
    let mut u = Uredjivac::novi();
    let d = |x: f64, y: f64| Vrh::novi(SREDINA[0] + x, SREDINA[1] + y);
    let lik = sloj(&mut u, "Parcele", VrstaGeometrije::Poligon);
    let g = Geometrija::Poligon(vec![
        vec![
            d(-0.012, -0.004),
            d(0.0, -0.007),
            d(0.009, -0.001),
            d(0.004, 0.006),
            d(-0.009, 0.005),
        ],
        vec![
            d(-0.003, -0.002),
            d(0.001, -0.002),
            d(0.001, 0.001),
            d(-0.003, 0.001),
        ],
    ]);
    let n = NaredbaObjekta::Dodaj {
        sloj: lik,
        naziv: "Lik".into(),
        geometrija: g,
    };
    u.izvrsi(n).expect("lik");
    let crta = sloj(&mut u, "Trase", VrstaGeometrije::Linija);
    let g = Geometrija::Linija(vec![d(-0.02, 0.009), d(-0.004, 0.012), d(0.018, 0.004)]);
    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: crta,
        naziv: "Crta".into(),
        geometrija: g,
    })
    .expect("crta");
    let tocka = sloj(&mut u, "Točke", VrstaGeometrije::Tocka);
    let g = Geometrija::Tocka(d(0.014, -0.006));
    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: tocka,
        naziv: "Točka".into(),
        geometrija: g,
    })
    .expect("točka");
    u
}

impl eframe::App for Prozor {
    /// Pri snimanju program sam radi zadane klikove, da se vidi rezultat na slici.
    fn raw_input_hook(&mut self, _ctx: &egui::Context, ulaz: &mut egui::RawInput) {
        if self.pocetak.elapsed() > Duration::from_secs(4)
            && let Some(dogadaj) = self.dogadaji.pop()
        {
            ulaz.events.push(dogadaj);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _okvir: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        egui::Panel::top("alati").show(ui, |ui| {
            ui.horizontal(|ui| {
                for (ime, alat) in [
                    ("Odaberi", Alat::Odaberi),
                    ("Točka", Alat::Tocka),
                    ("Linija", Alat::Linija),
                    ("Poligon", Alat::Poligon),
                    ("Mjeri", Alat::Mjeri),
                ] {
                    if ui
                        .selectable_label(self.karta.crtanje().alat() == alat, ime)
                        .clicked()
                    {
                        self.karta.crtanje().postavi_alat(alat);
                    }
                }
                if ui.button("Poništi").clicked() {
                    let _nista_za_ponistiti_nije_greska = self.u.ponisti();
                }
                if ui.button("Ponovi").clicked() {
                    let _nista_za_ponoviti_nije_greska = self.u.ponovi();
                }
            });
        });
        let vidljiv = |_: slojevi::IdSloja| true;
        let naziv = |v: VrstaGeometrije| format!("{v:?}");
        let opis = |m: &Mjere| opis_mjere(m);
        let duljina = |d: f64| metri(d);
        let naziv_objekta = |v: VrstaGeometrije, redni: usize| format!("{v:?} {redni}");
        let ok = Okolina {
            vidljiv: &vidljiv,
            naziv_sloja: &naziv,
            opis_mjere: &opis,
            opis_duljine: &duljina,
            naziv_objekta: &naziv_objekta,
        };
        if self.karta.crtanje().alat() == Alat::Mjeri {
            egui::Panel::top("mjerenje").show(ui, |ui| self.ploca_mjerenja(ui, &ok));
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.karta.prikazi(ui, &mut self.u, &ok);
            });
        self.snimi(&ctx);
    }
}

fn klik(pos: egui::Pos2) -> [egui::Event; 3] {
    let tipka = |pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    [egui::Event::PointerMoved(pos), tipka(true), tipka(false)]
}

fn dogadaji(nacin: &str) -> Vec<egui::Event> {
    let mjesta = match nacin {
        "mjeri" => vec![(200.0, 500.0), (480.0, 420.0), (700.0, 560.0)],
        "snimka" => vec![(495.0, 357.0)],
        _ => Vec::new(),
    };
    let mut d: Vec<egui::Event> = mjesta
        .into_iter()
        .flat_map(|(x, y)| klik(egui::pos2(x, y)))
        .collect();
    if nacin == "mjeri" {
        d.push(egui::Event::PointerMoved(egui::pos2(900.0, 300.0)));
    }
    d.reverse();
    d
}

fn metri(d: f64) -> String {
    let tekst = if d >= 1000.0 {
        format!("{:.2} km", d / 1000.0)
    } else {
        format!("{d:.0} m")
    };
    tekst.replace('.', ",")
}

fn opis_mjere(m: &Mjere) -> String {
    match m {
        Mjere::Tocka => String::new(),
        Mjere::Linija(c) => metri(c.duljina_m),
        Mjere::Poligon(l) => format!(
            "{:.2} ha, opseg {}",
            l.povrsina_m2 / 10_000.0,
            metri(l.opseg_m)
        )
        .replace('.', ","),
    }
}

impl Prozor {
    /// Samo za probu. Pravu ploču crta `prozor`.
    fn ploca_mjerenja(&mut self, ui: &mut egui::Ui, ok: &Okolina<'_>) {
        ui.horizontal(|ui| {
            let povrsina = self.karta.crtanje().mjerenje_povrsine();
            if ui.selectable_label(!povrsina, "Udaljenost").clicked() {
                self.karta.crtanje().mjeri_povrsinu(false);
            }
            if ui.selectable_label(povrsina, "Površina").clicked() {
                self.karta.crtanje().mjeri_povrsinu(true);
            }
            let Some(m) = self.karta.crtanje().mjerenje() else {
                ui.label("Klikni na kartu za prvu točku.");
                return;
            };
            ui.strong(opis_mjere(&m.mjere));
            ui.label(format!(
                "Ukupno, {} dijela. Računato na elipsoidu WGS84.",
                m.dijelova
            ));
            if ui.button("Spremi").clicked() {
                self.karta.crtanje().spremi_mjerenje(&mut self.u, ok);
            }
            if ui.button("Očisti").clicked() {
                self.karta.crtanje().ocisti_mjerenje();
            }
        });
    }

    fn snimi(&mut self, ctx: &egui::Context) {
        let Some(put) = self.snimka.clone() else {
            return;
        };
        ctx.request_repaint();
        if !self.trazena && self.pocetak.elapsed() > Duration::from_secs(8) {
            self.trazena = true;
            let s = self.slika.clone();
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
                egui::ScreenshotCallback::new(move |i| {
                    *s.lock().expect("slika") = Some(i);
                }),
            ));
        }
        let gotova = self.slika.lock().expect("slika").take();
        if let Some(s) = gotova {
            let [w, h] = s.size;
            let b: Vec<u8> = s.pixels.iter().flat_map(|p| p.to_array()).collect();
            image::save_buffer(&put, &b, w as u32, h as u32, image::ColorType::Rgba8)
                .expect("spremanje");
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

fn main() -> eframe::Result {
    let snimka = std::env::args().nth(1);
    let nacin = match (&snimka, std::env::args().nth(2)) {
        (Some(_), Some(n)) => n,
        (Some(_), None) => "snimka".to_string(),
        _ => String::new(),
    };
    let opcije = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_title("TerraSimple, proba karte"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "TerraSimple proba karte",
        opcije,
        Box::new(move |cc| {
            let stanje = cc.wgpu_render_state.as_ref().ok_or("nema wgpu")?;
            karta::ugradi(stanje, 1);
            let mut slova = egui::FontDefinitions::empty();
            let inter = egui::FontData::from_static(include_bytes!(
                "../../probe/izgled/fonti/Inter-Regular.otf"
            ));
            slova.font_data.insert("inter".into(), inter.into());
            for o in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
                slova.families.insert(o, vec!["inter".into()]);
            }
            cc.egui_ctx.set_fonts(slova);
            let mut karta = Karta::nova(
                Podloga::OpenFreeMap { tamna: false },
                cc.egui_ctx.clone(),
                SREDINA[0],
                SREDINA[1],
            );
            karta.memorija().set_zoom(14.0)?;
            if nacin == "mjeri" {
                karta.crtanje().postavi_alat(Alat::Mjeri);
            }
            Ok(Box::new(Prozor {
                karta,
                u: primjer(),
                snimka: snimka.clone(),
                pocetak: Instant::now(),
                slika: Arc::new(Mutex::new(None)),
                trazena: false,
                dogadaji: dogadaji(&nacin),
            }))
        }),
    )
}
