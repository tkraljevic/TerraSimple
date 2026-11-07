//! Pravila crtanja i iscrtavanje objekata. Ovo je ono što proba zapravo provjerava.

use eframe::egui::{self, Color32, FontId, Mesh, Modifiers, Pos2, Rect, Response, Shape, Stroke, Ui, pos2, vec2};
use geographiclib_rs::{Geodesic, InverseGeodesic, PolygonArea, Winding};
use lyon::math::point;
use lyon::path::Path;
use lyon::tessellation::{BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers};
use walkers::{Position, Projector, lon_lat};

const NARANCASTA: Color32 = Color32::from_rgb(0xE8, 0x59, 0x0C);
const POLUMJER_HVATA: f32 = 9.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Alat {
    #[default]
    Odaberi,
    Tocka,
    Linija,
    Poligon,
}

#[derive(Clone)]
pub struct Objekt {
    pub vrsta: Alat,
    pub tocke: Vec<Position>,
}

impl Objekt {
    /// Lik za mjerenje s n vrhova oko sredine.
    pub fn zvijezda(s: Position, n: usize, ostra: bool) -> Self {
        let tocke = (0..n)
            .map(|i| {
                let a = i as f64 / n as f64 * std::f64::consts::TAU;
                // Valovita granica, nalik pravoj parceli ili obuhvatu. Oštra zvijezda s dugim
                // šiljcima nije stvaran lik, a s njom se 10 000 vrhova nije dalo mjeriti.
                let r = if ostra { if i % 2 == 0 { 0.012 } else { 0.010 } } else { 0.011 + 0.0006 * (a * 37.0).sin() + 0.0002 * (a * 211.0).sin() };
                lon_lat(s.x() + r * 1.4 * a.cos(), s.y() + r * a.sin())
            })
            .collect();
        Self { vrsta: Alat::Poligon, tocke }
    }

    fn najmanje_tocaka(&self) -> usize {
        match self.vrsta {
            Alat::Poligon => 3,
            Alat::Linija => 2,
            _ => 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cilj {
    Crtez,
    Objekt(usize),
}

#[derive(Clone, Copy, Debug)]
pub struct Hvatiste {
    pub cilj: Cilj,
    pub i: usize,
    pub srednje: bool,
    pub mjesto: Pos2,
}

#[derive(Default)]
pub struct Crtanje {
    pub alat: Alat,
    pub objekti: Vec<Objekt>,
    pub crtez: Option<Objekt>,
    pub odabran: Option<usize>,
    pub vuce: Option<(Cilj, usize)>,
    pub hvatista: Vec<Hvatiste>,
    povijest: Vec<Vec<Objekt>>,
    poruka: String,
}

impl Crtanje {
    pub fn poruka(&self) -> String {
        if !self.poruka.is_empty() {
            return self.poruka.clone();
        }
        match self.alat {
            Alat::Odaberi => "Klikni objekt. Povuci točku, a desni klik je briše.".to_string(),
            Alat::Tocka => "Klik dodaje točku.".to_string(),
            _ => "Klik dodaje točku, povlačenje je pomiče, desni klik briše, dvoklik završava.".to_string(),
        }
    }

    /// Točka pod mišem, gledano ondje gdje je tipka pritisnuta. Inače bi brzi potez
    /// promašio točku i umjesto nje pomaknuo kartu.
    fn pod_misem(&self, ctx: &egui::Context) -> Option<Hvatiste> {
        let m = ctx.input(|i| i.pointer.press_origin().or(i.pointer.latest_pos()))?;
        self.najblize(m)
    }

    pub fn najblize(&self, m: Pos2) -> Option<Hvatiste> {
        let mut naj: Option<(f32, Hvatiste)> = None;
        for h in &self.hvatista {
            // Prava točka ima prednost pred srednjom kad su jako blizu.
            let d = h.mjesto.distance(m) + if h.srednje { 2.0 } else { 0.0 };
            if d <= POLUMJER_HVATA && naj.is_none_or(|(b, _)| d < b) {
                naj = Some((d, *h));
            }
        }
        naj.map(|(_, h)| h)
    }

    pub fn smije_pomicati_kartu(&self, ctx: &egui::Context) -> bool {
        self.vuce.is_none() && self.pod_misem(ctx).is_none()
    }

    pub fn tipkovnica(&mut self, ctx: &egui::Context) {
        let (esc, nazad, ponisti) = ctx.input_mut(|i| {
            (
                i.consume_key(Modifiers::NONE, egui::Key::Escape),
                i.consume_key(Modifiers::NONE, egui::Key::Backspace),
                i.consume_key(Modifiers::COMMAND, egui::Key::Z),
            )
        });
        if esc {
            self.crtez = None;
            self.poruka.clear();
        }
        if nazad {
            if let Some(c) = &mut self.crtez {
                c.tocke.pop();
                if c.tocke.is_empty() {
                    self.crtez = None;
                }
            }
        }
        if ponisti {
            if let Some(p) = self.povijest.pop() {
                self.objekti = p;
                self.odabran = self.odabran.filter(|i| *i < self.objekti.len());
            }
        }
    }

    fn zapamti(&mut self) {
        self.povijest.push(self.objekti.clone());
    }

    fn tocke_cilja(&mut self, c: Cilj) -> Option<&mut Vec<Position>> {
        match c {
            Cilj::Crtez => self.crtez.as_mut().map(|o| &mut o.tocke),
            Cilj::Objekt(i) => self.objekti.get_mut(i).map(|o| &mut o.tocke),
        }
    }

    pub fn kadar(&mut self, ui: &mut Ui, o: &Response, p: &Projector) {
        let ctx = ui.ctx().clone();
        self.mis(&ctx, o, p);
        self.iscrtaj(ui, p, &ctx);
    }

    fn mis(&mut self, ctx: &egui::Context, o: &Response, p: &Projector) {
        let tu = ctx.input(|i| i.pointer.latest_pos());
        let ispod = self.pod_misem(ctx);

        if o.drag_started_by(egui::PointerButton::Primary) {
            if let Some(h) = ispod {
                if matches!(h.cilj, Cilj::Objekt(_)) {
                    self.zapamti();
                }
                let mut i = h.i;
                if h.srednje {
                    i += 1;
                    if let (Some(t), Some(m)) = (self.tocke_cilja(h.cilj), tu) {
                        t.insert(i, p.unproject(m.to_vec2()));
                    }
                }
                self.vuce = Some((h.cilj, i));
            }
        }
        if let Some((c, i)) = self.vuce {
            if let Some(m) = tu {
                let nova = p.unproject(m.to_vec2());
                if let Some(t) = self.tocke_cilja(c).and_then(|t| t.get_mut(i)) {
                    *t = nova;
                }
            }
            if o.drag_stopped() || !ctx.input(|i| i.pointer.primary_down()) {
                self.vuce = None;
            }
        }

        // Dvoklik je i klik, pa se mora gledati prvi. Prvi klik para već je dodao točku.
        if o.double_clicked_by(egui::PointerButton::Primary) {
            self.zavrsi();
        } else if o.clicked_by(egui::PointerButton::Primary) {
            if let Some(m) = tu {
                self.klik(m, ispod, p);
            }
        }
        if o.clicked_by(egui::PointerButton::Secondary) {
            if let Some(h) = ispod.filter(|h| !h.srednje) {
                self.obrisi_tocku(h);
            }
        }
    }

    fn klik(&mut self, m: Pos2, ispod: Option<Hvatiste>, p: &Projector) {
        self.poruka.clear();
        let g = p.unproject(m.to_vec2());
        match self.alat {
            Alat::Odaberi => self.odabir(m, p),
            Alat::Tocka => {
                self.zapamti();
                self.objekti.push(Objekt { vrsta: Alat::Tocka, tocke: vec![g] });
                self.odabran = Some(self.objekti.len() - 1);
            }
            Alat::Linija | Alat::Poligon => {
                if let Some(h) = ispod.filter(|h| h.cilj == Cilj::Crtez && !h.srednje) {
                    // Klik na prvu točku poligona zatvara lik, a ostale ne dodaju ništa.
                    if h.i == 0 && self.alat == Alat::Poligon {
                        self.zavrsi();
                    }
                    return;
                }
                let vrsta = self.alat;
                self.crtez.get_or_insert_with(|| Objekt { vrsta, tocke: Vec::new() }).tocke.push(g);
            }
        }
    }

    fn zavrsi(&mut self) {
        let Some(c) = self.crtez.take() else { return };
        if c.tocke.len() < c.najmanje_tocaka() {
            self.poruka = format!("Za ovaj lik trebaju barem {} točke.", c.najmanje_tocaka());
            self.crtez = Some(c);
            return;
        }
        self.zapamti();
        self.objekti.push(c);
        self.odabran = Some(self.objekti.len() - 1);
        self.alat = Alat::Odaberi;
    }

    fn obrisi_tocku(&mut self, h: Hvatiste) {
        match h.cilj {
            Cilj::Crtez => {
                if let Some(c) = &mut self.crtez {
                    if h.i < c.tocke.len() {
                        c.tocke.remove(h.i);
                    }
                    if c.tocke.is_empty() {
                        self.crtez = None;
                    }
                }
            }
            Cilj::Objekt(i) => {
                let Some(o) = self.objekti.get(i) else { return };
                let najmanje = o.najmanje_tocaka();
                if o.tocke.len() <= najmanje {
                    self.poruka = format!("Ovaj objekt mora imati barem {najmanje} točke.");
                    return;
                }
                self.zapamti();
                if let Some(o) = self.objekti.get_mut(i) {
                    o.tocke.remove(h.i);
                }
            }
        }
    }

    fn odabir(&mut self, m: Pos2, p: &Projector) {
        self.odabran = None;
        for (i, o) in self.objekti.iter().enumerate().rev() {
            let t: Vec<Pos2> = o.tocke.iter().map(|g| p.project(*g).to_pos2()).collect();
            let pogodak = match o.vrsta {
                Alat::Poligon => unutar(&t, m),
                Alat::Linija => t.windows(2).any(|d| udaljenost_do_duzine(m, d[0], d[1]) < 6.0),
                _ => t.first().is_some_and(|c| c.distance(m) < 12.0),
            };
            if pogodak {
                self.odabran = Some(i);
                return;
            }
        }
    }

    fn iscrtaj(&mut self, ui: &mut Ui, p: &Projector, ctx: &egui::Context) {
        let slikar = ui.painter().clone();
        let mut hvatista = Vec::new();

        for (i, o) in self.objekti.iter().enumerate() {
            let t: Vec<Pos2> = o.tocke.iter().map(|g| p.project(*g).to_pos2()).collect();
            let odabran = self.odabran == Some(i);
            nacrtaj_objekt(&slikar, o.vrsta, &t, odabran);
            // Dok traje crtanje, tuđe točke se ne hvataju, da se susjedni lik može crtati uz njih.
            if odabran && self.crtez.is_none() {
                dodaj_hvatista(&mut hvatista, Cilj::Objekt(i), &t, o.vrsta, true);
            }
        }

        let mis = ctx.input(|i| i.pointer.latest_pos());
        if let Some(c) = &self.crtez {
            let t: Vec<Pos2> = c.tocke.iter().map(|g| p.project(*g).to_pos2()).collect();
            if c.vrsta == Alat::Poligon && t.len() >= 2 {
                let mut lik = t.clone();
                lik.extend(mis);
                ispuna(&slikar, &lik);
            }
            slikar.add(Shape::line(t.clone(), Stroke::new(2.5, NARANCASTA)));
            if let (Some(zadnja), Some(m)) = (t.last(), mis) {
                let mut crtica = vec![*zadnja, m];
                if c.vrsta == Alat::Poligon && t.len() >= 2 {
                    crtica.push(t[0]);
                }
                slikar.extend(Shape::dashed_line(&crtica, Stroke::new(1.75, Color32::WHITE), 6.0, 5.0));
            }
            dodaj_hvatista(&mut hvatista, Cilj::Crtez, &t, c.vrsta, false);
            if let Some(m) = mis {
                let mut g = c.tocke.clone();
                g.push(p.unproject(m.to_vec2()));
                natpis_mjere(&slikar, m, &opis_mjere(c.vrsta, &g));
            }
        }

        for h in &hvatista {
            if h.srednje {
                slikar.circle(h.mjesto, 4.0, Color32::from_white_alpha(179), Stroke::new(1.0, NARANCASTA));
            } else {
                slikar.circle(h.mjesto, 5.5, Color32::WHITE, Stroke::new(2.0, NARANCASTA));
            }
        }
        self.hvatista = hvatista;
    }
}

fn dodaj_hvatista(v: &mut Vec<Hvatiste>, cilj: Cilj, t: &[Pos2], vrsta: Alat, srednje: bool) {
    for (i, m) in t.iter().enumerate() {
        v.push(Hvatiste { cilj, i, srednje: false, mjesto: *m });
    }
    if !srednje {
        return;
    }
    let n = t.len();
    let koliko = if vrsta == Alat::Poligon { n } else { n.saturating_sub(1) };
    if vrsta == Alat::Tocka {
        return;
    }
    for i in 0..koliko {
        let (a, b) = (t[i], t[(i + 1) % n]);
        v.push(Hvatiste { cilj, i, srednje: true, mjesto: pos2((a.x + b.x) / 2.0, (a.y + b.y) / 2.0) });
    }
}

fn nacrtaj_objekt(s: &egui::Painter, vrsta: Alat, t: &[Pos2], odabran: bool) {
    let debljina = if odabran { 3.0 } else { 1.75 };
    match vrsta {
        Alat::Poligon => {
            ispuna(s, t);
            s.add(Shape::closed_line(t.to_vec(), Stroke::new(debljina, NARANCASTA)));
        }
        Alat::Linija => {
            s.add(Shape::line(t.to_vec(), Stroke::new(debljina, Color32::from_rgb(0x15, 0x65, 0xC0))));
        }
        _ => {
            if let Some(c) = t.first() {
                s.circle(*c, 7.0, Color32::from_rgb(0xC6, 0x28, 0x28), Stroke::new(1.5, Color32::WHITE));
            }
        }
    }
}

/// Puni i udubljene likove. Sam egui zna samo izbočene.
fn ispuna(s: &egui::Painter, t: &[Pos2]) {
    if t.len() < 3 {
        return;
    }
    let mut b = Path::builder();
    b.begin(point(t[0].x, t[0].y));
    for m in &t[1..] {
        b.line_to(point(m.x, m.y));
    }
    b.end(true);
    let put = b.build();
    let mut geo: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let gotovo = FillTessellator::new().tessellate_path(&put, &FillOptions::default(), &mut BuffersBuilder::new(&mut geo, |v: FillVertex| v.position().to_array()));
    if gotovo.is_err() {
        return; // Lik koji se ne da popuniti ostaje samo crta.
    }
    let boja = Color32::from_rgba_unmultiplied(0xE8, 0x59, 0x0C, 64);
    let mut m = Mesh::default();
    for v in &geo.vertices {
        m.colored_vertex(pos2(v[0], v[1]), boja);
    }
    m.indices = geo.indices;
    s.add(Shape::mesh(m));
}

fn natpis_mjere(s: &egui::Painter, m: Pos2, t: &str) {
    let g = s.layout_no_wrap(t.to_string(), FontId::proportional(13.0), Color32::WHITE);
    let o = Rect::from_min_size(pos2(m.x + 14.0, m.y - 26.0), vec2(g.size().x + 16.0, 24.0));
    s.rect_filled(o, 6.0, Color32::from_rgb(0x1E, 0x1F, 0x23));
    s.galley(pos2(o.min.x + 8.0, o.center().y - g.size().y / 2.0), g, Color32::WHITE);
}

pub fn duljina_m(t: &[Position]) -> f64 {
    let g = Geodesic::wgs84();
    t.windows(2)
        .map(|d| {
            let s: f64 = g.inverse(d[0].y(), d[0].x(), d[1].y(), d[1].x());
            s
        })
        .sum()
}

/// Opseg u metrima i površina u hektarima, na elipsoidu WGS84.
pub fn opseg_i_povrsina(t: &[Position]) -> (f64, f64) {
    let g = Geodesic::wgs84();
    let mut p = PolygonArea::new(&g, Winding::CounterClockwise);
    for m in t {
        p.add_point(m.y(), m.x());
    }
    // Bez predznaka bi lik nacrtan u smjeru kazaljke dao površinu ostatka Zemlje.
    let (opseg, povrsina, _) = p.compute(true);
    (opseg, povrsina.abs() / 10000.0)
}

/// Grubi račun u ravnini, neovisan o biblioteci, samo za usporedbu u provjerama.
pub fn povrsina_u_ravnini_ha(t: &[Position]) -> f64 {
    let Some(prva) = t.first() else { return 0.0 };
    let cos = prva.y().to_radians().cos();
    let xy: Vec<(f64, f64)> = t.iter().map(|m| (m.x() * 111_320.0 * cos, m.y() * 111_132.0)).collect();
    let n = xy.len();
    let z: f64 = (0..n).map(|i| xy[i].0 * xy[(i + 1) % n].1 - xy[(i + 1) % n].0 * xy[i].1).sum();
    z.abs() / 2.0 / 10000.0
}

#[cfg(test)]
mod testovi {
    use super::*;

    fn kvadrat() -> Vec<Position> {
        // Oko 1 km x 1 km kod Osijeka.
        let (dx, dy) = (1000.0 / (111_320.0 * 45.55_f64.to_radians().cos()), 1000.0 / 111_132.0);
        vec![lon_lat(18.69, 45.55), lon_lat(18.69 + dx, 45.55), lon_lat(18.69 + dx, 45.55 + dy), lon_lat(18.69, 45.55 + dy)]
    }

    #[test]
    fn smjer_crtanja_ne_mijenja_povrsinu() {
        let mut t = kvadrat();
        let (o1, p1) = opseg_i_povrsina(&t);
        t.reverse();
        let (o2, p2) = opseg_i_povrsina(&t);
        assert!((p1 - p2).abs() < 1e-6, "{p1} i {p2}");
        assert!((o1 - o2).abs() < 1e-6);
    }

    #[test]
    fn kvadrat_od_kilometra_ima_oko_sto_hektara() {
        for smjer in [false, true] {
            let mut t = kvadrat();
            if smjer {
                t.reverse();
            }
            let (o, p) = opseg_i_povrsina(&t);
            assert!((p - 100.0).abs() < 1.0, "površina {p} ha");
            assert!((o - 4000.0).abs() < 20.0, "opseg {o} m");
        }
    }
}

fn opis_mjere(vrsta: Alat, g: &[Position]) -> String {
    if vrsta == Alat::Poligon && g.len() >= 3 {
        let (o, p) = opseg_i_povrsina(g);
        format!("{p:.2} ha · {o:.0} m").replace('.', ",")
    } else {
        format!("{:.0} m", duljina_m(g))
    }
}

fn unutar(t: &[Pos2], m: Pos2) -> bool {
    let mut u = false;
    let mut j = t.len().wrapping_sub(1);
    for i in 0..t.len() {
        let (a, b) = (t[i], t[j]);
        if (a.y > m.y) != (b.y > m.y) && m.x < (b.x - a.x) * (m.y - a.y) / (b.y - a.y) + a.x {
            u = !u;
        }
        j = i;
    }
    u
}

fn udaljenost_do_duzine(m: Pos2, a: Pos2, b: Pos2) -> f32 {
    let d = b - a;
    let duljina = d.length_sq();
    if duljina == 0.0 {
        return m.distance(a);
    }
    let u = ((m - a).dot(d) / duljina).clamp(0.0, 1.0);
    m.distance(a + d * u)
}
