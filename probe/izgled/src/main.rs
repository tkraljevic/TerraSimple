//! Proba faze 0: može li egui izgledati kao zaslon A iz nacrta.
//!
//! Ovo se baca. Odgovara na jedno pitanje i ne ulazi u program.
//! `--snimka put.png [--tamna] [--lijevo]` snimi prozor i izađe.
#![forbid(unsafe_code)]

mod boje;
mod crtanje;
mod ikone;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use eframe::egui::{
    self, Align2, Color32, ColorImage, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Painter, Pos2, Rect, ScreenshotCallback, Sense, Shadow, Stroke, StrokeKind, Ui,
    ViewportCommand, pos2, vec2,
};

use boje::{Boje, SVIJETLA, TAMNA};
use crtanje::{crta, isprekidana, lik, put_crta, put_lik};
use ikone::ikona;

const MENI: f32 = 40.0;
const STATUS: f32 = 28.0;
const PLOCA: f32 = 360.0;
const RUB: f32 = 12.0;
const M_PO_PX: f32 = 0.2646e-3 * 4500.0;

const PARCELA: [(f32, f32); 6] = [(420.0, 250.0), (560.0, 215.0), (640.0, 290.0), (610.0, 390.0), (470.0, 410.0), (395.0, 335.0)];
const SUSJED1: [(f32, f32); 4] = [(650.0, 220.0), (780.0, 250.0), (770.0, 350.0), (640.0, 380.0)];
const SUSJED2: [(f32, f32); 4] = [(285.0, 150.0), (400.0, 130.0), (440.0, 225.0), (315.0, 250.0)];

fn obicno(v: f32) -> FontId {
    FontId::new(v, FontFamily::Proportional)
}
fn srednje(v: f32) -> FontId {
    FontId::new(v, FontFamily::Name("Srednji".into()))
}
fn kurziv(v: f32) -> FontId {
    FontId::new(v, FontFamily::Name("Kurziv".into()))
}

struct Redak {
    razina: usize,
    ikona: &'static str,
    naziv: &'static str,
    opis: &'static str,
    boja: Option<Color32>,
    otvoriv: Option<bool>,
    ukljucen: bool,
    odabran: bool,
    oznaka: &'static str,
    brava: bool,
}

struct Proba {
    tema_tamna: bool,
    alat: usize,
    ploca_lijevo: bool,
    iskljuceni: HashSet<&'static str>,
    zatvoreni: HashSet<&'static str>,
    podloga: usize,
    snimka: Option<String>,
    slika: Arc<Mutex<Option<Arc<ColorImage>>>>,
    okvir: u32,
}

fn pomak(o: Rect, x: f32, y: f32) -> Pos2 {
    pos2(o.min.x + x, o.min.y + y)
}

fn tocke_u(o: Rect, t: &[(f32, f32)]) -> Vec<Pos2> {
    t.iter().map(|(x, y)| pomak(o, *x, *y)).collect()
}

fn natpis(p: &Painter, b: &Boje, t: &str, mjesto: Pos2) {
    let g = p.layout_no_wrap(t.to_string(), srednje(12.0), b.tekst);
    let o = Rect::from_min_size(mjesto, vec2(g.size().x + 16.0, 20.0));
    p.rect(o, 10.0, b.ploha, Stroke::new(1.0, b.rub), StrokeKind::Inside);
    p.galley(pos2(o.min.x + 8.0, o.center().y - g.size().y / 2.0), g, b.tekst);
}

fn karta(p: &Painter, b: &Boje, o: Rect) {
    p.rect_filled(o, 0.0, b.k_zemlja);
    for z in [
        "M60 420 C140 380 240 400 300 470 C340 530 280 620 190 630 C100 640 20 540 60 420 Z",
        "M660 40 C760 10 900 30 960 90 C1000 150 900 210 800 190 C710 175 610 110 660 40 Z",
        "M720 600 C800 560 920 590 960 660 C990 730 880 800 780 780 C700 760 660 650 720 600 Z",
    ] {
        put_lik(p, z, o.min, b.k_suma);
    }
    for z in [
        "M130 90 L250 70 L300 130 L270 210 L160 220 L110 160 Z",
        "M760 300 L870 280 L930 340 L900 420 L790 430 L740 370 Z",
        "M420 620 L510 600 L560 660 L520 730 L430 720 Z",
    ] {
        put_lik(p, z, o.min, b.k_naselje);
    }
    for (z, d, boja) in [
        ("M-20 120 C200 170 480 80 700 250 C860 370 1100 330 1460 390", 5.0, b.k_glavna),
        ("M210 -20 C230 200 200 400 480 660 C560 730 600 800 620 860", 3.5, b.k_cesta),
        ("M830 -20 C840 120 830 260 830 360 C830 500 940 700 960 860", 3.5, b.k_cesta),
        ("M-20 700 C200 690 340 660 480 660", 3.0, b.k_cesta),
    ] {
        put_crta(p, z, o.min, d + 2.0, b.k_cesta_rub);
        put_crta(p, z, o.min, d, boja);
    }
    put_crta(p, "M300 -20 C340 90 420 160 470 240 C520 320 590 370 640 400", o.min, 3.0, b.k_voda);
    put_crta(p, "M-20 250 C120 300 260 330 400 420 C520 500 660 420 800 500 C940 580 1100 540 1460 620", o.min, 9.0, b.k_voda);
    for (t, x, y, f) in [
        ("Novo Selo", 165.0, 132.0, srednje(13.0)),
        ("Brestovac", 790.0, 342.0, srednje(13.0)),
        ("Dolina", 450.0, 652.0, srednje(13.0)),
        ("Hrastova šuma", 730.0, 92.0, obicno(12.0)),
        ("Lug", 165.0, 500.0, obicno(12.0)),
    ] {
        p.text(pomak(o, x, y), Align2::LEFT_TOP, t, f, b.k_natpis);
    }
    p.text(pomak(o, 600.0, 424.0), Align2::LEFT_TOP, "Rijeka", kurziv(12.0), b.k_voda_natpis);

    for (t, d) in [(&SUSJED1[..], 1.75), (&SUSJED2[..], 1.75), (&PARCELA[..], 3.0)] {
        let v = tocke_u(o, t);
        lik(p, &v, b.crtez_ploha);
        crta(p, &v, true, d, b.crtez);
    }
    put_crta(p, "M120 300 C180 280 220 340 270 330 C320 320 340 290 395 335", o.min, 3.0, Color32::from_rgb(0x15, 0x65, 0xC0));

    let v = tocke_u(o, &PARCELA);
    let mut zatvoreno = v.clone();
    zatvoreno.push(v[0]);
    isprekidana(p, &zatvoreno, 1.0, b.ploha, 5.0, 4.0);
    for i in 0..v.len() {
        let (a, c) = (v[i], v[(i + 1) % v.len()]);
        let s = pos2((a.x + c.x) / 2.0, (a.y + c.y) / 2.0);
        p.circle(s, 4.0, Color32::from_white_alpha(179), Stroke::new(1.0, b.crtez));
    }
    for t in &v {
        p.circle(*t, 5.5, Color32::WHITE, Stroke::new(2.0, b.crtez));
    }

    for (x, y, t) in [(170.0, 240.0, "Zdenac"), (720.0, 470.0, "Ulaz na parcelu")] {
        pribadaca(p, pomak(o, x, y));
        natpis(p, b, t, pomak(o, x + 12.0, y - 30.0));
    }
    natpis(p, b, "Parcela 1124/3", pomak(o, 440.0, 300.0));
}

fn pribadaca(p: &Painter, t: Pos2) {
    let crvena = Color32::from_rgb(0xC6, 0x28, 0x28);
    let z = "M0 0 C-3 -8 -9 -11 -9 -18 C-9 -24 -5 -28 0 -28 C5 -28 9 -24 9 -18 C9 -11 3 -8 0 0 Z";
    put_lik(p, z, t, crvena);
    put_crta(p, z, t, 1.5, Color32::WHITE);
    p.circle_filled(pos2(t.x, t.y - 18.0), 3.0, Color32::WHITE);
}

fn sjena(p: &Painter, b: &Boje, o: Rect, r: f32) {
    let s = Shadow { offset: [0, 2], blur: 8, spread: 0, color: b.sjena };
    p.add(s.as_shape(o, CornerRadius::same(r as u8)));
}

fn razdjelnik(p: &Painter, b: &Boje, x1: f32, x2: f32, y: f32) {
    p.rect_filled(Rect::from_min_max(pos2(x1, y), pos2(x2, y + 1.0)), 0.0, b.rub);
}

fn prekidac(p: &Painter, b: &Boje, o: Rect, uk: bool) {
    if uk {
        p.rect_filled(o, 10.0, b.glavna);
        p.circle_filled(pos2(o.max.x - 10.0, o.center().y), 7.0, b.na_glavnoj);
    } else {
        p.rect(o, 10.0, b.visoko, Stroke::new(1.0, b.rub), StrokeKind::Inside);
        p.circle_filled(pos2(o.min.x + 10.0, o.center().y), 6.0, b.blijedo);
    }
}

fn gumb_ikona(ui: &mut Ui, id: &str, o: Rect, ime: &str, velicina: f32, boja: Color32) -> bool {
    let r = ui.interact(o, ui.scope_id().with(id), Sense::click());
    if r.hovered() {
        ui.painter().rect_filled(o, 6.0, boja.gamma_multiply(0.1));
    }
    ikona(ui.painter(), ime, pos2(o.center().x - velicina / 2.0, o.center().y - velicina / 2.0), velicina, boja);
    r.clicked()
}

fn opseg_m(t: &[(f32, f32)]) -> f32 {
    (0..t.len())
        .map(|i| {
            let (a, c) = (t[i], t[(i + 1) % t.len()]);
            ((a.0 - c.0).powi(2) + (a.1 - c.1).powi(2)).sqrt()
        })
        .sum::<f32>()
        * M_PO_PX
}

fn povrsina_ha(t: &[(f32, f32)]) -> f32 {
    let z: f32 = (0..t.len())
        .map(|i| {
            let (a, c) = (t[i], t[(i + 1) % t.len()]);
            a.0 * c.1 - c.0 * a.1
        })
        .sum();
    z.abs() / 2.0 * M_PO_PX * M_PO_PX / 10000.0
}

impl Proba {
    fn traka_izbornika(&mut self, ui: &mut Ui, b: &Boje, o: Rect) {
        let p = ui.painter().clone();
        p.rect_filled(o, 0.0, b.vrh);
        razdjelnik(&p, b, o.min.x, o.max.x, o.max.y - 1.0);
        let z = Rect::from_min_size(pos2(o.min.x + 12.0, o.min.y + 9.0), vec2(22.0, 22.0));
        p.rect_filled(z, 6.0, b.glavna);
        put_crta(&p, "M11 4 L18 9 L15.5 18 L6.5 18 L4 9 Z", z.min, 1.75, b.na_glavnoj);
        let mut x = o.min.x + 46.0;
        for t in ["Datoteka", "Uredi", "Crtanje", "Slojevi", "Prikaz", "Ispis", "Alati", "Pomoć"] {
            let g = p.layout_no_wrap(t.to_string(), obicno(13.0), b.tekst);
            let w = g.size().x + 20.0;
            let r = Rect::from_min_size(pos2(x, o.min.y + 6.0), vec2(w, 28.0));
            if ui.interact(r, ui.scope_id().with(t), Sense::click()).hovered() {
                p.rect_filled(r, 6.0, b.visoko);
            }
            p.galley(pos2(x + 10.0, r.center().y - g.size().y / 2.0), g, b.tekst);
            x += w;
        }
        p.text(o.center(), Align2::CENTER_CENTER, "Teren 2026.gpkg  ·  spremljeno 20:41", obicno(13.0), b.tekst);
        let mut d = o.max.x - 8.0 - 32.0;
        for (i, (ime, boja)) in [("pomoc", b.sivo), ("mjesec", b.sivo), ("ponovi", b.blijedo), ("ponisti", b.sivo)].iter().enumerate() {
            let r = Rect::from_min_size(pos2(d, o.min.y + 4.0), vec2(32.0, 32.0));
            if gumb_ikona(ui, ime, r, ime, 18.0, *boja) && i == 1 {
                self.tema_tamna = !self.tema_tamna;
            }
            d -= 32.0;
        }
        let g = p.layout_no_wrap("Otvori u QGIS-u".to_string(), srednje(12.0), b.glavna);
        let w = g.size().x + 24.0;
        let q = Rect::from_min_size(pos2(d - w - 2.0, o.min.y + 7.0), vec2(w, 26.0));
        p.rect_stroke(q, 13.0, Stroke::new(1.0, b.rub), StrokeKind::Inside);
        p.galley(pos2(q.min.x + 12.0, q.center().y - g.size().y / 2.0), g, b.glavna);
    }

    fn statusna(&self, p: &Painter, b: &Boje, o: Rect) {
        p.rect_filled(o, 0.0, b.vrh);
        razdjelnik(p, b, o.min.x, o.max.x, o.min.y);
        let y = o.center().y;
        p.text(pos2(12.0, y), Align2::LEFT_CENTER, "E 655 120   N 5 046 380", obicno(12.0), b.tekst);
        p.text(pos2(173.0, y), Align2::LEFT_CENTER, "HTRS96/TM", srednje(12.0), b.tekst);
        ikona(p, "dolje", pos2(247.0, y - 7.0), 14.0, b.sivo);
        p.text(pos2(277.0, y), Align2::LEFT_CENTER, "1 : 4 500", srednje(12.0), b.tekst);
        ikona(p, "dolje", pos2(330.0, y - 7.0), 14.0, b.sivo);
        p.text(pos2(o.max.x - 12.0, y), Align2::RIGHT_CENTER, "Mreža radi", obicno(12.0), b.tekst);
        ikona(p, "mreza", pos2(o.max.x - 94.0, y - 7.0), 14.0, b.sivo);
        p.text(pos2(o.max.x - 110.0, y), Align2::RIGHT_CENTER, "OpenFreeMap", obicno(12.0), b.tekst);
        ikona(p, "karta", pos2(o.max.x - 211.0, y - 7.0), 14.0, b.sivo);
    }

    fn alati(&mut self, ui: &mut Ui, b: &Boje, x: f32, y: f32) {
        let p = ui.painter().clone();
        let o = Rect::from_min_size(pos2(x, y), vec2(68.0, 259.0));
        sjena(&p, b, o, 12.0);
        p.rect_filled(o, 12.0, b.ploha);
        let mut gy = y + 4.0;
        for (i, (ime, t)) in [("odaberi", "Odaberi"), ("tocka", "Točka"), ("linija", "Linija"), ("poligon", "Poligon"), ("ravnalo", "Mjeri")].iter().enumerate() {
            if i == 4 {
                razdjelnik(&p, b, x + 4.0, x + 64.0, gy + 1.0);
                gy += 3.0;
            }
            let r = Rect::from_min_size(pos2(x + 4.0, gy), vec2(60.0, 48.0));
            let a = self.alat == i;
            let odziv = ui.interact(r, ui.scope_id().with(*t), Sense::click());
            if odziv.clicked() {
                self.alat = i;
            }
            if a {
                p.rect_filled(r, 8.0, b.tint);
            } else if odziv.hovered() {
                p.rect_filled(r, 8.0, b.visoko);
            }
            let boja = if a { b.na_tintu } else { b.sivo };
            ikona(&p, ime, pos2(r.center().x - 10.0, gy + 7.0), 20.0, boja);
            p.text(pos2(r.center().x, gy + 31.0), Align2::CENTER_TOP, *t, if a { srednje(11.0) } else { obicno(11.0) }, boja);
            gy += 50.0;
        }
    }

    fn cip(&mut self, ui: &mut Ui, b: &Boje, x: f32, y: f32, t: &str, i: usize) -> f32 {
        let p = ui.painter().clone();
        let a = self.podloga == i;
        let boja = if a { b.na_tintu } else { b.tekst };
        let g = p.layout_no_wrap(t.to_string(), srednje(13.0), boja);
        let w = g.size().x + 24.0 + if a { 22.0 } else { 0.0 };
        let r = Rect::from_min_size(pos2(x, y), vec2(w, 32.0));
        if ui.interact(r, ui.scope_id().with(t), Sense::click()).clicked() {
            self.podloga = i;
        }
        if a {
            p.rect_filled(r, 8.0, b.tint);
            ikona(&p, "kvacica", pos2(x + 12.0, y + 8.0), 16.0, b.na_tintu);
            p.galley(pos2(x + 34.0, r.center().y - g.size().y / 2.0), g, boja);
        } else {
            p.rect_stroke(r, 8.0, Stroke::new(1.0, b.rub), StrokeKind::Inside);
            p.galley(pos2(x + 12.0, r.center().y - g.size().y / 2.0), g, boja);
        }
        w
    }

    fn gumb_obrub(&self, p: &Painter, b: &Boje, x: f32, y: f32, ime: &str, t: &str) -> f32 {
        let g = p.layout_no_wrap(t.to_string(), srednje(14.0), b.glavna);
        let w = g.size().x + 44.0;
        let r = Rect::from_min_size(pos2(x, y), vec2(w, 32.0));
        p.rect_stroke(r, 16.0, Stroke::new(1.0, b.rub), StrokeKind::Inside);
        ikona(p, ime, pos2(x + 10.0, y + 7.0), 18.0, b.glavna);
        p.galley(pos2(x + 32.0, r.center().y - g.size().y / 2.0), g, b.glavna);
        w
    }

    fn ploca(&mut self, ui: &mut Ui, b: &Boje, o: Rect, sredina_x: f32) {
        let p = ui.painter().clone();
        sjena(&p, b, o, 16.0);
        p.rect_filled(o, 16.0, b.vrh);
        let (x, s) = (o.min.x + 16.0, o.width() - 32.0);

        // Glava: za nju se ploča povlači na drugi rub.
        let glava = Rect::from_min_size(o.min, vec2(o.width() - 44.0, 52.0));
        let odziv = ui.interact(glava, ui.scope_id().with("glava ploče"), Sense::drag());
        if odziv.drag_stopped() {
            if let Some(t) = odziv.interact_pointer_pos() {
                self.ploca_lijevo = t.x < sredina_x;
            }
        }
        ikona(&p, "hvataljka", pos2(x, o.min.y + 19.0), 18.0, b.blijedo);
        p.text(pos2(x + 22.0, o.min.y + 28.0), Align2::LEFT_CENTER, "Slojevi", obicno(20.0), b.tekst);
        ikona(&p, "x", pos2(o.max.x - 34.0, o.min.y + 18.0), 20.0, b.sivo);

        let mut gx = x;
        for (ime, t) in [("plus", "Novi sloj"), ("uvezi", "Uvezi sloj"), ("nova_mapa", "Mapa")] {
            gx += self.gumb_obrub(&p, b, gx, o.min.y + 56.0, ime, t) + 6.0;
        }

        let parcele = !self.zatvoreni.contains("Parcele");
        let teren = !self.zatvoreni.contains("Teren 2026");
        let uvezeno = !self.zatvoreni.contains("Uvezeno");
        let (narancasta, plava, crvena) = (b.crtez, Color32::from_rgb(0x15, 0x65, 0xC0), Color32::from_rgb(0xC6, 0x28, 0x28));
        let r = |razina, ikona, naziv, opis, boja, otvoriv| Redak { razina, ikona, naziv, opis, boja, otvoriv, ukljucen: true, odabran: false, oznaka: "", brava: false };
        let mut retci = vec![r(0, "mapa", "Teren 2026", "3 sloja", None, Some(teren))];
        if teren {
            retci.push(r(1, "poligon", "Parcele", "12 objekata", Some(narancasta), Some(parcele)));
            if parcele {
                retci.push(r(2, "poligon", "Parcela 1118/2", "2,31 ha", Some(narancasta), None));
                let mut o = r(2, "poligon", "Parcela 1124/3", "4,83 ha", Some(narancasta), None);
                o.odabran = true;
                retci.push(o);
                retci.push(r(2, "poligon", "Parcela 1126", "1,87 ha", Some(narancasta), None));
            }
            retci.push(r(1, "linija", "Trase", "1 objekt", Some(plava), Some(false)));
            retci.push(r(1, "tocka", "Točke", "2 objekta", Some(crvena), Some(false)));
        }
        let mut p25 = r(0, "mapa", "Parcele 2025", "2 sloja", None, Some(false));
        p25.ukljucen = false;
        retci.push(p25);
        retci.push(r(0, "mapa", "Uvezeno", "2 sloja", None, Some(uvezeno)));
        if uvezeno {
            let mut k = r(1, "datoteka", "obilazak", "23 objekta", None, Some(false));
            k.oznaka = "KML";
            retci.push(k);
            let mut g = r(1, "baza", "nasipi", "samo za čitanje", None, Some(false));
            g.oznaka = "GPKG";
            g.brava = true;
            g.ukljucen = false;
            retci.push(g);
        }

        let mut y = o.min.y + 96.0;
        for d in &retci {
            self.redak(ui, b, d, x, y, s);
            y += 46.0;
        }

        let y = o.max.y - 160.0;
        razdjelnik(&p, b, x, x + s, y);
        p.text(pos2(x, y + 22.0), Align2::LEFT_CENTER, "Podloga", obicno(13.0), b.sivo);
        let w = self.cip(ui, b, x, y + 39.0, "Karta", 0);
        self.cip(ui, b, x + w + 8.0, y + 39.0, "DGU ortofoto", 1);
        self.cip(ui, b, x, y + 81.0, "ortofoto_2024.tif", 2);
        let t = y + 123.0;
        ikona(&p, "uvezi", pos2(x, t), 18.0, b.glavna);
        p.text(pos2(x + 26.0, t + 9.0), Align2::LEFT_CENTER, "Učitaj datoteku", srednje(14.0), b.glavna);
        ikona(&p, "mreza", pos2(x + 142.0, t), 18.0, b.glavna);
        p.text(pos2(x + 168.0, t + 9.0), Align2::LEFT_CENTER, "Dodaj WMS", srednje(14.0), b.glavna);
    }

    fn redak(&mut self, ui: &mut Ui, b: &Boje, d: &Redak, x: f32, y: f32, s: f32) {
        let p = ui.painter().clone();
        let o = Rect::from_min_size(pos2(x, y), vec2(s, 46.0));
        if d.odabran {
            p.rect_filled(o, 8.0, b.tint);
        }
        let mut tx = x + 4.0 + d.razina as f32 * 18.0;
        if let Some(otvoren) = d.otvoriv {
            let r = Rect::from_min_size(pos2(tx, y + 14.0), vec2(18.0, 18.0));
            if ui.interact(r, ui.scope_id().with((d.naziv, "otvori")), Sense::click()).clicked() {
                if otvoren {
                    self.zatvoreni.insert(d.naziv);
                } else {
                    self.zatvoreni.remove(d.naziv);
                }
            }
            ikona(&p, if otvoren { "dolje" } else { "desno" }, r.min, 18.0, b.sivo);
        }
        tx += 24.0;
        ikona(&p, d.ikona, pos2(tx, y + 14.0), 18.0, d.boja.unwrap_or(b.sivo));
        tx += 24.0;

        let mut desno = x + s - 6.0 - 36.0;
        let promijenjen = self.iskljuceni.contains(d.naziv);
        let uk = d.ukljucen != promijenjen;
        let t = Rect::from_min_size(pos2(desno, y + 13.0), vec2(36.0, 20.0));
        if ui.interact(t, ui.scope_id().with((d.naziv, "prekidač")), Sense::click()).clicked() && !self.iskljuceni.remove(d.naziv) {
            self.iskljuceni.insert(d.naziv);
        }
        prekidac(&p, b, t, uk);
        if d.brava {
            desno -= 20.0;
            ikona(&p, "brava", pos2(desno, y + 16.0), 14.0, b.sivo);
        }
        if !d.oznaka.is_empty() {
            let g = p.layout_no_wrap(d.oznaka.to_string(), srednje(11.0), b.sivo);
            let w = g.size().x + 12.0;
            desno -= w + 6.0;
            let r = Rect::from_min_size(pos2(desno, y + 15.0), vec2(w, 16.0));
            p.rect_stroke(r, 4.0, Stroke::new(1.0, b.rub), StrokeKind::Inside);
            p.galley(pos2(desno + 6.0, r.center().y - g.size().y / 2.0), g, b.sivo);
        }

        let boja = if d.odabran { b.na_tintu } else { b.tekst };
        let f = if d.otvoriv.is_some() { srednje(14.0) } else { obicno(14.0) };
        p.text(pos2(tx, y + 15.0), Align2::LEFT_CENTER, d.naziv, f, boja);
        p.text(pos2(tx, y + 33.0), Align2::LEFT_CENTER, d.opis, obicno(12.0), if d.odabran { b.na_tintu } else { b.sivo });
    }

    fn kontrole(&self, p: &Painter, b: &Boje, x: f32, gore: f32, dolje: f32) {
        let o = Rect::from_min_size(pos2(x, gore), vec2(40.0, 81.0));
        sjena(p, b, o, 8.0);
        p.rect_filled(o, 8.0, b.ploha);
        ikona(p, "zaslon", pos2(x + 10.0, gore + 10.0), 20.0, b.sivo);
        razdjelnik(p, b, x, x + 40.0, gore + 40.0);
        ikona(p, "ispis", pos2(x + 10.0, gore + 51.0), 20.0, b.sivo);

        let d = Rect::from_min_size(pos2(x, dolje - 129.0), vec2(40.0, 40.0));
        sjena(p, b, d, 8.0);
        p.rect_filled(d, 8.0, b.ploha);
        ikona(p, "sjever", pos2(x + 10.0, d.min.y + 10.0), 20.0, b.sivo);
        let z = Rect::from_min_size(pos2(x, dolje - 81.0), vec2(40.0, 81.0));
        sjena(p, b, z, 8.0);
        p.rect_filled(z, 8.0, b.ploha);
        ikona(p, "plus", pos2(x + 10.0, z.min.y + 10.0), 20.0, b.sivo);
        razdjelnik(p, b, x, x + 40.0, z.min.y + 40.0);
        ikona(p, "minus", pos2(x + 10.0, z.min.y + 51.0), 20.0, b.sivo);
    }

    fn kartica(&self, ui: &mut Ui, b: &Boje, x: f32, y: f32) {
        let p = ui.painter().clone();
        let o = Rect::from_min_size(pos2(x, y), vec2(440.0, 152.0));
        sjena(&p, b, o, 16.0);
        p.rect_filled(o, 16.0, b.ploha);
        ikona(&p, "poligon", pos2(x + 16.0, y + 14.0), 22.0, b.crtez);
        p.text(pos2(x + 48.0, y + 24.0), Align2::LEFT_CENTER, "Parcela 1124/3", srednje(16.0), b.tekst);
        p.text(pos2(x + 48.0, y + 42.0), Align2::LEFT_CENTER, "Poligon · sloj Parcele, mapa Teren 2026", obicno(12.0), b.sivo);
        ikona(&p, "x", pos2(o.max.x - 39.0, y + 19.0), 18.0, b.sivo);
        let ha = povrsina_ha(&PARCELA);
        let m = opseg_m(&PARCELA);
        let mjere = [("Površina", format!("{ha:.2} ha").replace('.', ",")), ("Opseg", format!("{m:.0} m")), ("Vrhova", "6".to_string()), ("Kultura", "oranica".to_string())];
        let mut mx = x + 16.0;
        for (a, v) in &mjere {
            p.text(pos2(mx, y + 68.0), Align2::LEFT_CENTER, *a, obicno(12.0), b.sivo);
            p.text(pos2(mx, y + 87.0), Align2::LEFT_CENTER, v, obicno(14.0), b.tekst);
            mx += 79.0;
        }
        for (t, d) in [("Uredi", 28.0), ("Izvezi", 88.0), ("Atributi", 151.0)] {
            p.text(pos2(o.max.x - d, y + 126.0), Align2::RIGHT_CENTER, t, srednje(14.0), b.glavna);
        }
    }

    fn snimi(&mut self, ctx: &egui::Context) {
        let Some(put) = self.snimka.clone() else { return };
        self.okvir += 1;
        ctx.request_repaint();
        if self.okvir == 8 {
            let spremnik = self.slika.clone();
            let povratak = ScreenshotCallback::new(move |s| {
                if let Ok(mut m) = spremnik.lock() {
                    *m = Some(s);
                }
            });
            ctx.send_viewport_cmd(ViewportCommand::Screenshot(povratak));
        }
        let gotova = self.slika.lock().ok().and_then(|mut m| m.take());
        if let Some(s) = gotova {
            let [w, h] = s.size;
            let bajtovi: Vec<u8> = s.pixels.iter().flat_map(|p| p.to_array()).collect();
            match image::save_buffer(&put, &bajtovi, w as u32, h as u32, image::ColorType::Rgba8) {
                Ok(()) => println!("snimljeno {put} {w}x{h}, mjera zaslona {}", ctx.pixels_per_point()),
                Err(g) => eprintln!("snimka nije spremljena: {g}"),
            }
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

impl eframe::App for Proba {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let b = if self.tema_tamna { TAMNA } else { SVIJETLA };
        let c = ctx.content_rect();
        let meni = Rect::from_min_size(c.min, vec2(c.width(), MENI));
        let status = Rect::from_min_max(pos2(c.min.x, c.max.y - STATUS), c.max);
        let k = Rect::from_min_max(pos2(c.min.x, meni.max.y), pos2(c.max.x, status.min.y));

        let p = ui.painter().clone();
        karta(&p.with_clip_rect(k), &b, k);

        let ploca_x = if self.ploca_lijevo { k.min.x + RUB } else { k.max.x - RUB - PLOCA };
        let alati_x = if self.ploca_lijevo { k.min.x + RUB + PLOCA + 12.0 } else { k.min.x + RUB };

        let tr = Rect::from_min_size(pos2(alati_x, k.min.y + RUB), vec2(300.0, 44.0));
        sjena(&p, &b, tr, 22.0);
        p.rect_filled(tr, 22.0, b.ploha);
        ikona(&p, "trazi", pos2(tr.min.x + 14.0, tr.min.y + 12.0), 20.0, b.sivo);
        p.text(pos2(tr.min.x + 44.0, tr.center().y), Align2::LEFT_CENTER, "Traži mjesto ili koordinate", obicno(14.0), b.blijedo);

        self.alati(ui, &b, alati_x, k.min.y + RUB + 56.0);

        let kontrole_x = if self.ploca_lijevo { k.max.x - RUB - 40.0 } else { k.max.x - RUB - PLOCA - 12.0 - 40.0 };
        self.kontrole(&p, &b, kontrole_x, k.min.y + RUB, k.max.y - RUB);
        let kartica_x = if self.ploca_lijevo { 660.0 } else { 330.0 };
        self.kartica(ui, &b, kartica_x, k.max.y - 188.0);

        let sx = if self.ploca_lijevo { alati_x } else { k.min.x + RUB };
        let d = 100.0 / M_PO_PX;
        let (d1, y1) = (sx + 4.0, k.max.y - 26.0);
        p.rect_filled(Rect::from_min_size(pos2(d1, y1), vec2(d, 2.0)), 0.0, b.tekst);
        p.rect_filled(Rect::from_min_size(pos2(d1, y1 - 6.0), vec2(2.0, 8.0)), 0.0, b.tekst);
        p.rect_filled(Rect::from_min_size(pos2(d1 + d - 2.0, y1 - 6.0), vec2(2.0, 8.0)), 0.0, b.tekst);
        p.text(pos2(d1 + 4.0, y1 - 13.0), Align2::LEFT_CENTER, "100 m", obicno(12.0), b.tekst);

        let zasluge_x = if self.ploca_lijevo { k.max.x - 300.0 } else { 780.0 };
        p.text(pos2(zasluge_x, k.max.y - 16.0), Align2::LEFT_CENTER, "© OpenFreeMap · OpenStreetMap suradnici", obicno(11.0), b.tekst);

        let ploca = Rect::from_min_size(pos2(ploca_x, k.min.y + RUB), vec2(PLOCA, k.height() - 2.0 * RUB));
        self.ploca(ui, &b, ploca, c.center().x);

        self.traka_izbornika(ui, &b, meni);
        self.statusna(&p, &b, status);

        self.snimi(&ctx);
    }
}

fn fontovi() -> FontDefinitions {
    let mut f = FontDefinitions::default();
    let mut dodaj = |ime: &str, bajtovi: &'static [u8]| {
        f.font_data.insert(ime.to_string(), Arc::new(FontData::from_static(bajtovi)));
    };
    dodaj("Inter", include_bytes!("../fonti/Inter-Regular.otf"));
    dodaj("InterSrednji", include_bytes!("../fonti/Inter-Medium.otf"));
    dodaj("InterKurziv", include_bytes!("../fonti/Inter-MediumItalic.otf"));
    if let Some(v) = f.families.get_mut(&FontFamily::Proportional) {
        v.insert(0, "Inter".to_string());
    }
    f.families.insert(FontFamily::Name("Srednji".into()), vec!["InterSrednji".to_string(), "Inter".to_string()]);
    f.families.insert(FontFamily::Name("Kurziv".into()), vec!["InterKurziv".to_string(), "Inter".to_string()]);
    f
}

fn main() -> eframe::Result {
    let argumenti: Vec<String> = std::env::args().collect();
    let snimka = argumenti.iter().position(|a| a == "--snimka").and_then(|i| argumenti.get(i + 1)).cloned();
    let tamna = argumenti.iter().any(|a| a == "--tamna");
    let lijevo = argumenti.iter().any(|a| a == "--lijevo");
    let opcije = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 900.0]).with_title("TerraSimple, proba izgleda"),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "TerraSimple proba",
        opcije,
        Box::new(move |cc| {
            cc.egui_ctx.set_fonts(fontovi());
            Ok(Box::new(Proba {
                tema_tamna: tamna,
                alat: 0,
                ploca_lijevo: lijevo,
                iskljuceni: HashSet::new(),
                zatvoreni: HashSet::new(),
                podloga: 0,
                snimka,
                slika: Arc::new(Mutex::new(None)),
                okvir: 0,
            }))
        }),
    )
}
