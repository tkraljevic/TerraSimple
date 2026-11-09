use eframe::egui;
use prozor::aplikacija::{Aplikacija, PostavkeAplikacije, rasclani_argumente};
use prozor::tema::Razmaci;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};
use std::time::{Duration, Instant};

const IKONA: &[u8] = include_bytes!("../../resursi/ikone/terrasimple-64.png");

pub fn pokreni(argumenti: Vec<String>) -> Result<(), String> {
    let (provjera, kvar) = zastavice(&argumenti);
    zastita_od_zastoja(provjera);
    let stanje = Arc::new(AtomicU8::new(0));
    prikazi(
        postavke(&argumenti, provjera),
        provjera,
        kvar,
        stanje.clone(),
    )?;
    provjeri_ishod(provjera, stanje.load(Ordering::SeqCst))
}

fn zastavice(argumenti: &[String]) -> (bool, bool) {
    let provjera = argumenti.iter().any(|a| a == "--provjeri-pokretanje");
    let kvar = provjera && argumenti.iter().any(|a| a == "--pokvari-prvi-okvir");
    (provjera, kvar)
}

fn zastita_od_zastoja(provjera: bool) {
    if provjera {
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_secs(12));
            eprintln!("Provjera pokretanja predugo traje (12 sekundi).");
            std::process::exit(1);
        });
    }
}

fn postavke(argumenti: &[String], provjera: bool) -> PostavkeAplikacije {
    if provjera {
        PostavkeAplikacije {
            primjer: true,
            podloga_s_mreze: false,
            ..Default::default()
        }
    } else {
        let mut p = rasclani_argumente(argumenti);
        p.podloga_s_mreze = true;
        p
    }
}

fn opcije() -> Result<eframe::NativeOptions, String> {
    let slika = image::load_from_memory(IKONA)
        .map_err(|e| e.to_string())?
        .into_rgba8();
    let ikona = egui::IconData {
        width: slika.width(),
        height: slika.height(),
        rgba: slika.into_raw(),
    };
    Ok(eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA])
            .with_title(format!("TerraSimple {}", env!("CARGO_PKG_VERSION")))
            .with_icon(ikona),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    })
}

fn prikazi(
    p: PostavkeAplikacije,
    provjera: bool,
    kvar: bool,
    stanje: Arc<AtomicU8>,
) -> Result<(), String> {
    eframe::run_native(
        "TerraSimple",
        opcije()?,
        tvornica(p, provjera, kvar, stanje),
    )
    .map_err(|e| format!("Program nije mogao otvoriti grafički prozor: {e}"))
}

fn tvornica(
    p: PostavkeAplikacije,
    provjera: bool,
    kvar: bool,
    stanje: Arc<AtomicU8>,
) -> eframe::AppCreator<'static> {
    Box::new(move |cc| {
        pripremi_kartu(cc)?;
        Ok(Box::new(Omotac {
            aplikacija: pripremi_aplikaciju(p, provjera),
            provjera,
            kvar,
            stanje,
            okviri: 0,
            pocetak: Instant::now(),
        }))
    })
}

fn pripremi_aplikaciju(p: PostavkeAplikacije, provjera: bool) -> Aplikacija {
    let mut aplikacija = Aplikacija::nova(p);
    if provjera {
        // Izmišljeni primjer provjere nema korisničkih promjena za spremanje.
        aplikacija.uredjivac.oznaci_spremljeno();
    }
    aplikacija
}

fn pripremi_kartu(
    cc: &eframe::CreationContext<'_>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kartica = cc
        .wgpu_render_state
        .as_ref()
        .ok_or("Karta treba dostupnu grafičku karticu (wgpu).")?;
    karta::ugradi(kartica, 1);
    Aplikacija::postavi_kontekst(&cc.egui_ctx);
    Ok(())
}

fn provjeri_ishod(provjera: bool, stanje: u8) -> Result<(), String> {
    if provjera && stanje != 1 {
        return Err(
            "Provjera pokretanja nije uspjela: nije potvrđen ispravno iscrtan okvir karte.".into(),
        );
    }
    Ok(())
}

struct Omotac {
    aplikacija: Aplikacija,
    provjera: bool,
    kvar: bool,
    stanje: Arc<AtomicU8>,
    okviri: u32,
    pocetak: Instant,
}
impl eframe::App for Omotac {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        if self.provjera {
            self.crtaj_provjeru(ui, self.pocetak.elapsed());
        } else {
            self.aplikacija.ui(ui, frame);
        }
    }
}
impl Omotac {
    fn crtaj_provjeru(&mut self, ui: &mut egui::Ui, trajanje: Duration) {
        self.okviri += 1;
        if self.kvar && self.okviri == 1 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, egui::Color32::WHITE);
        } else {
            self.aplikacija.crtaj_sucelje(ui);
        }
        self.provjeri(ui.ctx(), trajanje);
    }
    fn provjeri(&mut self, ctx: &egui::Context, trajanje: Duration) {
        if self.okviri == 1 {
            let stanje = self.stanje.clone();
            let karta_prikazana = self.aplikacija.karta.status().is_some();
            ctx.request_screenshot(move |slika| {
                potvrdi_snimku(&stanje, karta_prikazana, &slika);
            });
        }
        if gotovo(self.okviri, self.stanje.load(Ordering::SeqCst), trajanje) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint();
    }
}
fn potvrdi_snimku(stanje: &AtomicU8, karta_prikazana: bool, slika: &egui::ColorImage) {
    let dobro = karta_prikazana && valjana_slika(slika);
    stanje.store(if dobro { 1 } else { 2 }, Ordering::SeqCst);
    println!("Provjera prvog okvira karte: {dobro}");
}

fn gotovo(okviri: u32, stanje: u8, trajanje: Duration) -> bool {
    (okviri >= 4 && stanje != 0) || trajanje >= Duration::from_secs(8)
}
fn valjana_slika(slika: &egui::ColorImage) -> bool {
    if slika.width() < 100 || slika.height() < 100 {
        return false;
    }
    let boje: std::collections::BTreeSet<_> = slika.pixels.iter().map(|p| p.to_array()).collect();
    boje.len() > 16
}

#[cfg(test)]
mod testovi {
    use super::*;
    #[test]
    fn kvar_se_dopusta_samo_u_provjeri() {
        assert_eq!(zastavice(&[]), (false, false));
        assert_eq!(zastavice(&["--pokvari-prvi-okvir".into()]), (false, false));
        assert_eq!(zastavice(&["--provjeri-pokretanje".into()]), (true, false));
        assert_eq!(
            zastavice(&[
                "--provjeri-pokretanje".into(),
                "--pokvari-prvi-okvir".into()
            ]),
            (true, true)
        );
    }

    #[test]
    fn provjera_crta_okvire_i_salje_zatvaranje_tek_nakon_potvrde() {
        for kvar in [false, true] {
            let stanje = Arc::new(AtomicU8::new(0));
            let mut app = Omotac {
                aplikacija: pripremi_aplikaciju(postavke(&[], true), true),
                provjera: true,
                kvar,
                stanje: stanje.clone(),
                okviri: 0,
                pocetak: Instant::now(),
            };
            let ctx = egui::Context::default();
            Aplikacija::postavi_kontekst(&ctx);
            ctx.options_mut(|o| o.max_passes = std::num::NonZeroUsize::new(1).unwrap());
            for okvir in 1..=4 {
                if okvir == 2 {
                    stanje.store(1, Ordering::SeqCst);
                }
                let ulaz = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA),
                    )),
                    ..Default::default()
                };
                let izlaz = ctx.run_ui(ulaz, |ui| app.crtaj_provjeru(ui, Duration::ZERO));
                let zatvara = izlaz.viewport_output[&egui::ViewportId::ROOT]
                    .commands
                    .contains(&egui::ViewportCommand::Close);
                izlaz.drop_without_applying_deltas();
                assert_eq!(app.okviri, okvir);
                if okvir == 1 {
                    assert_eq!(app.aplikacija.karta.status().is_some(), !kvar);
                }
                assert_eq!(zatvara, okvir == 4);
            }
        }
    }

    #[test]
    fn potvrda_trazi_i_kartu_i_sadrzaj_snimke() {
        let stanje = AtomicU8::new(0);
        let mut slika = egui::ColorImage::filled([100, 100], egui::Color32::WHITE);
        potvrdi_snimku(&stanje, true, &slika);
        assert_eq!(stanje.load(Ordering::SeqCst), 2);
        for i in 0..32 {
            slika.pixels[i] = egui::Color32::from_rgb(i as u8, 0, 0);
        }
        potvrdi_snimku(&stanje, false, &slika);
        assert_eq!(stanje.load(Ordering::SeqCst), 2);
        potvrdi_snimku(&stanje, true, &slika);
        assert_eq!(stanje.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn provjera_ne_uzima_mrezu_ni_tudje_argumente() {
        let a = vec![
            "--snimka".into(),
            "ne-smije-se-pisati.png".into(),
            "--dijalog".into(),
            "greska".into(),
        ];
        let p = postavke(&a, true);
        assert!(p.primjer && !p.podloga_s_mreze && p.snimka.is_none() && p.dijalog.is_none());
        assert!(postavke(&[], false).podloga_s_mreze);
        let o = opcije().unwrap();
        assert_eq!(o.viewport.icon.as_ref().unwrap().width, 64);
    }
    #[test]
    fn samo_provjera_moze_zatvoriti_primjer_bez_potvrde_spremanja() {
        let provjera = pripremi_aplikaciju(postavke(&[], true), true);
        assert!(!provjera.uredjivac.je_li_izmijenjen());
        let obicna = pripremi_aplikaciju(postavke(&[], true), false);
        assert!(obicna.uredjivac.je_li_izmijenjen());
    }

    #[test]
    fn nepotvrdjen_okvir_i_istek_ne_mogu_dati_prolaz() {
        for s in [0, 2] {
            assert!(provjeri_ishod(true, s).is_err());
        }
        assert!(provjeri_ishod(true, 1).is_ok());
        assert!(provjeri_ishod(false, 0).is_ok());
        assert!(!gotovo(3, 1, Duration::ZERO));
        assert!(!gotovo(4, 0, Duration::ZERO));
        assert!(gotovo(4, 1, Duration::ZERO));
        assert!(gotovo(4, 2, Duration::ZERO));
        assert!(gotovo(1, 0, Duration::from_secs(8)));
        assert!(!valjana_slika(&egui::ColorImage::filled(
            [20, 20],
            egui::Color32::WHITE
        )));
        assert!(!valjana_slika(&egui::ColorImage::filled(
            [100, 100],
            egui::Color32::WHITE
        )));
        let mut slika = egui::ColorImage::filled([100, 100], egui::Color32::WHITE);
        for i in 0..32 {
            slika.pixels[i] = egui::Color32::from_rgb(i as u8, 0, 0);
        }
        assert!(valjana_slika(&slika));
    }
}
