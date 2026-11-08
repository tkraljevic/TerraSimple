#![cfg(test)]
//! Odbijanje stvarnih SQLite operacija: greška ne smije ostaviti pola projekta.
use super::*;
use rusqlite::hooks::{AuthContext, Authorization};
use slojevi::*;
fn prazna() -> Datoteka {
    let c = Connection::open_in_memory().unwrap();
    shema::stvori(&c).unwrap();
    geometrija::funkcije(&c).unwrap();
    let verzija = c
        .query_row("PRAGMA data_version", [], |r| r.get(0))
        .unwrap();
    Datoteka {
        veza: c,
        prethodni: Projekt::default(),
        verzija,
        procitano: true,
        upozorenja: Vec::new(),
        trajanje_upisa: std::time::Duration::ZERO,
    }
}
pub(super) fn model() -> Uredjivac {
    let mut u = Uredjivac::novi();
    let Some(Novo::Mapa(m)) = u
        .izvrsi(NaredbaStabla::DodajMapu {
            u: None,
            naziv: "Mapa".into(),
        })
        .unwrap()
    else {
        panic!()
    };
    for i in 0..2 {
        let Some(Novo::Sloj(s)) = u
            .izvrsi(NaredbaStabla::DodajSloj {
                u: Some(m),
                naziv: format!("Sloj {i}"),
                vrsta: VrstaGeometrije::Tocka,
            })
            .unwrap()
        else {
            panic!()
        };
        u.izvrsi(NaredbaObjekta::Dodaj {
            sloj: s,
            naziv: "Objekt".into(),
            geometrija: Geometrija::Tocka(Vrh::novi(18., 45.)),
        })
        .unwrap();
    }
    u
}
fn odbij(c: &mut Connection, n: usize) {
    let mut broj = 0;
    c.authorizer(Some(move |_: AuthContext<'_>| {
        let odluka = if broj == n {
            Authorization::Deny
        } else {
            Authorization::Allow
        };
        broj += 1;
        odluka
    }))
    .unwrap();
}
fn dopusti(c: &mut Connection) {
    c.authorizer(Some(|_: AuthContext<'_>| Authorization::Allow))
        .unwrap();
}
#[test]
fn sql_greske_ne_pretvaraju_se_u_prolaz() {
    let mut padova = 0;
    for n in 0..160 {
        let mut d = prazna();
        let u = model();
        d.spremi(u.projekt()).unwrap();
        odbij(&mut d.veza, n);
        if d.ucitaj().is_err() {
            padova += 1;
        }
    }
    assert!(padova > 25, "{padova}");
}
#[test]
fn svaki_prekinuti_upis_vraca_staro_stanje() {
    let mut padova = 0;
    for n in 0..300 {
        let mut d = prazna();
        let u = model();
        d.spremi(u.projekt()).unwrap();
        let staro = d.ucitaj().unwrap();
        let mut u = Uredjivac::iz_projekta(staro.clone());
        let (s, o) = staro
            .slojevi()
            .find_map(|(s, l)| l.objekti.keys().next().map(|o| (s, *o)))
            .unwrap();
        u.izvrsi(NaredbaObjekta::Izmijeni {
            sloj: s,
            objekt: o,
            izmjena: IzmjenaObjekta::Naziv("Novo".into()),
        })
        .unwrap();
        u.izvrsi(NaredbaStabla::PostaviStil {
            sloj: s,
            stil: Stil {
                debljina_mm: 2.,
                ..Stil::default()
            },
        })
        .unwrap();
        let obrisi = staro.slojevi().last().unwrap().0;
        u.izvrsi(NaredbaStabla::Obrisi {
            stavka: Stavka::Sloj(obrisi),
        })
        .unwrap();
        odbij(&mut d.veza, n);
        let rezultat = d.spremi(u.projekt());
        dopusti(&mut d.veza);
        if rezultat.is_err() {
            padova += 1;
            assert_eq!(d.ucitaj().unwrap(), staro, "SQL poziv {n}");
        } else {
            let procitan = d.ucitaj().unwrap();
            assert_eq!(
                procitan.slojevi().collect::<Vec<_>>(),
                u.projekt().slojevi().collect::<Vec<_>>()
            );
            assert_eq!(procitan.korijen(), u.projekt().korijen());
            assert_eq!(procitan.mapa(IdMape(1)), u.projekt().mapa(IdMape(1)));
        }
    }
    assert!(padova > 30, "{padova}");
}
#[test]
fn kvar_stvaranja_i_brojanja_tablica_je_greska() {
    let mut padova = 0;
    for n in 0..180 {
        let mut c = Connection::open_in_memory().unwrap();
        odbij(&mut c, n);
        if shema::stvori(&c).is_err() {
            padova += 1;
        }
    }
    assert!(padova > 25);
}

#[test]
fn prekid_tijekom_citanja_vraca_gresku() {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    let mut prekinuto = 0;
    for n in 0..500 {
        let mut d = prazna();
        let u = model();
        d.spremi(u.projekt()).unwrap();
        let broj = Arc::new(AtomicUsize::new(0));
        d.veza
            .progress_handler(1, Some(move || broj.fetch_add(1, Ordering::Relaxed) == n))
            .unwrap();
        if d.ucitaj().is_err() {
            prekinuto += 1;
        }
    }
    assert!(prekinuto > 100, "{prekinuto}");
}

#[test]
#[ignore = "Pokreće ga roditeljski test, jer namjerno prekida proces."]
fn podproces_spremanja() {
    let put = std::env::var("TERRASIMPLE_PREKID_PUT").unwrap();
    let mut d = Datoteka::otvori(put).unwrap();
    let p = d.ucitaj().unwrap();
    let (s, o) = p
        .slojevi()
        .find_map(|(s, l)| l.objekti.keys().next().map(|o| (s, *o)))
        .unwrap();
    let f = rusqlite::functions::FunctionFlags::SQLITE_UTF8;
    d.veza
        .create_scalar_function("prekini_proces", 0, f, |_| -> rusqlite::Result<i64> {
            std::process::exit(73)
        })
        .unwrap();
    d.veza.execute_batch(&format!("CREATE TEMP TRIGGER prekid BEFORE UPDATE ON sloj_{} BEGIN SELECT prekini_proces(); END;",s.0)).unwrap();
    let mut u = Uredjivac::iz_projekta(p);
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Naziv("Nedovršeno".into()),
    })
    .unwrap();
    d.spremi(u.projekt()).unwrap();
    panic!("Proces nije prekinut unutar upisa.");
}
#[test]
fn prekid_procesa_usred_stvarnog_upisa_ne_gubi_projekt() {
    let put = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../.quality/prekid-procesa-{}.gpkg",
        std::process::id()
    ));
    std::fs::create_dir_all(put.parent().unwrap()).unwrap();
    if put.exists() {
        std::fs::remove_file(&put).unwrap();
    }
    let mut d = Datoteka::otvori(&put).unwrap();
    let u = model();
    d.spremi(u.projekt()).unwrap();
    let staro = d.ucitaj().unwrap();
    drop(d);
    let izlaz = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "kvarovi::podproces_spremanja", "--ignored"])
        .env("TERRASIMPLE_PREKID_PUT", &put)
        .output()
        .unwrap();
    assert_eq!(
        izlaz.status.code(),
        Some(73),
        "{}",
        String::from_utf8_lossy(&izlaz.stderr)
    );
    let mut d = Datoteka::otvori(put).unwrap();
    assert_eq!(d.ucitaj().unwrap(), staro);
    assert_eq!(
        d.veza
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
fn kvarovi_prvog_spremanja_ne_ostavljaju_dio_novih_slojeva() {
    let mut padova = 0;
    for n in 0..240 {
        let mut d = prazna();
        let u = model();
        odbij(&mut d.veza, n);
        let r = d.spremi(u.projekt());
        dopusti(&mut d.veza);
        if r.is_err() {
            padova += 1;
            assert_eq!(d.ucitaj().unwrap(), Projekt::default());
        }
    }
    assert!(padova > 30);
}

#[test]
fn pogreske_pripreme_veze_i_kopiranja_nisu_uspjeh() {
    let mut c = Connection::open_in_memory().unwrap();
    odbij(&mut c, 0);
    assert!(Datoteka::iz_veze(c).is_err());
    let mut d = prazna();
    let u = model();
    d.spremi(u.projekt()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.quality");
    assert!(
        d.spremi_kao(root.join("nepostojeca-mapa/izlaz.gpkg"), u.projekt())
            .is_err()
    );
    let mut stmt = d.veza.prepare("SELECT ST_IsEmpty(NULL)").unwrap();
    let mut red = stmt.query([]).unwrap();
    red.next().unwrap();
    assert!(pripremi_vezu(&d.veza).is_err());
    drop(red);
    drop(stmt);
    let put = root.join(format!("pogresna-kopija-{}.gpkg", std::process::id()));
    if put.exists() {
        std::fs::remove_file(&put).unwrap();
    }
    d.veza.execute_batch("UPDATE gpkg_spatial_ref_sys SET organization='NONE',definition='nepoznato' WHERE srs_id=4326").unwrap();
    assert!(d.spremi_kao(&put, u.projekt()).is_err());
}

#[test]
fn nevaljana_izvorna_shema_odbija_kopiju() {
    let mut d = prazna();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.quality");
    let put = root.join(format!("bez-sheme-{}.gpkg", std::process::id()));
    if put.exists() {
        std::fs::remove_file(&put).unwrap();
    }
    d.veza.execute_batch("DROP TABLE gpkg_contents").unwrap();
    assert!(d.spremi_kao(put, &Projekt::default()).is_err());
}
