use super::*;
use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::datoteka::LazniBirac;
use crate::dijalozi::AktivniDijalog;
use crate::tema::Razmaci;
use eframe::egui::vec2;
use egui_kittest::{Harness, kittest::Queryable};
use slojevi::{Boja, Geometrija, Vrh};

fn put(ime: &str) -> PathBuf {
    let m = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.quality/testovi");
    std::fs::create_dir_all(&m).unwrap();
    let p = m.join(format!("{ime}-{}.gpkg", std::process::id()));
    let _ = std::fs::remove_file(&p);
    p
}

fn zadnji_sloj(u: &Uredjivac) -> IdSloja {
    u.projekt().slojevi().map(|(id, _)| id).max().unwrap()
}

fn dodaj_sloj(
    u: &mut Uredjivac,
    mapa: Option<IdMape>,
    naziv: &str,
    vrsta: VrstaGeometrije,
) -> IdSloja {
    u.izvrsi(NaredbaStabla::DodajSloj {
        u: mapa,
        naziv: naziv.into(),
        vrsta,
    })
    .unwrap();
    zadnji_sloj(u)
}

fn dodaj_objekt(
    u: &mut Uredjivac,
    sloj: IdSloja,
    naziv: &str,
    g: Geometrija,
) -> slojevi::IdObjekta {
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
}

/// GeoPackage kakav bi došao iz QGIS-a: mapa sa slojevima, objekt bez naziva sa stupcem `name`
/// i stupac `duljina` koji projekt ne pamti.
fn izvorna_datoteka(ime: &str) -> (PathBuf, IdSloja) {
    let mut u = Uredjivac::novi();
    u.izvrsi(NaredbaStabla::DodajMapu {
        u: None,
        naziv: "Sektor".into(),
    })
    .unwrap();
    let m = *u
        .projekt()
        .korijen()
        .iter()
        .find_map(|s| match s {
            Stavka::Mapa(m) => Some(m),
            Stavka::Sloj(_) => None,
        })
        .unwrap();
    let nasipi = dodaj_sloj(&mut u, Some(m), "nasipi", VrstaGeometrije::Linija);
    let crta =
        |x: f64| Geometrija::Linija(vec![Vrh::novi(18.6 + x, 45.5), Vrh::novi(18.7 + x, 45.51)]);
    let a = dodaj_objekt(&mut u, nasipi, "Nasip A", crta(0.0));
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: nasipi,
        objekt: a,
        izmjena: IzmjenaObjekta::Opis("Lijevi nasip".into()),
    })
    .unwrap();
    let crvena = Boja {
        r: 200,
        g: 0,
        b: 0,
        a: 255,
    };
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: nasipi,
        objekt: a,
        izmjena: IzmjenaObjekta::Stil(StilObjekta {
            boja_crte: Some(crvena),
            ..StilObjekta::default()
        }),
    })
    .unwrap();
    dodaj_objekt(&mut u, nasipi, "", crta(0.2));
    let objekti = dodaj_sloj(&mut u, Some(m), "objekti", VrstaGeometrije::Tocka);
    dodaj_objekt(
        &mut u,
        objekti,
        "Crpka",
        Geometrija::Tocka(Vrh::novi(18.65, 45.52)),
    );
    let obuhvati = dodaj_sloj(&mut u, None, "obuhvati", VrstaGeometrije::Poligon);
    let lik = vec![
        Vrh::novi(18.6, 45.5),
        Vrh::novi(18.7, 45.5),
        Vrh::novi(18.7, 45.6),
    ];
    dodaj_objekt(&mut u, obuhvati, "Obuhvat", Geometrija::Poligon(vec![lik]));
    let p = put(ime);
    geopaket::Datoteka::otvori(&p)
        .unwrap()
        .spremi(u.projekt())
        .unwrap();
    let c = rusqlite::Connection::open(&p).unwrap();
    let t = format!("sloj_{}", nasipi.0);
    c.execute_batch(&format!(
        "ALTER TABLE {t} ADD COLUMN NAME TEXT; ALTER TABLE {t} ADD COLUMN duljina REAL; UPDATE {t} SET NAME='Nasip B', duljina=12.5 WHERE naziv='';"
    ))
    .unwrap();
    (p, obuhvati)
}

#[test]
fn odabrani_slojevi_idu_u_novu_mapu_s_mapama_nazivima_i_stilom() {
    let k = KatalogPoruka::hrvatski();
    let (staza, obuhvati) = izvorna_datoteka("uvoz-izvor");
    let mut p = pripremi(&staza, &k).unwrap();
    assert_eq!(p.slojevi.len(), 3);
    assert_eq!(p.odabranih(), 3);
    let nasipi = p.slojevi.iter().find(|s| s.naziv == "nasipi").unwrap();
    assert_eq!((nasipi.objekata, nasipi.sustav), (2, "WGS84"));
    assert_eq!(p.izostavljena_polja(), vec!["duljina".to_owned()]);
    p.slojevi
        .iter_mut()
        .find(|s| s.id == obuhvati)
        .unwrap()
        .odabran = false;

    let mut u = Uredjivac::novi();
    let s = uvezi(&mut u, &p).unwrap();
    assert_eq!((s.slojeva, s.objekata), (2, 3));
    let projekt = u.projekt();
    let mapa = projekt.mapa(s.mapa).unwrap();
    assert_eq!(mapa.naziv, p.naziv);
    let Stavka::Mapa(sektor) = mapa.djeca[0] else {
        panic!("mapa Sektor");
    };
    assert_eq!(projekt.mapa(sektor).unwrap().naziv, "Sektor");
    let objekti: Vec<&Objekt> = projekt
        .slojevi()
        .flat_map(|(_, s)| s.objekti.values())
        .collect();
    let nazivi: BTreeSet<&str> = objekti.iter().map(|o| o.naziv.as_str()).collect();
    assert_eq!(nazivi, ["Crpka", "Nasip A", "Nasip B"].into());
    let a = objekti.iter().find(|o| o.naziv == "Nasip A").unwrap();
    assert_eq!(a.opis, "Lijevi nasip");
    assert_eq!(a.stil.boja_crte.map(|b| b.r), Some(200));
    assert!(projekt.slojevi().all(|(_, s)| s.naziv != "obuhvati"));

    u.ponisti().unwrap();
    assert!(
        u.projekt().korijen().is_empty(),
        "cijeli uvoz je jedan korak"
    );
}

#[test]
fn losa_i_prazna_datoteka_i_sitnice() {
    let k = KatalogPoruka::hrvatski();
    let losa = put("uvoz-losa");
    std::fs::write(&losa, b"nije GeoPackage").unwrap();
    assert_eq!(
        pripremi(&losa, &k).unwrap_err(),
        k.tekst("uvoz-gpkg-greska")
    );
    let prazna = put("uvoz-prazna");
    geopaket::Datoteka::otvori(&prazna)
        .unwrap()
        .spremi(&Projekt::default())
        .unwrap();
    assert_eq!(
        pripremi(&prazna, &k).unwrap_err(),
        k.tekst("uvoz-gpkg-prazno")
    );
    assert_eq!(broj(None), None);
    let o = Objekt {
        id: slojevi::IdObjekta(1),
        naziv: String::new(),
        opis: String::new(),
        geometrija: Geometrija::Tocka(Vrh::novi(18.0, 45.0)),
        stil: StilObjekta::default(),
    };
    assert_eq!(naziv_objekta(&Projekt::default(), IdSloja(1), &o), "");
    let nazivi: BTreeSet<&str> = [
        Sustav::Wgs84,
        Sustav::WebMercator,
        Sustav::Htrs96Tm,
        Sustav::HdksZona5,
        Sustav::HdksZona6,
    ]
    .map(naziv_sustava)
    .into();
    assert_eq!(nazivi.len(), 5);
}

fn prozor(birac: &LazniBirac) -> Harness<'static, Aplikacija> {
    let app = Aplikacija::nova(PostavkeAplikacije::default());
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.state_mut().postavi_birac(Box::new(birac.clone()));
    h.run_steps(3);
    h
}

#[test]
fn gumb_uvezi_sloj_nudi_slojeve_geopackagea() {
    let (staza, _) = izvorna_datoteka("uvoz-prozor");
    let birac = LazniBirac::novi();
    let mut h = prozor(&birac);
    birac.postavi_uvoz(&staza);
    h.get_by_label("Uvezi sloj").click();
    h.run_steps(3);
    assert!(matches!(
        h.state().aktivni_dijalog,
        Some(AktivniDijalog::UvozGpkg(_))
    ));
    h.get_by_label("obuhvati").click();
    h.run_steps(3);
    h.get_by_label("Uvezi 2 sloja").click();
    h.run_steps(3);
    let Some(AktivniDijalog::Obavijest { poruka, .. }) = &h.state().aktivni_dijalog else {
        panic!("{:?}", h.state().aktivni_dijalog);
    };
    assert_eq!(poruka, "Uvezeno je 3 objekta u 2 sloja.");
    h.get_by_label("U redu").click();
    h.run_steps(3);

    // Odustani ne uvozi ništa, a pokvarena datoteka javlja grešku.
    birac.postavi_uvoz(&staza);
    h.get_by_label("Uvezi sloj").click();
    h.run_steps(3);
    h.get_by_label("Odustani").click();
    h.run_steps(3);
    assert!(h.state().aktivni_dijalog.is_none());
    let losa = put("uvoz-prozor-losa");
    std::fs::write(&losa, b"x").unwrap();
    birac.postavi_uvoz(&losa);
    h.get_by_label("Uvezi sloj").click();
    h.run_steps(3);
    assert!(matches!(
        h.state().aktivni_dijalog,
        Some(AktivniDijalog::PorukaGreske { .. })
    ));
}
