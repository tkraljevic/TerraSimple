//! Naš račun: Krügerov niz, Helmert i Mercator, bez ikakve biblioteke.
//!
//! Služi kao neovisna provjera biblioteke `proj4rs`, a mogao bi je i zamijeniti.
//! Niz je iz Karney (2011.), „Transverse Mercator with an accuracy of a few
//! nanometers”, do šestog reda.

use std::f64::consts::FRAC_PI_4;

#[derive(Clone, Copy)]
pub struct Elipsoid {
    pub a: f64,
    pub f: f64,
}

pub const WGS84: Elipsoid = Elipsoid { a: 6_378_137.0, f: 1.0 / 298.257_223_563 };
pub const GRS80: Elipsoid = Elipsoid { a: 6_378_137.0, f: 1.0 / 298.257_222_101 };
pub const BESSEL: Elipsoid = Elipsoid { a: 6_377_397.155, f: 1.0 / 299.152_812_8 };
pub const SFERA_MERCATOR: f64 = 6_378_137.0;

#[derive(Clone, Copy)]
pub struct Tm {
    pub elipsoid: Elipsoid,
    pub srednji: f64,
    pub k: f64,
    pub istok: f64,
    pub sjever: f64,
    pub sirina0: f64,
}

impl Tm {
    fn konstante(&self) -> (f64, f64, [f64; 6], [f64; 6]) {
        let f = self.elipsoid.f;
        let n = f / (2.0 - f);
        let e = (f * (2.0 - f)).sqrt();
        let (n2, n3, n4, n5, n6) = (n * n, n.powi(3), n.powi(4), n.powi(5), n.powi(6));
        let a = self.elipsoid.a / (1.0 + n) * (1.0 + n2 / 4.0 + n4 / 64.0 + n6 / 256.0);
        let alfa = [
            n / 2.0 - 2.0 / 3.0 * n2 + 5.0 / 16.0 * n3 + 41.0 / 180.0 * n4 - 127.0 / 288.0 * n5
                + 7891.0 / 37800.0 * n6,
            13.0 / 48.0 * n2 - 3.0 / 5.0 * n3 + 557.0 / 1440.0 * n4 + 281.0 / 630.0 * n5
                - 1983433.0 / 1935360.0 * n6,
            61.0 / 240.0 * n3 - 103.0 / 140.0 * n4 + 15061.0 / 26880.0 * n5
                + 167603.0 / 181440.0 * n6,
            49561.0 / 161280.0 * n4 - 179.0 / 168.0 * n5 + 6601661.0 / 7257600.0 * n6,
            34729.0 / 80640.0 * n5 - 3418889.0 / 1995840.0 * n6,
            212378941.0 / 319334400.0 * n6,
        ];
        let beta = [
            n / 2.0 - 2.0 / 3.0 * n2 + 37.0 / 96.0 * n3 - 1.0 / 360.0 * n4 - 81.0 / 512.0 * n5
                + 96199.0 / 604800.0 * n6,
            1.0 / 48.0 * n2 + 1.0 / 15.0 * n3 - 437.0 / 1440.0 * n4 + 46.0 / 105.0 * n5
                - 1118711.0 / 3870720.0 * n6,
            17.0 / 480.0 * n3 - 37.0 / 840.0 * n4 - 209.0 / 4480.0 * n5 + 5569.0 / 90720.0 * n6,
            4397.0 / 161280.0 * n4 - 11.0 / 504.0 * n5 - 830251.0 / 7257600.0 * n6,
            4583.0 / 161280.0 * n5 - 108847.0 / 3991680.0 * n6,
            20648693.0 / 638668800.0 * n6,
        ];
        (e, a, alfa, beta)
    }

    fn tau_crta(tau: f64, e: f64) -> f64 {
        let korijen = (1.0 + tau * tau).sqrt();
        let sigma = (e * (e * tau / korijen).atanh()).sinh();
        tau * (1.0 + sigma * sigma).sqrt() - sigma * korijen
    }

    fn u_ravninu(&self, lon: f64, lat: f64) -> (f64, f64) {
        let (e, a, alfa, _) = self.konstante();
        let lambda = (lon - self.srednji).to_radians();
        let tau_c = Self::tau_crta(lat.to_radians().tan(), e);
        let ksi_c = tau_c.atan2(lambda.cos());
        let eta_c = (lambda.sin() / (tau_c * tau_c + lambda.cos().powi(2)).sqrt()).asinh();
        let (mut ksi, mut eta) = (ksi_c, eta_c);
        for (j, c) in alfa.iter().enumerate() {
            let m = 2.0 * (j as f64 + 1.0);
            ksi += c * (m * ksi_c).sin() * (m * eta_c).cosh();
            eta += c * (m * ksi_c).cos() * (m * eta_c).sinh();
        }
        (self.k * a * eta, self.k * a * ksi)
    }

    pub fn naprijed(&self, lon: f64, lat: f64) -> (f64, f64) {
        let (x, y) = self.u_ravninu(lon, lat);
        let (_, y0) = self.u_ravninu(self.srednji, self.sirina0);
        (x + self.istok, y - y0 + self.sjever)
    }

    pub fn natrag(&self, x: f64, y: f64) -> (f64, f64) {
        let (e, a, _, beta) = self.konstante();
        let (_, y0) = self.u_ravninu(self.srednji, self.sirina0);
        let ksi = (y - self.sjever + y0) / (self.k * a);
        let eta = (x - self.istok) / (self.k * a);
        let (mut ksi_c, mut eta_c) = (ksi, eta);
        for (j, c) in beta.iter().enumerate() {
            let m = 2.0 * (j as f64 + 1.0);
            ksi_c -= c * (m * ksi).sin() * (m * eta).cosh();
            eta_c -= c * (m * ksi).cos() * (m * eta).sinh();
        }
        let lambda = eta_c.sinh().atan2(ksi_c.cos());
        let tau_c = ksi_c.sin() / (eta_c.sinh().powi(2) + ksi_c.cos().powi(2)).sqrt();
        let e2 = e * e;
        let mut tau = tau_c / (1.0 - e2);
        for _ in 0..6 {
            let t = Self::tau_crta(tau, e);
            tau += (tau_c - t) / (1.0 + t * t).sqrt() * (1.0 + (1.0 - e2) * tau * tau)
                / ((1.0 - e2) * (1.0 + tau * tau).sqrt());
        }
        (self.srednji + lambda.to_degrees(), tau.atan().to_degrees())
    }
}

pub type Helmert = [f64; 7];
pub const MGI_1: Helmert = [682.0, -203.0, 480.0, 0.0, 0.0, 0.0, 0.0];
pub const MGI_4: Helmert = [551.7, 162.9, 467.9, 6.04, 1.96, -11.38, -4.82];

fn u_kartezijeve(el: Elipsoid, lon: f64, lat: f64) -> [f64; 3] {
    let (la, fi) = (lon.to_radians(), lat.to_radians());
    let e2 = el.f * (2.0 - el.f);
    let n = el.a / (1.0 - e2 * fi.sin().powi(2)).sqrt();
    [n * fi.cos() * la.cos(), n * fi.cos() * la.sin(), n * (1.0 - e2) * fi.sin()]
}

fn iz_kartezijevih(el: Elipsoid, t: [f64; 3]) -> (f64, f64) {
    let e2 = el.f * (2.0 - el.f);
    let p = t[0].hypot(t[1]);
    let mut fi = t[2].atan2(p * (1.0 - e2));
    for _ in 0..8 {
        let n = el.a / (1.0 - e2 * fi.sin().powi(2)).sqrt();
        fi = (t[2] + e2 * n * fi.sin()).atan2(p);
    }
    (t[1].atan2(t[0]).to_degrees(), fi.to_degrees())
}

fn rotacija(h: &Helmert) -> ([f64; 3], f64, f64, f64, f64) {
    let sekunda = (1.0_f64 / 3600.0).to_radians();
    ([h[0], h[1], h[2]], h[3] * sekunda, h[4] * sekunda, h[5] * sekunda, 1.0 + h[6] * 1e-6)
}

/// Konvencija „position vector”, ista kao `+towgs84` i kao EPSG 1033.
pub fn helmert_naprijed(h: &Helmert, t: [f64; 3]) -> [f64; 3] {
    let (d, rx, ry, rz, s) = rotacija(h);
    [
        d[0] + s * (t[0] - rz * t[1] + ry * t[2]),
        d[1] + s * (rz * t[0] + t[1] - rx * t[2]),
        d[2] + s * (-ry * t[0] + rx * t[1] + t[2]),
    ]
}

pub fn helmert_natrag(h: &Helmert, t: [f64; 3]) -> [f64; 3] {
    let (d, rx, ry, rz, s) = rotacija(h);
    let (x, y, z) = ((t[0] - d[0]) / s, (t[1] - d[1]) / s, (t[2] - d[2]) / s);
    [x + rz * y - ry * z, -rz * x + y + rx * z, ry * x - rx * y + z]
}

pub fn bessel_u_wgs(h: &Helmert, lon: f64, lat: f64) -> (f64, f64) {
    iz_kartezijevih(WGS84, helmert_naprijed(h, u_kartezijeve(BESSEL, lon, lat)))
}

pub fn wgs_u_bessel(h: &Helmert, lon: f64, lat: f64) -> (f64, f64) {
    iz_kartezijevih(BESSEL, helmert_natrag(h, u_kartezijeve(WGS84, lon, lat)))
}

pub fn mercator_naprijed(lon: f64, lat: f64) -> (f64, f64) {
    let y = (FRAC_PI_4 + lat.to_radians() / 2.0).tan().ln();
    (SFERA_MERCATOR * lon.to_radians(), SFERA_MERCATOR * y)
}

pub fn mercator_natrag(x: f64, y: f64) -> (f64, f64) {
    let fi = 2.0 * (y / SFERA_MERCATOR).exp().atan() - std::f64::consts::FRAC_PI_2;
    ((x / SFERA_MERCATOR).to_degrees(), fi.to_degrees())
}

pub fn htrs() -> Tm {
    Tm { elipsoid: GRS80, srednji: 16.5, k: 0.9999, istok: 500_000.0, sjever: 0.0, sirina0: 0.0 }
}

pub fn hdks(zona: u8) -> Tm {
    Tm {
        elipsoid: BESSEL,
        srednji: f64::from(zona) * 3.0,
        k: 0.9999,
        istok: f64::from(zona) * 1_000_000.0 + 500_000.0,
        sjever: 0.0,
        sirina0: 0.0,
    }
}

#[cfg(test)]
mod testovi {
    use super::*;

    // Primjeri iz EPSG Guidance Note 7-2. Ne ovise ni o QGIS-u ni o PROJ-u.

    #[test]
    fn epsg_primjer_tm() {
        let osgb = Tm {
            elipsoid: Elipsoid { a: 6_377_563.396, f: 1.0 / 299.324_964_6 },
            srednji: -2.0,
            k: 0.999_601_271_7,
            istok: 400_000.0,
            sjever: -100_000.0,
            sirina0: 49.0,
        };
        let (x, y) = osgb.naprijed(0.5, 50.5);
        assert!((x - 577_274.99).abs() < 0.01, "{x}");
        assert!((y - 69_740.50).abs() < 0.01, "{y}");
        let (lon, lat) = osgb.natrag(x, y);
        assert!((lon - 0.5).abs() < 1e-9 && (lat - 50.5).abs() < 1e-9);
    }

    #[test]
    fn epsg_primjer_mercator() {
        let lat = 24.0 + 22.0 / 60.0 + 54.433 / 3600.0;
        let lon = -(100.0 + 20.0 / 60.0);
        let (x, y) = mercator_naprijed(lon, lat);
        assert!((x + 11_169_055.58).abs() < 0.01, "{x}");
        assert!((y - 2_800_000.00).abs() < 0.01, "{y}");
    }

    #[test]
    fn epsg_primjer_helmert() {
        let h: Helmert = [0.0, 0.0, 4.5, 0.0, 0.0, 0.554, 0.219];
        let ulaz = [3_657_660.66, 255_768.55, 5_201_382.11];
        let t = helmert_naprijed(&h, ulaz);
        assert!((t[0] - 3_657_660.78).abs() < 0.01, "{}", t[0]);
        assert!((t[1] - 255_778.43).abs() < 0.01, "{}", t[1]);
        assert!((t[2] - 5_201_387.75).abs() < 0.01, "{}", t[2]);
    }

    #[test]
    fn hdks_ima_prave_zone() {
        assert_eq!(hdks(5).srednji, 15.0);
        assert_eq!(hdks(6).srednji, 18.0);
        assert_eq!(hdks(5).istok, 5_500_000.0);
        assert_eq!(hdks(6).istok, 6_500_000.0);
    }

    #[test]
    fn krug_kroz_hdks_vraca_istu_tocku() {
        // Krug kroz HDKS nije savršen ni u QGIS-u: točka se vrati nekoliko milimetara
        // pored (izmjereno u probi: 3,9 i 7,3 mm). Zato se neizmijenjeno ne preračunava.
        for (h, dopusteno) in [(MGI_1, 1e-7), (MGI_4, 1e-7)] {
            let (b_lon, b_lat) = wgs_u_bessel(&h, 18.69, 45.55);
            let (x, y) = hdks(6).naprijed(b_lon, b_lat);
            let (b2_lon, b2_lat) = hdks(6).natrag(x, y);
            let (lon, lat) = bessel_u_wgs(&h, b2_lon, b2_lat);
            assert!(
                (lon - 18.69).abs() < dopusteno && (lat - 45.55).abs() < dopusteno,
                "{lon} {lat}"
            );
        }
    }
}
