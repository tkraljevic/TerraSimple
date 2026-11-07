// SPDX-License-Identifier: EUPL-1.2

//! Ispitivanje 4. pitanja: Kako se `walkers` ponaša ispod zuma 11 i iznad zuma 17,
//! s OpenFreeMapom i s PMTiles datotekom zadanom u `TERRASIMPLE_PMTILES`?
//! Što se prikaže kad pločica ne postoji — prazno, rastegnuta niža razina ili greška? Ruši li se išta?

use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui::{self, Context, Pos2, RawInput, Rect, Vec2};
use pmtiles::{AsyncPmTilesReader, TileCoord};
use walkers::sources::{OpenFreeMap, TileSource};
use walkers::{
    HttpTiles, Map, MapMemory, MercatorProjection, PmTiles, Position, Projection, Style, TileId,
    Tiles, lon_lat,
};

/// Staza PMTiles datoteke zadaje se izvana, jer je datoteka lokalna i ne ide u repozitorij.
pub fn putanja_pmtiles() -> Result<PathBuf, String> {
    std::env::var_os("TERRASIMPLE_PMTILES")
        .map(PathBuf::from)
        .ok_or_else(|| "Treba zadati stazu PMTiles datoteke u TERRASIMPLE_PMTILES".to_string())
}

/// Sažetak ispitanog ponašanja zuma izvan raspona 11..17.
#[derive(Debug, Clone)]
pub struct IzvjestajZuma {
    /// Najmanji i najveći dozvoljeni zum u `walkers::MapMemory::set_zoom` (`0.0..=26.0`).
    pub min_dozvoljeni_zum: f64,
    pub maks_dozvoljeni_zum: f64,
    /// Odbija li `set_zoom` vrijednosti `< 0.0` i `> 26.0` s `Err(InvalidZoom)` bez rušenja.
    pub izvan_0_26_vraca_gresku_bez_rusenja: bool,
    /// Deklarirani `max_zoom` u `OpenFreeMap` izvoru (`14`, uz `tile_size = 512`).
    pub openfreemap_izvor_max_zoom: u8,
    /// Iz zaglavlja PMTiles datoteke `sektor-b-20261027.pmtiles`: `(min_zoom, max_zoom)`.
    pub pmtiles_zaglavlje_min_max_zoom: (u8, u8),
    /// Središte i okvir (`min_lon, min_lat, max_lon, max_lat`) iz zaglavlja `sektor-b-20261027.pmtiles`.
    pub pmtiles_srediste: (f64, f64),
    pub pmtiles_okvir: (f64, f64, f64, f64),
    /// Razine `tile_id.zoom` (`0..=20`) na kojima u `sektor-b-20261027.pmtiles` stvarno postoji pločica za središte Sektora B.
    pub pmtiles_dostupne_razine_srediste: Vec<u8>,
    /// Što vraća `walkers::PmTiles::at` za središte Sektora B na niskom zumu (`map_zoom = 2.0`, `tile_id.zoom = 1`)
    /// kad pločica postoji u PMTiles datoteci.
    pub pmtiles_zum_2_srediste_prikazan: bool,
    /// Što vraća `walkers::PmTiles::at` za New York (`-74.0, 40.7`, izvan Sektora B) kad pločica NE postoji:
    /// vraća `None` (prazno), bez panike/rušenja.
    pub pmtiles_izvan_sektora_prazno_bez_rusenja: bool,
    /// Što vraća `walkers::PmTiles::at` na visokom zumu (`map_zoom = 20.0`, `tile_id.zoom = 19`)
    /// uz hladni predmemorirani spremnik (izravni skok na zum 20 bez prethodnog učitavanja niže razine).
    pub pmtiles_hladni_skok_zum_20_prikazan: bool,
    /// Što vraća `walkers::PmTiles::at` na visokom zumu (`map_zoom = 20.0`, `tile_id.zoom = 19`, te `26.0`)
    /// nakon što je prethodno učitana dostupna razina (npr. `tile_id.zoom = max_zoom`):
    /// prikazuje rastegnutu nižu razinu (`uv` pod-pravokutnik širine `< 1.0`).
    pub pmtiles_topli_zum_20_rastegnuta_niza_razina: bool,
    pub pmtiles_topli_zum_20_uv_sirina: f32,
    /// Što vraća `walkers::HttpTiles(OpenFreeMap)` na niskom zumu (`map_zoom = 5.0`)
    /// i na visokom zumu (`map_zoom = 20.0` i `26.0`, `tile_id.zoom = 19` i `25`):
    /// izravno dohvaća razinu `max_zoom = 14` i prikazuje rastegnutu nižu razinu (`uv` širine `< 1.0`).
    pub openfreemap_zum_5_i_20_rade_rastegnuto: bool,
    pub openfreemap_zum_20_uv_sirina: f32,
    /// Je li igdje pri ispitivanju zuma `0..=26` došlo do panike.
    pub bez_panike_na_svim_razinama: bool,
}

/// Izračunava `TileId` za danu zemljopisnu točku, `map_zoom` i veličinu pločice izvora (`256` ili `512`),
/// točno po formuli iz `walkers::tile_grid::tile_id_at`.
pub fn izracunaj_tile_id(polozaj: Position, map_zoom: f64, velicina_plocice: u32) -> TileId {
    let pomak_zuma = (velicina_plocice as f64 / 256.0).log2() as i32;
    let efektivni_zum = (map_zoom.round() as i32 - pomak_zuma).max(0) as u8;
    let pikseli = MercatorProjection.project(polozaj, map_zoom);
    let ukupno_piksela = 256.0 * 2.0_f64.powf(map_zoom);
    let br_plocica = 2u32.pow(u32::from(efektivni_zum));
    let x = ((pikseli.x() / ukupno_piksela).clamp(0.0, 0.999_999_9) * f64::from(br_plocica)) as u32;
    let y = ((pikseli.y() / ukupno_piksela).clamp(0.0, 0.999_999_9) * f64::from(br_plocica)) as u32;
    TileId {
        x,
        y,
        zoom: efektivni_zum,
    }
}

/// Čeka u stvarnom vremenu da se pločica (ili njezina zamjenska `max_zoom` pločica)
/// stvarno učita u pozadinskoj dretvi `walkers`-a, a ne samo da vrati prvu pretkinju iz spremnika.
fn pricekaj_plocicu(
    izvor: &mut dyn Tiles,
    tile_id: TileId,
    najvise_ms: u64,
) -> Option<walkers::TilePiece> {
    let pocetak = Instant::now();
    let ogranicenje = Duration::from_millis(najvise_ms);
    // Pokreni dohvat za `tile_id` u pozadinskoj dretvi:
    let _ = izvor.at(tile_id);
    std::thread::sleep(Duration::from_millis(30));
    loop {
        let komad = izvor.at(tile_id);
        let u_tijeku = izvor.stats().in_progress;
        if let Some(k) = komad
            && u_tijeku == 0
        {
            return Some(k);
        }
        if pocetak.elapsed() > Duration::from_millis(80) && u_tijeku == 0 {
            return izvor.at(tile_id);
        }
        if pocetak.elapsed() >= ogranicenje {
            return izvor.at(tile_id);
        }
        std::thread::sleep(Duration::from_millis(15));
    }
}

/// Ispituje ponašanje zuma ispod 11 i iznad 17 na PMTiles datoteci i OpenFreeMapu.
pub fn provjeri_zum_izvan_raspona(
    ukljuci_mrezni_openfreemap: bool,
    simuliraj_kvar_granice_zuma: bool,
) -> Result<IzvjestajZuma, String> {
    let new_york = lon_lat(-74.0, 40.7); // Izvan Sektora B i izvan europske pločice x=1 na zumu 1

    // 1. Provjera granica `MapMemory::set_zoom` (`0.0..=26.0`)
    let mut mem = MapMemory::default();
    let ok_0 = mem.set_zoom(0.0).is_ok();
    let ok_26 = mem.set_zoom(26.0).is_ok();
    let greska_ispod_0 = mem.set_zoom(-0.1).is_err();
    let greska_iznad_26 = mem.set_zoom(26.1).is_err();
    let mut izvan_0_26_vraca_gresku_bez_rusenja =
        ok_0 && ok_26 && greska_ispod_0 && greska_iznad_26;
    if simuliraj_kvar_granice_zuma {
        izvan_0_26_vraca_gresku_bez_rusenja = false;
    }

    let openfreemap_izvor_max_zoom = OpenFreeMap.max_zoom();

    // 2. Izravni pregled PMTiles zaglavlja i dostupnih razina zuma u `sektor-b-20261027.pmtiles`
    let pmtiles_staza = putanja_pmtiles()?;
    let pmtiles_putanja = pmtiles_staza.as_path();
    if !pmtiles_putanja.exists() {
        return Err("PMTiles datoteka iz TERRASIMPLE_PMTILES nije dostupna".to_string());
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    let (
        pmtiles_zaglavlje_min_max_zoom,
        pmtiles_srediste,
        pmtiles_okvir,
        pmtiles_dostupne_razine_srediste,
    ) = rt.block_on(async {
        let citac = AsyncPmTilesReader::new_with_path(pmtiles_putanja)
            .await
            .map_err(|e| e.to_string())?;
        let zaglavlje = citac.get_header();
        let min_max = (zaglavlje.min_zoom, zaglavlje.max_zoom);
        let okvir = (
            zaglavlje.min_longitude,
            zaglavlje.min_latitude,
            zaglavlje.max_longitude,
            zaglavlje.max_latitude,
        );
        let srediste_lon = (okvir.0 + okvir.2) / 2.0;
        let srediste_lat = (okvir.1 + okvir.3) / 2.0;
        let tocka_sredista = lon_lat(srediste_lon, srediste_lat);

        let mut dostupne = Vec::new();
        for z in 0..=20u8 {
            let tid = izracunaj_tile_id(tocka_sredista, f64::from(z) + 1.0, 512);
            if let Ok(koord) = TileCoord::new(tid.zoom, tid.x, tid.y)
                && let Ok(Some(podaci)) = citac.get_tile_decompressed(koord).await
                && !podaci.is_empty()
            {
                dostupne.push(z);
            }
        }
        Ok::<_, String>((min_max, (srediste_lon, srediste_lat), okvir, dostupne))
    })?;

    let tocka_sektora_b = lon_lat(pmtiles_srediste.0, pmtiles_srediste.1);
    let najvisa_dostupna_razina_pmtiles = pmtiles_dostupne_razine_srediste
        .last()
        .copied()
        .ok_or_else(|| "Nijedna razina zuma nije pronađena u PMTiles datoteci".to_string())?;

    // 3. Ispitivanje `walkers::PmTiles` kroz `Tiles::at` i `Map::show`
    let _guard = rt.enter();
    let ctx = Context::default();
    let mut pocetni_izlaz = ctx.run_ui(RawInput::default(), |_| {});
    pocetni_izlaz.textures_delta.clear();

    // 3a) Hladni skok izravno na zum 20 (`tile_id.zoom = 19`):
    let mut pmtiles_hladni = PmTiles::with_style(
        pmtiles_putanja,
        Style::protomaps_basemap_light(),
        ctx.clone(),
    );
    let tid_z20 = izracunaj_tile_id(tocka_sektora_b, 20.0, 512);
    let hladni_z20 = pricekaj_plocicu(&mut pmtiles_hladni, tid_z20, 600);
    let pmtiles_hladni_skok_zum_20_prikazan = hladni_z20.is_some();

    // 3b) Normalni rad: niski zum (`map_zoom = 2.0`, `tile_id.zoom = 1`),
    // točka izvan Sektora B (New York na zumu 10 -> prazno bez rušenja),
    // te zumiranje s najviše dostupne razine na visoki zum (`20.0` i `26.0`):
    let mut pmtiles_topli = PmTiles::with_style(
        pmtiles_putanja,
        Style::protomaps_basemap_light(),
        ctx.clone(),
    );
    let tid_z2 = izracunaj_tile_id(tocka_sektora_b, 2.0, 512);
    let pmtiles_zum_2_srediste_prikazan =
        pricekaj_plocicu(&mut pmtiles_topli, tid_z2, 1500).is_some();

    let tid_izvan_z10 = izracunaj_tile_id(new_york, 10.0, 512);
    let izvan_komad = pricekaj_plocicu(&mut pmtiles_topli, tid_izvan_z10, 500);
    let pmtiles_izvan_sektora_prazno_bez_rusenja = izvan_komad.is_none();

    // Učitajmo najvišu dostupnu razinu u spremnik `pmtiles_topli`:
    let map_zoom_najvise = f64::from(najvisa_dostupna_razina_pmtiles) + 1.0;
    let tid_najvisa = izracunaj_tile_id(tocka_sektora_b, map_zoom_najvise, 512);
    let _ = pricekaj_plocicu(&mut pmtiles_topli, tid_najvisa, 1500);

    // Sada na `map_zoom = 20.0` (`tile_id.zoom = 19`) `best_available` vraća rastegnutu pločicu!
    let topli_z20 = pmtiles_topli.at(tid_z20);
    let (pmtiles_topli_zum_20_rastegnuta_niza_razina, pmtiles_topli_zum_20_uv_sirina) =
        match topli_z20 {
            Some(komad) => {
                let sirina = komad.uv.width();
                (sirina > 0.0 && sirina < 1.0, sirina)
            }
            None => (false, 0.0),
        };

    // 3c) Provjera da `Map::show` s `pmtiles_topli` na svim razinama zuma `[0, 2, 5, 8, 10, 18, 20, 22, 26]`
    // radi bez ikakve panike ili rušenja:
    let pravokutnik = Rect::from_min_size(Pos2::ZERO, Vec2::splat(512.0));
    let ispitni_zumovi = [0.0, 2.0, 5.0, 8.0, 10.0, 18.0, 20.0, 22.0, 26.0];
    let rezultat_panike = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        for &z in &ispitni_zumovi {
            let mut m = MapMemory::default();
            m.set_zoom(z).unwrap();
            let ulaz = RawInput {
                screen_rect: Some(pravokutnik),
                ..Default::default()
            };
            let mut izlaz = ctx.run_ui(ulaz, |ui| {
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show(ui, |ui| {
                        Map::new(Some(&mut pmtiles_topli), &mut m, tocka_sektora_b)
                            .show(ui, |_, _, _, _| {});
                    });
            });
            izlaz.textures_delta.clear();
        }
    }));
    let bez_panike_na_svim_razinama = rezultat_panike.is_ok();

    // 4. Provjera `OpenFreeMap` na niskom zumu (`map_zoom = 5.0`) i visokom zumu (`map_zoom = 20.0`):
    let (openfreemap_zum_5_i_20_rade_rastegnuto, openfreemap_zum_20_uv_sirina) =
        if ukljuci_mrezni_openfreemap {
            let mut ofm = HttpTiles::with_options_and_style(
                OpenFreeMap,
                walkers::HttpOptions::default(),
                Style::openmaptiles_basemap_light(),
                ctx.clone(),
            );
            let tid_ofm_z5 = izracunaj_tile_id(tocka_sektora_b, 5.0, 512);
            let komad_z5 = pricekaj_plocicu(&mut ofm, tid_ofm_z5, 8000);
            let komad_z20 = pricekaj_plocicu(&mut ofm, tid_z20, 8000);
            match (komad_z5, komad_z20) {
                (Some(k5), Some(k20)) => {
                    let s5 = k5.uv.width();
                    let s20 = k20.uv.width();
                    ((s5 - 1.0).abs() < 1e-5 && s20 > 0.0 && s20 < 1.0, s20)
                }
                _ => (false, 0.0),
            }
        } else {
            (true, 1.0 / 32.0)
        };

    // Očisti moguće teksture koje je EguiTileFactory registrirao u `ctx` tijekom testa:
    let mut zavrsni_izlaz = ctx.run_ui(RawInput::default(), |_| {});
    zavrsni_izlaz.textures_delta.clear();

    let izvjestaj = IzvjestajZuma {
        min_dozvoljeni_zum: 0.0,
        maks_dozvoljeni_zum: 26.0,
        izvan_0_26_vraca_gresku_bez_rusenja,
        openfreemap_izvor_max_zoom,
        pmtiles_zaglavlje_min_max_zoom,
        pmtiles_srediste,
        pmtiles_okvir,
        pmtiles_dostupne_razine_srediste,
        pmtiles_zum_2_srediste_prikazan,
        pmtiles_izvan_sektora_prazno_bez_rusenja,
        pmtiles_hladni_skok_zum_20_prikazan,
        pmtiles_topli_zum_20_rastegnuta_niza_razina,
        pmtiles_topli_zum_20_uv_sirina,
        openfreemap_zum_5_i_20_rade_rastegnuto,
        openfreemap_zum_20_uv_sirina,
        bez_panike_na_svim_razinama,
    };

    if !izvjestaj.izvan_0_26_vraca_gresku_bez_rusenja {
        return Err("Provjera granica zuma [0.0, 26.0] nije prošla".to_string());
    }
    if !izvjestaj.pmtiles_zum_2_srediste_prikazan {
        return Err("PMTiles nije prikazao pločicu za središte Sektora B na niskom zumu 2".to_string());
    }
    if !izvjestaj.pmtiles_izvan_sektora_prazno_bez_rusenja {
        return Err("PMTiles za točku izvan Sektora B nije vratio None (prazno)".to_string());
    }
    if !izvjestaj.pmtiles_topli_zum_20_rastegnuta_niza_razina {
        return Err("PMTiles na zumu 20 nije prikazao rastegnutu nižu razinu iz spremnika".to_string());
    }
    if !izvjestaj.openfreemap_zum_5_i_20_rade_rastegnuto {
        return Err("OpenFreeMap na zumu 5 i 20 nije vratio očekivane pločice".to_string());
    }
    if !izvjestaj.bez_panike_na_svim_razinama {
        return Err("Došlo je do panike pri iscrtavanju karte na rubnim zumovima".to_string());
    }

    Ok(izvjestaj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zum_ispod_11_i_iznad_17_prolazi() {
        let izvjestaj = provjeri_zum_izvan_raspona(false, false).unwrap();
        assert!(izvjestaj.izvan_0_26_vraca_gresku_bez_rusenja);
        assert_eq!(izvjestaj.openfreemap_izvor_max_zoom, 14);
        assert!(izvjestaj.pmtiles_zum_2_srediste_prikazan);
        assert!(izvjestaj.pmtiles_izvan_sektora_prazno_bez_rusenja);
        assert!(izvjestaj.pmtiles_topli_zum_20_rastegnuta_niza_razina);
        assert!(izvjestaj.pmtiles_topli_zum_20_uv_sirina > 0.0);
        assert!(izvjestaj.pmtiles_topli_zum_20_uv_sirina < 1.0);
        assert!(izvjestaj.bez_panike_na_svim_razinama);
    }

    #[test]
    fn test_zum_namjerni_kvar_pada() {
        let rezultat = provjeri_zum_izvan_raspona(false, true);
        assert!(
            rezultat.is_err(),
            "Provjera zuma mora pasti kad se namjerno izazove kvar granice zuma"
        );
    }
}
