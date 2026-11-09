//! Lik koji siječe sam sebe, prema GEOS-u iz QGIS-a 3.44.15. Tablicu izrađuje
//! `provjera/presjek.py`; svi likovi su izmišljeni.

use geometrija::{Tocka, presjek_lika};

const TABLICA: &str = include_str!("podaci/presjek_qgis.csv");

/// GEOS ovo smatra neispravnim, ali to nije presjek ruba, pa ga naša provjera ne traži.
const NIJE_PRESJEK: &[&str] = &["Hole lies outside shell"];

fn tocke(zapis: &str) -> Vec<Tocka> {
    zapis
        .split_whitespace()
        .map(|t| {
            let (x, y) = t.split_once(':').expect("točka");
            [x.parse().expect("dužina"), y.parse().expect("širina")]
        })
        .collect()
}

#[test]
fn slaze_se_s_geos_om() {
    let mut razlike = Vec::new();
    let mut broj = 0;
    for redak in TABLICA.lines().skip(1) {
        let s: Vec<&str> = redak.split(',').collect();
        let prstenovi: Vec<Vec<Tocka>> = s[3].split('|').map(tocke).collect();
        let geos_presjek = s[1] == "0" && !NIJE_PRESJEK.contains(&s[2]);
        let nas = presjek_lika(&prstenovi).is_some();
        if nas != geos_presjek {
            razlike.push(format!("{} (GEOS: {:?}, mi: {nas})", s[0], s[2]));
        }
        broj += 1;
    }
    assert!(broj >= 100, "premalo likova: {broj}");
    assert!(
        razlike.is_empty(),
        "{} razlika:\n{}",
        razlike.len(),
        razlike.join("\n")
    );
}
