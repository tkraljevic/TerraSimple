//! Shapefile: geometrija iz `.shp`, nazivi iz `.dbf` i sustav iz `.prj`. Pretvara se u isti oblik
//! kao KML, pa dalje ide istim putem. Kad `.prj` nema ili ga program ne razumije, uvoz traži da
//! korisnik kaže sustav (zaslon L), a ne pretpostavlja ga.

use crate::citac::GreskaKml;
use crate::dbf;
use crate::model::{DokumentKml, GeometrijaKml, Gubici, MapaKml, ObjektKml, ispravan};
use koordinate::{Pretvornik, Sustav};
use slojevi::Vrh;
use std::path::Path;

/// Potpis `.shp` datoteke: broj 9994 zapisan od najvišeg bajta.
pub(crate) const POTPIS: [u8; 4] = [0, 0, 0x27, 0x0A];

const NAZIVI: [&str; 6] = ["naziv", "name", "ime", "title", "oznaka", "opis_naziv"];
const OPISI: [&str; 2] = ["opis", "description"];

fn f64_le(b: &[u8], od: usize) -> Option<f64> {
    b.get(od..od + 8)
        .and_then(|s| s.try_into().ok())
        .map(f64::from_le_bytes)
}

fn i32_le(b: &[u8], od: usize) -> Option<usize> {
    b.get(od..od + 4)
        .and_then(|s| s.try_into().ok())
        .map(i32::from_le_bytes)
        .and_then(|v| usize::try_from(v).ok())
}

fn i32_be(b: &[u8], od: usize) -> Option<usize> {
    b.get(od..od + 4)
        .and_then(|s| s.try_into().ok())
        .map(i32::from_be_bytes)
        .and_then(|v| usize::try_from(v).ok())
}

/// Pravila za `.prj`: svaka skupina mora imati barem jedan od natpisa.
const PRAVILA_PRJ: [(&[&[&str]], Sustav); 4] = [
    (&[&["htrs96"], &["tm", "transverse"]], Sustav::Htrs96Tm),
    (
        &[&["mercator_auxiliary", "pseudo_mercator"]],
        Sustav::WebMercator,
    ),
    (&[HDKS, &["zone_5", "balkans_5"]], Sustav::HdksZona5),
    (&[HDKS, &["zone_6", "balkans_6"]], Sustav::HdksZona6),
];
const HDKS: &[&str] = &["mgi", "hdks", "balkans", "gauss"];

/// Sustav iz `.prj` (WKT). Prepoznaju se samo sustavi s popisa programa.
#[must_use]
pub fn sustav_iz_prj(wkt: &str) -> Option<Sustav> {
    let t = wkt.to_ascii_lowercase().replace([' ', '-'], "_");
    let vrijedi = |skupine: &[&[&str]]| skupine.iter().all(|s| s.iter().any(|n| t.contains(n)));
    PRAVILA_PRJ
        .iter()
        .find(|(skupine, _)| vrijedi(skupine))
        .map(|(_, s)| *s)
        .or_else(|| (t.starts_with("geogcs") && t.contains("wgs")).then_some(Sustav::Wgs84))
}

/// Najvjerojatniji sustav po veličini koordinata, kao prijedlog u prozorčiću.
#[must_use]
pub fn prijedlog_sustava(istok: i64, sjever: i64) -> Sustav {
    match istok {
        -180..=180 if (-90..=90).contains(&sjever) => Sustav::Wgs84,
        5_300_000..=5_700_000 => Sustav::HdksZona5,
        6_300_000..=6_700_000 => Sustav::HdksZona6,
        _ => Sustav::Htrs96Tm,
    }
}

/// Točke zapisa: istok, sjever i visina ako je ima.
type Tocke = Vec<(f64, f64, Option<f64>)>;

fn tocke(c: &[u8], od: usize, n: usize, z: Option<usize>) -> Option<Tocke> {
    (0..n)
        .map(|i| {
            let xy = f64_le(c, od + 16 * i).zip(f64_le(c, od + 16 * i + 8));
            xy.map(|(x, y)| (x, y, z.and_then(|z| f64_le(c, z + 8 * i))))
        })
        .collect()
}

/// Jedan zapis: vrsta i dijelovi. Za točku je jedan dio s jednom točkom.
fn zapis(c: &[u8]) -> Option<(u32, Vec<Tocke>)> {
    let vrsta = i32_le(c, 0).and_then(|v| u32::try_from(v).ok())?;
    let z = (11..=18).contains(&vrsta);
    // 1 točka, 3 linija, 5 lik, 8 skupina točaka; +10 s visinom, +20 s mjerom.
    let osnovna = [1, 3, 5, 8]
        .into_iter()
        .find(|o| [*o, o + 10, o + 20].contains(&vrsta))?;
    match osnovna {
        1 => tocke(c, 4, 1, z.then_some(20)).map(|t| (1, vec![t])),
        8 => skupina(c, z),
        v => dijelovi(c, z).map(|d| (v, d)),
    }
}

fn skupina(c: &[u8], z: bool) -> Option<(u32, Vec<Tocke>)> {
    i32_le(c, 36)
        .and_then(|n| tocke(c, 40, n, z.then_some(40 + 16 * n + 16)))
        .map(|t| (8, t.into_iter().map(|t| vec![t]).collect()))
}

fn dijelovi(c: &[u8], z: bool) -> Option<Vec<Tocke>> {
    let (np, n) = i32_le(c, 36).zip(i32_le(c, 40))?;
    let od = 44 + 4 * np;
    let granice: Option<Vec<usize>> = (0..np)
        .map(|j| i32_le(c, 44 + 4 * j))
        .chain(std::iter::once(Some(n)))
        .collect();
    let sve = tocke(c, od, n, z.then_some(od + 16 * n + 16));
    sve.zip(granice).map(|(sve, g)| {
        g.windows(2)
            .filter_map(|g| sve.get(g[0]..g[1]).map(<[_]>::to_vec))
            .collect()
    })
}

/// Zapisi iz `.shp`, redom. Prazan ili nepoznat zapis je `None`.
fn zapisi(b: &[u8]) -> Vec<Option<(u32, Vec<Tocke>)>> {
    let kraj = b.len();
    let mut od = 100;
    let mut izlaz = Vec::new();
    while let Some(duljina) = i32_be(b, od + 4).filter(|_| od + 8 <= kraj) {
        let sadrzaj = b.get(od + 8..od + 8 + 2 * duljina).unwrap_or_default();
        izlaz.push(zapis(sadrzaj));
        od += 8 + 2 * duljina;
    }
    izlaz
}

/// Površina sa predznakom. Vanjski prsten shapefilea ide u smjeru kazaljke, pa je negativna.
fn povrsina(r: &[Vrh]) -> f64 {
    r.iter()
        .zip(r.iter().cycle().skip(1))
        .map(|(a, b)| a.duzina * b.sirina - b.duzina * a.sirina)
        .sum::<f64>()
        / 2.0
}

fn unutra(r: &[Vrh], t: Vrh) -> bool {
    r.iter()
        .zip(r.iter().cycle().skip(1))
        .filter(|(a, b)| (a.sirina > t.sirina) != (b.sirina > t.sirina))
        .filter(|(a, b)| {
            let x =
                a.duzina + (t.sirina - a.sirina) * (b.duzina - a.duzina) / (b.sirina - a.sirina);
            t.duzina < x
        })
        .count()
        % 2
        == 1
}

/// Prstenove slaže u likove: rupa ide u vanjski prsten koji je sadrži. Ako nijedan prsten nije
/// zapisan kao vanjski, svaki je svoj lik.
fn likovi(prstenovi: Vec<Vec<Vrh>>) -> Vec<Vec<Vec<Vrh>>> {
    let (vanjski, rupe): (Vec<_>, Vec<_>) = prstenovi.into_iter().partition(|r| povrsina(r) < 0.0);
    if vanjski.is_empty() {
        return rupe.into_iter().map(|r| vec![r]).collect();
    }
    let mut likovi: Vec<Vec<Vec<Vrh>>> = vanjski.into_iter().map(|r| vec![r]).collect();
    for rupa in rupe {
        let cilj = likovi.iter().position(|l| unutra(&l[0], rupa[0]));
        match cilj {
            Some(i) => likovi[i].push(rupa),
            None => likovi.push(vec![rupa]),
        }
    }
    likovi
}

struct Pretvorba(Option<Pretvornik>);

impl Pretvorba {
    fn vrhovi(&self, t: &Tocke) -> Option<Vec<Vrh>> {
        t.iter()
            .map(|(x, y, z)| {
                let [d, s] = match &self.0 {
                    Some(p) => p.pretvori([*x, *y]).ok()?,
                    None => [*x, *y],
                };
                ispravan(d, s, *z)
            })
            .collect()
    }
}

fn geometrije(vrsta: u32, d: Vec<Vec<Vrh>>) -> Vec<GeometrijaKml> {
    match vrsta {
        1 | 8 => d.into_iter().flatten().map(GeometrijaKml::Tocka).collect(),
        3 => d
            .into_iter()
            .filter(|v| v.len() >= 2)
            .map(GeometrijaKml::Linija)
            .collect(),
        _ => {
            let prstenovi = d
                .into_iter()
                .filter_map(crate::model::otvoren_prsten)
                .collect();
            likovi(prstenovi)
                .into_iter()
                .map(GeometrijaKml::Poligon)
                .collect()
        }
    }
}

fn objekt(
    z: Option<(u32, Vec<Tocke>)>,
    p: &Pretvorba,
    tablica: Option<&dbf::Tablica>,
    i: usize,
    g: &mut Gubici,
) -> Option<ObjektKml> {
    let (vrsta, dijelovi) = z?;
    let vrhovi: Option<Vec<Vec<Vrh>>> = dijelovi.iter().map(|d| p.vrhovi(d)).collect();
    if vrhovi.is_none() {
        g.neispravne_koordinate += 1;
    }
    let geometrije = geometrije(vrsta, vrhovi?);
    let tekst = |n: &[&str]| {
        tablica
            .and_then(|t| t.vrijednost(i, n))
            .unwrap_or_default()
            .to_owned()
    };
    (!geometrije.is_empty()).then(|| ObjektKml {
        naziv: tekst(&NAZIVI),
        opis: tekst(&OPISI),
        geometrije,
        ..ObjektKml::default()
    })
}

fn citaj(staza: &Path, nastavak: &str) -> Option<Vec<u8>> {
    std::fs::read(staza.with_extension(nastavak)).ok()
}

/// Sustav: zadani, iz `.prj` ili, ako ga nema, greška s koordinatom iz zaglavlja za prijedlog.
fn sustav(staza: &Path, b: &[u8], zadani: Option<Sustav>) -> Result<Sustav, GreskaKml> {
    let iz_prj = citaj(staza, "prj").and_then(|p| sustav_iz_prj(&dbf::dekodiraj(&p)));
    zadani.or(iz_prj).ok_or_else(|| GreskaKml::BezSustava {
        istok: f64_le(b, 36).unwrap_or_default() as i64,
        sjever: f64_le(b, 44).unwrap_or_default() as i64,
    })
}

fn ostala_polja(t: Option<&dbf::Tablica>) -> std::collections::BTreeSet<String> {
    let poznato = |n: &String| {
        NAZIVI
            .iter()
            .chain(&OPISI)
            .any(|p| n.eq_ignore_ascii_case(p))
    };
    t.iter()
        .flat_map(|t| &t.nazivi)
        .filter(|n| !poznato(n))
        .cloned()
        .collect()
}

/// Čita shapefile sa `staza` (`.shp`) i datotekama uz njega.
pub(crate) fn procitaj(
    staza: &Path,
    b: &[u8],
    zadani: Option<Sustav>,
) -> Result<DokumentKml, GreskaKml> {
    let s = sustav(staza, b, zadani)?;
    let p = Pretvorba(
        (s != Sustav::Wgs84)
            .then(|| Pretvornik::novi(s, Sustav::Wgs84).ok())
            .flatten(),
    );
    let tablica = citaj(staza, "dbf").and_then(|d| dbf::procitaj(&d));
    let mut gubici = Gubici {
        izostavljena_svojstva: ostala_polja(tablica.as_ref()),
        ..Gubici::default()
    };
    let mut objekti = Vec::new();
    for (i, z) in zapisi(b).into_iter().enumerate() {
        match objekt(z, &p, tablica.as_ref(), i, &mut gubici) {
            Some(o) => objekti.push(o),
            None => gubici.bez_geometrije += 1,
        }
    }
    let naziv = staza
        .file_stem()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(DokumentKml {
        korijen: MapaKml {
            naziv,
            objekti,
            ..MapaKml::default()
        },
        gubici,
    })
}

#[cfg(test)]
#[path = "shp_testovi.rs"]
mod testovi;
