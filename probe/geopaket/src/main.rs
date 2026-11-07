//! Proba faze 0: razumije li QGIS naš GeoPackage, i mi njegov. Ovo se baca.
//!
//! proba_geopaket pisi <datoteka>                        novi projekt s tri sloja i stilom
//! proba_geopaket citaj <datoteka>                       JSON svega što program vidi
//! proba_geopaket dodaj <datoteka> <sloj> <x> <y> <tekst> [stupac]   dodaje objekt u bilo koji sloj
//! proba_geopaket izmijeni <datoteka> <sloj> <fid> <stupac> <tekst>  mijenja vrijednost
//! proba_geopaket drzi <datoteka> <sekundi>              drži zaključano pisanje
//! proba_geopaket pazi <datoteka> <sekundi>              javi čim netko drugi promijeni datoteku
#![forbid(unsafe_code)]

mod paket;
mod stil;

use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use geo_types::{Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon};
use rusqlite::Connection;
use rusqlite::types::Value;

use paket::{Rezultat, dodaj_objekt, dodaj_sloj, lik, otvori, pravokutnik, spremi_stil, stvori};
use stil::{Simbol, qml};

fn tekst(s: &str) -> Value {
    Value::Text(s.to_string())
}

fn pisi(put: &str) -> Rezultat<()> {
    if std::path::Path::new(put).exists() {
        return Err(format!("{put} već postoji").into());
    }
    let mut c = otvori(put)?;
    let t = c.transaction()?;
    stvori(&t)?;
    tocke(&t)?;
    linije(&t)?;
    parcele(&t)?;
    nase_tablice(&t)?;
    t.commit()?;
    Ok(())
}

fn tocke(c: &Connection) -> Rezultat<()> {
    dodaj_sloj(c, "tocke", "POINT", 4326, &[
        ("naziv", "TEXT"), ("opis", "TEXT"), ("broj", "INTEGER"), ("visina", "REAL"), ("datum", "DATE"), ("vrijeme", "DATETIME"), ("aktivno", "BOOLEAN"),
        ("stil_boja", "TEXT"), ("stil_velicina", "REAL"), ("stil_znak", "TEXT"),
    ])?;
    let zajedno = |naziv: &str, opis: &str, broj: i64, stil: [Value; 3]| {
        let [a, b, d] = stil;
        vec![
            ("naziv", tekst(naziv)), ("opis", tekst(opis)), ("broj", Value::Integer(broj)), ("visina", Value::Real(84.25)),
            ("datum", tekst("2026-11-07")), ("vrijeme", tekst("2026-11-07T10:20:30.000Z")), ("aktivno", Value::Integer(1)),
            ("stil_boja", a), ("stil_velicina", b), ("stil_znak", d),
        ]
    };
    let bez = || [Value::Null, Value::Null, Value::Null];
    dodaj_objekt(c, "tocke", &Geometry::Point(Point::new(18.68, 45.54)), &zajedno("Šećerana čćžšđ ČĆŽŠĐ", "Red\nnovi 'navodnik' \"dvostruki\" & <znak>", 1, bez()))?;
    dodaj_objekt(c, "tocke", &Geometry::Point(Point::new(18.69, 45.54)), &zajedno("Crveni kvadrat", "vlastiti stil", 2, [tekst("#DC2626"), Value::Real(8.0), tekst("square")]))?;
    dodaj_objekt(c, "tocke", &Geometry::Point(Point::new(18.70, 45.54)), &zajedno("Treća točka", "za brisanje u QGIS-u", 3, bez()))?;
    let stupci = ["fid", "naziv", "opis", "broj", "visina", "datum", "vrijeme", "aktivno", "stil_boja", "stil_velicina", "stil_znak"];
    spremi_stil(c, "tocke", "TerraSimple", &qml(&Simbol::Tocka { boja: "#0f766e", velicina: 4.0, znak: "circle" }, &stupci, "naziv"))
}

fn linije(c: &Connection) -> Rezultat<()> {
    let sloj = "Nasipi čđžšć";
    dodaj_sloj(c, sloj, "LINESTRING", 3765, &[("naziv", "TEXT"), ("stil_boja", "TEXT"), ("stil_debljina", "REAL"), ("stil_crta", "TEXT")])?;
    let crta = |y: f64| Geometry::LineString(LineString::from(vec![(670000.0, y), (671000.0, y), (672000.0, y)]));
    dodaj_objekt(c, sloj, &crta(5044800.0), &[("naziv", tekst("Stil sloja"))])?;
    dodaj_objekt(c, sloj, &crta(5044300.0), &[("naziv", tekst("Vlastiti stil")), ("stil_boja", tekst("#DC2626")), ("stil_debljina", Value::Real(1.6)), ("stil_crta", tekst("dash"))])?;
    // Bez vlastitog stila, a iscrtava se odmah iza objekta s njim.
    dodaj_objekt(c, sloj, &crta(5044550.0), &[("naziv", tekst("Opet stil sloja"))])?;
    let stupci = ["fid", "naziv", "stil_boja", "stil_debljina", "stil_crta"];
    spremi_stil(c, sloj, "TerraSimple", &qml(&Simbol::Linija { boja: "#0f766e", debljina: 0.8, crta: "solid" }, &stupci, "naziv"))
}

fn parcele(c: &Connection) -> Rezultat<()> {
    dodaj_sloj(c, "parcele", "POLYGON", 3765, &[("naziv", "TEXT"), ("stil_boja", "TEXT"), ("stil_ispuna", "TEXT"), ("stil_debljina", "REAL")])?;
    let s_rupom = lik(pravokutnik(670000.0, 5045000.0, 670800.0, 5045800.0), vec![pravokutnik(670300.0, 5045300.0, 670500.0, 5045500.0)]);
    dodaj_objekt(c, "parcele", &s_rupom, &[("naziv", tekst("S rupom"))])?;
    let plava = lik(pravokutnik(671000.0, 5045000.0, 671800.0, 5045800.0), vec![]);
    dodaj_objekt(c, "parcele", &plava, &[("naziv", tekst("Vlastita ispuna")), ("stil_boja", tekst("#1D4ED8")), ("stil_ispuna", tekst("#2563EB")), ("stil_debljina", Value::Real(1.0))])?;
    let opet = lik(pravokutnik(671900.0, 5045000.0, 672100.0, 5045200.0), vec![]);
    dodaj_objekt(c, "parcele", &opet, &[("naziv", tekst("Opet"))])?;
    let stupci = ["fid", "naziv", "stil_boja", "stil_ispuna", "stil_debljina"];
    spremi_stil(c, "parcele", "TerraSimple", &qml(&Simbol::Lik { ispuna: "#fbbf24", rub: "#92400e", debljina: 0.5 }, &stupci, "naziv"))
}

/// Mape i stanje sučelja. Tablice s predmetkom `gpkg_` QGIS ne nudi kao slojeve, a GeoPackage
/// taj predmetak dopušta samo proširenjima, pa je ovo upisano kao proširenje.
fn nase_tablice(c: &Connection) -> Rezultat<()> {
    c.execute_batch(
        "CREATE TABLE gpkg_ts_stablo (fid INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, roditelj INTEGER, naziv TEXT, tablica TEXT, redoslijed INTEGER);
         INSERT INTO gpkg_ts_stablo (roditelj, naziv, tablica, redoslijed) VALUES (NULL, 'Teren čćž', NULL, 1), (1, 'Točke', 'tocke', 1), (1, 'Nasipi', 'Nasipi čđžšć', 2);
         CREATE TABLE gpkg_ts_stanje (kljuc TEXT PRIMARY KEY, vrijednost TEXT);
         INSERT INTO gpkg_ts_stanje VALUES ('tema', 'tamna'), ('ploca', 'desno');
         INSERT INTO gpkg_extensions VALUES ('gpkg_ts_stablo', NULL, 'ts_projekt', 'TerraSimple: mape slojeva', 'read-write'), ('gpkg_ts_stanje', NULL, 'ts_projekt', 'TerraSimple: stanje sučelja', 'read-write');",
    )?;
    Ok(())
}

/// Objekt vrste kakvu sloj traži, oko zadane točke, u mjerilu sustava sloja.
fn objekt_za_sloj(vrsta: &str, x: f64, y: f64, korak: f64) -> Rezultat<Geometry<f64>> {
    let crta = || LineString::from(vec![(x, y), (x + korak, y), (x + korak * 2.0, y + korak)]);
    let kvadrat = || Polygon::new(pravokutnik(x, y, x + korak, y + korak), vec![]);
    Ok(match vrsta {
        "POINT" => Geometry::Point(Point::new(x, y)),
        "LINESTRING" => Geometry::LineString(crta()),
        "POLYGON" => Geometry::Polygon(kvadrat()),
        "MULTIPOINT" => Geometry::MultiPoint(MultiPoint(vec![Point::new(x, y)])),
        "MULTILINESTRING" => Geometry::MultiLineString(MultiLineString(vec![crta()])),
        "MULTIPOLYGON" => Geometry::MultiPolygon(MultiPolygon(vec![kvadrat()])),
        d => return Err(format!("vrstu {d} proba ne zna dodati").into()),
    })
}

fn dodaj(put: &str, sloj: &str, x: f64, y: f64, vrijednost: &str, stupac: &str) -> Rezultat<()> {
    let mut c = otvori(put)?;
    let t = c.transaction()?;
    let (vrsta, srs): (String, i32) = t.query_row("SELECT geometry_type_name, srs_id FROM gpkg_geometry_columns WHERE table_name = ?1", [sloj], |r| Ok((r.get(0)?, r.get(1)?)))?;
    let korak = if srs == 4326 { 0.002 } else { 100.0 };
    let g = objekt_za_sloj(&vrsta, x, y, korak)?;
    let fid = dodaj_objekt(&t, sloj, &g, &[(stupac, tekst(vrijednost))])?;
    t.commit()?;
    println!("{fid}");
    Ok(())
}

fn izmijeni(put: &str, sloj: &str, fid: i64, stupac: &str, vrijednost: &str) -> Rezultat<()> {
    let c = otvori(put)?;
    let n = c.execute(&format!("UPDATE \"{sloj}\" SET \"{stupac}\" = ?1 WHERE fid = ?2"), rusqlite::params![vrijednost, fid])?;
    println!("{n}");
    Ok(())
}

fn drzi(put: &str, sekundi: f64) -> Rezultat<()> {
    let mut c = otvori(put)?;
    let t = c.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
    println!("zaključano");
    std::io::stdout().flush()?;
    sleep(Duration::from_secs_f64(sekundi));
    t.commit()?;
    Ok(())
}

/// Otvoreni program mora sam primijetiti da je QGIS nešto spremio.
fn pazi(put: &str, sekundi: f64) -> Rezultat<()> {
    let c = otvori(put)?;
    let inacica = |c: &Connection| c.query_row("PRAGMA data_version", [], |r| r.get::<_, i64>(0));
    let prva = inacica(&c)?;
    println!("gledam");
    std::io::stdout().flush()?;
    let pocetak = std::time::Instant::now();
    while pocetak.elapsed().as_secs_f64() < sekundi {
        if inacica(&c)? != prva {
            println!("promjena");
            return Ok(());
        }
        sleep(Duration::from_millis(50));
    }
    println!("ništa");
    Ok(())
}

fn pokreni() -> Rezultat<()> {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let a: Vec<&str> = a.iter().map(String::as_str).collect();
    match a.as_slice() {
        ["pisi", put] => pisi(put),
        ["citaj", put] => {
            let c = otvori(put)?;
            println!("{}", serde_json::to_string_pretty(&paket::citaj_sve(&c)?)?);
            Ok(())
        }
        ["dodaj", put, sloj, x, y, vrijednost] => dodaj(put, sloj, x.parse()?, y.parse()?, vrijednost, "naziv"),
        ["dodaj", put, sloj, x, y, vrijednost, stupac] => dodaj(put, sloj, x.parse()?, y.parse()?, vrijednost, stupac),
        ["izmijeni", put, sloj, fid, stupac, vrijednost] => izmijeni(put, sloj, fid.parse()?, stupac, vrijednost),
        ["drzi", put, sekundi] => drzi(put, sekundi.parse()?),
        ["pazi", put, sekundi] => pazi(put, sekundi.parse()?),
        _ => Err("nepoznata naredba, vidi početak main.rs".into()),
    }
}

fn main() {
    if let Err(e) = pokreni() {
        eprintln!("GREŠKA: {e}");
        std::process::exit(1);
    }
}
