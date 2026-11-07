//! Proba koordinata: slaže li se naš račun s pravim QGIS-om.
//!
//! `proba_koordinate usporedi <referenca.csv>`

mod racun;

use proj4rs::proj::Proj;
use proj4rs::transform::transform;
use std::collections::BTreeMap;
use std::process::ExitCode;

/// Zapisana granica iz plana: 1 mm prema QGIS-u.
const GRANICA_M: f64 = 0.001;
const GRANICA_KUT: f64 = 0.001;
const METAR_PO_STUPNJU: f64 = 111_320.0;

fn definicija(sustav: &str) -> String {
    let mgi = |zona: u8, pomak: &str| {
        format!(
            "+proj=tmerc +lat_0=0 +lon_0={} +k=0.9999 +x_0={} +y_0=0 +ellps=bessel +towgs84={pomak} +units=m +no_defs",
            u32::from(zona) * 3,
            u32::from(zona) * 1_000_000 + 500_000
        )
    };
    let prvi = "682,-203,480,0,0,0,0";
    let cetvrti = "551.7,162.9,467.9,6.04,1.96,-11.38,-4.82";
    match sustav {
        "4326" => "+proj=longlat +datum=WGS84 +no_defs".into(),
        "3857" => "+proj=merc +a=6378137 +b=6378137 +lat_ts=0 +lon_0=0 +x_0=0 +y_0=0 +k=1 +units=m +nadgrids=@null +wktext +no_defs".into(),
        "3765" => "+proj=tmerc +lat_0=0 +lon_0=16.5 +k=0.9999 +x_0=500000 +y_0=0 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs".into(),
        "3907" | "8677" => mgi(5, prvi),
        "3908" | "8678" => mgi(6, prvi),
        "3907#4" => mgi(5, cetvrti),
        "3908#4" => mgi(6, cetvrti),
        _ => panic!("nepoznat sustav {sustav}"),
    }
}

/// Pretvorba preko biblioteke `proj4rs`. Geografske koordinate su joj u radijanima.
fn biblioteka(iz: &Proj, u: &Proj, geo_iz: bool, geo_u: bool, x: f64, y: f64) -> (f64, f64) {
    let mut t = if geo_iz { (x.to_radians(), y.to_radians(), 0.0) } else { (x, y, 0.0) };
    transform(iz, u, &mut t).expect("pretvorba proj4rs");
    if geo_u {
        (t.0.to_degrees(), t.1.to_degrees())
    } else {
        (t.0, t.1)
    }
}

/// Naš račun, s WGS-om 84 u sredini. Tako će raditi i program.
fn u_wgs(sustav: &str, x: f64, y: f64) -> (f64, f64) {
    let bessel = |zona: u8, h: &racun::Helmert| {
        let (lon, lat) = racun::hdks(zona).natrag(x, y);
        racun::bessel_u_wgs(h, lon, lat)
    };
    match sustav {
        "4326" => (x, y),
        "3857" => racun::mercator_natrag(x, y),
        "3765" => racun::htrs().natrag(x, y),
        "3907" | "8677" => bessel(5, &racun::MGI_1),
        "3908" | "8678" => bessel(6, &racun::MGI_1),
        "3907#4" => bessel(5, &racun::MGI_4),
        "3908#4" => bessel(6, &racun::MGI_4),
        _ => panic!("nepoznat sustav {sustav}"),
    }
}

fn iz_wgs(sustav: &str, lon: f64, lat: f64) -> (f64, f64) {
    let bessel = |zona: u8, h: &racun::Helmert| {
        let (b_lon, b_lat) = racun::wgs_u_bessel(h, lon, lat);
        racun::hdks(zona).naprijed(b_lon, b_lat)
    };
    match sustav {
        "4326" => (lon, lat),
        "3857" => racun::mercator_naprijed(lon, lat),
        "3765" => racun::htrs().naprijed(lon, lat),
        "3907" | "8677" => bessel(5, &racun::MGI_1),
        "3908" | "8678" => bessel(6, &racun::MGI_1),
        "3907#4" => bessel(5, &racun::MGI_4),
        "3908#4" => bessel(6, &racun::MGI_4),
        _ => panic!("nepoznat sustav {sustav}"),
    }
}

fn razlika_m(u: &str, a: (f64, f64), b: (f64, f64)) -> f64 {
    if u == "4326" {
        let de = (a.0 - b.0) * METAR_PO_STUPNJU * b.1.to_radians().cos();
        let dn = (a.1 - b.1) * METAR_PO_STUPNJU;
        de.hypot(dn)
    } else {
        (a.0 - b.0).hypot(a.1 - b.1)
    }
}

#[derive(Default)]
struct Grupa {
    tocaka: usize,
    biblioteka: f64,
    nas: f64,
}

fn usporedi_pretvorbe(redci: &[Vec<&str>]) -> bool {
    let mut grupe: BTreeMap<String, Grupa> = BTreeMap::new();
    let mut sustavi: BTreeMap<String, Proj> = BTreeMap::new();
    for r in redci {
        let Some((iz, u)) = r[0].split_once('>') else { continue };
        for s in [iz, u] {
            sustavi.entry(s.to_string()).or_insert_with(|| {
                Proj::from_proj_string(&definicija(s)).expect("definicija sustava")
            });
        }
        let br = |i: usize| r[i].parse::<f64>().expect("broj");
        let (x, y) = (br(1), br(2));
        let qgis = (br(3), br(4));
        let b = biblioteka(&sustavi[iz], &sustavi[u], iz == "4326", u == "4326", x, y);
        let wgs = u_wgs(iz, x, y);
        let n = iz_wgs(u, wgs.0, wgs.1);
        let g = grupe.entry(r[0].to_string()).or_default();
        g.tocaka += 1;
        g.biblioteka = g.biblioteka.max(razlika_m(u, b, qgis));
        g.nas = g.nas.max(razlika_m(u, n, qgis));
    }
    println!("{:<16} {:>6} {:>16} {:>16}", "smjer", "točaka", "proj4rs, mm", "naš račun, mm");
    let mut sve_dobro = !grupe.is_empty();
    for (ime, g) in &grupe {
        let dobro = g.biblioteka <= GRANICA_M && g.nas <= GRANICA_M;
        sve_dobro &= dobro;
        println!(
            "{:<16} {:>6} {:>16.6} {:>16.6} {}",
            ime,
            g.tocaka,
            g.biblioteka * 1000.0,
            g.nas * 1000.0,
            if dobro { "" } else { "IZNAD GRANICE" }
        );
    }
    sve_dobro
}

/// Zakret okvira lista na ekranu: sjever u HTRS96/TM gledan u Mercatoru.
fn usporedi_kut(redci: &[Vec<&str>]) -> bool {
    let mut najveca: f64 = 0.0;
    let mut osijek = (0.0_f64, f64::MAX);
    let mut ima = false;
    for r in redci.iter().filter(|r| r[0] == "kut") {
        ima = true;
        let br = |i: usize| r[i].parse::<f64>().expect("broj");
        let (x, y) = (br(1), br(2));
        let tocka = |x: f64, y: f64| {
            let (lon, lat) = racun::htrs().natrag(x, y);
            racun::mercator_naprijed(lon, lat)
        };
        let (x0, y0) = tocka(x, y);
        let (x1, y1) = tocka(x, y + 1000.0);
        let kut = (x1 - x0).atan2(y1 - y0).to_degrees();
        najveca = najveca.max((kut - br(3)).abs());
        let udaljenost = (x - 670_996.0).hypot(y - 5_047_898.0);
        if udaljenost < osijek.1 {
            osijek = (kut, udaljenost);
        }
    }
    println!(
        "zakret okvira: najveća razlika prema QGIS-u {najveca:.7}°, u Osijeku {:.3}°",
        osijek.0
    );
    ima && najveca <= GRANICA_KUT
}

/// Mjerilo ekrana: koliko griješi jednostavno 1/cos(širina) prema pravom tlu.
fn usporedi_mjerilo(redci: &[Vec<&str>]) {
    let (mut najv_i, mut najv_s): (f64, f64) = (0.0, 0.0);
    for r in redci.iter().filter(|r| r[0] == "mjerilo") {
        let br = |i: usize| r[i].parse::<f64>().expect("broj");
        let jednostavno = 1.0 / br(2).to_radians().cos();
        najv_i = najv_i.max((jednostavno / br(3) - 1.0).abs());
        najv_s = najv_s.max((jednostavno / br(4) - 1.0).abs());
    }
    println!(
        "mjerilo ekrana 1/cos(širina): najveća greška istok-zapad {:.3} %, sjever-jug {:.3} %",
        najv_i * 100.0,
        najv_s * 100.0
    );
}

/// Krug u QGIS-u: iz WGS-a u sustav i natrag. Koliko se vrati pored početne točke.
fn pomak_kruga(redci: &[Vec<&str>], sustav: &str) -> f64 {
    let naprijed = format!("4326>{sustav}");
    let natrag = format!("{sustav}>4326");
    let br = |r: &Vec<&str>, i: usize| r[i].parse::<f64>().expect("broj");
    let prvi: Vec<_> = redci.iter().filter(|r| r[0] == naprijed).collect();
    let drugi: Vec<_> = redci.iter().filter(|r| r[0] == natrag).collect();
    assert_eq!(prvi.len(), drugi.len(), "{sustav}");
    prvi.iter().zip(&drugi).fold(0.0, |naj: f64, (a, b)| {
        naj.max(razlika_m("4326", (br(b, 3), br(b, 4)), (br(a, 1), br(a, 2))))
    })
}

/// Čitanje sustava iz definicije u datoteci, preko biblioteke `proj4wkt`.
fn iz_definicije(mapa: &str) {
    println!("{:<6} {:>14}  definicija pretvorena u PROJ", "sustav", "pored QGIS-a");
    let wgs = Proj::from_proj_string(&definicija("4326")).expect("WGS");
    for s in ["3765", "3857", "3907", "3908", "8677", "8678"] {
        let wkt = std::fs::read_to_string(format!("{mapa}/wkt_{s}.txt")).expect("čitanje definicije");
        let tekst = match proj4wkt::wkt_to_projstring(&wkt) {
            Ok(t) => t,
            Err(g) => {
                println!("{s:<6} {:>14}  ne čita se: {g:?}", "-");
                continue;
            }
        };
        let sustav = match Proj::from_proj_string(&tekst) {
            Ok(p) => p,
            Err(g) => {
                println!("{s:<6} {:>14}  {tekst} ne prima: {g:?}", "-");
                continue;
            }
        };
        let tocna = iz_wgs(s, 18.69, 45.55);
        let cilj = (18.69, 45.55);
        let mut t = (tocna.0, tocna.1, 0.0);
        let rezultat = transform(&sustav, &wgs, &mut t);
        let pomak = match rezultat {
            Ok(()) => razlika_m("4326", (t.0.to_degrees(), t.1.to_degrees()), cilj),
            Err(g) => {
                println!("{s:<6} {:>14}  {tekst} pretvorba: {g:?}", "-");
                continue;
            }
        };
        println!("{s:<6} {pomak:>12.3} m  {tekst}");
    }
}

/// Koliko traje pretvorba milijuna točaka iz HDKS-a u Mercator ekrana.
fn brzina() {
    const BROJ: u32 = 1_000_000;
    let iz = Proj::from_proj_string(&definicija("3908")).expect("3908");
    let u = Proj::from_proj_string(&definicija("3857")).expect("3857");
    let tocka = |i: u32| (6_554_280.0 + f64::from(i % 1000), 5_045_300.0 + f64::from(i % 777));
    let pocetak = std::time::Instant::now();
    let mut zbroj = 0.0;
    for i in 0..BROJ {
        let (x, y) = tocka(i);
        zbroj += biblioteka(&iz, &u, false, false, x, y).0;
    }
    let trajanje_b = pocetak.elapsed();
    let pocetak = std::time::Instant::now();
    for i in 0..BROJ {
        let (x, y) = tocka(i);
        let wgs = u_wgs("3908", x, y);
        zbroj += iz_wgs("3857", wgs.0, wgs.1).0;
    }
    println!(
        "milijun točaka 3908 > 3857: proj4rs {:.0} ms, naš račun {:.0} ms (zbroj {zbroj:.0})",
        trajanje_b.as_secs_f64() * 1000.0,
        pocetak.elapsed().as_secs_f64() * 1000.0
    );
}

fn main() -> ExitCode {
    let argumenti: Vec<String> = std::env::args().collect();
    if argumenti.len() == 2 && argumenti[1] == "brzina" {
        brzina();
        return ExitCode::SUCCESS;
    }
    if argumenti.len() == 3 && argumenti[1] == "definicije" {
        iz_definicije(&argumenti[2]);
        return ExitCode::SUCCESS;
    }
    if argumenti.len() != 3 || argumenti[1] != "usporedi" {
        eprintln!("upotreba: proba_koordinate usporedi <referenca.csv> | definicije <mapa>");
        return ExitCode::from(2);
    }
    let tekst = std::fs::read_to_string(&argumenti[2]).expect("čitanje referentne datoteke");
    let redci: Vec<Vec<&str>> = tekst.lines().skip(1).map(|l| l.split(',').collect()).collect();
    let pretvorbe = usporedi_pretvorbe(&redci);
    let kut = usporedi_kut(&redci);
    usporedi_mjerilo(&redci);
    for s in ["3765", "3857", "3908", "3908#4"] {
        println!("krug u QGIS-u kroz {s:<7} vrati se {:.3} mm pored", pomak_kruga(&redci, s) * 1000.0);
    }
    if pretvorbe && kut {
        println!("PROŠLO: sve unutar {} mm i {GRANICA_KUT}°", GRANICA_M * 1000.0);
        ExitCode::SUCCESS
    } else {
        println!("PALO: ima razlika iznad granice");
        ExitCode::FAILURE
    }
}
