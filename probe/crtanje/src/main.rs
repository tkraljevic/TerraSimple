//! Proba faze 0: radi li crtanje mišem na pravoj karti, po dogovorenim pravilima.
//!
//! Lijevi klik dodaje točku, povlačenje pomiče točku, povlačenje po praznoj karti pomiče
//! kartu, desni klik briše točku, dvoklik završava. Ovo se baca.
//!
//! proba_crtanje [--izvor ofm|put.pmtiles]            za ručno isprobavanje
//! proba_crtanje --samoprovjera [--snimka mapa]       program sam izvede radnje i provjeri ih
//! proba_crtanje --brzina N                           mjerenje s likom od N vrhova
#![forbid(unsafe_code)]

mod crtanje;
mod skripta;

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

use eframe::egui::{self, Color32, CornerRadius, DragPanButtons, Frame, Id, Margin, Pos2, Rect, Ui, pos2};
use walkers::sources::OpenFreeMap;
use walkers::{HttpOptions, HttpTiles, Map, MapMemory, PmTiles, Position, Style, Tiles, lon_lat};

use crtanje::{Alat, Crtanje, Objekt};
use skripta::{Korak, Mjerenje};

pub struct Proba {
    tiles: Box<dyn Tiles>,
    memorija: MapMemory,
    sredina: Position,
    pub c: Crtanje,
    pub gumbi: HashMap<Alat, Rect>,
    pub skripta: VecDeque<Korak>,
    pub cekaj_do: f64,
    pub zadnji_mis: Pos2,
    pub rezultati: Vec<(&'static str, bool)>,
    pub mapa_snimki: Option<String>,
    pub mjerenje: Mjerenje,
    zadnji_okvir: Instant,
    okvir: u32,
}

impl Proba {
    pub fn centar(&self) -> Position {
        self.memorija.detached().unwrap_or(self.sredina)
    }

    pub fn karta_klizi(&self) -> bool {
        self.memorija.animating()
    }

    pub fn zum(&self) -> f64 {
        self.memorija.zoom()
    }
}

impl eframe::App for Proba {
    fn raw_input_hook(&mut self, ctx: &egui::Context, ulaz: &mut egui::RawInput) {
        skripta::izvedi(self, ctx, ulaz);
    }

    fn ui(&mut self, ui: &mut Ui, _f: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let pocetak = Instant::now();
        let razmak = pocetak.duration_since(self.zadnji_okvir).as_secs_f32() * 1000.0;
        self.zadnji_okvir = pocetak;
        self.okvir += 1;
        if self.okvir == 1 {
            self.memorija.center_at(self.sredina);
            let _ = self.memorija.set_zoom(15.0); // 15 je valjan zum
        }

        self.c.tipkovnica(&ctx);
        let smije = self.c.smije_pomicati_kartu(&ctx);
        let sredina = self.sredina;
        let c = &mut self.c;
        Map::new(Some(self.tiles.as_mut()), &mut self.memorija, sredina)
            // `panning` u walkersu vrijedi samo za kotačić. Povlačenje odlučuje ovaj izbor tipki.
            .double_click_to_zoom(false)
            .drag_pan_buttons(if smije { DragPanButtons::PRIMARY } else { DragPanButtons::empty() })
            .show(ui, |ui, odziv, projektor, _| c.kadar(ui, odziv, projektor));

        self.traka_alata(&ctx);
        self.mjerenje.zabiljezi(razmak, pocetak.elapsed().as_secs_f32() * 1000.0);
        if !self.skripta.is_empty() {
            ctx.request_repaint();
        }
    }
}

impl Proba {
    fn traka_alata(&mut self, ctx: &egui::Context) {
        let c = &mut self.c;
        let gumbi = &mut self.gumbi;
        egui::Area::new(Id::new("alati")).fixed_pos(pos2(12.0, 12.0)).show(ctx, |ui| {
            Frame::new().fill(Color32::WHITE).corner_radius(CornerRadius::same(12)).inner_margin(Margin::same(8)).shadow(egui::Shadow { offset: [0, 2], blur: 8, spread: 0, color: Color32::from_black_alpha(38) }).show(ui, |ui| {
                ui.style_mut().visuals = egui::Visuals::light();
                ui.horizontal(|ui| {
                    for (a, t) in [(Alat::Odaberi, "Odaberi"), (Alat::Tocka, "Točka"), (Alat::Linija, "Linija"), (Alat::Poligon, "Poligon")] {
                        let o = ui.selectable_value(&mut c.alat, a, t);
                        gumbi.insert(a, o.rect);
                    }
                    ui.separator();
                    ui.label(egui::RichText::new(c.poruka()).color(Color32::from_rgb(0x44, 0x47, 0x4F)));
                });
            });
        });
    }
}

fn vrijednost(a: &[String], ime: &str) -> Option<String> {
    a.iter().position(|x| x == ime).and_then(|i| a.get(i + 1)).cloned()
}

fn izvor_karte(izvor: &str, ctx: egui::Context) -> Box<dyn Tiles> {
    if izvor == "ofm" {
        let opcije = HttpOptions { user_agent: walkers::HeaderValue::from_str("TerraSimple-proba/0.1").ok(), ..Default::default() };
        Box::new(HttpTiles::with_options_and_style(OpenFreeMap, opcije, Style::openmaptiles_basemap_light(), ctx))
    } else {
        Box::new(PmTiles::with_style(izvor, Style::protomaps_basemap_light(), ctx))
    }
}

fn main() -> eframe::Result {
    let a: Vec<String> = std::env::args().collect();
    let izvor = vrijednost(&a, "--izvor").unwrap_or_else(|| "ofm".to_string());
    let mapa_snimki = vrijednost(&a, "--snimka");
    let samoprovjera = a.iter().any(|x| x == "--samoprovjera");
    let brzina: Option<usize> = vrijednost(&a, "--brzina").and_then(|v| v.parse().ok());
    let ostra = a.iter().any(|x| x == "--ostra");
    let sredina = lon_lat(18.69, 45.55);

    let opcije = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]).with_title("TerraSimple, proba crtanja"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "TerraSimple proba crtanja",
        opcije,
        Box::new(move |cc| {
            walkers::install_renderer(cc.wgpu_render_state.as_ref());
            let mut c = Crtanje::default();
            let skripta = if let Some(n) = brzina {
                c.objekti.push(Objekt::zvijezda(sredina, n, ostra));
                c.odabran = Some(0);
                skripta::mjerenje_brzine()
            } else if samoprovjera {
                skripta::samoprovjera()
            } else {
                VecDeque::new()
            };
            Ok(Box::new(Proba {
                tiles: izvor_karte(&izvor, cc.egui_ctx.clone()),
                memorija: MapMemory::default(),
                sredina,
                c,
                gumbi: HashMap::new(),
                skripta,
                cekaj_do: 0.0,
                zadnji_mis: pos2(0.0, 0.0),
                rezultati: Vec::new(),
                mapa_snimki,
                mjerenje: Mjerenje::default(),
                zadnji_okvir: Instant::now(),
                okvir: 0,
            }))
        }),
    )
}
