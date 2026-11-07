#![forbid(unsafe_code)]

use eframe::egui;
use prozor::aplikacija::{Aplikacija, rasclani_argumente};
use prozor::poruke::KatalogPoruka;
use prozor::tema::Razmaci;

fn main() -> eframe::Result {
    let postavke = rasclani_argumente(std::env::args().skip(1));
    let naslov = KatalogPoruka::hrvatski().tekst("naslov-aplikacije");
    let opcije = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA])
            .with_title(naslov.clone()),
        ..Default::default()
    };
    eframe::run_native(
        &naslov,
        opcije,
        Box::new(move |cc| {
            Aplikacija::postavi_kontekst(&cc.egui_ctx);
            Ok(Box::new(Aplikacija::nova(postavke)))
        }),
    )
}
