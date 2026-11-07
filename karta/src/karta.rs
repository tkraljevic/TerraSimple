//! Karta na ekranu: podloga, slojevi i miš.

use crate::crtac::{self, Poziv};
use crate::crtanje::{Crtanje, Okolina, Pretvorba, slojevi_odozgo};
use crate::crtez_na_karti;
use crate::kamera::Kamera;
use crate::mercator::{metara_po_stupnju, u_mercator};
use crate::podloga::Podloga;
use crate::priprema::{Razina, pripremi_sloj};
use eframe::egui::{self, DragPanButtons, Key, Modifiers, PointerButton, Pos2, Response, Ui};
use slojevi::{IdObjekta, IdSloja, Uredjivac, Vrh};
use std::collections::BTreeMap;
use std::sync::Arc;
use walkers::{Map, MapMemory, Position, Projector, Tiles, lon_lat};

impl Pretvorba for Projector<'_> {
    fn u_ekran(&self, v: &Vrh) -> Pos2 {
        self.project(lon_lat(v.duzina, v.sirina)).to_pos2()
    }

    fn na_kartu(&self, m: Pos2) -> Vrh {
        let p = self.unproject(m.to_vec2());
        Vrh::novi(p.x(), p.y())
    }
}

/// Pripremljen sloj, s onim što je vrijedilo kad je pripremljen.
struct Pripremljen {
    inacica: u64,
    skriveni: Option<IdObjekta>,
    razine: Arc<Vec<Razina>>,
}

pub struct Karta {
    memorija: MapMemory,
    izvor: Option<Box<dyn Tiles>>,
    podloga: Option<Podloga>,
    pocetak: Position,
    crtanje: Crtanje,
    pripremljeni: BTreeMap<IdSloja, Pripremljen>,
    /// Klik dok karta klizi samo je zaustavlja (Odluke (7. 11. 2026.)).
    guta_klik: bool,
}

impl Karta {
    /// Karta s podlogom. Pločice se traže u pozadini, kroz `ctx`.
    pub fn nova(podloga: Podloga, ctx: egui::Context, duzina: f64, sirina: f64) -> Self {
        let mut karta = Self::bez_podloge(duzina, sirina);
        karta.izvor = Some(podloga.izvor(ctx));
        karta.podloga = Some(podloga);
        karta
    }

    /// Karta bez podloge, npr. kad nema mreže ili u testovima.
    pub fn bez_podloge(duzina: f64, sirina: f64) -> Self {
        Self {
            memorija: MapMemory::default(),
            izvor: None,
            podloga: None,
            pocetak: lon_lat(duzina, sirina),
            crtanje: Crtanje::default(),
            pripremljeni: BTreeMap::new(),
            guta_klik: false,
        }
    }

    pub fn crtanje(&mut self) -> &mut Crtanje {
        &mut self.crtanje
    }

    pub fn memorija(&mut self) -> &mut MapMemory {
        &mut self.memorija
    }

    /// Mijenja podlogu, a položaj i zum ostaju.
    pub fn postavi_podlogu(&mut self, podloga: Podloga, ctx: egui::Context) {
        self.izvor = Some(podloga.izvor(ctx));
        self.podloga = Some(podloga);
    }

    /// Prikazuje kartu i obrađuje miš i tipkovnicu.
    pub fn prikazi(&mut self, ui: &mut Ui, u: &mut Uredjivac, ok: &Okolina<'_>) {
        self.tipkovnica(ui);
        self.zaustavi_pri_pritisku(ui);
        let pritisak = ui.input(|i| i.pointer.press_origin());
        let pomice = self.crtanje.smije_pomicati_kartu(pritisak);
        let slojevi = self.uskladi_slojeve(u, ok);
        let tipke = if pomice {
            DragPanButtons::PRIMARY
        } else {
            DragPanButtons::empty()
        };
        let (crtanje, guta) = (&mut self.crtanje, &mut self.guta_klik);
        let podloga = self.podloga;
        let izvor: Option<&mut dyn Tiles> = match self.izvor.as_mut() {
            Some(i) => Some(i.as_mut()),
            None => None,
        };
        // Dvoklik i povlačenje imaju značenje u crtanju, pa ih karta ne smije preuzeti.
        Map::new(izvor, &mut self.memorija, self.pocetak)
            .double_click_to_zoom(false)
            .drag_pan_buttons(tipke)
            .show(ui, |ui, odgovor, projektor, _| {
                ui.painter().add(crtac::poziv(
                    odgovor.rect,
                    Poziv {
                        kamera: kamera(odgovor, projektor),
                        slojevi,
                    },
                ));
                crtanje.osvjezi_hvatista(u.projekt(), projektor);
                mis(ui, odgovor, projektor, crtanje, guta, u, ok);
                crtanje.osvjezi_hvatista(u.projekt(), projektor);
                let polozaj = ui.input(|i| i.pointer.latest_pos());
                crtez_na_karti::nacrtaj(ui.painter(), crtanje, u.projekt(), projektor, polozaj, ok);
                if let Some(p) = podloga {
                    crtez_na_karti::nacrtaj_navod(ui.painter(), odgovor.rect, p.navod());
                }
            });
    }

    fn tipkovnica(&mut self, ui: &mut Ui) {
        let (esc, nazad) = ui.input_mut(|i| {
            (
                i.consume_key(Modifiers::NONE, Key::Escape),
                i.consume_key(Modifiers::NONE, Key::Backspace),
            )
        });
        if esc {
            self.crtanje.odustani();
        }
        if nazad {
            self.crtanje.natrag();
        }
    }

    /// Mora biti prije `Map::show`: poslije nje karta je u okviru puštanja već skočila (probe/walkers-dodaci).
    fn zaustavi_pri_pritisku(&mut self, ui: &mut Ui) {
        let pritisnuto = ui.input(|i| i.pointer.primary_pressed());
        let na_karti = ui.rect_contains_pointer(ui.max_rect());
        if pritisnuto && na_karti && self.memorija.animating() {
            if let Some(polozaj) = self.memorija.detached() {
                self.memorija.center_at(polozaj);
            }
            self.guta_klik = true;
        }
    }

    /// Priprema samo slojeve koji su se promijenili, a vraća ih redom crtanja, odozdo prema gore.
    fn uskladi_slojeve(
        &mut self,
        u: &Uredjivac,
        ok: &Okolina<'_>,
    ) -> Arc<Vec<(IdSloja, Arc<Vec<Razina>>)>> {
        let projekt = u.projekt();
        let skriveni = self.crtanje.skriveni();
        let odozgo = slojevi_odozgo(projekt);
        self.pripremljeni.retain(|id, _| odozgo.contains(id));
        let mut izlaz = Vec::with_capacity(odozgo.len());
        for id in odozgo.into_iter().rev().filter(|s| (ok.vidljiv)(*s)) {
            let Some(sloj) = projekt.sloj(id) else {
                continue;
            };
            let inacica = u.inacica_sloja(id);
            let skriven = skriveni.filter(|(s, _)| *s == id).map(|(_, o)| o);
            let vrijedi = self
                .pripremljeni
                .get(&id)
                .is_some_and(|p| p.inacica == inacica && p.skriveni == skriven);
            if !vrijedi {
                let razine = Arc::new(pripremi_sloj(sloj, skriven));
                self.pripremljeni.insert(
                    id,
                    Pripremljen {
                        inacica,
                        skriveni: skriven,
                        razine,
                    },
                );
            }
            if let Some(p) = self.pripremljeni.get(&id) {
                izlaz.push((id, p.razine.clone()));
            }
        }
        Arc::new(izlaz)
    }
}

fn kamera(odgovor: &Response, p: &Projector<'_>) -> Kamera {
    let okvir = odgovor.rect;
    let sredina = p.unproject(okvir.center().to_vec2());
    let (a, b) = (
        p.project(sredina),
        p.project(lon_lat(sredina.x() + 0.01, sredina.y())),
    );
    Kamera {
        sredina: u_mercator(sredina.x(), sredina.y()),
        metara_po_px: 0.01 * metara_po_stupnju() / f64::from(b.x - a.x),
        pola: [okvir.width() / 2.0, okvir.height() / 2.0],
    }
}

/// Prevodi događaje `egui`-ja u pravila crtanja.
fn mis(
    ui: &Ui,
    odgovor: &Response,
    p: &Projector<'_>,
    c: &mut Crtanje,
    guta: &mut bool,
    u: &mut Uredjivac,
    ok: &Okolina<'_>,
) {
    let (tu, pritisak) = ui.input(|i| (i.pointer.latest_pos(), i.pointer.press_origin()));
    if odgovor.drag_started_by(PointerButton::Primary)
        && let (Some(izvor), Some(m)) = (pritisak, tu)
    {
        c.pocni_vuci(izvor, p.na_kartu(m), u.projekt());
    }
    if let (true, Some(m)) = (c.vuce(), tu) {
        c.vuci(p.na_kartu(m));
    }
    if c.vuce() && !ui.input(|i| i.pointer.primary_down()) {
        c.pusti(u);
    }
    klikovi(odgovor, tu, p, c, guta, u, ok);
    if ui.input(|i| i.pointer.any_released()) {
        *guta = false;
    }
}

fn klikovi(
    odgovor: &Response,
    tu: Option<Pos2>,
    p: &Projector<'_>,
    c: &mut Crtanje,
    guta: &bool,
    u: &mut Uredjivac,
    ok: &Okolina<'_>,
) {
    let Some(m) = tu else {
        return;
    };
    // Dvoklik je i klik, pa se mora gledati prvi. Prvi klik para već je dodao točku.
    if odgovor.double_clicked_by(PointerButton::Primary) {
        c.zavrsi(u, ok);
    } else if odgovor.clicked_by(PointerButton::Primary) && !*guta {
        c.klik(m, u, ok, p);
    }
    if odgovor.clicked_by(PointerButton::Secondary) {
        c.desni_klik(m, u);
    }
}

#[cfg(test)]
#[path = "karta_testovi.rs"]
mod testovi;
