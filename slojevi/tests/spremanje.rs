use slojevi::*;
use std::collections::BTreeMap;
fn prazni_sloj() -> Sloj {
    Sloj {
        naziv: "Pokus".into(),
        roditelj: None,
        vrsta: VrstaGeometrije::Tocka,
        stanje: StanjeSloja::Uredivo,
        stil: Stil::default(),
        objekti: BTreeMap::new(),
    }
}
#[test]
fn nevaljano_stablo_i_objekt_ne_ulaze_u_uredjivac() {
    let m = Mapa {
        naziv: "Mapa".into(),
        roditelj: None,
        djeca: Vec::new(),
    };
    assert!(
        Projekt::iz_zapisa(
            vec![Stavka::Mapa(IdMape(1)), Stavka::Mapa(IdMape(1))],
            BTreeMap::from([(IdMape(1), m.clone())]),
            BTreeMap::new(),
            Zapisi::default()
        )
        .is_err()
    );
    assert!(
        Projekt::iz_zapisa(
            Vec::new(),
            BTreeMap::from([(IdMape(1), m)]),
            BTreeMap::new(),
            Zapisi::default()
        )
        .is_err()
    );
    let mut s = prazni_sloj();
    let o = Objekt {
        id: IdObjekta(3),
        naziv: "Pokus".into(),
        opis: String::new(),
        geometrija: Geometrija::Tocka(Vrh::novi(18., 45.)),
        stil: StilObjekta::default(),
    };
    s.objekti.insert(IdObjekta(9), o.clone());
    assert!(
        Projekt::iz_zapisa(
            vec![Stavka::Sloj(IdSloja(2))],
            BTreeMap::new(),
            BTreeMap::from([(IdSloja(2), s.clone())]),
            Zapisi::default()
        )
        .is_err()
    );
    s.objekti.clear();
    s.objekti.insert(o.id, o);
    s.vrsta = VrstaGeometrije::Linija;
    assert!(
        Projekt::iz_zapisa(
            vec![Stavka::Sloj(IdSloja(2))],
            BTreeMap::new(),
            BTreeMap::from([(IdSloja(2), s)]),
            Zapisi::default()
        )
        .is_err()
    );
}
#[test]
fn ucitani_brojevi_se_ne_ponavljaju_a_ponistavanje_ostaje_promjena() {
    let p = Projekt::iz_zapisa(
        vec![Stavka::Sloj(IdSloja(8))],
        BTreeMap::new(),
        BTreeMap::from([(IdSloja(8), prazni_sloj())]),
        Zapisi::default(),
    )
    .unwrap();
    let mut u = Uredjivac::iz_projekta(p.clone());
    let Some(Novo::Objekt(id)) = u
        .izvrsi(NaredbaObjekta::Dodaj {
            sloj: IdSloja(8),
            naziv: "Novi".into(),
            geometrija: Geometrija::Tocka(Vrh::novi(18., 45.)),
        })
        .unwrap()
    else {
        panic!()
    };
    assert!(id.0 > 8);
    let spremljeno = u.projekt().clone();
    u.ponisti().unwrap();
    assert_eq!(u.projekt().izmjene_od(&spremljeno).obrisi.len(), 1);
    assert_eq!(u.projekt(), &p);
}

#[test]
fn oznaka_stanja_prati_spremanje_ponistavanje_i_ponavljanje() {
    let p = Projekt::iz_zapisa(
        vec![Stavka::Sloj(IdSloja(8))],
        BTreeMap::new(),
        BTreeMap::from([(IdSloja(8), prazni_sloj())]),
        Zapisi::default(),
    )
    .unwrap();
    let mut u = Uredjivac::iz_projekta(p);
    assert!(!u.je_li_izmijenjen());
    assert_eq!(u.oznaka_stanja(), 0);

    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: IdSloja(8),
        naziv: "T1".into(),
        geometrija: Geometrija::Tocka(Vrh::novi(18., 45.)),
    })
    .unwrap();
    assert!(u.je_li_izmijenjen());
    assert_ne!(u.oznaka_stanja(), 0);

    u.oznaci_spremljeno();
    assert!(!u.je_li_izmijenjen());

    u.ponisti().unwrap();
    assert!(u.je_li_izmijenjen());

    u.ponovi().unwrap();
    assert!(!u.je_li_izmijenjen());

    u.ponisti().unwrap();
    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: IdSloja(8),
        naziv: "T2".into(),
        geometrija: Geometrija::Tocka(Vrh::novi(19., 46.)),
    })
    .unwrap();
    assert!(u.je_li_izmijenjen());
}
