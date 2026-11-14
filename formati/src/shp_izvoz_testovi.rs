use super::*;
use crate::model::GeometrijaKml;
use slojevi::{IdMape, IzmjenaObjekta, NaredbaObjekta, NaredbaStabla, Uredjivac, ZapisObjekta};

fn mapa(ime: &str) -> PathBuf {
    let m = std::env::temp_dir().join(format!("terrasimple-shp-{ime}-{}", std::process::id()));
    std::fs::create_dir_all(&m).unwrap();
    m
}

fn sloj(u: &mut Uredjivac, mapa: Option<IdMape>, naziv: &str, vrsta: VrstaGeometrije) -> IdSloja {
    u.izvrsi(NaredbaStabla::DodajSloj {
        u: mapa,
        naziv: naziv.into(),
        vrsta,
    })
    .unwrap();
    u.projekt().slojevi().map(|(id, _)| id).max().unwrap()
}

fn dodaj(u: &mut Uredjivac, s: IdSloja, naziv: &str, g: Geometrija) -> slojevi::IdObjekta {
    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: naziv.into(),
        geometrija: g,
    })
    .unwrap();
    *u.projekt().sloj(s).unwrap().objekti.keys().max().unwrap()
}

fn kvadrat(x: f64, y: f64, a: f64, uzlazno: bool) -> Vec<Vrh> {
    let mut r = vec![
        Vrh::novi(x, y),
        Vrh::novi(x + a, y),
        Vrh::novi(x + a, y + a),
        Vrh::novi(x, y + a),
    ];
    if !uzlazno {
        r.reverse();
    }
    r
}

#[test]
fn sloj_ide_u_htrs96_i_cita_se_natrag_na_isto_mjesto() {
    let mut u = Uredjivac::novi();
    let s = sloj(&mut u, None, "Obuhvati", VrstaGeometrije::Poligon);
    let mut vanjski = kvadrat(18.6, 45.5, 0.1, true);
    vanjski.push(vanjski[0]);
    let o = dodaj(
        &mut u,
        s,
        "Retencija Čađavica",
        Geometrija::Poligon(vec![vanjski, kvadrat(18.62, 45.52, 0.02, false)]),
    );
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Opis("ž".repeat(200)),
    })
    .unwrap();
    let m = mapa("lik");
    let staze = izvezi_shp(
        u.projekt(),
        OpsegIzvoza::Projekt,
        Sustav::Htrs96Tm,
        &m.join("obuhvat.shp"),
    )
    .unwrap();
    assert_eq!(staze, vec![m.join("obuhvat.shp")]);
    let prj_tekst = std::fs::read_to_string(m.join("obuhvat.prj")).unwrap();
    assert_eq!(crate::sustav_iz_prj(&prj_tekst), Some(Sustav::Htrs96Tm));
    assert_eq!(
        std::fs::read_to_string(m.join("obuhvat.cpg")).unwrap(),
        "UTF-8"
    );

    let d = crate::procitaj(&m.join("obuhvat.shp")).unwrap();
    let o = &d.korijen.objekti[0];
    assert_eq!(o.naziv, "Retencija Čađavica");
    assert_eq!(
        o.opis.chars().count(),
        127,
        "opis skraćen na 254 bajta bez lomljenja slova"
    );
    let GeometrijaKml::Poligon(p) = &o.geometrije[0] else {
        panic!("lik");
    };
    assert_eq!(p.len(), 2, "rupa ostaje rupa");
    assert!(
        p[0].iter()
            .any(|v| (v.duzina - 18.7).abs() < 1e-7 && (v.sirina - 45.6).abs() < 1e-7)
    );
    std::fs::remove_dir_all(&m).unwrap();
}

#[test]
fn vise_slojeva_dobiva_vise_datoteka_s_nazivom_sloja() {
    let mut u = Uredjivac::novi();
    u.izvrsi(NaredbaStabla::DodajMapu {
        u: None,
        naziv: "Teren".into(),
    })
    .unwrap();
    let m_id = IdMape(u.projekt().najveci_id());
    let a = sloj(&mut u, Some(m_id), "Crpke 1/2", VrstaGeometrije::Tocka);
    dodaj(&mut u, a, "C1", Geometrija::Tocka(Vrh::novi(18.6, 45.5)));
    let b = sloj(&mut u, Some(m_id), "Crpke 1 2", VrstaGeometrije::Tocka);
    dodaj(&mut u, b, "C2", Geometrija::Tocka(Vrh::novi(18.7, 45.6)));
    let c = sloj(&mut u, Some(m_id), "Kanali", VrstaGeometrije::Linija);
    let crta = vec![Vrh::novi(18.6, 45.5), Vrh::novi(18.7, 45.6)];
    dodaj(&mut u, c, "K1", Geometrija::Linija(crta));
    sloj(&mut u, Some(m_id), "Prazan", VrstaGeometrije::Tocka);
    let m = mapa("vise");
    let staze = izvezi_shp(
        u.projekt(),
        OpsegIzvoza::Projekt,
        Sustav::Wgs84,
        &m.join("teren.shp"),
    )
    .unwrap();
    let imena: Vec<String> = staze
        .iter()
        .map(|s| s.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        imena,
        [
            "teren_Crpke_1_2.shp",
            &format!("teren_Crpke_1_2_{}.shp", b.0),
            "teren_Kanali.shp"
        ]
    );
    let d = crate::procitaj(&staze[2]).unwrap();
    assert!(
        matches!(&d.korijen.objekti[0].geometrije[0], GeometrijaKml::Linija(v) if v.len() == 2)
    );
    let samo = OpsegIzvoza::Stavka(Stavka::Sloj(a));
    let jedan = izvezi_shp(u.projekt(), samo, Sustav::Wgs84, &m.join("crpke.shp")).unwrap();
    assert_eq!(jedan, vec![m.join("crpke.shp")]);
    assert!(izvezi_shp(u.projekt(), samo, Sustav::Wgs84, &m.join("nema/x.shp")).is_err());
    assert_eq!(ime("§"), "_");
    assert_eq!(ime(""), "sloj");
    std::fs::remove_dir_all(&m).unwrap();
}

#[test]
fn stupci_iz_geopackagea_idu_u_dbf_s_kratkim_nazivima() {
    let mut u = Uredjivac::novi();
    let s = sloj(&mut u, None, "Bunari", VrstaGeometrije::Tocka);
    let o = dodaj(&mut u, s, "B1", Geometrija::Tocka(Vrh::novi(18.0, 45.0)));
    let p = u.projekt();
    let mut zapis = ZapisObjekta {
        geometrija: None,
        fid: 1,
        nepoznati: Default::default(),
    };
    zapis
        .nepoznati
        .insert("dubina_bunara_m".into(), Vrijednost::Broj(12.5));
    zapis
        .nepoznati
        .insert("dubina_bunara_x".into(), Vrijednost::Cijeli(3));
    zapis
        .nepoznati
        .insert("naziv".into(), Vrijednost::Tekst("dvostruko".into()));
    zapis.nepoznati.insert("§§".into(), Vrijednost::Prazno);
    zapis
        .nepoznati
        .insert("slika".into(), Vrijednost::Bajtovi(vec![1]));
    let mut zapisi = p.zapisi().clone();
    zapisi.slojevi.insert(
        s,
        slojevi::IzvorSloja {
            srs_id: 4326,
            sql: String::new(),
            stupci: vec![],
            indeksi: vec![],
            okidaci: vec![],
            stil: slojevi::Stil::default(),
            sustav: Sustav::Wgs84,
            tablica: "bunari".into(),
            fid_stupac: "fid".into(),
            geometrija_stupac: "geom".into(),
            objekti: [(o, zapis)].into(),
            nepodrzani: 0,
        },
    );
    let p = Projekt::iz_zapisa(
        p.korijen().to_vec(),
        Default::default(),
        p.slojevi().map(|(i, s)| (i, s.clone())).collect(),
        zapisi,
    )
    .unwrap();
    let nazivi: Vec<String> = polja(&p, s).into_iter().map(|(n, _)| n).collect();
    assert_eq!(
        nazivi,
        [
            "NAZIV",
            "OPIS",
            "dubinabuna",
            "POLJE1",
            "POLJE2",
            "slika",
            "POLJE3"
        ]
    );
    let m = mapa("dbf");
    izvezi_shp(&p, OpsegIzvoza::Projekt, Sustav::Wgs84, &m.join("b.shp")).unwrap();
    let t = crate::dbf::procitaj(&std::fs::read(m.join("b.dbf")).unwrap()).unwrap();
    assert_eq!(t.vrijednost(0, &["dubinabuna"]), Some("12.5"));
    assert_eq!(t.vrijednost(0, &["POLJE1"]), Some("3"));
    assert_eq!(t.vrijednost(0, &["POLJE2"]), Some("dvostruko"));
    assert_eq!(t.vrijednost(0, &["NAZIV"]), Some("B1"));
    assert_eq!(t.vrijednost(0, &["slika"]), None);
    std::fs::remove_dir_all(&m).unwrap();
}

#[test]
fn prazan_okvir_i_svi_sustavi() {
    let w = Pisac {
        shp: Vec::new(),
        shx: Vec::new(),
        okvir: [
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        ],
    };
    let z = w.zaglavlje(1, 0);
    assert_eq!(z.len(), 100);
    for s in [
        Sustav::Wgs84,
        Sustav::WebMercator,
        Sustav::Htrs96Tm,
        Sustav::HdksZona5,
        Sustav::HdksZona6,
    ] {
        assert_eq!(crate::sustav_iz_prj(prj(s)), Some(s));
    }
    assert!(
        napisi_sloj(
            &Projekt::default(),
            IdSloja(9),
            Sustav::Wgs84,
            Path::new("x.shp")
        )
        .is_ok()
    );
}

#[test]
fn prsten_dobiva_smjer_i_zatvaranje() {
    let kazaljka = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]];
    let r = prsten(kazaljka.clone(), true);
    assert_eq!(r[..4], kazaljka[..], "vanjski u smjeru kazaljke ostaje");
    assert_eq!(r.len(), 5);
    let rupa = prsten(kazaljka.clone(), false);
    assert_eq!(rupa[0], [1.0, 0.0], "rupa se okreće");
    let mut zatvoren = kazaljka;
    zatvoren.push([0.0, 0.0]);
    assert_eq!(
        prsten(zatvoren, true).len(),
        5,
        "zatvoren se ne zatvara opet"
    );
}
