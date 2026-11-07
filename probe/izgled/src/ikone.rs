//! Ikone iz nacrta (stil Lucide, mreža 24 x 24), iste kao u dizajn/skripte/temelj.py.

use eframe::egui::{Color32, Painter, Pos2, Rect, Stroke, StrokeKind, pos2, vec2};

use crate::crtanje::{crta, procitaj_put};

pub enum Dio {
    P(&'static str),
    K(f32, f32, f32),
    E(f32, f32, f32, f32),
    R(f32, f32, f32, f32, f32),
}
use Dio::{E, K, P, R};

const MAPA: &str = "M4 5 L9 5 L11 8 L20 8 C21 8 22 9 22 10 L22 18 C22 19 21 20 20 20 L4 20 C3 20 2 19 2 18 L2 7 C2 6 3 5 4 5 Z";

pub fn dijelovi(ime: &str) -> Vec<Dio> {
    match ime {
        "trazi" => vec![K(11.0, 11.0, 8.0), P("M21 21 L16.7 16.7")],
        "x" => vec![P("M18 6 L6 18"), P("M6 6 L18 18")],
        "desno" => vec![P("M9 6 L15 12 L9 18")],
        "dolje" => vec![P("M6 9 L12 15 L18 9")],
        "mapa" => vec![P(MAPA)],
        "nova_mapa" => vec![P(MAPA), P("M12 11 L12 17"), P("M9 14 L15 14")],
        "uvezi" => vec![P("M21 15 L21 19 C21 20 20 21 19 21 L5 21 C4 21 3 20 3 19 L3 15"), P("M7 10 L12 15 L17 10"), P("M12 15 L12 3")],
        "tocka" => vec![P("M20 10 C20 16 12 22 12 22 C12 22 4 16 4 10 C4 5.6 7.6 2 12 2 C16.4 2 20 5.6 20 10 Z"), K(12.0, 10.0, 3.0)],
        "linija" => vec![K(5.0, 19.0, 2.5), K(19.0, 5.0, 2.5), P("M6.8 17.2 L17.2 6.8")],
        "poligon" => vec![P("M12 3 L21 10 L17.5 21 L6.5 21 L3 10 Z")],
        "ravnalo" => vec![P("M2.7 15.3 L15.3 2.7 L21.3 8.7 L8.7 21.3 Z"), P("M7.5 10.5 L9.5 12.5"), P("M10.5 7.5 L13.5 10.5"), P("M13.5 4.5 L15.5 6.5"), P("M4.5 13.5 L7.5 16.5")],
        "odaberi" => vec![P("M4 4 L11 21 L13.5 13.5 L21 11 Z")],
        "zaslon" => vec![P("M8 3 L5 3 C4 3 3 4 3 5 L3 8"), P("M21 8 L21 5 C21 4 20 3 19 3 L16 3"), P("M3 16 L3 19 C3 20 4 21 5 21 L8 21"), P("M16 21 L19 21 C20 21 21 20 21 19 L21 16")],
        "ispis" => vec![P("M6 9 L6 2 L18 2 L18 9"), P("M6 18 L4 18 C3 18 2 17 2 16 L2 11 C2 10 3 9 4 9 L20 9 C21 9 22 10 22 11 L22 16 C22 17 21 18 20 18 L18 18"), R(6.0, 14.0, 12.0, 8.0, 0.0)],
        "plus" => vec![P("M12 5 L12 19"), P("M5 12 L19 12")],
        "minus" => vec![P("M5 12 L19 12")],
        "sjever" => vec![P("M12 2 L19 21 L12 17 L5 21 Z")],
        "ponisti" => vec![P("M3 7 L3 13 L9 13"), P("M3 13 C6 8 10 6.5 14 7 C18 7.5 20.5 11 21 17")],
        "ponovi" => vec![P("M21 7 L21 13 L15 13"), P("M21 13 C18 8 14 6.5 10 7 C6 7.5 3.5 11 3 17")],
        "mjesec" => vec![P("M21 12.8 C20 17 16.5 21 12 21 C7 21 3 17 3 12 C3 7.5 7 4 11.2 3 C9.5 6 10 10 12 12 C14 14 18 14.5 21 12.8 Z")],
        "datoteka" => vec![P("M14 2 L6 2 C5 2 4 3 4 4 L4 20 C4 21 5 22 6 22 L18 22 C19 22 20 21 20 20 L20 8 Z"), P("M14 2 L14 8 L20 8")],
        "baza" => vec![E(12.0, 5.0, 9.0, 3.0), P("M3 5 L3 19 C3 20.7 7 22 12 22 C17 22 21 20.7 21 19 L21 5"), P("M3 12 C3 13.7 7 15 12 15 C17 15 21 13.7 21 12")],
        "kvacica" => vec![P("M20 6 L9 17 L4 12")],
        "brava" => vec![R(3.0, 11.0, 18.0, 11.0, 2.0), P("M7 11 L7 7 C7 4 9 2 12 2 C15 2 17 4 17 7 L17 11")],
        "mreza" => vec![K(12.0, 12.0, 10.0), P("M2 12 L22 12"), P("M12 2 C15 5 16 9 16 12 C16 15 15 19 12 22 C9 19 8 15 8 12 C8 9 9 5 12 2 Z")],
        "karta" => vec![P("M9 3 L3 6 L3 21 L9 18 L15 21 L21 18 L21 3 L15 6 Z"), P("M9 3 L9 18"), P("M15 6 L15 21")],
        "pomoc" => vec![K(12.0, 12.0, 10.0), P("M9 9 C9 7 10.5 6 12 6 C13.5 6 15 7 15 9 C15 11 12 11.5 12 14"), P("M12 17.5 L12 17.6")],
        "hvataljka" => vec![K(9.0, 6.0, 1.0), K(15.0, 6.0, 1.0), K(9.0, 12.0, 1.0), K(15.0, 12.0, 1.0), K(9.0, 18.0, 1.0), K(15.0, 18.0, 1.0)],
        _ => vec![],
    }
}

pub fn ikona(p: &Painter, ime: &str, gore_lijevo: Pos2, velicina: f32, boja: Color32) {
    let s = velicina / 24.0;
    let debljina = 1.75 * velicina / 20.0;
    for d in dijelovi(ime) {
        match d {
            P(zapis) => {
                for put in procitaj_put(zapis, gore_lijevo, s) {
                    crta(p, &put.tocke, put.zatvoren, debljina, boja);
                }
            }
            K(x, y, r) => {
                p.circle_stroke(pos2(gore_lijevo.x + x * s, gore_lijevo.y + y * s), r * s, Stroke::new(debljina, boja));
            }
            E(x, y, rx, ry) => {
                let t: Vec<Pos2> = (0..32)
                    .map(|i| {
                        let a = i as f32 / 32.0 * std::f32::consts::TAU;
                        pos2(gore_lijevo.x + (x + rx * a.cos()) * s, gore_lijevo.y + (y + ry * a.sin()) * s)
                    })
                    .collect();
                crta(p, &t, true, debljina, boja);
            }
            R(x, y, w, h, r) => {
                let o = Rect::from_min_size(pos2(gore_lijevo.x + x * s, gore_lijevo.y + y * s), vec2(w * s, h * s));
                p.rect_stroke(o, r * s, Stroke::new(debljina, boja), StrokeKind::Middle);
            }
        }
    }
}
