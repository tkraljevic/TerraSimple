//! Provjere koje ne ovise o biblioteci kojom program računa: objavljeni rezultati i vlastiti račun.

use geometrija::{Tocka, mjeri_crtu, mjeri_lik};

const A: f64 = 6_378_137.0;
const F: f64 = 1.0 / 298.257_223_563;

/// Vincentyjeve formule (1975.). Sporije i ne rade za gotovo suprotne točke, ali su drugi račun.
fn vincenty_m(p: Tocka, q: Tocka) -> f64 {
    let b = A * (1.0 - F);
    let u1 = ((1.0 - F) * p[1].to_radians().tan()).atan();
    let u2 = ((1.0 - F) * q[1].to_radians().tan()).atan();
    let l = (q[0] - p[0]).to_radians();
    let (su1, cu1, su2, cu2) = (u1.sin(), u1.cos(), u2.sin(), u2.cos());
    let mut lam = l;
    let (mut ss, mut cs, mut sig, mut c2a, mut c2m) = (0.0, 0.0, 0.0, 0.0, 0.0);
    for _ in 0..200 {
        let (sl, cl) = lam.sin_cos();
        ss = f64::hypot(cu2 * sl, cu1 * su2 - su1 * cu2 * cl);
        cs = su1 * su2 + cu1 * cu2 * cl;
        sig = f64::atan2(ss, cs);
        let sa = cu1 * cu2 * sl / ss;
        c2a = 1.0 - sa * sa;
        c2m = cs - 2.0 * su1 * su2 / c2a;
        let c = F / 16.0 * c2a * (4.0 + F * (4.0 - 3.0 * c2a));
        let novi =
            l + (1.0 - c) * F * sa * (sig + c * ss * (c2m + c * cs * (-1.0 + 2.0 * c2m * c2m)));
        let gotovo = (novi - lam).abs() < 1e-13;
        lam = novi;
        if gotovo {
            break;
        }
    }
    let uu = c2a * (A * A - b * b) / (b * b);
    let aa = 1.0 + uu / 16384.0 * (4096.0 + uu * (-768.0 + uu * (320.0 - 175.0 * uu)));
    let bb = uu / 1024.0 * (256.0 + uu * (-128.0 + uu * (74.0 - 47.0 * uu)));
    let ds = bb
        * ss
        * (c2m
            + bb / 4.0
                * (cs * (-1.0 + 2.0 * c2m * c2m)
                    - bb / 6.0 * c2m * (-3.0 + 4.0 * ss * ss) * (-3.0 + 4.0 * c2m * c2m)));
    b * aa * (sig - ds)
}

/// Pojas između dvije paralele i dva meridijana ima točnu površinu u zatvorenom obliku.
fn povrsina_pojasa_m2(duzina_stupnjeva: f64, sirina1: f64, sirina2: f64) -> f64 {
    let e2 = F * (2.0 - F);
    let e = e2.sqrt();
    let q = |sirina: f64| {
        let s = sirina.to_radians().sin();
        s / (1.0 - e2 * s * s) + ((1.0 + e * s) / (1.0 - e * s)).ln() / (2.0 * e)
    };
    let b = A * (1.0 - F);
    duzina_stupnjeva.to_radians() * b * b / 2.0 * (q(sirina2) - q(sirina1))
}

/// Isti pojas kao lik, s toliko točaka na paralelama da se rub drži paralele.
fn pojas_kao_lik(x1: f64, x2: f64, y1: f64, y2: f64, n: usize) -> Vec<Tocka> {
    let korak = |i: usize| x1 + (x2 - x1) * i as f64 / n as f64;
    let dolje = (0..=n).map(|i| [korak(i), y1]);
    let gore = (0..=n).rev().map(|i| [korak(i), y2]);
    dolje.chain(gore).collect()
}

#[test]
fn objavljeni_primjer_duljine() {
    // GeographicLib, primjer JFK–LHR: 5551759,400 m.
    let m = mjeri_crtu(&[[-73.8, 40.6], [-0.5, 51.6]]).expect("mjere");
    assert!((m.duljina_m - 5_551_759.400).abs() < 0.001, "{m:?}");
}

#[test]
fn objavljeni_primjer_povrsine() {
    // GeographicLib, primjer Antarktika: 16831067,893 m i 13662703680020,1 m².
    let t: [Tocka; 18] = [
        [-58.0, -63.1],
        [-74.0, -72.9],
        [-102.0, -71.9],
        [-102.0, -74.9],
        [-131.0, -74.3],
        [-163.0, -77.5],
        [163.0, -77.4],
        [172.0, -71.7],
        [140.0, -65.9],
        [113.0, -65.7],
        [88.0, -66.6],
        [59.0, -66.9],
        [25.0, -69.8],
        [-4.0, -70.0],
        [-14.0, -71.0],
        [-33.0, -77.3],
        [-46.0, -77.9],
        [-61.0, -74.7],
    ];
    let m = mjeri_lik(&[t]).expect("mjere");
    assert!((m.opseg_m - 16_831_067.893).abs() < 0.001, "{m:?}");
    assert!((m.povrsina_m2 - 13_662_703_680_020.1).abs() < 1.0, "{m:?}");
}

#[test]
fn duljine_se_slazu_s_vincentyjem() {
    let parovi: [(Tocka, Tocka); 6] = [
        ([18.69, 45.55], [15.98, 45.81]),
        ([15.98, 45.81], [16.44, 43.51]),
        ([18.69, 45.55], [18.6901, 45.5501]),
        ([13.85, 44.87], [19.38, 45.22]),
        ([0.0, 0.0], [90.0, 45.0]),
        ([-73.8, 40.6], [-0.5, 51.6]),
    ];
    for (p, q) in parovi {
        let m = mjeri_crtu(&[p, q]).expect("mjere");
        let d = (m.duljina_m - vincenty_m(p, q)).abs();
        assert!(d < 1e-3, "{p:?} {q:?}: {d} m");
    }
}

#[test]
fn povrsina_pojasa_slaze_se_s_tocnom_formulom() {
    for (x1, x2, y1, y2) in [
        (18.0, 19.0, 45.0, 46.0),
        (13.5, 19.5, 42.0, 46.5),
        (18.0, 18.01, 45.0, 45.01),
        (-10.0, 10.0, -5.0, 5.0),
        (10.0, 30.0, 70.0, 80.0),
    ] {
        let tocno = povrsina_pojasa_m2(x2 - x1, y1, y2);
        let m = mjeri_lik(&[pojas_kao_lik(x1, x2, y1, y2, 4000)]).expect("mjere");
        let udio = (m.povrsina_m2 - tocno).abs() / tocno;
        assert!(udio < 1e-7, "{x1} {x2} {y1} {y2}: udio {udio}, {tocno} m²");
    }
}

#[test]
fn opseg_je_zbroj_duljina_stranica() {
    let t: [Tocka; 4] = [[18.0, 45.0], [18.2, 45.1], [18.3, 45.3], [18.05, 45.2]];
    let zatvoreno = [t[0], t[1], t[2], t[3], t[0]];
    let crta = mjeri_crtu(&zatvoreno).expect("mjere");
    let lik = mjeri_lik(&[t]).expect("mjere");
    assert!((crta.duljina_m - lik.opseg_m).abs() < 1e-6);
}
