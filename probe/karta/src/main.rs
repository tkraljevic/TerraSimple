//! Proba faze 0: vuče li `walkers` kartu, i radi li DGU WMS kao podloga.
//!
//! Ovo se baca. Program sam pomiče i približava kartu i mjeri trajanje okvira.
//! proba_karta --izvor ofm|dgu|<put.pmtiles> [--tamna] [--snimka mapa] [--vrhova N]
#![forbid(unsafe_code)]

use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::egui::{self, Color32, ColorImage, Response, ScreenshotCallback, Stroke, Ui, ViewportCommand, pos2};
use walkers::sources::{Attribution, OpenFreeMap, TileSource};
use walkers::{HttpOptions, HttpTiles, Map, MapMemory, PmTiles, Plugin, Position, Projector, Style, TileId, Tiles, lon_lat};

const UA: &str = "TerraSimple-proba/0.1";
const POLA_OPSEGA: f64 = 20037508.342789244;

/// DGU WMS kao pločice. EPSG:3857 radi iako ga DGU ne navodi. EPSG:3785, koji navodi,
/// vraća krivo mjesto (provjereno 6. 11. 2026. usporedbom slika Osijeka).
struct DguWms {
    sloj: &'static str,
}

impl TileSource for DguWms {
    fn tile_url(&self, t: TileId) -> String {
        let n = f64::from(1u32 << t.zoom);
        let d = 2.0 * POLA_OPSEGA / n;
        let (x1, y2) = (-POLA_OPSEGA + f64::from(t.x) * d, POLA_OPSEGA - f64::from(t.y) * d);
        let (x2, y1) = (x1 + d, y2 - d);
        format!(
            "https://geoportal.dgu.hr/wms?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS={}&STYLES=&CRS=EPSG:3857&BBOX={x1},{y1},{x2},{y2}&WIDTH=256&HEIGHT=256&FORMAT=image/jpeg",
            self.sloj
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution { text: "© Državna geodetska uprava", url: "https://geoportal.dgu.hr", logo_light: None, logo_dark: None }
    }

    fn max_zoom(&self) -> u8 {
        20
    }
}

/// Korisnikova crta s puno vrhova, da se vidi koliko crtanje usporava kartu.
struct Crta {
    tocke: Arc<Vec<Position>>,
}

impl Plugin for Crta {
    fn run(self: Box<Self>, ui: &mut Ui, _o: &Response, projektor: &Projector, _m: &MapMemory) {
        let v: Vec<_> = self.tocke.iter().map(|p| projektor.project(*p).to_pos2()).collect();
        ui.painter().add(egui::Shape::line(v.clone(), Stroke::new(2.5, Color32::from_rgb(0xE8, 0x59, 0x0C))));
        for t in &v {
            ui.painter().circle_filled(pos2(t.x, t.y), 3.0, Color32::WHITE);
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Korak {
    Ucitavanje,
    Pomicanje,
    Priblizavanje,
    Kraj,
}

struct Proba {
    tiles: Box<dyn Tiles>,
    memorija: MapMemory,
    sredina: Position,
    korak: Korak,
    pocetak: Instant,
    pocetak_koraka: Instant,
    zadnji: Instant,
    okviri: Vec<(Korak, f32)>,
    crtanje: Vec<(Korak, f32)>,
    prva_slika: Option<f32>,
    izvor: String,
    mapa_snimki: Option<String>,
    slika: Arc<Mutex<Option<Arc<ColorImage>>>>,
    snimka_trazena: bool,
    crta: Arc<Vec<Position>>,
}

fn opis(mut t: Vec<f32>) -> String {
    if t.is_empty() {
        return "nema okvira".to_string();
    }
    t.sort_by(f32::total_cmp);
    let n = t.len();
    let udio = |u: f64| t[((n - 1) as f64 * u) as usize];
    let preko = t.iter().filter(|m| **m > 16.7).count();
    format!("{n} okvira, medijan {:.1} ms, 95 % {:.1} ms, 99 % {:.1} ms, najviše {:.1} ms, sporijih od 60 fps {preko}", udio(0.5), udio(0.95), udio(0.99), t[n - 1])
}

impl eframe::App for Proba {
    fn ui(&mut self, ui: &mut Ui, _f: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let sad = Instant::now();
        let ms = sad.duration_since(self.zadnji).as_secs_f32() * 1000.0;
        self.zadnji = sad;
        self.okviri.push((self.korak, ms));
        let u_koraku = sad.duration_since(self.pocetak_koraka).as_secs_f64();

        match self.korak {
            Korak::Ucitavanje => {
                self.memorija.center_at(self.sredina);
                let _ = self.memorija.set_zoom(14.0); // zoom 14 je valjan, greška bi bila vidljiva na snimci
                let u_tijeku = self.tiles.stats().in_progress;
                if u_tijeku == 0 && self.prva_slika.is_none() && u_koraku > 0.5 {
                    self.prva_slika = Some(sad.duration_since(self.pocetak).as_secs_f32());
                }
                let gotovo = (self.prva_slika.is_some() && u_koraku > 3.0) || u_koraku > 25.0;
                if gotovo && !self.snimka_trazena {
                    self.snimka_trazena = true;
                    if self.mapa_snimki.is_some() {
                        let s = self.slika.clone();
                        ctx.send_viewport_cmd(ViewportCommand::Screenshot(ScreenshotCallback::new(move |i| {
                            if let Ok(mut m) = s.lock() {
                                *m = Some(i);
                            }
                        })));
                    }
                    self.korak = Korak::Pomicanje;
                    self.pocetak_koraka = sad;
                }
            }
            Korak::Pomicanje => {
                // Oko 2 km u sekundi, tako da stalno ulaze nove pločice.
                let d = u_koraku * 0.03;
                self.memorija.center_at(lon_lat(self.sredina.x() + d, self.sredina.y() + d * 0.4));
                if u_koraku > 8.0 {
                    self.korak = Korak::Priblizavanje;
                    self.pocetak_koraka = sad;
                }
            }
            Korak::Priblizavanje => {
                let z = 14.0 + 3.0 * (u_koraku * 0.8).sin();
                let _ = self.memorija.set_zoom(z); // z je uvijek između 11 i 17
                if u_koraku > 8.0 {
                    self.korak = Korak::Kraj;
                }
            }
            Korak::Kraj => {
                let po = |k| self.okviri.iter().filter(|(a, _)| *a == k).skip(2).map(|(_, m)| *m).collect::<Vec<f32>>();
                println!("izvor: {}", self.izvor);
                println!("prva slika gotova nakon: {:?} s", self.prva_slika);
                println!("pomicanje: {}", opis(po(Korak::Pomicanje)));
                println!("približavanje: {}", opis(po(Korak::Priblizavanje)));
                let cr = |k| self.crtanje.iter().filter(|(a, _)| *a == k).skip(2).map(|(_, m)| *m).collect::<Vec<f32>>();
                println!("vrijeme crtanja, pomicanje: {}", opis(cr(Korak::Pomicanje)));
                println!("vrijeme crtanja, približavanje: {}", opis(cr(Korak::Priblizavanje)));
                ctx.send_viewport_cmd(ViewportCommand::Close);
                return;
            }
        }

        if let (Some(mapa), Some(s)) = (&self.mapa_snimki, self.slika.lock().ok().and_then(|mut m| m.take())) {
            let [w, h] = s.size;
            let bajtovi: Vec<u8> = s.pixels.iter().flat_map(|p| p.to_array()).collect();
            let ime = std::path::Path::new(&self.izvor).file_stem().and_then(|i| i.to_str()).unwrap_or("karta");
            if let Err(g) = image::save_buffer(format!("{mapa}/{ime}.png"), &bajtovi, w as u32, h as u32, image::ColorType::Rgba8) {
                eprintln!("snimka nije spremljena: {g}");
            }
        }

        let sredina = self.sredina;
        let mut karta = Map::new(Some(self.tiles.as_mut()), &mut self.memorija, sredina);
        if !self.crta.is_empty() {
            karta = karta.with_plugin(Crta { tocke: self.crta.clone() });
        }
        karta.show(ui, |_, _, _, _| {});
        self.crtanje.push((self.korak, sad.elapsed().as_secs_f32() * 1000.0));
        ctx.request_repaint();
    }
}

fn izvor_karte(izvor: &str, tamna: bool, ctx: egui::Context) -> Box<dyn Tiles> {
    let opcije = || HttpOptions {
        user_agent: walkers::HeaderValue::from_str(UA).ok(),
        ..Default::default()
    };
    match izvor {
        "ofm" => {
            let stil = if tamna { Style::openmaptiles_basemap_dark() } else { Style::openmaptiles_basemap_light() };
            Box::new(HttpTiles::with_options_and_style(OpenFreeMap, opcije(), stil, ctx))
        }
        "dgu" => Box::new(HttpTiles::with_options(DguWms { sloj: "DOF" }, opcije(), ctx)),
        "dgutk" => Box::new(HttpTiles::with_options(DguWms { sloj: "TK25" }, opcije(), ctx)),
        put => {
            let stil = if tamna { Style::protomaps_basemap_dark() } else { Style::protomaps_basemap_light() };
            Box::new(PmTiles::with_style(put, stil, ctx))
        }
    }
}

fn vrijednost(a: &[String], ime: &str) -> Option<String> {
    a.iter().position(|x| x == ime).and_then(|i| a.get(i + 1)).cloned()
}

fn main() -> eframe::Result {
    let a: Vec<String> = std::env::args().collect();
    let izvor = vrijednost(&a, "--izvor").unwrap_or_else(|| "ofm".to_string());
    let mapa_snimki = vrijednost(&a, "--snimka");
    let tamna = a.iter().any(|x| x == "--tamna");
    let vrhova: usize = vrijednost(&a, "--vrhova").and_then(|v| v.parse().ok()).unwrap_or(0);
    let sredina = lon_lat(18.69, 45.55); // Osijek
    let crta: Vec<Position> = (0..vrhova)
        .map(|i| {
            let u = i as f64 / vrhova.max(1) as f64;
            lon_lat(18.60 + u * 0.3, 45.55 + (u * 200.0).sin() * 0.01)
        })
        .collect();
    let opcije = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]).with_title("TerraSimple, proba karte"),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration { surface: eframe::egui_wgpu::SurfaceConfig { present_mode: eframe::wgpu::PresentMode::AutoNoVsync, desired_maximum_frame_latency: Some(2) }, ..Default::default() },
        ..Default::default()
    };
    eframe::run_native(
        "TerraSimple proba karte",
        opcije,
        Box::new(move |cc| {
            walkers::install_renderer(cc.wgpu_render_state.as_ref());
            let sad = Instant::now();
            Ok(Box::new(Proba {
                tiles: izvor_karte(&izvor, tamna, cc.egui_ctx.clone()),
                memorija: MapMemory::default(),
                sredina,
                korak: Korak::Ucitavanje,
                pocetak: sad,
                pocetak_koraka: sad,
                zadnji: sad,
                okviri: Vec::new(),
                crtanje: Vec::new(),
                prva_slika: None,
                izvor,
                mapa_snimki,
                slika: Arc::new(Mutex::new(None)),
                snimka_trazena: false,
                crta: Arc::new(crta),
            }))
        }),
    )
}
