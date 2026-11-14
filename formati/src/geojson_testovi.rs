use super::*;
use crate::uvoz::{NaziviSlojeva, uvezi};
use slojevi::{IdMape, IzmjenaObjekta, NaredbaObjekta, NaredbaStabla, Uredjivac, VrstaGeometrije};

fn citaj(t: &str) -> DokumentKml {
    procitaj(t.as_bytes(), None).unwrap()
}

#[test]
fn zbirka_s_nazivima_stilom_i_skupnim_geometrijama() {
    let d = citaj(
        r##"{"type":"FeatureCollection","name":"Kanali","features":[
        {"type":"Feature","properties":{"Name":"Kanal A","opis":"Glavni","stroke":"#ff0000","stroke-width":3,"stroke-opacity":0.5,"duljina":12},
         "geometry":{"type":"LineString","coordinates":[[18.6,45.5],[18.7,45.6,90]]}},
        {"type":"Feature","properties":{"naziv":"Crpke","marker-color":"#00ff00","marker-size":"large"},
         "geometry":{"type":"MultiPoint","coordinates":[[18.6,45.5],[18.61,45.51]]}},
        {"type":"Feature","properties":{"name":"Jezero","fill":"#0000ff","fill-opacity":0.25},
         "geometry":{"type":"MultiPolygon","coordinates":[[[[18,45],[18.1,45],[18.1,45.1],[18,45]],[[18.02,45.02],[18.04,45.02],[18.04,45.04]]]]}},
        {"type":"Feature","properties":{"name":"Zbirka"},
         "geometry":{"type":"GeometryCollection","geometries":[{"type":"Point","coordinates":[18,45]},{"type":"MultiLineString","coordinates":[[[18,45],[18.1,45.1]]]}]}},
        {"type":"Feature","properties":{"name":"Bez"},"geometry":null}
        ]}"##,
    );
    assert_eq!(d.korijen.naziv, "Kanali");
    let o = &d.korijen.objekti;
    assert_eq!(o.len(), 4);
    assert_eq!(
        (o[0].naziv.as_str(), o[0].opis.as_str()),
        ("Kanal A", "Glavni")
    );
    let crta = o[0].stil.boja_crte.unwrap();
    assert_eq!((crta.r, crta.a), (255, 128));
    assert_eq!(o[0].stil.sirina_px, Some(3.0));
    assert!(matches!(&o[0].geometrije[0], GeometrijaKml::Linija(v) if v[1].visina == Some(90.0)));
    assert_eq!(o[1].geometrije.len(), 2);
    assert_eq!(o[1].stil.mjerilo_ikone, Some(1.5));
    assert!(
        matches!(&o[2].geometrije[0], GeometrijaKml::Poligon(p) if p.len() == 2 && p[0].len() == 3)
    );
    assert_eq!(o[2].stil.boja_ispune.map(|b| b.a), Some(64));
    assert_eq!(o[3].geometrije.len(), 2);
    assert_eq!(d.gubici.bez_geometrije, 1);
    assert_eq!(
        d.gubici.izostavljena_svojstva.iter().collect::<Vec<_>>(),
        ["duljina"]
    );
}

#[test]
fn losi_dijelovi_se_broje_a_ne_ruse_uvoz() {
    let d = citaj(
        r#"{"type":"FeatureCollection","features":[
        {"type":"Feature","geometry":{"type":"LineString","coordinates":[[18,45]]}},
        {"type":"Feature","geometry":{"type":"LineString","coordinates":[[18,45],[500,45]]}},
        {"type":"Feature","geometry":{"type":"Polygon","coordinates":[[[18,45],[18.1,45]]]}},
        {"type":"Feature","geometry":{"type":"Point","coordinates":["a",45]}},
        {"type":"Feature","geometry":{"type":"Krug","coordinates":[18,45]}},
        {"type":"Feature","geometry":{"type":"Point","coordinates":[18]}},
        {"type":"Feature","geometry":{"type":"Point","coordinates":"x"}},
        {"type":"Feature","geometry":{"type":"LineString","coordinates":"x"}},
        {"type":"Feature"}
        ]}"#,
    );
    assert!(d.korijen.objekti.is_empty());
    let g = &d.gubici;
    assert_eq!(g.bez_geometrije, 9);
    assert_eq!(g.neispravne_koordinate, 5);
    assert_eq!(g.prazne_geometrije, 3);
}

#[test]
fn jedan_objekt_gola_geometrija_i_greske() {
    let d = citaj(
        r#"{"type":"Feature","properties":{"title":"T"},"geometry":{"type":"Point","coordinates":[18,45]}}"#,
    );
    assert_eq!(d.korijen.objekti[0].naziv, "T");
    let d = citaj(r#"{"type":"Point","coordinates":[18,45]}"#);
    assert_eq!(d.korijen.objekti.len(), 1);
    assert_eq!(
        procitaj(b"{nije json", None).unwrap_err(),
        GreskaKml::NeispravanJson
    );
    assert_eq!(
        procitaj(b"{\"a\":1}", None).unwrap_err(),
        GreskaKml::NeispravanJson
    );
    let d = citaj(r#"{"type":"FeatureCollection"}"#);
    assert!(d.korijen.objekti.is_empty());
}

#[test]
fn stari_crs_se_preracunava_a_nepoznati_odbija() {
    let htrs = r#"{"type":"FeatureCollection","crs":{"type":"name","properties":{"name":"urn:ogc:def:crs:EPSG::3765"}},
        "features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[661000,5047000]}}]}"#;
    let d = citaj(htrs);
    let GeometrijaKml::Tocka(v) = d.korijen.objekti[0].geometrije[0] else {
        panic!("točka");
    };
    assert!(
        (18.5..18.8).contains(&v.duzina) && (45.4..45.6).contains(&v.sirina),
        "{v:?}"
    );
    let wgs = htrs
        .replace("EPSG::3765", "OGC:1.3:CRS84")
        .replace("661000,5047000", "18,45");
    assert_eq!(citaj(&wgs).korijen.objekti.len(), 1);
    let nepoznat = htrs.replace("3765", "31276");
    assert_eq!(
        procitaj(nepoznat.as_bytes(), None).unwrap_err(),
        GreskaKml::NepoznatSustav
    );
    let los = htrs.replace("EPSG::3765", "nesto");
    assert_eq!(
        procitaj(los.as_bytes(), None).unwrap_err(),
        GreskaKml::NepoznatSustav
    );
}

#[test]
fn boje_i_velicine_znaka() {
    assert_eq!(
        boja(Some("#0a0b0c"), None).map(|b| (b.r, b.g, b.b, b.a)),
        Some((10, 11, 12, 255))
    );
    assert!(boja(Some("crvena"), None).is_none());
    assert!(boja(Some("#12"), None).is_none());
    assert!(boja(Some("#zz0000"), None).is_none());
    let mut p = Map::new();
    for (velicina, mjerilo) in [
        ("small", Some(0.75)),
        ("medium", Some(1.0)),
        ("golem", None),
    ] {
        p.insert("marker-size".into(), json!(velicina));
        assert_eq!(stil(&p).mjerilo_ikone, mjerilo);
    }
}

fn nazivi() -> NaziviSlojeva {
    NaziviSlojeva {
        tocke: "Točke".into(),
        linije: "Linije".into(),
        poligoni: "Poligoni".into(),
        bez_naziva: "Bez naziva".into(),
    }
}

#[test]
fn izvoz_i_ponovni_uvoz_cuvaju_nazive_geometriju_i_mapu() {
    let mut u = Uredjivac::novi();
    u.izvrsi(NaredbaStabla::DodajMapu {
        u: None,
        naziv: "Teren".into(),
    })
    .unwrap();
    let m = IdMape(u.projekt().najveci_id());
    let dodaj_sloj = |u: &mut Uredjivac, naziv: &str, vrsta| {
        u.izvrsi(NaredbaStabla::DodajSloj {
            u: Some(m),
            naziv: naziv.into(),
            vrsta,
        })
        .unwrap();
        u.projekt().slojevi().map(|(id, _)| id).max().unwrap()
    };
    let dodaj = |u: &mut Uredjivac, sloj, naziv: &str, g| {
        u.izvrsi(NaredbaObjekta::Dodaj {
            sloj,
            naziv: naziv.into(),
            geometrija: g,
        })
        .unwrap();
        *u.projekt()
            .sloj(sloj)
            .unwrap()
            .objekti
            .keys()
            .max()
            .unwrap()
    };
    let linije = dodaj_sloj(&mut u, "Kanali", VrstaGeometrije::Linija);
    let mut v = Vrh::novi(18.7, 45.6);
    v.visina = Some(91.5);
    let k = dodaj(
        &mut u,
        linije,
        "Kanal",
        Geometrija::Linija(vec![Vrh::novi(18.6, 45.5), v]),
    );
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: linije,
        objekt: k,
        izmjena: IzmjenaObjekta::Opis("Opis".into()),
    })
    .unwrap();
    let tocke = dodaj_sloj(&mut u, "Crpke", VrstaGeometrije::Tocka);
    dodaj(
        &mut u,
        tocke,
        "Crpka",
        Geometrija::Tocka(Vrh::novi(18.65, 45.55)),
    );
    let likovi = dodaj_sloj(&mut u, "Jezera", VrstaGeometrije::Poligon);
    let prsten = vec![
        Vrh::novi(18.0, 45.0),
        Vrh::novi(18.1, 45.0),
        Vrh::novi(18.1, 45.1),
    ];
    dodaj(&mut u, likovi, "Jezero", Geometrija::Poligon(vec![prsten]));
    let mut stil = u.projekt().sloj(likovi).unwrap().stil;
    stil.vrsta_ispune = VrstaIspune::Bez;
    u.izvrsi(NaredbaStabla::PostaviStil { sloj: likovi, stil })
        .unwrap();

    let tekst = napisi(u.projekt(), OpsegIzvoza::Projekt, "Teren");
    let v: Value = serde_json::from_str(&tekst).unwrap();
    assert_eq!(v["features"].as_array().unwrap().len(), 3);
    assert_eq!(v["features"][0]["properties"]["mapa"], "Teren");
    assert_eq!(v["features"][0]["properties"]["sloj"], "Kanali");
    assert_eq!(v["features"][2]["properties"]["fill-opacity"], 0.0);
    assert_eq!(
        v["features"][2]["geometry"]["coordinates"][0]
            .as_array()
            .unwrap()
            .len(),
        4
    );

    let d = procitaj(tekst.as_bytes(), None).unwrap();
    let mut novi = Uredjivac::novi();
    let s = uvezi(&mut novi, &d, "Teren", &nazivi()).unwrap();
    assert_eq!(s.objekata, 3);
    let kanal = novi
        .projekt()
        .slojevi()
        .flat_map(|(_, s)| s.objekti.values())
        .find(|o| o.naziv == "Kanal")
        .unwrap();
    assert_eq!(kanal.opis, "Opis");
    assert!(matches!(&kanal.geometrija, Geometrija::Linija(v) if v[1].visina == Some(91.5)));
    assert!(d.gubici.izostavljena_svojstva.contains("mapa"));

    let jedan = napisi(
        u.projekt(),
        OpsegIzvoza::Stavka(Stavka::Sloj(tocke)),
        "Crpke",
    );
    let v: Value = serde_json::from_str(&jedan).unwrap();
    assert_eq!(v["features"][0]["properties"]["mapa"], "");
    assert_eq!(v["features"][0]["properties"]["marker-color"], "#e65100");
    let nema = napisi(
        u.projekt(),
        OpsegIzvoza::Stavka(Stavka::Mapa(IdMape(999))),
        "x",
    );
    assert!(nema.contains("\"features\":[]"));
    let nema = napisi(
        u.projekt(),
        OpsegIzvoza::Stavka(Stavka::Sloj(IdSloja(999))),
        "x",
    );
    assert!(nema.contains("\"features\":[]"));
}

#[test]
fn vrijednosti_stupaca() {
    assert_eq!(u_vrijednost(&Vrijednost::Prazno), Some(Value::Null));
    assert_eq!(u_vrijednost(&Vrijednost::Cijeli(3)), Some(json!(3)));
    assert_eq!(u_vrijednost(&Vrijednost::Broj(2.5)), Some(json!(2.5)));
    assert_eq!(
        u_vrijednost(&Vrijednost::Tekst("a".into())),
        Some(json!("a"))
    );
    assert_eq!(u_vrijednost(&Vrijednost::Bajtovi(vec![1])), None);
    let zatvoren = [
        Vrh::novi(18.0, 45.0),
        Vrh::novi(18.1, 45.0),
        Vrh::novi(18.0, 45.0),
    ];
    assert_eq!(prsten(&zatvoren).as_array().unwrap().len(), 3);
}

#[test]
fn datoteka_se_prepoznaje_po_sadrzaju_i_s_bom_om() {
    let mapa = std::env::temp_dir().join(format!("terrasimple-geojson-{}", std::process::id()));
    std::fs::create_dir_all(&mapa).unwrap();
    let staza = mapa.join("krivi_nastavak.kml");
    let mut sadrzaj = b"\xEF\xBB\xBF\n  ".to_vec();
    sadrzaj.extend_from_slice(br#"{"type":"Point","coordinates":[18,45]}"#);
    std::fs::write(&staza, sadrzaj).unwrap();
    let d = crate::procitaj(&staza).unwrap();
    assert_eq!(d.korijen.objekti.len(), 1);
    std::fs::remove_dir_all(&mapa).unwrap();
}

#[test]
fn stupci_iz_geopackagea_idu_u_svojstva() {
    let mut u = Uredjivac::novi();
    u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Bunari".into(),
        vrsta: VrstaGeometrije::Tocka,
    })
    .unwrap();
    let s = u.projekt().slojevi().next().unwrap().0;
    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: "B1".into(),
        geometrija: Geometrija::Tocka(Vrh::novi(18.0, 45.0)),
    })
    .unwrap();
    let p = u.projekt();
    let o = *p.sloj(s).unwrap().objekti.keys().next().unwrap();
    let mut zapisi = p.zapisi().clone();
    let mut zapis = slojevi::ZapisObjekta {
        geometrija: None,
        fid: 1,
        nepoznati: Default::default(),
    };
    zapis
        .nepoznati
        .insert("dubina".into(), Vrijednost::Broj(12.5));
    zapis
        .nepoznati
        .insert("name".into(), Vrijednost::Tekst("ne gazi naziv".into()));
    zapis
        .nepoznati
        .insert("slika".into(), Vrijednost::Bajtovi(vec![1]));
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
    let v: Value = serde_json::from_str(&napisi(&p, OpsegIzvoza::Projekt, "B")).unwrap();
    let svojstva = &v["features"][0]["properties"];
    assert_eq!(svojstva["dubina"], 12.5);
    assert_eq!(svojstva["name"], "B1");
    assert!(svojstva.get("slika").is_none());
}

#[test]
fn bez_crs_s_metrima_trazi_sustav_a_sa_zadanim_preracunava() {
    let t = br#"{"type":"FeatureCollection","features":[{"type":"Feature","geometry":{"type":"Point","coordinates":[661000,5047000]}}]}"#;
    assert_eq!(
        procitaj(t, None).unwrap_err(),
        GreskaKml::BezSustava {
            istok: 661_000,
            sjever: 5_047_000
        }
    );
    let d = procitaj(t, Some(Sustav::Htrs96Tm)).unwrap();
    assert_eq!(d.korijen.objekti.len(), 1);
    let stupnjevi = br#"{"type":"Point","coordinates":[18,45]}"#;
    assert_eq!(
        procitaj(stupnjevi, Some(Sustav::Wgs84))
            .unwrap()
            .korijen
            .objekti
            .len(),
        1
    );
    assert_eq!(prvi_polozaj(&json!({"type": "Point"})), None);
    assert_eq!(prvi_polozaj(&json!([1])), None);
    assert_eq!(prvi_polozaj(&json!(["a", 1])), None);
}
