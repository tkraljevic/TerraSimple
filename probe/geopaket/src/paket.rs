//! Čitanje i pisanje GeoPackagea preko običnog SQLite-a.
//!
//! Sheme tablica i okidača prepisane su iz datoteke koju je zapisao QGIS 3.44 (GDAL 3.13).

use std::time::Duration;

use geo_types::{Coord, Geometry, LineString, Polygon, Rect};
use geozero::wkb::{GpkgWkb, WkbDialect};
use geozero::wkt::WktDialect;
use geozero::{CoordDimensions, ToGeo, ToWkb, ToWkt};
use rusqlite::functions::FunctionFlags;
use rusqlite::types::{Value, ValueRef};
use rusqlite::{Connection, params};
use serde_json::{Map, Value as Json, json};

pub type Rezultat<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

const WKT_3765: &str = r#"PROJCS["HTRS96 / Croatia TM",GEOGCS["HTRS96",DATUM["Croatian_Terrestrial_Reference_System",SPHEROID["GRS 1980",6378137,298.257222101,AUTHORITY["EPSG","7019"]],AUTHORITY["EPSG","6761"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AUTHORITY["EPSG","4761"]],PROJECTION["Transverse_Mercator"],PARAMETER["latitude_of_origin",0],PARAMETER["central_meridian",16.5],PARAMETER["scale_factor",0.9999],PARAMETER["false_easting",500000],PARAMETER["false_northing",0],UNIT["metre",1,AUTHORITY["EPSG","9001"]],AXIS["Easting",EAST],AXIS["Northing",NORTH],AUTHORITY["EPSG","3765"]]"#;
const WKT_4326: &str = r#"GEOGCS["WGS 84",DATUM["WGS_1984",SPHEROID["WGS 84",6378137,298.257223563,AUTHORITY["EPSG","7030"]],AUTHORITY["EPSG","6326"]],PRIMEM["Greenwich",0,AUTHORITY["EPSG","8901"]],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AXIS["Latitude",NORTH],AXIS["Longitude",EAST],AUTHORITY["EPSG","4326"]]"#;

/// Otvara datoteku i dodaje funkcije koje traže okidači prostornog indeksa.
pub fn otvori(put: &str) -> Rezultat<Connection> {
    let c = Connection::open(put)?;
    c.busy_timeout(Duration::from_secs(5))?;
    dodaj_funkcije(&c)?;
    Ok(c)
}

/// `ST_*` nisu dio SQLite-a. GDAL ih zapisuje u okidače, pa bez njih nije moguće dodati
/// objekt ni u jednu datoteku koju je napravio QGIS.
pub fn dodaj_funkcije(c: &Connection) -> rusqlite::Result<()> {
    let z = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC | FunctionFlags::SQLITE_INNOCUOUS;
    c.create_scalar_function("ST_IsEmpty", 1, z, |ctx| Ok(ctx.get_raw(0).as_blob().ok().map(je_prazna)))?;
    c.create_scalar_function("ST_MinX", 1, z, |ctx| Ok(ctx.get_raw(0).as_blob().ok().and_then(omotnica).map(|o| o.min().x)))?;
    c.create_scalar_function("ST_MaxX", 1, z, |ctx| Ok(ctx.get_raw(0).as_blob().ok().and_then(omotnica).map(|o| o.max().x)))?;
    c.create_scalar_function("ST_MinY", 1, z, |ctx| Ok(ctx.get_raw(0).as_blob().ok().and_then(omotnica).map(|o| o.min().y)))?;
    c.create_scalar_function("ST_MaxY", 1, z, |ctx| Ok(ctx.get_raw(0).as_blob().ok().and_then(omotnica).map(|o| o.max().y)))?;
    Ok(())
}

/// Četvrti bajt zaglavlja nosi oznaku prazne geometrije. `geozero` je pročita pa baci.
pub fn je_prazna(blob: &[u8]) -> bool {
    blob.len() < 4 || blob[3] & 0b0001_0000 != 0
}

fn koordinate(g: &Geometry<f64>, sve: &mut Vec<Coord<f64>>) {
    match g {
        Geometry::Point(p) => sve.push(p.0),
        Geometry::Line(l) => sve.extend([l.start, l.end]),
        Geometry::LineString(l) => sve.extend(l.0.iter().copied()),
        Geometry::Polygon(p) => {
            sve.extend(p.exterior().0.iter().copied());
            p.interiors().iter().for_each(|i| sve.extend(i.0.iter().copied()));
        }
        Geometry::MultiPoint(m) => sve.extend(m.0.iter().map(|p| p.0)),
        Geometry::MultiLineString(m) => m.0.iter().for_each(|l| sve.extend(l.0.iter().copied())),
        Geometry::MultiPolygon(m) => m.0.iter().for_each(|p| koordinate(&Geometry::Polygon(p.clone()), sve)),
        Geometry::GeometryCollection(k) => k.0.iter().for_each(|g| koordinate(g, sve)),
        Geometry::Rect(r) => sve.extend([r.min(), r.max()]),
        Geometry::Triangle(t) => sve.extend(t.to_array()),
    }
}

/// Omotnica se računa iz koordinata, jer je GDAL ne zapisuje uz točke.
pub fn omotnica(blob: &[u8]) -> Option<Rect<f64>> {
    if je_prazna(blob) {
        return None;
    }
    let g = GpkgWkb(blob).to_geo().ok()?;
    omotnica_geometrije(&g)
}

fn omotnica_geometrije(g: &Geometry<f64>) -> Option<Rect<f64>> {
    let mut sve = Vec::new();
    koordinate(g, &mut sve);
    let prva = *sve.first()?;
    let (mut a, mut b) = (prva, prva);
    for k in &sve {
        a = Coord { x: a.x.min(k.x), y: a.y.min(k.y) };
        b = Coord { x: b.x.max(k.x), y: b.y.max(k.y) };
    }
    Some(Rect::new(a, b))
}

/// Geometrija u zapisu GeoPackagea, s omotnicom kako je piše i QGIS, osim za točke.
pub fn u_blob(g: &Geometry<f64>, srs: i32) -> Rezultat<Vec<u8>> {
    let omot = match g {
        Geometry::Point(_) => Vec::new(),
        _ => omotnica_geometrije(g).map(|o| vec![o.min().x, o.max().x, o.min().y, o.max().y]).unwrap_or_default(),
    };
    Ok(g.to_wkb_dialect(WkbDialect::Geopackage, CoordDimensions::xy(), Some(srs), omot)?)
}

pub fn stvori(c: &Connection) -> Rezultat<()> {
    c.execute_batch(
        "PRAGMA application_id = 1196444487;
         PRAGMA user_version = 10400;
         CREATE TABLE gpkg_spatial_ref_sys (srs_name TEXT NOT NULL,srs_id INTEGER NOT NULL PRIMARY KEY,organization TEXT NOT NULL,organization_coordsys_id INTEGER NOT NULL,definition TEXT NOT NULL,description TEXT);
         CREATE TABLE gpkg_contents (table_name TEXT NOT NULL PRIMARY KEY,data_type TEXT NOT NULL,identifier TEXT UNIQUE,description TEXT DEFAULT '',last_change DATETIME NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),min_x DOUBLE, min_y DOUBLE,max_x DOUBLE, max_y DOUBLE,srs_id INTEGER,CONSTRAINT fk_gc_r_srs_id FOREIGN KEY (srs_id) REFERENCES gpkg_spatial_ref_sys(srs_id));
         CREATE TABLE gpkg_geometry_columns (table_name TEXT NOT NULL,column_name TEXT NOT NULL,geometry_type_name TEXT NOT NULL,srs_id INTEGER NOT NULL,z TINYINT NOT NULL,m TINYINT NOT NULL,CONSTRAINT pk_geom_cols PRIMARY KEY (table_name, column_name),CONSTRAINT uk_gc_table_name UNIQUE (table_name),CONSTRAINT fk_gc_tn FOREIGN KEY (table_name) REFERENCES gpkg_contents(table_name),CONSTRAINT fk_gc_srs FOREIGN KEY (srs_id) REFERENCES gpkg_spatial_ref_sys (srs_id));
         CREATE TABLE gpkg_extensions (table_name TEXT,column_name TEXT,extension_name TEXT NOT NULL,definition TEXT NOT NULL,scope TEXT NOT NULL,CONSTRAINT ge_tce UNIQUE (table_name, column_name, extension_name));
         CREATE TABLE layer_styles (id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, f_table_catalog TEXT(256), f_table_schema TEXT(256), f_table_name TEXT(256), f_geometry_column TEXT(256), styleName TEXT(30), styleQML TEXT, styleSLD TEXT, useAsDefault BOOLEAN, description TEXT, owner TEXT(30), ui TEXT(30), update_time DATETIME DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')));",
    )?;
    let mut upis = c.prepare("INSERT INTO gpkg_spatial_ref_sys VALUES (?,?,?,?,?,?)")?;
    upis.execute(params!["Undefined Cartesian SRS", -1, "NONE", -1, "undefined", "undefined Cartesian coordinate reference system"])?;
    upis.execute(params!["Undefined geographic SRS", 0, "NONE", 0, "undefined", "undefined geographic coordinate reference system"])?;
    upis.execute(params!["WGS 84 geodetic", 4326, "EPSG", 4326, WKT_4326, "longitude/latitude coordinates in decimal degrees on the WGS 84 spheroid"])?;
    upis.execute(params!["HTRS96 / Croatia TM", 3765, "EPSG", 3765, WKT_3765, Option::<String>::None])?;
    // Sustavi moraju biti upisani prije ovoga: rusqlite ima uključene vanjske ključeve.
    c.execute("INSERT INTO gpkg_contents (table_name, data_type, identifier, srs_id) VALUES ('layer_styles', 'attributes', 'layer_styles', 0)", [])?;
    Ok(())
}

fn navodnici(ime: &str) -> String {
    format!("\"{}\"", ime.replace('"', "\"\""))
}

/// Novi sloj s prostornim indeksom i istim okidačima koje piše GDAL.
pub fn dodaj_sloj(c: &Connection, tablica: &str, vrsta: &str, srs: i32, stupci: &[(&str, &str)]) -> Rezultat<()> {
    let t = navodnici(tablica);
    let ostali: Vec<String> = stupci.iter().map(|(i, v)| format!("{} {v}", navodnici(i))).collect();
    c.execute_batch(&format!("CREATE TABLE {t} (\"fid\" INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, \"geom\" {vrsta}, {});", ostali.join(", ")))?;
    c.execute("INSERT INTO gpkg_contents (table_name, data_type, identifier, srs_id) VALUES (?1, 'features', ?1, ?2)", params![tablica, srs])?;
    c.execute("INSERT INTO gpkg_geometry_columns VALUES (?1, 'geom', ?2, ?3, 0, 0)", params![tablica, vrsta, srs])?;
    c.execute("INSERT INTO gpkg_extensions VALUES (?1, 'geom', 'gpkg_rtree_index', 'http://www.geopackage.org/spec120/#extension_rtree', 'write-only')", params![tablica])?;
    let r = navodnici(&format!("rtree_{tablica}_geom"));
    let unos = format!("VALUES (NEW.\"fid\",ST_MinX(NEW.\"geom\"), ST_MaxX(NEW.\"geom\"),ST_MinY(NEW.\"geom\"), ST_MaxY(NEW.\"geom\"))");
    c.execute_batch(&format!(
        "CREATE VIRTUAL TABLE {r} USING rtree(id, minx, maxx, miny, maxy);
         CREATE TRIGGER {} AFTER INSERT ON {t} WHEN (new.\"geom\" NOT NULL AND NOT ST_IsEmpty(NEW.\"geom\")) BEGIN INSERT OR REPLACE INTO {r} {unos}; END;
         CREATE TRIGGER {} AFTER UPDATE OF \"geom\" ON {t} WHEN OLD.\"fid\" = NEW.\"fid\" AND (NEW.\"geom\" ISNULL OR ST_IsEmpty(NEW.\"geom\")) BEGIN DELETE FROM {r} WHERE id = OLD.\"fid\"; END;
         CREATE TRIGGER {} AFTER UPDATE ON {t} WHEN OLD.\"fid\" != NEW.\"fid\" AND (NEW.\"geom\" ISNULL OR ST_IsEmpty(NEW.\"geom\")) BEGIN DELETE FROM {r} WHERE id IN (OLD.\"fid\", NEW.\"fid\"); END;
         CREATE TRIGGER {} AFTER UPDATE ON {t} WHEN OLD.\"fid\" != NEW.\"fid\" AND (NEW.\"geom\" NOTNULL AND NOT ST_IsEmpty(NEW.\"geom\")) BEGIN DELETE FROM {r} WHERE id = OLD.\"fid\"; INSERT OR REPLACE INTO {r} {unos}; END;
         CREATE TRIGGER {} AFTER UPDATE OF \"geom\" ON {t} WHEN OLD.\"fid\" = NEW.\"fid\" AND (NEW.\"geom\" NOTNULL AND NOT ST_IsEmpty(NEW.\"geom\")) AND (OLD.\"geom\" NOTNULL AND NOT ST_IsEmpty(OLD.\"geom\")) BEGIN UPDATE {r} SET minx = ST_MinX(NEW.\"geom\"), maxx = ST_MaxX(NEW.\"geom\"),miny = ST_MinY(NEW.\"geom\"), maxy = ST_MaxY(NEW.\"geom\") WHERE id = NEW.\"fid\";END;
         CREATE TRIGGER {} AFTER UPDATE OF \"geom\" ON {t} WHEN OLD.\"fid\" = NEW.\"fid\" AND (NEW.\"geom\" NOTNULL AND NOT ST_IsEmpty(NEW.\"geom\")) AND (OLD.\"geom\" ISNULL OR ST_IsEmpty(OLD.\"geom\")) BEGIN INSERT INTO {r} {unos}; END;
         CREATE TRIGGER {} AFTER DELETE ON {t} WHEN old.\"geom\" NOT NULL BEGIN DELETE FROM {r} WHERE id = OLD.\"fid\"; END;",
        navodnici(&format!("rtree_{tablica}_geom_insert")),
        navodnici(&format!("rtree_{tablica}_geom_update2")),
        navodnici(&format!("rtree_{tablica}_geom_update4")),
        navodnici(&format!("rtree_{tablica}_geom_update5")),
        navodnici(&format!("rtree_{tablica}_geom_update6")),
        navodnici(&format!("rtree_{tablica}_geom_update7")),
        navodnici(&format!("rtree_{tablica}_geom_delete")),
    ))?;
    Ok(())
}

/// Dodaje objekt i osvježava obuhvat sloja. Vraća `fid`.
pub fn dodaj_objekt(c: &Connection, tablica: &str, g: &Geometry<f64>, stupci: &[(&str, Value)]) -> Rezultat<i64> {
    let srs: i32 = c.query_row("SELECT srs_id FROM gpkg_geometry_columns WHERE table_name = ?1", [tablica], |r| r.get(0))?;
    let stupac: String = c.query_row("SELECT column_name FROM gpkg_geometry_columns WHERE table_name = ?1", [tablica], |r| r.get(0))?;
    let mut imena = vec![navodnici(&stupac)];
    imena.extend(stupci.iter().map(|(i, _)| navodnici(i)));
    let upitnici = vec!["?"; imena.len()].join(",");
    let mut vrijednosti = vec![Value::Blob(u_blob(g, srs)?)];
    vrijednosti.extend(stupci.iter().map(|(_, v)| v.clone()));
    c.execute(
        &format!("INSERT INTO {} ({}) VALUES ({upitnici})", navodnici(tablica), imena.join(",")),
        rusqlite::params_from_iter(vrijednosti),
    )?;
    let fid = c.last_insert_rowid();
    prosiri_obuhvat(c, tablica, g)?;
    Ok(fid)
}

fn prosiri_obuhvat(c: &Connection, tablica: &str, g: &Geometry<f64>) -> Rezultat<()> {
    let Some(o) = omotnica_geometrije(g) else { return Ok(()) };
    c.execute(
        "UPDATE gpkg_contents SET
           min_x = min(coalesce(min_x, ?2), ?2), min_y = min(coalesce(min_y, ?3), ?3),
           max_x = max(coalesce(max_x, ?4), ?4), max_y = max(coalesce(max_y, ?5), ?5),
           last_change = strftime('%Y-%m-%dT%H:%M:%fZ','now')
         WHERE table_name = ?1",
        params![tablica, o.min().x, o.min().y, o.max().x, o.max().y],
    )?;
    Ok(())
}

pub fn spremi_stil(c: &Connection, tablica: &str, naziv: &str, qml: &str) -> Rezultat<()> {
    c.execute(
        "INSERT INTO layer_styles (f_table_catalog, f_table_schema, f_table_name, f_geometry_column, styleName, styleQML, styleSLD, useAsDefault, description, owner, ui)
         VALUES ('', '', ?1, 'geom', ?2, ?3, '', 1, '', '', '')",
        params![tablica, naziv, qml],
    )?;
    Ok(())
}

fn u_json(v: ValueRef<'_>) -> Json {
    match v {
        ValueRef::Null => Json::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(r) => json!(r),
        ValueRef::Text(t) => json!(String::from_utf8_lossy(t)),
        ValueRef::Blob(b) => json!(format!("<blob {}>", b.len())),
    }
}

/// Sve što program vidi u datoteci, kao JSON za provjeru izvana.
pub fn citaj_sve(c: &Connection) -> Rezultat<Json> {
    let mut slojevi = Map::new();
    let mut upit = c.prepare("SELECT g.table_name, g.column_name, g.geometry_type_name, g.srs_id, g.z, g.m, k.min_x, k.min_y, k.max_x, k.max_y FROM gpkg_geometry_columns g JOIN gpkg_contents k USING (table_name) ORDER BY 1")?;
    let redovi: Vec<(String, String, String, i64, i64, i64, Option<f64>, Option<f64>, Option<f64>, Option<f64>)> = upit
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?, r.get(7)?, r.get(8)?, r.get(9)?)))?
        .collect::<Result<_, _>>()?;
    for (tablica, stupac, vrsta, srs, z, m, a, b, d, e) in redovi {
        let objekti = citaj_objekte(c, &tablica, &stupac)?;
        slojevi.insert(tablica.clone(), json!({
            "vrsta": vrsta, "srs": srs, "z": z, "m": m, "obuhvat": [a, b, d, e],
            "stupci": stupci(c, &tablica)?, "objekti": objekti, "stil": crate::stil::procitaj_zadani(c, &tablica)?,
        }));
    }
    let mut tablice = Vec::new();
    let mut u = c.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'rtree_%' AND (name NOT LIKE 'gpkg_%' OR name LIKE 'gpkg_ts_%') AND name NOT LIKE 'sqlite_%' ORDER BY 1")?;
    for t in u.query_map([], |r| r.get::<_, String>(0))? {
        tablice.push(t?);
    }
    let dnevnik: String = c.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
    Ok(json!({ "slojevi": slojevi, "tablice": tablice, "dnevnik": dnevnik }))
}

fn stupci(c: &Connection, tablica: &str) -> Rezultat<Vec<Json>> {
    let mut u = c.prepare(&format!("PRAGMA table_info({})", navodnici(tablica)))?;
    let r = u.query_map([], |r| Ok(json!([r.get::<_, String>(1)?, r.get::<_, String>(2)?])))?;
    Ok(r.collect::<Result<_, _>>()?)
}

fn citaj_objekte(c: &Connection, tablica: &str, stupac: &str) -> Rezultat<Vec<Json>> {
    let mut u = c.prepare(&format!("SELECT * FROM {} ORDER BY 1", navodnici(tablica)))?;
    let imena: Vec<String> = u.column_names().map(|s| s.to_string()).collect();
    let mut r = u.query([])?;
    let mut svi = Vec::new();
    while let Some(red) = r.next()? {
        let mut o = Map::new();
        for (i, ime) in imena.iter().enumerate() {
            let v = red.get_ref(i)?;
            if ime == stupac {
                o.insert("_geom".into(), opis_geometrije(v));
            } else {
                o.insert(ime.clone(), u_json(v));
            }
        }
        svi.push(Json::Object(o));
    }
    Ok(svi)
}

/// Čitanje geometrije ne smije srušiti čitanje sloja, pa se greška zapisuje uz objekt.
fn opis_geometrije(v: ValueRef<'_>) -> Json {
    let ValueRef::Blob(b) = v else { return Json::Null };
    let tekst = GpkgWkb(b).to_wkt().map_err(|e| e.to_string());
    let tekst_3d = GpkgWkb(b).to_wkt_with_opts(WktDialect::Wkt, CoordDimensions::xyz(), None).ok();
    let geo = GpkgWkb(b).to_geo().map_err(|e| e.to_string());
    let omot = omotnica(b).map(|o| [o.min().x, o.min().y, o.max().x, o.max().y]);
    json!({ "wkt": tekst.as_ref().ok(), "wkt_3d": tekst_3d, "greska_wkt": tekst.as_ref().err(), "geo": geo.as_ref().map(|g| format!("{g:?}")).ok(), "greska_geo": geo.as_ref().err(), "prazna": je_prazna(b), "omotnica": omot })
}

pub fn pravokutnik(x0: f64, y0: f64, x1: f64, y1: f64) -> LineString<f64> {
    LineString::from(vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1), (x0, y0)])
}

pub fn lik(vanjski: LineString<f64>, rupe: Vec<LineString<f64>>) -> Geometry<f64> {
    Geometry::Polygon(Polygon::new(vanjski, rupe))
}

#[cfg(test)]
mod testovi {
    use super::*;
    use geo_types::Point;

    #[test]
    fn tocka_ima_omotnicu_iako_je_zapisana_bez_nje() {
        let b = u_blob(&Geometry::Point(Point::new(18.7, 45.5)), 4326).unwrap();
        assert_eq!(b[3] & 0b0000_1110, 0, "točka se piše bez omotnice");
        let o = omotnica(&b).unwrap();
        assert_eq!((o.min().x, o.min().y, o.max().x, o.max().y), (18.7, 45.5, 18.7, 45.5));
    }

    #[test]
    fn oznaka_prazne_geometrije_se_cita() {
        let mut b = u_blob(&Geometry::Point(Point::new(1.0, 2.0)), 4326).unwrap();
        assert!(!je_prazna(&b));
        b[3] |= 0b0001_0000;
        assert!(je_prazna(&b));
        assert!(omotnica(&b).is_none());
    }

    #[test]
    fn bez_nasih_funkcija_okidaci_ne_rade() {
        let c = Connection::open_in_memory().unwrap();
        dodaj_funkcije(&c).unwrap();
        stvori(&c).unwrap();
        dodaj_sloj(&c, "s", "POINT", 4326, &[("naziv", "TEXT")]).unwrap();
        let g = Geometry::Point(Point::new(18.0, 45.0));
        assert_eq!(dodaj_objekt(&c, "s", &g, &[]).unwrap(), 1);
        let u_indeksu: i64 = c.query_row("SELECT count(*) FROM \"rtree_s_geom\"", [], |r| r.get(0)).unwrap();
        assert_eq!(u_indeksu, 1);

        // Ista datoteka, novi spoj bez funkcija: tako bi prošao program koji ih ne zna.
        let put = std::env::temp_dir().join(format!("proba-gpkg-{}.gpkg", std::process::id()));
        let d = Connection::open(&put).unwrap();
        dodaj_funkcije(&d).unwrap();
        stvori(&d).unwrap();
        dodaj_sloj(&d, "s", "POINT", 4326, &[("naziv", "TEXT")]).unwrap();
        drop(d);
        let golo = Connection::open(&put).unwrap();
        let greska = golo.execute("INSERT INTO s (geom) VALUES (?1)", [u_blob(&g, 4326).unwrap()]).unwrap_err();
        assert!(greska.to_string().contains("ST_IsEmpty"), "{greska}");
        drop(golo);
        let _ = std::fs::remove_file(put);
    }
}
