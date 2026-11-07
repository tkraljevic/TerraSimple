// SPDX-License-Identifier: EUPL-1.2

//! Ispitivanje 1. pitanja: Što točno radi `Map::with_projection` u `walkers` 0.65?
//!
//! Provjerava:
//! - Može li ekran biti u HTRS96/TM (EPSG:3765) umjesto u Web Mercatoru (EPSG:3857)?
//! - Što se tada događa s pretvaranjem točaka u piksele (`Projector::project` i `unproject`)?
//! - Što se događa s crtanjem u `Plugin`-u?
//! - Što se događa s rasterskim pločicama, vektorskim pločicama (`OpenFreeMap`, `PmTiles`)
//!   i DGU-ovim WMS-om (`CRS=EPSG:3765` naspram `CRS=EPSG:3857`)?

use std::cell::Cell;
use std::rc::Rc;

use eframe::egui::{self, Context, Pos2, Rect, Response, Ui, Vec2};
use proj4rs::Proj;
use walkers::sources::{Attribution, TileSource};
use walkers::{
    Map, MapMemory, MercatorProjection, Pixels, Plugin, Position, Projection, Projector, TileId,
    TilePiece, Tiles, lon_lat,
};

/// Veličina osnovne pločice na razini zuma 0 u `walkers`-u (256 × 256 piksela).
const OSNOVNA_PLOCICA_PX: f64 = 256.0;

/// PROJ opis za WGS84 geodetske koordinate (lon/lat u radijanima za `proj4rs`).
const PROJ_WGS84: &str = "+proj=longlat +datum=WGS84 +no_defs";

/// PROJ opis za HTRS96 / Croatia TM (EPSG:3765).
const PROJ_HTRS96: &str = "+proj=tmerc +lat_0=0 +lon_0=16.5 +k=0.9999 +x_0=500000 +y_0=0 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs";

/// Kvadratni isječak u metrima HTRS96/TM-a koji na razini zuma 0 stane u jednu pločicu 256×256.
/// Odabran tako da pokriva cijelu Hrvatsku i okolicu (duljina stranice 40 075 016,686 m
/// jednaka opsegu Zemlje, sa središtem u E = 500 000 m, N = 5 000 000 m) kako bi mjerilo zuma
/// bilo izravno usporedivo sa standardnim razinama zuma 0..26.
const HTRS96_OPSEG_M: f64 = 40_075_016.685_578_49;
const HTRS96_MIN_E: f64 = 500_000.0 - HTRS96_OPSEG_M / 2.0;
const HTRS96_MAX_N: f64 = 5_000_000.0 + HTRS96_OPSEG_M / 2.0;

/// Vlastita implementacija `walkers::Projection` za HTRS96/TM (EPSG:3765).
pub struct Htrs96Projekcija {
    wgs84: Proj,
    htrs96: Proj,
    /// Ako je uključeno, namjerno kvari projekciju radi provjere pada testa.
     kvar_pomaka_px: f64,
}

impl Htrs96Projekcija {
    /// Stvara ispravnu HTRS96/TM projekciju.
    pub fn nova() -> Result<Self, String> {
        let wgs84 = Proj::from_proj_string(PROJ_WGS84).map_err(|e| e.to_string())?;
        let htrs96 = Proj::from_proj_string(PROJ_HTRS96).map_err(|e| e.to_string())?;
        Ok(Self {
            wgs84,
            htrs96,
            kvar_pomaka_px: 0.0,
        })
    }

    /// Stvara namjerno pokvarenu projekciju (dodaje pomak pri `project`) za dokaz pada provjere.
    pub fn nova_pokvarena(kvar_pomaka_px: f64) -> Result<Self, String> {
        let mut p = Self::nova()?;
        p.kvar_pomaka_px = kvar_pomaka_px;
        Ok(p)
    }

    /// Pretvara `(lon, lat)` u stupnjevima u HTRS96/TM `(istočno_m, sjeverno_m)`.
    pub fn u_htrs96_metre(&self, polozaj: Position) -> Result<(f64, f64), String> {
        let mut tocka = (
            polozaj.x().to_radians(),
            polozaj.y().to_radians(),
            0.0_f64,
        );
        proj4rs::transform::transform(&self.wgs84, &self.htrs96, &mut tocka)
            .map_err(|e| e.to_string())?;
        Ok((tocka.0, tocka.1))
    }

    /// Pretvara HTRS96/TM `(istočno_m, sjeverno_m)` natrag u WGS84 `(lon, lat)` u stupnjevima.
    pub fn iz_htrs96_metara(&self, istocno_m: f64, sjeverno_m: f64) -> Result<Position, String> {
        let mut tocka = (istocno_m, sjeverno_m, 0.0_f64);
        proj4rs::transform::transform(&self.htrs96, &self.wgs84, &mut tocka)
            .map_err(|e| e.to_string())?;
        Ok(lon_lat(tocka.0.to_degrees(), tocka.1.to_degrees()))
    }

    /// Izračunava HTRS96/TM pravokutnik (`min_e, min_n, max_e, max_n`) za dani `TileId`.
    pub fn htrs96_okvir_plocice(&self, tile_id: TileId) -> (f64, f64, f64, f64) {
        let broj_plocica = 2.0_f64.powi(i32::from(tile_id.zoom));
        let korak_m = HTRS96_OPSEG_M / broj_plocica;
        let min_e = HTRS96_MIN_E + f64::from(tile_id.x) * korak_m;
        let max_e = min_e + korak_m;
        let max_n = HTRS96_MAX_N - f64::from(tile_id.y) * korak_m;
        let min_n = max_n - korak_m;
        (min_e, min_n, max_e, max_n)
    }
}

impl Projection for Htrs96Projekcija {
    fn project(&self, position: Position, zoom: f64) -> Pixels {
        let ukupno_piksela = OSNOVNA_PLOCICA_PX * 2.0_f64.powf(zoom);
        let Ok((istocno_m, sjeverno_m)) = self.u_htrs96_metre(position) else {
            return Pixels::new(0.0, 0.0);
        };
        let udio_x = (istocno_m - HTRS96_MIN_E) / HTRS96_OPSEG_M;
        let udio_y = (HTRS96_MAX_N - sjeverno_m) / HTRS96_OPSEG_M;
        Pixels::new(
            udio_x * ukupno_piksela + self.kvar_pomaka_px,
            udio_y * ukupno_piksela,
        )
    }

    fn unproject(&self, pixels: Pixels, zoom: f64) -> Position {
        let ukupno_piksela = OSNOVNA_PLOCICA_PX * 2.0_f64.powf(zoom);
        let udio_x = pixels.x() / ukupno_piksela;
        let udio_y = pixels.y() / ukupno_piksela;
        let istocno_m = HTRS96_MIN_E + udio_x * HTRS96_OPSEG_M;
        let sjeverno_m = HTRS96_MAX_N - udio_y * HTRS96_OPSEG_M;
        self.iz_htrs96_metara(istocno_m, sjeverno_m)
            .unwrap_or_else(|_| lon_lat(16.5, 45.0))
    }

    fn scale_pixel_per_meter(&self, _position: Position, zoom: f64) -> f64 {
        // U poprečnoj Mercatorovoj projekciji (HTRS96/TM) mjerilo je u metrima ravnine
        // jednoliko po X i Y osi (~0.9999 uz središnji meridijan 16.5° E).
        let ukupno_piksela = OSNOVNA_PLOCICA_PX * 2.0_f64.powf(zoom);
        ukupno_piksela / HTRS96_OPSEG_M
    }
}

/// Lažni izvor pločica koji bilježi koje je točno `TileId` pločice `walkers::Map` zatražio.
pub struct BiljeznikPlocica {
    velicina_plocice: u32,
    pub zatrazeni: Vec<TileId>,
}

impl BiljeznikPlocica {
    pub fn novi(velicina_plocice: u32) -> Self {
        Self {
            velicina_plocice,
            zatrazeni: Vec::new(),
        }
    }
}

impl Tiles for BiljeznikPlocica {
    fn at(&mut self, tile_id: TileId) -> Option<TilePiece> {
        self.zatrazeni.push(tile_id);
        None
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "Bilježnik pločica",
            url: "",
            logo_light: None,
            logo_dark: None,
        }
    }

    fn tile_size(&self) -> u32 {
        self.velicina_plocice
    }
}

/// Primjer WMS izvora za DGU u `CRS=EPSG:3765` koji računa `BBOX` iz `Htrs96Projekcija`.
pub struct DguWmsHtrs96Izvor {
    projekcija: Htrs96Projekcija,
}

impl DguWmsHtrs96Izvor {
    pub fn novi() -> Result<Self, String> {
        Ok(Self {
            projekcija: Htrs96Projekcija::nova()?,
        })
    }
}

impl TileSource for DguWmsHtrs96Izvor {
    fn tile_url(&self, tile_id: TileId) -> String {
        let (min_e, min_n, max_e, max_n) = self.projekcija.htrs96_okvir_plocice(tile_id);
        format!(
            "https://geoportal.dgu.hr/services/inspire/orthophoto_2021_2022/wms?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS=OI.OrthoimageCoverage&STYLES=&CRS=EPSG:3765&BBOX={min_e:.2},{min_n:.2},{max_e:.2},{max_n:.2}&WIDTH=256&HEIGHT=256&FORMAT=image/jpeg"
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "DGU WMS EPSG:3765",
            url: "https://geoportal.dgu.hr",
            logo_light: None,
            logo_dark: None,
        }
    }
}

/// Ispitni `Plugin` koji crta točku i bilježi njezin zaslonski položaj preko `Projector`-a.
struct IspitniPlugin {
    polozaj: Position,
    zabiljezeni_ekran: Rc<Cell<Option<Pos2>>>,
}

impl Plugin for IspitniPlugin {
    fn run(
        self: Box<Self>,
        ui: &mut Ui,
        _response: &Response,
        projector: &Projector,
        _map_memory: &MapMemory,
    ) {
        let zaslonski = projector.project(self.polozaj).to_pos2();
        ui.painter()
            .circle_filled(zaslonski, 4.0, egui::Color32::RED);
        self.zabiljezeni_ekran.set(Some(zaslonski));
    }
}

/// Sažetak mjerenja ponašanja `Map::with_projection`.
#[derive(Debug, Clone)]
pub struct IzvjestajProjekcije {
    /// Najveća povratna pogreška `unproject(project(p))` u stupnjevima.
    pub maks_povratna_greska_stupnjevi: f64,
    /// Omjer zaslonskih piksela za 1 km istočno naspram 1 km sjeverno u HTRS96Projekciji.
    pub omjer_istok_sjever_htrs96: f64,
    /// Omjer zaslonskih piksela za 1 km istočno naspram 1 km sjeverno u MercatorProjection (na 45.8° N).
    pub omjer_istok_sjever_mercator: f64,
    /// Kut konvergencije meridijana u stupnjevima na istoku Hrvatske (Ilok, 19.37° E) u HTRS96/TM-u
    /// naspram Web Mercatora (u kojem je meridijan uvijek okomit, 0°).
    pub kut_konvergencije_ilok_stupnjevi: f64,
    /// Je li `Plugin` dobio zaslonsku točku točno jednaku `Projector::project` u HTRS96/TM-u.
    pub plugin_koristi_htrs96_projektor: bool,
    /// Središnja pločica zatražena u `MercatorProjection` za Zagreb na zumu 14.
    pub plocica_mercator_zagreb_z14: TileId,
    /// Središnja pločica zatražena u `Htrs96Projekcija` za Zagreb na zumu 14.
    pub plocica_htrs96_zagreb_z14: TileId,
    /// Primjer izračunatog WMS BBOX-a u EPSG:3765 koji pokriva Zagreb za `plocica_htrs96_zagreb_z14`.
    pub dgu_wms_htrs96_sadrzi_zagreb: bool,
}

/// Izvršava cjelovitu provjeru `with_projection` s `Htrs96Projekcija` i `MercatorProjection`.
pub fn provjeri_with_projection(kvar_pomaka_px: f64) -> Result<IzvjestajProjekcije, String> {
    let htrs96 = Htrs96Projekcija::nova_pokvarena(kvar_pomaka_px)?;
    let mercator = MercatorProjection;

    let ispitne_tocke = [
        lon_lat(15.9819, 45.8150), // Zagreb
        lon_lat(16.4402, 43.5081), // Split
        lon_lat(18.6955, 45.5550), // Osijek
        lon_lat(14.4422, 45.3271), // Rijeka
    ];

    let pravokutnik = Rect::from_min_size(Pos2::ZERO, Vec2::splat(800.0));
    let mut memorija = MapMemory::default();
    memorija.set_zoom(14.0).map_err(|e| e.to_string())?;

    let projektor_htrs96 = Projector::new(pravokutnik, &htrs96, &memorija, ispitne_tocke[0]);
    let projektor_mercator = Projector::new(pravokutnik, &mercator, &memorija, ispitne_tocke[0]);

    // 1. Povratna točnost Projector::project -> Projector::unproject
    let mut maks_greska = 0.0_f64;
    for &tocka in &ispitne_tocke {
        let na_ekranu = projektor_htrs96.project(tocka);
        let natrag = projektor_htrs96.unproject(na_ekranu);
        let dx = (natrag.x() - tocka.x()).abs();
        let dy = (natrag.y() - tocka.y()).abs();
        maks_greska = maks_greska.max(dx).max(dy);
    }

    // 2. Izotropnost mjerila u HTRS96/TM-u: pomak od 1000 m istočno i 1000 m sjeverno u HTRS96/TM
    let (zg_e, zg_n) = htrs96.u_htrs96_metre(ispitne_tocke[0])?;
    let tocka_istok_1km = htrs96.iz_htrs96_metara(zg_e + 1000.0, zg_n)?;
    let tocka_sjever_1km = htrs96.iz_htrs96_metara(zg_e, zg_n + 1000.0)?;

    let p0_h = projektor_htrs96.project(ispitne_tocke[0]);
    let px_istok_h = (projektor_htrs96.project(tocka_istok_1km) - p0_h).length() as f64;
    let px_sjever_h = (projektor_htrs96.project(tocka_sjever_1km) - p0_h).length() as f64;
    let omjer_istok_sjever_htrs96 = px_istok_h / px_sjever_h;

    let p0_m = projektor_mercator.project(ispitne_tocke[0]);
    let px_istok_m = (projektor_mercator.project(tocka_istok_1km) - p0_m).length() as f64;
    let px_sjever_m = (projektor_mercator.project(tocka_sjever_1km) - p0_m).length() as f64;
    let omjer_istok_sjever_mercator = px_istok_m / px_sjever_m;

    // 3. Konvergencija meridijana u Iloku (19.37° E, daleko od 16.5° E):
    // U Web Mercatoru točka 0.05° sjevernije ima isti X piksel (dx = 0).
    // U HTRS96/TM-u meridijan 19.37° E nagnut je prema središnjem meridijanu 16.5° E za ~2.04°!
    let ilok_jug = lon_lat(19.37, 45.20);
    let ilok_sjever = lon_lat(19.37, 45.25);
    let v_htrs = projektor_htrs96.project(ilok_sjever) - projektor_htrs96.project(ilok_jug);
    let kut_konvergencije_ilok_stupnjevi = (v_htrs.x as f64).atan2(-f64::from(v_htrs.y)).abs().to_degrees();

    // 4. Stvarno izvođenje `Map::with_projection` u `egui::Context::run_ui`:
    // Provjeravamo koje `TileId` pločice `walkers` traži i što dobiva `Plugin`.
    let ctx = Context::default();
    let mut biljeznik_mercator = BiljeznikPlocica::novi(256);
    let mut biljeznik_htrs96 = BiljeznikPlocica::novi(256);
    let zabiljezeni_plugin_ekran = Rc::new(Cell::new(None));
    let tocka_za_plugin = ispitne_tocke[1]; // Split

    let mut ulaz = egui::RawInput {
        screen_rect: Some(pravokutnik),
        ..Default::default()
    };
    ulaz.time = Some(0.0);

    let mut izlaz_m = ctx.run_ui(ulaz.clone(), |ui| {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let mut mem_m = memorija.clone();
                Map::new(Some(&mut biljeznik_mercator), &mut mem_m, ispitne_tocke[0])
                    .show(ui, |_, _, _, _| {});
            });
    });
    izlaz_m.textures_delta.clear();

    let klon_celije = Rc::clone(&zabiljezeni_plugin_ekran);
    let mut izlaz_h = ctx.run_ui(ulaz, |ui| {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let mut mem_h = memorija.clone();
                Map::new(Some(&mut biljeznik_htrs96), &mut mem_h, ispitne_tocke[0])
                    .with_projection(&htrs96)
                    .with_plugin(IspitniPlugin {
                        polozaj: tocka_za_plugin,
                        zabiljezeni_ekran: Rc::clone(&klon_celije),
                    })
                    .show(ui, |_, _, _, _| {});
            });
    });
    izlaz_h.textures_delta.clear();

    let plocica_mercator_zagreb_z14 = biljeznik_mercator
        .zatrazeni
        .first()
        .copied()
        .ok_or_else(|| "Mercator nije zatražio nijednu pločicu".to_string())?;
    let plocica_htrs96_zagreb_z14 = biljeznik_htrs96
        .zatrazeni
        .first()
        .copied()
        .ok_or_else(|| "HTRS96 nije zatražio nijednu pločicu".to_string())?;

    let plugin_ekran = zabiljezeni_plugin_ekran
        .get()
        .ok_or_else(|| "Plugin se nije izvršio u Map::show".to_string())?;
    let ocekivani_split = projektor_htrs96.project(tocka_za_plugin).to_pos2();
    let plugin_koristi_htrs96_projektor = (plugin_ekran - ocekivani_split).length() < 1e-3;

    // 5. Provjera DGU WMS okvira za središnju pločicu u HTRS96Projekciji
    let (min_e, min_n, max_e, max_n) = htrs96.htrs96_okvir_plocice(plocica_htrs96_zagreb_z14);
    let dgu_wms_htrs96_sadrzi_zagreb =
        zg_e >= min_e && zg_e <= max_e && zg_n >= min_n && zg_n <= max_n;

    let izvjestaj = IzvjestajProjekcije {
        maks_povratna_greska_stupnjevi: maks_greska,
        omjer_istok_sjever_htrs96,
        omjer_istok_sjever_mercator,
        kut_konvergencije_ilok_stupnjevi,
        plugin_koristi_htrs96_projektor,
        plocica_mercator_zagreb_z14,
        plocica_htrs96_zagreb_z14,
        dgu_wms_htrs96_sadrzi_zagreb,
    };

    if izvjestaj.maks_povratna_greska_stupnjevi > 1e-6 {
        return Err(format!(
            "Povratna pogreška projekcije prevelika: {:.8}°",
            izvjestaj.maks_povratna_greska_stupnjevi
        ));
    }
    if (izvjestaj.omjer_istok_sjever_htrs96 - 1.0).abs() > 1e-4 {
        return Err(format!(
            "HTRS96 mjerilo nije izotropno: {}",
            izvjestaj.omjer_istok_sjever_htrs96
        ));
    }
    if !izvjestaj.plugin_koristi_htrs96_projektor {
        return Err("Plugin nije dobio očekivane HTRS96 piksele".to_string());
    }
    if !izvjestaj.dgu_wms_htrs96_sadrzi_zagreb {
        return Err("DGU WMS HTRS96 okvir središnje pločice ne sadrži Zagreb".to_string());
    }

    Ok(izvjestaj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_projection_htrs96_prolazi() {
        let izvjestaj = provjeri_with_projection(0.0).unwrap();
        assert!(izvjestaj.maks_povratna_greska_stupnjevi < 1e-7);
        assert!((izvjestaj.omjer_istok_sjever_htrs96 - 1.0).abs() < 1e-6);
        assert!(izvjestaj.kut_konvergencije_ilok_stupnjevi > 1.8);
        assert!(izvjestaj.plugin_koristi_htrs96_projektor);
        assert!(izvjestaj.dgu_wms_htrs96_sadrzi_zagreb);
        // Zbog različitog ishodišta mreže, TileId u HTRS96 i Web Mercatoru nisu isti:
        assert_ne!(
            izvjestaj.plocica_mercator_zagreb_z14,
            izvjestaj.plocica_htrs96_zagreb_z14
        );
    }

    #[test]
    fn test_with_projection_namjerni_kvar_pada() {
        // Namjerno pomaknemo projekciju za 25 piksela: povratna provjera i okvir pločice moraju pasti!
        let rezultat = provjeri_with_projection(25.0);
        assert!(
            rezultat.is_err(),
            "Provjera projekcije mora pasti kad je projekcija namjerno pokvarena"
        );
    }
}
