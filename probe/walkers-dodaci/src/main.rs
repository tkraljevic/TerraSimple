// SPDX-License-Identifier: EUPL-1.2
#![forbid(unsafe_code)]

//! Glavni program za `probe/walkers-dodaci`.
//!
//! Načini rada:
//! - `cargo run --release -- --snimi-teme`
//!   Otvara stvarni `wgpu` prozor s OpenFreeMap podlogom, čeka u stvarnom vremenu da se
//!   vektorske pločice učitaju i iscrtaju, sprema `snimke/svijetla_tema.png` i
//!   `snimke/tamna_tema.png` te zatvara prozor.
//! - `cargo run --release -- --provjeri-sve`
//!   Izvršava sve automatske provjere za svih 5 pitanja i sve provjere namjernog kvara.

use std::path::Path;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::egui;
use proba_walkers_dodaci::dodaci::provjeri_walkers_extras;
use proba_walkers_dodaci::inercija::{provjeri_inerciju, zaustavi_inerciju_pri_crtanju};
use proba_walkers_dodaci::projekcija::provjeri_with_projection;
use proba_walkers_dodaci::teme::{
    PUTANJA_SNIMKE_SVIJETLA, PUTANJA_SNIMKE_TAMNA, analiziraj_sliku_karte, provjeri_teme,
    spremi_color_image_u_png,
};
use proba_walkers_dodaci::zum::provjeri_zum_izvan_raspona;
use walkers::sources::OpenFreeMap;
use walkers::{HttpOptions, HttpTiles, Map, MapMemory, Position, Style, Tiles, lon_lat};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FazaSnimanja {
    UcitavanjeSvijetle,
    CekanjeSnimkeSvijetle,
    UcitavanjeTamne,
    CekanjeSnimkeTamne,
    Gotovo,
}

struct Aplikacija {
    izvor_svijetli: HttpTiles,
    izvor_tamni: HttpTiles,
    memorija: MapMemory,
    tamna_tema: bool,
    crtanje_aktivno: bool,
    automatsko_snimanje: bool,
    faza: FazaSnimanja,
    pocetak_faze: Instant,
    zadnji_zahtjev_snimke: Instant,
    primljena_snimka: Arc<Mutex<Option<Arc<egui::ColorImage>>>>,
}

impl Aplikacija {
    fn nova(egui_ctx: egui::Context, automatsko_snimanje: bool) -> Self {
        egui_ctx.set_visuals(egui::Visuals::light());

        let izvor_svijetli = HttpTiles::with_options_and_style(
            OpenFreeMap,
            HttpOptions::default(),
            Style::openmaptiles_basemap_light(),
            egui_ctx.clone(),
        );
        let izvor_tamni = HttpTiles::with_options_and_style(
            OpenFreeMap,
            HttpOptions::default(),
            Style::openmaptiles_basemap_dark(),
            egui_ctx,
        );

        let mut memorija = MapMemory::default();
        let _ = memorija.set_zoom(13.0);

        let sada = Instant::now();
        Self {
            izvor_svijetli,
            izvor_tamni,
            memorija,
            tamna_tema: false,
            crtanje_aktivno: true,
            automatsko_snimanje,
            faza: FazaSnimanja::UcitavanjeSvijetle,
            pocetak_faze: sada,
            zadnji_zahtjev_snimke: sada,
            primljena_snimka: Arc::new(Mutex::new(None)),
        }
    }

    fn zatrazi_snimku(&mut self, ctx: &egui::Context) {
        let spremnik = Arc::clone(&self.primljena_snimka);
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(
            egui::ScreenshotCallback::new(move |slika| {
                if let Ok(mut guard) = spremnik.lock() {
                    *guard = Some(slika);
                }
            }),
        ));
        self.zadnji_zahtjev_snimke = Instant::now();
    }

    fn obradi_automatsko_snimanje(&mut self, ctx: &egui::Context, u_tijeku: usize) {
        if !self.automatsko_snimanje {
            return;
        }

        let primljena = self
            .primljena_snimka
            .lock()
            .ok()
            .and_then(|mut g| g.take());

        let proteklo = self.pocetak_faze.elapsed();
        if proteklo > Duration::from_secs(25) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        match self.faza {
            FazaSnimanja::UcitavanjeSvijetle => {
                self.tamna_tema = false;
                ctx.set_visuals(egui::Visuals::light());
                if proteklo >= Duration::from_millis(2200) && u_tijeku == 0 {
                    self.zatrazi_snimku(ctx);
                    self.faza = FazaSnimanja::CekanjeSnimkeSvijetle;
                }
            }
            FazaSnimanja::CekanjeSnimkeSvijetle => {
                if let Some(slika) = primljena {
                    let putanja = Path::new(PUTANJA_SNIMKE_SVIJETLA);
                    if spremi_color_image_u_png(&slika, putanja).is_err() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    }
                    if let Ok(ucitana) = image::open(putanja) {
                        let (svjetlina, boje) = analiziraj_sliku_karte(&ucitana.to_rgba8());
                        if boje >= 15 && svjetlina >= 150.0 {
                            self.tamna_tema = true;
                            ctx.set_visuals(egui::Visuals::dark());
                            self.pocetak_faze = Instant::now();
                            self.faza = FazaSnimanja::UcitavanjeTamne;
                            return;
                        }
                    }
                    self.faza = FazaSnimanja::UcitavanjeSvijetle;
                } else if self.zadnji_zahtjev_snimke.elapsed() > Duration::from_millis(600) {
                    self.faza = FazaSnimanja::UcitavanjeSvijetle;
                }
            }
            FazaSnimanja::UcitavanjeTamne => {
                self.tamna_tema = true;
                ctx.set_visuals(egui::Visuals::dark());
                if proteklo >= Duration::from_millis(2200) && u_tijeku == 0 {
                    self.zatrazi_snimku(ctx);
                    self.faza = FazaSnimanja::CekanjeSnimkeTamne;
                }
            }
            FazaSnimanja::CekanjeSnimkeTamne => {
                if let Some(slika) = primljena {
                    let putanja = Path::new(PUTANJA_SNIMKE_TAMNA);
                    if spremi_color_image_u_png(&slika, putanja).is_err() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        return;
                    }
                    if let Ok(ucitana) = image::open(putanja) {
                        let (svjetlina, boje) = analiziraj_sliku_karte(&ucitana.to_rgba8());
                        if boje >= 15 && svjetlina <= 85.0 {
                            self.faza = FazaSnimanja::Gotovo;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            return;
                        }
                    }
                    self.faza = FazaSnimanja::UcitavanjeTamne;
                } else if self.zadnji_zahtjev_snimke.elapsed() > Duration::from_millis(600) {
                    self.faza = FazaSnimanja::UcitavanjeTamne;
                }
            }
            FazaSnimanja::Gotovo => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }
}

impl eframe::App for Aplikacija {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let zagreb: Position = lon_lat(15.9819, 45.8150);

        egui::Panel::top("alatna_traka").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("OpenFreeMap (OpenMapTiles stil):");
                if ui.selectable_label(!self.tamna_tema, "Svijetla tema").clicked() {
                    self.tamna_tema = false;
                    ctx.set_visuals(egui::Visuals::light());
                }
                if ui.selectable_label(self.tamna_tema, "Tamna tema").clicked() {
                    self.tamna_tema = true;
                    ctx.set_visuals(egui::Visuals::dark());
                }
                ui.separator();
                ui.checkbox(&mut self.crtanje_aktivno, "Crtanje aktivno (bez inercije)");
                ui.separator();
                ui.label(format!("Zum: {:.1}", self.memorija.zoom()));
            });
        });

        // Mora biti prije `Map::show`, inače je karta u okviru puštanja tipke već skočila.
        zaustavi_inerciju_pri_crtanju(&mut self.memorija, self.crtanje_aktivno);

        let mut u_tijeku = 0usize;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let izvor: &mut dyn Tiles = if self.tamna_tema {
                    u_tijeku = self.izvor_tamni.stats().in_progress;
                    &mut self.izvor_tamni
                } else {
                    u_tijeku = self.izvor_svijetli.stats().in_progress;
                    &mut self.izvor_svijetli
                };
                Map::new(Some(izvor), &mut self.memorija, zagreb).show(ui, |_, _, _, _| {});
            });

        if self.automatsko_snimanje {
            self.obradi_automatsko_snimanje(&ctx, u_tijeku);
            ctx.request_repaint();
        }
    }
}

fn pokreni_sve_provjere() -> Result<(), String> {
    println!("=== 1. Provjera: Map::with_projection (HTRS96/TM EPSG:3765) ===");
    let izv_proj = provjeri_with_projection(0.0)?;
    println!("  Maks. povratna greška unproject(project(p)): {:.9}°", izv_proj.maks_povratna_greska_stupnjevi);
    println!("  Omjer istok/sjever za 1 km u HTRS96/TM:      {:.6}", izv_proj.omjer_istok_sjever_htrs96);
    println!("  Omjer istok/sjever za 1 km u Web Mercatoru:  {:.6}", izv_proj.omjer_istok_sjever_mercator);
    println!("  Kut konvergencije meridijana (Ilok 19.37°E): {:.4}°", izv_proj.kut_konvergencije_ilok_stupnjevi);
    println!("  Plugin koristi HTRS96 projektor:             {}", izv_proj.plugin_koristi_htrs96_projektor);
    println!("  Središnja pločica Mercator (Zagreb z14):     {:?}", izv_proj.plocica_mercator_zagreb_z14);
    println!("  Središnja pločica HTRS96   (Zagreb z14):     {:?}", izv_proj.plocica_htrs96_zagreb_z14);
    println!("  DGU WMS EPSG:3765 BBOX sadrži Zagreb:        {}", izv_proj.dgu_wms_htrs96_sadrzi_zagreb);
    if provjeri_with_projection(25.0).is_ok() {
        return Err("Namjerni kvar projekcije NIJE srušio provjeru!".to_string());
    }
    println!("  [OK] Namjerni kvar projekcije uredno pada.");

    println!("\n=== 2. Provjera: walkers_extras (KmlLayer, GeoJsonLayer, GroupedPlacesTree) ===");
    let izv_dod = provjeri_walkers_extras(false)?;
    println!("  Rupe u odvojenim <innerBoundaryIs>:          {} ({} trokuta)", izv_dod.rupe_odvojeni_inner_boundary, izv_dod.trokuti_odvojeni_inner_boundary);
    println!("  Rupe u istom <innerBoundaryIs>:              {} ({} trokuta)", izv_dod.rupe_isti_inner_boundary, izv_dod.trokuti_isti_inner_boundary);
    println!("  Rupa iste orijentacije prebojana (EvenOdd):  {}", izv_dod.rupa_iste_orijentacije_prebojana);
    println!("  Čuva KML stilove (<Style>, <PolyStyle>):     {}", izv_dod.cuva_kml_stilove);
    println!("  Čuva 3D Z visine:                            {}", izv_dod.cuva_z_visine);
    println!("  Pad na praznoj točki <Point>:                {}", izv_dod.panic_na_praznoj_tocki);
    println!("  Broj geometrija iz <gx:Track>:               {} (tiho izgubljeno)", izv_dod.broj_geometrija_iz_gx_track);
    println!("  GeoJsonLayer i GroupedPlacesTree rade:       {}", izv_dod.geojson_i_grouped_places_rade);
    if provjeri_walkers_extras(true).is_ok() {
        return Err("Namjerni kvar walkers_extras NIJE srušio provjeru!".to_string());
    }
    println!("  [OK] Namjerni kvar walkers_extras uredno pada.");

    println!("\n=== 3. Provjera: Inercija (klizanje karte nakon povlačenja u egui-ju) ===");
    let izv_in = provjeri_inerciju(false)?;
    println!("  Zbroj pomaka miša:                           {:.2} px", izv_in.bez_poziva.pomak_misa_px);
    for (ime, v) in [("Bez poziva", izv_in.bez_poziva), ("Poslije Map::show", izv_in.poziv_poslije_karte), ("Prije Map::show", izv_in.poziv_prije_karte)] {
        println!("  {ime:<18} skok pri puštanju {:.2} px, klizanje poslije {:.2} px, kraj od zbroja miša {:.2} px", v.skok_u_okviru_pustanja_px, v.klizanje_nakon_pustanja_px, v.kraj_od_zbroja_misa_px);
    }
    println!("  Sporo povlačenje, kašnjenje za mišem:        {:.2} px", izv_in.sporo_kasnjenje_px);
    println!("  Prekid već započetog klizanja u letu radi:   {}", izv_in.prekid_vec_zapocetog_klizanja_radi);
    if provjeri_inerciju(true).is_ok() {
        return Err("Namjerni kvar inercije NIJE srušio provjeru!".to_string());
    }
    println!("  [OK] Namjerni kvar inercije uredno pada.");

    println!("\n=== 4. Provjera: Zum izvan ispitanog raspona (< 11 i > 17) ===");
    let izv_zum = provjeri_zum_izvan_raspona(true, false)?;
    println!("  Dozvoljeni raspon u walkers::Zoom:           {:.1} ..= {:.1}", izv_zum.min_dozvoljeni_zum, izv_zum.maks_dozvoljeni_zum);
    println!("  Izvan [0, 26] vraća Err(InvalidZoom):        {}", izv_zum.izvan_0_26_vraca_gresku_bez_rusenja);
    println!("  OpenFreeMap deklarirani max_zoom:            {} (512 px)", izv_zum.openfreemap_izvor_max_zoom);
    println!("  PMTiles zaglavlje (min_zoom, max_zoom):      {:?}", izv_zum.pmtiles_zaglavlje_min_max_zoom);
    println!("  PMTiles okvir (min_lon..max_lon, lat):       {:?}", izv_zum.pmtiles_okvir);
    println!("  PMTiles središte:                            {:?}", izv_zum.pmtiles_srediste);
    println!("  PMTiles stvarno dostupne razine u središtu:  {:?}", izv_zum.pmtiles_dostupne_razine_srediste);
    println!("  PMTiles zum 2 (središte) prikazan:           {}", izv_zum.pmtiles_zum_2_srediste_prikazan);
    println!("  PMTiles izvan sektora (New York) -> prazno:  {}", izv_zum.pmtiles_izvan_sektora_prazno_bez_rusenja);
    println!("  PMTiles hladni skok izravno na zum 20:       {}", izv_zum.pmtiles_hladni_skok_zum_20_prikazan);
    println!("  PMTiles topli zum 20 (rastegnuta niža raz.): {} (uv širina = {:.5})", izv_zum.pmtiles_topli_zum_20_rastegnuta_niza_razina, izv_zum.pmtiles_topli_zum_20_uv_sirina);
    println!("  OpenFreeMap zum 5 i zum 20 (rastegnuto):     {} (uv širina = {:.5})", izv_zum.openfreemap_zum_5_i_20_rade_rastegnuto, izv_zum.openfreemap_zum_20_uv_sirina);
    println!("  Bez pada na zumu 0,2,5,8,10,18,20,22,26:     {}", izv_zum.bez_panike_na_svim_razinama);
    if provjeri_zum_izvan_raspona(false, true).is_ok() {
        return Err("Namjerni kvar zuma NIJE srušio provjeru!".to_string());
    }
    println!("  [OK] Namjerni kvar zuma uredno pada.");

    println!("\n=== 5. Provjera: Svijetla i tamna tema karte ===");
    let izv_teme = provjeri_teme(true, false)?;
    println!("  Broj slojeva OpenMapTiles (svijetla/tamna):  {} / {}", izv_teme.broj_slojeva_openmaptiles_svijetla, izv_teme.broj_slojeva_openmaptiles_tamna);
    println!("  Broj slojeva Protomaps    (svijetla/tamna):  {} / {}", izv_teme.broj_slojeva_protomaps_svijetla, izv_teme.broj_slojeva_protomaps_tamna);
    println!("  MVT svjetlina vrhova (svijetla / tamna):     {:.1} / {:.1}", izv_teme.mvt_svjetlina_svijetla, izv_teme.mvt_svjetlina_tamna);
    println!("  Snimka zaslona svijetla (svjetlina / boje):  {:.1} / {}", izv_teme.snimka_svjetlina_svijetla.unwrap_or(0.0), izv_teme.snimka_raznolikost_boja_svijetla);
    println!("  Snimka zaslona tamna    (svjetlina / boje):  {:.1} / {}", izv_teme.snimka_svjetlina_tamna.unwrap_or(0.0), izv_teme.snimka_raznolikost_boja_tamna);
    if provjeri_teme(false, true).is_ok() {
        return Err("Namjerni kvar tema NIJE srušio provjeru!".to_string());
    }
    println!("  [OK] Namjerni kvar tema uredno pada.");

    println!("\nSVE PROVJERE USPJEŠNO PROŠLE!");
    Ok(())
}

fn main() -> ExitCode {
    let argumenti: Vec<String> = std::env::args().collect();
    let samo_provjeri = argumenti.iter().any(|a| a == "--provjeri-sve");
    let snimi_teme = argumenti.iter().any(|a| a == "--snimi-teme");

    if samo_provjeri {
        match pokreni_sve_provjere() {
            Ok(()) => return ExitCode::SUCCESS,
            Err(greska) => {
                eprintln!("GREŠKA PRI PROVJERI: {greska}");
                return ExitCode::FAILURE;
            }
        }
    }

    let opcije = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_title("Proba: walkers dodaci i teme (OpenFreeMap)"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    let rezultat = eframe::run_native(
        "Proba: walkers dodaci",
        opcije,
        Box::new(move |cc| {
            walkers::install_renderer(cc.wgpu_render_state.as_ref());
            Ok(Box::new(Aplikacija::nova(cc.egui_ctx.clone(), snimi_teme)))
        }),
    );

    if let Err(e) = rezultat {
        eprintln!("Greška pri pokretanju prozora: {e}");
        return ExitCode::FAILURE;
    }

    if snimi_teme {
        match provjeri_teme(true, false) {
            Ok(izv) => {
                println!(
                    "Snimke uspješno spremljene i provjerene: svijetla={:.1} ({} boja), tamna={:.1} ({} boja)",
                    izv.snimka_svjetlina_svijetla.unwrap_or(0.0),
                    izv.snimka_raznolikost_boja_svijetla,
                    izv.snimka_svjetlina_tamna.unwrap_or(0.0),
                    izv.snimka_raznolikost_boja_tamna,
                );
            }
            Err(e) => {
                eprintln!("Greška pri provjeri snimljenih slika: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
