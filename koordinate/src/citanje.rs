//! Čitanje koordinata iz teksta za potrebe tražilice i unosa položaja.

use crate::pretvorba::Pretvornik;
use crate::sustav::Sustav;

/// Razlog zašto je koordinata izvan dopuštenog raspona.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RazlogRaspona {
    Sirina,
    Duzina,
    Minute,
    Sekunde,
    Htrs,
    PreviseBrojeva,
}

/// Ishod prepoznavanja koordinata iz korisničkog teksta.
#[derive(Clone, Debug, PartialEq)]
pub enum IshodPrepoznavanja {
    /// Prepoznata jedna ili više mogućih točaka.
    Prepoznato(Vec<PrepoznataKoordinata>),
    /// Tekst uopće ne sliči koordinatama.
    NijeKoordinata,
    /// Tekst sliči koordinatama, ali je vrijednost izvan dopuštenog raspona.
    IzvanRaspona(RazlogRaspona),
}

/// Prepoznata točka u WGS84 s izvornim sustavom i metapodacima.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrepoznataKoordinata {
    pub duzina: f64,
    pub sirina: f64,
    pub sustav: Sustav,
    pub zamijenjen_redoslijed: bool,
    pub izvorni_htrs: Option<[f64; 2]>,
}

impl PrepoznataKoordinata {
    #[must_use]
    pub fn wgs84(&self) -> [f64; 2] {
        [self.duzina, self.sirina]
    }
}

/// Je li točka unutar pravokutnog obuhvata Hrvatske u WGS84.
#[must_use]
pub fn je_u_hrvatskoj(sirina: f64, duzina: f64) -> bool {
    (42.0..=47.0).contains(&sirina) && (13.0..=19.6).contains(&duzina)
}

/// Provjerava sadrži li tekst nedopuštena slova koja ukazuju na obične riječi.
fn ima_nedopustena_slova(tekst: &str) -> bool {
    const DOPUSTENA: &str = "nsewjizmt";
    tekst
        .chars()
        .filter(|c| c.is_alphabetic())
        .any(|c| !DOPUSTENA.contains(c.to_ascii_lowercase()))
}

fn je_htrs_znak(c: char) -> bool {
    const ZNAKOVI: &str = "°'\"′″’”NSEWJIZnsewjiz";
    ZNAKOVI.contains(c)
}

fn ima_htrs_znak(dijelovi: &[&str]) -> bool {
    dijelovi.iter().any(|d| d.chars().any(je_htrs_znak))
}

fn ocisti_i_pretvori_broj(tekst: &str) -> Option<f64> {
    let cist: String = tekst
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == ',' || *c == '-')
        .collect();
    cist.replace(',', ".").parse::<f64>().ok()
}

fn par_htrs_brojeva(dijelovi: &[&str]) -> Option<(f64, f64)> {
    if dijelovi.len() != 2 || ima_htrs_znak(dijelovi) {
        return None;
    }
    let br1 = ocisti_i_pretvori_broj(dijelovi[0])?;
    let br2 = ocisti_i_pretvori_broj(dijelovi[1])?;
    Some((br1, br2))
}

fn htrs_rasponi(e: f64, n: f64) -> bool {
    (100_000.0..=900_000.0).contains(&e) && (4_000_000.0..=6_000_000.0).contains(&n)
}

fn pretvori_htrs_par(istok: f64, sjever: f64, zamjena: bool) -> Option<IshodPrepoznavanja> {
    let p = Pretvornik::novi(Sustav::Htrs96Tm, Sustav::Wgs84).ok()?;
    let t = p.pretvori([istok, sjever]).ok()?;
    Some(IshodPrepoznavanja::Prepoznato(vec![PrepoznataKoordinata {
        duzina: t[0],
        sirina: t[1],
        sustav: Sustav::Htrs96Tm,
        zamijenjen_redoslijed: zamjena,
        izvorni_htrs: Some([istok, sjever]),
    }]))
}

fn obradi_htrs_brojeve(br1: f64, br2: f64) -> Option<IshodPrepoznavanja> {
    if htrs_rasponi(br1, br2) {
        pretvori_htrs_par(br1, br2, false)
    } else if htrs_rasponi(br2, br1) {
        pretvori_htrs_par(br2, br1, true)
    } else {
        Some(IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Htrs))
    }
}

/// Prepoznaje HTRS96/TM koordinate ako tekst sadrži dva velika broja u metrima.
fn pokusaj_htrs(dijelovi: &[&str]) -> Option<IshodPrepoznavanja> {
    let (br1, br2) = par_htrs_brojeva(dijelovi)?;
    if br1.abs() < 10_000.0 || br2.abs() < 10_000.0 {
        return None;
    }
    obradi_htrs_brojeve(br1, br2)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Smjer {
    Sjever,
    Jug,
    Istok,
    Zapad,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum VrstaOznake {
    Sirina(f64),
    Duzina(f64),
    Neodredjeno(f64),
}

fn slovo_dijela(tekst: &str) -> Option<char> {
    tekst
        .chars()
        .find(|c| c.is_alphabetic())
        .map(|c| c.to_ascii_uppercase())
}

fn smjer_slova_s(drugi: Option<char>) -> Smjer {
    if drugi == Some('E') || drugi == Some('W') {
        Smjer::Jug
    } else {
        Smjer::Sjever
    }
}

fn smjer_osnovnog_slova(c: char) -> Option<Smjer> {
    const ISTOK: &str = "IE";
    const ZAPAD: &str = "ZW";
    if c == 'N' {
        Some(Smjer::Sjever)
    } else if c == 'J' {
        Some(Smjer::Jug)
    } else if ISTOK.contains(c) {
        Some(Smjer::Istok)
    } else if ZAPAD.contains(c) {
        Some(Smjer::Zapad)
    } else {
        None
    }
}

fn razresi_smjer(slovo: Option<char>, drugi: Option<char>) -> Option<Smjer> {
    let c = slovo?;
    if c == 'S' {
        Some(smjer_slova_s(drugi))
    } else {
        smjer_osnovnog_slova(c)
    }
}

fn odredi_oznaku(stupnjevi: f64, smjer: Option<Smjer>) -> VrstaOznake {
    let s = stupnjevi.abs();
    match smjer {
        Some(Smjer::Sjever) => VrstaOznake::Sirina(s),
        Some(Smjer::Jug) => VrstaOznake::Sirina(-s),
        Some(Smjer::Istok) => VrstaOznake::Duzina(s),
        Some(Smjer::Zapad) => VrstaOznake::Duzina(-s),
        None => VrstaOznake::Neodredjeno(stupnjevi),
    }
}

fn provjeri_minute_i_sekunde(minute: f64, sekunde: Option<f64>) -> Result<(), RazlogRaspona> {
    if minute >= 60.0 {
        return Err(RazlogRaspona::Minute);
    }
    if let Some(s) = sekunde
        && s >= 60.0
    {
        return Err(RazlogRaspona::Sekunde);
    }
    Ok(())
}

fn izracunaj_stupnjeve(brojevi: &[f64], negativan: bool) -> Result<f64, RazlogRaspona> {
    let val = match brojevi {
        [deg] => *deg,
        [deg, min] => {
            provjeri_minute_i_sekunde(*min, None)?;
            *deg + *min / 60.0
        }
        [deg, min, sec] => {
            provjeri_minute_i_sekunde(*min, Some(*sec))?;
            *deg + *min / 60.0 + *sec / 3600.0
        }
        _ => return Err(RazlogRaspona::PreviseBrojeva),
    };
    Ok(if negativan { -val } else { val })
}

fn izdvoji_brojeve(tekst: &str) -> Option<Vec<f64>> {
    let dijelovi: Vec<&str> = tekst
        .split(|c: char| !c.is_ascii_digit() && c != '.' && c != ',')
        .filter(|s| !s.is_empty())
        .collect();
    if dijelovi.is_empty() {
        return None;
    }
    let mut brojevi = Vec::with_capacity(dijelovi.len());
    for dio in dijelovi {
        let broj = dio.replace(',', ".").parse::<f64>().ok()?;
        brojevi.push(broj);
    }
    Some(brojevi)
}

enum IshodKomponente {
    Valjano(f64, Option<char>),
    NijeKoordinata,
    IzvanRaspona(RazlogRaspona),
}

fn rasclani_komponentu(tekst: &str) -> IshodKomponente {
    let t = tekst.trim();
    if t.is_empty() {
        return IshodKomponente::NijeKoordinata;
    }
    let negativan = t.starts_with('-');
    let slovo = slovo_dijela(t);
    let Some(brojevi) = izdvoji_brojeve(t) else {
        return IshodKomponente::NijeKoordinata;
    };
    match izracunaj_stupnjeve(&brojevi, negativan) {
        Ok(stupnjevi) => IshodKomponente::Valjano(stupnjevi, slovo),
        Err(r) => IshodKomponente::IzvanRaspona(r),
    }
}

/// Razdvaja tekst na dva dijela koji predstavljaju dvije koordinate.
fn razdvoji_na_dva_dijela(tekst: &str) -> Option<(String, String)> {
    if let Some((a, b)) = tekst.split_once(';') {
        return Some((a.trim().to_string(), b.trim().to_string()));
    }
    if let Some((a, b)) = razdvoji_po_strani_svijeta(tekst) {
        return Some((a, b));
    }
    if let Some((a, b)) = razdvoji_po_zarezu(tekst) {
        return Some((a, b));
    }
    razdvoji_po_razmaku(tekst)
}

fn razdvoji_po_strani_svijeta(tekst: &str) -> Option<(String, String)> {
    const SLOVA: &str = "NSEWJIZnsewjiz";
    let idx = tekst.find(|c| SLOVA.contains(c))?;
    let (prvi, drugi) = tekst.split_at(idx + 1);
    if drugi.chars().any(|c| c.is_ascii_digit()) {
        let d = drugi.trim_start_matches([',', ' ', ';']);
        return Some((prvi.trim().to_string(), d.trim().to_string()));
    }
    None
}

fn je_cijeli_broj(s: &str) -> bool {
    let s = s.strip_prefix('-').unwrap_or(s);
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn zarez_dopusten(tekst: &str, a: &str, b: &str) -> bool {
    tekst.contains('.') || (je_cijeli_broj(a) && je_cijeli_broj(b))
}

fn razdvoji_jedan_zarez(tekst: &str) -> Option<(String, String)> {
    let (a, b) = tekst.split_once(',')?;
    let (a, b) = (a.trim(), b.trim());
    if a.is_empty() || b.is_empty() {
        return None;
    }
    if zarez_dopusten(tekst, a, b) {
        Some((a.to_string(), b.to_string()))
    } else {
        None
    }
}

fn razdvoji_tri_zareza(tekst: &str) -> Option<(String, String)> {
    let d: Vec<&str> = tekst.split(',').collect();
    if d.len() == 4 {
        Some((
            format!("{}.{}", d[0].trim(), d[1].trim()),
            format!("{}.{}", d[2].trim(), d[3].trim()),
        ))
    } else {
        None
    }
}

fn razdvoji_po_zarezu(tekst: &str) -> Option<(String, String)> {
    let br = tekst.chars().filter(|c| *c == ',').count();
    match br {
        1 => razdvoji_jedan_zarez(tekst),
        3 => razdvoji_tri_zareza(tekst),
        _ => None,
    }
}

fn razdvoji_po_razmaku(tekst: &str) -> Option<(String, String)> {
    let dijelovi: Vec<&str> = tekst.split_whitespace().collect();
    match dijelovi.len() {
        2 => Some((dijelovi[0].to_string(), dijelovi[1].to_string())),
        3 => {
            if dijelovi[2].chars().all(|c| "NSEWJIZnsewjiz".contains(c)) {
                Some((
                    dijelovi[0].to_string(),
                    format!("{} {}", dijelovi[1], dijelovi[2]),
                ))
            } else {
                None
            }
        }
        4 => {
            let p = format!("{} {}", dijelovi[0], dijelovi[1]);
            let d = format!("{} {}", dijelovi[2], dijelovi[3]);
            Some((p, d))
        }
        6 => {
            let p = format!("{} {} {}", dijelovi[0], dijelovi[1], dijelovi[2]);
            let d = format!("{} {} {}", dijelovi[3], dijelovi[4], dijelovi[5]);
            Some((p, d))
        }
        _ => None,
    }
}

/// Glavna funkcija za čitanje koordinata iz teksta polja tražilice.
#[must_use]
pub fn prepoznaj_koordinate(tekst: &str) -> IshodPrepoznavanja {
    let t = tekst.trim();
    if !t.chars().any(|c| c.is_ascii_digit()) || ima_nedopustena_slova(t) {
        return IshodPrepoznavanja::NijeKoordinata;
    }
    let dijelovi_razmak: Vec<&str> = t.split_whitespace().collect();
    if let Some(ishod) = pokusaj_htrs(&dijelovi_razmak) {
        return ishod;
    }
    let Some((prvi, drugi)) = razdvoji_na_dva_dijela(t) else {
        return IshodPrepoznavanja::NijeKoordinata;
    };
    sastavi_geografske_koordinate(&prvi, &drugi)
}

fn sastavi_geografske_koordinate(prvi: &str, drugi: &str) -> IshodPrepoznavanja {
    let k1 = match rasclani_komponentu(prvi) {
        IshodKomponente::Valjano(deg, slovo) => (deg, slovo),
        IshodKomponente::IzvanRaspona(r) => return IshodPrepoznavanja::IzvanRaspona(r),
        IshodKomponente::NijeKoordinata => return IshodPrepoznavanja::NijeKoordinata,
    };
    let k2 = match rasclani_komponentu(drugi) {
        IshodKomponente::Valjano(deg, slovo) => (deg, slovo),
        IshodKomponente::IzvanRaspona(r) => return IshodPrepoznavanja::IzvanRaspona(r),
        IshodKomponente::NijeKoordinata => return IshodPrepoznavanja::NijeKoordinata,
    };
    let smjer1 = razresi_smjer(k1.1, k2.1);
    let smjer2 = razresi_smjer(k2.1, k1.1);
    let oznaka1 = odredi_oznaku(k1.0, smjer1);
    let oznaka2 = odredi_oznaku(k2.0, smjer2);
    spoji_komponente(oznaka1, oznaka2)
}

fn spoji_sirinu_i_duzinu(k1: VrstaOznake, k2: VrstaOznake) -> Option<(f64, f64)> {
    match k1 {
        VrstaOznake::Sirina(s) => match k2 {
            VrstaOznake::Duzina(d) | VrstaOznake::Neodredjeno(d) => Some((s, d)),
            _ => None,
        },
        VrstaOznake::Duzina(d) => match k2 {
            VrstaOznake::Sirina(s) | VrstaOznake::Neodredjeno(s) => Some((s, d)),
            _ => None,
        },
        _ => None,
    }
}

fn spoji_komponente(k1: VrstaOznake, k2: VrstaOznake) -> IshodPrepoznavanja {
    if let (VrstaOznake::Neodredjeno(a), VrstaOznake::Neodredjeno(b)) = (k1, k2) {
        return obradi_neodredjeni_par(a, b);
    }
    let par = spoji_sirinu_i_duzinu(k1, k2).or_else(|| spoji_sirinu_i_duzinu(k2, k1));
    if let Some((s, d)) = par {
        provjeri_i_vrati_par(s, d, false)
    } else {
        IshodPrepoznavanja::NijeKoordinata
    }
}

fn provjeri_i_vrati_par(sirina: f64, duzina: f64, obrnut_ponudjen: bool) -> IshodPrepoznavanja {
    if !(-90.0..=90.0).contains(&sirina) {
        return IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Sirina);
    }
    if !(-180.0..=180.0).contains(&duzina) {
        return IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Duzina);
    }
    let mut rez = vec![PrepoznataKoordinata {
        duzina,
        sirina,
        sustav: Sustav::Wgs84,
        zamijenjen_redoslijed: false,
        izvorni_htrs: None,
    }];
    if obrnut_ponudjen {
        rez.push(PrepoznataKoordinata {
            duzina: sirina,
            sirina: duzina,
            sustav: Sustav::Wgs84,
            zamijenjen_redoslijed: true,
            izvorni_htrs: None,
        });
    }
    IshodPrepoznavanja::Prepoznato(rez)
}

fn obradi_neodredjeni_par(a: f64, b: f64) -> IshodPrepoznavanja {
    let u_hr_kao_zadano = je_u_hrvatskoj(a, b);
    let u_hr_zamijenjeno = je_u_hrvatskoj(b, a);
    let ponudi_obrnuto = !u_hr_kao_zadano && u_hr_zamijenjeno;
    provjeri_i_vrati_par(a, b, ponudi_obrnuto)
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn decimalni_stupnjevi_s_tockom_i_zarezom() {
        let ishod = prepoznaj_koordinate("45.5547, 18.6955");
        let tocke = match ishod {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke.len(), 1);
        assert_eq!(tocke[0].wgs84(), [18.6955, 45.5547]);
        assert_eq!(tocke[0].sustav, Sustav::Wgs84);
        assert!(!tocke[0].zamijenjen_redoslijed);
        assert_eq!(tocke[0].izvorni_htrs, None);

        let ishod_zarez = prepoznaj_koordinate("45,5547; 18,6955");
        let tocke_zarez = match ishod_zarez {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_zarez.len(), 1);
        assert!((tocke_zarez[0].sirina - 45.5547).abs() < 1e-4);
        assert!((tocke_zarez[0].duzina - 18.6955).abs() < 1e-4);

        let ishod_tri = prepoznaj_koordinate("45,5547, 18,6955");
        let tocke_tri = match ishod_tri {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_tri.len(), 1);
        assert!((tocke_tri[0].sirina - 45.5547).abs() < 1e-4);
        assert!((tocke_tri[0].duzina - 18.6955).abs() < 1e-4);

        let ishod_dva = prepoznaj_koordinate("45, 18");
        let tocke_dva = match ishod_dva {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_dva.len(), 1);
        assert_eq!(tocke_dva[0].sirina, 45.0);
        assert_eq!(tocke_dva[0].duzina, 18.0);

        let ishod_neg_dva = prepoznaj_koordinate("-45, -18");
        let tocke_neg_dva = match ishod_neg_dva {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_neg_dva.len(), 1);
        assert_eq!(tocke_neg_dva[0].sirina, -45.0);
        assert_eq!(tocke_neg_dva[0].duzina, -18.0);
    }

    #[test]
    fn stupnjevi_minute_sekunde_i_strane_svijeta() {
        let ishod = prepoznaj_koordinate("45°33'17\"N 18°41'44\"E");
        let tocke = match ishod {
            IshodPrepoznavanja::Prepoznato(t) => t,
            other => panic!("očekivano prepoznavanje, dobiveno: {other:?}"),
        };
        assert_eq!(tocke.len(), 1);
        assert!((tocke[0].sirina - (45.0 + 33.0 / 60.0 + 17.0 / 3600.0)).abs() < 1e-4);
        assert!((tocke[0].duzina - (18.0 + 41.0 / 60.0 + 44.0 / 3600.0)).abs() < 1e-4);

        // Obrnuti redoslijed s eksplicitnim stranama
        let ishod_obrnut = prepoznaj_koordinate("18°41'44\"E 45°33'17\"N");
        let tocke_obrnut = match ishod_obrnut {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_obrnut.len(), 1);
        assert!((tocke_obrnut[0].sirina - (45.0 + 33.0 / 60.0 + 17.0 / 3600.0)).abs() < 1e-4);
        assert!((tocke_obrnut[0].duzina - (18.0 + 41.0 / 60.0 + 44.0 / 3600.0)).abs() < 1e-4);

        // Hrvatski zapis: S uz I je sjever
        let ishod_hr = prepoznaj_koordinate("45 33 17 S 18 41 44 I");
        let tocke_hr = match ishod_hr {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_hr.len(), 1);
        assert!(tocke_hr[0].sirina > 0.0);
        assert!((tocke_hr[0].sirina - (45.0 + 33.0 / 60.0 + 17.0 / 3600.0)).abs() < 1e-4);
        assert!(tocke_hr[0].duzina > 0.0);
        assert!((tocke_hr[0].duzina - (18.0 + 41.0 / 60.0 + 44.0 / 3600.0)).abs() < 1e-4);

        // Hrvatski zapis: S uz Z je sjever
        let ishod_sz = prepoznaj_koordinate("51 30 00 S 0 7 12 Z");
        let tocke_sz = match ishod_sz {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_sz.len(), 1);
        assert!(tocke_sz[0].sirina > 0.0);
        assert!((tocke_sz[0].sirina - 51.5).abs() < 1e-4);
        assert!(tocke_sz[0].duzina < 0.0);
        assert!((tocke_sz[0].duzina - (-7.2 / 60.0)).abs() < 1e-4);

        // Engleski zapis: S uz E je jug
        let ishod_se = prepoznaj_koordinate("33 51 36 S 151 12 00 E");
        let tocke_se = match ishod_se {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_se.len(), 1);
        assert!(tocke_se[0].sirina < 0.0);
        assert!((tocke_se[0].sirina - (-(33.0 + 51.0 / 60.0 + 36.0 / 3600.0))).abs() < 1e-4);
        assert!(tocke_se[0].duzina > 0.0);
        assert!((tocke_se[0].duzina - (151.0 + 12.0 / 60.0)).abs() < 1e-4);

        // Engleski zapis: S uz W je jug
        let ishod_sw = prepoznaj_koordinate("33 51 36 S 70 39 36 W");
        let tocke_sw = match ishod_sw {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_sw.len(), 1);
        assert!(tocke_sw[0].sirina < 0.0);
        assert!(tocke_sw[0].duzina < 0.0);

        // Zapad i Jug: J je negativan, Z je negativan
        let ishod_zj = prepoznaj_koordinate("45 33 17 J 18 41 44 Z");
        let tocke_zj = match ishod_zj {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_zj.len(), 1);
        assert!(tocke_zj[0].sirina < 0.0);
        assert!((tocke_zj[0].sirina - (-(45.0 + 33.0 / 60.0 + 17.0 / 3600.0))).abs() < 1e-4);
        assert!(tocke_zj[0].duzina < 0.0);
        assert!((tocke_zj[0].duzina - (-(18.0 + 41.0 / 60.0 + 44.0 / 3600.0))).abs() < 1e-4);

        // Decimalne minute
        let ishod_dm = prepoznaj_koordinate("45 33.283 N 18 41.733 E");
        let tocke_dm = match ishod_dm {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_dm.len(), 1);
        assert!((tocke_dm[0].sirina - (45.0 + 33.283 / 60.0)).abs() < 1e-5);
        assert!((tocke_dm[0].duzina - (18.0 + 41.733 / 60.0)).abs() < 1e-5);

        // Bez slova: 4 broja i 6 brojeva
        let ishod_4 = prepoznaj_koordinate("45 33.283 18 41.733");
        let tocke_4 = match ishod_4 {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_4.len(), 1);
        assert!((tocke_4[0].sirina - (45.0 + 33.283 / 60.0)).abs() < 1e-5);
        assert!((tocke_4[0].duzina - (18.0 + 41.733 / 60.0)).abs() < 1e-5);

        let ishod_6 = prepoznaj_koordinate("45 33 17 18 41 44");
        let tocke_6 = match ishod_6 {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_6.len(), 1);
        assert!((tocke_6[0].sirina - (45.0 + 33.0 / 60.0 + 17.0 / 3600.0)).abs() < 1e-5);
        assert!((tocke_6[0].duzina - (18.0 + 41.0 / 60.0 + 44.0 / 3600.0)).abs() < 1e-5);

        // Miješan par: N i I
        let ishod_ni = prepoznaj_koordinate("45 N 18 I");
        let tocke_ni = match ishod_ni {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_ni.len(), 1);
        assert_eq!(tocke_ni[0].sirina, 45.0);
        assert_eq!(tocke_ni[0].duzina, 18.0);

        // S bez slova u drugoj koordinati je sjever
        let ishod_s_bez = prepoznaj_koordinate("45 S 18");
        let tocke_s_bez = match ishod_s_bez {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_s_bez.len(), 1);
        assert_eq!(tocke_s_bez[0].sirina, 45.0);
        assert_eq!(tocke_s_bez[0].duzina, 18.0);

        // Različite kombinacije određene i neodređene komponente
        let t_d_n = match prepoznaj_koordinate("18 E 45") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("18 E 45"),
        };
        assert_eq!(t_d_n[0].duzina, 18.0);
        assert_eq!(t_d_n[0].sirina, 45.0);

        let t_n_d = match prepoznaj_koordinate("45 18 E") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("45 18 E"),
        };
        assert_eq!(t_n_d[0].duzina, 18.0);
        assert_eq!(t_n_d[0].sirina, 45.0);

        let t_n_s = match prepoznaj_koordinate("18 45 N") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("18 45 N"),
        };
        assert_eq!(t_n_s[0].duzina, 18.0);
        assert_eq!(t_n_s[0].sirina, 45.0);
    }

    #[test]
    fn htrs_koordinate_u_metrima() {
        let ishod = prepoznaj_koordinate("670996.75 5047898.65");
        let tocke = match ishod {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke.len(), 1);
        assert_eq!(tocke[0].sustav, Sustav::Htrs96Tm);
        assert!(!tocke[0].zamijenjen_redoslijed);
        assert_eq!(tocke[0].izvorni_htrs, Some([670996.75, 5047898.65]));
        assert!((tocke[0].duzina - 18.69).abs() < 1e-2);
        assert!((tocke[0].sirina - 45.55).abs() < 1e-2);

        // Zamijenjen istok i sjever
        let ishod_zamjena = prepoznaj_koordinate("5047898.65 670996.75");
        let tocke_zamjena = match ishod_zamjena {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke_zamjena.len(), 1);
        assert!(tocke_zamjena[0].zamijenjen_redoslijed);
        assert_eq!(tocke_zamjena[0].izvorni_htrs, Some([670996.75, 5047898.65]));

        // HTRS izvan raspona
        let ishod_kriv = prepoznaj_koordinate("20000.0 30000.0");
        assert_eq!(
            ishod_kriv,
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Htrs)
        );

        // Negativan broj u HTRS rasponu mora biti odbijen kao HTRS izvan raspona
        let ishod_neg = prepoznaj_koordinate("-500000.0 5000000.0");
        assert_eq!(
            ishod_neg,
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Htrs)
        );

        // Granica 10_000 u pokusaj_htrs
        assert_eq!(
            prepoznaj_koordinate("10000.0 5000000.0"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Htrs)
        );
        assert_eq!(
            prepoznaj_koordinate("5000000.0 10000.0"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Htrs)
        );
        assert_eq!(
            prepoznaj_koordinate("89.0 5000000.0"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Duzina)
        );
        assert_eq!(
            prepoznaj_koordinate("5000000.0 89.0"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Sirina)
        );
    }

    #[test]
    fn zamjena_redoslijeda_nudi_hrvatsku() {
        let ishod = prepoznaj_koordinate("18.6955 45.5547");
        let tocke = match ishod {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje"),
        };
        assert_eq!(tocke.len(), 2);
        assert_eq!(tocke[0].sirina, 18.6955);
        assert!(!tocke[0].zamijenjen_redoslijed);
        assert_eq!(tocke[1].sirina, 45.5547);
        assert!(tocke[1].zamijenjen_redoslijed);

        // 0.0 0.0 mora dati točno 1 rezultat (ubija mutant && u || u obradi_neodredjeni_par)
        let ishod_nula = prepoznaj_koordinate("0.0 0.0");
        let tocke_nula = match ishod_nula {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje za 0.0 0.0"),
        };
        assert_eq!(tocke_nula.len(), 1);
        assert_eq!(tocke_nula[0].sirina, 0.0);
        assert_eq!(tocke_nula[0].duzina, 0.0);
    }

    #[test]
    fn izvan_raspona_i_nije_koordinata() {
        assert_eq!(
            prepoznaj_koordinate("cesta"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("Ulica grada Vukovara 58"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("12345"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(prepoznaj_koordinate(""), IshodPrepoznavanja::NijeKoordinata);
        assert_eq!(
            prepoznaj_koordinate("   "),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("abc"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("45 abc 18"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("s"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("n"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("m"),
            IshodPrepoznavanja::NijeKoordinata
        );

        // Upis koji sliči broju s neusklađenim zarezima mora vratiti NijeKoordinata
        assert_eq!(
            prepoznaj_koordinate("45.5, 18,5"),
            IshodPrepoznavanja::NijeKoordinata
        );

        assert_eq!(
            prepoznaj_koordinate("95.0, 18.0"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Sirina)
        );
        assert_eq!(
            prepoznaj_koordinate("-95.0, 18.0"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Sirina)
        );
        assert_eq!(
            prepoznaj_koordinate("45.0, 195.0"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Duzina)
        );
        assert_eq!(
            prepoznaj_koordinate("45.0, -195.0"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Duzina)
        );
        assert_eq!(
            prepoznaj_koordinate("45 65 00 N 18 00 00 E"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Minute)
        );
        assert_eq!(
            prepoznaj_koordinate("45 00 75 N 18 00 00 E"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Sekunde)
        );
    }

    #[test]
    fn pokrivenost_svih_grana_i_mutanata() {
        assert_eq!(
            prepoznaj_koordinate(",18"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("45,"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("45, 18a"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("45,55,18"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("45,55,18,66,77"),
            IshodPrepoznavanja::NijeKoordinata
        );

        // Previše brojeva u jednoj koordinati (4 broja)
        assert_eq!(
            prepoznaj_koordinate("45 33 17 22 N 18 41 44 E"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::PreviseBrojeva)
        );

        // Slovo na kraju bez druge koordinate
        assert_eq!(
            prepoznaj_koordinate("45N"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("45N   "),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("45N m"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("N18"),
            IshodPrepoznavanja::NijeKoordinata
        );

        // Zarez bez razmaka s decimalnim točkama
        let t_zarez_bez_razmaka = match prepoznaj_koordinate("45.5,18.5") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje za 45.5,18.5"),
        };
        assert_eq!(t_zarez_bez_razmaka[0].sirina, 45.5);
        assert_eq!(t_zarez_bez_razmaka[0].duzina, 18.5);

        // Razdvajanje po strani svijeta sa zarezom, točkom sa zarezom, bez razmaka
        let t_n18 = match prepoznaj_koordinate("45N18E") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje za 45N18E"),
        };
        assert_eq!(t_n18[0].sirina, 45.0);
        assert_eq!(t_n18[0].duzina, 18.0);

        let t_s18 = match prepoznaj_koordinate("45S 18E") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje za 45S 18E"),
        };
        assert_eq!(t_s18[0].sirina, -45.0);
        assert_eq!(t_s18[0].duzina, 18.0);

        let t_n_zarez = match prepoznaj_koordinate("45N, 18E") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje za 45N, 18E"),
        };
        assert_eq!(t_n_zarez[0].sirina, 45.0);
        assert_eq!(t_n_zarez[0].duzina, 18.0);

        let t_n_tocka_zarez = match prepoznaj_koordinate("45N; 18E") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje za 45N; 18E"),
        };
        assert_eq!(t_n_tocka_zarez[0].sirina, 45.0);
        assert_eq!(t_n_tocka_zarez[0].duzina, 18.0);

        // Pogrešna prva ili druga komponenta
        assert_eq!(
            prepoznaj_koordinate("45 65 N 18 E"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Minute)
        );
        assert_eq!(
            prepoznaj_koordinate("45 N 18 65 E"),
            IshodPrepoznavanja::IzvanRaspona(RazlogRaspona::Minute)
        );

        // Dvije širine ili dvije dužine
        assert_eq!(
            prepoznaj_koordinate("45 N 46 N"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert_eq!(
            prepoznaj_koordinate("18 E 19 E"),
            IshodPrepoznavanja::NijeKoordinata
        );

        // HTRS rubni slučajevi
        assert_eq!(
            prepoznaj_koordinate("500000 5000000 100"),
            IshodPrepoznavanja::NijeKoordinata
        );
        assert!(par_htrs_brojeva(&["-", "5000000"]).is_none());
        assert!(par_htrs_brojeva(&["500000", "-"]).is_none());

        // Nepoznato slovo
        assert_eq!(
            prepoznaj_koordinate("45X 18Y"),
            IshodPrepoznavanja::NijeKoordinata
        );

        // Prazan unos u rasclani_komponentu
        assert!(matches!(
            rasclani_komponentu(""),
            IshodKomponente::NijeKoordinata
        ));
        assert!(matches!(
            rasclani_komponentu("   "),
            IshodKomponente::NijeKoordinata
        ));

        // pretvori_htrs_par rubni slučaj s nevaljanim brojevima
        assert!(pretvori_htrs_par(f64::NAN, f64::NAN, false).is_none());

        // razdvoji pomoćne funkcije s netočnim formatima
        assert!(razdvoji_jedan_zarez("bez zareza").is_none());
        assert_eq!(razdvoji_jedan_zarez("45.5,"), None);
        assert_eq!(razdvoji_jedan_zarez(",18.5"), None);
        assert!(razdvoji_tri_zareza("1,2,3").is_none());
    }

    #[test]
    fn rubne_vrijednosti_i_negativne() {
        let ishod_90 = match prepoznaj_koordinate("90.0 180.0") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje 90 180"),
        };
        assert_eq!(ishod_90[0].sirina, 90.0);
        assert_eq!(ishod_90[0].duzina, 180.0);

        let ishod_neg_90 = match prepoznaj_koordinate("-90.0 -180.0") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje -90 -180"),
        };
        assert_eq!(ishod_neg_90[0].sirina, -90.0);
        assert_eq!(ishod_neg_90[0].duzina, -180.0);

        // -0 15 30 18 0 0 mora dati južnu širinu
        let ishod_neg_nula = match prepoznaj_koordinate("-0 15 30 18 0 0") {
            IshodPrepoznavanja::Prepoznato(t) => t,
            _ => panic!("očekivano prepoznavanje za -0 15 30"),
        };
        assert_eq!(ishod_neg_nula.len(), 1);
        assert!(
            ishod_neg_nula[0].sirina < 0.0,
            "širina mora biti negativna (južna): {}",
            ishod_neg_nula[0].sirina
        );
        let ocekivano = -(15.0 / 60.0 + 30.0 / 3600.0);
        assert!((ishod_neg_nula[0].sirina - ocekivano).abs() < 1e-6);
        assert_eq!(ishod_neg_nula[0].duzina, 18.0);
    }

    #[test]
    fn stupnjevi_i_minute_bez_sekundi() {
        let ishod = prepoznaj_koordinate("45°30' N 18°30' E");
        let IshodPrepoznavanja::Prepoznato(tocke) = ishod else {
            panic!("mora prepoznati stupnjeve i minute");
        };
        assert!((tocke[0].sirina - 45.5).abs() < 1e-6);
        assert!((tocke[0].duzina - 18.5).abs() < 1e-6);
    }

    #[test]
    fn granice_hrvatske() {
        assert!(je_u_hrvatskoj(45.0, 16.0));
        assert!(!je_u_hrvatskoj(40.0, 16.0));
        assert!(!je_u_hrvatskoj(48.0, 16.0));
        assert!(!je_u_hrvatskoj(45.0, 12.0));
        assert!(!je_u_hrvatskoj(45.0, 20.0));
        assert!(!je_u_hrvatskoj(40.0, 12.0));
    }

    #[test]
    fn pomocne_provjere_zareza_i_brojeva() {
        assert!(je_cijeli_broj("45"));
        assert!(!je_cijeli_broj("45.5"));
        assert!(!je_cijeli_broj("45,5"));
        assert!(zarez_dopusten("45.5, 18.5", "45.5", "18.5"));
        assert!(zarez_dopusten("45, 18", "45", "18"));
        assert!(!zarez_dopusten("45 33, 18", "45 33", "18"));

        assert_eq!(smjer_osnovnog_slova('N'), Some(Smjer::Sjever));
        assert_eq!(smjer_osnovnog_slova('J'), Some(Smjer::Jug));
        assert_eq!(smjer_osnovnog_slova('I'), Some(Smjer::Istok));
        assert_eq!(smjer_osnovnog_slova('E'), Some(Smjer::Istok));
        assert_eq!(smjer_osnovnog_slova('Z'), Some(Smjer::Zapad));
        assert_eq!(smjer_osnovnog_slova('W'), Some(Smjer::Zapad));
        assert_eq!(smjer_osnovnog_slova('M'), None);
        assert_eq!(smjer_osnovnog_slova('T'), None);
    }
}
