//! Proba GeoTIFF-a: otvara li se velika slika bez čekanja i bez punjenja memorije.
//!
//! `proba_geotiff prolaz <datoteka> [mapa za slike]` mjeri otvaranje, prvu sliku i kretanje.
//! `proba_geotiff usporedi <datoteka> <referenca.csv>` uspoređuje s GDAL-om.
//! `proba_geotiff cijela <datoteka>` pročita sve, što je donja granica za izradu piramida.

mod citac;

use citac::{Slika, EKRAN};
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

/// Najveći utrošak memorije procesa u MB. Na macOS-u `ru_maxrss` je u bajtovima.
fn vrh_memorije_mb() -> f64 {
    let mut r: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut r) };
    r.ru_maxrss as f64 / 1e6
}

/// Izvor slučajnih brojeva s istim sjemenom, da svako mjerenje ide istim putem.
struct Slucajni(u64);

impl Slucajni {
    fn sljedeci(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn do_(&mut self, n: u64) -> u64 {
        self.sljedeci() % n.max(1)
    }
}

fn spremi_sliku(mapa: &str, ime: &str, s: usize, v: usize, podaci: &[u8]) {
    let mut izlaz = format!("P6\n{s} {v}\n255\n").into_bytes();
    izlaz.extend_from_slice(podaci);
    std::fs::write(format!("{mapa}/{ime}.ppm"), izlaz).expect("pisanje slike");
}

fn prolaz(put: &str, mapa: Option<&String>) -> Result<(), String> {
    let t = Instant::now();
    let mut slika = Slika::otvori(Path::new(put))?;
    let otvaranje = t.elapsed().as_secs_f64() * 1000.0;
    let (s, v) = slika.velicina();

    let t = Instant::now();
    let cijela = slika.pogled(0.0, 0.0, s as f64, v as f64)?;
    let prva = t.elapsed().as_secs_f64() * 1000.0;
    if let Some(m) = mapa {
        spremi_sliku(m, &format!("{}_cijela", citac::ime(put)), EKRAN.0, EKRAN.1, &cijela);
    }

    // Kretanje po slici: mjerila kao pri približavanju, s nasumičnim mjestima.
    let mut slucajni = Slucajni(0x2026_1107);
    let mut vremena = Vec::new();
    let mut zadnji = Vec::new();
    for korak in 0..100 {
        let mjerilo = [1.0, 2.0, 4.0, 8.0, 16.0][(korak % 5) as usize];
        let (sp, vp) = (EKRAN.0 as f64 * mjerilo, EKRAN.1 as f64 * mjerilo);
        let x = slucajni.do_((s as f64 - sp).max(1.0) as u64) as f64;
        let y = slucajni.do_((v as f64 - vp).max(1.0) as u64) as f64;
        let t = Instant::now();
        zadnji = slika.pogled(x, y, sp, vp)?;
        vremena.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    if let Some(m) = mapa {
        spremi_sliku(m, &format!("{}_detalj", citac::ime(put)), EKRAN.0, EKRAN.1, &zadnji);
    }
    vremena.sort_by(f64::total_cmp);
    println!(
        "{:<28} {:>6.0} MB  {:>6} x {:<6} razina {:>2}  otvaranje {:>7.1} ms  prva {:>8.0} ms  pomak med {:>6.1} / naj {:>7.1} ms  vrh {:>6.0} MB",
        citac::ime(put),
        std::fs::metadata(put).map(|m| m.len() as f64 / 1e6).unwrap_or(0.0),
        s,
        v,
        slika.broj_razina(),
        otvaranje,
        prva,
        vremena[vremena.len() / 2],
        vremena[vremena.len() - 1],
        vrh_memorije_mb()
    );
    println!("  {}", slika.opis());
    Ok(())
}

fn blizu(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// Vraća koliko je provjera palo, a Err ako se datoteka nije mogla ni otvoriti.
fn usporedi(put: &str, referenca: &str) -> Result<u32, String> {
    let tekst = std::fs::read_to_string(referenca).map_err(|g| g.to_string())?;
    let mapa = Path::new(referenca).parent().expect("mapa reference");
    if tekst.starts_with("neotvoriva") {
        return match Slika::otvori(Path::new(put)) {
            Err(g) => {
                println!("  u redu  GDAL je ne otvara, a ni mi: uredna greška bez pada ({})", g.chars().take(60).collect::<String>());
                Ok(0)
            }
            Ok(_) => {
                println!("  --      GDAL je ne otvara, a mi je otvaramo");
                Ok(0)
            }
        };
    }
    let mut slika = Slika::otvori(Path::new(put))?;
    let mut palo = 0;
    let mut provjeri = |uvjet: bool, opis: String| {
        println!("  {}  {opis}", if uvjet { "u redu" } else { "PALO  " });
        if !uvjet {
            palo += 1;
        }
    };
    let (mut isjecaka, mut najveca) = (0, 0_u8);
    let (mut zbroj, mut vece_od_osam, mut uzoraka) = (0_u64, 0_u64, 0_u64);
    for redak in tekst.lines() {
        let p: Vec<&str> = redak.split(';').collect();
        match p[0] {
            "geo" => {
                let geo = slika.polozaj();
                let ocekivano: Vec<f64> = p[1..p.len() - 1].iter().filter_map(|v| v.parse().ok()).collect();
                let epsg = p[p.len() - 1];
                if ocekivano.len() == 6 {
                    let g = geo.as_ref().map(|g| g.preobrazba);
                    let dobro = g.is_some_and(|g| g.iter().zip(&ocekivano).all(|(a, b)| blizu(*a, *b)));
                    provjeri(dobro, format!("položaj {:?}", g.map(|g| [g[0], g[1], g[3], g[5]])));
                } else {
                    provjeri(geo.is_none(), "položaja nema ni kod GDAL-a".into());
                }
                let nas = geo.as_ref().and_then(|g| g.epsg).map(|e| e.to_string()).unwrap_or_default();
                provjeri(nas == epsg, format!("sustav: naš '{nas}', GDAL '{epsg}'"));
            }
            "razina" => {
                let (i, s, v): (usize, u32, u32) = (p[1].parse().unwrap(), p[2].parse().unwrap(), p[3].parse().unwrap());
                let nasa = slika.velicina_razine(i);
                provjeri(nasa == Some((s, v)), format!("razina {i}: {nasa:?}"));
            }
            "isjecak" => {
                let n: Vec<u32> = p[1..6].iter().map(|v| v.parse().unwrap()).collect();
                let ocekivano = std::fs::read(mapa.join(p[6])).map_err(|g| g.to_string())?;
                let nas = slika.isjecak(n[0] as usize, n[1], n[2], n[3], n[4])?;
                isjecaka += 1;
                if nas.len() != ocekivano.len() {
                    provjeri(false, format!("isječak {}: {} bajtova, GDAL {}", p[6], nas.len(), ocekivano.len()));
                    continue;
                }
                for (a, b) in nas.iter().zip(&ocekivano) {
                    let d = a.abs_diff(*b);
                    najveca = najveca.max(d);
                    zbroj += u64::from(d);
                    vece_od_osam += u64::from(d > 8);
                    uzoraka += 1;
                }
            }
            _ => {}
        }
    }
    let prosjek = zbroj as f64 / uzoraka.max(1) as f64;
    let udio = vece_od_osam as f64 / uzoraka.max(1) as f64 * 100.0;
    let opis = format!("{isjecaka} isječaka, razlika u boji: najveća {najveca}, prosječna {prosjek:.2} od 255, iznad 8 u {udio:.3} % uzoraka");
    // Dva JPEG dekodera ne moraju dati isti bajt: razlikuju se oko oštrih rubova, jer boju
    // rastežu drukčije. Zato se gleda prosjek i udio. Sve ostalo mora biti jednako.
    let dobro = if slika.jpeg() { prosjek <= 2.0 && udio <= 2.0 } else { najveca == 0 };
    provjeri(isjecaka > 0 && dobro, opis);
    Ok(palo)
}

fn cijela(put: &str) -> Result<(), String> {
    let t = Instant::now();
    let mut slika = Slika::otvori(Path::new(put))?;
    let bajtova = slika.procitaj_sve()?;
    println!(
        "{:<28} cijela: {:.1} s, {:.0} MB piksela, vrh {:.0} MB",
        citac::ime(put),
        t.elapsed().as_secs_f64(),
        bajtova as f64 / 1e6,
        vrh_memorije_mb()
    );
    Ok(())
}

fn main() -> ExitCode {
    let a: Vec<String> = std::env::args().collect();
    let rezultat = match (a.get(1).map(String::as_str), a.get(2)) {
        (Some("prolaz"), Some(put)) => prolaz(put, a.get(3)).map(|()| 0),
        (Some("usporedi"), Some(put)) => usporedi(put, a.get(3).expect("referenca")),
        (Some("cijela"), Some(put)) => cijela(put).map(|()| 0),
        _ => {
            eprintln!("upotreba: proba_geotiff prolaz|usporedi|cijela <datoteka> [...]");
            return ExitCode::from(2);
        }
    };
    match rezultat {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(g) => {
            println!("  GREŠKA: {g}");
            ExitCode::from(3)
        }
    }
}
