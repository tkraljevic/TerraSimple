//! Lik koji siječe sam sebe.
//!
//! Takvom liku površina je razlika njegovih dijelova, kao u QGIS-u, pa osmica može imati površinu 0,
//! a QGIS (GEOS `isValid`) ga smatra neispravnim. Provjera daje jedno mjesto presjeka, da ga karta
//! može označiti.
//!
//! Računa se u ravnini dužine i širine, kao GEOS za sloj u EPSG:4326. Lik preko 180. meridijana
//! zato ispada kao da obilazi Zemlju i može dobiti lažan presjek; takvih likova u našem radu nema.

use crate::Tocka;

/// Kako se dva ruba dodiruju.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Dodir {
    /// Rubovi se križaju u točki koja nije kraj nijednog od njih.
    Krizanje(Tocka),
    /// Kraj jednog ruba leži na drugom rubu, npr. dva prstena se dotiču u vrhu.
    Kraj(Tocka),
    /// Rubovi leže na istom pravcu i dijele komad duljine veće od nule.
    Preklapanje(Tocka),
}

fn orijentacija(a: Tocka, b: Tocka, c: Tocka) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// Leži li `t` na dužini `a`–`b`, uz uvjet da su sve tri točke na istom pravcu.
fn unutar(a: Tocka, b: Tocka, t: Tocka) -> bool {
    a[0].min(b[0]) <= t[0]
        && t[0] <= a[0].max(b[0])
        && a[1].min(b[1]) <= t[1]
        && t[1] <= a[1].max(b[1])
}

/// Mjesto po kojem se dužine na istom pravcu poredaju: x, a za okomite y.
fn polozaj(a: Tocka, b: Tocka, t: Tocka) -> f64 {
    if a[0] == b[0] { t[1] } else { t[0] }
}

/// Dvije dužine na istom pravcu: dijele li komad, i gdje.
fn preklapanje(p: [Tocka; 2], q: [Tocka; 2]) -> Option<Dodir> {
    let mut tocke = [p[0], p[1], q[0], q[1]];
    tocke.sort_by(|x, y| polozaj(p[0], p[1], *x).total_cmp(&polozaj(p[0], p[1], *y)));
    let unutarnje = [tocke[1], tocke[2]];
    let u_oba = unutarnje
        .iter()
        .all(|t| unutar(p[0], p[1], *t) && unutar(q[0], q[1], *t));
    if !u_oba {
        return None;
    }
    Some(if unutarnje[0] == unutarnje[1] {
        Dodir::Kraj(unutarnje[0])
    } else {
        Dodir::Preklapanje(unutarnje[0])
    })
}

/// Kraj jedne dužine koji leži na drugoj, ako ga ima.
fn kraj_na_rubu(p: [Tocka; 2], q: [Tocka; 2], o: [f64; 4]) -> Option<Dodir> {
    [
        (o[0], q, p[0]),
        (o[1], q, p[1]),
        (o[2], p, q[0]),
        (o[3], p, q[1]),
    ]
    .into_iter()
    .find(|(oj, d, t)| *oj == 0.0 && unutar(d[0], d[1], *t))
    .map(|(_, _, t)| Dodir::Kraj(t))
}

/// Dodir dviju dužina `p` i `q`.
fn dodir(p: [Tocka; 2], q: [Tocka; 2]) -> Option<Dodir> {
    let o = [
        orijentacija(q[0], q[1], p[0]),
        orijentacija(q[0], q[1], p[1]),
        orijentacija(p[0], p[1], q[0]),
        orijentacija(p[0], p[1], q[1]),
    ];
    if o.iter().all(|x| *x == 0.0) {
        return preklapanje(p, q);
    }
    if o[0] * o[1] < 0.0 && o[2] * o[3] < 0.0 {
        let t = o[2] / (o[2] - o[3]);
        let x = [
            q[0][0] + t * (q[1][0] - q[0][0]),
            q[0][1] + t * (q[1][1] - q[0][1]),
        ];
        return Some(Dodir::Krizanje(x));
    }
    kraj_na_rubu(p, q, o)
}

/// Prsten bez zatvarajuće točke i bez iste točke ponovljene zaredom. Ponovljena točka nije
/// presjek, samo rub duljine 0.
fn ocisti(prsten: &[Tocka]) -> Vec<Tocka> {
    let mut v: Vec<Tocka> = Vec::with_capacity(prsten.len());
    for t in prsten {
        if v.last() != Some(t) {
            v.push(*t);
        }
    }
    if v.len() > 1 && v.first() == v.last() {
        v.pop();
    }
    v
}

/// Rub lika: kojem prstenu pripada, koji je po redu i gdje su mu krajevi.
#[derive(Clone, Copy, Debug)]
struct Rub {
    prsten: usize,
    redni: usize,
    duljina_prstena: usize,
    krajevi: [Tocka; 2],
}

impl Rub {
    fn x_raspon(&self) -> [f64; 2] {
        let [a, b] = self.krajevi;
        [a[0].min(b[0]), a[0].max(b[0])]
    }

    /// Susjedni rubovi istog prstena dijele vrh, pa njihov dodir u tom vrhu nije presjek.
    fn susjed(&self, drugi: &Rub) -> bool {
        let razlika = self.redni.abs_diff(drugi.redni);
        self.prsten == drugi.prsten && (razlika == 1 || razlika == self.duljina_prstena - 1)
    }
}

/// Dodiri dvaju različitih prstenova u vrhu. Kao u GEOS-u, dva prstena smiju se dotaknuti u
/// jednoj točki. Drugo mjesto dodira znači da jedan prelazi drugi kroz vrhove ili da dijele
/// unutrašnjost lika na dva dijela, a oboje je presjek.
#[derive(Default)]
struct DodiriPrstenova(std::collections::HashMap<(usize, usize), Tocka>);

impl DodiriPrstenova {
    fn dodaj(&mut self, prstenovi: (usize, usize), t: Tocka) -> Option<Tocka> {
        let prvi = *self.0.entry(prstenovi).or_insert(t);
        (prvi != t).then_some(t)
    }
}

/// Je li dodir dvaju rubova presjek. Unutar prstena se broji svaki dodir rubova koji nisu
/// susjedi. Između prstenova križanje i preklapanje su presjek, a dodir u vrhu tek drugi put.
fn presjek_rubova(a: &Rub, b: &Rub, dodiri: &mut DodiriPrstenova) -> Option<Tocka> {
    match dodir(a.krajevi, b.krajevi)? {
        Dodir::Krizanje(t) | Dodir::Preklapanje(t) => Some(t),
        Dodir::Kraj(t) if a.prsten == b.prsten => Some(t),
        Dodir::Kraj(t) => dodiri.dodaj((a.prsten.min(b.prsten), a.prsten.max(b.prsten)), t),
    }
}

/// Vrh u kojem se rub vraća po sebi, npr. šiljak `a`–`b`–`a` s različitim duljinama: na istom
/// pravcu, a jedan od susjeda leži na rubu prema drugome.
fn povratak(prsten: &[Tocka]) -> Option<Tocka> {
    let n = prsten.len();
    (0..n).find_map(|i| {
        let [a, b, c] = [prsten[(i + n - 1) % n], prsten[i], prsten[(i + 1) % n]];
        let natrag = unutar(b, a, c) || unutar(b, c, a);
        (orijentacija(a, b, c) == 0.0 && natrag).then_some(b)
    })
}

/// Prstenovi s bar tri različita vrha; manje od toga još nije lik.
fn prstenovi_lika<P: AsRef<[Tocka]>>(prstenovi: &[P]) -> Vec<Vec<Tocka>> {
    prstenovi
        .iter()
        .map(|p| ocisti(p.as_ref()))
        .filter(|p| p.len() >= 3)
        .collect()
}

fn rubovi(prstenovi: &[Vec<Tocka>]) -> Vec<Rub> {
    let mut v = Vec::new();
    for (prsten, tocke) in prstenovi.iter().enumerate() {
        let n = tocke.len();
        v.extend((0..n).map(|redni| Rub {
            prsten,
            redni,
            duljina_prstena: n,
            krajevi: [tocke[redni], tocke[(redni + 1) % n]],
        }));
    }
    v
}

fn presjek_dvaju(a: &Rub, b: &Rub, dodiri: &mut DodiriPrstenova) -> Option<Tocka> {
    if a.susjed(b) {
        return None;
    }
    presjek_rubova(a, b, dodiri)
}

/// Mjesto na kojem lik siječe sam sebe, ili `None` ako ga nema. Prvi prsten je vanjski rub, a
/// ostali su rupe; smjer crtanja i zatvarajuća točka ne mijenjaju rezultat.
///
/// Rubovi se poredaju po najmanjoj dužini i uspoređuju samo s onima koji se s njima preklapaju po
/// dužini, pa uobičajen lik od milijun vrhova ne traži milijun puta milijun usporedbi.
pub fn presjek_lika<P: AsRef<[Tocka]>>(prstenovi: &[P]) -> Option<Tocka> {
    let prstenovi = prstenovi_lika(prstenovi);
    if let Some(t) = prstenovi.iter().find_map(|p| povratak(p)) {
        return Some(t);
    }
    let mut r = rubovi(&prstenovi);
    r.sort_by(|a, b| a.x_raspon()[0].total_cmp(&b.x_raspon()[0]));
    let mut dodiri = DodiriPrstenova::default();
    r.iter().enumerate().find_map(|(i, a)| {
        let kraj = a.x_raspon()[1];
        r[i + 1..]
            .iter()
            .take_while(|b| b.x_raspon()[0] <= kraj)
            .find_map(|b| presjek_dvaju(a, b, &mut dodiri))
    })
}

#[cfg(test)]
#[path = "presjek_testovi.rs"]
mod testovi;
