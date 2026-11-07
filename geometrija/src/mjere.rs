//! Duljina crte te opseg i površina lika na elipsoidu.
//!
//! Točke su u WGS84, u stupnjevima, redom dužina i širina. Visina se ne računa, kao ni u QGIS-u.
//! Rezultat je isti kao QGIS-ov uz elipsoid WGS84 (7. 11. 2026., QGIS 3.44.15).

use geographiclib_rs::{Geodesic, InverseGeodesic, PolygonArea, Winding};

/// Dužina i širina u stupnjevima.
pub type Tocka = [f64; 2];

/// Veće dužine ne postoje ni u jednom zapisu, pa je takav broj gotovo sigurno metar, a ne stupanj.
const NAJVECA_DUZINA: f64 = 360.0;
const NAJVECA_SIRINA: f64 = 90.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Greska {
    /// Koordinata nije konačan broj.
    NijeBroj,
    /// Koordinata nije u stupnjevima, npr. sloj u metrima predan kao WGS84.
    IzvanRaspona,
}

impl std::fmt::Display for Greska {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::NijeBroj => "koordinata nije konačan broj",
            Self::IzvanRaspona => "koordinata nije u stupnjevima",
        })
    }
}

impl std::error::Error for Greska {}

/// Kako je mjera dobivena, jer uz svaku duljinu i površinu mora pisati metoda.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metoda {
    /// Najkraći put po elipsoidu WGS84.
    ElipsoidWgs84,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MjereCrte {
    pub duljina_m: f64,
    pub metoda: Metoda,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MjereLika {
    /// Vanjski rub i rubovi svih rupa, kao u QGIS-u.
    pub opseg_m: f64,
    /// Površina vanjskog ruba bez površine rupa.
    pub povrsina_m2: f64,
    pub metoda: Metoda,
}

/// Je li točka valjana dužina i širina u stupnjevima.
pub fn provjeri_tocku(t: Tocka) -> Result<(), Greska> {
    if !(t[0].is_finite() && t[1].is_finite()) {
        return Err(Greska::NijeBroj);
    }
    if t[0].abs() > NAJVECA_DUZINA || t[1].abs() > NAJVECA_SIRINA {
        return Err(Greska::IzvanRaspona);
    }
    Ok(())
}

fn provjeri(tocke: &[Tocka]) -> Result<(), Greska> {
    tocke.iter().try_for_each(|t| provjeri_tocku(*t))
}

/// Duljina crte. Crta bez točaka ili s jednom točkom ima duljinu 0, jer tako počinje svako crtanje.
pub fn mjeri_crtu(tocke: &[Tocka]) -> Result<MjereCrte, Greska> {
    provjeri(tocke)?;
    let g = Geodesic::wgs84();
    let duljina_m = tocke
        .windows(2)
        .map(|d| -> f64 { g.inverse(d[0][1], d[0][0], d[1][1], d[1][0]) })
        .sum();
    Ok(MjereCrte {
        duljina_m,
        metoda: Metoda::ElipsoidWgs84,
    })
}

/// GeoPackage i KML prsten zatvaraju ponovljenom prvom točkom, a crtanje ne.
fn bez_zatvaranja(tocke: &[Tocka]) -> &[Tocka] {
    match tocke.split_last() {
        Some((zadnja, ostale)) if tocke.first() == Some(zadnja) => ostale,
        _ => tocke,
    }
}

/// Opseg i površina jednog prstena. Manje od tri različite točke još nije lik, pa su mjere 0.
fn mjeri_prsten(g: &Geodesic, tocke: &[Tocka]) -> Result<(f64, f64), Greska> {
    provjeri(tocke)?;
    let tocke = bez_zatvaranja(tocke);
    if tocke.len() < 3 {
        return Ok((0.0, 0.0));
    }
    let mut lik = PolygonArea::new(g, Winding::CounterClockwise);
    for t in tocke {
        lik.add_point(t[1], t[0]);
    }
    // Bez predznaka bi lik nacrtan u smjeru kazaljke dao površinu ostatka Zemlje (proba crtanja, 6. 11. 2026.).
    let (opseg, povrsina, _) = lik.compute(true);
    Ok((opseg, povrsina.abs()))
}

/// Mjere lika. Prvi prsten je vanjski rub, a ostali su rupe. Smjer crtanja ne mijenja rezultat.
///
/// Lik koji siječe sam sebe dobiva razliku svojih dijelova, kao u QGIS-u, pa osmica može imati
/// površinu 0. Površina rupe se oduzima i kad rupa nije unutar lika, a takav lik QGIS smatra
/// neispravnim.
pub fn mjeri_lik<P: AsRef<[Tocka]>>(prstenovi: &[P]) -> Result<MjereLika, Greska> {
    let g = Geodesic::wgs84();
    let (mut opseg_m, mut povrsina_m2) = (0.0, 0.0);
    for (i, prsten) in prstenovi.iter().enumerate() {
        let (opseg, povrsina) = mjeri_prsten(&g, prsten.as_ref())?;
        opseg_m += opseg;
        povrsina_m2 += if i == 0 { povrsina } else { -povrsina };
    }
    Ok(MjereLika {
        opseg_m,
        povrsina_m2,
        metoda: Metoda::ElipsoidWgs84,
    })
}

#[cfg(test)]
mod testovi {
    use super::*;

    const KVADRAT: [Tocka; 4] = [[18.0, 45.0], [18.01, 45.0], [18.01, 45.01], [18.0, 45.01]];
    const MALI: [Tocka; 4] = [
        [18.004, 45.004],
        [18.006, 45.004],
        [18.006, 45.006],
        [18.004, 45.006],
    ];

    fn lik(t: &[Tocka]) -> MjereLika {
        mjeri_lik(&[t]).expect("mjere")
    }

    #[test]
    fn crta_bez_duljine_nije_greska() {
        let prazna: [Tocka; 0] = [];
        assert_eq!(mjeri_crtu(&prazna).expect("mjere").duljina_m, 0.0);
        assert_eq!(mjeri_crtu(&[[18.0, 45.0]]).expect("mjere").duljina_m, 0.0);
    }

    #[test]
    fn smjer_i_prva_tocka_ne_mijenjaju_mjere() {
        let a = lik(&KVADRAT);
        assert!(a.povrsina_m2 > 800_000.0, "{a:?}");
        let mut obrnuto = KVADRAT;
        obrnuto.reverse();
        let b = lik(&obrnuto);
        assert!((a.povrsina_m2 - b.povrsina_m2).abs() < 1e-3);
        assert!((a.opseg_m - b.opseg_m).abs() < 1e-6);
        let mut pomaknuto = KVADRAT;
        pomaknuto.rotate_left(2);
        let c = lik(&pomaknuto);
        assert!((a.povrsina_m2 - c.povrsina_m2).abs() < 1e-3);
        assert!((a.opseg_m - c.opseg_m).abs() < 1e-6);
    }

    #[test]
    fn zatvoren_prsten_ima_iste_mjere_kao_otvoren() {
        let otvoren = lik(&KVADRAT);
        let mut zatvoren = KVADRAT.to_vec();
        zatvoren.push(KVADRAT[0]);
        assert_eq!(lik(&zatvoren), otvoren);
    }

    #[test]
    fn manje_od_tri_razlicite_tocke_nije_lik() {
        let nista = MjereLika {
            opseg_m: 0.0,
            povrsina_m2: 0.0,
            metoda: Metoda::ElipsoidWgs84,
        };
        let [a, b, ..] = KVADRAT;
        assert_eq!(lik(&[]), nista);
        assert_eq!(lik(&[a]), nista);
        assert_eq!(lik(&[a, b]), nista);
        assert_eq!(lik(&[a, b, a]), nista, "zatvorena crta od dvije točke");
        let bez_prstena: [&[Tocka]; 0] = [];
        assert_eq!(mjeri_lik(&bez_prstena), Ok(nista));
    }

    #[test]
    fn rupa_se_oduzima_od_povrsine_a_dodaje_opsegu() {
        let (cijeli, rupa) = (lik(&KVADRAT), lik(&MALI));
        let s_rupom = mjeri_lik(&[&KVADRAT[..], &MALI[..]]).expect("mjere");
        let razlika = cijeli.povrsina_m2 - rupa.povrsina_m2;
        assert!((s_rupom.povrsina_m2 - razlika).abs() < 1e-6);
        assert!((s_rupom.opseg_m - cijeli.opseg_m - rupa.opseg_m).abs() < 1e-6);
    }

    #[test]
    fn vec_prima_kao_i_isjecak() {
        let prstenovi = vec![KVADRAT.to_vec(), MALI.to_vec()];
        let isjecci = [&KVADRAT[..], &MALI[..]];
        assert_eq!(mjeri_lik(&prstenovi), mjeri_lik(&isjecci));
    }

    #[test]
    fn duzina_preko_180_je_isto_mjesto() {
        let a = mjeri_crtu(&[[-170.0, 10.0], [-169.0, 10.0]]).expect("mjere");
        let b = mjeri_crtu(&[[190.0, 10.0], [191.0, 10.0]]).expect("mjere");
        assert!((a.duljina_m - b.duljina_m).abs() < 1e-6);
    }

    #[test]
    fn krive_koordinate_su_greska_a_ne_broj() {
        let losa = [
            ([f64::NAN, 45.0], Greska::NijeBroj),
            ([18.0, f64::NAN], Greska::NijeBroj),
            ([f64::INFINITY, 45.0], Greska::NijeBroj),
            ([18.0, f64::NEG_INFINITY], Greska::NijeBroj),
            ([18.0, 90.001], Greska::IzvanRaspona),
            ([18.0, -90.001], Greska::IzvanRaspona),
            ([360.001, 45.0], Greska::IzvanRaspona),
            ([-360.001, 45.0], Greska::IzvanRaspona),
            ([655_120.0, 5_046_380.0], Greska::IzvanRaspona),
        ];
        for (t, ocekivano) in losa {
            let [a, b, c, _] = KVADRAT;
            assert_eq!(mjeri_crtu(&[a, t]), Err(ocekivano), "crta {t:?}");
            assert_eq!(mjeri_lik(&[[a, b, c, t]]), Err(ocekivano), "lik {t:?}");
            let rupa = [&KVADRAT[..], &[a, b, t][..]];
            assert_eq!(mjeri_lik(&rupa), Err(ocekivano), "rupa {t:?}");
            // I u prstenu koji još nije lik, jer kriva točka ne smije proći samo zato što je sama.
            assert_eq!(mjeri_lik(&[[t]]), Err(ocekivano), "sama {t:?}");
        }
    }

    #[test]
    fn granicne_koordinate_su_valjane() {
        for t in [[360.0, 90.0], [-360.0, -90.0], [180.0, 90.0], [0.0, -90.0]] {
            assert_eq!(provjeri_tocku(t), Ok(()), "{t:?}");
        }
    }

    #[test]
    fn poruke_postoje_za_svaku_gresku() {
        assert!(Greska::NijeBroj.to_string().contains("broj"));
        assert!(Greska::IzvanRaspona.to_string().contains("stupnjevima"));
    }
}
