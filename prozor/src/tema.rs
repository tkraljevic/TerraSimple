use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, FontId};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Boje {
    pub prozor: Color32,
    pub ploca: Color32,
    pub ploca_2: Color32,
    pub izbornik: Color32,
    pub status: Color32,
    pub obrub: Color32,
    pub obrub_2: Color32,
    pub tekst: Color32,
    pub tekst_2: Color32,
    pub tekst_3: Color32,
    pub primarna: Color32,
    pub primarna_tamna: Color32,
    pub primarna_svijetla: Color32,
    pub primarna_tekst: Color32,
    pub alat_odabran: Color32,
    pub alat_odabran_tekst: Color32,
    pub plava: Color32,
    pub plava_svijetla: Color32,
    pub narancasta: Color32,
    pub narancasta_svijetla: Color32,
    pub zelena: Color32,
    pub zelena_svijetla: Color32,
    pub crvena: Color32,
    pub crvena_svijetla: Color32,
    pub upozorenje: Color32,
    pub upozorenje_svijetla: Color32,
    pub odabir: Color32,
    pub odabir_obrub: Color32,
    pub lebdeca: Color32,
    pub k_zemlja: Color32,
}

impl Boje {
    #[must_use]
    pub const fn za_temu(tamna: bool) -> Self {
        if tamna { TAMNA } else { SVIJETLA }
    }
}

pub const SVIJETLA: Boje = Boje {
    prozor: Color32::from_rgb(245, 245, 241),
    ploca: Color32::from_rgb(251, 251, 248),
    ploca_2: Color32::from_rgb(241, 241, 236),
    izbornik: Color32::from_rgb(250, 250, 247),
    status: Color32::from_rgb(239, 239, 233),
    obrub: Color32::from_rgb(217, 217, 208),
    obrub_2: Color32::from_rgb(198, 198, 187),
    tekst: Color32::from_rgb(29, 35, 31),
    tekst_2: Color32::from_rgb(89, 97, 91),
    tekst_3: Color32::from_rgb(138, 145, 138),
    primarna: Color32::from_rgb(25, 118, 102),
    primarna_tamna: Color32::from_rgb(18, 92, 79),
    primarna_svijetla: Color32::from_rgb(225, 242, 238),
    primarna_tekst: Color32::from_rgb(255, 255, 255),
    alat_odabran: Color32::from_rgb(25, 118, 102),
    alat_odabran_tekst: Color32::from_rgb(255, 255, 255),
    plava: Color32::from_rgb(33, 105, 184),
    plava_svijetla: Color32::from_rgb(227, 238, 250),
    narancasta: Color32::from_rgb(209, 106, 31),
    narancasta_svijetla: Color32::from_rgb(251, 236, 222),
    zelena: Color32::from_rgb(47, 133, 72),
    zelena_svijetla: Color32::from_rgb(228, 244, 232),
    crvena: Color32::from_rgb(194, 59, 50),
    crvena_svijetla: Color32::from_rgb(250, 230, 228),
    upozorenje: Color32::from_rgb(180, 122, 20),
    upozorenje_svijetla: Color32::from_rgb(251, 242, 218),
    odabir: Color32::from_rgb(230, 243, 239),
    odabir_obrub: Color32::from_rgb(135, 194, 183),
    lebdeca: Color32::from_rgba_premultiplied(251, 251, 248, 244),
    k_zemlja: Color32::from_rgb(234, 231, 220),
};

pub const TAMNA: Boje = Boje {
    prozor: Color32::from_rgb(23, 27, 25),
    ploca: Color32::from_rgb(31, 36, 33),
    ploca_2: Color32::from_rgb(39, 45, 42),
    izbornik: Color32::from_rgb(27, 31, 29),
    status: Color32::from_rgb(25, 29, 27),
    obrub: Color32::from_rgb(54, 62, 58),
    obrub_2: Color32::from_rgb(72, 82, 76),
    tekst: Color32::from_rgb(234, 238, 235),
    tekst_2: Color32::from_rgb(166, 175, 169),
    tekst_3: Color32::from_rgb(115, 125, 118),
    primarna: Color32::from_rgb(47, 163, 143),
    primarna_tamna: Color32::from_rgb(33, 128, 111),
    primarna_svijetla: Color32::from_rgb(33, 62, 57),
    primarna_tekst: Color32::from_rgb(17, 24, 22),
    alat_odabran: Color32::from_rgb(11, 92, 86),
    alat_odabran_tekst: Color32::from_rgb(255, 255, 255),
    plava: Color32::from_rgb(90, 160, 232),
    plava_svijetla: Color32::from_rgb(31, 51, 74),
    narancasta: Color32::from_rgb(235, 142, 71),
    narancasta_svijetla: Color32::from_rgb(68, 46, 31),
    zelena: Color32::from_rgb(86, 184, 112),
    zelena_svijetla: Color32::from_rgb(31, 59, 39),
    crvena: Color32::from_rgb(229, 101, 92),
    crvena_svijetla: Color32::from_rgb(69, 34, 32),
    upozorenje: Color32::from_rgb(224, 170, 62),
    upozorenje_svijetla: Color32::from_rgb(64, 51, 24),
    odabir: Color32::from_rgb(34, 59, 54),
    odabir_obrub: Color32::from_rgb(62, 128, 116),
    lebdeca: Color32::from_rgba_premultiplied(31, 36, 33, 244),
    k_zemlja: Color32::from_rgb(30, 35, 31),
};

pub struct Razmaci;

impl Razmaci {
    pub const SIRINA_PROZORA: f32 = 1440.0;
    pub const VISINA_PROZORA: f32 = 900.0;
    pub const VISINA_IZBORNIKA: f32 = 40.0;
    pub const VISINA_STATUSA: f32 = 28.0;
    pub const SIRINA_PLOCE: f32 = 360.0;
    pub const RUB: f32 = 12.0;
    pub const ZAOBLJENJE_PLOCE: f32 = 14.0;
    pub const ZAOBLJENJE_GUMBA: f32 = 8.0;
    pub const ZAOBLJENJE_RETKA: f32 = 7.0;
    pub const VISINA_RETKA_STABLA: f32 = 42.0;
    pub const UVLAKA_STABLA: f32 = 16.0;
}

#[must_use]
pub fn obitelj_srednja() -> FontFamily {
    FontFamily::Name("srednji".into())
}

#[must_use]
pub fn obitelj_kurziv() -> FontFamily {
    FontFamily::Name("kurziv".into())
}

#[must_use]
pub fn font_obicni(velicina: f32) -> FontId {
    FontId::new(velicina, FontFamily::Proportional)
}

#[must_use]
pub fn font_srednji(velicina: f32) -> FontId {
    FontId::new(velicina, obitelj_srednja())
}

#[must_use]
pub fn font_kurziv(velicina: f32) -> FontId {
    FontId::new(velicina, obitelj_kurziv())
}

#[must_use]
pub fn fontovi() -> FontDefinitions {
    let mut def = FontDefinitions::empty();
    def.font_data.insert(
        "inter_obicni".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../resursi/fontovi/Inter-Regular.otf"
        ))),
    );
    def.font_data.insert(
        "inter_srednji".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../resursi/fontovi/Inter-Medium.otf"
        ))),
    );
    def.font_data.insert(
        "inter_kurziv".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../../resursi/fontovi/Inter-MediumItalic.otf"
        ))),
    );
    def.families
        .insert(FontFamily::Proportional, vec!["inter_obicni".into()]);
    def.families
        .insert(FontFamily::Monospace, vec!["inter_obicni".into()]);
    def.families
        .insert(obitelj_srednja(), vec!["inter_srednji".into()]);
    def.families
        .insert(obitelj_kurziv(), vec!["inter_kurziv".into()]);
    def
}

pub fn primijeni_stil(ctx: &egui::Context, boje: &Boje, tamna: bool) {
    let mut vizuali = if tamna {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    vizuali.panel_fill = boje.prozor;
    vizuali.window_fill = boje.ploca;
    vizuali.override_text_color = Some(boje.tekst);
    ctx.set_visuals(vizuali);
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn odabir_boja_i_ucitavanje_fontova() {
        assert_eq!(Boje::za_temu(false), SVIJETLA);
        assert_eq!(Boje::za_temu(true), TAMNA);
        assert_ne!(SVIJETLA.prozor, TAMNA.prozor);
        let def = fontovi();
        assert!(def.font_data.contains_key("inter_obicni"));
        assert!(def.font_data.contains_key("inter_srednji"));
        assert!(def.font_data.contains_key("inter_kurziv"));
        assert_eq!(font_obicni(14.0).size, 14.0);
        assert_eq!(font_srednji(13.0).size, 13.0);
        assert_eq!(font_kurziv(11.0).size, 11.0);
    }

    #[test]
    fn font_ima_znakove_precaca_za_mac() {
        let ctx = egui::Context::default();
        ctx.set_fonts(fontovi());
        ctx.run_ui(egui::RawInput::default(), |_| {})
            .textures_delta
            .clear();
        assert!(ctx.fonts_mut(|f| f.has_glyphs(&font_obicni(13.0), "⌘⇧")));
        assert!(ctx.fonts_mut(|f| f.has_glyphs(&font_srednji(13.0), "⌘⇧")));
    }
}
