use crate::{Greska, citanje, fid::Fidovi, geometrija, shema, stil};
use koordinate::Sustav;
use rusqlite::{Connection, TransactionBehavior, params, types::Value};
use slojevi::*;
use std::collections::{BTreeMap, BTreeSet};
struct Upis {
    sql: String,
    v: Vec<Value>,
}
struct Plan {
    novi: Vec<citanje::SlojBaze>,
    upisi: Vec<Upis>,
    metapodaci: Vec<Upis>,
    stablo: Vec<Upis>,
    visine: Vec<String>,
    promijenjeni: BTreeSet<IdSloja>,
    dodatni: Vec<(String, String)>,
}
fn broj(id: u64) -> Result<i64, Greska> {
    Ok(id.try_into()?)
}
fn tablica(p: &Projekt, id: IdSloja) -> String {
    p.zapisi()
        .slojevi
        .get(&id)
        .map_or_else(|| format!("sloj_{}", id.0), |s| s.tablica.clone())
}
fn izvor(b: &BTreeMap<IdSloja, citanje::SlojBaze>, p: &Projekt, s: IdSloja) -> citanje::SlojBaze {
    b.get(&s).cloned().unwrap_or_else(|| izvor_iz_modela(p, s))
}
fn izvor_iz_modela(p: &Projekt, s: IdSloja) -> citanje::SlojBaze {
    let iz = p.zapisi().slojevi.get(&s);
    citanje::SlojBaze {
        id: s,
        postoji: false,
        tablica: tablica(p, s),
        geom: iz.map_or("geom", |z| z.geometrija_stupac.as_str()).into(),
        fid: iz.map_or("fid", |z| z.fid_stupac.as_str()).into(),
        srs: iz.map_or(4326, |z| z.srs_id),
        sustav: iz.map_or(Sustav::Wgs84, |z| z.sustav),
        vrsta: p.sloj(s).map_or(VrstaGeometrije::Tocka, |s| s.vrsta),
        naziv: p.sloj(s).map(|s| s.naziv.clone()).unwrap_or_default(),
        sql: iz.map(|z| z.sql.clone()),
        indeksi: iz.map(|z| z.indeksi.clone()).unwrap_or_default(),
        okidaci: iz.map(|z| z.okidaci.clone()).unwrap_or_default(),
        izvorni_stil: iz.map(|z| z.stil),
    }
}

pub fn spremi(
    c: &mut Connection,
    prije: &Projekt,
    p: &Projekt,
    verzija: i64,
) -> Result<std::time::Duration, Greska> {
    let b = postojeci(c, p)?;
    let plan = potpun_plan(c, prije, p, &b)?;
    let tx = transakcija(c, verzija)?;
    let pocetak = std::time::Instant::now();
    primijeni(&tx, prije, p, &b, plan)?;
    tx.commit()?;
    Ok(pocetak.elapsed())
}
fn transakcija(c: &mut Connection, verzija: i64) -> Result<rusqlite::Transaction<'_>, Greska> {
    let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
    provjeri_verziju(&tx, verzija)?;
    Ok(tx)
}
fn potpun_plan(
    c: &Connection,
    prije: &Projekt,
    p: &Projekt,
    b: &BTreeMap<IdSloja, citanje::SlojBaze>,
) -> Result<Plan, Greska> {
    let zm = p.izmjene_od(prije);
    let fids = crate::fid::plan(c, prije, p, b, &zm)?;
    let mut plan = pripremi(c, prije, p, b, &fids)?;
    plan.dodatni = dodatni_stupci(c, prije, p)?;
    plan.visine = crate::dimenzije::pripremi(c, p, b, &zm)?;
    Ok(plan)
}
fn postojeci(c: &Connection, p: &Projekt) -> Result<BTreeMap<IdSloja, citanje::SlojBaze>, Greska> {
    let n = citanje::cvorovi(c)?;
    let mut id = 0;
    let mut b: BTreeMap<_, _> = citanje::slojevi(c, &n, &mut id)?
        .into_iter()
        .map(|mut b| {
            povezi_izvor(p, &mut b);
            (b.id, b)
        })
        .collect();
    for (id, _) in p.slojevi() {
        b.entry(id).or_insert_with(|| izvor_iz_modela(p, id));
    }
    Ok(b)
}

pub(crate) fn provjeri_verziju(c: &Connection, verzija: i64) -> Result<(), Greska> {
    let trenutna: i64 = c.query_row("PRAGMA data_version", [], |r| r.get(0))?;
    if trenutna != verzija {
        return Err(
            "Datoteku je promijenila druga aplikacija. Ponovno je učitajte prije spremanja.".into(),
        );
    }
    Ok(())
}
fn pripremi(
    c: &Connection,
    prije: &Projekt,
    p: &Projekt,
    b: &BTreeMap<IdSloja, citanje::SlojBaze>,
    fids: &Fidovi,
) -> Result<Plan, Greska> {
    let promjene = p.izmjene_od(prije);
    let upisi = upisi_objekata(prije, p, b, fids, &promjene)?;
    let novi = p
        .slojevi()
        .filter(|(s, _)| b.get(s).is_none_or(|b| !b.postoji))
        .map(|(s, _)| izvor(b, p, s))
        .collect();
    Ok(Plan {
        novi,
        upisi,
        dodatni: Vec::new(),
        metapodaci: metapodaci(prije, p, fids)?,
        stablo: upisi_stablo(c, prije, p)?,
        visine: Vec::new(),
        promijenjeni: p
            .slojevi()
            .filter(|(id, s)| prije.sloj(*id) != Some(*s))
            .map(|(id, _)| id)
            .collect(),
    })
}
fn upisi_objekata(
    prije: &Projekt,
    p: &Projekt,
    b: &BTreeMap<IdSloja, citanje::SlojBaze>,
    fids: &Fidovi,
    zm: &IzmjeneSpremanja,
) -> Result<Vec<Upis>, Greska> {
    let mut r = Vec::new();
    for (s, id) in zm.obrisi.iter().filter(|(s, _)| p.sloj(*s).is_some()) {
        r.push(brisanje(&izvor(b, prije, *s), fids.get(*s, *id)?));
    }
    for (s, id) in &zm.upisi {
        r.push(objekt_za_upis(
            p,
            prije,
            &izvor(b, p, *s),
            *id,
            fids.get(*s, *id)?,
        )?);
    }
    Ok(r)
}
fn objekt_za_upis(
    p: &Projekt,
    prije: &Projekt,
    b: &citanje::SlojBaze,
    id: IdObjekta,
    fid: i64,
) -> Result<Upis, Greska> {
    upis(
        p,
        prije,
        b,
        p.objekt(b.id, id)
            .ok_or("Objekt za spremanje ne postoji.")?,
        fid,
    )
}
fn brisanje(b: &citanje::SlojBaze, fid: i64) -> Upis {
    Upis {
        sql: format!(
            "DELETE FROM {} WHERE {}=?",
            shema::navodnici(&b.tablica),
            shema::navodnici(&b.fid)
        ),
        v: vec![Value::Integer(fid)],
    }
}
fn upis(
    p: &Projekt,
    prije: &Projekt,
    b: &citanje::SlojBaze,
    o: &Objekt,
    fid: i64,
) -> Result<Upis, Greska> {
    let staro = prije.objekt(b.id, o.id);
    let v = stil::vrijednosti(&o.naziv, &o.opis, o.stil);
    let stari = staro.map(|o| stil::vrijednosti(&o.naziv, &o.opis, o.stil));
    let mut polja: Vec<(String, Value)> = stil::STUPCI
        .iter()
        .zip(v)
        .enumerate()
        .filter(|(i, (_, v))| stari.as_ref().is_none_or(|s| &s[*i] != v))
        .map(|(_, (n, v))| (n.to_string(), v))
        .collect();
    if staro.is_none_or(|s| s.geometrija != o.geometrija) {
        polja.push((b.geom.clone(), Value::Blob(geometrija_za_upis(p, b, o)?)));
    }
    if staro.is_none() {
        pridruzi_nepoznate(&mut polja, b, p, o.id)?;
    }
    sql_upisa(&b.tablica, &b.fid, fid, polja, staro.is_some())
}
fn sql_upisa(
    t: &str,
    pk: &str,
    fid: i64,
    mut polja: Vec<(String, Value)>,
    postoji: bool,
) -> Result<Upis, Greska> {
    if polja.is_empty() {
        return Err("Izmjena nema vrijednosti za upis.".into());
    }
    let sql = if postoji {
        let postavi = polja
            .iter()
            .map(|(n, _)| format!("{}=?", shema::navodnici(n)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "UPDATE {} SET {postavi} WHERE {}=?",
            shema::navodnici(t),
            shema::navodnici(pk)
        )
    } else {
        polja.push((pk.into(), Value::Integer(fid)));
        let imena = polja
            .iter()
            .map(|(n, _)| shema::navodnici(n))
            .collect::<Vec<_>>()
            .join(",");
        let upitnici = vec!["?"; polja.len()].join(",");
        format!(
            "INSERT INTO {} ({imena}) VALUES({upitnici})",
            shema::navodnici(t)
        )
    };
    let mut v: Vec<_> = polja.into_iter().map(|(_, v)| v).collect();
    if postoji {
        v.push(Value::Integer(fid));
    }
    Ok(Upis { sql, v })
}
fn primijeni(
    c: &Connection,
    prije: &Projekt,
    p: &Projekt,
    b: &BTreeMap<IdSloja, citanje::SlojBaze>,
    plan: Plan,
) -> Result<(), Greska> {
    pripremi_tablice(c, prije, p, b, &plan.novi, &plan.promijenjeni)?;
    dodaj_stupce(c, plan.dodatni)?;
    crate::dimenzije::primijeni(c, &plan.visine)?;
    izvrsi_upise(
        c,
        plan.upisi
            .into_iter()
            .chain(plan.stablo)
            .chain(plan.metapodaci),
    )?;
    obnovi_okidace(c, &plan.novi)?;
    Ok(())
}
fn pripremi_tablice(
    c: &Connection,
    prije: &Projekt,
    p: &Projekt,
    b: &BTreeMap<IdSloja, citanje::SlojBaze>,
    novi: &[citanje::SlojBaze],
    promijenjeni: &BTreeSet<IdSloja>,
) -> Result<(), Greska> {
    shema::nase(c)?;
    stvori_slojeve(c, novi)?;
    obrisi_izbacene(c, prije, p, b)?;
    for (s, sloj) in p.slojevi().filter(|(id, _)| promijenjeni.contains(id)) {
        let mut iz = izvor(b, p, s);
        iz.izvorni_stil = p.zapisi().slojevi.get(&s).map(|s| s.stil);
        pripremi_sloj(c, &iz, sloj, prije.sloj(s))?;
    }
    Ok(())
}
fn stvori_slojeve(c: &Connection, novi: &[citanje::SlojBaze]) -> Result<(), Greska> {
    for b in novi {
        shema::sloj_baze(c, b)?;
    }
    Ok(())
}
fn obrisi_izbacene(
    c: &Connection,
    prije: &Projekt,
    p: &Projekt,
    b: &BTreeMap<IdSloja, citanje::SlojBaze>,
) -> Result<(), Greska> {
    for (s, _) in prije.slojevi().filter(|(id, _)| p.sloj(*id).is_none()) {
        let iz = izvor(b, prije, s);
        provjeri_uklanjanje(c, prije, &iz)?;
        obrisi_sloj(c, &iz)?;
    }
    Ok(())
}
fn tip_stupca(n: &str) -> &'static str {
    match n {
        "stil_debljina" | "stil_velicina" => "REAL",
        _ => "TEXT",
    }
}
fn pripremi_sloj(
    c: &Connection,
    b: &citanje::SlojBaze,
    s: &Sloj,
    prije: Option<&Sloj>,
) -> Result<(), Greska> {
    let stupci = shema::stupci(c, &b.tablica)?;
    for n in stil::STUPCI
        .iter()
        .filter(|n| !stupci.iter().any(|(s, _)| s == *n))
    {
        c.execute_batch(&format!(
            "ALTER TABLE {} ADD COLUMN {} {}",
            shema::navodnici(&b.tablica),
            shema::navodnici(n),
            tip_stupca(n)
        ))?;
    }
    if treba_stil(c, b, s, prije)? {
        spremi_stil(c, b, s)?;
    }
    Ok(())
}
fn spremi_stil(c: &Connection, b: &citanje::SlojBaze, s: &Sloj) -> Result<(), Greska> {
    c.execute_batch("CREATE TABLE IF NOT EXISTS layer_styles(id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,f_table_catalog TEXT,f_table_schema TEXT,f_table_name TEXT,f_geometry_column TEXT,styleName TEXT,styleQML TEXT,styleSLD TEXT,useAsDefault BOOLEAN,description TEXT,owner TEXT,ui TEXT,update_time DATETIME DEFAULT(strftime('%Y-%m-%dT%H:%M:%fZ','now')));")?;
    c.execute(
        "UPDATE layer_styles SET useAsDefault=0 WHERE f_table_name=?1",
        [&b.tablica],
    )?;
    c.execute("INSERT INTO layer_styles(f_table_catalog,f_table_schema,f_table_name,f_geometry_column,styleName,styleQML,styleSLD,useAsDefault,description,owner,ui) VALUES('','',?1,?2,'TerraSimple',?3,'',1,'','','')",params![b.tablica,b.geom,stil::qml(s.vrsta,s.stil)])?;
    Ok(())
}
fn obrisi_sloj(c: &Connection, b: &citanje::SlojBaze) -> Result<(), Greska> {
    ukloni_indeks(c, b)?;
    c.execute(
        "DELETE FROM gpkg_geometry_columns WHERE table_name=?1",
        [&b.tablica],
    )?;
    c.execute(
        "DELETE FROM gpkg_contents WHERE table_name=?1",
        [&b.tablica],
    )?;
    c.execute(
        "DELETE FROM gpkg_extensions WHERE table_name=?1",
        [&b.tablica],
    )?;
    c.execute_batch(&format!("DROP TABLE {};", shema::navodnici(&b.tablica)))?;
    Ok(())
}
fn roditelj_broj(id: Option<IdMape>) -> Result<Option<i64>, Greska> {
    id.map(|id| broj(id.0)).transpose()
}
fn vrijednost_id(id: u64) -> Result<Value, Greska> {
    broj(id).map(Value::Integer)
}
fn stablo(p: &Projekt) -> Result<Vec<Vec<Value>>, Greska> {
    let mut red: Vec<_> = p
        .korijen()
        .iter()
        .enumerate()
        .map(|(i, s)| (*s, None, i))
        .collect();
    let mut r = Vec::new();
    while let Some((s, roditelj, i)) = red.pop() {
        let (id, vrsta, naziv, tablica, stil, stanje) = match s {
            Stavka::Mapa(id) => {
                let m = p.mapa(id).ok_or("Mapa ne postoji.")?;
                red.extend(m.djeca.iter().enumerate().map(|(i, s)| (*s, Some(id), i)));
                (id.0, "mapa", m.naziv.clone(), None, None, 0)
            }
            Stavka::Sloj(id) => {
                let s = p.sloj(id).ok_or("Sloj ne postoji.")?;
                (
                    id.0,
                    "sloj",
                    s.naziv.clone(),
                    Some(tablica(p, id)),
                    Some(stil::json(s.stil)),
                    stanje(s.stanje),
                )
            }
        };
        r.push(redak_stabla(
            (id, vrsta, naziv, tablica, stil, stanje),
            roditelj,
            i,
        )?);
    }
    Ok(r)
}
type PodaciStavke = (
    u64,
    &'static str,
    String,
    Option<String>,
    Option<String>,
    i64,
);
fn redak_stabla(v: PodaciStavke, roditelj: Option<IdMape>, i: usize) -> Result<Vec<Value>, Greska> {
    let (id, vrsta, naziv, tablica, stil, stanje) = v;
    Ok(vec![
        vrijednost_id(id)?,
        vrsta.to_owned().into(),
        roditelj_broj(roditelj)?.into(),
        Value::Integer(i as i64),
        naziv.into(),
        tablica.into(),
        stil.into(),
        Value::Integer(stanje),
    ])
}
fn stanje(s: StanjeSloja) -> i64 {
    match s {
        StanjeSloja::Uredivo => 0,
        StanjeSloja::SamoCitanje => 1,
        StanjeSloja::CekaSlanje => 2,
        StanjeSloja::USukobu => 3,
    }
}
fn upisi_stablo(c: &Connection, prije: &Projekt, p: &Projekt) -> Result<Vec<Upis>, Greska> {
    let n = stablo(p)?;
    if !nedostaje_stablo(c, p)? && stablo(prije)? == n {
        return Ok(Vec::new());
    }
    let mut r = vec![Upis {
        sql: "DELETE FROM gpkg_ts_stablo".into(),
        v: Vec::new(),
    }];
    r.extend(n.into_iter().map(|v| Upis {
        sql: "INSERT INTO gpkg_ts_stablo VALUES(?,?,?,?,?,?,?,?)".into(),
        v,
    }));
    Ok(r)
}
fn id_upis(p: &Projekt, s: IdSloja, id: IdObjekta, fid: i64) -> Result<Upis, Greska> {
    Ok(Upis{sql:"INSERT INTO gpkg_ts_objekti(tablica,fid,id,otvoreni) VALUES(?1,?2,?3,?4) ON CONFLICT(tablica,id) DO UPDATE SET fid=excluded.fid,otvoreni=excluded.otvoreni ON CONFLICT(tablica,fid) DO UPDATE SET id=excluded.id,otvoreni=excluded.otvoreni".into(),v:vec![tablica(p,s).into(),fid.into(),broj(id.0)?.into(),p.objekt(s,id).and_then(|o|geometrija::otvoreni(&o.geometrija)).into()]})
}
fn metapodaci(prije: &Projekt, p: &Projekt, fids: &Fidovi) -> Result<Vec<Upis>, Greska> {
    let mut r = Vec::new();
    let izmjene = p.izmjene_od(prije);
    for (s, id) in &izmjene.upisi {
        r.push(id_upis(p, *s, *id, fids.get(*s, *id)?)?);
    }
    let slojevi: BTreeSet<_> = izmjene
        .upisi
        .iter()
        .chain(&izmjene.obrisi)
        .map(|(s, _)| s)
        .collect();
    r.extend(slojevi.into_iter().map(|s|Upis{sql:"UPDATE gpkg_contents SET last_change=strftime('%Y-%m-%dT%H:%M:%fZ','now'),min_x=NULL,min_y=NULL,max_x=NULL,max_y=NULL WHERE table_name=?1".into(),v:vec![tablica(p,*s).into()]}));
    Ok(r)
}

fn nepoznati_objekta(p: &Projekt, id: IdObjekta) -> BTreeMap<String, Vrijednost> {
    p.zapisi()
        .slojevi
        .values()
        .find_map(|s| s.objekti.get(&id))
        .map(|o| o.nepoznati.clone())
        .unwrap_or_default()
}
fn u_sql(v: Vrijednost) -> Value {
    match v {
        Vrijednost::Prazno => Value::Null,
        Vrijednost::Cijeli(i) => Value::Integer(i),
        Vrijednost::Broj(v) => Value::Real(v),
        Vrijednost::Tekst(s) => Value::Text(s),
        Vrijednost::Bajtovi(b) => Value::Blob(b),
    }
}
fn dodatni_stupci(
    c: &Connection,
    prije: &Projekt,
    p: &Projekt,
) -> Result<Vec<(String, String)>, Greska> {
    let zm = p.izmjene_od(prije);
    let mut trazeni: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (s, id) in zm
        .upisi
        .into_iter()
        .filter(|(s, id)| prije.objekt(*s, *id).is_none())
    {
        trazeni
            .entry(tablica(p, s))
            .or_default()
            .extend(nepoznati_objekta(p, id).into_keys());
    }
    let mut r = Vec::new();
    for (t, n) in trazeni {
        let postoje = poznati_stupci(c, p, &t)?;
        r.extend(
            n.into_iter()
                .filter(|n| !postoje.iter().any(|(p, _)| p == n))
                .map(|n| (t.clone(), n)),
        );
    }
    Ok(r)
}
fn dodaj_stupce(c: &Connection, n: Vec<(String, String)>) -> Result<(), Greska> {
    for (t, n) in n {
        c.execute_batch(&format!(
            "ALTER TABLE {} ADD COLUMN {} BLOB",
            shema::navodnici(&t),
            shema::navodnici(&n)
        ))?;
    }
    Ok(())
}

#[cfg(test)]
mod testovi {
    use super::*;
    fn projekt(id: u64, stavka: u64) -> Projekt {
        let o = Objekt {
            id: IdObjekta(id),
            naziv: "Pokus".into(),
            opis: String::new(),
            geometrija: Geometrija::Tocka(Vrh::novi(18., 45.)),
            stil: StilObjekta::default(),
        };
        let s = Sloj {
            naziv: "Sloj".into(),
            roditelj: None,
            vrsta: VrstaGeometrije::Tocka,
            stanje: StanjeSloja::Uredivo,
            stil: Stil::default(),
            objekti: BTreeMap::from([(o.id, o)]),
        };
        Projekt::iz_zapisa(
            vec![Stavka::Sloj(IdSloja(stavka))],
            BTreeMap::new(),
            BTreeMap::from([(IdSloja(stavka), s)]),
            Zapisi::default(),
        )
        .unwrap()
    }
    #[test]
    fn preveliki_i_nepostojeci_identiteti_nisu_tihi_upisi() {
        assert!(broj(u64::MAX).is_err());
        let p = projekt(u64::MAX, 1);
        let prazno = Projekt::default();
        let b = izvor(&BTreeMap::new(), &p, IdSloja(1));
        let c = Connection::open_in_memory().unwrap();
        assert!(pripremi(&c, &prazno, &p, &BTreeMap::new(), &Fidovi::default()).is_err());
        assert!(potpun_plan(&c, &prazno, &p, &BTreeMap::new()).is_err());
        assert!(objekt_za_upis(&p, &prazno, &b, IdObjekta(9), 1).is_err());
        assert!(id_upis(&p, IdSloja(1), IdObjekta(u64::MAX), 1).is_err());
        let preveliki_sloj = projekt(2, u64::MAX);
        assert!(upisi_stablo(&c, &prazno, &preveliki_sloj).is_err());
        assert!(upisi_stablo(&c, &preveliki_sloj, &prazno).is_err());
        assert!(metapodaci(&p, &prazno, &Fidovi::default()).is_ok());
        assert!(
            redak_stabla(
                (1, "mapa", "Mapa".into(), None, None, 0),
                Some(IdMape(u64::MAX)),
                0
            )
            .is_err()
        );
        assert!(sql_upisa("t", "fid", 1, Vec::new(), true).is_err());
    }
    #[test]
    fn dopuna_qgis_tablice_ne_brise_postojece_stupce() {
        let c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        c.execute_batch("CREATE TABLE qgis(fid INTEGER PRIMARY KEY,shape POINT,tudje BLOB); INSERT INTO gpkg_contents(table_name,data_type,srs_id) VALUES('qgis','features',4326); INSERT INTO gpkg_geometry_columns VALUES('qgis','shape','POINT',4326,0,0)").unwrap();
        let mut id = 0;
        let b = citanje::slojevi(&c, &[], &mut id).unwrap().pop().unwrap();
        let p = projekt(2, b.id.0);
        let s = p.sloj(b.id).unwrap();
        pripremi_sloj(&c, &b, s, None).unwrap();
        for n in ["stil_debljina", "stil_velicina"] {
            assert_eq!(
                c.query_row(
                    "SELECT type FROM pragma_table_info('qgis') WHERE name=?1",
                    [n],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
                "REAL"
            );
        }

        assert!(
            shema::stupci(&c, "qgis")
                .unwrap()
                .iter()
                .any(|(n, _)| n == "tudje")
        );
        c.execute_batch("PRAGMA query_only=ON").unwrap();
        assert!(dodaj_stupce(&c, vec![("qgis".into(), "novi".into())]).is_err());
        c.execute_batch("PRAGMA query_only=OFF; ALTER TABLE qgis DROP COLUMN stil_debljina; PRAGMA query_only=ON;").unwrap();
        assert!(pripremi_sloj(&c, &b, s, None).is_err());
    }
}

#[cfg(test)]
mod prekid_pripreme {
    use super::*;
    #[test]
    fn odbijeno_citanje_stupaca_prethodi_transakciji() {
        use rusqlite::hooks::{AuthContext, Authorization};
        let mut c = Connection::open_in_memory().unwrap();
        let mut u = Uredjivac::novi();
        let Some(Novo::Sloj(s)) = u
            .izvrsi(NaredbaStabla::DodajSloj {
                u: None,
                naziv: "Sloj".into(),
                vrsta: VrstaGeometrije::Tocka,
            })
            .unwrap()
        else {
            panic!()
        };
        u.izvrsi(NaredbaObjekta::Dodaj {
            sloj: s,
            naziv: "Pokus".into(),
            geometrija: Geometrija::Tocka(Vrh::novi(18., 45.)),
        })
        .unwrap();
        c.authorizer(Some(|_: AuthContext<'_>| Authorization::Deny))
            .unwrap();
        assert!(potpun_plan(&c, &Projekt::default(), u.projekt(), &BTreeMap::new()).is_err());
        assert!(c.is_autocommit());
    }
}

fn pridruzi_nepoznate(
    polja: &mut Vec<(String, Value)>,
    b: &citanje::SlojBaze,
    p: &Projekt,
    id: IdObjekta,
) -> Result<(), Greska> {
    for (n, v) in nepoznati_objekta(p, id) {
        if [b.geom.as_str(), b.fid.as_str()].contains(&n.as_str()) {
            return Err(format!("Nepoznati stupac {n} sukobljava se sa stupcem odredišnog sloja. Premještanje nije spremljeno.").into());
        }
        polja.push((n, u_sql(v)));
    }
    Ok(())
}

fn provjeri_uklanjanje(
    c: &Connection,
    prije: &Projekt,
    b: &citanje::SlojBaze,
) -> Result<(), Greska> {
    let sloj = prije.sloj(b.id).ok_or("Sloj za uklanjanje ne postoji.")?;
    let broj: i64 = c.query_row(
        &format!("SELECT count(*) FROM {}", shema::navodnici(&b.tablica)),
        [],
        |r| r.get(0),
    )?;
    if broj > sloj.objekti.len() as i64 {
        return Err("Sloj s nepodržanim ili praznim objektima ne može se ukloniti spremanjem. Izvorni zapisi ostaju netaknuti.".into());
    }
    Ok(())
}

fn izvrsi_upise(c: &Connection, upisi: impl Iterator<Item = Upis>) -> Result<(), Greska> {
    for u in upisi {
        c.prepare_cached(&u.sql)?
            .execute(rusqlite::params_from_iter(u.v))?;
    }
    Ok(())
}
fn obnovi_okidace(c: &Connection, novi: &[citanje::SlojBaze]) -> Result<(), Greska> {
    for b in novi {
        for sql in &b.okidaci {
            c.execute(sql, [])?;
        }
    }
    Ok(())
}
fn geometrija_za_upis(p: &Projekt, b: &citanje::SlojBaze, o: &Objekt) -> Result<Vec<u8>, Greska> {
    if let Some(raw) = izvorna_geometrija(p, b, o.id) {
        let g = geometrija::procitaj(raw, b.sustav)?;
        if geometrija::ista(&g, &o.geometrija) {
            return Ok(raw.to_vec());
        }
    }
    geometrija::zapisi(&o.geometrija, b.sustav, b.srs)
}
fn izvorna_geometrija<'a>(
    p: &'a Projekt,
    b: &citanje::SlojBaze,
    id: IdObjekta,
) -> Option<&'a [u8]> {
    p.zapisi()
        .slojevi
        .values()
        .filter(|z| z.srs_id == b.srs)
        .find_map(|z| z.objekti.get(&id))
        .and_then(|o| o.geometrija.as_deref())
}
fn treba_stil(
    c: &Connection,
    b: &citanje::SlojBaze,
    s: &Sloj,
    prije: Option<&Sloj>,
) -> Result<bool, Greska> {
    if let Some(p) = prije {
        return Ok(p.stil != s.stil);
    }
    if b.izvorni_stil == Some(s.stil) && stil_postoji(c, &b.tablica)? {
        return Ok(false);
    }
    Ok(true)
}
fn stil_postoji(c: &Connection, t: &str) -> Result<bool, Greska> {
    if !shema::postoji(c, "layer_styles")? {
        return Ok(false);
    }
    Ok(c.query_row(
        "SELECT EXISTS(SELECT 1 FROM layer_styles WHERE f_table_name=?1 AND useAsDefault=1)",
        [t],
        |r| r.get(0),
    )?)
}

fn poznati_stupci(c: &Connection, p: &Projekt, t: &str) -> Result<Vec<(String, bool)>, Greska> {
    let postoje = shema::stupci(c, t)?;
    if !postoje.is_empty() {
        return Ok(postoje);
    }
    Ok(p.zapisi()
        .slojevi
        .values()
        .find(|z| z.tablica == t)
        .map(|z| z.stupci.iter().map(|n| (n.clone(), false)).collect())
        .unwrap_or_default())
}

fn ukloni_indeks(c: &Connection, b: &citanje::SlojBaze) -> Result<(), Greska> {
    let postoji:bool=c.query_row("SELECT EXISTS(SELECT 1 FROM gpkg_extensions WHERE table_name=?1 AND column_name=?2 AND extension_name='gpkg_rtree_index')",params![b.tablica,b.geom],|r|r.get(0))?;
    if postoji {
        c.execute(
            &format!(
                "DROP TABLE IF EXISTS {}",
                shema::navodnici(&format!("rtree_{}_{}", b.tablica, b.geom))
            ),
            [],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod obnova_i_greske {
    use super::*;
    #[test]
    fn metapodaci_zahtijevaju_stvarne_brojeve() {
        let u = crate::kvarovi::model();
        let p = u.projekt();
        let prazno = Projekt::default();
        assert!(metapodaci(&prazno, p, &Fidovi::default()).is_err());
        let mut slojevi: BTreeMap<_, _> = p.slojevi().map(|(id, s)| (id, s.clone())).collect();
        let (id, s) = slojevi.iter_mut().next().unwrap();
        let id = *id;
        s.roditelj = None;
        let mut o = s.objekti.pop_first().unwrap().1;
        o.id = IdObjekta(u64::MAX);
        s.objekti.insert(o.id, o);
        let p = Projekt::iz_zapisa(
            vec![Stavka::Sloj(id)],
            BTreeMap::new(),
            BTreeMap::from([(id, slojevi[&id].clone())]),
            Zapisi::default(),
        )
        .unwrap();
        let mut c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        let b = postojeci(&c, &p).unwrap();
        let f = crate::fid::plan(&c, &prazno, &p, &b, &p.izmjene_od(&prazno)).unwrap();
        assert!(metapodaci(&prazno, &p, &f).is_err());
        c.execute_batch("PRAGMA query_only=ON").unwrap();
        assert!(spremi(&mut c, &prazno, &p, 1).is_err());
    }
    #[test]
    fn obnova_ne_prikriva_nevaljan_okidac_ili_indeks() {
        let mut c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        let u = crate::kvarovi::model();
        let (id, s) = u.projekt().slojevi().next().unwrap();
        let mut b = izvor_iz_modela(u.projekt(), id);
        shema::sloj_baze(&c, &b).unwrap();
        b.okidaci = vec!["CREATE TRIGGER nevaljano".into()];
        assert!(obnovi_okidace(&c, &[b.clone()]).is_err());
        b.izvorni_stil = Some(s.stil);
        c.execute_batch("DROP TABLE layer_styles").unwrap();
        assert!(!stil_postoji(&c, &b.tablica).unwrap());
        assert!(treba_stil(&c, &b, s, None).unwrap());
        c.execute_batch("CREATE TABLE layer_styles(f_table_name TEXT,useAsDefault INTEGER)")
            .unwrap();
        assert!(!stil_postoji(&c, &b.tablica).unwrap());
        c.execute("INSERT INTO layer_styles VALUES(?1,1)", [&b.tablica])
            .unwrap();
        assert!(!treba_stil(&c, &b, s, None).unwrap());
        c.execute_batch("DROP TABLE layer_styles; CREATE TABLE layer_styles(nevaljano TEXT)")
            .unwrap();
        assert!(stil_postoji(&c, &b.tablica).is_err());
        {
            use rusqlite::hooks::{AuthContext, Authorization};
            c.authorizer(Some(|_: AuthContext<'_>| Authorization::Deny))
                .unwrap();
            assert!(stil_postoji(&c, &b.tablica).is_err());
            c.authorizer(Some(|_: AuthContext<'_>| Authorization::Allow))
                .unwrap();
        }

        assert!(treba_stil(&c, &b, s, None).is_err());
        assert!(pripremi_sloj(&c, &b, s, None).is_err());
        let ime = format!("rtree_{}_{}", b.tablica, b.geom);
        c.execute_batch(&format!(
            "CREATE VIRTUAL TABLE {ime} USING rtree(id,min_x,max_x,min_y,max_y)"
        ))
        .unwrap();
        c.execute(
            "INSERT INTO gpkg_extensions VALUES(?1,?2,'gpkg_rtree_index','test','write-only')",
            params![b.tablica, b.geom],
        )
        .unwrap();
        ukloni_indeks(&c, &b).unwrap();
        assert!(!shema::postoji(&c, &ime).unwrap());
        c.execute_batch(&format!("CREATE VIRTUAL TABLE {ime} USING rtree(id,min_x,max_x,min_y,max_y); PRAGMA query_only=ON")).unwrap();
        assert!(ukloni_indeks(&c, &b).is_err());
        c.execute_batch("PRAGMA query_only=OFF; DROP TABLE gpkg_extensions")
            .unwrap();
        assert!(ukloni_indeks(&c, &b).is_err());
    }
    #[test]
    fn neuspjeli_okidac_unutar_primjene_vraca_gresku() {
        let mut c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        let u = crate::kvarovi::model();
        let p = u.projekt();
        let prazno = Projekt::default();
        let b = postojeci(&c, p).unwrap();
        let mut plan = potpun_plan(&c, &prazno, p, &b).unwrap();
        plan.novi[0].okidaci.push("CREATE TRIGGER nevaljano".into());
        let tx = c.transaction().unwrap();
        assert!(primijeni(&tx, &prazno, p, &b, plan).is_err());
        tx.rollback().unwrap();
        assert_eq!(citanje::projekt(&c).unwrap().0, prazno);
    }
}

// Broj sloja dolazi iz učitanog projekta, a ne iz novog prolaza po tablicama.
fn povezi_izvor(p: &Projekt, b: &mut citanje::SlojBaze) {
    if let Some((id, _)) = p
        .zapisi()
        .slojevi
        .iter()
        .find(|(_, z)| z.tablica == b.tablica)
    {
        b.id = *id;
    }
}
fn nedostaje_stablo(c: &Connection, p: &Projekt) -> Result<bool, Greska> {
    let n = citanje::cvorovi(c)?;
    Ok(p.zapisi()
        .slojevi
        .iter()
        .filter(|(id, _)| p.sloj(**id).is_some())
        .any(|(id, z)| {
            !n.iter()
                .any(|n| n.id == id.0 && n.tablica.as_deref() == Some(z.tablica.as_str()))
        }))
}

#[cfg(test)]
mod provjera_novih_upisa {
    use super::*;

    #[test]
    fn stablo_veze_identitet_i_tablicu_u_istom_retku() {
        let mut c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        geometrija::funkcije(&c).unwrap();
        let u = crate::kvarovi::model();
        spremi(&mut c, &Projekt::default(), u.projekt(), 1).unwrap();
        let p = citanje::projekt(&c).unwrap().0;
        assert!(!nedostaje_stablo(&c, &p).unwrap());
        assert!(upisi_stablo(&c, &p, &p).unwrap().is_empty());
        // Ista imena tablica uz zamijenjene brojeve nisu ista veza.
        c.execute_batch("UPDATE gpkg_ts_stablo SET tablica='zamjena_' || tablica WHERE vrsta='sloj'; UPDATE gpkg_ts_stablo SET tablica=CASE tablica WHEN 'zamjena_sloj_2' THEN 'sloj_4' WHEN 'zamjena_sloj_4' THEN 'sloj_2' END WHERE vrsta='sloj'").unwrap();
        assert!(nedostaje_stablo(&c, &p).unwrap());
        let plan = upisi_stablo(&c, &p, &p).unwrap();
        assert!(!plan.is_empty());
        izvrsi_upise(&c, plan.into_iter()).unwrap();
        assert_eq!(citanje::projekt(&c).unwrap().0, p);
        assert!(!nedostaje_stablo(&c, &p).unwrap());
    }

    #[test]
    fn odbijena_promjena_dimenzije_vraca_cijeli_upis() {
        let mut c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        geometrija::funkcije(&c).unwrap();
        let u = crate::kvarovi::model();
        spremi(&mut c, &Projekt::default(), u.projekt(), 1).unwrap();
        c.execute_batch("UPDATE gpkg_geometry_columns SET z=0; CREATE TRIGGER odbij_z BEFORE UPDATE OF z ON gpkg_geometry_columns BEGIN SELECT RAISE(ABORT,'odbijena dimenzija'); END").unwrap();
        let p = citanje::projekt(&c).unwrap().0;
        let s = p.slojevi().next().unwrap().0;
        let mut u = Uredjivac::iz_projekta(p.clone());
        u.izvrsi(NaredbaObjekta::Dodaj {
            sloj: s,
            naziv: "Visina".into(),
            geometrija: Geometrija::Tocka(Vrh {
                duzina: 18.,
                sirina: 45.,
                visina: Some(7.),
            }),
        })
        .unwrap();
        let e = spremi(&mut c, &p, u.projekt(), 1).unwrap_err();
        assert!(e.to_string().contains("odbijena dimenzija"));
        assert!(c.is_autocommit());
        assert_eq!(citanje::projekt(&c).unwrap().0, p);
        assert_eq!(
            c.query_row("SELECT max(z) FROM gpkg_geometry_columns", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
}
