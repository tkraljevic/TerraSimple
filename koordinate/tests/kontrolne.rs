//! Kontrolne točke: deset gradova, u svakom smjeru, kako ih je izračunao QGIS 3.44.15 (PROJ 9.8).
//!
//! Zapisana granica je 1 mm. Tablica je iz `probe/koordinate`, a gradovi su javne koordinate.

use koordinate::{Pretvornik, Sustav};

const TABLICA: &str = include_str!("podaci/kontrolne.csv");
const GRANICA_M: f64 = 0.001;
const METAR_PO_STUPNJU: f64 = 111_320.0;

fn sustav(broj: &str) -> Sustav {
    Sustav::iz_epsg(broj.parse().expect("broj sustava")).expect("sustav s popisa")
}

fn razlika_m(u: &str, a: [f64; 2], b: [f64; 2]) -> f64 {
    if u == "4326" {
        ((a[0] - b[0]) * METAR_PO_STUPNJU * b[1].to_radians().cos())
            .hypot((a[1] - b[1]) * METAR_PO_STUPNJU)
    } else {
        (a[0] - b[0]).hypot(a[1] - b[1])
    }
}

#[test]
fn svi_smjerovi_su_unutar_milimetra_od_qgisa() {
    let mut redaka = 0;
    let mut smjerovi = std::collections::BTreeSet::new();
    for redak in TABLICA.lines().skip(1) {
        let p: Vec<&str> = redak.split(',').collect();
        let (iz, u) = p[0].split_once('>').expect("smjer");
        let br = |i: usize| p[i].parse::<f64>().expect("broj");
        let pretvornik = Pretvornik::novi(sustav(iz), sustav(u)).expect("pretvornik");
        let nas = pretvornik.pretvori([br(1), br(2)]).expect("pretvorba");
        let d = razlika_m(u, nas, [br(3), br(4)]);
        assert!(d <= GRANICA_M, "{redak}: {d} m");
        redaka += 1;
        smjerovi.insert(p[0].to_string());
    }
    // Da prazna ili odrezana tablica ne prođe kao uspjeh.
    assert_eq!(smjerovi.len(), 17, "{smjerovi:?}");
    assert_eq!(redaka, 170);
}

/// Primjer iz EPSG Guidance Note 7-2. Ne ovisi ni o QGIS-u ni o našem popisu sustava.
#[test]
fn epsg_primjer_mercatora() {
    let p = Pretvornik::novi(Sustav::Wgs84, Sustav::WebMercator).expect("pretvornik");
    let lat = 24.0 + 22.0 / 60.0 + 54.433 / 3600.0;
    let lon = -(100.0 + 20.0 / 60.0);
    let t = p.pretvori([lon, lat]).expect("pretvorba");
    assert!((t[0] + 11_169_055.58).abs() < 0.01, "{}", t[0]);
    assert!((t[1] - 2_800_000.00).abs() < 0.01, "{}", t[1]);
}

/// Krug kroz HDKS i u QGIS-u vrati točku pored, do oko 4 mm, pa se neizmijenjeno ne preračunava.
#[test]
fn krug_kroz_sustave() {
    let osijek = [18.69, 45.55];
    for (s, dopusteno) in [
        (Sustav::Htrs96Tm, 1e-6),
        (Sustav::WebMercator, 1e-6),
        (Sustav::HdksZona5, 0.006),
        (Sustav::HdksZona6, 0.006),
    ] {
        let tamo = Pretvornik::novi(Sustav::Wgs84, s).expect("tamo");
        let natrag = Pretvornik::novi(s, Sustav::Wgs84).expect("natrag");
        let t = natrag
            .pretvori(tamo.pretvori(osijek).expect("tamo"))
            .expect("natrag");
        let d = razlika_m("4326", t, osijek);
        assert!(d <= dopusteno, "{s:?}: {d} m");
    }
}

/// Stari i novi broj HDKS-a daju isto, jer je to isti sustav s drugim službenim redom osi.
#[test]
fn novi_i_stari_broj_daju_isti_rezultat() {
    for (stari, novi) in [(3907, 8677), (3908, 8678)] {
        let a = Pretvornik::novi(Sustav::Wgs84, Sustav::iz_epsg(stari).expect("stari")).expect("a");
        let b = Pretvornik::novi(Sustav::Wgs84, Sustav::iz_epsg(novi).expect("novi")).expect("b");
        assert_eq!(a.pretvori([18.69, 45.55]), b.pretvori([18.69, 45.55]));
    }
}
