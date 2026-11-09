//! Integracijski testovi za čitanje koordinata iz teksta.

use koordinate::{IshodPrepoznavanja, Pretvornik, RazlogRaspona, Sustav, prepoznaj_koordinate};

const KONTROLNE_TOCKE: &str = include_str!("podaci/kontrolne.csv");

/// Jednostavni LCG generator nasumičnih brojeva sa stalnim sjemenom, bez vanjskih paketa.
struct Lcg(u64);

impl Lcg {
    fn novi(seed: u64) -> Self {
        Self(seed)
    }

    fn sljedeci(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn f64_u_rasponu(&mut self, min: f64, max: f64) -> f64 {
        let val = (self.sljedeci() >> 11) as f64 / (1u64 << 53) as f64;
        min + val * (max - min)
    }
}

#[test]
fn htrs_provjera_s_qgisom_za_deset_gradova() {
    for redak in KONTROLNE_TOCKE.lines().skip(1) {
        let p: Vec<&str> = redak.split(',').collect();
        if p[0] != "4326>3765" {
            continue;
        }
        let wgs_lon: f64 = p[1].parse().expect("lon");
        let wgs_lat: f64 = p[2].parse().expect("lat");
        let htrs_e: f64 = p[3].parse().expect("easting");
        let htrs_n: f64 = p[4].parse().expect("northing");

        // Formatiramo kao korisnički unos u metrima: istok sjever
        let unos = format!("{htrs_e:.2} {htrs_n:.2}");
        let ishod = prepoznaj_koordinate(&unos);
        let tocke = match ishod {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("Nije prepoznat HTRS za {unos}: {other:?}"),
        };
        assert_eq!(tocke.len(), 1);
        assert_eq!(tocke[0].sustav, Sustav::Htrs96Tm);
        assert!(!tocke[0].zamijenjen_redoslijed);
        let razlika_lat = (tocke[0].sirina - wgs_lat).abs();
        let razlika_lon = (tocke[0].duzina - wgs_lon).abs();
        assert!(razlika_lat < 1e-4, "lat razlika {razlika_lat} za {unos}");
        assert!(razlika_lon < 1e-4, "lon razlika {razlika_lon} za {unos}");
    }
}

fn formatiraj_prikaz_hr(lat: f64, lon: f64) -> String {
    let s_slovo = if lat >= 0.0 { 'S' } else { 'J' };
    let d_slovo = if lon >= 0.0 { 'I' } else { 'Z' };
    let s_tekst = format!("{:.4}", lat.abs()).replace('.', ",");
    let d_tekst = format!("{:.4}", lon.abs()).replace('.', ",");
    format!("{s_tekst}° {s_slovo} {d_tekst}° {d_slovo}")
}

#[test]
fn krug_tocka_u_tekst_pa_natrag() {
    let tocke = [
        (45.81, 15.98),   // Zagreb
        (45.55, 18.69),   // Osijek
        (45.33, 14.44),   // Rijeka
        (43.51, 16.44),   // Split
        (42.65, 18.09),   // Dubrovnik
        (0.0, 0.0),       // Ishodište
        (-33.86, 151.20), // Sydney (južna polutka)
        (51.50, -0.12),   // London (zapadna dužina)
        (-41.28, 174.77), // Wellington (južna i istočna)
        (-34.60, -58.38), // Buenos Aires (južna i zapadna)
    ];

    for (lat, lon) in tocke {
        // 1. Hrvatski format prikaza programa: S/J i I/Z
        let unos_prikaz = formatiraj_prikaz_hr(lat, lon);
        let ishod_p = prepoznaj_koordinate(&unos_prikaz);
        let t_p = match ishod_p {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("Greška za prikaz '{unos_prikaz}': {other:?}"),
        };
        assert_eq!(t_p.len(), 1);
        assert!(
            (t_p[0].sirina - lat).abs() < 1e-3,
            "sirina za '{unos_prikaz}': dobiveno {}, ocekivano {}",
            t_p[0].sirina,
            lat
        );
        assert!(
            (t_p[0].duzina - lon).abs() < 1e-3,
            "duzina za '{unos_prikaz}': dobiveno {}, ocekivano {}",
            t_p[0].duzina,
            lon
        );

        // 2. Decimalni stupnjevi s točkom
        let unos_tocka = format!("{lat:.4}, {lon:.4}");
        let ishod = prepoznaj_koordinate(&unos_tocka);
        let t = match ishod {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("Greška za {unos_tocka}: {other:?}"),
        };
        assert!((t[0].sirina - lat).abs() < 1e-3);
        assert!((t[0].duzina - lon).abs() < 1e-3);

        // 3. Decimalni stupnjevi sa zarezom
        let s_zarez = format!("{lat:.4}").replace('.', ",");
        let d_zarez = format!("{lon:.4}").replace('.', ",");
        let unos_zarez = format!("{s_zarez}; {d_zarez}");
        let ishod_z = prepoznaj_koordinate(&unos_zarez);
        let t_z = match ishod_z {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("Greška za {unos_zarez}: {other:?}"),
        };
        assert!((t_z[0].sirina - lat).abs() < 1e-3);
        assert!((t_z[0].duzina - lon).abs() < 1e-3);
    }
}

#[test]
fn rubni_slucajevi_i_sekunde_59_999() {
    let ishod = prepoznaj_koordinate("45° 33' 59.999\" N 18° 41' 59.999\" E");
    let tocke = match ishod {
        IshodPrepoznavanja::Prepoznato(t) => t,
        other => panic!("Greška za 59.999: {other:?}"),
    };
    assert_eq!(tocke.len(), 1);
    assert!((tocke[0].sirina - (45.0 + 33.0 / 60.0 + 59.999 / 3600.0)).abs() < 1e-5);
    assert!((tocke[0].duzina - (18.0 + 41.0 / 60.0 + 59.999 / 3600.0)).abs() < 1e-5);

    // Točno 60 sekundi se odbija
    let ishod_60 = prepoznaj_koordinate("45° 33' 60\" N 18° 41' 00\" E");
    assert_eq!(
        ishod_60,
        IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Sekunde)
    );

    // Točno 60 minuta se odbija
    let ishod_60m = prepoznaj_koordinate("45° 60' 00\" N 18° 41' 00\" E");
    assert_eq!(
        ishod_60m,
        IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Minute)
    );

    // Širina 91 se odbija
    let ishod_91 = prepoznaj_koordinate("91° 00' 00\" N 18° 00' 00\" E");
    assert_eq!(
        ishod_91,
        IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Sirina)
    );

    // Dužina 181 se odbija
    let ishod_181 = prepoznaj_koordinate("45° 00' 00\" N 181° 00' 00\" E");
    assert_eq!(
        ishod_181,
        IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Duzina)
    );
}

#[test]
fn nasumicne_tocke_svih_pet_zapisa_krug() {
    let mut lcg = Lcg::novi(202_611_091_200);

    // 1. Svjetske točke za 4 formata stupnjeva
    for _ in 0..50 {
        let lat = lcg.f64_u_rasponu(-85.0, 85.0);
        let lon = lcg.f64_u_rasponu(-175.0, 175.0);

        // a) Decimalni s točkom
        let unos_tocka = format!("{lat:.4}, {lon:.4}");
        let tocke_t = match prepoznaj_koordinate(&unos_tocka) {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("greška za '{unos_tocka}': {other:?}"),
        };
        assert!((tocke_t[0].sirina - lat).abs() < 1e-3);
        assert!((tocke_t[0].duzina - lon).abs() < 1e-3);

        // b) Decimalni sa zarezom
        let s_z = format!("{lat:.4}").replace('.', ",");
        let d_z = format!("{lon:.4}").replace('.', ",");
        let unos_z = format!("{s_z}; {d_z}");
        let tocke_z = match prepoznaj_koordinate(&unos_z) {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("greška za '{unos_z}': {other:?}"),
        };
        assert!((tocke_z[0].sirina - lat).abs() < 1e-3);
        assert!((tocke_z[0].duzina - lon).abs() < 1e-3);

        // c) Stupnjevi, minute i sekunde s hrvatskim oznakama (S/J, I/Z)
        let s_slovo = if lat >= 0.0 { 'S' } else { 'J' };
        let d_slovo = if lon >= 0.0 { 'I' } else { 'Z' };
        let lat_a = lat.abs();
        let lon_a = lon.abs();
        let deg_s = lat_a.floor();
        let rem_s = (lat_a - deg_s) * 60.0;
        let min_s = rem_s.floor();
        let sec_s = (rem_s - min_s) * 60.0;

        let deg_d = lon_a.floor();
        let rem_d = (lon_a - deg_d) * 60.0;
        let min_d = rem_d.floor();
        let sec_d = (rem_d - min_d) * 60.0;

        let unos_dms = format!(
            "{:.0}° {:.0}' {:.2}\" {} {:.0}° {:.0}' {:.2}\" {}",
            deg_s, min_s, sec_s, s_slovo, deg_d, min_d, sec_d, d_slovo
        );
        let tocke_dms = match prepoznaj_koordinate(&unos_dms) {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("greška za '{unos_dms}': {other:?}"),
        };
        assert!((tocke_dms[0].sirina - lat).abs() < 1e-3);
        assert!((tocke_dms[0].duzina - lon).abs() < 1e-3);

        // d) Stupnjevi i decimalne minute s hrvatskim oznakama
        let unos_dm = format!(
            "{:.0}° {:.4}' {} {:.0}° {:.4}' {}",
            deg_s, rem_s, s_slovo, deg_d, rem_d, d_slovo
        );
        let tocke_dm = match prepoznaj_koordinate(&unos_dm) {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("greška za '{unos_dm}': {other:?}"),
        };
        assert!((tocke_dm[0].sirina - lat).abs() < 1e-3);
        assert!((tocke_dm[0].duzina - lon).abs() < 1e-3);
    }

    // 2. Točke u Hrvatskoj za HTRS96/TM i sve ostale formate
    let pretvornik =
        Pretvornik::novi(Sustav::Wgs84, Sustav::Htrs96Tm).expect("pretvornik WGS u HTRS");
    for _ in 0..50 {
        let lat = lcg.f64_u_rasponu(42.5, 46.5);
        let lon = lcg.f64_u_rasponu(13.5, 19.0);

        // e) HTRS96/TM unos
        let htrs = pretvornik.pretvori([lon, lat]).expect("pretvorba");
        let unos_htrs = format!("{:.2} {:.2}", htrs[0], htrs[1]);
        let tocke_h = match prepoznaj_koordinate(&unos_htrs) {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("greška za HTRS '{unos_htrs}': {other:?}"),
        };
        assert_eq!(tocke_h.len(), 1);
        assert_eq!(tocke_h[0].sustav, Sustav::Htrs96Tm);
        assert!((tocke_h[0].sirina - lat).abs() < 1e-4);
        assert!((tocke_h[0].duzina - lon).abs() < 1e-4);
    }
}
