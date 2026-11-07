//! Priprema za grafičku karticu: razine pojednostavljenja, trokuti i komadi karte.
//!
//! Koordinate su u `f32` u odnosu na ishodište komada. Web Mercator ima vrijednosti od
//! nekoliko milijuna metara, pa bi izravno u `f32` točnost pala na pola metra.

use crate::podaci::Znacajka;
use bytemuck::{Pod, Zeroable};
use lyon_tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon_tessellation::math::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{FillOptions, FillRule, FillTessellator, FillVertex};
use std::collections::BTreeMap;

/// Dopuštena greška razine u metrima. Razina se bira da greška bude ispod jednog piksela.
pub const RAZINE: [f64; 6] = [0.0, 2.0, 8.0, 32.0, 128.0, 512.0];
const KOMAD_M: f64 = 5_000.0;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VrhIspune {
    pub polozaj: [f32; 2],
    pub boja: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Odsjecak {
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub boja: [u8; 4],
    pub sirina: f32,
}

#[derive(Default)]
pub struct Komad {
    pub ishodiste: [f64; 2],
    pub okvir: [f64; 4],
    pub vrhovi: Vec<VrhIspune>,
    pub indeksi: Vec<u32>,
    pub odsjecci: Vec<Odsjecak>,
}

pub struct Razina {
    pub greska_m: f64,
    pub komadi: Vec<Komad>,
}

impl Razina {
    pub fn vrhova_odsjecaka(&self) -> (usize, usize, usize) {
        let z = |f: fn(&Komad) -> usize| self.komadi.iter().map(f).sum();
        (z(|k| k.vrhovi.len()), z(|k| k.indeksi.len() / 3), z(|k| k.odsjecci.len()))
    }
}

/// Douglas-Peucker bez rekurzije, da dug niz ne napuni stog.
pub fn pojednostavi(niz: &[[f64; 2]], greska: f64) -> Vec<[f64; 2]> {
    if greska <= 0.0 || niz.len() < 3 {
        return niz.to_vec();
    }
    let mut zadrzi = vec![false; niz.len()];
    zadrzi[0] = true;
    zadrzi[niz.len() - 1] = true;
    let mut stog = vec![(0, niz.len() - 1)];
    // Zatvoren prsten ima isti prvi i zadnji vrh, pa ga najprije dijeli najdalji vrh.
    if niz[0] == niz[niz.len() - 1] {
        let (mut najdalji, mut d2) = (0, 0.0);
        for (i, t) in niz.iter().enumerate() {
            let d = (t[0] - niz[0][0]).powi(2) + (t[1] - niz[0][1]).powi(2);
            if d > d2 {
                (najdalji, d2) = (i, d);
            }
        }
        if najdalji > 0 {
            zadrzi[najdalji] = true;
            stog = vec![(0, najdalji), (najdalji, niz.len() - 1)];
        }
    }
    let g2 = greska * greska;
    while let Some((p, k)) = stog.pop() {
        if k <= p + 1 {
            continue;
        }
        let (a, b) = (niz[p], niz[k]);
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let duljina2 = dx * dx + dy * dy;
        let (mut najdalji, mut d2) = (0, 0.0);
        for (i, t) in niz.iter().enumerate().take(k).skip(p + 1) {
            let d = if duljina2 == 0.0 {
                (t[0] - a[0]).powi(2) + (t[1] - a[1]).powi(2)
            } else {
                let c = (t[0] - a[0]) * dy - (t[1] - a[1]) * dx;
                c * c / duljina2
            };
            if d > d2 {
                (najdalji, d2) = (i, d);
            }
        }
        if d2 > g2 {
            zadrzi[najdalji] = true;
            stog.push((p, najdalji));
            stog.push((najdalji, k));
        }
    }
    niz.iter().zip(&zadrzi).filter(|(_, z)| **z).map(|(t, _)| *t).collect()
}

fn okvir(z: &Znacajka) -> [f64; 4] {
    let mut o = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for t in z.dijelovi.iter().flatten() {
        o = [o[0].min(t[0]), o[1].min(t[1]), o[2].max(t[0]), o[3].max(t[1])];
    }
    o
}

fn dodaj_ispunu(
    ten: &mut FillTessellator,
    k: &mut Komad,
    dijelovi: &[Vec<[f64; 2]>],
    boja: [u8; 4],
    pogresaka: &mut usize,
) {
    let mut b = Path::builder();
    for prsten in dijelovi {
        if prsten.len() < 3 {
            continue;
        }
        let u = |t: [f64; 2]| point((t[0] - k.ishodiste[0]) as f32, (t[1] - k.ishodiste[1]) as f32);
        b.begin(u(prsten[0]));
        for t in &prsten[1..] {
            b.line_to(u(*t));
        }
        b.end(true);
    }
    let put = b.build();
    let mut izlaz: VertexBuffers<VrhIspune, u32> = VertexBuffers::new();
    let opcije = FillOptions::default().with_fill_rule(FillRule::EvenOdd);
    let rezultat = ten.tessellate_path(
        &put,
        &opcije,
        &mut BuffersBuilder::new(&mut izlaz, |v: FillVertex| VrhIspune { polozaj: v.position().to_array(), boja }),
    );
    if rezultat.is_err() {
        *pogresaka += 1;
        return;
    }
    let pomak = k.vrhovi.len() as u32;
    k.vrhovi.extend_from_slice(&izlaz.vertices);
    k.indeksi.extend(izlaz.indices.iter().map(|i| i + pomak));
}

pub struct Priprema {
    pub razine: Vec<Razina>,
    pub pogresaka_ispune: usize,
}

pub fn pripremi(sve: &[Znacajka], samo_vidljivo: bool) -> Priprema {
    let mut ten = FillTessellator::new();
    let mut pogresaka_ispune = 0;
    let mut razine = Vec::new();
    for (r, greska) in RAZINE.iter().enumerate() {
        let velicina = KOMAD_M * f64::from(1u32 << r);
        let mut komadi: BTreeMap<(i64, i64), Komad> = BTreeMap::new();
        for z in sve.iter().filter(|z| !samo_vidljivo || z.vidljivo) {
            let o = okvir(z);
            let najveci = (o[2] - o[0]).max(o[3] - o[1]);
            // Lik manji od pola piksela te razine ne vidi se, pa se ni ne šalje.
            if !z.tocka && *greska > 0.0 && najveci < greska / 2.0 {
                continue;
            }
            let c = [(o[0] + o[2]) / 2.0, (o[1] + o[3]) / 2.0];
            let kljuc = ((c[0] / velicina).floor() as i64, (c[1] / velicina).floor() as i64);
            let k = komadi.entry(kljuc).or_insert_with(|| Komad {
                ishodiste: [(kljuc.0 as f64 + 0.5) * velicina, (kljuc.1 as f64 + 0.5) * velicina],
                okvir: [f64::MAX, f64::MAX, f64::MIN, f64::MIN],
                ..Komad::default()
            });
            k.okvir = [k.okvir[0].min(o[0]), k.okvir[1].min(o[1]), k.okvir[2].max(o[2]), k.okvir[3].max(o[3])];
            let dijelovi: Vec<Vec<[f64; 2]>> = z.dijelovi.iter().map(|d| pojednostavi(d, *greska)).collect();
            if let (true, Some(b)) = (z.lik, z.ispuna) {
                dodaj_ispunu(&mut ten, k, &dijelovi, b, &mut pogresaka_ispune);
            }
            let ish = k.ishodiste;
            let u = |t: &[f64; 2]| [(t[0] - ish[0]) as f32, (t[1] - ish[1]) as f32];
            for d in &dijelovi {
                if z.tocka {
                    k.odsjecci.push(Odsjecak { a: u(&d[0]), b: u(&d[0]), boja: z.crta, sirina: z.sirina });
                } else {
                    k.odsjecci.extend(d.windows(2).map(|w| Odsjecak { a: u(&w[0]), b: u(&w[1]), boja: z.crta, sirina: z.sirina }));
                }
            }
        }
        razine.push(Razina { greska_m: *greska, komadi: komadi.into_values().collect() });
    }
    Priprema { razine, pogresaka_ispune }
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn pojednostavljenje_ne_prelazi_gresku() {
        let niz: Vec<[f64; 2]> = (0..2000).map(|i| [f64::from(i), (f64::from(i) / 40.0).sin() * 30.0]).collect();
        let m = pojednostavi(&niz, 2.0);
        assert!(m.len() < niz.len() / 4, "{}", m.len());
        for t in &niz {
            let najbliza = m
                .windows(2)
                .map(|s| {
                    let (a, b) = (s[0], s[1]);
                    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
                    let u = (((t[0] - a[0]) * dx + (t[1] - a[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
                    ((a[0] + u * dx - t[0]).powi(2) + (a[1] + u * dy - t[1]).powi(2)).sqrt()
                })
                .fold(f64::MAX, f64::min);
            assert!(najbliza <= 2.0 + 1e-9, "{najbliza}");
        }
    }

    #[test]
    fn zatvoren_prsten_ostaje_zatvoren_i_nije_srusen() {
        let mut niz: Vec<[f64; 2]> = (0..100)
            .map(|i| {
                let a = f64::from(i) / 100.0 * std::f64::consts::TAU;
                [100.0 * a.cos(), 100.0 * a.sin()]
            })
            .collect();
        niz.push(niz[0]);
        let m = pojednostavi(&niz, 5.0);
        assert_eq!(m.first(), m.last());
        assert!(m.len() >= 4 && m.len() < 40, "{}", m.len());
    }

    #[test]
    fn koordinate_u_komadu_ostaju_tocne_u_f32() {
        // Na najmanjoj razini ostatak je najviše 2,5 km, pa f32 drži ispod 1 mm.
        let x = 2_080_561.283_f64;
        let ishodiste = 2_082_500.0_f64;
        let u = (x - ishodiste) as f32;
        assert!((f64::from(u) + ishodiste - x).abs() < 0.001);
    }
}
