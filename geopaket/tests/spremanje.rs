use geopaket::Datoteka;
use rusqlite::Connection;
use slojevi::*;
use std::path::PathBuf;
fn put(ime: &str) -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.quality/testovi");
    std::fs::create_dir_all(&p).unwrap();
    p.join(format!("{ime}-{}.gpkg", std::process::id()))
}
fn pripremi(ime: &str) -> (PathBuf, Datoteka, Uredjivac, IdSloja, IdObjekta) {
    let p = put(ime);
    if p.exists() {
        std::fs::remove_file(&p).unwrap();
    }
    let mut u = Uredjivac::novi();
    let Some(Novo::Mapa(m)) = u
        .izvrsi(NaredbaStabla::DodajMapu {
            u: None,
            naziv: "Pokus žćš".into(),
        })
        .unwrap()
    else {
        panic!()
    };
    let Some(Novo::Sloj(s)) = u
        .izvrsi(NaredbaStabla::DodajSloj {
            u: Some(m),
            naziv: "Točke".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
        .unwrap()
    else {
        panic!()
    };
    let Some(Novo::Objekt(o)) = u
        .izvrsi(NaredbaObjekta::Dodaj {
            sloj: s,
            naziv: "Prva".into(),
            geometrija: Geometrija::Tocka(Vrh {
                duzina: 18.69,
                sirina: 45.55,
                visina: Some(88.25),
            }),
        })
        .unwrap()
    else {
        panic!()
    };
    let d = Datoteka::otvori(&p).unwrap();
    (p, d, u, s, o)
}
#[test]
fn cijeli_krug_i_ponistavanje_se_sprema() {
    let (p, mut d, u, s, o) = pripremi("krug");
    d.spremi(u.projekt()).unwrap();
    let procitan = d.ucitaj().unwrap();
    assert_eq!(procitan.korijen(), u.projekt().korijen());
    assert_eq!(procitan.sloj(s), u.projekt().sloj(s));
    let mut u = Uredjivac::iz_projekta(procitan);
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Opis("Promjena".into()),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    u.ponisti().unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(d.ucitaj().unwrap().objekt(s, o).unwrap().opis, "");
    let c = Connection::open(&p).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM gpkg_ts_stablo", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        2
    );
}
#[test]
fn nepoznati_stupci_i_neizmijenjeni_blob_ostaju() {
    let (p, mut d, u, s, o) = pripremi("nepoznati");
    d.spremi(u.projekt()).unwrap();
    let c = Connection::open(&p).unwrap();
    let t = format!("sloj_{}", s.0);
    c.execute_batch(&format!("ALTER TABLE {t} ADD COLUMN tudji BLOB; UPDATE {t} SET tudji=x'0011FE'; CREATE TABLE biljeske(v TEXT); INSERT INTO biljeske VALUES('ostaje');")).unwrap();
    let prije: Vec<u8> = c
        .query_row(&format!("SELECT geom FROM {t}"), [], |r| r.get(0))
        .unwrap();
    let p = d.ucitaj().unwrap();
    assert_eq!(
        p.zapisi().slojevi[&s].objekti[&o].nepoznati["tudji"],
        Vrijednost::Bajtovi(vec![0, 17, 254])
    );
    let mut u = Uredjivac::iz_projekta(p);
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Naziv("Druga".into()),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    let poslije: Vec<u8> = c
        .query_row(&format!("SELECT geom FROM {t}"), [], |r| r.get(0))
        .unwrap();
    assert_eq!(prije, poslije);
    let drugi = put("kopija");
    if drugi.exists() {
        std::fs::remove_file(&drugi).unwrap();
    }
    d.spremi_kao(&drugi, u.projekt()).unwrap();
    let c = Connection::open(drugi).unwrap();
    assert_eq!(
        c.query_row("SELECT v FROM biljeske", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ostaje"
    );
}
#[test]
fn vanjska_izmjena_je_sukob_a_ne_prepisivanje() {
    let (_p, mut d, u, s, _) = pripremi("sukob");
    d.spremi(u.projekt()).unwrap();
    let p = d.ucitaj().unwrap();
    let c = Connection::open(put("sukob")).unwrap();
    c.execute(&format!("UPDATE sloj_{} SET naziv='QGIS'", s.0), [])
        .unwrap();
    assert!(
        d.spremi(&p)
            .unwrap_err()
            .to_string()
            .contains("druga aplikacija")
    );
    assert_eq!(
        d.ucitaj()
            .unwrap()
            .sloj(s)
            .unwrap()
            .objekti
            .values()
            .next()
            .unwrap()
            .naziv,
        "QGIS"
    );
}
#[test]
fn pad_pravog_sql_upisa_vraca_cijelu_transakciju() {
    let (p, mut d, mut u, s, o) = pripremi("prekid");
    d.spremi(u.projekt()).unwrap();
    let c = Connection::open(&p).unwrap();
    c.execute_batch(&format!("CREATE TRIGGER pokvari BEFORE UPDATE ON sloj_{} BEGIN SELECT RAISE(ABORT,'namjerni prekid'); END;",s.0)).unwrap();
    let stari = d.ucitaj().unwrap();
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Naziv("Ne smije ostati".into()),
    })
    .unwrap();
    assert!(d.spremi(u.projekt()).is_err());
    assert_eq!(d.ucitaj().unwrap(), stari);
}

#[test]
fn brisanje_premjestanje_stil_i_pravi_st_okidaci() {
    let (p, mut d, mut u, s, o) = pripremi("okidaci");
    d.spremi(u.projekt()).unwrap();
    let c = Connection::open(&p).unwrap();
    c.execute_batch(&format!("ALTER TABLE sloj_{} ADD COLUMN x_int INTEGER; ALTER TABLE sloj_{} ADD COLUMN x_text TEXT; ALTER TABLE sloj_{} ADD COLUMN x_real REAL; ALTER TABLE sloj_{} ADD COLUMN x_blob BLOB; ALTER TABLE sloj_{} ADD COLUMN x_null TEXT; UPDATE sloj_{} SET x_int=7,x_text='ostaje',x_real=1.25,x_blob=x'0011';",s.0,s.0,s.0,s.0,s.0,s.0)).unwrap();
    c.execute_batch(&format!("CREATE VIRTUAL TABLE rtree_pokus USING rtree(id,minx,maxx,miny,maxy); CREATE TRIGGER pokus_update AFTER UPDATE OF geom ON sloj_{} BEGIN INSERT OR REPLACE INTO rtree_pokus VALUES(NEW.fid,ST_MinX(NEW.geom),ST_MaxX(NEW.geom),ST_MinY(NEW.geom),ST_MaxY(NEW.geom)); END;",s.0)).unwrap();
    let p = d.ucitaj().unwrap();
    u = Uredjivac::iz_projekta(p);
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Geometrija(Geometrija::Tocka(Vrh::novi(19., 46.))),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(
        c.query_row("SELECT minx FROM rtree_pokus", [], |r| r.get::<_, f64>(0))
            .unwrap(),
        19.
    );
    u.izvrsi(NaredbaStabla::PostaviStil {
        sloj: s,
        stil: Stil {
            debljina_mm: 2.,
            oblik_znaka: OblikZnaka::Kvadrat,
            ..Stil::default()
        },
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    let Some(Novo::Sloj(novi)) = u
        .izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "Novi".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
        .unwrap()
    else {
        panic!()
    };
    u.izvrsi(NaredbaObjekta::Premjesti {
        iz: s,
        u: novi,
        objekt: o,
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    assert!(d.ucitaj().unwrap().objekt(novi, o).is_some());
    let dodatno: (i64, String, f64, Vec<u8>, Option<String>) = c
        .query_row(
            &format!(
                "SELECT x_int,x_text,x_real,x_blob,x_null FROM sloj_{}",
                novi.0
            ),
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .unwrap();
    assert_eq!(dodatno, (7, "ostaje".into(), 1.25, vec![0, 17], None));

    u.izvrsi(NaredbaObjekta::Obrisi {
        sloj: novi,
        objekt: o,
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    u.izvrsi(NaredbaStabla::Obrisi {
        stavka: Stavka::Sloj(novi),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    assert!(d.ucitaj().unwrap().sloj(novi).is_none());
}

#[test]
fn pogresna_datoteka_i_odrediste_se_ne_prepisuju() {
    let p = put("pogresna");
    std::fs::write(&p, b"Ovo nije baza").unwrap();
    assert!(Datoteka::otvori(&p).is_err());
    assert_eq!(std::fs::read(&p).unwrap(), b"Ovo nije baza");
    let db = put("obicni_sqlite");
    if db.exists() {
        std::fs::remove_file(&db).unwrap();
    }
    let c = Connection::open(&db).unwrap();
    c.execute_batch("CREATE TABLE tudje(v TEXT)").unwrap();
    drop(c);
    assert!(Datoteka::otvori(&db).is_err());
    assert!(Datoteka::otvori(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).is_err());
    let (_p, mut d, u, _, _) = pripremi("postoji");
    assert!(d.spremi_kao(&p, u.projekt()).is_err());
}

#[test]
fn cijeli_poligon_s_rupom_visinom_i_stilom_vraca_iste_vrhove() {
    for zatvoren in [false, true] {
        let (put, mut d, mut u, _, _) = pripremi(if zatvoren { "zatvoren" } else { "otvoren" });
        let Some(Novo::Sloj(s)) = u
            .izvrsi(NaredbaStabla::DodajSloj {
                u: None,
                naziv: "Poligon".into(),
                vrsta: VrstaGeometrije::Poligon,
            })
            .unwrap()
        else {
            panic!()
        };
        let v = |x, y| Vrh {
            duzina: x,
            sirina: y,
            visina: Some(23.123456789),
        };
        let mut rub = vec![v(18.123456789, 45.123456789), v(18.2, 45.1), v(18.2, 45.2)];
        if zatvoren {
            rub.push(rub[0]);
        }
        let g = Geometrija::Poligon(vec![rub.clone(), rub]);
        let Some(Novo::Objekt(o)) = u
            .izvrsi(NaredbaObjekta::Dodaj {
                sloj: s,
                naziv: "S rupom".into(),
                geometrija: g.clone(),
            })
            .unwrap()
        else {
            panic!()
        };
        u.izvrsi(NaredbaObjekta::Izmijeni {
            sloj: s,
            objekt: o,
            izmjena: IzmjenaObjekta::Stil(StilObjekta {
                debljina_mm: Some(2.),
                boja_crte: Some(Boja {
                    r: 1,
                    g: 2,
                    b: 3,
                    a: 4,
                }),
                ..StilObjekta::default()
            }),
        })
        .unwrap();
        d.spremi(u.projekt()).unwrap();
        drop(d);
        let mut d = Datoteka::otvori(put).unwrap();
        let p = d.ucitaj().unwrap();
        assert_eq!(&p.objekt(s, o).unwrap().geometrija, &g);
        assert_eq!(p.sloj(s), u.projekt().sloj(s));
    }
}

#[test]
fn sva_stanja_sloja_vracaju_se_iz_datoteke() {
    let (_p, mut d, mut u, s, _) = pripremi("stanja");
    for stanje in [
        StanjeSloja::SamoCitanje,
        StanjeSloja::CekaSlanje,
        StanjeSloja::USukobu,
        StanjeSloja::Uredivo,
    ] {
        u.postavi_stanje_sloja(s, stanje).unwrap();
        d.spremi(u.projekt()).unwrap();
        assert_eq!(d.ucitaj().unwrap().sloj(s).unwrap().stanje, stanje);
    }
}

#[test]
fn jednaki_nazivi_slojeva_su_dopusteni() {
    let (_p, mut d, mut u, _, _) = pripremi("jednaki-nazivi");
    let Some(Novo::Sloj(s)) = u
        .izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "Točke".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
        .unwrap()
    else {
        panic!()
    };
    d.spremi(u.projekt()).unwrap();
    assert_eq!(d.ucitaj().unwrap().sloj(s).unwrap().naziv, "Točke");
}
#[test]
fn kolizija_nepoznatog_stupca_odbija_premjestanje_bez_gubitka() {
    let (put, mut d, u, s, o) = pripremi("kolizija");
    d.spremi(u.projekt()).unwrap();
    let c = Connection::open(&put).unwrap();
    c.execute_batch(&format!("ALTER TABLE sloj_{} RENAME COLUMN geom TO shape; UPDATE gpkg_geometry_columns SET column_name='shape'; ALTER TABLE sloj_{} ADD COLUMN geom TEXT; UPDATE sloj_{} SET geom='Sačuvaj me';",s.0,s.0,s.0)).unwrap();
    let mut u = Uredjivac::iz_projekta(d.ucitaj().unwrap());
    let Some(Novo::Sloj(t)) = u
        .izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "Novi".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
        .unwrap()
    else {
        panic!()
    };
    u.izvrsi(NaredbaObjekta::Premjesti {
        iz: s,
        u: t,
        objekt: o,
    })
    .unwrap();
    assert!(
        d.spremi(u.projekt())
            .unwrap_err()
            .to_string()
            .contains("sukobljava")
    );
    assert_eq!(
        c.query_row(&format!("SELECT geom FROM sloj_{}", s.0), [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "Sačuvaj me"
    );
    assert!(d.ucitaj().unwrap().objekt(s, o).is_some());
}
#[test]
fn sukob_se_provjerava_i_prije_kopiranja() {
    let (put, mut d, u, s, _) = pripremi("sukob-kopije");
    d.spremi(u.projekt()).unwrap();
    let p = d.ucitaj().unwrap();
    let c = Connection::open(&put).unwrap();
    c.execute(&format!("UPDATE sloj_{} SET naziv='Tuđa izmjena'", s.0), [])
        .unwrap();
    let izlaz = put.with_file_name(format!("sukob-kopije-izlaz-{}.gpkg", std::process::id()));
    assert!(d.spremi_kao(&izlaz, &p).is_err());
    assert!(!izlaz.exists());
}

#[test]
fn ucitani_projekt_ide_u_kopiju_izvora_a_ne_u_praznu_bazu() {
    let (put, mut d, u, _, _) = pripremi("zastita-kopije");
    d.spremi(u.projekt()).unwrap();
    let p = d.ucitaj().unwrap();
    let novi = put.with_file_name(format!("prazna-kopija-{}.gpkg", std::process::id()));
    if novi.exists() {
        std::fs::remove_file(&novi).unwrap();
    }
    let mut prazna = Datoteka::otvori(novi).unwrap();
    assert!(
        prazna
            .spremi(&p)
            .unwrap_err()
            .to_string()
            .contains("spremi_kao")
    );
    let mut neprocitana = Datoteka::otvori(put).unwrap();
    assert!(
        neprocitana
            .spremi(&p)
            .unwrap_err()
            .to_string()
            .contains("najprije")
    );
}
#[test]
fn sloj_s_nepodrzanim_zapisom_broji_ga_i_brise_se_s_njim() {
    let (put, mut d, u, s, _) = pripremi("nepodrzani-brisanje");
    let mapa = Stavka::Mapa(u.projekt().sloj(s).unwrap().roditelj.unwrap());
    assert_eq!(u.projekt().nepodrzani_zapisi(Stavka::Sloj(s)), 0);
    d.spremi(u.projekt()).unwrap();
    let c = Connection::open(&put).unwrap();
    let mut krivulja = vec![b'G', b'P', 0, 1, 230, 16, 0, 0, 1, 9, 0, 0, 0];
    krivulja.extend(0_u32.to_le_bytes());
    let tablica = format!("sloj_{}", s.0);
    c.execute(
        &format!("INSERT INTO {tablica}(fid,geom) VALUES(999,?1)"),
        [krivulja.clone()],
    )
    .unwrap();
    c.execute(
        &format!("INSERT INTO {tablica}(fid,geom) VALUES(1000,NULL)"),
        [],
    )
    .unwrap();
    let mut u = Uredjivac::iz_projekta(d.ucitaj().unwrap());
    assert_eq!(u.projekt().nepodrzani_zapisi(Stavka::Sloj(s)), 2);
    assert_eq!(u.projekt().nepodrzani_zapisi(mapa), 2);
    let broj = |c: &Connection| {
        c.query_row(&format!("SELECT count(*) FROM {tablica}"), [], |r| {
            r.get::<_, i64>(0)
        })
    };
    u.izvrsi(NaredbaStabla::Obrisi {
        stavka: Stavka::Sloj(s),
    })
    .unwrap();
    assert_eq!(u.projekt().nepodrzani_zapisi(Stavka::Sloj(s)), 0);
    assert_eq!(u.projekt().nepodrzani_zapisi(mapa), 0);
    u.ponisti().unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(broj(&c).unwrap(), 3, "poništeno brisanje ne dira zapise");
    u.ponovi().unwrap();
    d.spremi(u.projekt()).unwrap();
    assert!(broj(&c).is_err(), "tablica sloja mora nestati");
    let procitan = d.ucitaj().unwrap();
    assert!(procitan.sloj(s).is_none());
    assert_eq!(procitan.nepodrzani_zapisi(mapa), 0);
}
#[test]
fn premjestanje_i_ponistavanje_cuvaju_fid_i_kad_se_brojevi_u_slojevima_preklapaju() {
    let (put, mut d, mut u, a, o) = pripremi("fid-premjestanje");
    let Some(Novo::Sloj(b)) = u
        .izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "Drugi".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
        .unwrap()
    else {
        panic!()
    };
    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: b,
        naziv: "Drugi objekt".into(),
        geometrija: Geometrija::Tocka(Vrh::novi(19., 46.)),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    let p = d.ucitaj().unwrap();
    let izvorni = p.zapisi().slojevi[&a].objekti[&o].fid;
    let mut u = Uredjivac::iz_projekta(p);
    u.izvrsi(NaredbaObjekta::Premjesti {
        iz: a,
        u: b,
        objekt: o,
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    let p = d.ucitaj().unwrap();
    assert_eq!(p.sloj(b).unwrap().objekti.len(), 2);
    u.ponisti().unwrap();
    d.spremi(u.projekt()).unwrap();
    let p = d.ucitaj().unwrap();
    assert_eq!(p.zapisi().slojevi[&a].objekti[&o].fid, izvorni);
    // I brisanje novog objekta, koji nije bio u početnom učitanom projektu.
    let Some(Novo::Objekt(novi)) = u
        .izvrsi(NaredbaObjekta::Dodaj {
            sloj: a,
            naziv: "Naknadni".into(),
            geometrija: Geometrija::Tocka(Vrh::novi(18.5, 45.5)),
        })
        .unwrap()
    else {
        panic!()
    };
    d.spremi(u.projekt()).unwrap();
    let nfid = d.ucitaj().unwrap().zapisi().slojevi[&a].objekti[&novi].fid;
    u.izvrsi(NaredbaObjekta::Obrisi {
        sloj: a,
        objekt: novi,
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    u.ponisti().unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(
        d.ucitaj().unwrap().zapisi().slojevi[&a].objekti[&novi].fid,
        nfid
    );
    let c = Connection::open(put).unwrap();
    assert_eq!(
        c.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}
#[test]
fn uklonjen_sloj_se_vraca_sa_shemom_sustavom_i_istim_bajtovima() {
    let put = put("obnova-sheme");
    if put.exists() {
        std::fs::remove_file(&put).unwrap();
    }
    let mut d = Datoteka::otvori(&put).unwrap();
    let c = Connection::open(&put).unwrap();
    let q: serde_json::Value = serde_json::from_str(include_str!("podaci/qgis.json")).unwrap();
    let s = q["srs"].as_array().unwrap();
    c.execute(
        "INSERT INTO gpkg_spatial_ref_sys VALUES(?1,3765,'EPSG',3765,?2,NULL)",
        rusqlite::params![s[0].as_str().unwrap(), s[4].as_str().unwrap()],
    )
    .unwrap();
    c.execute_batch("CREATE TABLE izvor(fid INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,shape LINESTRING,naziv TEXT,opis TEXT,tudji INTEGER NOT NULL DEFAULT 17 UNIQUE);CREATE UNIQUE INDEX ime_izvora ON izvor(naziv);CREATE TABLE dnevnik(n INTEGER);INSERT INTO dnevnik VALUES(0);CREATE TRIGGER promjena_opisa AFTER UPDATE OF opis ON izvor BEGIN UPDATE dnevnik SET n=n+1;END;INSERT INTO gpkg_contents(table_name,data_type,identifier,srs_id) VALUES('izvor','features','Izvor',3765);INSERT INTO gpkg_geometry_columns VALUES('izvor','shape','LINESTRING',3765,1,0);").unwrap();
    let hex = q["geometrija_hex"].as_str().unwrap();
    let blob: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect();
    c.execute(
        "INSERT INTO izvor(fid,shape,naziv,opis) VALUES(1000,?1,'Izmišljeno','')",
        [blob.clone()],
    )
    .unwrap();
    let p = d.ucitaj().unwrap();
    let (id, sloj) = p.slojevi().next().unwrap();
    let o = *sloj.objekti.keys().next().unwrap();
    let mut u = Uredjivac::iz_projekta(p);
    u.izvrsi(NaredbaStabla::Obrisi {
        stavka: Stavka::Sloj(id),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    u.ponisti().unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(
        c.query_row("SELECT shape FROM izvor WHERE fid=1000", [], |r| r
            .get::<_, Vec<u8>>(0))
            .unwrap(),
        blob
    );
    assert_eq!(
        c.query_row(
            "SELECT srs_id FROM gpkg_geometry_columns WHERE table_name='izvor'",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        3765
    );
    let sql: String = c
        .query_row(
            "SELECT sql FROM sqlite_master WHERE name='izvor'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(sql.contains("tudji INTEGER NOT NULL DEFAULT 17 UNIQUE"));
    assert!(
        c.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name='ime_izvora')",
            [],
            |r| r.get::<_, bool>(0)
        )
        .unwrap()
    );
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: id,
        objekt: o,
        izmjena: IzmjenaObjekta::Opis("Nakon obnove".into()),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(
        c.query_row("SELECT n FROM dnevnik", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
}

#[test]
fn qgis_novi_sloj_ulazi_u_postojece_stablo_i_stil_je_u_layer_styles() {
    let (put, mut d, u, _, _) = pripremi("qgis-novi-sloj");
    d.spremi(u.projekt()).unwrap();
    let c = Connection::open(&put).unwrap();
    assert_eq!(
        c.query_row(
            "SELECT count(*) FROM layer_styles WHERE useAsDefault=1",
            [],
            |r| r.get::<_, i64>(0)
        )
        .unwrap(),
        1
    );
    c.execute_batch("CREATE TABLE novi(fid INTEGER PRIMARY KEY,geom POINT,naziv TEXT);INSERT INTO gpkg_contents(table_name,data_type,identifier,srs_id) VALUES('novi','features','Novi iz QGIS-a',4326);INSERT INTO gpkg_geometry_columns VALUES('novi','geom','POINT',4326,2,0);").unwrap();
    let p = d.ucitaj().unwrap();
    assert_eq!(p.slojevi().count(), 2);
    assert!(p.slojevi().any(|(_, s)| s.naziv == "Novi iz QGIS-a"));
}
#[test]
fn nepoznati_stupac_se_dodaje_i_postojecem_odredisnom_sloju() {
    let (put, mut d, mut u, a, o) = pripremi("poznat-odredisni");
    let Some(Novo::Sloj(b)) = u
        .izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "Postojeći".into(),
            vrsta: VrstaGeometrije::Tocka,
        })
        .unwrap()
    else {
        panic!()
    };
    d.spremi(u.projekt()).unwrap();
    let c = Connection::open(&put).unwrap();
    c.execute_batch(&format!(
        "ALTER TABLE sloj_{} ADD COLUMN tudji TEXT;UPDATE sloj_{} SET tudji='ostaje';",
        a.0, a.0
    ))
    .unwrap();
    let mut u = Uredjivac::iz_projekta(d.ucitaj().unwrap());
    u.izvrsi(NaredbaObjekta::Premjesti {
        iz: a,
        u: b,
        objekt: o,
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(
        c.query_row(&format!("SELECT tudji FROM sloj_{}", b.0), [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "ostaje"
    );
}
#[test]
fn spremljeni_stil_odgovara_novoj_vrijednosti_a_ne_starom_qml_u() {
    let (put, mut d, mut u, s, _) = pripremi("stil-promjena");
    d.spremi(u.projekt()).unwrap();
    let stil = Stil {
        debljina_mm: 1.7,
        oblik_znaka: OblikZnaka::Kvadrat,
        boja_crte: Boja {
            r: 10,
            g: 20,
            b: 30,
            a: 180,
        },
        ..Stil::default()
    };
    u.izvrsi(NaredbaStabla::PostaviStil { sloj: s, stil })
        .unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(d.ucitaj().unwrap().sloj(s).unwrap().stil, stil);
    let c = Connection::open(put).unwrap();
    let q: String = c
        .query_row(
            "SELECT styleQML FROM layer_styles WHERE useAsDefault=1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(q.contains("square"));
    assert!(q.contains("10,20,30,180"));
    assert!(q.contains("1.7"));
}

#[test]
fn novi_projekt_sprema_vise_izmjena_bez_ponovnog_ucitavanja() {
    let (put, mut d, mut u, s, o) = pripremi("vise-spremanja");
    d.spremi(u.projekt()).unwrap();
    let c = Connection::open(&put).unwrap();
    let t = format!("sloj_{}", s.0);
    let (fid, blob): (i64, Vec<u8>) = c
        .query_row(&format!("SELECT fid,geom FROM {t}"), [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Opis("Drugi zapis bez čitanja".into()),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(
        c.query_row(&format!("SELECT opis FROM {t} WHERE fid=?1"), [fid], |r| {
            r.get::<_, String>(0)
        })
        .unwrap(),
        "Drugi zapis bez čitanja"
    );
    assert_eq!(
        c.query_row(&format!("SELECT geom FROM {t} WHERE fid=?1"), [fid], |r| {
            r.get::<_, Vec<u8>>(0)
        })
        .unwrap(),
        blob
    );
    u.ponisti().unwrap();
    d.spremi(u.projekt()).unwrap();
    u.izvrsi(NaredbaObjekta::Obrisi { sloj: s, objekt: o })
        .unwrap();
    d.spremi(u.projekt()).unwrap();
    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: "Privremeni novi".into(),
        geometrija: Geometrija::Tocka(Vrh::novi(18., 45.)),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    let novi_fid: i64 = c
        .query_row(&format!("SELECT fid FROM {t}"), [], |r| r.get(0))
        .unwrap();
    assert!(novi_fid > fid);
    u.ponisti().unwrap();
    u.ponisti().unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(
        c.query_row(&format!("SELECT fid,geom FROM {t}"), [], |r| Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, Vec<u8>>(1)?
        )))
        .unwrap(),
        (fid, blob)
    );
    assert_eq!(d.ucitaj().unwrap().objekt(s, o), u.projekt().objekt(s, o));
}

#[test]
fn ugnijezdene_mape_i_rucni_redoslijed_ostaju_nakon_spremanja_i_ponistavanja() {
    let (_put, mut d, mut u, s, _) = pripremi("redoslijed-mapa");
    let m = u.projekt().sloj(s).unwrap().roditelj.unwrap();
    let Some(Novo::Mapa(druga)) = u
        .izvrsi(NaredbaStabla::DodajMapu {
            u: None,
            naziv: "Druga mapa žć".into(),
        })
        .unwrap()
    else {
        panic!()
    };
    let Some(Novo::Mapa(unutra)) = u
        .izvrsi(NaredbaStabla::DodajMapu {
            u: Some(m),
            naziv: "Unutarnja".into(),
        })
        .unwrap()
    else {
        panic!()
    };
    let mut novi = Vec::new();
    for (roditelj, naziv) in [(Some(m), "Zadnji"), (Some(unutra), "Duboki"), (None, "Vrh")] {
        let Some(Novo::Sloj(id)) = u
            .izvrsi(NaredbaStabla::DodajSloj {
                u: roditelj,
                naziv: naziv.into(),
                vrsta: VrstaGeometrije::Tocka,
            })
            .unwrap()
        else {
            panic!()
        };
        novi.push(id);
    }
    u.izvrsi(NaredbaStabla::Premjesti {
        stavka: Stavka::Sloj(s),
        u: Some(druga),
        polozaj: 0,
    })
    .unwrap();
    u.izvrsi(NaredbaStabla::Premjesti {
        stavka: Stavka::Sloj(novi[0]),
        u: Some(m),
        polozaj: 0,
    })
    .unwrap();
    u.izvrsi(NaredbaStabla::Premjesti {
        stavka: Stavka::Sloj(novi[2]),
        u: None,
        polozaj: 0,
    })
    .unwrap();
    let provjeri = |p: &Projekt, q: &Projekt| {
        assert_eq!(p.korijen(), q.korijen());
        for id in [m, druga, unutra] {
            assert_eq!(p.mapa(id), q.mapa(id));
        }
        assert_eq!(
            p.slojevi().collect::<Vec<_>>(),
            q.slojevi().collect::<Vec<_>>()
        );
    };
    d.spremi(u.projekt()).unwrap();
    provjeri(&d.ucitaj().unwrap(), u.projekt());
    u.izvrsi(NaredbaStabla::Preimenuj {
        stavka: Stavka::Mapa(druga),
        naziv: "Preimenovana šđč".into(),
    })
    .unwrap();
    u.izvrsi(NaredbaStabla::Premjesti {
        stavka: Stavka::Mapa(unutra),
        u: None,
        polozaj: 1,
    })
    .unwrap();
    u.izvrsi(NaredbaStabla::Premjesti {
        stavka: Stavka::Sloj(novi[2]),
        u: Some(unutra),
        polozaj: 0,
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    provjeri(&d.ucitaj().unwrap(), u.projekt());
    for _ in 0..3 {
        u.ponisti().unwrap();
    }
    d.spremi(u.projekt()).unwrap();
    provjeri(&d.ucitaj().unwrap(), u.projekt());
}

#[test]
fn sloj_bez_zapisanog_stabla_moze_se_ponovno_otvoriti_i_spremiti() {
    let (put, mut d, u, _, _) = pripremi("strani-identitet");
    d.spremi(u.projekt()).unwrap();
    drop(d);
    let c = Connection::open(&put).unwrap();
    c.execute_batch("DELETE FROM gpkg_ts_stablo; UPDATE gpkg_ts_objekti SET id=100")
        .unwrap();
    let mut d = Datoteka::otvori(&put).unwrap();
    let p = d.ucitaj().unwrap();
    let s = p.slojevi().next().unwrap().0;
    let o = *p.sloj(s).unwrap().objekti.keys().next().unwrap();
    let mut u = Uredjivac::iz_projekta(p);
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Opis("Prvi zapis stranog sloja".into()),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(
        c.query_row("SELECT count(*) FROM gpkg_ts_stablo", [], |r| r
            .get::<_, i64>(0))
            .unwrap(),
        1
    );
    drop(d);
    let mut d = Datoteka::otvori(&put).unwrap();
    let p = d.ucitaj().unwrap();
    assert_eq!(p.slojevi().next().unwrap().0, s);
    assert_eq!(p.objekt(s, o).unwrap().opis, "Prvi zapis stranog sloja");
    let mut u = Uredjivac::iz_projekta(p);
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Naziv("Ponovno otvoren".into()),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    assert_eq!(
        d.ucitaj().unwrap().objekt(s, o).unwrap().naziv,
        "Ponovno otvoren"
    );
}

#[test]
fn mijesanje_visina_prilagodjava_samo_deklaraciju_a_ne_stare_geometrije() {
    for (deklaracija, visina) in [(1, None), (0, Some(17.))] {
        let (put, mut d, mut u, s, o) = pripremi(&format!("dimenzije-{deklaracija}"));
        if deklaracija == 0 {
            u.izvrsi(NaredbaObjekta::Izmijeni {
                sloj: s,
                objekt: o,
                izmjena: IzmjenaObjekta::Geometrija(Geometrija::Tocka(Vrh::novi(18.69, 45.55))),
            })
            .unwrap();
        }
        d.spremi(u.projekt()).unwrap();
        let c = Connection::open(&put).unwrap();
        c.execute("UPDATE gpkg_geometry_columns SET z=?1", [deklaracija])
            .unwrap();
        let t = format!("sloj_{}", s.0);
        let blob: Vec<u8> = c
            .query_row(&format!("SELECT geom FROM {t} WHERE fid=1"), [], |r| {
                r.get(0)
            })
            .unwrap();
        let mut u = Uredjivac::iz_projekta(d.ucitaj().unwrap());
        let Some(Novo::Objekt(novi)) = u
            .izvrsi(NaredbaObjekta::Dodaj {
                sloj: s,
                naziv: "Druga dimenzija".into(),
                geometrija: Geometrija::Tocka(Vrh {
                    duzina: 18.7,
                    sirina: 45.6,
                    visina,
                }),
            })
            .unwrap()
        else {
            panic!()
        };
        d.spremi(u.projekt()).unwrap();
        assert_eq!(
            c.query_row("SELECT z FROM gpkg_geometry_columns", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            c.query_row(&format!("SELECT geom FROM {t} WHERE fid=1"), [], |r| r
                .get::<_, Vec<u8>>(0))
                .unwrap(),
            blob
        );
        let p = d.ucitaj().unwrap();
        let Geometrija::Tocka(v) = p.objekt(s, novi).unwrap().geometrija else {
            panic!()
        };
        assert_eq!(v.visina, visina);
    }
}
