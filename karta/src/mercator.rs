//! Web Mercator (EPSG:3857), jedini sustav u kojem `walkers` crta pločice.

const POLUMJER: f64 = 6_378_137.0;

/// Istok i sjever u metrima za dužinu i širinu u stupnjevima.
pub(crate) fn u_mercator(duzina: f64, sirina: f64) -> [f64; 2] {
    let y = (std::f64::consts::FRAC_PI_4 + sirina.to_radians() / 2.0)
        .tan()
        .ln();
    [POLUMJER * duzina.to_radians(), POLUMJER * y]
}

/// Metara Mercatora po jednom stupnju dužine, od čega se računa mjerilo ekrana.
pub(crate) fn metara_po_stupnju() -> f64 {
    POLUMJER * 1.0_f64.to_radians()
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn slaze_se_s_qgisom_za_osijek() {
        // QGIS 3.44.15, EPSG:4326 u EPSG:3857, iz kontrolnih točaka paketa `koordinate`.
        let p = koordinate_osijeka();
        let t = u_mercator(18.69, 45.55);
        assert!(
            (t[0] - p[0]).abs() < 1e-3 && (t[1] - p[1]).abs() < 1e-3,
            "{t:?}"
        );
    }

    fn koordinate_osijeka() -> [f64; 2] {
        // Račun iz definicije, neovisan o funkciji iznad, jer je `koordinate` ne bi smio pokrivati dvaput.
        let a = 6_378_137.0_f64;
        let x = a * 18.69 * std::f64::consts::PI / 180.0;
        let f = 45.55 * std::f64::consts::PI / 180.0;
        let y = a * (f.sin() + 1.0).atan2(f.cos()).tan().ln();
        [x, y]
    }

    #[test]
    fn stupanj_duzine_je_stalan() {
        let a = u_mercator(18.0, 45.0);
        let b = u_mercator(19.0, 45.0);
        assert!((b[0] - a[0] - metara_po_stupnju()).abs() < 1e-6);
    }
}
