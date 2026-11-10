use crate::Greska;
use rusqlite::{Connection, params};
use slojevi::VrstaGeometrije;
pub fn navodnici(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
pub fn postoji(c: &Connection, ime: &str) -> Result<bool, Greska> {
    Ok(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        [ime],
        |r| r.get(0),
    )?)
}
pub fn stvori(c: &Connection) -> Result<(), Greska> {
    c.execute_batch(include_str!("shema.sql"))?;
    nase(c)
}
pub fn nase(c: &Connection) -> Result<(), Greska> {
    c.execute_batch("CREATE TABLE IF NOT EXISTS gpkg_extensions(table_name TEXT,column_name TEXT,extension_name TEXT NOT NULL,definition TEXT NOT NULL,scope TEXT NOT NULL,UNIQUE(table_name,column_name,extension_name));
    CREATE TABLE IF NOT EXISTS gpkg_ts_stablo(id INTEGER PRIMARY KEY,vrsta TEXT NOT NULL,roditelj INTEGER,polozaj INTEGER NOT NULL,naziv TEXT NOT NULL,tablica TEXT UNIQUE,stil TEXT,stanje INTEGER NOT NULL);
    CREATE TABLE IF NOT EXISTS gpkg_ts_objekti(tablica TEXT NOT NULL,fid INTEGER NOT NULL,id INTEGER NOT NULL,otvoreni TEXT,PRIMARY KEY(tablica,fid),UNIQUE(tablica,id));")?;
    dopuni_objekte(c)?;
    for t in ["gpkg_ts_stablo", "gpkg_ts_objekti"] {
        c.execute("INSERT OR IGNORE INTO gpkg_extensions VALUES(?1,NULL,'terrasimple_stablo','urn:terrasimple:stablo:1','read-write')",[t])?;
    }
    Ok(())
}
pub fn vrsta(v: VrstaGeometrije) -> &'static str {
    match v {
        VrstaGeometrije::Tocka => "POINT",
        VrstaGeometrije::Linija => "LINESTRING",
        VrstaGeometrije::Poligon => "POLYGON",
    }
}
#[cfg(test)]
pub fn sloj(c: &Connection, t: &str, n: &str, v: VrstaGeometrije) -> Result<(), Greska> {
    let b = crate::citanje::SlojBaze {
        id: slojevi::IdSloja(1),
        postoji: false,
        tablica: t.into(),
        naziv: n.into(),
        vrsta: v,
        geom: "geom".into(),
        fid: "fid".into(),
        srs: 4326,
        sustav: koordinate::Sustav::Wgs84,
        sql: None,
        indeksi: Vec::new(),
        okidaci: Vec::new(),
        izvorni_stil: None,
    };
    sloj_baze(c, &b)
}
pub fn sloj_baze(c: &Connection, b: &crate::citanje::SlojBaze) -> Result<(), Greska> {
    stvori_tablicu(c, b)?;
    c.execute("INSERT INTO gpkg_contents(table_name,data_type,identifier,srs_id) VALUES(?1,'features',(SELECT ?2 WHERE NOT EXISTS(SELECT 1 FROM gpkg_contents WHERE identifier=?2)),?3)",params![b.tablica,b.naziv,b.srs])?;
    c.execute(
        "INSERT INTO gpkg_geometry_columns VALUES(?1,?2,?3,?4,2,0)",
        params![b.tablica, b.geom, vrsta(b.vrsta), b.srs],
    )?;
    for sql in &b.indeksi {
        c.execute(sql, [])?;
    }
    Ok(())
}
fn stvori_tablicu(c: &Connection, b: &crate::citanje::SlojBaze) -> Result<(), Greska> {
    let nova = format!(
        "CREATE TABLE {}({} INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,{} {},naziv TEXT,opis TEXT,stil_boja TEXT,stil_debljina REAL,stil_ispuna TEXT,stil_znak TEXT,stil_velicina REAL,stil_crta TEXT,stil_uzorak TEXT)",
        navodnici(&b.tablica),
        navodnici(&b.fid),
        navodnici(&b.geom),
        vrsta(b.vrsta)
    );
    c.execute(b.sql.as_deref().unwrap_or(&nova), [])?;
    Ok(())
}
pub fn stupci(c: &Connection, t: &str) -> Result<Vec<(String, bool)>, Greska> {
    let mut u = c.prepare("SELECT name,pk>0 FROM pragma_table_info(?1)")?;
    u.query_map([t], |r| r.try_into())
        .and_then(Iterator::collect)
        .map_err(Into::into)
}

fn dopuni_objekte(c: &Connection) -> Result<(), Greska> {
    if !stupci(c, "gpkg_ts_objekti")?
        .iter()
        .any(|(n, _)| n == "otvoreni")
    {
        c.execute_batch("ALTER TABLE gpkg_ts_objekti ADD COLUMN otvoreni TEXT")?;
    }
    Ok(())
}

#[cfg(test)]
mod testovi {
    use super::*;
    #[test]
    fn migracija_nase_tablice_i_pogreske_sheme() {
        let c = Connection::open_in_memory().unwrap();
        stvori(&c).unwrap();
        c.execute_batch("ALTER TABLE gpkg_ts_objekti DROP COLUMN otvoreni")
            .unwrap();
        nase(&c).unwrap();
        assert!(
            stupci(&c, "gpkg_ts_objekti")
                .unwrap()
                .iter()
                .any(|(n, _)| n == "otvoreni")
        );
        sloj(&c, "t", "Test", VrstaGeometrije::Tocka).unwrap();
        assert!(sloj(&c, "t", "Test", VrstaGeometrije::Tocka).is_err());
        c.execute(
            "INSERT INTO gpkg_contents(table_name,data_type) VALUES('u','features')",
            [],
        )
        .unwrap();
        assert!(sloj(&c, "u", "Test", VrstaGeometrije::Linija).is_err());
        c.execute_batch("DROP TABLE gpkg_geometry_columns").unwrap();
        assert!(sloj(&c, "v", "Drugi", VrstaGeometrije::Poligon).is_err());
        assert_eq!(navodnici("a\"b"), "\"a\"\"b\"");
    }
}

#[cfg(test)]
mod odbijena_migracija {
    use super::*;
    #[test]
    fn neuspjela_dopuna_ne_znaci_prolaz() {
        let c = Connection::open_in_memory().unwrap();
        stvori(&c).unwrap();
        c.execute_batch("ALTER TABLE gpkg_ts_objekti DROP COLUMN otvoreni; PRAGMA query_only=ON")
            .unwrap();
        assert!(dopuni_objekte(&c).is_err());
    }
}
