use eframe::egui::{
    self, Color32, ColorImage, Pos2, Rect, TextureHandle, TextureOptions, Vec2, pos2, vec2,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const MANIFEST_JSON: &str = include_str!("../../resursi/ikone/sucelje/efekti/manifest.json");
const ZNAK_32_PNG: &[u8] = include_bytes!("../../resursi/ikone/terrasimple-32.png");
const ZNAK_64_PNG: &[u8] = include_bytes!("../../resursi/ikone/terrasimple-64.png");

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct ProfilVelicine {
    pub crtez: u32,
    pub rub: u32,
    pub platno: u32,
    #[serde(default)]
    pub potez: u32,
    #[serde(default)]
    pub jednostavan: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct IzvozIkone {
    pub ikona: String,
    pub tema: String,
    pub stanje: String,
    pub crtez: u32,
    pub rub: u32,
    pub platno: u32,
    pub svg: String,
    #[serde(rename = "png1x")]
    pub png_1x: String,
    #[serde(rename = "png2x")]
    pub png_2x: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ManifestIkona {
    pub velicine: BTreeMap<String, ProfilVelicine>,
    pub izvozi: Vec<IzvozIkone>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SkalaIkone {
    JedanX,
    DvaX,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OdabirSkale {
    pub skala: SkalaIkone,
    pub neprovjerena_skala: bool,
}

impl OdabirSkale {
    #[must_use]
    pub fn za_faktor(pixels_per_point: f32) -> Self {
        let skala = if pixels_per_point >= 1.5 {
            SkalaIkone::DvaX
        } else {
            SkalaIkone::JedanX
        };
        let provjerena =
            (pixels_per_point - 1.0).abs() <= 0.01 || (pixels_per_point - 2.0).abs() <= 0.01;
        Self {
            skala,
            neprovjerena_skala: !provjerena,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct KljucTeksture {
    ikona: String,
    tema: String,
    stanje: String,
    crtez: u32,
    skala: SkalaIkone,
}

pub struct SkladisteIkona {
    manifest: ManifestIkona,
    korijen: PathBuf,
    teksture: BTreeMap<KljucTeksture, TextureHandle>,
    znak_1x: Option<TextureHandle>,
    znak_2x: Option<TextureHandle>,
}

impl Default for SkladisteIkona {
    fn default() -> Self {
        Self::novo()
    }
}

impl SkladisteIkona {
    #[must_use]
    pub fn novo() -> Self {
        let manifest: ManifestIkona =
            serde_json::from_str(MANIFEST_JSON).expect("neispravan manifest.json ikona");
        let korijen = Path::new(env!("CARGO_MANIFEST_DIR")).join("../resursi/ikone/sucelje/efekti");
        Self {
            manifest,
            korijen,
            teksture: BTreeMap::new(),
            znak_1x: None,
            znak_2x: None,
        }
    }

    #[must_use]
    pub fn manifest(&self) -> &ManifestIkona {
        &self.manifest
    }

    #[must_use]
    pub fn pronadi_izvoz(
        &self,
        ikona: &str,
        tamna: bool,
        stanje: &str,
        crtez: u32,
    ) -> Option<&IzvozIkone> {
        let naziv_teme = naziv_teme_za_izvoz(tamna);
        self.trazi_izvoz_tocno(ikona, naziv_teme, stanje, crtez)
            .or_else(|| self.trazi_izvoz_tocno(ikona, naziv_teme, "obicno", crtez))
    }

    fn trazi_izvoz_tocno(
        &self,
        ikona: &str,
        tema: &str,
        stanje: &str,
        crtez: u32,
    ) -> Option<&IzvozIkone> {
        self.manifest.izvozi.iter().find(|izvoz| {
            izvoz.ikona == ikona
                && izvoz.tema == tema
                && izvoz.stanje == stanje
                && izvoz.crtez == crtez
        })
    }

    #[must_use]
    pub fn ucitaj_sliku(&self, izvoz: &IzvozIkone, skala: SkalaIkone) -> ColorImage {
        let relativno = match skala {
            SkalaIkone::JedanX => &izvoz.png_1x,
            SkalaIkone::DvaX => &izvoz.png_2x,
        };
        let putanja = self.korijen.join(relativno);
        let bajtovi = std::fs::read(&putanja)
            .unwrap_or_else(|e| panic!("nije moguće pročitati ikonu {}: {e}", putanja.display()));
        dekodiraj_png(&bajtovi)
    }

    pub fn crtaj(
        &mut self,
        ui: &egui::Ui,
        okvir: Rect,
        ikona: &str,
        tamna: bool,
        stanje: &str,
        crtez: u32,
    ) -> Option<Rect> {
        let izvoz = self.pronadi_izvoz(ikona, tamna, stanje, crtez)?.clone();
        let faktor = ui.ctx().pixels_per_point();
        let odabir = OdabirSkale::za_faktor(faktor);
        let platno_okvir = izracunaj_platno_ikone(okvir, &izvoz, faktor);
        let kljuc = KljucTeksture {
            ikona: izvoz.ikona.clone(),
            tema: izvoz.tema.clone(),
            stanje: izvoz.stanje.clone(),
            crtez: izvoz.crtez,
            skala: odabir.skala,
        };
        if !self.teksture.contains_key(&kljuc) {
            let slika = self.ucitaj_sliku(&izvoz, odabir.skala);
            let naziv = format!(
                "{}_{}_{}_{}_{:?}",
                kljuc.ikona, kljuc.tema, kljuc.stanje, kljuc.crtez, kljuc.skala
            );
            let tekstura = ui.ctx().load_texture(naziv, slika, TextureOptions::LINEAR);
            self.teksture.insert(kljuc.clone(), tekstura);
        }
        let tekstura = self.teksture.get(&kljuc)?;
        crtaj_teksturu(ui, tekstura.id(), platno_okvir);
        Some(platno_okvir)
    }

    pub fn crtaj_znak_aplikacije(&mut self, ui: &egui::Ui, okvir: Rect) -> Rect {
        let odabir = OdabirSkale::za_faktor(ui.ctx().pixels_per_point());
        let id = match odabir.skala {
            SkalaIkone::JedanX => self
                .znak_1x
                .get_or_insert_with(|| {
                    ui.ctx().load_texture(
                        "terrasimple_znak_32",
                        dekodiraj_png(ZNAK_32_PNG),
                        TextureOptions::LINEAR,
                    )
                })
                .id(),
            SkalaIkone::DvaX => self
                .znak_2x
                .get_or_insert_with(|| {
                    ui.ctx().load_texture(
                        "terrasimple_znak_64",
                        dekodiraj_png(ZNAK_64_PNG),
                        TextureOptions::LINEAR,
                    )
                })
                .id(),
        };
        crtaj_teksturu(ui, id, okvir);
        okvir
    }
}

#[must_use]
pub const fn naziv_teme_za_izvoz(tamna: bool) -> &'static str {
    if tamna { "tamna" } else { "svijetla" }
}

#[must_use]
pub fn izracunaj_platno_ikone(okvir: Rect, izvoz: &IzvozIkone, pixels_per_point: f32) -> Rect {
    let rub = izvoz.rub as f32;
    let platno = izvoz.platno as f32;
    let korak = pixels_per_point.max(1.0);
    let min_x = ((okvir.min.x - rub) * korak).round() / korak;
    let min_y = ((okvir.min.y - rub) * korak).round() / korak;
    Rect::from_min_size(pos2(min_x, min_y), vec2(platno, platno))
}

#[must_use]
pub fn okvir_ikone_u_sredini(srediste: Pos2, crtez: u32) -> Rect {
    let d = crtez as f32;
    Rect::from_center_size(srediste, Vec2::splat(d))
}

fn crtaj_teksturu(ui: &egui::Ui, id: egui::TextureId, okvir: Rect) {
    let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
    ui.painter().image(id, okvir, uv, Color32::WHITE);
}

#[must_use]
pub fn dekodiraj_png(bajtovi: &[u8]) -> ColorImage {
    let slika = image::load_from_memory(bajtovi)
        .expect("neispravna PNG datoteka ikone")
        .into_rgba8();
    let dimenzije = [slika.width() as usize, slika.height() as usize];
    ColorImage::from_rgba_unmultiplied(dimenzije, slika.as_raw())
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn manifest_ima_sve_profile_i_platno_jednako_crtezu_plus_dva_ruba() {
        let skladiste = SkladisteIkona::novo();
        assert_eq!(skladiste.manifest().velicine.len(), 5);
        for izvoz in &skladiste.manifest().izvozi {
            assert_eq!(izvoz.platno, izvoz.crtez + 2 * izvoz.rub);
        }
    }

    #[test]
    fn platno_se_siri_oko_okvira_i_ne_stisce_u_crtez() {
        let skladiste = SkladisteIkona::novo();
        let izvoz = skladiste
            .pronadi_izvoz("odaberi", false, "odabrani_alat", 22)
            .expect("postoji odaberi 22px");
        assert_eq!(izvoz.crtez, 22);
        assert_eq!(izvoz.rub, 3);
        assert_eq!(izvoz.platno, 28);

        let okvir_crteza = Rect::from_min_size(pos2(20.0, 30.0), vec2(22.0, 22.0));
        let okvir_platna = izracunaj_platno_ikone(okvir_crteza, izvoz, 2.0);
        assert_eq!(okvir_platna.min, pos2(17.0, 27.0));
        assert_eq!(okvir_platna.size(), vec2(28.0, 28.0));
    }

    #[test]
    fn odabir_teme_ucitava_razlicite_ikone_za_svijetlu_i_tamnu_temu() {
        let skladiste = SkladisteIkona::novo();
        let svijetli = skladiste
            .pronadi_izvoz("mapa", false, "obicno", 18)
            .expect("svijetla ikona mape");
        let tamni = skladiste
            .pronadi_izvoz("mapa", true, "obicno", 18)
            .expect("tamna ikona mape");

        assert_eq!(svijetli.ikona, "mapa");
        assert_eq!(tamni.ikona, "mapa");
        assert_eq!(svijetli.tema, "svijetla");
        assert_eq!(tamni.tema, "tamna");
        assert_ne!(svijetli.png_2x, tamni.png_2x);

        let slika_svijetla = skladiste.ucitaj_sliku(svijetli, SkalaIkone::DvaX);
        let slika_tamna = skladiste.ucitaj_sliku(tamni, SkalaIkone::DvaX);
        assert_eq!(slika_svijetla.size, [48, 48]);
        assert_eq!(slika_tamna.size, [48, 48]);
        assert_ne!(slika_svijetla.pixels, slika_tamna.pixels);
    }

    #[test]
    fn odabir_skale_oznacava_neprovjerene_faktore() {
        let jedan = OdabirSkale::za_faktor(1.0);
        assert_eq!(jedan.skala, SkalaIkone::JedanX);
        assert!(!jedan.neprovjerena_skala);

        let dva = OdabirSkale::za_faktor(2.0);
        assert_eq!(dva.skala, SkalaIkone::DvaX);
        assert!(!dva.neprovjerena_skala);

        let medjuskala = OdabirSkale::za_faktor(1.25);
        assert_eq!(medjuskala.skala, SkalaIkone::JedanX);
        assert!(medjuskala.neprovjerena_skala);

        let visoka = OdabirSkale::za_faktor(1.75);
        assert_eq!(visoka.skala, SkalaIkone::DvaX);
        assert!(visoka.neprovjerena_skala);
    }

    #[test]
    fn crtanje_ikone_vraca_okvir_platna_i_radi_kesiranje() {
        let mut skladiste = SkladisteIkona::novo();
        let mut harness = egui_kittest::Harness::builder()
            .with_size(vec2(100.0, 100.0))
            .build_ui_state(
                move |ui, nacrtano: &mut Option<Rect>| {
                    let okvir = Rect::from_min_size(pos2(10.0, 10.0), vec2(18.0, 18.0));
                    *nacrtano = skladiste.crtaj(ui, okvir, "trazi", false, "obicno", 18);
                },
                None,
            );
        harness.run();
        let ima_sliku = harness.output().shapes.iter().any(|s| {
            matches!(&s.shape, egui::Shape::Mesh(m) if m.texture_id != egui::TextureId::default())
        });
        assert!(ima_sliku, "slika ikone mora stvarno biti na zaslonu");
        let nacrtano = harness.state().expect("ikona mora biti nacrtana");
        assert!(nacrtano.width() >= 18.0);
        assert!(nacrtano.height() >= 18.0);
    }

    #[test]
    fn dekodiranje_slike_daje_neprazne_piksele() {
        let skladiste = SkladisteIkona::novo();
        let izvoz = skladiste
            .pronadi_izvoz("trazi", false, "obicno", 18)
            .unwrap();
        let slika = skladiste.ucitaj_sliku(izvoz, SkalaIkone::DvaX);
        assert!(!slika.pixels.is_empty());
        assert_eq!(slika.size[0], 48);
    }
}
