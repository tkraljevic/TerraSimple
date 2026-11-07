//! Proba ispisa: A4 ili A3 u mjerilu 1 : 5 000 kao vektorski PDF.
//!
//! `proba_ispis <a4|a3> <radna mapa> [podloga.jpg]`
//!
//! Piše `nas.pdf`, a uz njega `objekti.csv` i `list.csv` da QGIS može nacrtati isti
//! list. Objekti su izmišljeni. Podloga, ako je zadana, ide na drugu stranicu.

use krilla::color::rgb;
use krilla::geom::{Path, PathBuilder, Point, Size, Transform};
use krilla::image::Image;
use krilla::metadata::Metadata;
use krilla::num::NormalizedF32;
use krilla::page::PageSettings;
use krilla::paint::{Fill, FillRule, LineCap, LineJoin, Stroke, StrokeDash};
use krilla::surface::Surface;
use krilla::text::{Font, TextDirection};
use krilla::Document;
use std::fmt::Write as _;

const MJERILO: f64 = 5000.0;
const ZAPAD: f64 = 670_000.0;
const SJEVER: f64 = 5_048_550.0;
const TEKST: &str = "#1F2937";

/// Papir i okvir karte u milimetrima. Gornji lijevi kut karte uvijek je isto mjesto.
#[derive(Clone, Copy)]
struct List {
    ime: &'static str,
    sirina: f64,
    visina: f64,
    x: f64,
    y: f64,
    okvir_s: f64,
    okvir_v: f64,
}

const A4: List = List { ime: "A4", sirina: 297.0, visina: 210.0, x: 8.5, y: 10.0, okvir_s: 280.0, okvir_v: 170.0 };
const A3: List = List { ime: "A3", sirina: 420.0, visina: 297.0, x: 10.0, y: 10.0, okvir_s: 400.0, okvir_v: 250.0 };

impl List {
    fn istok(&self) -> f64 {
        ZAPAD + self.okvir_s * MJERILO / 1000.0
    }

    fn jug(&self) -> f64 {
        SJEVER - self.okvir_v * MJERILO / 1000.0
    }

    /// Metri na karti u milimetre na papiru.
    fn u_mm(&self, e: f64, n: f64) -> (f64, f64) {
        (self.x + (e - ZAPAD) * 1000.0 / MJERILO, self.y + (SJEVER - n) * 1000.0 / MJERILO)
    }

    fn redak(&self) -> String {
        format!(
            "ime;sirina;visina;x;y;okvir_s;okvir_v;zapad;jug;istok;sjever\n{};{};{};{};{};{};{};{};{};{};{}\n",
            self.ime,
            self.sirina,
            self.visina,
            self.x,
            self.y,
            self.okvir_s,
            self.okvir_v,
            ZAPAD,
            self.jug(),
            self.istok(),
            SJEVER
        )
    }
}

fn pt(mm: f64) -> f32 {
    (mm * 72.0 / 25.4) as f32
}

fn boja(hex: &str) -> rgb::Color {
    let k = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).expect("boja");
    rgb::Color::new(k(1), k(3), k(5))
}

fn prozirno(v: f64) -> NormalizedF32 {
    NormalizedF32::new(v as f32).expect("prozirnost")
}

enum Oblik {
    Tocka { e: f64, n: f64, znak: &'static str },
    Crte(Vec<Vec<(f64, f64)>>),
    Lik(Vec<Vec<(f64, f64)>>),
}

struct Objekt {
    ime: &'static str,
    boja: &'static str,
    ispuna: &'static str,
    prozirnost: f64,
    /// Debljina crte, a za točku promjer znaka, u milimetrima.
    debljina: f64,
    crtice: Option<(f64, f64)>,
    mjeri: bool,
    oblik: Oblik,
}

impl Objekt {
    fn wkt(&self) -> String {
        let niz = |t: &[(f64, f64)]| {
            let dijelovi: Vec<String> = t.iter().map(|(e, n)| format!("{e} {n}")).collect();
            format!("({})", dijelovi.join(","))
        };
        let nizovi = |d: &[Vec<(f64, f64)>]| {
            let dijelovi: Vec<String> = d.iter().map(|t| niz(t)).collect();
            format!("({})", dijelovi.join(","))
        };
        match &self.oblik {
            Oblik::Tocka { e, n, .. } => format!("POINT({e} {n})"),
            Oblik::Crte(d) => format!("MULTILINESTRING{}", nizovi(d)),
            Oblik::Lik(d) => format!("POLYGON{}", nizovi(d)),
        }
    }

    fn redak(&self) -> String {
        let (vrsta, znak) = match &self.oblik {
            Oblik::Tocka { znak, .. } => ("tocka", *znak),
            Oblik::Crte(_) => ("crta", ""),
            Oblik::Lik(_) => ("lik", ""),
        };
        let crtice = self.crtice.map_or(String::new(), |(a, b)| format!("\"{a};{b}\""));
        format!(
            "{};{vrsta};{};{};{};{};{crtice};{znak};{};{}\n",
            self.ime,
            self.boja,
            self.ispuna,
            self.prozirnost,
            self.debljina,
            if self.mjeri { "1" } else { "" },
            self.wkt()
        )
    }
}

/// Položaj od gornjeg lijevog kuta karte, u metrima prema istoku i jugu.
fn od_kuta(desno: f64, dolje: f64) -> (f64, f64) {
    (ZAPAD + desno, SJEVER - dolje)
}

fn pravokutnik(l: f64, g: f64, d: f64, n: f64) -> Vec<(f64, f64)> {
    vec![od_kuta(l, n), od_kuta(d, n), od_kuta(d, g), od_kuta(l, g), od_kuta(l, n)]
}

fn mreza(list: &List) -> Vec<Vec<(f64, f64)>> {
    let mut crte = Vec::new();
    let mut e = ZAPAD + 100.0;
    while e < list.istok() {
        crte.push(vec![(e, list.jug()), (e, SJEVER)]);
        e += 100.0;
    }
    let mut n = SJEVER - 100.0;
    while n > list.jug() {
        crte.push(vec![(ZAPAD, n), (list.istok(), n)]);
        n -= 100.0;
    }
    crte
}

/// Svi objekti stanu i na A4, pa su na oba lista na istom mjestu od gornjeg lijevog kuta.
fn objekti(list: &List) -> Vec<Objekt> {
    let obican = |ime, boja, debljina, oblik| Objekt {
        ime,
        boja,
        ispuna: "",
        prozirnost: 1.0,
        debljina,
        crtice: None,
        mjeri: true,
        oblik,
    };
    let crta_od = |a: (f64, f64), b: (f64, f64)| Oblik::Crte(vec![vec![od_kuta(a.0, a.1), od_kuta(b.0, b.1)]]);
    let tocka = |d: f64, g: f64, znak| {
        let (e, n) = od_kuta(d, g);
        Oblik::Tocka { e, n, znak }
    };
    let okvir = vec![
        (ZAPAD, list.jug()),
        (list.istok(), list.jug()),
        (list.istok(), SJEVER),
        (ZAPAD, SJEVER),
        (ZAPAD, list.jug()),
    ];
    vec![
        Objekt { mjeri: false, ..obican("Mreža", "#B4B4B4", 0.1, Oblik::Crte(mreza(list))) },
        Objekt {
            ispuna: "#FFC000",
            ..obican(
                "Lik s rupom",
                "#8B3A00",
                0.4,
                Oblik::Lik(vec![pravokutnik(100.0, 100.0, 500.0, 500.0), pravokutnik(200.0, 200.0, 400.0, 400.0)]),
            )
        },
        Objekt {
            ispuna: "#2060E0",
            ..obican("Plavi lik", "#102070", 0.6, Oblik::Lik(vec![pravokutnik(650.0, 100.0, 950.0, 400.0)]))
        },
        Objekt {
            ispuna: "#A000A0",
            prozirnost: 0.5,
            mjeri: false,
            ..obican("Prozirni lik", "#A000A0", 0.2, Oblik::Lik(vec![pravokutnik(800.0, 150.0, 1150.0, 350.0)]))
        },
        obican("Kosa crta", "#FF8000", 0.5, crta_od((50.0, 800.0), (1350.0, 50.0))),
        obican("Crta 1 km", "#FF0000", 0.5, crta_od((100.0, 650.0), (1100.0, 650.0))),
        obican("Crta 500 m", "#00A000", 0.5, crta_od((1230.0, 100.0), (1230.0, 600.0))),
        Objekt {
            crtice: Some((4.0, 2.0)),
            ..obican("Isprekidana", "#00C0C0", 1.0, crta_od((100.0, 750.0), (1200.0, 750.0)))
        },
        obican("Krug", "#E000E0", 4.0, tocka(1300.0, 150.0, "circle")),
        obican("Kvadrat", "#604000", 4.0, tocka(1300.0, 300.0, "square")),
        obican("Okvir", "#006060", 0.3, Oblik::Crte(vec![okvir])),
    ]
}

fn put_niza(list: &List, nizovi: &[Vec<(f64, f64)>], zatvori: bool) -> Path {
    let mut p = PathBuilder::new();
    for niz in nizovi {
        for (i, (e, n)) in niz.iter().enumerate() {
            let (x, y) = list.u_mm(*e, *n);
            if i == 0 {
                p.move_to(pt(x), pt(y));
            } else {
                p.line_to(pt(x), pt(y));
            }
        }
        if zatvori {
            p.close();
        }
    }
    p.finish().expect("put")
}

fn krug(x: f64, y: f64, r: f64) -> Path {
    const K: f64 = 0.552_284_749_8;
    let mut p = PathBuilder::new();
    p.move_to(pt(x + r), pt(y));
    for (a, b, c, d, e, f) in [
        (r, K * r, K * r, r, 0.0, r),
        (-K * r, r, -r, K * r, -r, 0.0),
        (-r, -K * r, -K * r, -r, 0.0, -r),
        (K * r, -r, r, -K * r, r, 0.0),
    ] {
        p.cubic_to(pt(x + a), pt(y + b), pt(x + c), pt(y + d), pt(x + e), pt(y + f));
    }
    p.close();
    p.finish().expect("krug")
}

fn kvadrat(x: f64, y: f64, r: f64) -> Path {
    let mut p = PathBuilder::new();
    p.move_to(pt(x - r), pt(y - r));
    p.line_to(pt(x + r), pt(y - r));
    p.line_to(pt(x + r), pt(y + r));
    p.line_to(pt(x - r), pt(y + r));
    p.close();
    p.finish().expect("kvadrat")
}

fn ispuna(hex: &str, prozirnost: f64) -> Fill {
    Fill { paint: boja(hex).into(), opacity: prozirno(prozirnost), rule: FillRule::EvenOdd }
}

fn crta(hex: &str, debljina: f64, prozirnost: f64, crtice: Option<(f64, f64)>) -> Stroke {
    Stroke {
        paint: boja(hex).into(),
        width: pt(debljina),
        miter_limit: 10.0,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Miter,
        opacity: prozirno(prozirnost),
        dash: crtice.map(|(a, b)| StrokeDash { array: vec![pt(a), pt(b)], offset: 0.0 }),
    }
}

fn nacrtaj_objekt(s: &mut Surface, list: &List, o: &Objekt) {
    match &o.oblik {
        Oblik::Tocka { e, n, znak } => {
            let (x, y) = list.u_mm(*e, *n);
            s.set_stroke(None);
            s.set_fill(Some(ispuna(o.boja, o.prozirnost)));
            let r = o.debljina / 2.0;
            let put = if *znak == "circle" { krug(x, y, r) } else { kvadrat(x, y, r) };
            s.draw_path(&put);
        }
        Oblik::Crte(d) => {
            s.set_fill(None);
            s.set_stroke(Some(crta(o.boja, o.debljina, o.prozirnost, o.crtice)));
            s.draw_path(&put_niza(list, d, false));
        }
        Oblik::Lik(d) => {
            s.set_fill(Some(ispuna(o.ispuna, o.prozirnost)));
            s.set_stroke(Some(crta(o.boja, o.debljina, o.prozirnost, None)));
            s.draw_path(&put_niza(list, d, true));
        }
    }
    s.set_fill(None);
    s.set_stroke(None);
}

fn pisi(s: &mut Surface, font: &Font, x: f64, y: f64, velicina: f32, tekst: &str) {
    s.set_fill(Some(ispuna(TEKST, 1.0)));
    s.set_stroke(None);
    s.draw_text(Point::from_xy(pt(x), pt(y)), font.clone(), velicina, tekst, false, TextDirection::Auto);
    s.set_fill(None);
}

/// Bijeli obrub ispod natpisa, da se tekst čita na ortofotu. Crta se kao put, pa se tekst
/// pri kopiranju iz PDF-a ne ponavlja.
fn obrub(s: &mut Surface, font: &Font, x: f64, y: f64, velicina: f32, tekst: &str) {
    s.set_fill(None);
    s.set_stroke(Some(Stroke { line_join: LineJoin::Round, ..crta("#FFFFFF", 0.7, 1.0, None) }));
    s.draw_text(Point::from_xy(pt(x), pt(y)), font.clone(), velicina, tekst, true, TextDirection::Auto);
    s.set_stroke(None);
}

fn natpisi_karte(s: &mut Surface, list: &List, font: &Font, s_obrubom: bool) {
    for (d, g, tekst) in [
        (110.0, 85.0, "Šećerana čćžšđ"),
        (660.0, 85.0, "Čađavica"),
        (450.0, 635.0, "Crta od 1 km mora biti duga 200 mm"),
        (1060.0, 480.0, "500 m = 100 mm"),
        (1325.0, 160.0, "Žažina"),
        (1325.0, 310.0, "Đakovo"),
    ] {
        let (e, n) = od_kuta(d, g);
        let (x, y) = list.u_mm(e, n);
        if s_obrubom {
            obrub(s, font, x, y, 9.0, tekst);
        }
        pisi(s, font, x, y, 9.0, tekst);
    }
}

fn traka_mjerila(s: &mut Surface, list: &List, font: &Font) {
    let x0 = list.x + list.okvir_s - 110.0;
    let y = list.y + list.okvir_v + 9.0;
    s.set_fill(None);
    s.set_stroke(Some(crta(TEKST, 0.3, 1.0, None)));
    let mut p = PathBuilder::new();
    p.move_to(pt(x0), pt(y));
    p.line_to(pt(x0 + 100.0), pt(y));
    for i in 0..=10 {
        let x = x0 + f64::from(i) * 10.0;
        p.move_to(pt(x), pt(y));
        p.line_to(pt(x), pt(y + if i % 5 == 0 { 3.0 } else { 1.5 }));
    }
    s.draw_path(&p.finish().expect("traka"));
    s.set_stroke(None);
    for (x, tekst) in [(x0, "0"), (x0 + 50.0, "250 m"), (x0 + 100.0, "500 m")] {
        pisi(s, font, x, y - 1.5, 8.0, tekst);
    }
    pisi(s, font, x0, y + 7.0, 8.0, "Traka od 500 m duga je 100 mm.");
}

fn zaglavlje(s: &mut Surface, list: &List, font: &Font, opis: &str, mjere: &str) {
    let dno = list.y + list.okvir_v;
    pisi(s, font, list.x, dno + 6.0, 11.0, opis);
    pisi(s, font, list.x, dno + 11.5, 11.0, "Šećerana Čađavica Žažina Đakovo Vođinci Ćićarija — ČĆŽŠĐ čćžšđ");
    pisi(s, font, list.x, dno + 16.5, 9.0, "Ispisati u stvarnoj veličini (100 %), bez prilagodbe listu.");
    pisi(s, font, list.x, dno + 20.5, 9.0, mjere);
    traka_mjerila(s, list, font);
}

fn stranica_objekata(d: &mut Document, list: &List, font: &Font, sve: &[Objekt]) {
    let mut stranica = d.start_page_with(PageSettings::from_wh(pt(list.sirina), pt(list.visina)).expect("list"));
    let mut s = stranica.surface();
    for o in sve {
        nacrtaj_objekt(&mut s, list, o);
    }
    natpisi_karte(&mut s, list, font, false);
    zaglavlje(
        &mut s,
        list,
        font,
        &format!("Proba ispisa TerraSimple · mjerilo 1 : 5 000 · list {} · HTRS96/TM", list.ime),
        &format!("Razmak mreže je 20 mm, a okvir {} × {} mm.", list.okvir_s, list.okvir_v),
    );
    s.finish();
    stranica.finish();
}

fn stranica_podloge(d: &mut Document, list: &List, font: &Font, sve: &[Objekt], podloga: Image) {
    let mut stranica = d.start_page_with(PageSettings::from_wh(pt(list.sirina), pt(list.visina)).expect("list"));
    let mut s = stranica.surface();
    s.push_transform(&Transform::from_translate(pt(list.x), pt(list.y)));
    s.draw_image(podloga, Size::from_wh(pt(list.okvir_s), pt(list.okvir_v)).expect("veličina"));
    s.pop();
    for o in sve {
        nacrtaj_objekt(&mut s, list, o);
    }
    natpisi_karte(&mut s, list, font, true);
    zaglavlje(
        &mut s,
        list,
        font,
        &format!("Podloga DOF © Državna geodetska uprava · 1 : 5 000 · list {}", list.ime),
        &format!("Razmak mreže je 20 mm, a okvir {} × {} mm.", list.okvir_s, list.okvir_v),
    );
    s.finish();
    stranica.finish();
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let list = match a.get(1).map(String::as_str) {
        Some("a4") => A4,
        Some("a3") => A3,
        _ => {
            eprintln!("upotreba: proba_ispis <a4|a3> <radna mapa> [podloga.jpg]");
            std::process::exit(2);
        }
    };
    let mapa = a.get(2).expect("radna mapa");
    let sve = objekti(&list);
    let mut csv = String::from("ime;vrsta;boja;ispuna;prozirnost;debljina;crtice;znak;mjeri;wkt\n");
    for o in &sve {
        write!(csv, "{}", o.redak()).expect("csv");
    }
    std::fs::write(format!("{mapa}/objekti.csv"), csv).expect("pisanje objekata");
    std::fs::write(format!("{mapa}/list.csv"), list.redak()).expect("pisanje lista");

    let font = Font::new(include_bytes!("../../izgled/fonti/Inter-Regular.otf").to_vec().into(), 0)
        .expect("font");
    let mut d = Document::new();
    d.set_metadata(Metadata::new().title("Proba ispisa TerraSimple".into()).language("hr".into()));
    stranica_objekata(&mut d, &list, &font, &sve);
    if let Some(put) = a.get(3) {
        let bajtovi = std::fs::read(put).expect("čitanje podloge");
        let slika = Image::from_jpeg(bajtovi.into(), false).expect("podloga");
        println!("podloga {:?} piksela", slika.size());
        stranica_podloge(&mut d, &list, &font, &sve, slika);
    }
    let pdf = d.finish().expect("PDF");
    std::fs::write(format!("{mapa}/nas.pdf"), &pdf).expect("pisanje PDF-a");
    println!("{} {} KB", list.ime, pdf.len() / 1024);
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn kilometar_je_dvjesto_milimetara_na_oba_lista() {
        for list in [A4, A3] {
            let (x0, _) = list.u_mm(ZAPAD + 100.0, SJEVER);
            let (x1, _) = list.u_mm(ZAPAD + 1100.0, SJEVER);
            assert!((x1 - x0 - 200.0).abs() < 1e-9);
        }
    }

    #[test]
    fn okvir_tocno_pokriva_podrucje() {
        for list in [A4, A3] {
            assert_eq!(list.u_mm(ZAPAD, SJEVER), (list.x, list.y));
            let (x, y) = list.u_mm(list.istok(), list.jug());
            assert!((x - list.x - list.okvir_s).abs() < 1e-9);
            assert!((y - list.y - list.okvir_v).abs() < 1e-9);
        }
    }

    #[test]
    fn sjever_je_gore_na_papiru() {
        assert!(A4.u_mm(ZAPAD, SJEVER).1 < A4.u_mm(ZAPAD, A4.jug()).1);
    }

    #[test]
    fn okvir_stane_na_list() {
        for list in [A4, A3] {
            assert!(list.x + list.okvir_s < list.sirina);
            // Tekst završava 21 mm ispod okvira, a pisač ostavlja bar 5 mm ruba.
            assert!(list.y + list.okvir_v + 21.0 <= list.visina - 5.0, "{}", list.ime);
        }
    }

    #[test]
    fn listovi_u_tockama() {
        assert!((f64::from(pt(420.0)) - 1190.551).abs() < 0.001);
        assert!((f64::from(pt(297.0)) - 841.890).abs() < 0.001);
        assert!((f64::from(pt(210.0)) - 595.276).abs() < 0.001);
    }

    #[test]
    fn crtice_se_mogu_procitati_iz_retka() {
        let sve = objekti(&A4);
        let o = sve.iter().find(|o| o.crtice.is_some()).expect("isprekidana");
        assert!(o.redak().contains("\"4;2\""), "{}", o.redak());
    }

    #[test]
    fn svi_objekti_stanu_na_a4() {
        for o in objekti(&A4) {
            let tocke: Vec<(f64, f64)> = match &o.oblik {
                Oblik::Tocka { e, n, .. } => vec![(*e, *n)],
                Oblik::Crte(d) | Oblik::Lik(d) => d.iter().flatten().copied().collect(),
            };
            for (e, n) in tocke {
                assert!(e >= ZAPAD && e <= A4.istok() && n >= A4.jug() && n <= SJEVER, "{}", o.ime);
            }
        }
    }
}
