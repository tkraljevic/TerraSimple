//! Kontrolne mjere iz QGIS-a 3.44.15 i PROJ-a 9.8. Tablicu izrađuje `provjera/pokreni.sh`.
//!
//! Ta dva računa su ista biblioteka, pa neovisnu provjeru daje `neovisne.rs`.

use geometrija::{Metoda, Tocka, mjeri_crtu, mjeri_lik};

const TABLICA: &str = include_str!("podaci/kontrolne.csv");
const GRANICA_DULJINE_M: f64 = 1e-5;
const GRANICA_POVRSINE_M2: f64 = 1e-2;
const NAJMANJE_REDAKA: usize = 150;

struct Redak<'a> {
    ime: &'a str,
    vrsta: &'a str,
    prstenovi: Vec<Vec<Tocka>>,
    /// Duljina ili opseg te površina: prvo PROJ, zatim QGIS.
    mjere: [f64; 4],
}

fn tocke(zapis: &str) -> Vec<Tocka> {
    zapis
        .split_whitespace()
        .map(|t| {
            let (x, y) = t.split_once(':').expect("točka");
            [x.parse().expect("dužina"), y.parse().expect("širina")]
        })
        .collect()
}

fn redci() -> Vec<Redak<'static>> {
    TABLICA
        .lines()
        .skip(1)
        .map(|r| {
            let s: Vec<&str> = r.split(',').collect();
            let broj = |i: usize| s[i].parse::<f64>().expect("broj");
            Redak {
                ime: s[0],
                vrsta: s[1],
                prstenovi: s[2].split('|').map(tocke).collect(),
                mjere: [broj(3), broj(4), broj(5), broj(6)],
            }
        })
        .collect()
}

#[test]
fn tablica_nije_odrezana() {
    let svi = redci();
    assert!(svi.len() >= NAJMANJE_REDAKA, "{}", svi.len());
    for ime in [
        "jfk-lhr",
        "antarktika",
        "sa-dvije-rupe",
        "kvadrat-1km-obrnuto",
    ] {
        assert!(svi.iter().any(|r| r.ime == ime), "{ime}");
    }
    assert!(svi.iter().filter(|r| r.mjere[0] > 0.0).count() > 140);
}

#[test]
fn crte_su_unutar_granice_prema_proju_i_qgisu() {
    let mut mjereno = 0;
    for r in redci().iter().filter(|r| r.vrsta == "crta") {
        let m = mjeri_crtu(&r.prstenovi[0]).expect(r.ime);
        assert_eq!(m.metoda, Metoda::ElipsoidWgs84);
        for ocekivano in [r.mjere[0], r.mjere[2]] {
            let d = (m.duljina_m - ocekivano).abs();
            assert!(d <= GRANICA_DULJINE_M, "{}: {d} m", r.ime);
        }
        mjereno += 1;
    }
    assert_eq!(mjereno, 11);
}

#[test]
fn likovi_su_unutar_granice_prema_proju_i_qgisu() {
    let mut mjereno = 0;
    for r in redci().iter().filter(|r| r.vrsta == "lik") {
        let m = mjeri_lik(&r.prstenovi).expect(r.ime);
        assert_eq!(m.metoda, Metoda::ElipsoidWgs84);
        for (opseg, povrsina) in [(r.mjere[0], r.mjere[1]), (r.mjere[2], r.mjere[3])] {
            let d = (m.opseg_m - opseg).abs();
            assert!(d <= GRANICA_DULJINE_M, "{} opseg: {d} m", r.ime);
            // Veliki likovi imaju brojeve od 1e13 m², gdje ni računalo ne razlikuje centimetre.
            let dopusteno = GRANICA_POVRSINE_M2.max(povrsina.abs() * 1e-12);
            let d = (m.povrsina_m2 - povrsina).abs();
            assert!(d <= dopusteno, "{} površina: {d} m²", r.ime);
        }
        mjereno += 1;
    }
    assert!(mjereno >= 139, "{mjereno}");
}
