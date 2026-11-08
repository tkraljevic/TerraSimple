//! Veza unutarnjeg broja objekta i fid-a u svakoj tablici, uključujući poništena brisanja.
use crate::{Greska, citanje::SlojBaze, shema};
use rusqlite::Connection;
use slojevi::{IdObjekta, IdSloja, IzmjeneSpremanja, Projekt};
use std::collections::{BTreeMap, btree_map::Entry};
#[derive(Default)]
pub(crate) struct Fidovi(BTreeMap<(IdSloja, IdObjekta), i64>);
type Povijest = BTreeMap<String, BTreeMap<IdObjekta, i64>>;
impl Fidovi {
    pub fn get(&self, s: IdSloja, id: IdObjekta) -> Result<i64, Greska> {
        self.0
            .get(&(s, id))
            .copied()
            .ok_or_else(|| "Nedostaje veza objekta i fid-a.".into())
    }
}
pub fn plan(
    c: &Connection,
    prije: &Projekt,
    p: &Projekt,
    b: &BTreeMap<IdSloja, SlojBaze>,
    zm: &IzmjeneSpremanja,
) -> Result<Fidovi, Greska> {
    let h = povijest(c)?;
    let mut r = brisanja(prije, p, b, zm, &h)?;
    let mut brojaci = BTreeMap::new();
    for (s, id) in &zm.upisi {
        let iz = sloj(b, *s)?;
        r.0.insert((*s, *id), dodijeli(c, prije, p, iz, *id, &h, &mut brojaci)?);
    }
    Ok(r)
}
fn brisanja(
    prije: &Projekt,
    p: &Projekt,
    b: &BTreeMap<IdSloja, SlojBaze>,
    zm: &IzmjeneSpremanja,
    h: &Povijest,
) -> Result<Fidovi, Greska> {
    let mut r = Fidovi::default();
    for (s, id) in &zm.obrisi {
        if p.sloj(*s).is_some() {
            r.0.insert((*s, *id), postojeci(prije, sloj(b, *s)?, *id, h)?);
        }
    }
    Ok(r)
}

fn sloj(b: &BTreeMap<IdSloja, SlojBaze>, id: IdSloja) -> Result<&SlojBaze, Greska> {
    b.get(&id)
        .ok_or_else(|| "Nedostaje sloj za dodjelu fid-a.".into())
}
fn povijest(c: &Connection) -> Result<Povijest, Greska> {
    let mut r = Povijest::new();
    for ((t, fid), id) in crate::citanje::ids(c)? {
        r.entry(t).or_default().insert(IdObjekta(id), fid);
    }
    Ok(r)
}
fn iz_modela(p: &Projekt, s: IdSloja, id: IdObjekta) -> Option<i64> {
    p.zapisi()
        .slojevi
        .get(&s)
        .and_then(|s| s.objekti.get(&id))
        .map(|o| o.fid)
}
fn kandidat(p: &Projekt, b: &SlojBaze, id: IdObjekta, h: &Povijest) -> Option<i64> {
    h.get(&b.tablica)
        .and_then(|r| r.get(&id))
        .copied()
        .or_else(|| iz_modela(p, b.id, id))
}
fn postojeci(p: &Projekt, b: &SlojBaze, id: IdObjekta, h: &Povijest) -> Result<i64, Greska> {
    kandidat(p, b, id, h).ok_or_else(|| "Nedostaje fid postojećeg objekta.".into())
}
fn dodijeli(
    c: &Connection,
    prije: &Projekt,
    p: &Projekt,
    b: &SlojBaze,
    id: IdObjekta,
    h: &Povijest,
    brojaci: &mut BTreeMap<IdSloja, i64>,
) -> Result<i64, Greska> {
    if prije.objekt(b.id, id).is_some() {
        return postojeci(prije, b, id, h);
    }
    if let Some(fid) = kandidat(p, b, id, h)
        && !zauzet(c, b, fid)?
    {
        return Ok(fid);
    }
    sljedeci(c, p, b, h, brojaci)
}
fn zauzet(c: &Connection, b: &SlojBaze, fid: i64) -> Result<bool, Greska> {
    if !b.postoji {
        return Ok(false);
    }
    Ok(c.query_row(
        &format!(
            "SELECT EXISTS(SELECT 1 FROM {} WHERE {}=?1)",
            shema::navodnici(&b.tablica),
            shema::navodnici(&b.fid)
        ),
        [fid],
        |r| r.get(0),
    )?)
}
fn sljedeci(
    c: &Connection,
    p: &Projekt,
    b: &SlojBaze,
    h: &Povijest,
    brojaci: &mut BTreeMap<IdSloja, i64>,
) -> Result<i64, Greska> {
    let n = match brojaci.entry(b.id) {
        Entry::Occupied(v) => v.into_mut(),
        Entry::Vacant(v) => v.insert(pocetni(c, p, b, h)?),
    };
    *n = n
        .checked_add(1)
        .ok_or("U tablici nema slobodnog broja fid.")?;
    Ok(*n)
}
fn pocetni(c: &Connection, p: &Projekt, b: &SlojBaze, h: &Povijest) -> Result<i64, Greska> {
    let stari = h
        .get(&b.tablica)
        .into_iter()
        .flat_map(|m| m.values())
        .copied()
        .max()
        .unwrap_or(0);
    let izvorni = p
        .zapisi()
        .slojevi
        .get(&b.id)
        .into_iter()
        .flat_map(|s| s.objekti.values())
        .map(|o| o.fid)
        .max()
        .unwrap_or(0);
    Ok(najveci(c, b)?.max(stari).max(izvorni).max(0))
}
fn najveci(c: &Connection, b: &SlojBaze) -> Result<i64, Greska> {
    if !b.postoji {
        return Ok(0);
    }
    Ok(c.query_row(
        &format!(
            "SELECT coalesce(max({}),0) FROM {}",
            shema::navodnici(&b.fid),
            shema::navodnici(&b.tablica)
        ),
        [],
        |r| r.get(0),
    )?)
}

#[cfg(test)]
mod testovi {
    use super::*;
    fn baza() -> (Connection, SlojBaze) {
        let c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        shema::sloj(&c, "t", "T", slojevi::VrstaGeometrije::Tocka).unwrap();
        let b = crate::citanje::slojevi(&c, &[], &mut 0)
            .unwrap()
            .pop()
            .unwrap();
        (c, b)
    }
    #[test]
    fn fidovi_se_ne_prelijevaju_i_ne_pretpostavljaju() {
        let (c, mut b) = baza();
        let p = Projekt::default();
        let h = Povijest::new();
        let mut n = BTreeMap::new();
        assert!(postojeci(&p, &b, IdObjekta(9), &h).is_err());
        assert_eq!(sljedeci(&c, &p, &b, &h, &mut n).unwrap(), 1);
        assert_eq!(sljedeci(&c, &p, &b, &h, &mut n).unwrap(), 2);
        n.clear();
        assert!(sloj(&BTreeMap::new(), b.id).is_err());
        assert!(Fidovi::default().get(b.id, IdObjekta(9)).is_err());
        c.execute("INSERT INTO t(fid) VALUES(9223372036854775807)", [])
            .unwrap();
        assert!(sljedeci(&c, &p, &b, &h, &mut n).is_err());
        b.tablica = "nema".into();
        assert!(zauzet(&c, &b, 1).is_err());
        assert!(najveci(&c, &b).is_err());
        assert!(sljedeci(&c, &p, &b, &h, &mut BTreeMap::new()).is_err());
        let h = BTreeMap::from([("nema".into(), BTreeMap::from([(IdObjekta(9), 1)]))]);
        assert!(dodijeli(&c, &p, &p, &b, IdObjekta(9), &h, &mut n).is_err());
        b.postoji = false;
        assert!(!zauzet(&c, &b, 1).unwrap());
        assert_eq!(najveci(&c, &b).unwrap(), 0);
    }
    #[test]
    fn nepotpun_plan_odbija_i_brisanje_i_dodavanje() {
        let (c, b) = baza();
        let p = crate::kvarovi::model().projekt().clone();
        let (s, l) = p.slojevi().next().unwrap();
        let id = *l.objekti.keys().next().unwrap();
        let mut zm = slojevi::IzmjeneSpremanja::default();
        zm.obrisi.insert((s, id));
        assert!(plan(&c, &p, &p, &BTreeMap::new(), &zm).is_err());
        let mut b = b;
        b.id = s;
        assert!(plan(&c, &p, &p, &BTreeMap::from([(s, b.clone())]), &zm).is_err());
        zm.obrisi.clear();
        zm.upisi.insert((s, id));
        assert!(plan(&c, &p, &p, &BTreeMap::new(), &zm).is_err());
        assert!(plan(&c, &p, &p, &BTreeMap::from([(s, b)]), &zm).is_err());
        c.execute_batch("DROP TABLE gpkg_ts_objekti; CREATE TABLE gpkg_ts_objekti(tablica TEXT,fid INTEGER,id TEXT); INSERT INTO gpkg_ts_objekti VALUES('t',1,'loše')").unwrap();
        assert!(plan(&c, &p, &p, &BTreeMap::new(), &zm).is_err());
    }
}
