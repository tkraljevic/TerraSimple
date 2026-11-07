//! Ono što se mijenja dok korisnik radi: crtež, odabrani objekt i točke za hvatanje. Crta se kroz
//! `egui`, jer su to mala količina i svaki okvir drukčiji podaci.

use crate::crtanje::{Alat, Crtanje, Crtez, Okolina, Pretvorba, je_lik, prstenovi};
use eframe::egui::{self, Color32, FontId, Mesh, Painter, Pos2, Rect, Shape, Stroke, pos2, vec2};
use lyon_tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon_tessellation::math::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{FillOptions, FillRule, FillTessellator, FillVertex};
use slojevi::{Geometrija, Projekt, Vrh};

const NARANCASTA: Color32 = Color32::from_rgb(0xE8, 0x59, 0x0C);
const ISPUNA: Color32 = Color32::from_rgba_premultiplied(58, 22, 3, 64);
const TAMNO: Color32 = Color32::from_rgb(0x1E, 0x1F, 0x23);

fn u_tocke(vrhovi: &[Vrh], p: &dyn Pretvorba) -> Vec<Pos2> {
    vrhovi.iter().map(|v| p.u_ekran(v)).collect()
}

/// Puni i udubljene likove, jer sam `egui` zna samo izbočene.
fn ispuna(s: &Painter, prstenovi: &[Vec<Pos2>]) {
    let mut b = Path::builder();
    for t in prstenovi.iter().filter(|t| t.len() >= 3) {
        b.begin(point(t[0].x, t[0].y));
        t[1..].iter().for_each(|m| {
            b.line_to(point(m.x, m.y));
        });
        b.end(true);
    }
    let put = b.build();
    let mut geo: VertexBuffers<[f32; 2], u32> = VertexBuffers::new();
    let opcije = FillOptions::default().with_fill_rule(FillRule::EvenOdd);
    let mut gradnja = BuffersBuilder::new(&mut geo, |v: FillVertex| v.position().to_array());
    // Lik koji se ne da popuniti ostaje samo crta.
    if FillTessellator::new()
        .tessellate_path(&put, &opcije, &mut gradnja)
        .is_ok()
    {
        let mut m = Mesh::default();
        geo.vertices
            .iter()
            .for_each(|v| m.colored_vertex(pos2(v[0], v[1]), ISPUNA));
        m.indices = geo.indices;
        s.add(Shape::mesh(m));
    }
}

fn nacrtaj_geometriju(s: &Painter, g: &Geometrija, p: &dyn Pretvorba) {
    let crta = Stroke::new(3.0, NARANCASTA);
    let t: Vec<Vec<Pos2>> = prstenovi(g).into_iter().map(|r| u_tocke(r, p)).collect();
    match g {
        Geometrija::Tocka(_) => {
            t.iter().flatten().for_each(|m| {
                s.circle(*m, 7.0, NARANCASTA, Stroke::new(1.5, Color32::WHITE));
            });
        }
        Geometrija::Linija(_) => {
            t.into_iter().for_each(|r| {
                s.add(Shape::line(r, crta));
            });
        }
        Geometrija::Poligon(_) => {
            ispuna(s, &t);
            t.into_iter().for_each(|r| {
                s.add(Shape::closed_line(r, crta));
            });
        }
    }
}

fn nacrtaj_crtez(s: &Painter, c: &Crtez, p: &dyn Pretvorba, mis: Option<Pos2>) {
    let t = u_tocke(&c.vrhovi, p);
    let lik = je_lik(c);
    if lik && t.len() >= 2 {
        let mut oko = t.clone();
        oko.extend(mis);
        ispuna(s, &[oko]);
    }
    s.add(Shape::line(t.clone(), Stroke::new(2.5, NARANCASTA)));
    if let (Some(zadnja), Some(m)) = (t.last(), mis) {
        let mut crtica = vec![*zadnja, m];
        if lik && t.len() >= 2 {
            crtica.push(t[0]);
        }
        s.extend(Shape::dashed_line(
            &crtica,
            Stroke::new(1.75, Color32::WHITE),
            6.0,
            5.0,
        ));
    }
}

/// Mjerenje je isprekidano i tamno, da se ne može zamijeniti s nacrtanim objektom. Ispod je svijetli
/// obrub, jer se samo tamna crta na ortofotu slabo vidi.
fn nacrtaj_mjerenje(s: &Painter, c: &Crtez, p: &dyn Pretvorba, mis: Option<Pos2>) {
    let mut t = u_tocke(&c.vrhovi, p);
    t.extend(mis.filter(|_| !c.gotov));
    if je_lik(c) && t.len() >= 3 {
        t.extend(t.first().copied());
    }
    s.extend(Shape::dashed_line(
        &t,
        Stroke::new(3.5, Color32::WHITE),
        7.0,
        4.0,
    ));
    s.extend(Shape::dashed_line(&t, Stroke::new(2.0, TAMNO), 7.0, 4.0));
}

fn nacrtaj_oznaku(s: &Painter, sredina: Pos2, tekst: String) {
    let g = s.layout_no_wrap(tekst, FontId::proportional(11.0), Color32::WHITE);
    let velicina = vec2(g.size().x + 12.0, g.size().y + 6.0);
    let o = Rect::from_center_size(sredina, velicina);
    s.rect_filled(o, 4.0, TAMNO);
    s.galley(pos2(o.min.x + 6.0, o.min.y + 3.0), g, Color32::WHITE);
}

fn nacrtaj_natpis(s: &Painter, m: Pos2, tekst: String) {
    let g = s.layout_no_wrap(tekst, FontId::proportional(13.0), Color32::WHITE);
    let o = Rect::from_min_size(pos2(m.x + 14.0, m.y - 26.0), vec2(g.size().x + 16.0, 24.0));
    s.rect_filled(o, 6.0, TAMNO);
    s.galley(
        pos2(o.min.x + 8.0, o.center().y - g.size().y / 2.0),
        g,
        Color32::WHITE,
    );
}

fn nacrtaj_trenutni(
    s: &Painter,
    c: &Crtanje,
    crtez: &Crtez,
    p: &dyn Pretvorba,
    mis: Option<Pos2>,
    ok: &Okolina<'_>,
) {
    let na_karti = mis.map(|m| p.na_kartu(m));
    if crtez.alat == Alat::Mjeri {
        nacrtaj_mjerenje(s, crtez, p, mis);
        for (sredina, duljina) in c.dijelovi_mjerenja(na_karti, p) {
            nacrtaj_oznaku(s, sredina, (ok.opis_duljine)(duljina));
        }
        return;
    }
    nacrtaj_crtez(s, crtez, p, mis);
    let polozaj = mis.or(crtez.vrhovi.last().map(|v| p.u_ekran(v)));
    if let (Some(mjere), Some(polozaj)) = (c.mjere_crteza(na_karti), polozaj) {
        nacrtaj_natpis(s, polozaj, (ok.opis_mjere)(&mjere));
    }
}

/// Sve što se crta preko slojeva.
pub(crate) fn nacrtaj(
    s: &Painter,
    c: &Crtanje,
    projekt: &Projekt,
    p: &dyn Pretvorba,
    mis: Option<Pos2>,
    ok: &Okolina<'_>,
) {
    let odabrano = c.odabran().and_then(|(sloj, o)| projekt.objekt(sloj, o));
    if let Some(g) = c.pregled().or(odabrano.map(|o| &o.geometrija)) {
        nacrtaj_geometriju(s, g, p);
    }
    if let Some(crtez) = &c.crtez {
        nacrtaj_trenutni(s, c, crtez, p, mis, ok);
    }
    for h in &c.hvatista {
        if h.srednje {
            s.circle(
                h.mjesto,
                4.0,
                Color32::from_white_alpha(179),
                Stroke::new(1.0, NARANCASTA),
            );
        } else {
            s.circle(h.mjesto, 5.5, Color32::WHITE, Stroke::new(2.0, NARANCASTA));
        }
    }
}

/// Oznaka izvora podloge, koja mora biti vidljiva, u donjem desnom kutu.
pub(crate) fn nacrtaj_navod(s: &Painter, okvir: Rect, navod: &str) {
    let g = s.layout_no_wrap(navod.to_string(), FontId::proportional(11.0), TAMNO);
    let o = Rect::from_min_size(
        pos2(
            okvir.max.x - g.size().x - 12.0,
            okvir.max.y - g.size().y - 8.0,
        ),
        vec2(g.size().x + 12.0, g.size().y + 8.0),
    );
    s.rect_filled(o, egui::CornerRadius::ZERO, Color32::from_white_alpha(200));
    s.galley(pos2(o.min.x + 6.0, o.min.y + 4.0), g, TAMNO);
}
