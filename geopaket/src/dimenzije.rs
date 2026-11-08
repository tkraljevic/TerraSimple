//! Postojeći sloj smije sadržavati i 2D i 3D objekte bez izmišljanja visine.
use crate::{Greska, citanje::SlojBaze, shema};
use rusqlite::Connection;
use slojevi::{Geometrija, IdObjekta, IdSloja, IzmjeneSpremanja, Projekt, VrstaGeometrije};
use std::collections::{BTreeMap, BTreeSet};

pub fn pripremi(
    c: &Connection,
    p: &Projekt,
    b: &BTreeMap<IdSloja, SlojBaze>,
    zm: &IzmjeneSpremanja,
) -> Result<Vec<String>, Greska> {
    let mut r = BTreeSet::new();
    for par in &zm.upisi {
        pripremi_objekt(c, p, b, *par, &mut r)?;
    }
    Ok(r.into_iter().collect())
}
fn pripremi_objekt(
    c: &Connection,
    p: &Projekt,
    b: &BTreeMap<IdSloja, SlojBaze>,
    (s, id): (IdSloja, IdObjekta),
    r: &mut BTreeSet<String>,
) -> Result<(), Greska> {
    let b = b.get(&s).ok_or("Nedostaje sloj za provjeru dimenzija.")?;
    if b.postoji {
        let o = p
            .objekt(s, id)
            .ok_or("Nedostaje objekt za provjeru dimenzija.")?;
        if treba_z(c, b, &o.geometrija)? {
            r.insert(b.tablica.clone());
        }
    }
    Ok(())
}
fn treba_z(c: &Connection, b: &SlojBaze, g: &Geometrija) -> Result<bool, Greska> {
    let (tip, z, m): (String, i64, i64) = c.query_row(
        "SELECT geometry_type_name,z,m FROM gpkg_geometry_columns WHERE table_name=?1",
        [&b.tablica],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    provjeri(&tip, z, m, b.vrsta, ima_visinu(g))
}
fn provjeri(tip: &str, z: i64, m: i64, v: VrstaGeometrije, ima: bool) -> Result<bool, Greska> {
    if tip != shema::vrsta(v) {
        return Err("Upis novih objekata u sloj s nepodržanom vrstom geometrije nije dopušten. Izvorni zapisi ostaju sačuvani.".into());
    }
    if ![0, 2].contains(&m) {
        return Err("Obvezna ili nepoznata M dimenzija nije podržana pri upisu.".into());
    }
    match z {
        0 => Ok(ima),
        1 => Ok(!ima),
        2 => Ok(false),
        _ => Err("Deklaracija Z dimenzije nije poznata.".into()),
    }
}
fn ima_visinu(g: &Geometrija) -> bool {
    match g {
        Geometrija::Tocka(v) => v.visina.is_some(),
        Geometrija::Linija(v) => v.iter().any(|v| v.visina.is_some()),
        Geometrija::Poligon(p) => p.iter().flatten().any(|v| v.visina.is_some()),
    }
}
pub fn primijeni(c: &Connection, tablice: &[String]) -> Result<(), Greska> {
    for t in tablice {
        c.execute(
            "UPDATE gpkg_geometry_columns SET z=2 WHERE table_name=?1 AND z<>2",
            [t],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod testovi {
    use super::*;
    use slojevi::*;
    fn baza() -> (Connection, Uredjivac, BTreeMap<IdSloja, SlojBaze>) {
        let c = Connection::open_in_memory().unwrap();
        shema::stvori(&c).unwrap();
        shema::sloj(&c, "t", "T", VrstaGeometrije::Tocka).unwrap();
        let b = crate::citanje::slojevi(&c, &[], &mut 0)
            .unwrap()
            .pop()
            .unwrap();
        let mut u = Uredjivac::novi();
        u.izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "T".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
        .unwrap();
        for x in [18., 18.1] {
            u.izvrsi(NaredbaObjekta::Dodaj {
                sloj: b.id,
                naziv: "Visina".into(),
                geometrija: Geometrija::Tocka(Vrh {
                    duzina: x,
                    sirina: 45.,
                    visina: Some(7.),
                }),
            })
            .unwrap();
        }
        (c, u, BTreeMap::from([(b.id, b)]))
    }
    #[test]
    fn deklaracije_svih_vrsta_i_visina_odgovaraju_geopaketu() {
        for v in [
            VrstaGeometrije::Tocka,
            VrstaGeometrije::Linija,
            VrstaGeometrije::Poligon,
        ] {
            for (z, ocekuje) in [(0, [false, true]), (1, [true, false]), (2, [false, false])] {
                for (ima, r) in [false, true].into_iter().zip(ocekuje) {
                    for m in [0, 2] {
                        assert_eq!(provjeri(shema::vrsta(v), z, m, v, ima).unwrap(), r);
                    }
                }
            }
            for z in [-1, 3] {
                assert!(provjeri(shema::vrsta(v), z, 0, v, false).is_err());
            }
            for m in [-1, 1, 3] {
                assert!(provjeri(shema::vrsta(v), 2, m, v, false).is_err());
            }
        }
        assert!(provjeri("COMPOUNDCURVE", 2, 0, VrstaGeometrije::Linija, false).is_err());
        assert!(provjeri("CURVEPOLYGON", 2, 0, VrstaGeometrije::Poligon, false).is_err());
    }
    #[test]
    fn visina_u_jednom_vrhu_zahtijeva_z_zapis() {
        let a = Vrh::novi(18., 45.);
        let b = Vrh::novi(18.1, 45.1);
        let c = Vrh {
            duzina: 18.2,
            sirina: 45.,
            visina: Some(7.),
        };
        for g in [
            Geometrija::Tocka(a),
            Geometrija::Linija(vec![a, b]),
            Geometrija::Poligon(vec![vec![a, b, Vrh::novi(18.2, 45.)]]),
        ] {
            assert!(!ima_visinu(&g));
        }
        for g in [
            Geometrija::Tocka(c),
            Geometrija::Linija(vec![a, c]),
            Geometrija::Poligon(vec![vec![a, b, c]]),
        ] {
            assert!(ima_visinu(&g));
        }
    }
    #[test]
    fn priprema_je_bez_upisa_a_primjena_je_jedna_po_sloju() {
        let (c, u, mut b) = baza();
        let p = u.projekt();
        let zm = p.izmjene_od(&Projekt::default());
        c.execute_batch("UPDATE gpkg_geometry_columns SET z=0")
            .unwrap();
        let r = pripremi(&c, p, &b, &zm).unwrap();
        assert_eq!(r, vec!["t"]);
        assert_eq!(
            c.query_row("SELECT z FROM gpkg_geometry_columns", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            0
        );
        primijeni(&c, &r).unwrap();
        assert_eq!(
            c.query_row("SELECT z FROM gpkg_geometry_columns", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert!(pripremi(&c, p, &b, &zm).unwrap().is_empty());
        assert!(pripremi(&c, p, &BTreeMap::new(), &zm).is_err());
        let mut krivo = IzmjeneSpremanja::default();
        krivo.upisi.insert((IdSloja(1), IdObjekta(99)));
        assert!(pripremi(&c, p, &b, &krivo).is_err());
        b.get_mut(&IdSloja(1)).unwrap().postoji = false;
        assert!(pripremi(&c, p, &b, &krivo).unwrap().is_empty());
        b.get_mut(&IdSloja(1)).unwrap().postoji = true;
        c.execute_batch("UPDATE gpkg_geometry_columns SET m=1")
            .unwrap();
        assert!(pripremi(&c, p, &b, &zm).is_err());
        c.execute_batch("PRAGMA query_only=ON").unwrap();
        assert!(primijeni(&c, &r).is_err());
    }
    #[test]
    fn nevaljani_tipovi_metapodataka_i_nepostojeci_red_nisu_pretpostavka() {
        use rusqlite::types::Value;
        let (_c, u, b) = baza();
        let b = &b[&IdSloja(1)];
        let g = &u
            .projekt()
            .objekt(IdSloja(1), IdObjekta(2))
            .unwrap()
            .geometrija;
        let c = Connection::open_in_memory().unwrap();
        assert!(treba_z(&c, b, g).is_err());
        c.execute_batch("CREATE TABLE gpkg_geometry_columns(table_name,geometry_type_name,z,m)")
            .unwrap();
        assert!(treba_z(&c, b, g).is_err());
        for vrednosti in [
            [Value::Blob(vec![1]), 2.into(), 0.into()],
            [
                Value::Text("POINT".into()),
                Value::Text("loše".into()),
                0.into(),
            ],
            [
                Value::Text("POINT".into()),
                2.into(),
                Value::Text("loše".into()),
            ],
        ] {
            c.execute("DELETE FROM gpkg_geometry_columns", []).unwrap();
            c.execute(
                "INSERT INTO gpkg_geometry_columns VALUES('t',?1,?2,?3)",
                rusqlite::params_from_iter(vrednosti),
            )
            .unwrap();
            assert!(treba_z(&c, b, g).is_err());
        }
    }
}
