//! Nazivi objekata na karti. Ono što ovisi samo o sloju priprema se u metrima Mercatora i čuva dok
//! se sloj ne promijeni, a mjesto natpisa traži se u svakom okviru, jer ovisi o zumu i o tome što je
//! već zauzeto.
//!
//! Pravila su kao na tiskanoj karti: naziv linije ide uz liniju i prati zavoj, naziv točke stoji uz
//! znak, a naziv lika u liku. Natpis koji ne stane ili bi prekrio drugi ne crta se, pa se karta kod
//! udaljavanja sama prorijedi. Prednost ima gornji sloj.

use crate::kamera::Kamera;
use crate::mercator::u_mercator;
use eframe::egui::text::{LayoutJob, TextFormat};
use eframe::egui::{Color32, FontFamily, FontId, Galley, Painter, Pos2, Rect, Shape, pos2, vec2};
use eframe::epaint::TextShape;
use slojevi::{Boja, Geometrija, IdObjekta, Natpis, Objekt, Sloj, Vrh};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::Arc;

const TOCAKA_PO_MM: f64 = 96.0 / 25.4;
const TOCAKA_PO_PT: f64 = 96.0 / 72.0;
/// Koliko se često natpis ponavlja na dugoj liniji, u točkama ekrana (od 500 do 1000).
const RAZMAK_PX: f64 = 500.0;
/// Najveći zaokret između dva susjedna slova, kao u QGIS-u.
const NAJVECI_ZAOKRET: f64 = 25.0 * PI / 180.0;
/// Prazan prostor oko natpisa, da se dva natpisa ne lijepe jedan uz drugi.
const RAZMAK_OKO: f32 = 2.0;
/// Razmak između znaka točke i njezina naziva.
const ODMAK_OD_ZNAKA: f32 = 4.0;
const CELIJA: f32 = 64.0;

/// Gdje natpis objekta može stajati, u metrima Mercatora.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Mjesto {
    Tocka {
        t: [f64; 2],
        polumjer: f32,
    },
    Linija(Vec<[f64; 2]>),
    /// Sredina najšireg vodoravnog presjeka lika i koliko je taj presjek širok.
    Lik {
        sredina: [f64; 2],
        sirina_m: f64,
        visina_m: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ObjektNatpisa {
    pub naziv: String,
    pub okvir: [f64; 4],
    pub mjesto: Mjesto,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NatpisiSloja {
    pub natpis: Natpis,
    pub objekti: Vec<ObjektNatpisa>,
}

fn u_metre(vrhovi: &[Vrh]) -> Vec<[f64; 2]> {
    vrhovi
        .iter()
        .map(|v| u_mercator(v.duzina, v.sirina))
        .collect()
}

fn okvir_tocaka(tocke: &[[f64; 2]]) -> [f64; 4] {
    tocke
        .iter()
        .fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |o, t| {
            [
                o[0].min(t[0]),
                o[1].min(t[1]),
                o[2].max(t[0]),
                o[3].max(t[1]),
            ]
        })
}

/// Sjecišta vodoravnog pravca s rubovima svih prstenova, slijeva nadesno.
fn sjecista(prstenovi: &[Vec<[f64; 2]>], y: f64) -> Vec<f64> {
    let mut x: Vec<f64> = prstenovi
        .iter()
        .flat_map(|p| p.iter().zip(p.iter().cycle().skip(1)))
        .filter(|(a, b)| (a[1] > y) != (b[1] > y))
        .map(|(a, b)| a[0] + (y - a[1]) * (b[0] - a[0]) / (b[1] - a[1]))
        .collect();
    x.sort_by(f64::total_cmp);
    x
}

/// Najširi komad pravca unutar lika: sredina i širina. Rupe se preskaču pravilom par-nepar.
fn najsiri_presjek(prstenovi: &[Vec<[f64; 2]>], y: f64) -> Option<([f64; 2], f64)> {
    sjecista(prstenovi, y)
        .as_chunks::<2>()
        .0
        .iter()
        .map(|[a, b]| ([(a + b) / 2.0, y], b - a))
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// Točka sigurno unutar lika, i kad je lik udubljen ili ima rupu, kao „točka na površini” u GIS-u.
pub(crate) fn tocka_unutar(prstenovi: &[Vec<[f64; 2]>]) -> Option<([f64; 2], f64)> {
    let o = okvir_tocaka(prstenovi.first()?);
    let visina = o[3] - o[1];
    // Mali pomak od sredine, da pravac ne prolazi točno kroz vrh.
    [0.5, 0.35, 0.65]
        .iter()
        .filter_map(|k| najsiri_presjek(prstenovi, o[1] + visina * (k + 1e-7)))
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

fn mjesto(objekt: &Objekt, velicina_znaka_mm: f64) -> Option<(Mjesto, [f64; 4])> {
    match &objekt.geometrija {
        Geometrija::Tocka(v) => {
            let t = u_mercator(v.duzina, v.sirina);
            let polumjer = (velicina_znaka_mm * TOCAKA_PO_MM / 2.0) as f32;
            Some((Mjesto::Tocka { t, polumjer }, [t[0], t[1], t[0], t[1]]))
        }
        Geometrija::Linija(v) => {
            let t = u_metre(v);
            let o = okvir_tocaka(&t);
            Some((Mjesto::Linija(t), o))
        }
        Geometrija::Poligon(p) => {
            let prstenovi: Vec<Vec<[f64; 2]>> = p.iter().map(|r| u_metre(r)).collect();
            let o = okvir_tocaka(prstenovi.first()?);
            let (sredina, sirina_m) = tocka_unutar(&prstenovi)?;
            let visina_m = o[3] - o[1];
            Some((
                Mjesto::Lik {
                    sredina,
                    sirina_m,
                    visina_m,
                },
                o,
            ))
        }
    }
}

/// Priprema natpise sloja. Točka bez naziva ostaje, jer ni njezin znak ne smije biti prekriven.
pub(crate) fn pripremi_natpise(sloj: &Sloj, skriveni: Option<IdObjekta>) -> NatpisiSloja {
    let objekti = sloj
        .objekti
        .values()
        .filter(|o| Some(o.id) != skriveni)
        .filter_map(|o| {
            let stil = o.stil.na_stilu_sloja(&sloj.stil);
            let naziv = o.naziv.trim().to_owned();
            let (mjesto, okvir) = mjesto(o, stil.velicina_znaka_mm)?;
            let tocka = matches!(mjesto, Mjesto::Tocka { .. });
            (tocka || !naziv.is_empty()).then_some(ObjektNatpisa {
                naziv,
                okvir,
                mjesto,
            })
        })
        .collect();
    NatpisiSloja {
        natpis: sloj.stil.natpis,
        objekti,
    }
}

/// Već zauzeta mjesta na ekranu, po ćelijama, da provjera ne prolazi kroz sve natpise.
#[derive(Default)]
pub(crate) struct Zauzeto {
    celije: HashMap<(i32, i32), Vec<Rect>>,
}

impl Zauzeto {
    fn celije(r: Rect) -> impl Iterator<Item = (i32, i32)> {
        let od = (
            (r.min.x / CELIJA).floor() as i32,
            (r.min.y / CELIJA).floor() as i32,
        );
        let do_ = (
            (r.max.x / CELIJA).floor() as i32,
            (r.max.y / CELIJA).floor() as i32,
        );
        (od.0..=do_.0).flat_map(move |x| (od.1..=do_.1).map(move |y| (x, y)))
    }

    pub fn slobodno(&self, r: Rect) -> bool {
        Self::celije(r).all(|c| {
            self.celije
                .get(&c)
                .is_none_or(|v| v.iter().all(|z| !z.intersects(r)))
        })
    }

    pub fn zauzmi(&mut self, r: Rect) {
        for c in Self::celije(r) {
            self.celije.entry(c).or_default().push(r);
        }
    }
}

/// Mjere natpisa: lijevi rub i širina svakog slova, te cijela širina i visina.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tekst {
    pub lijevo: Vec<f32>,
    pub sirine: Vec<f32>,
    pub sirina: f32,
    pub visina: f32,
}

/// Jedno slovo natpisa uz liniju.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Slovo {
    pub indeks: usize,
    pub sredina: Pos2,
    pub kut: f32,
}

/// Linija na ekranu s duljinom do svakog vrha. U `f64`, jer je duga linija daleko izvan ekrana.
pub(crate) struct Put {
    tocke: Vec<[f64; 2]>,
    duljine: Vec<f64>,
}

impl Put {
    pub fn novi(tocke: Vec<[f64; 2]>) -> Self {
        let mut duljine = Vec::with_capacity(tocke.len());
        let mut d = 0.0;
        for (i, t) in tocke.iter().enumerate() {
            if i > 0 {
                d += (t[0] - tocke[i - 1][0]).hypot(t[1] - tocke[i - 1][1]);
            }
            duljine.push(d);
        }
        Self { tocke, duljine }
    }

    pub fn duljina(&self) -> f64 {
        self.duljine.last().copied().unwrap_or(0.0)
    }

    /// Točka na udaljenosti `s` od početka.
    pub fn tocka(&self, s: f64) -> [f64; 2] {
        let i = self
            .duljine
            .partition_point(|d| *d < s)
            .clamp(1, self.tocke.len() - 1);
        let (a, b) = (self.tocke[i - 1], self.tocke[i]);
        let dio = self.duljine[i] - self.duljine[i - 1];
        let k = if dio > 0.0 {
            ((s - self.duljine[i - 1]) / dio).clamp(0.0, 1.0)
        } else {
            0.0
        };
        [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k]
    }
}

/// Mjesta na liniji gdje natpis može biti, jednako razmaknuta i u sredini linije. Ovise samo o
/// duljini i razmaku, pa se ne pomiču dok se karta vuče.
pub(crate) fn sredista(duljina: f64, razmak: f64) -> Vec<f64> {
    if duljina < razmak {
        return vec![duljina / 2.0];
    }
    let n = (duljina / razmak).floor();
    let pocetak = (duljina - n * razmak) / 2.0 + razmak / 2.0;
    (0..n as usize)
        .map(|k| pocetak + k as f64 * razmak)
        .collect()
}

/// Razmak u točkama ekrana za ovaj zum. Raste u skokovima po razinama zuma, da natpisi ne kližu dok
/// se zumira unutar iste razine.
pub(crate) fn razmak_na_ekranu(metara_po_px: f64) -> f64 {
    let razina = 2f64.powf(metara_po_px.log2().ceil());
    RAZMAK_PX * razina / metara_po_px
}

fn razlika_kuta(a: f64, b: f64) -> f64 {
    let d = (b - a).rem_euclid(2.0 * PI);
    if d > PI { 2.0 * PI - d } else { d }
}

/// Okvir zakrenutog slova, za provjeru preklapanja.
fn okvir_slova(sredina: Pos2, sirina: f32, visina: f32, kut: f32) -> Rect {
    let (s, c) = kut.sin_cos();
    let w = (sirina * c).abs() + (visina * s).abs();
    let h = (sirina * s).abs() + (visina * c).abs();
    Rect::from_center_size(sredina, vec2(w, h)).expand(RAZMAK_OKO)
}

/// Slova natpisa sa sredinom u `s`. Čita se slijeva nadesno, pa se na liniji koja ide ulijevo
/// natpis okreće.
fn slova_na_mjestu(put: &Put, s: f64, t: &Tekst) -> Option<Vec<Slovo>> {
    let pola = f64::from(t.sirina) / 2.0;
    if s - pola < 0.0 || s + pola > put.duljina() {
        return None;
    }
    let smjer = if put.tocka(s + pola)[0] >= put.tocka(s - pola)[0] {
        1.0
    } else {
        -1.0
    };
    let mut slova = Vec::with_capacity(t.sirine.len());
    let mut prethodni: Option<f64> = None;
    for (i, (l, w)) in t.lijevo.iter().zip(&t.sirine).enumerate() {
        let u = s + smjer * (f64::from(*l + w / 2.0) - pola);
        let pola_slova = smjer * f64::from(*w) / 2.0;
        let (a, b) = (put.tocka(u - pola_slova), put.tocka(u + pola_slova));
        let kut = (b[1] - a[1]).atan2(b[0] - a[0]);
        if prethodni.is_some_and(|p| razlika_kuta(p, kut) > NAJVECI_ZAOKRET) {
            return None;
        }
        prethodni = Some(kut);
        let sredina = pos2(((a[0] + b[0]) / 2.0) as f32, ((a[1] + b[1]) / 2.0) as f32);
        slova.push(Slovo {
            indeks: i,
            sredina,
            kut: kut as f32,
        });
    }
    Some(slova)
}

fn okviri_slova(slova: &[Slovo], t: &Tekst) -> Vec<Rect> {
    slova
        .iter()
        .map(|s| okvir_slova(s.sredina, t.sirine[s.indeks], t.visina, s.kut))
        .collect()
}

/// Smješta natpis uz liniju na svako slobodno mjesto. Ako je na mjestu oštar zavoj, pokušava malo
/// prije i poslije.
pub(crate) fn smjesti_uz_liniju(
    put: &Put,
    razmak: f64,
    t: &Tekst,
    ekran: Rect,
    z: &mut Zauzeto,
) -> Vec<Vec<Slovo>> {
    let pomak = f64::from(t.sirina) * 0.75;
    let mut izlaz = Vec::new();
    for s in sredista(put.duljina(), razmak) {
        let nadjeno = [s, s + pomak, s - pomak].into_iter().find_map(|s| {
            let slova = slova_na_mjestu(put, s, t)?;
            let okviri = okviri_slova(&slova, t);
            let vidi = okviri.iter().all(|o| ekran.contains_rect(*o));
            (vidi && okviri.iter().all(|o| z.slobodno(*o))).then_some((slova, okviri))
        });
        if let Some((slova, okviri)) = nadjeno {
            okviri.into_iter().for_each(|o| z.zauzmi(o));
            izlaz.push(slova);
        }
    }
    izlaz
}

/// Natpis točke: desno od znaka, a ako tamo nema mjesta lijevo, iznad ili ispod.
pub(crate) fn smjesti_uz_tocku(
    t: Pos2,
    polumjer: f32,
    tekst: &Tekst,
    ekran: Rect,
    z: &mut Zauzeto,
) -> Option<Rect> {
    let velicina = vec2(tekst.sirina, tekst.visina);
    let d = polumjer + ODMAK_OD_ZNAKA;
    let mjesta = [
        Rect::from_min_size(pos2(t.x + d, t.y - velicina.y / 2.0), velicina),
        Rect::from_min_size(pos2(t.x - d - velicina.x, t.y - velicina.y / 2.0), velicina),
        Rect::from_center_size(pos2(t.x, t.y - d - velicina.y / 2.0), velicina),
        Rect::from_center_size(pos2(t.x, t.y + d + velicina.y / 2.0), velicina),
    ];
    let r = mjesta
        .into_iter()
        .find(|r| ekran.contains_rect(*r) && z.slobodno(r.expand(RAZMAK_OKO)))?;
    z.zauzmi(r.expand(RAZMAK_OKO));
    Some(r)
}

/// Natpis lika, u sredini, samo kad stane u lik.
pub(crate) fn smjesti_u_lik(
    sredina: Pos2,
    sirina: f32,
    visina: f32,
    tekst: &Tekst,
    ekran: Rect,
    z: &mut Zauzeto,
) -> Option<Rect> {
    if tekst.sirina + 2.0 * ODMAK_OD_ZNAKA > sirina || tekst.visina > visina {
        return None;
    }
    let r = Rect::from_center_size(sredina, vec2(tekst.sirina, tekst.visina));
    (ekran.contains_rect(r) && z.slobodno(r.expand(RAZMAK_OKO))).then(|| {
        z.zauzmi(r.expand(RAZMAK_OKO));
        r
    })
}

/// Pretvorba iz metara Mercatora u točke ekrana.
#[derive(Clone, Copy)]
pub(crate) struct Ekran {
    pub kamera: Kamera,
    pub okvir: Rect,
}

impl Ekran {
    fn tocka(&self, t: [f64; 2]) -> [f64; 2] {
        let k = &self.kamera;
        let c = self.okvir.center();
        [
            f64::from(c.x) + (t[0] - k.sredina[0]) / k.metara_po_px,
            f64::from(c.y) - (t[1] - k.sredina[1]) / k.metara_po_px,
        ]
    }

    fn pos(&self, t: [f64; 2]) -> Pos2 {
        let [x, y] = self.tocka(t);
        pos2(x as f32, y as f32)
    }
}

fn boja(b: Boja) -> Color32 {
    Color32::from_rgba_unmultiplied(b.r, b.g, b.b, b.a)
}

/// Svijetla slova dobivaju tamni obrub, a tamna bijeli.
fn boja_obruba(b: Boja) -> Color32 {
    let svjetlina = 0.299 * f32::from(b.r) + 0.587 * f32::from(b.g) + 0.114 * f32::from(b.b);
    if svjetlina > 150.0 {
        Color32::from_rgba_unmultiplied(0x20, 0x21, 0x24, 220)
    } else {
        Color32::from_rgba_unmultiplied(255, 255, 255, 230)
    }
}

/// Izgled jednog sloja natpisa u ovom okviru.
struct Oblikovanje {
    format: TextFormat,
    obrub: Option<Color32>,
}

impl Oblikovanje {
    fn novo(n: &Natpis) -> Self {
        let format = TextFormat {
            font_id: FontId::new(
                (n.velicina_pt * TOCAKA_PO_PT) as f32,
                FontFamily::Proportional,
            ),
            color: boja(n.boja),
            italics: n.kurziv,
            ..Default::default()
        };
        Self {
            format,
            obrub: n.obrub.then(|| boja_obruba(n.boja)),
        }
    }

    fn galija(&self, s: &Painter, tekst: &str) -> Arc<Galley> {
        s.layout_job(LayoutJob::single_section(
            tekst.to_owned(),
            self.format.clone(),
        ))
    }
}

fn mjere(g: &Galley) -> Tekst {
    let glifovi = g
        .rows
        .first()
        .map(|r| r.glyphs.as_slice())
        .unwrap_or_default();
    Tekst {
        lijevo: glifovi.iter().map(|g| g.pos.x).collect(),
        sirine: glifovi.iter().map(|g| g.advance_width).collect(),
        sirina: g.size().x,
        visina: g.size().y,
    }
}

/// Pomaci obruba: slova se crtaju osam puta u boji obruba oko pravog mjesta.
const POMACI_OBRUBA: [(f32, f32); 8] = [
    (-1.5, 0.0),
    (1.5, 0.0),
    (0.0, -1.5),
    (0.0, 1.5),
    (-1.1, -1.1),
    (1.1, -1.1),
    (-1.1, 1.1),
    (1.1, 1.1),
];

fn crtaj_galiju(s: &Painter, gornji_lijevi: Pos2, kut: f32, g: &Arc<Galley>, o: &Oblikovanje) {
    if let Some(obrub) = o.obrub {
        for (dx, dy) in POMACI_OBRUBA {
            let sjena = TextShape::new(gornji_lijevi + vec2(dx, dy), g.clone(), obrub)
                .with_override_text_color(obrub)
                .with_angle(kut);
            s.add(Shape::Text(sjena));
        }
    }
    let tekst = TextShape::new(gornji_lijevi, g.clone(), o.format.color).with_angle(kut);
    s.add(Shape::Text(tekst));
}

/// Slovo se zakreće oko svoje sredine, a `egui` ga zakreće oko gornjeg lijevog kuta.
fn crtaj_slova(s: &Painter, naziv: &str, slova: &[Slovo], o: &Oblikovanje) {
    let znakovi: Vec<char> = naziv.chars().collect();
    for slovo in slova {
        let Some(z) = znakovi.get(slovo.indeks) else {
            continue;
        };
        let g = o.galija(s, &z.to_string());
        let pola = g.size() / 2.0;
        let (sin, cos) = slovo.kut.sin_cos();
        let zakrenuto = vec2(pola.x * cos - pola.y * sin, pola.x * sin + pola.y * cos);
        crtaj_galiju(s, slovo.sredina - zakrenuto, slovo.kut, &g, o);
    }
}

/// Znakovi točaka koje se vide su zauzeti prije svih natpisa, da ih natpis ne prekrije.
fn zauzmi_znakove(slojevi: &[Arc<NatpisiSloja>], e: &Ekran, z: &mut Zauzeto) {
    for o in slojevi.iter().flat_map(|s| &s.objekti) {
        if let Mjesto::Tocka { t, polumjer } = o.mjesto
            && e.kamera.vidi(o.okvir)
        {
            z.zauzmi(Rect::from_center_size(
                e.pos(t),
                vec2(2.0 * polumjer, 2.0 * polumjer),
            ));
        }
    }
}

fn crtaj_objekt(s: &Painter, o: &ObjektNatpisa, e: &Ekran, ob: &Oblikovanje, z: &mut Zauzeto) {
    let g = ob.galija(s, &o.naziv);
    let t = mjere(&g);
    let ekran = e.okvir.shrink(2.0);
    let mpp = e.kamera.metara_po_px;
    match &o.mjesto {
        Mjesto::Tocka { t: m, polumjer } => {
            if let Some(r) = smjesti_uz_tocku(e.pos(*m), *polumjer, &t, ekran, z) {
                crtaj_galiju(s, r.min, 0.0, &g, ob);
            }
        }
        Mjesto::Linija(tocke) => {
            let put = Put::novi(tocke.iter().map(|m| e.tocka(*m)).collect());
            for slova in smjesti_uz_liniju(&put, razmak_na_ekranu(mpp), &t, ekran, z) {
                crtaj_slova(s, &o.naziv, &slova, ob);
            }
        }
        Mjesto::Lik {
            sredina,
            sirina_m,
            visina_m,
        } => {
            let (w, h) = ((sirina_m / mpp) as f32, (visina_m / mpp) as f32);
            if let Some(r) = smjesti_u_lik(e.pos(*sredina), w, h, &t, ekran, z) {
                crtaj_galiju(s, r.min, 0.0, &g, ob);
            }
        }
    }
}

/// Crta natpise slojeva, redom od gornjeg sloja prema donjem.
pub(crate) fn nacrtaj(s: &Painter, e: &Ekran, slojevi: &[Arc<NatpisiSloja>]) {
    let mut z = Zauzeto::default();
    zauzmi_znakove(slojevi, e, &mut z);
    for sloj in slojevi.iter().filter(|s| s.natpis.prikazi) {
        let ob = Oblikovanje::novo(&sloj.natpis);
        sloj.objekti
            .iter()
            .filter(|o| !o.naziv.is_empty() && e.kamera.vidi(o.okvir))
            .for_each(|o| crtaj_objekt(s, o, e, &ob, &mut z));
    }
}

#[cfg(test)]
#[path = "natpisi_testovi.rs"]
mod testovi;
