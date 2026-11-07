//! Crtanje na pravoj grafičkoj kartici, bez prozora, s provjerom piksela na računatim mjestima.
//!
//! Na CI-ju za Linux radi softverska kartica. Ako kartice nema, test pada, jer preskakanje ne bi
//! ništa dokazalo.

use karta::provjera::{Slika, nacrtaj_bez_prozora};
use slojevi::{
    Boja, Geometrija, IdSloja, NaredbaObjekta, NaredbaStabla, Novo, Stil, Uredjivac, Vrh,
    VrstaGeometrije,
};

const SIRINA: u32 = 800;
const VISINA: u32 = 600;
const ZUM: f64 = 14.0;
const SREDINA: [f64; 2] = [18.69, 45.55];
const CRVENO: Boja = Boja {
    r: 255,
    g: 0,
    b: 0,
    a: 255,
};
const PLAVO: Boja = Boja {
    r: 0,
    g: 0,
    b: 255,
    a: 255,
};

/// Isti račun kao u `walkers`-u, napisan neovisno o paketu: piksel od sredine ekrana.
fn piksel(duzina: f64, sirina: f64) -> (u32, u32) {
    let po_stupnju = 256.0 * 2.0_f64.powf(ZUM) / 360.0;
    let m = |s: f64| {
        (std::f64::consts::FRAC_PI_4 + s.to_radians() / 2.0)
            .tan()
            .ln()
            .to_degrees()
    };
    (
        (f64::from(SIRINA) / 2.0 + (duzina - SREDINA[0]) * po_stupnju) as u32,
        (f64::from(VISINA) / 2.0 - (m(sirina) - m(SREDINA[1])) * po_stupnju) as u32,
    )
}

fn sloj(u: &mut Uredjivac, vrsta: VrstaGeometrije) -> IdSloja {
    let n = NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Proba".into(),
        vrsta,
    };
    match u.izvrsi(n) {
        Ok(Some(Novo::Sloj(s))) => s,
        r => panic!("{r:?}"),
    }
}

fn dodaj(u: &mut Uredjivac, s: IdSloja, geometrija: Geometrija) {
    let n = NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: String::new(),
        geometrija,
    };
    u.izvrsi(n).expect("objekt");
}

fn oko(x: f64, y: f64) -> Vrh {
    Vrh::novi(SREDINA[0] + x, SREDINA[1] + y)
}

fn pravokutnik(x: f64, y: f64, sirina: f64, visina: f64) -> Vec<Vrh> {
    vec![
        oko(x, y),
        oko(x + sirina, y),
        oko(x + sirina, y + visina),
        oko(x, y + visina),
    ]
}

fn nacrtaj(u: &Uredjivac, zum: f64) -> Slika {
    pollster::block_on(nacrtaj_bez_prozora(u, SREDINA, zum, [SIRINA, VISINA])).expect("kartica")
}

fn na(slika: &Slika, x: f64, y: f64) -> [u8; 4] {
    let (px, py) = piksel(SREDINA[0] + x, SREDINA[1] + y);
    slika.piksel(px, py)
}

#[test]
fn ispuna_rupa_rub_crta_i_tocka_su_na_racunatim_mjestima() {
    let mut u = Uredjivac::novi();
    let lik = sloj(&mut u, VrstaGeometrije::Poligon);
    let g = vec![
        pravokutnik(-0.01, -0.005, 0.02, 0.01),
        pravokutnik(-0.002, -0.002, 0.004, 0.004),
    ];
    dodaj(&mut u, lik, Geometrija::Poligon(g));
    let crta = sloj(&mut u, VrstaGeometrije::Linija);
    dodaj(
        &mut u,
        crta,
        Geometrija::Linija(vec![oko(-0.015, 0.008), oko(0.015, 0.008)]),
    );
    let tocka = sloj(&mut u, VrstaGeometrije::Tocka);
    dodaj(&mut u, tocka, Geometrija::Tocka(oko(-0.012, -0.008)));
    let s = nacrtaj(&u, ZUM);

    let ispuna = na(&s, 0.006, 0.0);
    assert!(
        ispuna[3] == 64 && ispuna[0] > ispuna[2],
        "ispuna {ispuna:?}"
    );
    assert!(ispuna[0] < 64, "boja je pomnožena prozirnošću: {ispuna:?}");
    assert_eq!(na(&s, 0.0, 0.0), [0, 0, 0, 0], "rupa");
    assert_eq!(na(&s, 0.018, 0.0), [0, 0, 0, 0], "izvan lika");
    for (ime, x, y) in [
        ("crta", 0.0, 0.008),
        ("rub", 0.01, 0.0),
        ("točka", -0.012, -0.008),
    ] {
        let b = na(&s, x, y);
        assert!(b[3] > 200 && b[0] > 150, "{ime}: {b:?}");
    }
    assert_eq!(
        na(&s, 0.0, 0.0085 + 0.002),
        [0, 0, 0, 0],
        "crta nije deblja nego što treba"
    );
}

#[test]
fn tocka_je_krug_a_ne_kvadrat() {
    let mut u = Uredjivac::novi();
    let t = sloj(&mut u, VrstaGeometrije::Tocka);
    dodaj(&mut u, t, Geometrija::Tocka(oko(0.0, 0.0)));
    let s = nacrtaj(&u, ZUM);
    let (x, y) = piksel(SREDINA[0], SREDINA[1]);
    // Promjer je 3 mm, oko 11,3 točke, pa je ugao opisanog kvadrata izvan kruga.
    assert!(s.piksel(x, y)[3] > 200);
    assert!(s.piksel(x + 4, y)[3] > 200);
    assert_eq!(s.piksel(x + 5, y + 5)[3], 0, "ugao kvadrata");
}

#[test]
fn prvi_sloj_u_stablu_je_na_vrhu() {
    let mut u = Uredjivac::novi();
    let gornji = sloj(&mut u, VrstaGeometrije::Poligon);
    let donji = sloj(&mut u, VrstaGeometrije::Poligon);
    for (s, b) in [(gornji, CRVENO), (donji, PLAVO)] {
        let stil = Stil {
            boja_ispune: b,
            ..Stil::default()
        };
        u.izvrsi(NaredbaStabla::PostaviStil { sloj: s, stil })
            .expect("stil");
        dodaj(
            &mut u,
            s,
            Geometrija::Poligon(vec![pravokutnik(-0.01, -0.005, 0.02, 0.01)]),
        );
    }
    let s = nacrtaj(&u, ZUM);
    assert_eq!(na(&s, 0.0, 0.0), [255, 0, 0, 255]);
}

#[test]
fn mnogo_komada_na_ekranu_prosiruje_spremnik_pogleda() {
    let mut u = Uredjivac::novi();
    let t = sloj(&mut u, VrstaGeometrije::Tocka);
    for i in 0..200 {
        let x = -1.5 + f64::from(i % 20) * 0.15;
        let y = -0.9 + f64::from(i / 20) * 0.18;
        dodaj(&mut u, t, Geometrija::Tocka(oko(x, y)));
    }
    let s = nacrtaj(&u, 9.0);
    let vidljivih = s.bajtovi.chunks(4).filter(|p| p[3] > 0).count();
    assert!(vidljivih > 1000, "nacrtano je {vidljivih} piksela");
}

#[test]
fn prazan_projekt_i_objekt_izvan_ekrana_ne_crtaju_nista() {
    let mut u = Uredjivac::novi();
    let s = nacrtaj(&u, ZUM);
    assert!(s.bajtovi.iter().all(|b| *b == 0));
    let t = sloj(&mut u, VrstaGeometrije::Tocka);
    dodaj(&mut u, t, Geometrija::Tocka(oko(1.0, 1.0)));
    let s = nacrtaj(&u, ZUM);
    assert!(s.bajtovi.iter().all(|b| *b == 0));
}
