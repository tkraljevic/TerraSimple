//! Crte i likovi iz istih zapisa puta kao u nacrtu (M, L, C, Z, apsolutno).
//!
//! egui sam puni samo izbočene likove. Zato se ovdje lik puni lepezom iz sredine,
//! što je dovoljno za probu. Pravi program treba `lyon`, jer parcele nisu izbočene.

use eframe::egui::{Color32, Mesh, Painter, Pos2, Shape, Stroke, pos2};

pub struct Put {
    pub tocke: Vec<Pos2>,
    pub zatvoren: bool,
}

pub fn procitaj_put(zapis: &str, pomak: Pos2, mjera: f32) -> Vec<Put> {
    let mut putovi = Vec::new();
    let mut tocke: Vec<Pos2> = Vec::new();
    let mut brojevi: Vec<f32> = Vec::new();
    let mut naredba = ' ';
    let t = |x: f32, y: f32| pos2(pomak.x + x * mjera, pomak.y + y * mjera);

    let dovrsi = |naredba: char, brojevi: &mut Vec<f32>, tocke: &mut Vec<Pos2>| {
        match naredba {
            'M' | 'L' => {
                for par in brojevi.chunks_exact(2) {
                    tocke.push(t(par[0], par[1]));
                }
            }
            'C' => {
                for s in brojevi.chunks_exact(6) {
                    let p0 = *tocke.last().unwrap_or(&t(s[0], s[1]));
                    let (p1, p2, p3) = (t(s[0], s[1]), t(s[2], s[3]), t(s[4], s[5]));
                    for i in 1..=16 {
                        let u = i as f32 / 16.0;
                        let v = 1.0 - u;
                        tocke.push(pos2(
                            v * v * v * p0.x + 3.0 * v * v * u * p1.x + 3.0 * v * u * u * p2.x + u * u * u * p3.x,
                            v * v * v * p0.y + 3.0 * v * v * u * p1.y + 3.0 * v * u * u * p2.y + u * u * u * p3.y,
                        ));
                    }
                }
            }
            _ => {}
        }
        brojevi.clear();
    };

    for dio in zapis.replace('M', " M ").replace('L', " L ").replace('C', " C ").replace('Z', " Z ").split_whitespace() {
        match dio {
            "M" | "L" | "C" => {
                dovrsi(naredba, &mut brojevi, &mut tocke);
                if dio == "M" && !tocke.is_empty() {
                    putovi.push(Put { tocke: std::mem::take(&mut tocke), zatvoren: false });
                }
                naredba = dio.chars().next().unwrap_or(' ');
            }
            "Z" => {
                dovrsi(naredba, &mut brojevi, &mut tocke);
                putovi.push(Put { tocke: std::mem::take(&mut tocke), zatvoren: true });
                naredba = ' ';
            }
            broj => {
                if let Ok(v) = broj.parse() {
                    brojevi.push(v);
                }
            }
        }
    }
    dovrsi(naredba, &mut brojevi, &mut tocke);
    if !tocke.is_empty() {
        putovi.push(Put { tocke, zatvoren: false });
    }
    putovi
}

pub fn crta(p: &Painter, tocke: &[Pos2], zatvoren: bool, debljina: f32, boja: Color32) {
    let s = Stroke::new(debljina, boja);
    let mut v = tocke.to_vec();
    if zatvoren {
        p.add(Shape::closed_line(v, s));
    } else {
        p.add(Shape::line(std::mem::take(&mut v), s));
    }
}

pub fn lik(p: &Painter, tocke: &[Pos2], ispuna: Color32) {
    if tocke.len() < 3 {
        return;
    }
    let n = tocke.len() as f32;
    let sredina = tocke.iter().fold(pos2(0.0, 0.0), |a, b| pos2(a.x + b.x / n, a.y + b.y / n));
    let mut m = Mesh::default();
    m.colored_vertex(sredina, ispuna);
    for t in tocke {
        m.colored_vertex(*t, ispuna);
    }
    let k = tocke.len() as u32;
    for i in 0..k {
        m.add_triangle(0, 1 + i, 1 + (i + 1) % k);
    }
    p.add(Shape::mesh(m));
}

pub fn put_crta(p: &Painter, zapis: &str, pomak: Pos2, debljina: f32, boja: Color32) {
    for d in procitaj_put(zapis, pomak, 1.0) {
        crta(p, &d.tocke, d.zatvoren, debljina, boja);
    }
}

pub fn put_lik(p: &Painter, zapis: &str, pomak: Pos2, ispuna: Color32) {
    for d in procitaj_put(zapis, pomak, 1.0) {
        lik(p, &d.tocke, ispuna);
        // Mreža nema zaglađen rub, pa ga daje tanka crta iste boje.
        crta(p, &d.tocke, true, 1.0, ispuna);
    }
}

pub fn isprekidana(p: &Painter, tocke: &[Pos2], debljina: f32, boja: Color32, crtica: f32, razmak: f32) {
    let v: Vec<Pos2> = tocke.to_vec();
    p.extend(Shape::dashed_line(&v, Stroke::new(debljina, boja), crtica, razmak));
}
