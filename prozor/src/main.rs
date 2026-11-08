#![forbid(unsafe_code)]

use eframe::egui;
use prozor::aplikacija::{Aplikacija, rasclani_argumente};
use prozor::poruke::KatalogPoruka;
use prozor::tema::Razmaci;

fn main() -> eframe::Result {
    let mut postavke = rasclani_argumente(std::env::args().skip(1));
    postavke.podloga_s_mreze = true;
    let naslov = KatalogPoruka::hrvatski().tekst("naslov-aplikacije");
    let opcije = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA])
            .with_title(naslov.clone()),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        &naslov,
        opcije,
        Box::new(move |cc| {
            let kartica = cc
                .wgpu_render_state
                .as_ref()
                .ok_or("Karta treba grafičku karticu (wgpu).")?;
            // Bez više uzoraka, jer ih walkers ne koristi (docs/plan.md, rezultat paketa karta).
            karta::ugradi(kartica, 1);
            Aplikacija::postavi_kontekst(&cc.egui_ctx);
            Ok(Box::new(Aplikacija::nova(postavke)))
        }),
    )
}
