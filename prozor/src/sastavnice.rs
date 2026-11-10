use crate::ikone::{SkladisteIkona, okvir_ikone_u_sredini};
use crate::tema::{Boje, Razmaci, font_obicni, font_srednji};
use eframe::egui::{
    self, Align2, Color32, FontId, Pos2, Rect, Response, Role, Sense, Stroke, StrokeKind,
    WidgetInfo, pos2, vec2,
};

pub fn zaobljeni_okvir(
    ui: &egui::Ui,
    okvir: Rect,
    ispuna: Color32,
    obrub: Option<Color32>,
    polumjer: f32,
) {
    ui.painter().rect_filled(okvir, polumjer, ispuna);
    if let Some(boja_obruba) = obrub {
        ui.painter().rect_stroke(
            okvir,
            polumjer,
            Stroke::new(1.0, boja_obruba),
            StrokeKind::Inside,
        );
    }
}

pub fn tekst_lijevo(ui: &egui::Ui, tocka: Pos2, tekst: &str, font: FontId, boja: Color32) {
    ui.painter()
        .text(tocka, Align2::LEFT_CENTER, tekst, font, boja);
}

pub fn tekst_sredina(ui: &egui::Ui, tocka: Pos2, tekst: &str, font: FontId, boja: Color32) {
    ui.painter()
        .text(tocka, Align2::CENTER_CENTER, tekst, font, boja);
}

pub fn tekst_desno(ui: &egui::Ui, tocka: Pos2, tekst: &str, font: FontId, boja: Color32) {
    ui.painter()
        .text(tocka, Align2::RIGHT_CENTER, tekst, font, boja);
}

pub struct OpisGumbaIkone<'a> {
    pub okvir: Rect,
    pub ikona: &'a str,
    pub stanje: &'a str,
    pub crtez: u32,
    pub obrubljen: bool,
    pub aktivno: bool,
    pub labela: &'a str,
}

pub fn gumb_ikona(
    ui: &mut egui::Ui,
    ikone: &mut SkladisteIkona,
    opis: &OpisGumbaIkone<'_>,
    tamna: bool,
    boje: &Boje,
) -> Response {
    if opis.aktivno {
        zaobljeni_okvir(
            ui,
            opis.okvir,
            boje.primarna_svijetla,
            Some(boje.odabir_obrub),
            Razmaci::ZAOBLJENJE_GUMBA,
        );
    } else if opis.obrubljen {
        zaobljeni_okvir(
            ui,
            opis.okvir,
            boje.ploca_2,
            Some(boje.obrub),
            Razmaci::ZAOBLJENJE_GUMBA,
        );
    }
    let okvir_ikone = okvir_ikone_u_sredini(opis.okvir.center(), opis.crtez);
    ikone.crtaj(ui, okvir_ikone, opis.ikona, tamna, opis.stanje, opis.crtez);
    interakcija_gumba(ui, opis.okvir, opis.labela)
}

pub fn gumb_obrub(
    ui: &mut egui::Ui,
    ikone: &mut SkladisteIkona,
    okvir: Rect,
    ikona: &str,
    tekst: &str,
    tamna: bool,
    boje: &Boje,
) -> Response {
    zaobljeni_okvir(
        ui,
        okvir,
        boje.ploca_2,
        Some(boje.obrub),
        Razmaci::ZAOBLJENJE_GUMBA,
    );
    let srediste_ikone = pos2(okvir.min.x + 17.0, okvir.center().y);
    ikone.crtaj(
        ui,
        okvir_ikone_u_sredini(srediste_ikone, 18),
        ikona,
        tamna,
        "obicno",
        18,
    );
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 30.0, okvir.center().y),
        tekst,
        font_srednji(12.0),
        boje.tekst,
    );
    interakcija_gumba(ui, okvir, tekst)
}

pub struct OpisGumbaAlata<'a> {
    pub okvir: Rect,
    pub ikona: &'a str,
    pub tekst: &'a str,
    pub odabran: bool,
}

pub fn gumb_alat(
    ui: &mut egui::Ui,
    ikone: &mut SkladisteIkona,
    opis: &OpisGumbaAlata<'_>,
    tamna: bool,
    boje: &Boje,
) -> Response {
    if opis.odabran {
        zaobljeni_okvir(ui, opis.okvir, boje.alat_odabran, None, 10.0);
    }
    let stanje = if opis.odabran {
        "odabrani_alat"
    } else {
        "obicno"
    };
    let srediste_ikone = pos2(opis.okvir.center().x, opis.okvir.min.y + 16.0);
    ikone.crtaj(
        ui,
        okvir_ikone_u_sredini(srediste_ikone, 22),
        opis.ikona,
        tamna,
        stanje,
        22,
    );
    let boja_teksta = if opis.odabran {
        boje.alat_odabran_tekst
    } else {
        boje.tekst_2
    };
    tekst_sredina(
        ui,
        pos2(opis.okvir.center().x, opis.okvir.min.y + 37.0),
        opis.tekst,
        font_srednji(11.0),
        boja_teksta,
    );
    let id = ui.make_persistent_id(("gumb_alat", opis.tekst));
    let odgovor = ui.interact(opis.okvir, id, Sense::click());
    odgovor.widget_info(|| {
        WidgetInfo::selected(Role::Button, ui.is_enabled(), opis.odabran, opis.tekst)
    });
    odgovor
}

pub fn cip(
    ui: &mut egui::Ui,
    ikone: &mut SkladisteIkona,
    okvir: Rect,
    tekst: &str,
    odabran: bool,
    tamna: bool,
    boje: &Boje,
) -> Response {
    let ispuna = if odabran {
        boje.primarna_svijetla
    } else {
        boje.ploca_2
    };
    let obrub = if odabran {
        boje.odabir_obrub
    } else {
        boje.obrub
    };
    zaobljeni_okvir(ui, okvir, ispuna, Some(obrub), Razmaci::ZAOBLJENJE_RETKA);
    if odabran {
        let srediste_ikone = pos2(okvir.min.x + 15.0, okvir.center().y);
        ikone.crtaj(
            ui,
            okvir_ikone_u_sredini(srediste_ikone, 16),
            "kvacica",
            tamna,
            "potvrda",
            16,
        );
        let boja_teksta = if tamna {
            boje.tekst
        } else {
            boje.primarna_tamna
        };
        tekst_lijevo(
            ui,
            pos2(okvir.min.x + 26.0, okvir.center().y),
            tekst,
            font_srednji(12.0),
            boja_teksta,
        );
    } else {
        tekst_sredina(ui, okvir.center(), tekst, font_srednji(12.0), boje.tekst_2);
    }
    let id = ui.make_persistent_id(("cip", tekst));
    let odgovor = ui.interact(okvir, id, Sense::click());
    odgovor.widget_info(|| WidgetInfo::selected(Role::Button, ui.is_enabled(), odabran, tekst));
    odgovor
}

pub fn prekidac(
    ui: &mut egui::Ui,
    okvir: Rect,
    ukljuceno: bool,
    boje: &Boje,
    labela: &str,
) -> Response {
    let ispuna = if ukljuceno {
        boje.primarna
    } else {
        boje.obrub_2
    };
    ui.painter().rect_filled(okvir, 10.0, ispuna);
    let krug_x = if ukljuceno {
        okvir.max.x - 10.0
    } else {
        okvir.min.x + 10.0
    };
    ui.painter()
        .circle_filled(pos2(krug_x, okvir.center().y), 7.5, Color32::WHITE);
    let id = ui.make_persistent_id(("prekidac", labela, okvir.min.y.to_bits()));
    let odgovor = ui.interact(okvir, id, Sense::click());
    odgovor
        .widget_info(|| WidgetInfo::selected(Role::CheckBox, ui.is_enabled(), ukljuceno, labela));
    odgovor
}

pub struct OpisRetkaStabla<'a> {
    pub id_kljuc: &'a str,
    pub okvir: Rect,
    pub uvlaka: usize,
    pub strelica: Option<bool>,
    pub rezerviraj_strelicu: bool,
    pub ikona: &'a str,
    pub stanje_ikone: &'a str,
    pub naslov: &'a str,
    pub naglasen_naslov: bool,
    pub podnaslov: &'a str,
    pub oznaka_formata: Option<&'a str>,
    pub zakljucano: bool,
    pub ukljuceno: Option<bool>,
    pub odabrano: bool,
    pub labela_strelice: &'a str,
    pub labela_vidljivosti: &'a str,
    /// `Some(oznaceno)` dok se naziv mijenja: naslov se crta u polju, sav označen ili s kursorom.
    pub uredjivanje: Option<bool>,
    /// Stavke izbornika na desni klik; prazno znači bez izbornika.
    pub stavke_izbornika: &'a [&'a str],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OdgovorRetkaStabla {
    pub kliknut_redak: bool,
    pub dvokliknut_redak: bool,
    pub desni_klik: bool,
    pub kliknuta_strelica: bool,
    pub kliknut_prekidac: bool,
    /// Redni broj stavke koju je korisnik odabrao u izborniku retka.
    pub stavka_izbornika: Option<usize>,
}

pub fn redak_stabla(
    ui: &mut egui::Ui,
    ikone: &mut SkladisteIkona,
    opis: &OpisRetkaStabla<'_>,
    tamna: bool,
    boje: &Boje,
) -> OdgovorRetkaStabla {
    if opis.odabrano {
        zaobljeni_okvir(
            ui,
            opis.okvir,
            boje.odabir,
            Some(boje.odabir_obrub),
            Razmaci::ZAOBLJENJE_RETKA,
        );
    }

    let sredina_y = opis.okvir.center().y;
    let mut x = opis.okvir.min.x + 8.0 + opis.uvlaka as f32 * Razmaci::UVLAKA_STABLA;
    let kliknuta_strelica = crtaj_strelicu_retka(ui, ikone, opis, &mut x, sredina_y, tamna);

    ikone.crtaj(
        ui,
        okvir_ikone_u_sredini(pos2(x + 9.0, sredina_y), 18),
        opis.ikona,
        tamna,
        opis.stanje_ikone,
        18,
    );
    x += 24.0;

    let (desna_granica, kliknut_prekidac) =
        crtaj_desnu_stranu_retka(ui, ikone, opis, sredina_y, tamna, boje);
    crtaj_tekst_retka(ui, opis, x, sredina_y, desna_granica, boje);

    let okvir_tijela = Rect::from_min_max(
        pos2(x - 24.0, opis.okvir.min.y),
        pos2(desna_granica.max(x), opis.okvir.max.y),
    );
    let id_retka = ui.make_persistent_id(("redak_stabla", opis.id_kljuc));
    let odgovor_tijela = ui.interact(okvir_tijela, id_retka, Sense::click());
    odgovor_tijela.widget_info(|| {
        WidgetInfo::selected(Role::Button, ui.is_enabled(), opis.odabrano, opis.naslov)
    });

    OdgovorRetkaStabla {
        kliknut_redak: odgovor_tijela.clicked(),
        dvokliknut_redak: odgovor_tijela.double_clicked(),
        desni_klik: odgovor_tijela.secondary_clicked(),
        kliknuta_strelica,
        kliknut_prekidac,
        stavka_izbornika: izbornik_retka(&odgovor_tijela, opis.stavke_izbornika),
    }
}

fn crtaj_tekst_retka(
    ui: &egui::Ui,
    opis: &OpisRetkaStabla<'_>,
    x: f32,
    sredina_y: f32,
    desna_granica: f32,
    boje: &Boje,
) {
    let font_naslova = if opis.naglasen_naslov {
        font_srednji(13.0)
    } else {
        font_obicni(13.0)
    };
    let y_naslova = if opis.podnaslov.is_empty() {
        sredina_y
    } else {
        tekst_lijevo(
            ui,
            pos2(x, sredina_y + 8.0),
            opis.podnaslov,
            font_obicni(11.0),
            boje.tekst_3,
        );
        sredina_y - 8.0
    };
    if let Some(oznaceno) = opis.uredjivanje {
        let polje = Rect::from_min_max(
            pos2(x - 4.0, y_naslova - 10.0),
            pos2(desna_granica.max(x + 40.0) - 4.0, y_naslova + 10.0),
        );
        polje_naziva(ui, polje, opis.naslov, &font_naslova, oznaceno, boje);
    }
    tekst_lijevo(
        ui,
        pos2(x, y_naslova),
        opis.naslov,
        font_naslova,
        boje.tekst,
    );
}

/// Polje oko naziva koji se mijenja: označen naziv dobije pozadinu, a inače se iza njega vidi kursor.
fn polje_naziva(
    ui: &egui::Ui,
    polje: Rect,
    naslov: &str,
    font: &FontId,
    oznaceno: bool,
    boje: &Boje,
) {
    let slikar = ui.painter();
    slikar.rect_filled(polje, 4.0, boje.ploca);
    slikar.rect_stroke(
        polje,
        4.0,
        Stroke::new(1.5, boje.primarna),
        StrokeKind::Inside,
    );
    let sirina = slikar
        .layout_no_wrap(naslov.to_owned(), font.clone(), boje.tekst)
        .size()
        .x;
    let lijevo = polje.min.x + 4.0;
    if oznaceno {
        let oznaka = Rect::from_min_max(
            pos2(lijevo - 1.0, polje.min.y + 3.0),
            pos2(lijevo + sirina + 1.0, polje.max.y - 3.0),
        );
        slikar.rect_filled(oznaka, 2.0, boje.primarna_svijetla);
    } else {
        let kursor_x = lijevo + sirina + 1.0;
        slikar.line_segment(
            [
                pos2(kursor_x, polje.min.y + 4.0),
                pos2(kursor_x, polje.max.y - 4.0),
            ],
            Stroke::new(1.0, boje.tekst),
        );
    }
}

/// Izbornik na desni klik. Vraća redni broj odabrane stavke.
fn izbornik_retka(odgovor: &Response, stavke: &[&str]) -> Option<usize> {
    if stavke.is_empty() {
        return None;
    }
    let mut odabrana = None;
    let _ = odgovor.context_menu(|ui| {
        for (redni, stavka) in stavke.iter().enumerate() {
            if ui.button(*stavka).clicked() {
                odabrana = Some(redni);
                ui.close();
            }
        }
    });
    odabrana
}

fn crtaj_strelicu_retka(
    ui: &mut egui::Ui,
    ikone: &mut SkladisteIkona,
    opis: &OpisRetkaStabla<'_>,
    x: &mut f32,
    sredina_y: f32,
    tamna: bool,
) -> bool {
    if let Some(otvoreno) = opis.strelica {
        let naziv_ikone = if otvoreno { "dolje" } else { "desno" };
        let okvir_strelice = okvir_ikone_u_sredini(pos2(*x + 9.0, sredina_y), 18);
        ikone.crtaj(ui, okvir_strelice, naziv_ikone, tamna, "obicno", 18);
        *x += 20.0;
        let labela = format!("{}: {}", opis.labela_strelice, opis.naslov);
        let id = ui.make_persistent_id(("strelica_stabla", opis.id_kljuc));
        let odgovor = ui.interact(okvir_strelice.expand(2.0), id, Sense::click());
        odgovor.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), labela.clone()));
        odgovor.clicked()
    } else {
        if opis.rezerviraj_strelicu {
            *x += 18.0;
        }
        false
    }
}

fn crtaj_desnu_stranu_retka(
    ui: &mut egui::Ui,
    ikone: &mut SkladisteIkona,
    opis: &OpisRetkaStabla<'_>,
    sredina_y: f32,
    tamna: bool,
    boje: &Boje,
) -> (f32, bool) {
    let mut desno = opis.okvir.max.x - 6.0;
    let mut kliknut_prekidac = false;
    if let Some(vidljivo) = opis.ukljuceno {
        let okvir_prekidaca =
            Rect::from_min_size(pos2(desno - 36.0, sredina_y - 10.0), vec2(36.0, 20.0));
        let labela = format!("{}: {}", opis.labela_vidljivosti, opis.naslov);
        let odg = prekidac(ui, okvir_prekidaca, vidljivo, boje, &labela);
        kliknut_prekidac = odg.clicked();
        desno -= 44.0;
    }
    if opis.zakljucano {
        let okvir_brave = okvir_ikone_u_sredini(pos2(desno - 7.0, sredina_y), 14);
        ikone.crtaj(ui, okvir_brave, "brava", tamna, "obicno", 14);
        desno -= 20.0;
    }
    if let Some(oznaka) = opis.oznaka_formata {
        let sirina = (oznaka.len() as f32 * 6.5 + 10.0).max(28.0);
        let okvir_oznake =
            Rect::from_min_size(pos2(desno - sirina, sredina_y - 8.5), vec2(sirina, 17.0));
        zaobljeni_okvir(ui, okvir_oznake, boje.ploca_2, Some(boje.obrub), 5.0);
        tekst_sredina(
            ui,
            okvir_oznake.center(),
            oznaka,
            font_srednji(10.0),
            boje.tekst_2,
        );
        desno -= sirina + 6.0;
    }
    (desno, kliknut_prekidac)
}

fn interakcija_gumba(ui: &mut egui::Ui, okvir: Rect, labela: &str) -> Response {
    let id = ui.make_persistent_id(("gumb", labela, okvir.min.x.to_bits(), okvir.min.y.to_bits()));
    let odgovor = ui.interact(okvir, id, Sense::click());
    odgovor.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), labela));
    odgovor
}

#[cfg(test)]
mod testovi {
    use super::*;
    use crate::tema::{SVIJETLA, TAMNA, fontovi};
    use egui_kittest::{Harness, kittest::Queryable};

    #[test]
    fn gumb_ikona_reagira_na_klik_misa() {
        let mut ikone = SkladisteIkona::novo();
        let mut harness = Harness::builder()
            .with_size(vec2(120.0, 80.0))
            .with_context_setup(|ctx| ctx.set_fonts(fontovi()))
            .build_ui_state(
                move |ui, kliknuto: &mut bool| {
                    let opis = OpisGumbaIkone {
                        okvir: Rect::from_min_size(pos2(10.0, 10.0), vec2(32.0, 32.0)),
                        ikona: "ponisti",
                        stanje: "obicno",
                        crtez: 18,
                        obrubljen: true,
                        aktivno: false,
                        labela: "Poništi",
                    };
                    if gumb_ikona(ui, &mut ikone, &opis, false, &SVIJETLA).clicked() {
                        *kliknuto = true;
                    }
                },
                false,
            );

        harness.get_by_label("Poništi").click();
        harness.run();
        assert!(*harness.state());
    }

    #[test]
    fn cip_mijenja_odabir_na_klik_misa() {
        let mut ikone = SkladisteIkona::novo();
        let mut harness = Harness::builder()
            .with_size(vec2(200.0, 80.0))
            .with_context_setup(|ctx| ctx.set_fonts(fontovi()))
            .build_ui_state(
                move |ui, odabran: &mut bool| {
                    let okvir = Rect::from_min_size(pos2(10.0, 10.0), vec2(100.0, 26.0));
                    if cip(ui, &mut ikone, okvir, "Karta", *odabran, false, &SVIJETLA).clicked() {
                        *odabran = !*odabran;
                    }
                },
                false,
            );

        harness.get_by_label("Karta").click();
        harness.run();
        assert!(*harness.state());
    }

    #[test]
    fn prekidac_mijenja_stanje_na_klik_misa() {
        let mut harness = Harness::builder()
            .with_size(vec2(120.0, 80.0))
            .with_context_setup(|ctx| ctx.set_fonts(fontovi()))
            .build_ui_state(
                |ui, ukljuceno: &mut bool| {
                    let okvir = Rect::from_min_size(pos2(10.0, 10.0), vec2(36.0, 20.0));
                    if prekidac(ui, okvir, *ukljuceno, &TAMNA, "Vidljivost").clicked() {
                        *ukljuceno = !*ukljuceno;
                    }
                },
                true,
            );

        harness.get_by_label("Vidljivost").click();
        harness.run();
        assert!(!*harness.state());
    }

    #[test]
    fn redak_stabla_razlikuje_klik_strelice_retka_i_prekidaca() {
        let mut ikone = SkladisteIkona::novo();
        let mut harness = Harness::builder()
            .with_size(vec2(360.0, 80.0))
            .with_context_setup(|ctx| ctx.set_fonts(fontovi()))
            .build_ui_state(
                move |ui, zadnji: &mut OdgovorRetkaStabla| {
                    let opis = OpisRetkaStabla {
                        id_kljuc: "sloj_1",
                        okvir: Rect::from_min_size(pos2(10.0, 10.0), vec2(336.0, 34.0)),
                        uvlaka: 1,
                        strelica: Some(true),
                        rezerviraj_strelicu: false,
                        ikona: "poligon",
                        stanje_ikone: "narancasti_poligon",
                        naslov: "Parcele",
                        naglasen_naslov: true,
                        podnaslov: "12 objekata",
                        oznaka_formata: Some("KML"),
                        zakljucano: true,
                        ukljuceno: Some(true),
                        odabrano: true,
                        labela_strelice: "Proširi ili skupi",
                        labela_vidljivosti: "Vidljivost",
                        uredjivanje: None,
                        stavke_izbornika: &[],
                    };
                    let odg = redak_stabla(ui, &mut ikone, &opis, false, &SVIJETLA);
                    if odg.kliknuta_strelica {
                        zadnji.kliknuta_strelica = true;
                    }
                    if odg.kliknut_redak {
                        zadnji.kliknut_redak = true;
                    }
                    if odg.kliknut_prekidac {
                        zadnji.kliknut_prekidac = true;
                    }
                },
                OdgovorRetkaStabla::default(),
            );

        harness.get_by_label("Proširi ili skupi: Parcele").click();
        harness.run();
        assert!(harness.state().kliknuta_strelica);
        assert!(!harness.state().kliknut_redak);

        harness.get_by_label("Parcele").click();
        harness.run();
        assert!(harness.state().kliknut_redak);
        assert!(!harness.state().kliknut_prekidac);

        harness.get_by_label("Vidljivost: Parcele").click();
        harness.run();
        assert!(harness.state().kliknut_prekidac);
    }
}
