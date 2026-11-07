//! Priprema sloja za grafičku karticu: razine pojednostavljenja, trokuti i komadi karte.
//!
//! Koordinate su u `f32` u odnosu na ishodište komada. Web Mercator ima vrijednosti od nekoliko
//! milijuna metara, pa bi izravno u `f32` točnost pala na pola metra.

use crate::mercator::u_mercator;
use bytemuck::{Pod, Zeroable};
use lyon_tessellation::geometry_builder::{BuffersBuilder, VertexBuffers};
use lyon_tessellation::math::point;
use lyon_tessellation::path::Path;
use lyon_tessellation::{FillOptions, FillRule, FillTessellator, FillVertex};
use slojevi::{Boja, Geometrija, IdObjekta, Objekt, Sloj, Stil, Vrh};
use std::collections::BTreeMap;

/// Dopuštena greška razine u metrima. Izmjereno na 3 milijuna vrhova (probe/mnogo).
pub(crate) const RAZINE: [f64; 6] = [0.0, 2.0, 8.0, 32.0, 128.0, 512.0];
const KOMAD_M: f64 = 5_000.0;
const TOCAKA_PO_MM: f64 = 96.0 / 25.4;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct VrhIspune {
    pub polozaj: [f32; 2],
    pub boja: [u8; 4],
}

/// Crta između dvije točke. Točka je odsječak s istim krajevima.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct Odsjecak {
    pub a: [f32; 2],
    pub b: [f32; 2],
    pub boja: [u8; 4],
    pub sirina: f32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Komad {
    pub ishodiste: [f64; 2],
    pub okvir: [f64; 4],
    pub vrhovi: Vec<VrhIspune>,
    pub indeksi: Vec<u32>,
    pub odsjecci: Vec<Odsjecak>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Razina {
    pub greska_m: f64,
    pub komadi: Vec<Komad>,
}

/// Objekt u metrima Mercatora, s izgledom.
struct Oblik {
    dijelovi: Vec<Vec<[f64; 2]>>,
    okvir: [f64; 4],
    crta: [u8; 4],
    ispuna: Option<[u8; 4]>,
    sirina: f32,
    tocka: bool,
}

fn boja(b: Boja) -> [u8; 4] {
    [b.r, b.g, b.b, b.a]
}

fn u_metre(vrhovi: &[Vrh]) -> Vec<[f64; 2]> {
    vrhovi
        .iter()
        .map(|v| u_mercator(v.duzina, v.sirina))
        .collect()
}

fn zatvoren(mut prsten: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    if prsten.first() != prsten.last() {
        prsten.extend(prsten.first().copied());
    }
    prsten
}

fn okvir(dijelovi: &[Vec<[f64; 2]>]) -> [f64; 4] {
    let mut o = [f64::MAX, f64::MAX, f64::MIN, f64::MIN];
    for t in dijelovi.iter().flatten() {
        o = [
            o[0].min(t[0]),
            o[1].min(t[1]),
            o[2].max(t[0]),
            o[3].max(t[1]),
        ];
    }
    o
}

fn u_oblik(objekt: &Objekt, stil_sloja: &Stil) -> Oblik {
    let stil = objekt.stil.na_stilu_sloja(stil_sloja);
    let (dijelovi, tocka, lik) = match &objekt.geometrija {
        Geometrija::Tocka(v) => (vec![u_metre(&[*v])], true, false),
        Geometrija::Linija(v) => (vec![u_metre(v)], false, false),
        Geometrija::Poligon(p) => (
            p.iter().map(|r| zatvoren(u_metre(r))).collect(),
            false,
            true,
        ),
    };
    let sirina_mm = if tocka {
        stil.velicina_znaka_mm
    } else {
        stil.debljina_mm
    };
    Oblik {
        okvir: okvir(&dijelovi),
        dijelovi,
        crta: boja(stil.boja_crte),
        ispuna: lik.then_some(boja(stil.boja_ispune)),
        sirina: (sirina_mm * TOCAKA_PO_MM) as f32,
        tocka,
    }
}

/// Koliko je točka `t` daleko od duljine `a`–`b`, na kvadrat.
fn udaljenost2(t: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let duljina2 = dx * dx + dy * dy;
    if duljina2 == 0.0 {
        return (t[0] - a[0]).powi(2) + (t[1] - a[1]).powi(2);
    }
    let c = (t[0] - a[0]) * dy - (t[1] - a[1]) * dx;
    c * c / duljina2
}

fn najdalji(niz: &[[f64; 2]], p: usize, k: usize) -> (usize, f64) {
    let (mut naj, mut d2) = (p, 0.0);
    for (i, t) in niz.iter().enumerate().take(k).skip(p + 1) {
        let d = udaljenost2(*t, niz[p], niz[k]);
        if d > d2 {
            (naj, d2) = (i, d);
        }
    }
    (naj, d2)
}

/// Douglas-Peucker bez rekurzije, da dug niz ne napuni stog. Zatvoren prsten najprije dijeli
/// vrh najdalji od početka, inače bi se srušio na jednu točku.
pub(crate) fn pojednostavi(niz: &[[f64; 2]], greska: f64) -> Vec<[f64; 2]> {
    if greska <= 0.0 || niz.len() < 3 {
        return niz.to_vec();
    }
    let zadnji = niz.len() - 1;
    let mut zadrzi = vec![false; niz.len()];
    zadrzi[0] = true;
    zadrzi[zadnji] = true;
    let (mut stog, g2) = (vec![(0, zadnji)], greska * greska);
    while let Some((p, k)) = stog.pop() {
        if k <= p + 1 {
            continue;
        }
        let (naj, d2) = najdalji(niz, p, k);
        if d2 > g2 {
            zadrzi[naj] = true;
            stog.push((p, naj));
            stog.push((naj, k));
        }
    }
    niz.iter()
        .zip(&zadrzi)
        .filter(|(_, z)| **z)
        .map(|(t, _)| *t)
        .collect()
}

fn dodaj_ispunu(
    ten: &mut FillTessellator,
    k: &mut Komad,
    dijelovi: &[Vec<[f64; 2]>],
    boja: [u8; 4],
) {
    let ish = k.ishodiste;
    let u = |t: [f64; 2]| point((t[0] - ish[0]) as f32, (t[1] - ish[1]) as f32);
    let mut b = Path::builder();
    for prsten in dijelovi.iter().filter(|p| p.len() >= 3) {
        b.begin(u(prsten[0]));
        prsten[1..].iter().for_each(|t| {
            b.line_to(u(*t));
        });
        b.end(true);
    }
    let put = b.build();
    let mut izlaz: VertexBuffers<VrhIspune, u32> = VertexBuffers::new();
    let opcije = FillOptions::default().with_fill_rule(FillRule::EvenOdd);
    let mut gradnja = BuffersBuilder::new(&mut izlaz, |v: FillVertex| VrhIspune {
        polozaj: v.position().to_array(),
        boja,
    });
    // Lik koji se ne da popuniti ostaje samo crta, a ostali se i dalje crtaju.
    if ten.tessellate_path(&put, &opcije, &mut gradnja).is_ok() {
        let pomak = k.vrhovi.len() as u32;
        k.vrhovi.extend_from_slice(&izlaz.vertices);
        k.indeksi.extend(izlaz.indices.iter().map(|i| i + pomak));
    }
}

fn prosiri(k: &mut Komad, o: [f64; 4]) {
    k.okvir = [
        k.okvir[0].min(o[0]),
        k.okvir[1].min(o[1]),
        k.okvir[2].max(o[2]),
        k.okvir[3].max(o[3]),
    ];
}

fn dodaj_crte(k: &mut Komad, oblik: &Oblik, dijelovi: &[Vec<[f64; 2]>]) {
    let ish = k.ishodiste;
    let u = |t: &[f64; 2]| [(t[0] - ish[0]) as f32, (t[1] - ish[1]) as f32];
    let (boja, sirina) = (oblik.crta, oblik.sirina);
    for d in dijelovi {
        if oblik.tocka {
            let a = u(&d[0]);
            k.odsjecci.push(Odsjecak {
                a,
                b: a,
                boja,
                sirina,
            });
        } else {
            k.odsjecci.extend(d.windows(2).map(|w| Odsjecak {
                a: u(&w[0]),
                b: u(&w[1]),
                boja,
                sirina,
            }));
        }
    }
}

/// Lik manji od pola dopuštene greške ne vidi se, pa se ni ne šalje, osim na najtočnijoj razini.
fn premali(oblik: &Oblik, greska: f64) -> bool {
    let najveci = (oblik.okvir[2] - oblik.okvir[0]).max(oblik.okvir[3] - oblik.okvir[1]);
    !oblik.tocka && greska > 0.0 && najveci < greska / 2.0
}

fn razina(ten: &mut FillTessellator, oblici: &[Oblik], r: usize, greska: f64) -> Razina {
    let velicina = KOMAD_M * f64::from(1u32 << r);
    let mut komadi: BTreeMap<(i64, i64), Komad> = BTreeMap::new();
    for oblik in oblici.iter().filter(|o| !premali(o, greska)) {
        let o = oblik.okvir;
        let kljuc = (
            ((o[0] + o[2]) / 2.0 / velicina).floor() as i64,
            ((o[1] + o[3]) / 2.0 / velicina).floor() as i64,
        );
        let k = komadi.entry(kljuc).or_insert_with(|| Komad {
            ishodiste: [
                (kljuc.0 as f64 + 0.5) * velicina,
                (kljuc.1 as f64 + 0.5) * velicina,
            ],
            okvir: [f64::MAX, f64::MAX, f64::MIN, f64::MIN],
            ..Komad::default()
        });
        prosiri(k, o);
        let dijelovi: Vec<_> = oblik
            .dijelovi
            .iter()
            .map(|d| pojednostavi(d, greska))
            .collect();
        if let Some(b) = oblik.ispuna {
            dodaj_ispunu(ten, k, &dijelovi, b);
        }
        dodaj_crte(k, oblik, &dijelovi);
    }
    Razina {
        greska_m: greska,
        komadi: komadi.into_values().collect(),
    }
}

/// Priprema sloj. `skriveni` je objekt koji se upravo uređuje i koji crta `crtez_na_karti`.
pub(crate) fn pripremi_sloj(sloj: &Sloj, skriveni: Option<IdObjekta>) -> Vec<Razina> {
    let oblici: Vec<Oblik> = sloj
        .objekti
        .values()
        .filter(|o| Some(o.id) != skriveni)
        .map(|o| u_oblik(o, &sloj.stil))
        .collect();
    let mut ten = FillTessellator::new();
    RAZINE
        .iter()
        .enumerate()
        .map(|(r, g)| razina(&mut ten, &oblici, r, *g))
        .collect()
}

#[cfg(test)]
mod testovi {
    use super::*;
    use slojevi::{Geometrija, NaredbaObjekta, NaredbaStabla, Novo, Uredjivac, VrstaGeometrije};

    fn sloj_s(vrsta: VrstaGeometrije, geometrije: Vec<Geometrija>) -> Sloj {
        let mut u = Uredjivac::novi();
        let n = NaredbaStabla::DodajSloj {
            u: None,
            naziv: "S".into(),
            vrsta,
        };
        let Ok(Some(Novo::Sloj(s))) = u.izvrsi(n) else {
            panic!("sloj")
        };
        for geometrija in geometrije {
            let n = NaredbaObjekta::Dodaj {
                sloj: s,
                naziv: String::new(),
                geometrija,
            };
            u.izvrsi(n).expect("objekt");
        }
        u.projekt().sloj(s).expect("sloj").clone()
    }

    fn kvadrat(x: f64, stranica: f64) -> Vec<Vrh> {
        [
            (x, 45.0),
            (x + stranica, 45.0),
            (x + stranica, 45.0 + stranica),
            (x, 45.0 + stranica),
        ]
        .iter()
        .map(|(a, b)| Vrh::novi(*a, *b))
        .collect()
    }

    #[test]
    fn pojednostavljenje_ne_prelazi_gresku_a_krajevi_ostaju() {
        let niz: Vec<[f64; 2]> = (0..2000)
            .map(|i| [f64::from(i), (f64::from(i) / 40.0).sin() * 30.0])
            .collect();
        let m = pojednostavi(&niz, 2.0);
        assert!(m.len() < niz.len() / 4, "{}", m.len());
        assert_eq!(m.first(), niz.first());
        assert_eq!(m.last(), niz.last());
        for t in &niz {
            let d = m
                .windows(2)
                .map(|s| do_duzine(*t, s[0], s[1]))
                .fold(f64::MAX, f64::min);
            assert!(d <= 2.0 + 1e-9, "{d}");
        }
    }

    fn do_duzine(t: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let u = (((t[0] - a[0]) * dx + (t[1] - a[1]) * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
        ((a[0] + u * dx - t[0]).powi(2) + (a[1] + u * dy - t[1]).powi(2)).sqrt()
    }

    #[test]
    fn bez_greske_i_kratak_niz_ostaju_cijeli() {
        let niz = [[0.0, 0.0], [1.0, 1.0], [2.0, 0.0]];
        assert_eq!(pojednostavi(&niz, 0.0), niz.to_vec());
        assert_eq!(pojednostavi(&niz[..2], 5.0), niz[..2].to_vec());
    }

    #[test]
    fn udaljenost_od_kose_crte_je_tocna() {
        // Stranica 3 i 4 daje duljinu 5, a brojnik 12, pa je kvadrat udaljenosti 144 / 25.
        let d = udaljenost2([4.0, 2.0], [1.0, 2.0], [4.0, 6.0]);
        assert!((d - 5.76).abs() < 1e-12, "{d}");
        let d = udaljenost2([0.0, 5.0], [0.0, 0.0], [10.0, 0.0]);
        assert!((d - 25.0).abs() < 1e-12, "{d}");
    }

    #[test]
    fn granica_pojednostavljenja_je_tocna() {
        let tri = |y: f64| [[0.0, 0.0], [5.0, y], [10.0, 0.0]];
        assert_eq!(pojednostavi(&tri(2.6), 3.0).len(), 2, "unutar greške");
        assert_eq!(pojednostavi(&tri(3.0), 3.0).len(), 2, "točno na granici");
        assert_eq!(pojednostavi(&tri(3.5), 3.0).len(), 3, "izvan greške");
        assert_eq!(pojednostavi(&tri(0.0), 5.0).len(), 2, "tri točke na pravcu");
        let bez_greske = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 1.0]];
        assert_eq!(
            pojednostavi(&bez_greske, 0.0).len(),
            4,
            "nula ne miče točke na pravcu"
        );
        assert!(pojednostavi(&[], 5.0).is_empty());
        assert_eq!(pojednostavi(&[[1.0, 1.0]], 5.0).len(), 1);
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
    fn komad_ima_tocan_okvir_i_ishodiste_na_svakoj_razini() {
        let s = sloj_s(
            VrstaGeometrije::Tocka,
            vec![Geometrija::Tocka(Vrh::novi(18.69, 45.55))],
        );
        let t = u_mercator(18.69, 45.55);
        for (r, razina) in pripremi_sloj(&s, None).iter().enumerate() {
            let k = &razina.komadi[0];
            assert_eq!(k.okvir, [t[0], t[1], t[0], t[1]], "razina {r}");
            let pola = KOMAD_M * f64::from(1u32 << r) / 2.0;
            assert!((t[0] - k.ishodiste[0]).abs() <= pola, "razina {r}");
            assert!((t[1] - k.ishodiste[1]).abs() <= pola, "razina {r}");
        }
    }

    #[test]
    fn velicina_komada_raste_s_razinom() {
        // Oko 8 km i oko 200 km između točaka, na istoj širini.
        let s = sloj_s(
            VrstaGeometrije::Tocka,
            vec![
                Geometrija::Tocka(Vrh::novi(18.0, 45.0)),
                Geometrija::Tocka(Vrh::novi(18.07, 45.0)),
                Geometrija::Tocka(Vrh::novi(20.0, 45.0)),
            ],
        );
        let r = pripremi_sloj(&s, None);
        assert_eq!(r[0].komadi.len(), 3, "komad je 5 km");
        assert_eq!(r[5].komadi.len(), 2, "komad je 160 km");
    }

    #[test]
    fn dva_lika_u_komadu_imaju_svaki_svoje_trokute() {
        let s = sloj_s(
            VrstaGeometrije::Poligon,
            vec![
                Geometrija::Poligon(vec![kvadrat(18.0, 0.001)]),
                Geometrija::Poligon(vec![kvadrat(18.002, 0.001)]),
            ],
        );
        let k = &pripremi_sloj(&s, None)[0].komadi[0];
        assert_eq!(k.indeksi.len(), 12);
        assert_eq!(k.vrhovi.len(), 8);
        assert!(
            k.indeksi[6..].iter().all(|i| *i >= 4 && *i < 8),
            "{:?}",
            k.indeksi
        );
    }

    #[test]
    fn lik_nestaje_prema_svojoj_velicini_a_ne_prema_polozaju() {
        let lik = |stranica| {
            sloj_s(
                VrstaGeometrije::Poligon,
                vec![Geometrija::Poligon(vec![kvadrat(18.0, stranica)])],
            )
        };
        // Razina 5 ima grešku 512 m, pa nestaju likovi manji od 256 m. U Mercatoru je na 45° širine
        // stupanj širine oko 157 km, pa strana od 0,002° ima oko 315 m, a od 0,0015° oko 236 m.
        assert_eq!(
            pripremi_sloj(&lik(0.002), None)[5].komadi.len(),
            1,
            "oko 315 m"
        );
        assert!(
            pripremi_sloj(&lik(0.0015), None)[5].komadi.is_empty(),
            "oko 236 m"
        );
    }

    #[test]
    fn udaljenost_od_iste_tocke_nije_dijeljenje_nulom() {
        assert_eq!(udaljenost2([3.0, 4.0], [0.0, 0.0], [0.0, 0.0]), 25.0);
        assert_eq!(udaljenost2([3.0, 4.0], [1.0, 1.0], [1.0, 1.0]), 13.0);
    }

    #[test]
    fn poligon_ima_ispunu_i_zatvoren_rub_a_linija_samo_crte() {
        let lik = sloj_s(
            VrstaGeometrije::Poligon,
            vec![Geometrija::Poligon(vec![kvadrat(18.0, 0.01)])],
        );
        let k = &pripremi_sloj(&lik, None)[0].komadi[0];
        assert_eq!(k.odsjecci.len(), 4, "zatvoren rub");
        assert_eq!(k.indeksi.len(), 6, "dva trokuta");
        let crta = sloj_s(
            VrstaGeometrije::Linija,
            vec![Geometrija::Linija(kvadrat(18.0, 0.01))],
        );
        let k = &pripremi_sloj(&crta, None)[0].komadi[0];
        assert_eq!(k.odsjecci.len(), 3);
        assert!(k.indeksi.is_empty());
    }

    #[test]
    fn rupa_ostaje_rupa_i_vec_zatvoren_prsten_se_ne_zatvara_dvaput() {
        let mut vanjski = kvadrat(18.0, 0.04);
        vanjski.push(vanjski[0]);
        let rupa = kvadrat(18.01, 0.01);
        let lik = sloj_s(
            VrstaGeometrije::Poligon,
            vec![Geometrija::Poligon(vec![vanjski, rupa])],
        );
        let k = &pripremi_sloj(&lik, None)[0].komadi[0];
        assert_eq!(k.odsjecci.len(), 8);
        let povrsina: f64 = k
            .indeksi
            .chunks(3)
            .map(|t| {
                let (a, b, c) = (
                    k.vrhovi[t[0] as usize].polozaj,
                    k.vrhovi[t[1] as usize].polozaj,
                    k.vrhovi[t[2] as usize].polozaj,
                );
                (f64::from(b[0] - a[0]) * f64::from(c[1] - a[1])
                    - f64::from(c[0] - a[0]) * f64::from(b[1] - a[1]))
                .abs()
                    / 2.0
            })
            .sum();
        let cijeli = k.okvir;
        let puni = (cijeli[2] - cijeli[0]) * (cijeli[3] - cijeli[1]);
        assert!(
            povrsina < puni * 0.95,
            "rupa nije izrezana: {povrsina} od {puni}"
        );
    }

    #[test]
    fn tocka_je_odsjecak_s_istim_krajevima_i_velicinom_znaka() {
        let s = sloj_s(
            VrstaGeometrije::Tocka,
            vec![Geometrija::Tocka(Vrh::novi(18.0, 45.0))],
        );
        for r in pripremi_sloj(&s, None) {
            let o = r.komadi[0].odsjecci[0];
            assert_eq!(o.a, o.b, "ne nestaje ni na jednoj razini");
            assert!((f64::from(o.sirina) - 3.0 * TOCAKA_PO_MM).abs() < 1e-4);
        }
    }

    #[test]
    fn premali_lik_nestaje_samo_na_grubim_razinama() {
        let s = sloj_s(
            VrstaGeometrije::Poligon,
            vec![Geometrija::Poligon(vec![kvadrat(18.0, 0.00001)])],
        );
        let r = pripremi_sloj(&s, None);
        assert_eq!(r[0].komadi.len(), 1);
        assert_eq!(r[1].komadi.len(), 1, "oko 1 m, greška 2 m");
        assert!(r[5].komadi.is_empty());
    }

    #[test]
    fn lik_tocno_na_pola_greske_ostaje() {
        let lik = |sirina: f64| Oblik {
            dijelovi: Vec::new(),
            okvir: [0.0, 0.0, sirina, 0.0],
            crta: [0; 4],
            ispuna: None,
            sirina: 1.0,
            tocka: false,
        };
        assert!(!premali(&lik(1.0), 2.0));
        assert!(premali(&lik(0.99), 2.0));
    }

    #[test]
    fn skriveni_objekt_se_ne_priprema() {
        let s = sloj_s(
            VrstaGeometrije::Tocka,
            vec![Geometrija::Tocka(Vrh::novi(18.0, 45.0))],
        );
        let id = *s.objekti.keys().next().expect("objekt");
        assert!(pripremi_sloj(&s, Some(id))[0].komadi.is_empty());
    }

    #[test]
    fn koordinate_u_komadu_ostaju_tocne_na_milimetar() {
        let s = sloj_s(
            VrstaGeometrije::Tocka,
            vec![Geometrija::Tocka(Vrh::novi(18.69, 45.55))],
        );
        let k = &pripremi_sloj(&s, None)[0].komadi[0];
        let t = u_mercator(18.69, 45.55);
        let o = k.odsjecci[0].a;
        let natrag = [
            f64::from(o[0]) + k.ishodiste[0],
            f64::from(o[1]) + k.ishodiste[1],
        ];
        assert!((natrag[0] - t[0]).abs() < 1e-3 && (natrag[1] - t[1]).abs() < 1e-3);
    }

    #[test]
    fn stil_objekta_pobjeduje_stil_sloja_a_daleki_objekti_su_u_raznim_komadima() {
        let mut s = sloj_s(
            VrstaGeometrije::Tocka,
            vec![
                Geometrija::Tocka(Vrh::novi(18.0, 45.0)),
                Geometrija::Tocka(Vrh::novi(19.0, 45.0)),
            ],
        );
        let plavo = Boja {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        };
        s.objekti
            .values_mut()
            .next()
            .expect("objekt")
            .stil
            .boja_crte = Some(plavo);
        let r = pripremi_sloj(&s, None);
        assert_eq!(r[0].komadi.len(), 2);
        assert_eq!(r[0].komadi[0].odsjecci[0].boja, [0, 0, 255, 255]);
        assert_eq!(
            r[0].komadi[1].odsjecci[0].boja,
            boja(Stil::default().boja_crte)
        );
    }
}
