use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::datoteka::LazniBirac;
use crate::dijalozi::AktivniDijalog;
use crate::poruke::KatalogPoruka;
use crate::stanje::{Odabir, StanjePrikaza};
use crate::tema::Razmaci;
use crate::uvoz::{obuhvat_mape, podmape, poruka_greske, poruka_sazetka, uvezi_datoteku};
use eframe::egui::{Key, Modifiers, vec2};
use egui_kittest::{Harness, kittest::Queryable};
use formati::{GreskaKml, Gubici, SazetakUvoza};
use slojevi::{IdMape, NaredbaStabla, Novo, Uredjivac};
use std::path::PathBuf;

fn podatak(ime: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../formati/tests/podaci")
        .join(ime)
}

#[test]
fn svaka_greska_ima_svoju_poruku() {
    let k = KatalogPoruka::hrvatski();
    let greske = [
        GreskaKml::Citanje,
        GreskaKml::NeispravanXml { polozaj: 5 },
        GreskaKml::PreDuboko,
        GreskaKml::NeispravnaArhiva,
        GreskaKml::NemaKmlUArhivi,
        GreskaKml::NedopustenaPutanja,
        GreskaKml::PrevelikaArhiva,
    ];
    let mut poruke: Vec<String> = greske.iter().map(|g| poruka_greske(&k, *g)).collect();
    assert!(poruke.iter().all(|p| p.ends_with('.')));
    poruke.sort();
    poruke.dedup();
    assert_eq!(poruke.len(), greske.len());
}

#[test]
fn sazetak_navodi_brojeve_i_ono_sto_nije_uvezeno() {
    let k = KatalogPoruka::hrvatski();
    let mut s = SazetakUvoza {
        mapa: IdMape(1),
        mapa_ukupno: 3,
        slojeva: 2,
        objekata: 1,
        odbijeno: 0,
        gubici: Gubici::default(),
    };
    let p = poruka_sazetka(&k, &s);
    assert!(p.contains("1 objekt u 2 sloja i 3 mape"), "{p}");
    assert!(!p.contains("Nije uvezeno"));

    s.gubici.preskoceno.insert("NetworkLink".into(), 52);
    s.gubici.preskoceno.insert("Nepoznato".into(), 1);
    s.gubici.prazne_geometrije = 3;
    s.odbijeno = 2;
    let p = poruka_sazetka(&k, &s);
    assert!(p.contains("mrežne veze: 52"), "{p}");
    assert!(p.contains("prazne ili nepotpune geometrije: 3"), "{p}");
    assert!(p.contains("geometrije koje projekt ne prima: 2"), "{p}");
    assert!(!p.contains("koordinatama"));
}

#[test]
fn uvoz_otvara_novu_mapu_a_podmape_zatvara() {
    let k = KatalogPoruka::hrvatski();
    let mut u = Uredjivac::novi();
    let mut stanje = StanjePrikaza::novo();
    let s =
        uvezi_datoteku(&mut u, &mut stanje, &k, &podatak("mijesani_slojevi.kml")).expect("uvoz");
    assert_eq!(stanje.odabrano, Some(Odabir::Mapa(s.mapa)));
    assert!(stanje.je_mapa_otvorena(s.mapa));
    let pod = podmape(u.projekt(), s.mapa);
    assert_eq!(pod.len(), 4);
    assert!(pod.iter().all(|m| !stanje.je_mapa_otvorena(*m)));
    let (d, sirina, _) = obuhvat_mape(u.projekt(), s.mapa).expect("obuhvat");
    assert!((18.5..18.8).contains(&d) && (45.4..45.7).contains(&sirina));

    // Prazna mapa nema obuhvata, a nepostojeća nema podmapa.
    let Ok(Some(Novo::Mapa(prazna))) = u.izvrsi(NaredbaStabla::DodajMapu {
        u: None,
        naziv: "Prazna".into(),
    }) else {
        panic!("mapa");
    };
    assert_eq!(obuhvat_mape(u.projekt(), prazna), None);
    assert!(podmape(u.projekt(), IdMape(9999)).is_empty());

    let greska = uvezi_datoteku(&mut u, &mut stanje, &k, &podatak("zip_bomba.kmz"));
    assert_eq!(greska, Err(poruka_greske(&k, GreskaKml::PrevelikaArhiva)));
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
fn gumb_uvezi_sloj_uvozi_kml_i_javlja_sazetak() {
    let birac = LazniBirac::novi();
    let mut h = prozor(&birac);
    birac.postavi_uvoz(podatak("mijesani_slojevi.kml"));
    h.get_by_label("Uvezi sloj").click();
    h.run_steps(4);
    assert!(matches!(
        h.state().aktivni_dijalog,
        Some(AktivniDijalog::Obavijest { .. })
    ));
    let Some(AktivniDijalog::Obavijest { naslov, poruka }) = &h.state().aktivni_dijalog else {
        panic!("obavijest");
    };
    assert_eq!(naslov, "Uvoz je gotov");
    assert!(poruka.contains("u 6 slojeva i 5 mapa"), "{poruka}");
    assert!(h.query_by_label("Ispit podjele na slojeve").is_some());
    h.get_by_label("U redu").click();
    h.run_steps(3);
    assert!(h.state().aktivni_dijalog.is_none());

    h.key_press_modifiers(Modifiers::COMMAND, Key::Z);
    h.run_steps(3);
    assert!(h.state().uredjivac.projekt().korijen().is_empty());

    // Bez odabrane datoteke ništa se ne događa.
    h.get_by_label("Uvezi sloj").click();
    h.run_steps(3);
    assert!(h.state().aktivni_dijalog.is_none());
}

#[test]
fn izbornik_datoteka_uvozi_a_pokvaren_kml_javlja_gresku() {
    let birac = LazniBirac::novi();
    let mut h = prozor(&birac);
    birac.postavi_uvoz(podatak("ostecen.kml"));
    h.get_by_label("Datoteka").click();
    h.run_steps(3);
    h.get_by_label("Uvezi KML, KMZ ili GeoPackage…").click();
    h.run_steps(4);
    assert!(matches!(
        h.state().aktivni_dijalog,
        Some(AktivniDijalog::PorukaGreske { .. })
    ));
    let Some(AktivniDijalog::PorukaGreske { naslov, .. }) = &h.state().aktivni_dijalog else {
        panic!("greška");
    };
    assert_eq!(naslov, "Uvoz nije uspio");
    assert!(h.query_by_label("U redu").is_some());
    assert!(h.state().uredjivac.projekt().korijen().is_empty());
}
