use crate::{Greska, geometrija, shema, stil};
use koordinate::Sustav;
use rusqlite::{Connection, OptionalExtension, types::Value};
use slojevi::*;
use std::collections::BTreeMap;

#[derive(Clone)]
pub(crate) struct SlojBaze {
    pub postoji: bool,
    pub sql: Option<String>,
    pub indeksi: Vec<String>,
    pub okidaci: Vec<String>,
    pub izvorni_stil: Option<Stil>,
    pub id: IdSloja,
    pub tablica: String,
    pub geom: String,
    pub fid: String,
    pub srs: i32,
    pub sustav: Sustav,
    pub vrsta: VrstaGeometrije,
    pub naziv: String,
}
#[derive(Clone)]
pub(crate) struct Cvor {
    pub id: u64,
    pub vrsta: String,
    pub roditelj: Option<u64>,
    pub naziv: String,
    pub tablica: Option<String>,
    pub stil: Option<String>,
    pub stanje: i64,
}
fn roditelj(r: &rusqlite::Row<'_>) -> rusqlite::Result<Option<u64>> {
    r.get::<_, Option<i64>>(2)?
        .map(|v| u64::try_from(v).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(2, v)))
        .transpose()
}
fn cvor_osnova(r: &rusqlite::Row<'_>) -> rusqlite::Result<Cvor> {
    Ok(Cvor {
        id: sql_id(r, 0)?,
        vrsta: r.get(1)?,
        roditelj: roditelj(r)?,
        naziv: r.get(4)?,
        tablica: None,
        stil: None,
        stanje: 0,
    })
}
fn cvor(r: &rusqlite::Row<'_>) -> rusqlite::Result<Cvor> {
    let mut c = cvor_osnova(r)?;
    c.tablica = r.get(5)?;
    c.stil = r.get(6)?;
    c.stanje = r.get(7)?;
    Ok(c)
}
pub(crate) fn cvorovi(c: &Connection) -> Result<Vec<Cvor>, Greska> {
    if !shema::postoji(c, "gpkg_ts_stablo")? {
        return Ok(Vec::new());
    }
    let mut u=c.prepare("SELECT id,vrsta,roditelj,polozaj,naziv,tablica,stil,stanje FROM gpkg_ts_stablo ORDER BY polozaj,id")?;
    u.query_map([], cvor)
        .and_then(Iterator::collect)
        .map_err(Into::into)
}
pub(crate) fn slojevi(
    c: &Connection,
    n: &[Cvor],
    sljedeci: &mut u64,
) -> Result<Vec<SlojBaze>, Greska> {
    let mut u=c.prepare("SELECT g.table_name,g.column_name,g.geometry_type_name,g.srs_id,coalesce(k.identifier,g.table_name),s.organization,s.organization_coordsys_id,s.definition FROM gpkg_geometry_columns g JOIN gpkg_contents k USING(table_name) LEFT JOIN gpkg_spatial_ref_sys s ON s.srs_id=g.srs_id ORDER BY g.table_name")?;
    let mut redci = u.raw_query();
    let mut svi = Vec::new();
    while let Some(r) = redci.next()? {
        svi.push(sloj(c, r, n, sljedeci)?);
    }
    Ok(svi)
}
fn kljuc_sloja(c: &Connection, t: &str) -> Result<String, Greska> {
    shema::stupci(c, t)?
        .into_iter()
        .find(|(_, pk)| *pk)
        .map(|(n, _)| n)
        .ok_or_else(|| "Sloj nema primarni ključ.".into())
}
fn vrsta_reda(r: &rusqlite::Row<'_>) -> Result<VrstaGeometrije, Greska> {
    vrsta(&r.get::<_, String>(2)?)
}
fn sloj_osnova(r: &rusqlite::Row<'_>) -> Result<SlojBaze, Greska> {
    Ok(SlojBaze {
        postoji: true,
        sql: None,
        indeksi: Vec::new(),
        okidaci: Vec::new(),
        izvorni_stil: None,
        id: IdSloja(0),
        tablica: r.get(0)?,
        geom: r.get(1)?,
        vrsta: vrsta_reda(r)?,
        srs: r.get(3)?,
        naziv: r.get(4)?,
        sustav: Sustav::Wgs84,
        fid: String::new(),
    })
}
fn sloj(
    c: &Connection,
    r: &rusqlite::Row<'_>,
    n: &[Cvor],
    sljedeci: &mut u64,
) -> Result<SlojBaze, Greska> {
    let mut b = sloj_osnova(r)?;
    b.id = IdSloja(
        n.iter()
            .find(|n| n.tablica.as_ref() == Some(&b.tablica))
            .map(|n| n.id)
            .map_or_else(|| novi_id(sljedeci), Ok)?,
    );
    b.sustav = sustav(r)?;
    b.fid = kljuc_sloja(c, &b.tablica)?;
    Ok(b)
}
fn sustav(r: &rusqlite::Row<'_>) -> Result<Sustav, Greska> {
    Sustav::iz_retka_geopaketa(&r.get::<_, String>(5)?, r.get(6)?, &r.get::<_, String>(7)?)
        .ok_or_else(|| "Koordinatni sustav nije poznat. Čitanje je prekinuto.".into())
}
fn vrsta(s: &str) -> Result<VrstaGeometrije, Greska> {
    match s {
        "POINT" => Ok(VrstaGeometrije::Tocka),
        s if [
            "LINESTRING",
            "COMPOUNDCURVE",
            "CIRCULARSTRING",
            "CURVE",
            "MULTICURVE",
        ]
        .contains(&s) =>
        {
            Ok(VrstaGeometrije::Linija)
        }
        s if ["POLYGON", "CURVEPOLYGON", "SURFACE", "MULTISURFACE"].contains(&s) => {
            Ok(VrstaGeometrije::Poligon)
        }
        _ => Err("Vrsta sloja nije podržana.".into()),
    }
}
fn novi_id(id: &mut u64) -> Result<u64, Greska> {
    *id = id
        .checked_add(1)
        .filter(|v| *v <= i64::MAX as u64)
        .ok_or("Nema slobodnog broja objekta.")?;
    Ok(*id)
}
fn redak_id(r: &rusqlite::Row<'_>) -> rusqlite::Result<((String, i64), u64)> {
    Ok(((r.get(0)?, r.get(1)?), sql_id(r, 2)?))
}
pub(crate) fn ids(c: &Connection) -> Result<BTreeMap<(String, i64), u64>, Greska> {
    if !shema::postoji(c, "gpkg_ts_objekti")? {
        return Ok(BTreeMap::new());
    }
    let mut u = c.prepare("SELECT tablica,fid,id FROM gpkg_ts_objekti")?;
    u.query_map([], redak_id)
        .and_then(Iterator::collect)
        .map_err(Into::into)
}
/// Sloj ili objekt bez zapisanog broja dobije prvi slobodan broj veći od `najmanji`.
pub(crate) fn projekt(c: &Connection, najmanji: u64) -> Result<(Projekt, Vec<String>), Greska> {
    let n = cvorovi(c)?;
    let ids = ids(c)?;
    let mut sljedeci = n
        .iter()
        .map(|n| n.id)
        .chain(ids.values().copied())
        .fold(najmanji, u64::max);
    let info = slojevi(c, &n, &mut sljedeci)?;
    let mut slojevi = BTreeMap::new();
    let mut zapisi = Zapisi::default();
    let mut upozorenja = Vec::new();
    for b in info {
        let (s, z) = ucitaj_sloj(c, &b, &n, &ids, &mut sljedeci, &mut upozorenja)?;
        slojevi.insert(b.id, s);
        zapisi.slojevi.insert(b.id, z);
    }
    zapisi.najveci_id = sljedeci;
    dovrsi_projekt(&n, slojevi, zapisi, upozorenja)
}
fn dovrsi_projekt(
    n: &[Cvor],
    slojevi: BTreeMap<IdSloja, Sloj>,
    zapisi: Zapisi,
    upozorenja: Vec<String>,
) -> Result<(Projekt, Vec<String>), Greska> {
    provjeri_brojeve(&slojevi)?;
    let (korijen, mape) = stablo(n, &slojevi)?;
    Ok((
        Projekt::iz_zapisa(korijen, mape, slojevi, zapisi)?,
        upozorenja,
    ))
}

fn stanje(i: i64) -> Result<StanjeSloja, Greska> {
    match i {
        0 => Ok(StanjeSloja::Uredivo),
        1 => Ok(StanjeSloja::SamoCitanje),
        2 => Ok(StanjeSloja::CekaSlanje),
        3 => Ok(StanjeSloja::USukobu),
        _ => Err("Stanje sloja nije poznato.".into()),
    }
}
fn ucitaj_sloj(
    c: &Connection,
    b: &SlojBaze,
    n: &[Cvor],
    ids: &BTreeMap<(String, i64), u64>,
    sljedeci: &mut u64,
    upozorenja: &mut Vec<String>,
) -> Result<(Sloj, IzvorSloja), Greska> {
    let nod = n.iter().find(|n| n.id == b.id.0);
    let zadani = nod
        .and_then(|n| n.stil.as_deref())
        .map(stil::iz_json)
        .transpose()?
        .unwrap_or_default();
    let stil = stil_sloja(c, b, zadani, upozorenja)?;
    let (objekti, zapisi) = objekti(c, b, ids, sljedeci, upozorenja)?;
    // Zapis se čuva za svaki redak tablice, a objekt samo za onaj koji program zna prikazati.
    let nepodrzani = zapisi.len() - objekti.len();
    let (sql, stupci, indeksi, okidaci) = opis_sheme(c, b)?;
    let s = Sloj {
        naziv: nod.map_or_else(|| b.naziv.clone(), |n| n.naziv.clone()),
        roditelj: nod.and_then(|n| n.roditelj).map(IdMape),
        vrsta: b.vrsta,
        stanje: stanje(nod.map_or(0, |n| n.stanje))?,
        stil,
        objekti,
    };
    Ok((
        s,
        IzvorSloja {
            srs_id: b.srs,
            sql,
            stupci,
            indeksi,
            okidaci,
            stil,
            sustav: b.sustav,
            tablica: b.tablica.clone(),
            fid_stupac: b.fid.clone(),
            geometrija_stupac: b.geom.clone(),
            objekti: zapisi,
            nepodrzani,
        },
    ))
}
fn stil_sloja(
    c: &Connection,
    b: &SlojBaze,
    zadani: Stil,
    u: &mut Vec<String>,
) -> Result<Stil, Greska> {
    if !shema::postoji(c, "layer_styles")? {
        return Ok(zadani);
    }
    let q:Option<String>=c.query_row("SELECT styleQML FROM layer_styles WHERE f_table_name=?1 AND useAsDefault=1 ORDER BY id DESC LIMIT 1",[&b.tablica],|r|r.get(0)).optional()?;
    match q.map(|q| stil::procitaj(&q, b.vrsta, zadani)).transpose() {
        Ok(s) => Ok(s.unwrap_or(zadani)),
        Err(e) => {
            u.push(format!("Sloj {}: {e}", b.tablica));
            Ok(zadani)
        }
    }
}
type Objekti = (
    BTreeMap<IdObjekta, Objekt>,
    BTreeMap<IdObjekta, ZapisObjekta>,
);
fn upit_objekata<'a>(c: &'a Connection, b: &SlojBaze) -> Result<rusqlite::Statement<'a>, Greska> {
    Ok(c.prepare(&format!(
        "SELECT * FROM {} ORDER BY {}",
        shema::navodnici(&b.tablica),
        shema::navodnici(&b.fid)
    ))?)
}
fn objekti(
    c: &Connection,
    b: &SlojBaze,
    ids: &BTreeMap<(String, i64), u64>,
    sljedeci: &mut u64,
    u: &mut Vec<String>,
) -> Result<Objekti, Greska> {
    let mut upit = upit_objekata(c, b)?;
    let imena: Vec<_> = upit.column_names().map(|s| s.to_string()).collect();
    let mut r = upit.raw_query();
    let mut objekti = BTreeMap::new();
    let mut zapisi = BTreeMap::new();
    while let Some(r) = r.next()? {
        let (v, id, fid) = podaci_objekta(r, &imena, b, ids, sljedeci)?;
        let nepoznati = nepoznati(&v, b);
        zapisi.insert(
            id,
            ZapisObjekta {
                fid,
                nepoznati,
                geometrija: izvorni_blob(v.get(&b.geom)),
            },
        );
        match ucitaj_objekt(c, r, b, &v, id, fid) {
            Ok(o) => {
                objekti.insert(id, o);
            }
            Err(e) => u.push(format!("Sloj {}, fid {fid}: {e}", b.tablica)),
        }
    }
    Ok((objekti, zapisi))
}
fn ucitaj_objekt(
    c: &Connection,
    r: &rusqlite::Row<'_>,
    b: &SlojBaze,
    v: &BTreeMap<String, Value>,
    id: IdObjekta,
    fid: i64,
) -> Result<Objekt, Greska> {
    let mut o = objekt(r, b, v, id)?;
    vrati_prstenove(c, b, fid, &mut o.geometrija)?;
    Ok(o)
}
fn podaci_objekta(
    r: &rusqlite::Row<'_>,
    imena: &[String],
    b: &SlojBaze,
    ids: &BTreeMap<(String, i64), u64>,
    sljedeci: &mut u64,
) -> Result<(BTreeMap<String, Value>, IdObjekta, i64), Greska> {
    let v = vrijednosti(r, imena)?;
    let (id, fid) = objekt_id(r, b, ids, sljedeci)?;
    Ok((v, id, fid))
}
fn objekt_id(
    r: &rusqlite::Row<'_>,
    b: &SlojBaze,
    ids: &BTreeMap<(String, i64), u64>,
    sljedeci: &mut u64,
) -> Result<(IdObjekta, i64), Greska> {
    let fid = r.get::<_, i64>(b.fid.as_str())?;
    let id = IdObjekta(
        ids.get(&(b.tablica.clone(), fid))
            .copied()
            .map_or_else(|| novi_id(sljedeci), Ok)?,
    );
    Ok((id, fid))
}
fn nepoznati(v: &BTreeMap<String, Value>, b: &SlojBaze) -> BTreeMap<String, Vrijednost> {
    v.iter()
        .filter(|(n, _)| !stil::STUPCI.contains(&n.as_str()) && *n != &b.fid && *n != &b.geom)
        .map(|(n, v)| (n.clone(), vrijednost(v.clone())))
        .collect()
}
fn vrijednosti(r: &rusqlite::Row<'_>, imena: &[String]) -> Result<BTreeMap<String, Value>, Greska> {
    imena
        .iter()
        .map(|n| Ok((n.clone(), r.get(n.as_str())?)))
        .collect()
}
fn vrijednost(v: Value) -> Vrijednost {
    match v {
        Value::Null => Vrijednost::Prazno,
        Value::Integer(i) => Vrijednost::Cijeli(i),
        Value::Real(f) => Vrijednost::Broj(f),
        Value::Text(s) => Vrijednost::Tekst(s),
        Value::Blob(b) => Vrijednost::Bajtovi(b),
    }
}
fn tekst(v: Option<&Value>) -> String {
    match v {
        Some(Value::Text(s)) => s.clone(),
        _ => String::new(),
    }
}
fn objekt(
    r: &rusqlite::Row<'_>,
    b: &SlojBaze,
    v: &BTreeMap<String, Value>,
    id: IdObjekta,
) -> Result<Objekt, Greska> {
    let blob: Option<Vec<u8>> = r.get(b.geom.as_str())?;
    let geometrija = geometrija::procitaj(
        blob.as_deref()
            .ok_or("Objekt ima NULL geometriju; zapis ostaje sačuvan.")?,
        b.sustav,
    )?;
    Ok(Objekt {
        id,
        naziv: tekst(v.get("naziv")),
        opis: tekst(v.get("opis")),
        geometrija,
        stil: stil::objekt(v)?,
    })
}
fn stablo(
    n: &[Cvor],
    s: &BTreeMap<IdSloja, Sloj>,
) -> Result<(Vec<Stavka>, BTreeMap<IdMape, Mapa>), Greska> {
    let mut m: BTreeMap<_, _> = n
        .iter()
        .filter(|n| n.vrsta == "mapa")
        .map(|n| {
            (
                IdMape(n.id),
                Mapa {
                    naziv: n.naziv.clone(),
                    roditelj: n.roditelj.map(IdMape),
                    djeca: Vec::new(),
                },
            )
        })
        .collect();
    let mut korijen = Vec::new();
    for n in n {
        let Some(stavka) = stavka(n, s)? else {
            continue;
        };
        dodaj(&mut korijen, &mut m, n.roditelj, stavka)?;
    }
    for id in s.keys().filter(|id| !n.iter().any(|n| n.id == id.0)) {
        korijen.push(Stavka::Sloj(*id));
    }
    Ok((korijen, m))
}
fn stavka(n: &Cvor, s: &BTreeMap<IdSloja, Sloj>) -> Result<Option<Stavka>, Greska> {
    match n.vrsta.as_str() {
        "mapa" => Ok(Some(Stavka::Mapa(IdMape(n.id)))),
        "sloj" => Ok(s
            .contains_key(&IdSloja(n.id))
            .then_some(Stavka::Sloj(IdSloja(n.id)))),
        _ => Err("Vrsta stavke stabla nije poznata.".into()),
    }
}
fn dodaj(
    k: &mut Vec<Stavka>,
    m: &mut BTreeMap<IdMape, Mapa>,
    roditelj: Option<u64>,
    s: Stavka,
) -> Result<(), Greska> {
    match roditelj {
        None => k.push(s),
        Some(id) => m
            .get_mut(&IdMape(id))
            .ok_or("Roditeljska mapa ne postoji.")?
            .djeca
            .push(s),
    };
    Ok(())
}

fn sql_id(r: &rusqlite::Row<'_>, i: usize) -> rusqlite::Result<u64> {
    let v = r.get::<_, i64>(i)?;
    v.try_into()
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(i, v))
}

#[cfg(test)]
mod testovi {
    use super::*;
    fn baza() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        geometrija::funkcije(&c).unwrap();
        c
    }
    fn snimka() -> serde_json::Value {
        serde_json::from_str(include_str!("../tests/podaci/qgis.json")).unwrap()
    }
    fn hex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }
    #[test]
    fn snimljeni_qgis_u_metrima_i_nepoznati_stupci() {
        let c = baza();
        let q = snimka();
        let s = q["srs"].as_array().unwrap();
        c.execute(
            "INSERT INTO gpkg_spatial_ref_sys VALUES(?1,3765,'EPSG',3765,?2,NULL)",
            rusqlite::params![s[0].as_str().unwrap(), s[4].as_str().unwrap()],
        )
        .unwrap();
        shema::sloj(&c, "izvor", "QGIS", VrstaGeometrije::Linija).unwrap();
        c.execute_batch("UPDATE gpkg_geometry_columns SET srs_id=3765; UPDATE gpkg_contents SET srs_id=3765 WHERE table_name='izvor'; ALTER TABLE izvor ADD COLUMN x_blob BLOB; ALTER TABLE izvor ADD COLUMN x_int INTEGER; ALTER TABLE izvor ADD COLUMN x_real REAL; ALTER TABLE izvor ADD COLUMN x_text TEXT; ALTER TABLE izvor ADD COLUMN x_null TEXT;").unwrap();
        c.execute("INSERT INTO izvor(geom,naziv,x_blob,x_int,x_real,x_text) VALUES(?1,'QGIS',x'0011',8,1.25,'tekst')",[hex(q["geometrija_hex"].as_str().unwrap())]).unwrap();
        c.execute(
            "INSERT INTO layer_styles(f_table_name,styleQML,useAsDefault) VALUES('izvor',?1,1)",
            [q["qml"].as_str().unwrap()],
        )
        .unwrap();
        c.execute_batch("DROP TABLE gpkg_ts_objekti; DROP TABLE gpkg_ts_stablo;")
            .unwrap();
        let (p, u) = projekt(&c, 0).unwrap();
        assert!(u.is_empty(), "{u:?}");
        let (id, s) = p.slojevi().next().unwrap();
        assert_eq!(s.objekti.len(), 1);
        assert_eq!(p.zapisi().slojevi[&id].sustav, Sustav::Htrs96Tm);
        let o = s.objekti.values().next().unwrap();
        let Geometrija::Linija(v) = &o.geometrija else {
            panic!()
        };
        assert_eq!(v[0].visina, Some(42.0));
        assert!(v[0].duzina > 18.0 && v[0].duzina < 19.0);
        assert_eq!(p.zapisi().slojevi[&id].objekti[&o.id].nepoznati.len(), 5);
        c.execute_batch("UPDATE gpkg_spatial_ref_sys SET organization='NONE',definition='PROJCS[\"x\",AUTHORITY[\"EPSG\",\"3765\"]]' WHERE srs_id=3765;").unwrap();
        assert!(projekt(&c, 0).is_ok());
        c.execute_batch("UPDATE gpkg_spatial_ref_sys SET organization='NONE',definition='nepoznato' WHERE srs_id=3765;").unwrap();
        assert!(projekt(&c, 0).is_err());
    }
    #[test]
    fn prazno_i_krivulja_ostaju_u_bazi() {
        let c = baza();
        shema::sloj(&c, "test", "Test", VrstaGeometrije::Tocka).unwrap();
        let mut b = vec![b'G', b'P', 0, 17, 230, 16, 0, 0, 1, 1, 0, 0, 0];
        b.extend(f64::NAN.to_le_bytes());
        b.extend(f64::NAN.to_le_bytes());
        c.execute("INSERT INTO test(geom) VALUES(?1)", [b.clone()])
            .unwrap();
        c.execute("INSERT INTO test(geom) VALUES(NULL)", [])
            .unwrap();
        b[3] = 1;
        b[9] = 9;
        c.execute("INSERT INTO test(geom) VALUES(?1)", [b]).unwrap();
        let (p, u) = projekt(&c, 0).unwrap();
        assert_eq!(u.len(), 3);
        assert_eq!(p.slojevi().next().unwrap().1.objekti.len(), 0);
        assert_eq!(
            c.query_row("SELECT count(*) FROM test", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            3
        );
    }
    #[test]
    fn nevaljani_metapodaci_i_svi_tipovi() {
        for (s, v) in [
            ("POINT", VrstaGeometrije::Tocka),
            ("LINESTRING", VrstaGeometrije::Linija),
            ("COMPOUNDCURVE", VrstaGeometrije::Linija),
            ("CIRCULARSTRING", VrstaGeometrije::Linija),
            ("POLYGON", VrstaGeometrije::Poligon),
            ("CURVEPOLYGON", VrstaGeometrije::Poligon),
        ] {
            assert_eq!(vrsta(s).unwrap(), v);
        }
        assert!(vrsta("TIN").is_err());
        for i in 0..4 {
            assert!(stanje(i).is_ok());
        }
        assert!(stanje(4).is_err());
        let c = baza();
        assert!(projekt(&c, 0).is_ok());
        let mut id = u64::MAX;
        assert!(novi_id(&mut id).is_err());
        c.execute_batch("CREATE TABLE bez_pk(v TEXT);").unwrap();
        assert!(kljuc_sloja(&c, "bez_pk").is_err());
        c.execute_batch("INSERT INTO gpkg_ts_stablo VALUES(1,'krivo',NULL,0,'Krivo',NULL,NULL,0)")
            .unwrap();
        assert!(projekt(&c, 0).is_err());
        c.execute_batch("UPDATE gpkg_ts_stablo SET vrsta='mapa',roditelj=99")
            .unwrap();
        assert!(projekt(&c, 0).is_err());
        c.execute_batch("UPDATE gpkg_ts_stablo SET roditelj=-1")
            .unwrap();
        assert!(projekt(&c, 0).is_err());
        c.execute_batch("UPDATE gpkg_ts_stablo SET roditelj=NULL,id=-1")
            .unwrap();
        assert!(projekt(&c, 0).is_err());
    }
}

#[cfg(test)]
mod osteceni_redci {
    use super::*;
    #[test]
    fn pogresni_tipovi_metapodataka_se_ne_prihvacaju() {
        let c = Connection::open_in_memory().unwrap();
        let osnovni = vec![
            Value::Integer(1),
            Value::Text("sloj".into()),
            Value::Integer(2),
            Value::Integer(0),
            Value::Text("Naziv".into()),
            Value::Text("tablica".into()),
            Value::Text("[]".into()),
            Value::Integer(0),
        ];
        for i in [0, 1, 2, 4, 5, 6, 7] {
            let mut v = osnovni.clone();
            v[i] = Value::Blob(vec![7]);
            assert!(
                c.query_row(
                    "SELECT ?1,?2,?3,?4,?5,?6,?7,?8",
                    rusqlite::params_from_iter(v),
                    cvor
                )
                .is_err()
            );
        }
        for i in 0..3 {
            let mut v = vec![
                Value::Text("tablica".into()),
                Value::Integer(1),
                Value::Integer(2),
            ];
            v[i] = Value::Blob(vec![]);
            assert!(
                c.query_row("SELECT ?1,?2,?3", rusqlite::params_from_iter(v), redak_id)
                    .is_err()
            );
        }
        for i in [0, 1, 2, 3, 4] {
            let mut v = vec![
                Value::Text("tablica".into()),
                Value::Text("geom".into()),
                Value::Text("POINT".into()),
                Value::Integer(4326),
                Value::Text("Naziv".into()),
            ];
            v[i] = Value::Blob(vec![9]);
            let mut u = c.prepare("SELECT ?1,?2,?3,?4,?5").unwrap();
            let mut r = u.query(rusqlite::params_from_iter(v)).unwrap();
            assert!(sloj_osnova(r.next().unwrap().unwrap()).is_err());
        }
        let osnovni = vec![
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Text("EPSG".into()),
            Value::Integer(4326),
            Value::Text("".into()),
        ];
        for i in [5, 6, 7] {
            let mut v = osnovni.clone();
            v[i] = Value::Blob(vec![]);
            let mut u = c.prepare("SELECT ?1,?2,?3,?4,?5,?6,?7,?8").unwrap();
            let mut r = u.query(rusqlite::params_from_iter(v)).unwrap();
            assert!(sustav(r.next().unwrap().unwrap()).is_err());
        }
    }
}

fn ima_prstenove(c: &Connection) -> Result<bool, Greska> {
    Ok(shema::stupci(c, "gpkg_ts_objekti")?
        .iter()
        .any(|(n, _)| n == "otvoreni"))
}
fn vrati_prstenove(
    c: &Connection,
    b: &SlojBaze,
    fid: i64,
    g: &mut Geometrija,
) -> Result<(), Greska> {
    if !matches!(g, Geometrija::Poligon(_)) {
        return Ok(());
    }
    if !ima_prstenove(c)? {
        return Ok(());
    }

    let zastavice: Option<String> = c
        .query_row(
            "SELECT otvoreni FROM gpkg_ts_objekti WHERE tablica=?1 AND fid=?2",
            rusqlite::params![b.tablica, fid],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    zastavice
        .as_deref()
        .map(|s| geometrija::vrati_otvorene(g, s))
        .transpose()
        .map(|_| ())
}

#[cfg(test)]
mod rubni_slucajevi {
    use super::*;
    fn baza() -> (Connection, SlojBaze) {
        let c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        shema::sloj(&c, "t", "T", VrstaGeometrije::Tocka).unwrap();
        let b = slojevi(&c, &[], &mut 0).unwrap().pop().unwrap();
        (c, b)
    }
    #[test]
    fn nevaljani_objekti_i_metapodaci_se_prijavljuju() {
        let (c, b) = baza();
        let mut id = 0;
        let ids = BTreeMap::new();
        let mut stmt = c.prepare("SELECT 'tekst' AS fid,'tekst' AS geom").unwrap();
        let mut red = stmt.query([]).unwrap();
        let r = red.next().unwrap().unwrap();
        assert!(podaci_objekta(r, &["nema".into()], &b, &ids, &mut id).is_err());
        assert!(podaci_objekta(r, &["fid".into()], &b, &ids, &mut id).is_err());
        assert!(objekt(r, &b, &BTreeMap::new(), IdObjekta(1)).is_err());
        let mut stmt = c
            .prepare("SELECT 1 AS fid,CAST(x'80' AS TEXT) AS los")
            .unwrap();
        let mut red = stmt.query([]).unwrap();
        let r = red.next().unwrap().unwrap();
        assert!(vrijednosti(r, &["los".into()]).is_err());
        let mut id = i64::MAX as u64;
        assert!(objekt_id(r, &b, &ids, &mut id).is_err());
        c.execute_batch("DROP TABLE layer_styles").unwrap();
        let mut u = Vec::new();
        assert_eq!(
            stil_sloja(&c, &b, Stil::default(), &mut u).unwrap(),
            Stil::default()
        );
        c.execute_batch("CREATE TABLE layer_styles(id INTEGER PRIMARY KEY,f_table_name TEXT,styleQML TEXT,useAsDefault INTEGER); INSERT INTO layer_styles VALUES(1,'t','<qgis><renderer-v2 type=\"categorizedSymbol\"/></qgis>',1);").unwrap();
        assert!(stil_sloja(&c, &b, Stil::default(), &mut u).is_ok());
        assert_eq!(u.len(), 1);
        c.execute_batch("INSERT INTO gpkg_ts_stablo VALUES(1,'mapa',1,0,'Mapa',NULL,NULL,0)")
            .unwrap();
        assert!(projekt(&c, 0).is_err());
        c.execute_batch("DELETE FROM gpkg_ts_stablo; INSERT INTO gpkg_ts_stablo VALUES(9223372036854775807,'mapa',NULL,0,'M',NULL,NULL,0)").unwrap();
        assert!(projekt(&c, 0).is_err());
        c.execute_batch("DELETE FROM gpkg_ts_stablo; CREATE TABLE bez(fid TEXT); INSERT INTO gpkg_contents(table_name,data_type,srs_id) VALUES('bez','features',4326); INSERT INTO gpkg_geometry_columns VALUES('bez','geom','POINT',4326,0,0)").unwrap();
        assert!(projekt(&c, 0).is_err());
    }
    #[test]
    fn podatak_o_prstenovima_mora_biti_valjan() {
        let (c, b) = baza();
        let v = Vrh::novi(18., 45.);
        let mut g = Geometrija::Poligon(vec![vec![v, v, v, v]]);
        vrati_prstenove(&c, &b, 1, &mut g).unwrap();
        c.execute("INSERT INTO gpkg_ts_objekti VALUES('t',1,2,'[true]')", [])
            .unwrap();
        vrati_prstenove(&c, &b, 1, &mut g).unwrap();
        c.execute("UPDATE gpkg_ts_objekti SET otvoreni='krivo'", [])
            .unwrap();
        assert!(vrati_prstenove(&c, &b, 1, &mut g).is_err());
        c.execute_batch("ALTER TABLE gpkg_ts_objekti DROP COLUMN otvoreni")
            .unwrap();
        vrati_prstenove(&c, &b, 1, &mut g).unwrap();
        c.execute_batch("DROP TABLE gpkg_ts_objekti").unwrap();
        vrati_prstenove(&c, &b, 1, &mut g).unwrap();
    }
}

#[cfg(test)]
mod prekid_metapodataka {
    use super::*;
    #[test]
    fn losi_podaci_prstena_i_sql_kvar_se_propagiraju() {
        use rusqlite::hooks::{AuthContext, Authorization};
        let mut c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        shema::sloj(&c, "t", "T", VrstaGeometrije::Poligon).unwrap();
        let b = slojevi(&c, &[], &mut 0).unwrap().pop().unwrap();
        let v = Vrh::novi(18., 45.);
        let g = Geometrija::Poligon(vec![vec![v, v, v, v]]);
        let blob = geometrija::zapisi(&g, Sustav::Wgs84, 4326).unwrap();
        c.execute("INSERT INTO t(fid,geom) VALUES(1,?1)", [blob])
            .unwrap();
        c.execute("INSERT INTO gpkg_ts_objekti VALUES('t',1,2,'pogresno')", [])
            .unwrap();
        let mut stmt = c.prepare("SELECT * FROM t").unwrap();
        let mut rows = stmt.query([]).unwrap();
        let r = rows.next().unwrap().unwrap();
        let mapa = vrijednosti(r, &["fid".into(), "geom".into()]).unwrap();
        assert!(ucitaj_objekt(&c, r, &b, &mapa, IdObjekta(2), 1).is_err());
        drop(rows);
        drop(stmt);
        c.authorizer(Some(|_: AuthContext<'_>| Authorization::Deny))
            .unwrap();
        assert!(ima_prstenove(&c).is_err());
        let mut g = g;
        assert!(vrati_prstenove(&c, &b, 1, &mut g).is_err());
        c.authorizer(Some(|_: AuthContext<'_>| Authorization::Allow))
            .unwrap();
        c.execute_batch("ALTER TABLE gpkg_ts_objekti RENAME COLUMN tablica TO krivo")
            .unwrap();
        assert!(vrati_prstenove(&c, &b, 1, &mut g).is_err());
    }
}

fn izvorni_blob(v: Option<&Value>) -> Option<std::sync::Arc<[u8]>> {
    match v {
        Some(Value::Blob(b)) => Some(std::sync::Arc::from(b.as_slice())),
        _ => None,
    }
}
type OpisSheme = (String, Vec<String>, Vec<String>, Vec<String>);
fn opis_sheme(c: &Connection, b: &SlojBaze) -> Result<OpisSheme, Greska> {
    let sql = c.query_row(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name=?1",
        [&b.tablica],
        |r| r.get(0),
    )?;
    let stupci = shema::stupci(c, &b.tablica)?
        .into_iter()
        .map(|(n, _)| n)
        .collect();
    let (indeksi, okidaci) = dodatna_shema(c, b)?;
    Ok((sql, stupci, indeksi, okidaci))
}
fn dodatna_shema(c: &Connection, b: &SlojBaze) -> Result<(Vec<String>, Vec<String>), Greska> {
    let mut u=c.prepare("SELECT type,name,sql FROM sqlite_master WHERE tbl_name=?1 AND type IN('index','trigger') AND sql IS NOT NULL ORDER BY name")?;
    let svi: Vec<(String, String, String)> = u
        .query_map([&b.tablica], |r| r.try_into())
        .and_then(Iterator::collect)?;
    let prefix = format!("rtree_{}_{}_", b.tablica, b.geom);
    let mut indeksi = Vec::new();
    let mut okidaci = Vec::new();
    for (tip, naziv, sql) in svi {
        if tip == "index" {
            indeksi.push(sql);
        } else if !naziv.starts_with(&prefix) {
            okidaci.push(sql);
        }
    }
    Ok((indeksi, okidaci))
}

fn provjeri_brojeve(s: &BTreeMap<IdSloja, Sloj>) -> Result<(), Greska> {
    let mut ids = std::collections::BTreeSet::new();
    for s in s.values() {
        for id in s.objekti.keys() {
            if !ids.insert(*id) {
                return Err("Datoteka ima ponovljen unutarnji broj objekta.".into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod ugovor_izvora {
    use super::*;
    #[test]
    fn ponovljeni_brojevi_objekta_nisu_dva_razlicita_objekta() {
        let u = crate::kvarovi::model();
        let mut s: BTreeMap<_, _> = u
            .projekt()
            .slojevi()
            .map(|(id, s)| (id, s.clone()))
            .collect();
        let prvi = s.values().next().unwrap().objekti.clone();
        s.values_mut().last().unwrap().objekti = prvi;
        assert!(provjeri_brojeve(&s).is_err());
        assert!(dovrsi_projekt(&[], s, Zapisi::default(), vec![]).is_err());
    }
    #[test]
    fn shema_prenosi_korisnicki_okidac_a_ne_rt_stablo() {
        let mut c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        shema::sloj(&c, "t", "T", VrstaGeometrije::Tocka).unwrap();
        let b = slojevi(&c, &[], &mut 0).unwrap().pop().unwrap();
        c.execute_batch("CREATE INDEX ix ON t(naziv); CREATE TRIGGER rtree_t_geom_insert AFTER INSERT ON t BEGIN SELECT 1; END; CREATE TRIGGER korisnik AFTER UPDATE ON t BEGIN SELECT 2; END;").unwrap();
        let (i, o) = dodatna_shema(&c, &b).unwrap();
        assert_eq!(i.len(), 1);
        assert_eq!(o.len(), 1);
        assert!(o[0].contains("korisnik"));
        use rusqlite::hooks::{AuthContext, Authorization};
        c.authorizer(Some(|_: AuthContext<'_>| Authorization::Deny))
            .unwrap();
        assert!(dodatna_shema(&c, &b).is_err());
    }
}
