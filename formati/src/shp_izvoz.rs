//! Izvoz u shapefile. Svaki sloj ide u svoj `.shp` s `.shx`, `.dbf`, `.prj` i `.cpg`, u sustavu koji
//! korisnik odabere, zadano HTRS96/TM, jer tako podloge traži direkcija. Tekst u `.dbf` je UTF-8, a
//! `.cpg` to kaže. Visina i stil ne idu u datoteku; pregled izvoza to javlja.

use crate::izvoz::{GreskaIzvoza, OpsegIzvoza};
use koordinate::{Pretvornik, Sustav};
use slojevi::{
    Geometrija, IdSloja, Objekt, Projekt, Sloj, Stavka, Vrh, Vrijednost, VrstaGeometrije,
};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Najveća širina tekstualnog polja u `.dbf`.
const SIRINA_POLJA: usize = 254;

/// Zapis sustava za `.prj`, kako ga piše GDAL (ESRI WKT).
#[must_use]
pub const fn prj(s: Sustav) -> &'static str {
    match s {
        Sustav::Wgs84 => {
            r#"GEOGCS["GCS_WGS_1984",DATUM["D_WGS_1984",SPHEROID["WGS_1984",6378137.0,298.257223563]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]]"#
        }
        Sustav::Htrs96Tm => {
            r#"PROJCS["HTRS96_Croatia_TM",GEOGCS["GCS_HTRS96",DATUM["D_Croatian_Terrestrial_Reference_System",SPHEROID["GRS_1980",6378137.0,298.257222101]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],PARAMETER["False_Easting",500000.0],PARAMETER["False_Northing",0.0],PARAMETER["Central_Meridian",16.5],PARAMETER["Scale_Factor",0.9999],PARAMETER["Latitude_Of_Origin",0.0],UNIT["Meter",1.0]]"#
        }
        Sustav::WebMercator => {
            r#"PROJCS["WGS_1984_Web_Mercator_Auxiliary_Sphere",GEOGCS["GCS_WGS_1984",DATUM["D_WGS_1984",SPHEROID["WGS_1984",6378137.0,298.257223563]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Mercator_Auxiliary_Sphere"],PARAMETER["False_Easting",0.0],PARAMETER["False_Northing",0.0],PARAMETER["Central_Meridian",0.0],PARAMETER["Standard_Parallel_1",0.0],PARAMETER["Auxiliary_Sphere_Type",0.0],UNIT["Meter",1.0]]"#
        }
        Sustav::HdksZona5 => {
            r#"PROJCS["MGI_1901_Balkans_zone_5",GEOGCS["GCS_MGI_1901",DATUM["D_MGI_1901",SPHEROID["Bessel_1841",6377397.155,299.1528128]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],PARAMETER["False_Easting",5500000.0],PARAMETER["False_Northing",0.0],PARAMETER["Central_Meridian",15.0],PARAMETER["Scale_Factor",0.9999],PARAMETER["Latitude_Of_Origin",0.0],UNIT["Meter",1.0]]"#
        }
        Sustav::HdksZona6 => {
            r#"PROJCS["MGI_1901_Balkans_zone_6",GEOGCS["GCS_MGI_1901",DATUM["D_MGI_1901",SPHEROID["Bessel_1841",6377397.155,299.1528128]],PRIMEM["Greenwich",0.0],UNIT["Degree",0.0174532925199433]],PROJECTION["Transverse_Mercator"],PARAMETER["False_Easting",6500000.0],PARAMETER["False_Northing",0.0],PARAMETER["Central_Meridian",18.0],PARAMETER["Scale_Factor",0.9999],PARAMETER["Latitude_Of_Origin",0.0],UNIT["Meter",1.0]]"#
        }
    }
}

/// Svi vrhovi geometrije, za provjeru visine i okvira.
pub(crate) fn vrhovi(g: &Geometrija) -> Box<dyn Iterator<Item = &Vrh> + '_> {
    match g {
        Geometrija::Tocka(v) => Box::new(std::iter::once(v)),
        Geometrija::Linija(v) => Box::new(v.iter()),
        Geometrija::Poligon(p) => Box::new(p.iter().flatten()),
    }
}

/// Slojevi u opsegu, redom stabla.
fn slojevi_opsega(p: &Projekt, opseg: OpsegIzvoza) -> Vec<IdSloja> {
    let mut stog: Vec<Stavka> = match opseg {
        OpsegIzvoza::Projekt => p.korijen().iter().rev().copied().collect(),
        OpsegIzvoza::Stavka(s) => vec![s],
    };
    let mut izlaz = Vec::new();
    while let Some(s) = stog.pop() {
        match s {
            Stavka::Sloj(id) => izlaz.push(id),
            Stavka::Mapa(m) => stog.extend(p.mapa(m).iter().flat_map(|m| m.djeca.iter().rev())),
        }
    }
    izlaz
}

/// Naziv datoteke od naziva sloja: slova, brojke i podvlaka.
fn ime(naziv: &str) -> String {
    let t: String = naziv
        .chars()
        .map(|z| if z.is_alphanumeric() { z } else { '_' })
        .collect();
    if t.is_empty() { "sloj".to_owned() } else { t }
}

/// Staze za slojeve: jedan sloj ide na odabranu stazu, a više njih dobiva naziv sloja iza nje.
fn staze(p: &Projekt, slojevi: &[IdSloja], staza: &Path) -> Vec<(IdSloja, PathBuf)> {
    if let [jedan] = slojevi {
        return vec![(*jedan, staza.with_extension("shp"))];
    }
    let osnova = staza
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut zauzeto = BTreeSet::new();
    slojevi
        .iter()
        .map(|id| {
            let naziv = ime(&p.sloj(*id).map(|s| s.naziv.clone()).unwrap_or_default());
            let mut kandidat = format!("{osnova}_{naziv}");
            if !zauzeto.insert(kandidat.clone()) {
                kandidat = format!("{kandidat}_{}", id.0);
                zauzeto.insert(kandidat.clone());
            }
            (*id, staza.with_file_name(kandidat).with_extension("shp"))
        })
        .collect()
}

struct Pisac {
    shp: Vec<u8>,
    shx: Vec<u8>,
    okvir: [f64; 4],
}

impl Pisac {
    fn zapis(&mut self, sadrzaj: &[u8]) {
        let broj = self.shx.len() as i32 / 8 + 1;
        let pomak = (100 + self.shp.len()) as i32 / 2;
        let duljina = sadrzaj.len() as i32 / 2;
        self.shp.extend(broj.to_be_bytes());
        self.shp.extend(duljina.to_be_bytes());
        self.shp.extend(sadrzaj);
        self.shx.extend(pomak.to_be_bytes());
        self.shx.extend(duljina.to_be_bytes());
    }

    fn prosiri(&mut self, t: [f64; 2]) {
        let o = &mut self.okvir;
        *o = [
            o[0].min(t[0]),
            o[1].min(t[1]),
            o[2].max(t[0]),
            o[3].max(t[1]),
        ];
    }

    fn zaglavlje(&self, vrsta: i32, duljina: usize) -> Vec<u8> {
        let mut z = Vec::with_capacity(100);
        z.extend(9994_i32.to_be_bytes());
        z.extend([0; 20]);
        z.extend(((100 + duljina) as i32 / 2).to_be_bytes());
        z.extend(1000_i32.to_le_bytes());
        z.extend(vrsta.to_le_bytes());
        let okvir = if self.okvir[0] <= self.okvir[2] {
            self.okvir
        } else {
            [0.0; 4]
        };
        okvir.iter().for_each(|v| z.extend(v.to_le_bytes()));
        z.extend([0; 32]);
        z
    }
}

fn okvir(t: &[[f64; 2]]) -> [f64; 4] {
    t.iter()
        .fold([f64::MAX, f64::MAX, f64::MIN, f64::MIN], |o, t| {
            [
                o[0].min(t[0]),
                o[1].min(t[1]),
                o[2].max(t[0]),
                o[3].max(t[1]),
            ]
        })
}

/// Površina sa predznakom; vanjski prsten shapefilea mora biti negativan (u smjeru kazaljke).
fn povrsina(r: &[[f64; 2]]) -> f64 {
    r.iter()
        .zip(r.iter().cycle().skip(1))
        .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
        .sum::<f64>()
        / 2.0
}

/// Zatvoren prsten u traženom smjeru.
fn prsten(mut r: Vec<[f64; 2]>, vanjski: bool) -> Vec<[f64; 2]> {
    if (povrsina(&r) < 0.0) != vanjski {
        r.reverse();
    }
    if r.first() != r.last() {
        r.extend(r.first().copied());
    }
    r
}

/// Sadržaj zapisa linije ili lika: okvir, dijelovi i točke.
fn dijelovi(vrsta: i32, d: &[Vec<[f64; 2]>]) -> Vec<u8> {
    let sve: Vec<[f64; 2]> = d.iter().flatten().copied().collect();
    let mut c = vrsta.to_le_bytes().to_vec();
    okvir(&sve).iter().for_each(|v| c.extend(v.to_le_bytes()));
    c.extend((d.len() as i32).to_le_bytes());
    c.extend((sve.len() as i32).to_le_bytes());
    let mut pocetak = 0;
    for dio in d {
        c.extend((pocetak as i32).to_le_bytes());
        pocetak += dio.len();
    }
    sve.iter().flatten().for_each(|v| c.extend(v.to_le_bytes()));
    c
}

/// Sadržaj zapisa i točke za okvir datoteke. Ništa ako se koordinata ne da pretvoriti.
fn sadrzaj(
    g: &Geometrija,
    pretvori: &dyn Fn(&Vrh) -> Option<[f64; 2]>,
) -> Option<(Vec<u8>, Vec<[f64; 2]>)> {
    let niz = |v: &[Vrh]| v.iter().map(pretvori).collect::<Option<Vec<_>>>();
    match g {
        Geometrija::Tocka(v) => pretvori(v).map(|t| {
            let mut c = 1_i32.to_le_bytes().to_vec();
            t.iter().for_each(|x| c.extend(x.to_le_bytes()));
            (c, vec![t])
        }),
        Geometrija::Linija(v) => niz(v).map(|t| (dijelovi(3, std::slice::from_ref(&t)), t)),
        Geometrija::Poligon(p) => {
            let prstenovi: Option<Vec<_>> = p.iter().map(|r| niz(r)).collect();
            prstenovi.map(|r| {
                let r: Vec<_> = r
                    .into_iter()
                    .enumerate()
                    .map(|(i, r)| prsten(r, i == 0))
                    .collect();
                let sve = r.iter().flatten().copied().collect();
                (dijelovi(5, &r), sve)
            })
        }
    }
}

/// Odakle dolazi vrijednost polja u `.dbf`.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Izvor {
    Naziv,
    Opis,
    Stupac(String),
}

/// Polja `.dbf`: naziv, opis i stupci iz GeoPackagea, s nazivima do 10 znakova bez ponavljanja.
fn polja(p: &Projekt, id: IdSloja) -> Vec<(String, Izvor)> {
    let mut izlaz = vec![
        ("NAZIV".to_owned(), Izvor::Naziv),
        ("OPIS".to_owned(), Izvor::Opis),
    ];
    let stupci: BTreeSet<&String> = p
        .zapisi()
        .slojevi
        .get(&id)
        .iter()
        .flat_map(|z| z.objekti.values())
        .flat_map(|o| o.nepoznati.keys())
        .collect();
    for stupac in stupci {
        let mut kratko: String = stupac
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .take(10)
            .collect();
        let mut i = 1;
        while kratko.is_empty() || izlaz.iter().any(|(n, _)| n.eq_ignore_ascii_case(&kratko)) {
            kratko = format!("POLJE{i}");
            i += 1;
        }
        izlaz.push((kratko, Izvor::Stupac(stupac.clone())));
    }
    izlaz
}

fn tekst(v: &Vrijednost) -> Option<String> {
    match v {
        Vrijednost::Cijeli(v) => Some(v.to_string()),
        Vrijednost::Broj(v) => Some(v.to_string()),
        Vrijednost::Tekst(t) => Some(t.clone()),
        Vrijednost::Prazno | Vrijednost::Bajtovi(_) => None,
    }
}

fn stupac(p: &Projekt, id: IdSloja, o: &Objekt, n: &str) -> String {
    p.zapisi()
        .slojevi
        .get(&id)
        .and_then(|z| z.objekti.get(&o.id))
        .and_then(|z| z.nepoznati.get(n))
        .and_then(tekst)
        .unwrap_or_default()
}

fn vrijednost(p: &Projekt, id: IdSloja, o: &Objekt, izvor: &Izvor) -> String {
    match izvor {
        Izvor::Naziv => o.naziv.clone(),
        Izvor::Opis => o.opis.clone(),
        Izvor::Stupac(n) => stupac(p, id, o, n),
    }
}

/// Tekst skraćen na širinu polja, bez lomljenja slova, i dopunjen razmacima.
fn polje(t: &str) -> Vec<u8> {
    let kraj = (0..=t.len().min(SIRINA_POLJA))
        .rev()
        .find(|k| t.is_char_boundary(*k))
        .unwrap_or_default();
    let mut b = t.as_bytes()[..kraj].to_vec();
    b.resize(SIRINA_POLJA, b' ');
    b
}

fn dbf(p: &Projekt, id: IdSloja, objekti: &[&Objekt]) -> Vec<u8> {
    let polja = polja(p, id);
    let zaglavlje = 32 + 32 * polja.len() + 1;
    let duljina = 1 + SIRINA_POLJA * polja.len();
    let mut b = vec![0x03, 126, 11, 14];
    b.extend((objekti.len() as u32).to_le_bytes());
    b.extend((zaglavlje as u16).to_le_bytes());
    b.extend((duljina as u16).to_le_bytes());
    b.resize(32, 0);
    for (naziv, _) in &polja {
        let mut o = [0u8; 32];
        o[..naziv.len()].copy_from_slice(naziv.as_bytes());
        o[11] = b'C';
        o[16] = SIRINA_POLJA as u8;
        b.extend(o);
    }
    b.push(0x0D);
    for o in objekti {
        b.push(b' ');
        for (_, izvor) in &polja {
            b.extend(polje(&vrijednost(p, id, o, izvor)));
        }
    }
    b.push(0x1A);
    b
}

const fn vrsta_shp(v: VrstaGeometrije) -> i32 {
    match v {
        VrstaGeometrije::Tocka => 1,
        VrstaGeometrije::Linija => 3,
        VrstaGeometrije::Poligon => 5,
    }
}

/// Datoteke jednog sloja. Objekt čija se koordinata ne da pretvoriti preskače se i broji.
fn datoteke_sloja(
    p: &Projekt,
    id: IdSloja,
    s: &Sloj,
    sustav: Sustav,
) -> ([Vec<u8>; 2], Vec<u8>, usize) {
    let pretvornik = (sustav != Sustav::Wgs84)
        .then(|| Pretvornik::novi(Sustav::Wgs84, sustav).ok())
        .flatten();
    let pretvori = |v: &Vrh| match &pretvornik {
        Some(pr) => pr.pretvori([v.duzina, v.sirina]).ok(),
        None => Some([v.duzina, v.sirina]),
    };
    let mut w = Pisac {
        shp: Vec::new(),
        shx: Vec::new(),
        okvir: [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
    };
    let mut zapisani = Vec::new();
    for o in s.objekti.values() {
        if let Some((c, tocke)) = sadrzaj(&o.geometrija, &pretvori) {
            tocke.iter().for_each(|t| w.prosiri(*t));
            w.zapis(&c);
            zapisani.push(o);
        }
    }
    let vrsta = vrsta_shp(s.vrsta);
    let shp = [w.zaglavlje(vrsta, w.shp.len()), w.shp.clone()].concat();
    let shx = [w.zaglavlje(vrsta, w.shx.len()), w.shx.clone()].concat();
    let preskoceno = s.objekti.len() - zapisani.len();
    ([shp, shx], dbf(p, id, &zapisani), preskoceno)
}

fn napisi_sloj(p: &Projekt, id: IdSloja, sustav: Sustav, staza: &Path) -> std::io::Result<()> {
    p.sloj(id).map_or(Ok(()), |s| {
        let ([shp, shx], dbf, _) = datoteke_sloja(p, id, s, sustav);
        [
            ("shp", shp),
            ("shx", shx),
            ("dbf", dbf),
            ("prj", prj(sustav).as_bytes().to_vec()),
            ("cpg", b"UTF-8".to_vec()),
        ]
        .into_iter()
        .try_for_each(|(n, b)| std::fs::write(staza.with_extension(n), b))
    })
}

/// Piše svaki sloj opsega koji ima objekte u svoj shapefile. Vraća staze napisanih `.shp`.
pub fn izvezi_shp(
    p: &Projekt,
    opseg: OpsegIzvoza,
    sustav: Sustav,
    staza: &Path,
) -> Result<Vec<PathBuf>, GreskaIzvoza> {
    let slojevi: Vec<IdSloja> = slojevi_opsega(p, opseg)
        .into_iter()
        .filter(|id| p.sloj(*id).is_some_and(|s| !s.objekti.is_empty()))
        .collect();
    let staze = staze(p, &slojevi, staza);
    for (id, staza) in &staze {
        napisi_sloj(p, *id, sustav, staza)?;
    }
    Ok(staze.into_iter().map(|(_, s)| s).collect())
}

#[cfg(test)]
#[path = "shp_izvoz_testovi.rs"]
mod testovi;
