use super::*;
use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::tema::Razmaci;
use eframe::egui::{Event, Key, Modifiers, PointerButton};
use egui_kittest::{
    Harness,
    kittest::{NodeT, Queryable},
};
use geometrija::{Metoda, MjereCrte, MjereLika};
use slojevi::{Geometrija, NaredbaObjekta, NaredbaStabla, Novo, Vrh};

/// Sredina karte u prozoru zadane veličine: ploča je desno, a alati lijevo.
fn sredina_karte() -> Pos2 {
    pos2(
        Razmaci::SIRINA_PROZORA / 2.0,
        (Razmaci::VISINA_IZBORNIKA + Razmaci::VISINA_PROZORA - Razmaci::VISINA_STATUSA) / 2.0,
    )
}

fn prozor(app: Aplikacija) -> Harness<'static, Aplikacija> {
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.run_steps(3);
    h
}

fn klikni(h: &mut Harness<'_, Aplikacija>, m: Pos2) {
    let tipka = |pressed| Event::PointerButton {
        pos: m,
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    h.input_mut().events.push(Event::PointerMoved(m));
    h.run_steps(1);
    h.input_mut().events.push(tipka(true));
    h.run_steps(1);
    h.input_mut().events.push(tipka(false));
    // Dovoljno dugo da sljedeći klik ne bude dvoklik.
    h.run_steps(10);
}

fn pritisni(h: &mut Harness<'_, Aplikacija>, key: Key) {
    for pressed in [true, false] {
        h.input_mut().events.push(Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        });
    }
    h.run_steps(2);
}

fn tekstovi(h: &Harness<'_, Aplikacija>) -> Vec<String> {
    h.output()
        .shapes
        .iter()
        .filter_map(|s| match &s.shape {
            egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
            _ => None,
        })
        .collect()
}

fn broj_objekata(app: &Aplikacija) -> usize {
    app.uredjivac
        .projekt()
        .slojevi()
        .map(|(_, s)| s.objekti.len())
        .sum()
}

fn novi_sloj(app: &mut Aplikacija, vrsta: VrstaGeometrije) -> IdSloja {
    let Ok(Some(Novo::Sloj(s))) = app.uredjivac.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Pokus".into(),
        vrsta,
    }) else {
        panic!("sloj nije dodan");
    };
    s
}

#[test]
fn mjerilo_je_okruglo_i_stane_u_traku() {
    assert_eq!(nazivnik_mjerila(1.177), Some(4_400.0));
    assert_eq!(nazivnik_mjerila(1.0), Some(3_800.0));
    assert_eq!(nazivnik_mjerila(0.0), None);
    assert_eq!(nazivnik_mjerila(f64::NAN), None);
    let pogled = |m| Pogled {
        metara_po_tocki: m,
        sredina: Vrh::novi(16.0, 45.0),
        mis: None,
    };
    let (d, w) = traka_mjerila(pogled(1.177)).expect("traka");
    assert_eq!(d, 100.0);
    assert!((w - 84.96).abs() < 0.01, "{w}");
    assert_eq!(traka_mjerila(pogled(3.0)).map(|t| t.0), Some(200.0));
    assert_eq!(traka_mjerila(pogled(4.0)).map(|t| t.0), Some(200.0));
    assert_eq!(traka_mjerila(pogled(6.0)).map(|t| t.0), Some(500.0));
    assert_eq!(traka_mjerila(pogled(0.0)), None);
    assert_eq!(oznaka_mjerila(2_000.0), "2 km");
    assert_eq!(oznaka_mjerila(500.0), "500 m");
    assert_eq!(oznaka_mjerila(0.5), "50 cm");
    let k = KatalogPoruka::hrvatski();
    assert_eq!(tekst_mjerila(&k, 4_400.0), "1 : 4 400");
}

#[test]
fn podloga_alat_mjere_i_poruke_prevode_se_za_kartu() {
    let mut s = StanjePrikaza::novo();
    s.tema_tamna = true;
    assert_eq!(zeljena_podloga(&s), Podloga::OpenFreeMap { tamna: true });
    s.podloga = 1;
    assert_eq!(zeljena_podloga(&s), Podloga::Dgu(ListDgu::Dof));
    s.podloga = 2;
    s.tema_tamna = false;
    assert_eq!(zeljena_podloga(&s), Podloga::OpenFreeMap { tamna: false });
    let parovi = [
        (Alat::Odaberi, karta::Alat::Odaberi),
        (Alat::Tocka, karta::Alat::Tocka),
        (Alat::Linija, karta::Alat::Linija),
        (Alat::Poligon, karta::Alat::Poligon),
        (Alat::Mjeri, karta::Alat::Mjeri),
    ];
    for (a, k) in parovi {
        assert_eq!(alat_karte(a), k);
    }
    let k = KatalogPoruka::hrvatski();
    let crta = Mjere::Linija(MjereCrte {
        duljina_m: 885.0,
        metoda: Metoda::ElipsoidWgs84,
    });
    assert_eq!(opis_mjere(&k, &crta), "885 m");
    let lik = Mjere::Poligon(MjereLika {
        opseg_m: 827.0,
        povrsina_m2: 48_300.0,
        metoda: Metoda::ElipsoidWgs84,
    });
    assert_eq!(opis_mjere(&k, &lik), "4,83 ha, opseg 827 m");
    assert_eq!(opis_mjere(&k, &Mjere::Tocka), "");
    assert_eq!(
        tekst_poruke(&k, Poruka::Premalo { najmanje: 3 }),
        "Za završetak treba bar 3 točke."
    );
    assert!(tekst_poruke(&k, Poruka::NeMozeBrisati { najmanje: 2 }).contains("nije obrisana"));
    assert_eq!(
        tekst_poruke(&k, Poruka::Naredba(slojevi::Greska::SlojZakljucan)),
        "Izmjena nije napravljena: sloj se ne smije mijenjati."
    );
    assert_eq!(kljuc_novog_sloja(VrstaGeometrije::Tocka), "sloj-tocke");
    assert_eq!(kljuc_novog_sloja(VrstaGeometrije::Linija), "sloj-linije");
    assert_eq!(kljuc_novog_sloja(VrstaGeometrije::Poligon), "sloj-poligoni");
}

#[test]
fn sloj_se_crta_samo_kad_su_on_i_sve_mape_iznad_njega_ukljuceni() {
    let mut u = Uredjivac::novi();
    let mapa = |u: &mut Uredjivac, iznad| match u.izvrsi(NaredbaStabla::DodajMapu {
        u: iznad,
        naziv: "Mapa".into(),
    }) {
        Ok(Some(Novo::Mapa(m))) => m,
        r => panic!("{r:?}"),
    };
    let gornja = mapa(&mut u, None);
    let donja = mapa(&mut u, Some(gornja));
    let Ok(Some(Novo::Sloj(s))) = u.izvrsi(NaredbaStabla::DodajSloj {
        u: Some(donja),
        naziv: "Sloj".into(),
        vrsta: VrstaGeometrije::Tocka,
    }) else {
        panic!("sloj");
    };
    let Ok(Some(Novo::Sloj(vani))) = u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Vani".into(),
        vrsta: VrstaGeometrije::Tocka,
    }) else {
        panic!("sloj");
    };
    let mut st = StanjePrikaza::novo();
    assert_eq!(
        vidljivi_slojevi(u.projekt(), &st),
        BTreeSet::from([s, vani])
    );
    st.postavi_vidljivost_mape(gornja, false);
    assert_eq!(vidljivi_slojevi(u.projekt(), &st), BTreeSet::from([vani]));
    st.postavi_vidljivost_mape(gornja, true);
    st.postavi_vidljivost_mape(donja, false);
    assert_eq!(vidljivi_slojevi(u.projekt(), &st), BTreeSet::from([vani]));
    st.postavi_vidljivost_mape(donja, true);
    st.postavi_vidljivost_sloja(s, false);
    assert_eq!(vidljivi_slojevi(u.projekt(), &st), BTreeSet::from([vani]));
}

#[test]
fn alat_tocka_iz_trake_dodaje_tocku_klikom_na_kartu_a_ponisti_je_brise() {
    let mut h = prozor(Aplikacija::default());
    h.get_by_label("Točka").click();
    h.run_steps(2);
    assert_eq!(h.state().stanje.alat, Alat::Tocka);
    klikni(&mut h, sredina_karte());
    assert_eq!(broj_objekata(h.state()), 1, "klik na kartu dodaje točku");
    let (_, sloj) = h
        .state()
        .uredjivac
        .projekt()
        .slojevi()
        .next()
        .expect("sloj");
    assert_eq!(sloj.naziv, "Točke", "novi sloj dobiva naziv iz kataloga");
    let objekt = sloj.objekti.values().next().expect("objekt");
    assert_eq!(
        objekt.naziv, "Točka 1",
        "novi objekt dobiva naziv s rednim brojem"
    );
    // Klik na praznu ploču Slojevi ne smije pasti na kartu ispod nje.
    klikni(&mut h, pos2(Razmaci::SIRINA_PROZORA - 200.0, 470.0));
    // Ni klik na traku alata, ispod zadnjeg gumba.
    klikni(&mut h, pos2(46.0, 360.0));
    assert_eq!(
        broj_objekata(h.state()),
        1,
        "ploča i alati zaklanjaju kartu"
    );
    h.state_mut().uredjivac.ponisti().expect("poništi");
    h.run_steps(2);
    assert_eq!(broj_objekata(h.state()), 0);
}

#[test]
fn backspace_dok_traje_crtanje_brise_tocku_crteza_a_ne_odabrani_sloj() {
    let mut app = Aplikacija::default();
    let s = novi_sloj(&mut app, VrstaGeometrije::Linija);
    app.stanje.odabrano = Some(Odabir::Sloj(s));
    app.stanje.alat = Alat::Linija;
    let mut h = prozor(app);
    let m = sredina_karte();
    klikni(&mut h, m);
    klikni(&mut h, m + vec2(80.0, 30.0));
    assert!(h.state_mut().karta.u_tijeku());
    pritisni(&mut h, Key::Backspace);
    assert!(
        h.state().uredjivac.projekt().sloj(s).is_some(),
        "Backspace u crtanju ne briše sloj"
    );
    assert!(h.state_mut().karta.u_tijeku(), "ostala je prva točka");
    pritisni(&mut h, Key::Escape);
    assert!(!h.state_mut().karta.u_tijeku(), "Escape odustaje od crteža");
    pritisni(&mut h, Key::Backspace);
    assert!(
        h.state().uredjivac.projekt().sloj(s).is_none(),
        "bez crtanja Backspace briše odabrani sloj"
    );
}

#[test]
fn odabir_objekta_ide_s_karte_u_plocu_i_iz_ploce_na_kartu() {
    let mut app = Aplikacija::default();
    let s = novi_sloj(&mut app, VrstaGeometrije::Tocka);
    let Ok(Some(Novo::Objekt(o))) = app.uredjivac.izvrsi(NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: "Zdenac".into(),
        geometrija: Geometrija::Tocka(Vrh::novi(16.2, 45.3)),
    }) else {
        panic!("objekt");
    };
    app.karta.karta().idi_na(16.2, 45.3, 14.0);
    let mut h = prozor(app);
    klikni(&mut h, sredina_karte());
    assert_eq!(h.state().stanje.odabrano, Some(Odabir::Objekt(s, o)));
    h.state_mut().stanje.odabrano = Some(Odabir::Sloj(s));
    h.run_steps(2);
    assert_eq!(h.state_mut().karta.karta().crtanje().odabran(), None);
    h.state_mut().stanje.odabrano = Some(Odabir::Objekt(s, o));
    h.run_steps(2);
    assert_eq!(
        h.state_mut().karta.karta().crtanje().odabran(),
        Some((s, o))
    );
}

#[test]
fn ploca_mjerenja_pokazuje_duljinu_sprema_liniju_i_zatvara_se() {
    let mut app = Aplikacija::default();
    app.stanje.alat = Alat::Mjeri;
    let mut h = prozor(app);
    assert!(tekstovi(&h).contains(&"Klikni na kartu za prvu točku.".to_owned()));
    let oznaceno = |h: &Harness<'_, Aplikacija>, l: &str| {
        h.get_by_label(l).accesskit_node().toggled() == Some(egui::accesskit::Toggled::True)
    };
    assert!(oznaceno(&h, "Udaljenost"));
    assert!(!oznaceno(&h, "Površina"));
    let m = sredina_karte();
    klikni(&mut h, m);
    klikni(&mut h, m + vec2(120.0, 0.0));
    let t = tekstovi(&h);
    assert!(
        t.contains(&"Ukupno, 1 dio. Računato na elipsoidu WGS84.".to_owned()),
        "{t:?}"
    );
    assert!(
        t.iter().any(|t| t.ends_with(" m") || t.ends_with(" km")),
        "{t:?}"
    );
    assert_eq!(broj_objekata(h.state()), 0, "mjerenje se samo ne sprema");
    h.get_by_label("Spremi kao liniju").click();
    h.run_steps(2);
    assert_eq!(broj_objekata(h.state()), 1);
    let (_, sloj) = h
        .state()
        .uredjivac
        .projekt()
        .slojevi()
        .next()
        .expect("sloj");
    assert_eq!(sloj.vrsta, VrstaGeometrije::Linija);
    h.get_by_label("Površina").click();
    h.run_steps(2);
    assert!(h.state_mut().karta.karta().crtanje().mjerenje_povrsine());
    h.get_by_label("Zatvori mjerenje").click();
    h.run_steps(2);
    assert_eq!(h.state().stanje.alat, Alat::Odaberi);
    assert!(h.query_by_label("Površina").is_none(), "ploča se zatvorila");
}

#[test]
fn premalo_tocaka_za_lik_javlja_poruku_na_karti() {
    let mut app = Aplikacija::default();
    app.stanje.alat = Alat::Poligon;
    let mut h = prozor(app);
    let m = sredina_karte();
    klikni(&mut h, m);
    // Dvoklik na istom mjestu završava lik, a on ima samo jednu točku.
    let tipka = |pressed| Event::PointerButton {
        pos: m + vec2(60.0, 0.0),
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    h.input_mut()
        .events
        .push(Event::PointerMoved(m + vec2(60.0, 0.0)));
    for p in [true, false, true, false] {
        h.input_mut().events.push(tipka(p));
        h.run_steps(1);
    }
    h.run_steps(2);
    assert!(
        tekstovi(&h)
            .iter()
            .any(|t| t.starts_with("Za završetak treba bar 3")),
        "{:?}",
        tekstovi(&h)
    );
}

#[test]
fn primjer_ima_pravo_mjerilo_koordinate_i_zum_iz_gumba() {
    let mut h = prozor(Aplikacija::nova(PostavkeAplikacije {
        primjer: true,
        ..Default::default()
    }));
    let status = h.state().karta.status().expect("karta je prikazana");
    let n = status.nazivnik.expect("mjerilo");
    assert!((3_000.0..6_000.0).contains(&n), "{n}");
    let [e, s] = status.koordinate.expect("koordinate");
    // Sredina primjera, 16,004° E i 45,802° N, je oko 38 km zapadno od središnjeg meridijana 16,5°.
    assert!(
        (e - 461_445.0).abs() < 50.0 && (s - 5_073_691.0).abs() < 50.0,
        "{e} {s}"
    );
    assert_eq!(status.podloga, None, "testovi rade bez mreže");
    assert!(tekstovi(&h).contains(&"100 m".to_owned()), "traka mjerila");
    let zum = h.state_mut().karta.karta().memorija().zoom();
    h.get_by_label("Približi").click();
    h.run_steps(2);
    let veci = h.state_mut().karta.karta().memorija().zoom();
    assert!((veci - zum - 1.0).abs() < 1e-9, "{zum} → {veci}");
    h.get_by_label("Udalji").click();
    h.run_steps(2);
    assert!((h.state_mut().karta.karta().memorija().zoom() - zum).abs() < 1e-9);
}

#[test]
fn navod_se_crta_samo_uz_postavljenu_podlogu() {
    let mut app = Aplikacija::default();
    app.karta.postavljena = Some(Podloga::OpenFreeMap { tamna: false });
    let h = prozor(app);
    assert!(
        tekstovi(&h)
            .iter()
            .any(|t| t == Podloga::OpenFreeMap { tamna: false }.navod())
    );
    let h = prozor(Aplikacija::default());
    assert!(!tekstovi(&h).iter().any(|t| t.contains("OpenMapTiles")));
}

#[test]
fn podloga_se_ukljucuje_samo_s_mrezom_i_prati_izbor_u_ploci() {
    let ctx = egui::Context::default();
    let mut st = StanjePrikaza::novo();
    let mut bez = KartaProzora::nova(false);
    bez.uskladi_podlogu(&ctx, &st);
    assert_eq!(bez.postavljena, None);
    let mut k = KartaProzora::nova(true);
    k.uskladi_podlogu(&ctx, &st);
    assert_eq!(k.postavljena, Some(Podloga::OpenFreeMap { tamna: false }));
    st.podloga = 1;
    k.uskladi_podlogu(&ctx, &st);
    assert_eq!(k.postavljena, Some(Podloga::Dgu(ListDgu::Dof)));
}

#[test]
fn nova_tocka_ide_u_sloj_odabran_u_ploci() {
    let mut app = Aplikacija::default();
    let a = novi_sloj(&mut app, VrstaGeometrije::Tocka);
    let b = novi_sloj(&mut app, VrstaGeometrije::Tocka);
    app.stanje.alat = Alat::Tocka;
    let mut h = prozor(app);
    let m = sredina_karte();
    for (sloj, pomak) in [(a, 0.0), (b, 90.0), (a, 180.0)] {
        h.state_mut().stanje.odabrano = Some(Odabir::Sloj(sloj));
        h.run_steps(2);
        klikni(&mut h, m + vec2(pomak, 0.0));
    }
    let broj = |s| {
        h.state()
            .uredjivac
            .projekt()
            .sloj(s)
            .map(|s| s.objekti.len())
    };
    assert_eq!((broj(a), broj(b)), (Some(2), Some(1)));
}

#[test]
fn poruka_kaze_kad_novi_objekt_ne_ide_u_odabrani_sloj() {
    let mut app = Aplikacija::default();
    let s = novi_sloj(&mut app, VrstaGeometrije::Poligon);
    let k = KatalogPoruka::hrvatski();
    let mut st = StanjePrikaza::novo();
    let p = app.uredjivac.projekt();
    st.alat = Alat::Tocka;
    assert_eq!(
        poruka_drugog_sloja(p, &st, &k),
        None,
        "bez odabira nema poruke"
    );
    st.odabrano = Some(Odabir::Sloj(s));
    assert_eq!(
        poruka_drugog_sloja(p, &st, &k).as_deref(),
        Some("Sloj „Pokus” sadrži poligone, pa novi objekt ide u sloj za točke.")
    );
    st.alat = Alat::Linija;
    assert!(poruka_drugog_sloja(p, &st, &k).is_some_and(|t| t.ends_with("za linije.")));
    st.alat = Alat::Poligon;
    assert_eq!(poruka_drugog_sloja(p, &st, &k), None, "ista vrsta");
    for alat in [Alat::Odaberi, Alat::Mjeri] {
        st.alat = alat;
        assert_eq!(poruka_drugog_sloja(p, &st, &k), None);
    }
    st.alat = Alat::Tocka;
    st.odabrano = Some(Odabir::Objekt(s, IdObjekta(7)));
    assert!(
        poruka_drugog_sloja(p, &st, &k).is_some(),
        "odabran objekt u sloju"
    );
    let mut h = prozor(app);
    h.state_mut().stanje.alat = Alat::Tocka;
    h.state_mut().stanje.odabrano = Some(Odabir::Sloj(s));
    h.run_steps(2);
    assert!(
        tekstovi(&h)
            .iter()
            .any(|t| t.starts_with("Sloj „Pokus” sadrži poligone"))
    );
    assert_eq!(
        kljuc_novog_objekta(VrstaGeometrije::Linija),
        "objekt-linija"
    );
    assert_eq!(
        kljuc_novog_objekta(VrstaGeometrije::Poligon),
        "objekt-poligon"
    );
}
