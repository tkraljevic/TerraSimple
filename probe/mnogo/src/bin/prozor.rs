//! Isto crtanje u pravom prozoru, preko `walkersa`, s podlogom ispod.
//!
//! `prozor izmisljeno|<kml> sve|vidljivo <podloga.pmtiles> [mapa za sliku]`
//!
//! Program sam pomiče i približava kartu i mjeri trajanje okvira. Stvarni podaci nikad se ne
//! snimaju, nego se ispisuju samo brojevi.

use eframe::egui::{self, ColorImage, Response, ScreenshotCallback, Ui, ViewportCommand};
use eframe::egui_wgpu::{self, Callback, CallbackResources, CallbackTrait, ScreenDescriptor};
use proba_mnogo::crtac::{Crtac, Kamera, Plan};
use proba_mnogo::{podaci, priprema};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use walkers::{lon_lat, Map, MapMemory, PmTiles, Plugin, Position, Projector, Style, Tiles};

const R: f64 = 6_378_137.0;
const OSIJEK: [f64; 2] = [18.69, 45.55];

struct Stanje {
    crtac: Crtac,
    plan: Plan,
}

struct Poziv {
    kamera: Kamera,
}

impl CallbackTrait for Poziv {
    fn prepare(
        &self,
        _uredaj: &wgpu::Device,
        red: &wgpu::Queue,
        _ekran: &ScreenDescriptor,
        _enkoder: &mut wgpu::CommandEncoder,
        resursi: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(s) = resursi.get_mut::<Stanje>() {
            let razina = s.crtac.razina_za(self.kamera.metara_po_px);
            s.plan = s.crtac.pripremi(red, self.kamera, razina);
        }
        Vec::new()
    }

    fn paint(&self, _info: egui::PaintCallbackInfo, prolaz: &mut wgpu::RenderPass<'static>, resursi: &CallbackResources) {
        if let Some(s) = resursi.get::<Stanje>() {
            s.crtac.crtaj(prolaz, &s.plan);
        }
    }
}

struct Sloj {
    oznaka: bool,
}

impl Plugin for Sloj {
    fn run(self: Box<Self>, ui: &mut Ui, odgovor: &Response, projektor: &Projector, _m: &MapMemory) {
        let okvir = odgovor.rect;
        let sredina = projektor.unproject(okvir.center().to_vec2());
        let (a, b) = (projektor.project(sredina), projektor.project(lon_lat(sredina.x() + 0.01, sredina.y())));
        let metara_po_tocki = R * 0.01_f64.to_radians() / f64::from(b.x - a.x);
        let kamera = Kamera {
            sredina: podaci::mercator(sredina.x(), sredina.y()),
            metara_po_px: metara_po_tocki,
            pola: [okvir.width() / 2.0, okvir.height() / 2.0],
        };
        ui.painter().add(Callback::new_paint_callback(okvir, Poziv { kamera }));
        // Ista točka preko `walkersa`, da se na slici izmjeri poklapaju li se.
        if self.oznaka {
            let t = projektor.project(lon_lat(OSIJEK[0], OSIJEK[1]));
            ui.painter().circle_stroke(egui::pos2(t.x, t.y), 14.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)));
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
    uredaj: wgpu::Device,
    oznaka: bool,
    tiles: Box<dyn Tiles>,
    memorija: MapMemory,
    sredina: Position,
    korak: Korak,
    pocetak_koraka: Instant,
    zadnji: Instant,
    okviri: Vec<(Korak, f32)>,
    mapa_slike: Option<String>,
    slika: Arc<Mutex<Option<Arc<ColorImage>>>>,
    snimka_trazena: bool,
}

fn opis(mut t: Vec<f32>) -> String {
    if t.is_empty() {
        return "nema okvira".to_string();
    }
    t.sort_by(f32::total_cmp);
    let n = t.len();
    let udio = |u: f64| t[((n - 1) as f64 * u) as usize];
    let preko = t.iter().filter(|m| **m > 33.4).count();
    format!("{n} okvira, medijan {:.1} ms, 95 % {:.1} ms, 99 % {:.1} ms, najviše {:.1} ms, sporijih od 30 fps {preko}", udio(0.5), udio(0.95), udio(0.99), t[n - 1])
}

impl eframe::App for Proba {
    fn ui(&mut self, ui: &mut Ui, _f: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Čeka da kartica završi prošli okvir. Bez toga, kad je ekran ugašen, broj okvira ne
        // znači ništa: izmjereno je 2 500 okvira u sekundi bez ikakvog stvarnog čekanja.
        let _ = self.uredaj.poll(wgpu::PollType::wait_indefinitely()); // greška znači samo da nema čekanja
        let sad = Instant::now();
        let ms = sad.duration_since(self.zadnji).as_secs_f32() * 1000.0;
        self.zadnji = sad;
        self.okviri.push((self.korak, ms));
        let u_koraku = sad.duration_since(self.pocetak_koraka).as_secs_f64();
        match self.korak {
            Korak::Ucitavanje => {
                self.memorija.center_at(self.sredina);
                let _ = self.memorija.set_zoom(9.0); // zoom 9 je valjan, greška bi se vidjela na slici
                if u_koraku > 6.0 {
                    if !self.snimka_trazena && self.mapa_slike.is_some() {
                        self.snimka_trazena = true;
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
                let d = u_koraku * 0.05;
                self.memorija.center_at(lon_lat(self.sredina.x() + d, self.sredina.y() + d * 0.3));
                if u_koraku > 10.0 {
                    self.korak = Korak::Priblizavanje;
                    self.pocetak_koraka = sad;
                }
            }
            Korak::Priblizavanje => {
                // Od cijele zemlje do pojedinih parcela i natrag.
                let z = 11.0 + 5.0 * (u_koraku * 0.6).sin();
                let _ = self.memorija.set_zoom(z); // z je uvijek između 6 i 16
                if u_koraku > 15.0 {
                    self.korak = Korak::Kraj;
                }
            }
            Korak::Kraj => {
                let po = |k| self.okviri.iter().filter(|(a, _)| *a == k).skip(2).map(|(_, m)| *m).collect::<Vec<f32>>();
                println!("pomicanje: {}", opis(po(Korak::Pomicanje)));
                println!("približavanje: {}", opis(po(Korak::Priblizavanje)));
                ctx.send_viewport_cmd(ViewportCommand::Close);
                return;
            }
        }
        if let (Some(mapa), Some(s)) = (&self.mapa_slike, self.slika.lock().ok().and_then(|mut m| m.take())) {
            let [w, h] = s.size;
            let bajtovi: Vec<u8> = s.pixels.iter().flat_map(|p| p.to_array()).collect();
            if let Err(g) = image::save_buffer(format!("{mapa}/prozor.png"), &bajtovi, w as u32, h as u32, image::ColorType::Rgba8) {
                eprintln!("slika nije spremljena: {g}");
            }
        }
        let sredina = self.sredina;
        Map::new(Some(self.tiles.as_mut()), &mut self.memorija, sredina).with_plugin(Sloj { oznaka: self.oznaka }).show(ui, |_, _, _, _| {});
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 4 {
        eprintln!("upotreba: prozor izmisljeno|<kml> sve|vidljivo <podloga.pmtiles> [mapa za sliku]");
        std::process::exit(2);
    }
    let stvarno = a[1] != "izmisljeno";
    let mut sve = if stvarno {
        match proba_kml::citac::procitaj_datoteku(Path::new(&a[1])) {
            Ok(st) => podaci::iz_kml(&st.korijen),
            Err(_) => {
                eprintln!("datoteka se ne može pročitati");
                std::process::exit(3);
            }
        }
    } else {
        podaci::izmisljeno(85_683, 1_482, 1_738, 34, 80)
    };
    if !stvarno {
        sve.push(podaci::Znacajka {
            ispuna: None,
            crta: [255, 0, 255, 255],
            sirina: 8.0,
            dijelovi: vec![vec![podaci::mercator(OSIJEK[0], OSIJEK[1])]],
            lik: false,
            tocka: true,
            vidljivo: true,
        });
    }
    let pr = priprema::pripremi(&sve, a[2] == "vidljivo");
    println!("izvor: {}, način {}, značajki {}", if stvarno { "stvarna datoteka" } else { "izmišljeno" }, a[2], sve.len());
    drop(sve);
    let razine = Mutex::new(Some(pr.razine));
    let podloga = a[3].clone();
    let mapa_slike = if stvarno { None } else { a.get(4).cloned() };
    let opcije = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]).with_title("TerraSimple, proba karte s mnogo vrhova"),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: egui_wgpu::WgpuConfiguration {
            surface: egui_wgpu::SurfaceConfig { present_mode: wgpu::PresentMode::AutoNoVsync, desired_maximum_frame_latency: Some(2) },
            ..Default::default()
        },
        ..Default::default()
    };
    eframe::run_native(
        "TerraSimple proba mnogo",
        opcije,
        Box::new(move |cc| {
            walkers::install_renderer(cc.wgpu_render_state.as_ref());
            let rs = cc.wgpu_render_state.as_ref().ok_or("nema wgpu")?;
            let granice = rs.adapter.limits();
            let r = razine.lock().map_err(|_| "brava")?.take().ok_or("razine")?;
            let crtac = Crtac::novi(&rs.device, &granice, r, rs.target_format, 1);
            println!("na kartici {:.0} MB, format {:?}", crtac.bajtova as f64 / 1e6, rs.target_format);
            rs.renderer.write().callback_resources.insert(Stanje { crtac, plan: Plan::default() });
            let sad = Instant::now();
            Ok(Box::new(Proba {
                uredaj: rs.device.clone(),
                oznaka: !stvarno,
                tiles: Box::new(PmTiles::with_style(podloga.clone(), Style::protomaps_basemap_light(), cc.egui_ctx.clone())),
                memorija: MapMemory::default(),
                sredina: lon_lat(16.8, 45.4),
                korak: Korak::Ucitavanje,
                pocetak_koraka: sad,
                zadnji: sad,
                okviri: Vec::new(),
                mapa_slike: mapa_slike.clone(),
                slika: Arc::new(Mutex::new(None)),
                snimka_trazena: false,
            }))
        }),
    )
}
