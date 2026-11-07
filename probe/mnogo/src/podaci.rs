//! Podaci za crtanje: stvarni KML ili izmišljeni podaci iste veličine.
//!
//! Sve je u metrima Web Mercatora, kao ekran u programu.

use proba_kml::model::{BojaRgba, Geometrija, Mapa, Objekt, Vrh};
use std::f64::consts::{FRAC_PI_4, PI};

const R: f64 = 6_378_137.0;

pub struct Znacajka {
    pub ispuna: Option<[u8; 4]>,
    pub crta: [u8; 4],
    pub sirina: f32,
    /// Prstenovi lika ili jedna crta. Točka ima jedan vrh.
    pub dijelovi: Vec<Vec<[f64; 2]>>,
    pub lik: bool,
    pub tocka: bool,
    pub vidljivo: bool,
}

pub fn mercator(lon: f64, lat: f64) -> [f64; 2] {
    [R * lon.to_radians(), R * (FRAC_PI_4 + lat.to_radians() / 2.0).tan().ln()]
}

fn u_niz(v: &[Vrh]) -> Vec<[f64; 2]> {
    v.iter().map(|t| mercator(t.duzina, t.sirina)).collect()
}

fn boja(b: Option<BojaRgba>, zadano: [u8; 4]) -> [u8; 4] {
    b.map_or(zadano, |c| [c.r, c.g, c.b, c.a])
}

fn iz_geometrije(g: &Geometrija, o: &Objekt, vidljivo: bool, izlaz: &mut Vec<Znacajka>) {
    let stil = o.rijeseni_stil.as_ref().map(|s| &s.normalni);
    let crta_stil = stil.and_then(|s| s.crta.as_ref());
    let crta = boja(crta_stil.and_then(|c| c.boja), [255, 255, 255, 255]);
    let sirina = crta_stil.map_or(1.0, |c| c.sirina.max(0.5)) as f32;
    let z = |dijelovi, lik, tocka, ispuna| Znacajka { ispuna, crta, sirina, dijelovi, lik, tocka, vidljivo };
    match g {
        Geometrija::Tocka { vrh, .. } => {
            let d = vec![vec![mercator(vrh.duzina, vrh.sirina)]];
            izlaz.push(z(d, false, true, None));
        }
        Geometrija::Linija { vrhovi, .. } | Geometrija::Trag { vrhovi, .. } => {
            if vrhovi.len() >= 2 {
                izlaz.push(z(vec![u_niz(vrhovi)], false, false, None));
            }
        }
        Geometrija::Poligon { vanjski, rupe, .. } => {
            let p = stil.and_then(|s| s.poligon.as_ref());
            let ispuna = match p {
                Some(p) if !p.ispuna => None,
                Some(p) => Some(boja(p.boja, [255, 255, 255, 255])),
                None => Some([255, 255, 255, 255]),
            };
            let mut d = vec![u_niz(vanjski)];
            d.extend(rupe.iter().map(|r| u_niz(r)));
            izlaz.push(z(d, true, false, ispuna));
        }
        Geometrija::ViseGeometrija(sve) => {
            for jedna in sve {
                iz_geometrije(jedna, o, vidljivo, izlaz);
            }
        }
    }
}

fn iz_mape(m: &Mapa, vidljivo: bool, izlaz: &mut Vec<Znacajka>) {
    let vidljivo = vidljivo && m.vidljivo;
    for o in &m.objekti {
        if let Some(g) = &o.geometrija {
            iz_geometrije(g, o, vidljivo && o.vidljivo, izlaz);
        }
    }
    for p in &m.podmape {
        iz_mape(p, vidljivo, izlaz);
    }
}

pub fn iz_kml(korijen: &Mapa) -> Vec<Znacajka> {
    let mut izlaz = Vec::new();
    iz_mape(korijen, true, &mut izlaz);
    izlaz
}

struct Slucajni(u64);

impl Slucajni {
    fn f(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % 1_000_000) as f64 / 1_000_000.0
    }
}

/// Izmišljeni podaci iste veličine kao stvarni, u nakupinama preko Hrvatske. Smiju se gledati.
pub fn izmisljeno(likova: usize, linija: usize, tocaka: usize, vrhova_po_liku: usize, vrhova_po_liniji: usize) -> Vec<Znacajka> {
    let mut s = Slucajni(0x2026_1107_5555);
    let mut izlaz = Vec::with_capacity(likova + linija + tocaka);
    let nakupine: Vec<[f64; 2]> = (0..400)
        .map(|_| mercator(13.5 + s.f() * 5.8, 42.5 + s.f() * 3.9))
        .collect();
    let mjesto = |s: &mut Slucajni| {
        let n = nakupine[(s.f() * 399.0) as usize];
        [n[0] + (s.f() - 0.5) * 40_000.0, n[1] + (s.f() - 0.5) * 40_000.0]
    };
    let paleta = [[46u8, 125, 50, 120u8], [25, 118, 210, 120], [245, 124, 0, 120], [106, 27, 154, 120], [0, 137, 123, 120]];
    for i in 0..likova {
        let c = mjesto(&mut s);
        let r = 30.0 + s.f() * s.f() * 800.0;
        let prsten: Vec<[f64; 2]> = (0..=vrhova_po_liku)
            .map(|k| {
                let a = 2.0 * PI * k as f64 / vrhova_po_liku as f64;
                let rr = r * (0.8 + 0.2 * (a * 5.0 + i as f64).sin());
                [c[0] + rr * a.cos(), c[1] + rr * a.sin()]
            })
            .collect();
        let b = paleta[i % paleta.len()];
        izlaz.push(Znacajka { ispuna: Some(b), crta: [b[0], b[1], b[2], 255], sirina: 1.0, dijelovi: vec![prsten], lik: true, tocka: false, vidljivo: i % 80 == 0 });
    }
    for i in 0..linija {
        let mut c = mjesto(&mut s);
        let mut smjer = s.f() * 2.0 * PI;
        let niz = (0..vrhova_po_liniji)
            .map(|_| {
                smjer += (s.f() - 0.5) * 0.6;
                c = [c[0] + 60.0 * smjer.cos(), c[1] + 60.0 * smjer.sin()];
                c
            })
            .collect();
        izlaz.push(Znacajka { ispuna: None, crta: [220, 30, 30, 255], sirina: 1.5, dijelovi: vec![niz], lik: false, tocka: false, vidljivo: i % 8 == 0 });
    }
    for i in 0..tocaka {
        let c = mjesto(&mut s);
        izlaz.push(Znacajka { ispuna: None, crta: [20, 20, 20, 255], sirina: 6.0, dijelovi: vec![vec![c]], lik: false, tocka: true, vidljivo: i % 8 == 0 });
    }
    izlaz
}
