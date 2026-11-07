// SPDX-License-Identifier: EUPL-1.2

//! Ispitivanje 5. pitanja: Radi li tamna tema karte (`Style::openmaptiles_basemap_dark` i
//! `Style::protomaps_basemap_dark`) iz `probe/karta`?
//!
//! Uključuje:
//! - Provjeru izravnog dekodiranja MVT pločice sa svijetlim i tamnim stilom.
//! - Provjeru spremljenih snimki zaslona (`snimke/svijetla_tema.png` i `snimke/tamna_tema.png`)
//!   snimljenih na OpenFreeMap podlozi (bez DGU WMS-a i bez PMTiles datoteke u repozitoriju!).

use std::collections::HashSet;
use std::path::Path;

use eframe::egui::ColorImage;
use walkers::{Drawable, Style, Tile};

/// Relativna putanja snimke svijetle teme (OpenFreeMap).
pub const PUTANJA_SNIMKE_SVIJETLA: &str = "snimke/svijetla_tema.png";
/// Relativna putanja snimke tamne teme (OpenFreeMap).
pub const PUTANJA_SNIMKE_TAMNA: &str = "snimke/tamna_tema.png";

/// Izvještaj o usporedbi svijetle i tamne teme.
#[derive(Debug, Clone)]
pub struct IzvjestajTema {
    /// Broj slojeva u `Style::openmaptiles_basemap_light()` i `dark()`.
    pub broj_slojeva_openmaptiles_svijetla: usize,
    pub broj_slojeva_openmaptiles_tamna: usize,
    /// Broj slojeva u `Style::protomaps_basemap_light()` i `dark()`.
    pub broj_slojeva_protomaps_svijetla: usize,
    pub broj_slojeva_protomaps_tamna: usize,
    /// Prosječna svjetlina (0..255) pozadine i objekata iz MVT pločice u svijetloj i tamnoj temi.
    pub mvt_svjetlina_svijetla: f64,
    pub mvt_svjetlina_tamna: f64,
    /// Prosječna svjetlina (0..255) središnjeg dijela snimke zaslona za svijetlu temu.
    pub snimka_svjetlina_svijetla: Option<f64>,
    /// Prosječna svjetlina (0..255) središnjeg dijela snimke zaslona za tamnu temu.
    pub snimka_svjetlina_tamna: Option<f64>,
    /// Broj različitih kvantiziranih boja na snimci svijetle i tamne teme (dokaz da karta nije prazna).
    pub snimka_raznolikost_boja_svijetla: usize,
    pub snimka_raznolikost_boja_tamna: usize,
}

/// Izračunava prosječnu relativnu svjetlinu (`0.2126 R + 0.7152 G + 0.0722 B`) svih vrhova u MVT pločici.
fn prosjecna_svjetlina_mvt_plocice(podaci_mvt: &[u8], stil: &Style) -> Result<f64, String> {
    let plocica = Tile::from_mvt(podaci_mvt, stil, 14, 512).map_err(|e| e.to_string())?;
    match plocica {
        Tile::Vector { drawables, .. } => {
            let mut zbroj = 0.0_f64;
            let mut broj = 0usize;
            for d in drawables.iter() {
                match d {
                    Drawable::Fill(mesh) => {
                        for v in &mesh.vertices {
                            let [r, g, b, _] = v.color.to_array();
                            zbroj += 0.2126 * f64::from(r)
                                + 0.7152 * f64::from(g)
                                + 0.0722 * f64::from(b);
                            broj += 1;
                        }
                    }
                    Drawable::Lines(linije) => {
                        for l in linije {
                            let [r, g, b, _] = l.color.to_array();
                            zbroj += 0.2126 * f64::from(r)
                                + 0.7152 * f64::from(g)
                                + 0.0722 * f64::from(b);
                            broj += 1;
                        }
                    }
                }
            }
            if broj == 0 {
                return Err("MVT pločica nije proizvela nijedan vrh za zadani stil".to_string());
            }
            Ok(zbroj / (broj as f64))
        }
        Tile::Raster(_) => Err("Očekivana vektorska pločica iz Tile::from_mvt".to_string()),
    }
}

/// Analizira središnje područje karte na slici (izbjegavajući gornju alatnu traku)
/// i vraća `(prosjecna_svjetlina_0_255, broj_kvantiziranih_boja)`.
pub fn analiziraj_sliku_karte(slika: &image::RgbaImage) -> (f64, usize) {
    let sirina = slika.width();
    let visina = slika.height();
    let min_x = sirina / 10;
    let max_x = sirina * 9 / 10;
    let min_y = visina / 5;
    let max_y = visina * 9 / 10;

    let mut zbroj_svjetline = 0.0_f64;
    let mut broj_piksela = 0usize;
    let mut kvantizirane_boje = HashSet::new();

    for y in min_y..max_y {
        for x in min_x..max_x {
            let p = slika.get_pixel(x, y);
            let [r, g, b, _] = p.0;
            let svjetlina =
                0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b);
            zbroj_svjetline += svjetlina;
            broj_piksela += 1;
            // Kvantizacija na 4 bita po kanalu (korak 16) za mjerenje raznolikosti boja
            kvantizirane_boje.insert((r / 8, g / 8, b / 8));
        }
    }

    let prosjek = if broj_piksela > 0 {
        zbroj_svjetline / (broj_piksela as f64)
    } else {
        0.0
    };
    (prosjek, kvantizirane_boje.len())
}

/// Sprema `egui::ColorImage` u PNG datoteku.
pub fn spremi_color_image_u_png(slika: &ColorImage, putanja: &Path) -> Result<(), String> {
    if let Some(roditelj) = putanja.parent() {
        std::fs::create_dir_all(roditelj).map_err(|e| e.to_string())?;
    }
    let sirina = slika.width() as u32;
    let visina = slika.height() as u32;
    let mut sirovi_rgba = Vec::with_capacity((sirina * visina * 4) as usize);
    for piksel in &slika.pixels {
        sirovi_rgba.extend_from_slice(&piksel.to_array());
    }
    image::save_buffer(
        putanja,
        &sirovi_rgba,
        sirina,
        visina,
        image::ColorType::Rgba8,
    )
    .map_err(|e| e.to_string())
}

/// Provjerava dvije slike (svijetlu i tamnu temu) i vraća grešku ako bilo koja slika
/// nije stvarno iscrtana karta ili ako svijetla tema nije osjetno svjetlija od tamne teme.
pub fn provjeri_par_snimki(
    slika_svijetla: &image::RgbaImage,
    slika_tamna: &image::RgbaImage,
) -> Result<(f64, usize, f64, usize), String> {
    if slika_svijetla.width() < 400 || slika_svijetla.height() < 300 {
        return Err("Snimka svijetle teme je premalih dimenzija".to_string());
    }
    if slika_tamna.width() < 400 || slika_tamna.height() < 300 {
        return Err("Snimka tamne teme je premalih dimenzija".to_string());
    }

    let (svjetlina_svijetla, boje_svijetla) = analiziraj_sliku_karte(slika_svijetla);
    let (svjetlina_tamna, boje_tamna) = analiziraj_sliku_karte(slika_tamna);

    if boje_svijetla < 15 {
        return Err(format!(
            "Snimka svijetle teme izgleda prazno (samo {boje_svijetla} kvantiziranih boja)"
        ));
    }
    if boje_tamna < 15 {
        return Err(format!(
            "Snimka tamne teme izgleda prazno (samo {boje_tamna} kvantiziranih boja)"
        ));
    }
    if svjetlina_svijetla < 150.0 {
        return Err(format!(
            "Snimka svijetle teme je pretamna (prosječna svjetlina = {svjetlina_svijetla:.1}, očekivano >= 150)"
        ));
    }
    if svjetlina_tamna > 85.0 {
        return Err(format!(
            "Snimka tamne teme je presvijetla (prosječna svjetlina = {svjetlina_tamna:.1}, očekivano <= 85)"
        ));
    }
    if svjetlina_svijetla - svjetlina_tamna < 90.0 {
        return Err(format!(
            "Premala razlika svjetline između svijetle ({svjetlina_svijetla:.1}) i tamne ({svjetlina_tamna:.1}) teme"
        ));
    }

    Ok((svjetlina_svijetla, boje_svijetla, svjetlina_tamna, boje_tamna))
}

/// Provjerava stilove tema (uključujući dekodiranje stvarne MVT pločice i provjeru PNG snimki ako postoje).
/// Ako je `zamijeni_teme_za_kvar = true`, namjerno zamjenjuje svijetlu i tamnu temu kako bi dokazao pad provjere.
pub fn provjeri_teme(
    zahtijevaj_snimke_na_disku: bool,
    zamijeni_teme_za_kvar: bool,
) -> Result<IzvjestajTema, String> {
    let omt_svijetla = Style::openmaptiles_basemap_light();
    let omt_tamna = Style::openmaptiles_basemap_dark();
    let pm_svijetla = Style::protomaps_basemap_light();
    let pm_tamna = Style::protomaps_basemap_dark();

    let broj_slojeva_openmaptiles_svijetla = omt_svijetla.layers.len();
    let broj_slojeva_openmaptiles_tamna = omt_tamna.layers.len();
    let broj_slojeva_protomaps_svijetla = pm_svijetla.layers.len();
    let broj_slojeva_protomaps_tamna = pm_tamna.layers.len();

    // Dohvatimo jednu stvarnu MVT pločicu iz PMTiles datoteke (samo u memoriji radi testa dekodiranja
    // stilova `protomaps_basemap_light` i `protomaps_basemap_dark`; ništa iz PMTilesa se ne snima na disk!).
    let pmtiles_putanja = crate::zum::putanja_pmtiles()?;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let mvt_bajtovi = rt.block_on(async {
        let citac = pmtiles::AsyncPmTilesReader::new_with_path(pmtiles_putanja)
            .await
            .map_err(|e| e.to_string())?;
        let zag = citac.get_header();
        let srediste = walkers::lon_lat(
            (zag.min_longitude + zag.max_longitude) / 2.0,
            (zag.min_latitude + zag.max_latitude) / 2.0,
        );
        let tid = crate::zum::izracunaj_tile_id(srediste, 14.0, 512);
        let koord = pmtiles::TileCoord::new(tid.zoom, tid.x, tid.y).map_err(|e| e.to_string())?;
        citac
            .get_tile_decompressed(koord)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Nedostaje MVT pločica središta Sektora B za provjeru stila".to_string())
    })?;

    let (stil_prvi, stil_drugi) = if zamijeni_teme_za_kvar {
        (&omt_tamna, &omt_svijetla)
    } else {
        (&omt_svijetla, &omt_tamna)
    };

    let mvt_svjetlina_svijetla = prosjecna_svjetlina_mvt_plocice(&mvt_bajtovi, stil_prvi)?;
    let mvt_svjetlina_tamna = prosjecna_svjetlina_mvt_plocice(&mvt_bajtovi, stil_drugi)?;

    if mvt_svjetlina_svijetla <= mvt_svjetlina_tamna + 40.0 {
        return Err(format!(
            "MVT svijetla tema ({mvt_svjetlina_svijetla:.1}) nije osjetno svjetlija od tamne ({mvt_svjetlina_tamna:.1})"
        ));
    }

    let putanja_sv = Path::new(PUTANJA_SNIMKE_SVIJETLA);
    let putanja_tm = Path::new(PUTANJA_SNIMKE_TAMNA);

    let (
        snimka_svjetlina_svijetla,
        snimka_svjetlina_tamna,
        snimka_raznolikost_boja_svijetla,
        snimka_raznolikost_boja_tamna,
    ) = if putanja_sv.exists() && putanja_tm.exists() {
        let img_sv = image::open(putanja_sv)
            .map_err(|e| e.to_string())?
            .to_rgba8();
        let img_tm = image::open(putanja_tm)
            .map_err(|e| e.to_string())?
            .to_rgba8();
        let (prva, druga) = if zamijeni_teme_za_kvar {
            (&img_tm, &img_sv)
        } else {
            (&img_sv, &img_tm)
        };
        let (sv_s, boje_s, sv_t, boje_t) = provjeri_par_snimki(prva, druga)?;
        (Some(sv_s), Some(sv_t), boje_s, boje_t)
    } else if zahtijevaj_snimke_na_disku {
        return Err(format!(
            "Nedostaju snimke zaslona na disku: {} i {}",
            PUTANJA_SNIMKE_SVIJETLA, PUTANJA_SNIMKE_TAMNA
        ));
    } else {
        (None, None, 0, 0)
    };

    Ok(IzvjestajTema {
        broj_slojeva_openmaptiles_svijetla,
        broj_slojeva_openmaptiles_tamna,
        broj_slojeva_protomaps_svijetla,
        broj_slojeva_protomaps_tamna,
        mvt_svjetlina_svijetla,
        mvt_svjetlina_tamna,
        snimka_svjetlina_svijetla,
        snimka_svjetlina_tamna,
        snimka_raznolikost_boja_svijetla,
        snimka_raznolikost_boja_tamna,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_teme_mvt_i_stilovi_prolaze() {
        let izvjestaj = provjeri_teme(false, false).unwrap();
        assert!(izvjestaj.broj_slojeva_openmaptiles_svijetla > 20);
        assert_eq!(
            izvjestaj.broj_slojeva_openmaptiles_svijetla,
            izvjestaj.broj_slojeva_openmaptiles_tamna
        );
        assert!(izvjestaj.mvt_svjetlina_svijetla > izvjestaj.mvt_svjetlina_tamna + 40.0);
    }

    #[test]
    fn test_teme_namjerni_kvar_pada() {
        let rezultat = provjeri_teme(false, true);
        assert!(
            rezultat.is_err(),
            "Provjera tema mora pasti kad se svijetla i tamna tema namjerno zamijene"
        );
    }
}
