//! Prozorčić Novi sloj po zaslonu F: naziv, vrsta i mapa. Polja i koordinatni sustav dolaze kasnije.

use crate::ikone::SkladisteIkona;
use crate::ploca::napravi_novi_sloj;
use crate::poruke::KatalogPoruka;
use crate::sastavnice::{cip, tekst_sredina, zaobljeni_okvir};
use crate::stanje::StanjePrikaza;
use crate::tema::{Boje, font_obicni, font_srednji};
use eframe::egui::{
    self, Key, Rect, RichText, Role, Sense, Stroke, TextEdit, WidgetInfo, pos2, vec2,
};
use slojevi::{IdMape, Projekt, Stavka, Uredjivac, VrstaGeometrije};

const SIRINA: f32 = 340.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Radnja {
    Nista,
    Odustani,
    Napravi,
}

/// Mape redom kao u stablu, s dubinom za uvlaku u izborniku.
#[must_use]
pub fn mape_stabla(projekt: &Projekt) -> Vec<(IdMape, usize)> {
    fn obidji(
        projekt: &Projekt,
        stavke: &[Stavka],
        dubina: usize,
        izlaz: &mut Vec<(IdMape, usize)>,
    ) {
        for stavka in stavke {
            if let Stavka::Mapa(id) = stavka
                && let Some(mapa) = projekt.mapa(*id)
            {
                izlaz.push((*id, dubina));
                obidji(projekt, &mapa.djeca, dubina + 1, izlaz);
            }
        }
    }
    let mut izlaz = Vec::new();
    obidji(projekt, projekt.korijen(), 0, &mut izlaz);
    izlaz
}

fn naziv_mape(projekt: &Projekt, katalog: &KatalogPoruka, mapa: Option<IdMape>) -> String {
    mapa.and_then(|m| projekt.mapa(m))
        .map_or_else(|| katalog.tekst("novi-sloj-bez-mape"), |m| m.naziv.clone())
}

/// Crta prozorčić dok je otvoren. Klik na „Napravi sloj” ili Enter u nazivu stvara sloj.
pub fn crtaj_prozorcic_novog_sloja(
    ui: &egui::Ui,
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
    ikone: &mut SkladisteIkona,
    boje: &Boje,
) {
    let Some(mut novi) = stanje.novi_sloj.clone() else {
        return;
    };
    let tamna = stanje.tema_tamna;
    let okvir = egui::Frame::new()
        .fill(boje.ploca)
        .stroke(Stroke::new(1.0, boje.obrub))
        .corner_radius(14)
        .inner_margin(20);
    let odgovor = egui::Modal::new(egui::Id::unique("prozorcic_novi_sloj"))
        .frame(okvir)
        .show(ui.ctx(), |ui| {
            ui.set_width(SIRINA);
            sadrzaj(
                ui,
                uredjivac.projekt(),
                &mut novi,
                katalog,
                ikone,
                tamna,
                boje,
            )
        });
    let radnja = if odgovor.should_close() {
        Radnja::Odustani
    } else {
        odgovor.inner
    };
    match radnja {
        Radnja::Odustani => stanje.novi_sloj = None,
        Radnja::Napravi => {
            stanje.novi_sloj = Some(novi);
            let _ = napravi_novi_sloj(uredjivac, stanje);
        }
        Radnja::Nista => stanje.novi_sloj = Some(novi),
    }
}

fn sadrzaj(
    ui: &mut egui::Ui,
    projekt: &Projekt,
    novi: &mut crate::stanje::NoviSloj,
    katalog: &KatalogPoruka,
    ikone: &mut SkladisteIkona,
    tamna: bool,
    boje: &Boje,
) -> Radnja {
    let mut radnja = Radnja::Nista;
    ui.label(
        RichText::new(katalog.tekst("novi-sloj-naslov"))
            .font(font_srednji(17.0))
            .color(boje.tekst),
    );
    ui.add_space(10.0);
    let naslov_polja = ui.label(oznaka(katalog, "novi-sloj-polje-naziv", boje));
    let id_polja = ui.make_persistent_id("novi_sloj_naziv");
    // Enter se gleda prije polja: polje ga samo preuzme i pusti fokus.
    if ui.memory(|m| m.has_focus(id_polja)) && ui.input(|i| i.key_pressed(Key::Enter)) {
        radnja = Radnja::Napravi;
    }
    let polje = ui
        .add(
            TextEdit::singleline(&mut novi.naziv)
                .id(id_polja)
                .desired_width(f32::INFINITY),
        )
        .labelled_by(naslov_polja.id);
    if ui.ctx().memory(|m| m.focused().is_none()) {
        polje.request_focus();
    }
    if novi.prazan_naziv {
        ui.label(
            RichText::new(katalog.tekst("novi-sloj-prazan-naziv"))
                .font(font_obicni(12.0))
                .color(boje.crvena),
        );
    }
    ui.add_space(10.0);
    ui.label(oznaka(katalog, "novi-sloj-vrsta", boje));
    izbor_vrste(ui, novi, katalog, ikone, tamna, boje);
    ui.add_space(10.0);
    let naslov_mape = ui.label(oznaka(katalog, "novi-sloj-mapa", boje));
    egui::ComboBox::from_id_salt("novi_sloj_mapa")
        .width(SIRINA)
        .selected_text(naziv_mape(projekt, katalog, novi.mapa))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut novi.mapa, None, katalog.tekst("novi-sloj-bez-mape"));
            for (id, dubina) in mape_stabla(projekt) {
                let naziv = naziv_mape(projekt, katalog, Some(id));
                ui.selectable_value(
                    &mut novi.mapa,
                    Some(id),
                    format!("{}{naziv}", "    ".repeat(dubina)),
                );
            }
        })
        .response
        .labelled_by(naslov_mape.id);
    ui.add_space(10.0);
    ui.label(
        RichText::new(katalog.tekst("novi-sloj-napomena"))
            .font(font_obicni(12.0))
            .color(boje.tekst_2),
    );
    ui.add_space(16.0);
    gumbi(ui, katalog, boje).unwrap_or(radnja)
}

fn oznaka(katalog: &KatalogPoruka, kljuc: &str, boje: &Boje) -> RichText {
    RichText::new(katalog.tekst(kljuc))
        .font(font_obicni(12.0))
        .color(boje.tekst_2)
}

fn izbor_vrste(
    ui: &mut egui::Ui,
    novi: &mut crate::stanje::NoviSloj,
    katalog: &KatalogPoruka,
    ikone: &mut SkladisteIkona,
    tamna: bool,
    boje: &Boje,
) {
    let (red, _) = ui.allocate_exact_size(vec2(SIRINA, 30.0), Sense::hover());
    let mut x = red.min.x;
    for (vrsta, kljuc, sirina) in [
        (VrstaGeometrije::Tocka, "novi-sloj-tocke", 84.0),
        (VrstaGeometrije::Linija, "novi-sloj-linije", 84.0),
        (VrstaGeometrije::Poligon, "novi-sloj-poligone", 104.0),
    ] {
        let okvir = Rect::from_min_size(pos2(x, red.min.y + 2.0), vec2(sirina, 26.0));
        if cip(
            ui,
            ikone,
            okvir,
            &katalog.tekst(kljuc),
            novi.vrsta == vrsta,
            tamna,
            boje,
        )
        .clicked()
        {
            novi.vrsta = vrsta;
        }
        x += sirina + 8.0;
    }
}

/// „Odustani” i „Napravi sloj”, desno dolje.
fn gumbi(ui: &mut egui::Ui, katalog: &KatalogPoruka, boje: &Boje) -> Option<Radnja> {
    let (red, _) = ui.allocate_exact_size(vec2(SIRINA, 34.0), Sense::hover());
    let napravi = Rect::from_min_size(pos2(red.max.x - 120.0, red.min.y), vec2(120.0, 34.0));
    let odustani = Rect::from_min_size(pos2(napravi.min.x - 96.0, red.min.y), vec2(88.0, 34.0));
    let tekst_o = katalog.tekst("gumb-odustani");
    let o = ui.interact(
        odustani,
        ui.make_persistent_id("novi_sloj_odustani"),
        Sense::click(),
    );
    o.widget_info(|| WidgetInfo::labeled(Role::Button, true, &tekst_o));
    if o.hovered() {
        zaobljeni_okvir(ui, odustani, boje.ploca_2, None, 17.0);
    }
    tekst_sredina(
        ui,
        odustani.center(),
        &tekst_o,
        font_srednji(13.0),
        boje.primarna_tamna,
    );
    let tekst_n = katalog.tekst("gumb-napravi-sloj");
    let n = ui.interact(
        napravi,
        ui.make_persistent_id("novi_sloj_napravi"),
        Sense::click(),
    );
    n.widget_info(|| WidgetInfo::labeled(Role::Button, true, &tekst_n));
    zaobljeni_okvir(ui, napravi, boje.alat_odabran, None, 17.0);
    tekst_sredina(
        ui,
        napravi.center(),
        &tekst_n,
        font_srednji(13.0),
        boje.alat_odabran_tekst,
    );
    if n.clicked() {
        Some(Radnja::Napravi)
    } else if o.clicked() {
        Some(Radnja::Odustani)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "novi_sloj_testovi.rs"]
mod testovi;
