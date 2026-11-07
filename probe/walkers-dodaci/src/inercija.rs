// SPDX-License-Identifier: EUPL-1.2

//! Ispitivanje 3. pitanja: može li se klizanje karte nakon povlačenja ugasiti samo dok traje
//! crtanje, i koji točno poziv treba?
//!
//! `Map::show` u okviru puštanja tipke stanje `Center::Moving` pretvara u `Center::Inertia` i u
//! istom okviru već pomiče kartu za `brzina * dt`. Zato poziv poslije `Map::show` kasni: karta je
//! skočila i tako je nacrtana. Prijašnja inačica ove probe to nije vidjela, jer je klizanje mjerila
//! tek nakon tog okvira.

use eframe::egui::{self, Context, Event, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};
use walkers::{Map, MapMemory, MercatorProjection, Position, Projection, lon_lat};

const ZUM: f64 = 14.0;
const KORAK_VREMENA_S: f64 = 0.016;
const DOPUSTENO_PX: f32 = 1e-2;

/// Gasi klizanje dok traje crtanje. **Poziva se u svakom okviru prije `Map::show`.**
///
/// `center_at` pretvara `Center::Moving` u `Center::Exact`, pa povlačenje radi dalje, jer ga
/// `Map::show` u istom okviru vrati u `Moving`, ali u okviru puštanja tipke nema od čega nastati
/// `Center::Inertia`.
pub fn zaustavi_inerciju_pri_crtanju(memorija: &mut MapMemory, crtanje_aktivno: bool) {
    if crtanje_aktivno && let Some(polozaj) = memorija.detached() {
        memorija.center_at(polozaj);
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Nacin {
    BezPoziva,
    /// Prijašnja preporuka. Ostaje samo kao usporedba i namjerni kvar.
    PozivPoslijeKarte,
    PozivPrijeKarte,
}

/// Što se dogodilo u jednoj vožnji mišem, gledano kroz ono što je `Map::show` stvarno nacrtao.
#[derive(Debug, Clone, Copy)]
pub struct Voznja {
    /// Zbroj pomaka miša.
    pub pomak_misa_px: f64,
    /// Koliko je karta skočila u samom okviru puštanja tipke.
    pub skok_u_okviru_pustanja_px: f64,
    /// Koliko je još klizila u 0,64 s nakon okvira puštanja.
    pub klizanje_nakon_pustanja_px: f64,
    /// Koliko je karta na kraju daleko od zbroja pomaka miša.
    pub kraj_od_zbroja_misa_px: f64,
    pub animira_nakon_pustanja: bool,
}

/// Izvještaj automatske provjere inercije u `egui`-ju.
#[derive(Debug, Clone, Copy)]
pub struct IzvjestajInercije {
    pub bez_poziva: Voznja,
    pub poziv_poslije_karte: Voznja,
    pub poziv_prije_karte: Voznja,
    /// Sporo povlačenje s pozivom: karta kasni za zbrojem miša za `egui`-jev prag početka povlačenja.
    pub sporo_kasnjenje_px: f64,
    pub prekid_vec_zapocetog_klizanja_radi: bool,
}

struct Simulacija {
    ctx: Context,
    memorija: MapMemory,
    pocetak: Position,
    nacin: Nacin,
    vrijeme_s: f64,
    /// Pomak središta karte od početka, kako ga vidi `Map::show` u svakom okviru.
    nacrtano: Vec<Vec2>,
}

fn u_pikselima(polozaj: Position) -> Vec2 {
    let p = MercatorProjection.project(polozaj, ZUM);
    Vec2::new(p.x() as f32, p.y() as f32)
}

impl Simulacija {
    fn nova(nacin: Nacin) -> Result<Self, String> {
        let mut memorija = MapMemory::default();
        memorija.set_zoom(ZUM).map_err(|e| e.to_string())?;
        Ok(Self {
            ctx: Context::default(),
            memorija,
            pocetak: lon_lat(15.9819, 45.8150),
            nacin,
            vrijeme_s: 0.0,
            nacrtano: Vec::new(),
        })
    }

    fn okvir(&mut self, dogadaji: Vec<Event>) {
        zaustavi_inerciju_pri_crtanju(&mut self.memorija, self.nacin == Nacin::PozivPrijeKarte);
        self.vrijeme_s += KORAK_VREMENA_S;
        let ulaz = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(800.0))),
            time: Some(self.vrijeme_s),
            events: dogadaji,
            ..Default::default()
        };
        let pocetak = self.pocetak;
        let (memorija, nacrtano) = (&mut self.memorija, &mut self.nacrtano);
        let mut izlaz = self.ctx.run_ui(ulaz, |ui| {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE)
                .show(ui, |ui| {
                    Map::new(None, memorija, pocetak).show(ui, |_, _, _, m| {
                        let sada = m.detached().unwrap_or(pocetak);
                        nacrtano.push(u_pikselima(sada) - u_pikselima(pocetak));
                    });
                });
        });
        izlaz.textures_delta.clear();
        if self.nacin == Nacin::PozivPoslijeKarte
            && self.memorija.animating()
            && let Some(polozaj) = self.memorija.detached()
        {
            self.memorija.center_at(polozaj);
        }
    }

    fn tipka(&mut self, pos: Pos2, pressed: bool) {
        self.okvir(vec![Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        }]);
    }

    /// Stvarni događaji povlačenja u `egui`-ju, i na kraju puštanje tipke u zasebnom okviru.
    fn povuci_i_pusti(&mut self, koraka: usize, korak: Vec2) -> Pos2 {
        let mut kursor = Pos2::new(400.0, 400.0);
        self.okvir(vec![Event::PointerMoved(kursor)]);
        self.tipka(kursor, true);
        for _ in 0..koraka {
            kursor += korak;
            self.okvir(vec![Event::PointerMoved(kursor)]);
        }
        self.tipka(kursor, false);
        kursor
    }

    fn zadnje(&self) -> Result<Vec2, String> {
        self.nacrtano
            .last()
            .copied()
            .ok_or_else(|| "Karta nije nacrtana".to_string())
    }
}

fn odvezi(nacin: Nacin, koraka: usize, korak: Vec2) -> Result<Voznja, String> {
    let mut s = Simulacija::nova(nacin)?;
    s.povuci_i_pusti(koraka, korak);
    let okvir_pustanja = s.nacrtano.len() - 1;
    let pust = s.nacrtano[okvir_pustanja];
    let prije = s.nacrtano[okvir_pustanja - 1];
    let animira_nakon_pustanja = s.memorija.animating();
    for _ in 0..40 {
        s.okvir(Vec::new());
    }
    let kraj = s.zadnje()?;
    let zbroj_misa = -korak * koraka as f32;
    Ok(Voznja {
        pomak_misa_px: f64::from(zbroj_misa.length()),
        skok_u_okviru_pustanja_px: f64::from((pust - prije).length()),
        klizanje_nakon_pustanja_px: f64::from((kraj - pust).length()),
        kraj_od_zbroja_misa_px: f64::from((kraj - zbroj_misa).length()),
        animira_nakon_pustanja,
    })
}

/// Karta već klizi, a korisnik tada uključi crtanje. Mora stati u sljedećem okviru.
fn prekid_vec_zapocetog_klizanja() -> Result<bool, String> {
    let mut s = Simulacija::nova(Nacin::BezPoziva)?;
    s.povuci_i_pusti(6, Vec2::new(30.0, 0.0));
    for _ in 0..3 {
        s.okvir(Vec::new());
    }
    if !s.memorija.animating() {
        return Err("Karta bi još morala kliziti nakon 3 okvira".to_string());
    }
    let pri_ukljucenju = s.zadnje()?;
    s.nacin = Nacin::PozivPrijeKarte;
    for _ in 0..15 {
        s.okvir(Vec::new());
    }
    let pomak = (s.zadnje()? - pri_ukljucenju).length();
    Ok(!s.memorija.animating() && pomak <= DOPUSTENO_PX)
}

/// Provjerava inerciju. Uz `kvar` se umjesto poziva prije karte koristi poziv poslije karte, što je
/// stvarna greška iz prijašnje inačice probe i mora srušiti provjeru.
pub fn provjeri_inerciju(kvar: bool) -> Result<IzvjestajInercije, String> {
    let brzo = Vec2::new(25.0, -15.0);
    let bez_poziva = odvezi(Nacin::BezPoziva, 6, brzo)?;
    let poziv_poslije_karte = odvezi(Nacin::PozivPoslijeKarte, 6, brzo)?;
    let nacin = if kvar {
        Nacin::PozivPoslijeKarte
    } else {
        Nacin::PozivPrijeKarte
    };
    let poziv_prije_karte = odvezi(nacin, 6, brzo)?;
    let sporo = odvezi(nacin, 10, Vec2::new(3.0, 0.0))?;
    let izvjestaj = IzvjestajInercije {
        bez_poziva,
        poziv_poslije_karte,
        poziv_prije_karte,
        sporo_kasnjenje_px: sporo.kraj_od_zbroja_misa_px,
        prekid_vec_zapocetog_klizanja_radi: prekid_vec_zapocetog_klizanja()?,
    };

    if !bez_poziva.animira_nakon_pustanja || bez_poziva.klizanje_nakon_pustanja_px < 100.0 {
        return Err(format!("Karta bez poziva nije klizila: {bez_poziva:?}"));
    }
    if poziv_poslije_karte.skok_u_okviru_pustanja_px < 10.0 {
        return Err(format!(
            "Poziv poslije karte više ne ostavlja skok, pa usporedba ne vrijedi: {poziv_poslije_karte:?}"
        ));
    }
    let dopusteno = f64::from(DOPUSTENO_PX);
    if poziv_prije_karte.animira_nakon_pustanja
        || poziv_prije_karte.skok_u_okviru_pustanja_px > dopusteno
        || poziv_prije_karte.klizanje_nakon_pustanja_px > dopusteno
        || poziv_prije_karte.kraj_od_zbroja_misa_px > dopusteno
    {
        return Err(format!("Karta s pozivom nije stala na mišu: {poziv_prije_karte:?}"));
    }
    if !izvjestaj.prekid_vec_zapocetog_klizanja_radi {
        return Err("Prekid već započetog klizanja nije zaustavio kartu".to_string());
    }
    Ok(izvjestaj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poziv_prije_karte_nema_skoka_a_poslije_ima() {
        let i = provjeri_inerciju(false).unwrap();
        assert!(i.bez_poziva.klizanje_nakon_pustanja_px > 150.0, "{i:?}");
        assert!(i.poziv_poslije_karte.skok_u_okviru_pustanja_px > 10.0, "{i:?}");
        assert!(i.poziv_prije_karte.skok_u_okviru_pustanja_px < 1e-2, "{i:?}");
        assert!(i.poziv_prije_karte.kraj_od_zbroja_misa_px < 1e-2, "{i:?}");
        assert!(i.prekid_vec_zapocetog_klizanja_radi);
    }

    #[test]
    fn test_poziv_poslije_karte_je_kvar_koji_pada() {
        assert!(provjeri_inerciju(true).is_err());
    }
}
