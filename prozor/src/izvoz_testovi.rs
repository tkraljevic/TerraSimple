use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::datoteka::LazniBirac;
use crate::dijalozi::{AktivniDijalog, StavkaIzbornikaDatoteka};
use crate::izvoz::{FormatIzvoza, Prolaz, naziv_izvoza, pripremi, stavke_geojson, stavke_pregleda};
use crate::poruke::KatalogPoruka;
use crate::tema::Razmaci;
use eframe::egui::vec2;
use egui_kittest::{Harness, kittest::Queryable};
use formati::{OpsegIzvoza, PregledIzvoza};
use slojevi::{NaredbaStabla, Novo, Stavka, Uredjivac, VrstaGeometrije};
use std::path::{Path, PathBuf};

fn podatak(ime: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../formati/tests/podaci")
        .join(ime)
}

#[test]
fn pregled_kaze_sto_ide_sto_se_mijenja_i_sto_ne_ide() {
    let k = KatalogPoruka::hrvatski();
    let prazno = stavke_pregleda(&k, &PregledIzvoza::default());
    assert_eq!(prazno.len(), 1);
    assert_eq!(prazno[0].prolaz, Prolaz::Ne);

    let p = PregledIzvoza {
        tocaka: 2,
        linija: 1,
        mapa: 1,
        atributi: ["oznaka".to_owned(), "povrsina".to_owned()].into(),
        izostavljene_vrijednosti: 3,
        drugi_sustav: true,
        isprekidane: 1,
        srafure: 1,
        natpisi_bez_tocaka: 1,
        kurziv: 1,
        ..PregledIzvoza::default()
    };
    let s = stavke_pregleda(&k, &p);
    assert_eq!(s.len(), 8);
    assert_eq!(s[0].opis, "3 objekta, 1 mapa.");
    assert!(s[1].opis.contains("oznaka, povrsina"));
    let pazi = s.iter().filter(|s| s.prolaz == Prolaz::Pazi).count();
    assert_eq!(pazi, 5);
    assert_eq!(s[7].prolaz, Prolaz::Ne);
    assert!(s[7].opis.ends_with("3."));
    assert!(
        s.iter()
            .all(|s| !s.naslov.is_empty() && s.opis.ends_with('.'))
    );
}

#[test]
fn naziv_je_mapa_sloj_ili_datoteka_projekta() {
    let k = KatalogPoruka::hrvatski();
    let mut u = Uredjivac::novi();
    let Ok(Some(Novo::Sloj(s))) = u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Kanali".into(),
        vrsta: VrstaGeometrije::Linija,
    }) else {
        panic!("sloj");
    };
    let p = u.projekt();
    let sloj = OpsegIzvoza::Stavka(Stavka::Sloj(s));
    assert_eq!(naziv_izvoza(p, sloj, None, &k), "Kanali");
    let staza = Path::new("/neka/mapa/Teren 2026.gpkg");
    assert_eq!(
        naziv_izvoza(p, OpsegIzvoza::Projekt, Some(staza), &k),
        "Teren 2026"
    );
    assert_eq!(
        naziv_izvoza(p, OpsegIzvoza::Projekt, None, &k),
        "TerraSimple"
    );
    let nema = OpsegIzvoza::Stavka(Stavka::Mapa(slojevi::IdMape(999)));
    assert_eq!(naziv_izvoza(p, nema, None, &k), "TerraSimple");
    assert!(!pripremi(p, sloj, None, &k).moze(), "prazan sloj");
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

fn obavijest(h: &Harness<'_, Aplikacija>) -> (String, String) {
    match &h.state().aktivni_dijalog {
        Some(AktivniDijalog::Obavijest { naslov, poruka })
        | Some(AktivniDijalog::PorukaGreske { naslov, poruka }) => (naslov.clone(), poruka.clone()),
        d => panic!("{d:?}"),
    }
}

#[test]
fn desni_klik_na_mapu_izvozi_je_u_kmz() {
    let birac = LazniBirac::novi();
    let mut h = prozor(&birac);
    birac.postavi_uvoz(podatak("mijesani_slojevi.kml"));
    h.get_by_label("Uvezi sloj").click();
    h.run_steps(4);
    h.get_by_label("U redu").click();
    h.run_steps(3);

    h.get_by_label("Ispit podjele na slojeve").click_secondary();
    h.run_steps(4);
    h.get_by_label("Izvezi…").click();
    h.run_steps(4);
    assert!(matches!(
        h.state().aktivni_dijalog,
        Some(AktivniDijalog::Izvoz(_))
    ));
    assert!(
        h.query_by_label("Sve geometrije, nazivi, opisi i mape")
            .is_none(),
        "tekst je na slikaru"
    );

    let mapa = std::env::temp_dir().join(format!("terrasimple-izvoz-ui-{}", std::process::id()));
    std::fs::create_dir_all(&mapa).unwrap();
    let staza = mapa.join("ispit.kmz");
    birac.postavi_izvoz(&staza);
    h.get_by_label("Izvezi…").click();
    h.run_steps(4);
    let (naslov, poruka) = obavijest(&h);
    assert_eq!(naslov, "Izvoz je gotov");
    assert!(poruka.contains("ispit.kmz"), "{poruka}");
    let d = formati::procitaj(&staza).expect("KMZ");
    assert_eq!(d.korijen.naziv, "Ispit podjele na slojeve");
    assert!(d.korijen.broj_objekata() > 0);

    // Izvoz u mapu koja ne postoji javlja grešku.
    h.get_by_label("U redu").click();
    h.run_steps(3);
    h.state_mut()
        .pokreni_radnju_datoteke(StavkaIzbornikaDatoteka::Izvezi);
    h.run_steps(3);
    birac.postavi_izvoz(mapa.join("nema").join("x.kml"));
    h.get_by_label("Izvezi…").click();
    h.run_steps(4);
    assert_eq!(obavijest(&h).0, "Izvoz nije uspio");
    std::fs::remove_dir_all(&mapa).unwrap();
}

#[test]
fn odustajanje_i_prazan_projekt_ne_pisu_nista() {
    let birac = LazniBirac::novi();
    let mut h = prozor(&birac);
    h.state_mut()
        .pokreni_radnju_datoteke(StavkaIzbornikaDatoteka::Izvezi);
    h.run_steps(3);
    let Some(AktivniDijalog::Izvoz(p)) = &h.state().aktivni_dijalog else {
        panic!("izvoz");
    };
    assert!(!p.moze());
    h.get_by_label("Izvezi…").click();
    h.run_steps(3);
    assert!(matches!(
        h.state().aktivni_dijalog,
        Some(AktivniDijalog::Izvoz(_))
    ));
    h.get_by_label("Odustani").click();
    h.run_steps(3);
    assert!(h.state().aktivni_dijalog.is_none());

    // Bez odabrane datoteke prozorčić se zatvara bez poruke.
    birac.postavi_uvoz(podatak("mijesani_slojevi.kml"));
    h.get_by_label("Uvezi sloj").click();
    h.run_steps(4);
    h.get_by_label("U redu").click();
    h.run_steps(3);
    h.state_mut()
        .pokreni_radnju_datoteke(StavkaIzbornikaDatoteka::Izvezi);
    h.run_steps(3);
    h.get_by_label("Izvezi…").click();
    h.run_steps(3);
    assert!(h.state().aktivni_dijalog.is_none());
}

#[test]
fn pregled_izvoza_otvara_se_i_za_snimku() {
    let app = Aplikacija::nova(PostavkeAplikacije {
        primjer: true,
        podloga_s_mreze: false,
        dijalog: Some("izvoz".into()),
        ..Default::default()
    });
    let Some(AktivniDijalog::Izvoz(p)) = &app.aktivni_dijalog else {
        panic!("izvoz");
    };
    assert!(p.moze());
    assert_eq!(p.naziv, "TerraSimple");
}

#[test]
fn geojson_ima_svoj_pregled() {
    let k = KatalogPoruka::hrvatski();
    assert_eq!(stavke_geojson(&k, &PregledIzvoza::default()).len(), 1);
    let p = PregledIzvoza {
        linija: 2,
        mapa: 1,
        isprekidane: 1,
        drugi_sustav: true,
        izostavljene_vrijednosti: 1,
        atributi: ["oznaka".to_owned()].into(),
        ..PregledIzvoza::default()
    };
    let s = stavke_geojson(&k, &p);
    let naslovi: Vec<&str> = s.iter().map(|s| s.naslov.as_str()).collect();
    assert_eq!(
        naslovi,
        [
            "Sve geometrije, nazivi, opisi i mape",
            "Atributi",
            "Neke vrijednosti ne idu u KML",
            "Mape postaju svojstvo",
            "Stil ide kao simplestyle",
            "Vrsta crte, šrafura i natpisi ne idu u GeoJSON",
            "Koordinate se pretvaraju u WGS84",
        ]
    );
    let samo = PregledIzvoza {
        tocaka: 1,
        ..PregledIzvoza::default()
    };
    assert_eq!(stavke_geojson(&k, &samo).len(), 2);
    let nazivi: Vec<&str> = FormatIzvoza::SVI.iter().map(|f| f.nastavak()).collect();
    assert_eq!(nazivi, ["kml", "kmz", "geojson"]);
}

#[test]
fn izbor_geojsona_u_prozorcicu_pise_geojson() {
    let birac = LazniBirac::novi();
    let mut h = prozor(&birac);
    birac.postavi_uvoz(podatak("mijesani_slojevi.kml"));
    h.get_by_label("Uvezi sloj").click();
    h.run_steps(4);
    h.get_by_label("U redu").click();
    h.run_steps(3);
    h.state_mut()
        .pokreni_radnju_datoteke(StavkaIzbornikaDatoteka::Izvezi);
    h.run_steps(3);
    h.get_by_label("GeoJSON").click();
    h.run_steps(3);
    let Some(AktivniDijalog::Izvoz(p)) = &h.state().aktivni_dijalog else {
        panic!("izvoz");
    };
    assert_eq!(p.format, FormatIzvoza::GeoJson);
    let mapa = std::env::temp_dir().join(format!("terrasimple-geojson-ui-{}", std::process::id()));
    std::fs::create_dir_all(&mapa).unwrap();
    let staza = mapa.join("ispit.geojson");
    birac.postavi_izvoz(&staza);
    h.get_by_label("Izvezi…").click();
    h.run_steps(4);
    let tekst = std::fs::read_to_string(&staza).unwrap();
    assert!(tekst.contains("\"type\":\"FeatureCollection\""));
    let d = formati::procitaj(&staza).unwrap();
    assert!(d.korijen.broj_objekata() > 0);
    std::fs::remove_dir_all(&mapa).unwrap();
}
