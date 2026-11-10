//! Prozorčić koji se otvara čim je objekt nacrtan, da mu se odmah upiše naziv. Predloženi naziv,
//! npr. „Poligon 3”, sav je označen, pa ga upisivanje zamijeni.

use crate::poruke::KatalogPoruka;
use crate::sastavnice::{tekst_sredina, zaobljeni_okvir};
use crate::stanje::{NazivNovog, StanjePrikaza};
use crate::tema::{Boje, font_obicni, font_srednji};
use eframe::egui::{
    self, Key, Rect, RichText, Role, Sense, Stroke, TextEdit, WidgetInfo, pos2, vec2,
};
use slojevi::{IzmjenaObjekta, NaredbaObjekta, Uredjivac};

const SIRINA: f32 = 320.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Radnja {
    Nista,
    Preskoci,
    Spremi,
}

/// Crta prozorčić dok je otvoren. Enter ili „Spremi naziv” mijenja naziv, a Esc ili „Preskoči”
/// ostavlja predloženi.
pub fn crtaj_prozorcic_naziva(
    ui: &egui::Ui,
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
    boje: &Boje,
) {
    let Some(mut novi) = stanje.naziv_novog.clone() else {
        return;
    };
    let okvir = egui::Frame::new()
        .fill(boje.ploca)
        .stroke(Stroke::new(1.0, boje.obrub))
        .corner_radius(14)
        .inner_margin(20);
    let odgovor = egui::Modal::new(egui::Id::unique("prozorcic_naziv_objekta"))
        .frame(okvir)
        .show(ui.ctx(), |ui| {
            ui.set_width(SIRINA);
            sadrzaj(ui, &mut novi, katalog, boje)
        });
    let radnja = if odgovor.should_close() {
        Radnja::Preskoci
    } else {
        odgovor.inner
    };
    match radnja {
        Radnja::Nista => stanje.naziv_novog = Some(novi),
        Radnja::Preskoci => stanje.naziv_novog = None,
        Radnja::Spremi => {
            stanje.naziv_novog = None;
            spremi_naziv(uredjivac, &novi);
        }
    }
}

/// Prazan ili nepromijenjen naziv ne ostavlja korak u povijesti poništavanja.
fn spremi_naziv(uredjivac: &mut Uredjivac, novi: &NazivNovog) {
    let naziv = novi.naziv.trim();
    let isti = uredjivac
        .projekt()
        .objekt(novi.sloj, novi.objekt)
        .is_none_or(|o| o.naziv == naziv);
    if naziv.is_empty() || isti {
        return;
    }
    let _ = uredjivac.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: novi.sloj,
        objekt: novi.objekt,
        izmjena: IzmjenaObjekta::Naziv(naziv.to_owned()),
    });
}

fn sadrzaj(
    ui: &mut egui::Ui,
    novi: &mut NazivNovog,
    katalog: &KatalogPoruka,
    boje: &Boje,
) -> Radnja {
    let mut radnja = Radnja::Nista;
    ui.label(
        RichText::new(katalog.tekst("naziv-objekta-naslov"))
            .font(font_srednji(17.0))
            .color(boje.tekst),
    );
    ui.add_space(10.0);
    let naslov_polja = ui.label(
        RichText::new(katalog.tekst("naziv-objekta-polje"))
            .font(font_obicni(12.0))
            .color(boje.tekst_2),
    );
    let id_polja = ui.make_persistent_id("naziv_objekta_polje");
    // Enter se gleda prije polja: polje ga samo preuzme i pusti fokus.
    if ui.memory(|m| m.has_focus(id_polja)) && ui.input(|i| i.key_pressed(Key::Enter)) {
        radnja = Radnja::Spremi;
    }
    let polje = ui
        .add(
            TextEdit::singleline(&mut novi.naziv)
                .id(id_polja)
                .desired_width(f32::INFINITY),
        )
        .labelled_by(naslov_polja.id);
    if std::mem::take(&mut novi.oznaci) {
        polje.request_focus();
        oznaci_sve(ui.ctx(), id_polja, &novi.naziv);
    }
    ui.add_space(8.0);
    ui.label(
        RichText::new(katalog.tekst("naziv-objekta-napomena"))
            .font(font_obicni(12.0))
            .color(boje.tekst_2),
    );
    ui.add_space(16.0);
    gumbi(ui, katalog, boje).unwrap_or(radnja)
}

fn oznaci_sve(ctx: &egui::Context, id: egui::Id, tekst: &str) {
    let mut stanje = egui::text_edit::TextEditState::load(ctx, id).unwrap_or_default();
    stanje
        .cursor
        .set_char_range(Some(egui::text_selection::CCursorRange::select_all_str(
            tekst,
        )));
    stanje.store(ctx, id);
}

/// „Preskoči” i „Spremi naziv”, desno dolje.
fn gumbi(ui: &mut egui::Ui, katalog: &KatalogPoruka, boje: &Boje) -> Option<Radnja> {
    let (red, _) = ui.allocate_exact_size(vec2(SIRINA, 34.0), Sense::hover());
    let spremi = Rect::from_min_size(pos2(red.max.x - 124.0, red.min.y), vec2(124.0, 34.0));
    let preskoci = Rect::from_min_size(pos2(spremi.min.x - 96.0, red.min.y), vec2(88.0, 34.0));
    let tekst_p = katalog.tekst("gumb-preskoci");
    let p = ui.interact(
        preskoci,
        ui.make_persistent_id("naziv_objekta_preskoci"),
        Sense::click(),
    );
    p.widget_info(|| WidgetInfo::labeled(Role::Button, true, &tekst_p));
    if p.hovered() {
        zaobljeni_okvir(ui, preskoci, boje.ploca_2, None, 17.0);
    }
    tekst_sredina(
        ui,
        preskoci.center(),
        &tekst_p,
        font_srednji(13.0),
        boje.primarna_tamna,
    );
    let tekst_s = katalog.tekst("gumb-spremi-naziv");
    let s = ui.interact(
        spremi,
        ui.make_persistent_id("naziv_objekta_spremi"),
        Sense::click(),
    );
    s.widget_info(|| WidgetInfo::labeled(Role::Button, true, &tekst_s));
    zaobljeni_okvir(ui, spremi, boje.alat_odabran, None, 17.0);
    tekst_sredina(
        ui,
        spremi.center(),
        &tekst_s,
        font_srednji(13.0),
        boje.alat_odabran_tekst,
    );
    if s.clicked() {
        Some(Radnja::Spremi)
    } else if p.clicked() {
        Some(Radnja::Preskoci)
    } else {
        None
    }
}
