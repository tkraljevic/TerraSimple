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

fn desni_klik(h: &mut Harness<'_, Aplikacija>, m: Pos2) {
    let tipka = |pressed| Event::PointerButton {
        pos: m,
        button: PointerButton::Secondary,
        pressed,
        modifiers: Modifiers::NONE,
    };
    h.input_mut().events.push(Event::PointerMoved(m));
    h.run_steps(1);
    h.input_mut().events.push(tipka(true));
    h.run_steps(1);
    h.input_mut().events.push(tipka(false));
    h.run_steps(3);
}

/// Prozorčić s nazivom novog objekta zatvara se s Esc, a naziv ostaje predloženi.
fn preskoci_naziv(h: &mut Harness<'_, Aplikacija>) {
    assert!(
        h.state().stanje.naziv_novog.is_some(),
        "prozorčić s nazivom"
    );
    h.run_steps(2);
    assert!(h.query_by_label("Naziv objekta").is_some());
    pritisni(h, Key::Escape);
    assert!(h.state().stanje.naziv_novog.is_none());
}

fn pritisni(h: &mut Harness<'_, Aplikacija>, key: Key) {
    pritisni_s(h, Modifiers::NONE, key);
}

fn pritisni_s(h: &mut Harness<'_, Aplikacija>, modifiers: Modifiers, key: Key) {
    for pressed in [true, false] {
        h.input_mut().events.push(Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers,
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
        h.state().uredjivac.projekt().sloj(s).is_some(),
        "sam Backspace ne briše sloj"
    );
    pritisni_s(&mut h, Modifiers::COMMAND, Key::Backspace);
    assert!(
        h.state().uredjivac.projekt().sloj(s).is_none(),
        "⌘ + Backspace briše odabrani sloj"
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
    preskoci_naziv(&mut h);
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
    klikni(&mut h, m + vec2(60.0, 0.0));
    // Desni klik završava lik, a on ima samo dvije točke.
    desni_klik(&mut h, m + vec2(60.0, 0.0));
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

/// Odluke (9. 11. 2026.): Ispis je skriven dok ne proradi, i na karti i u izborniku, a ostali
/// plutajući gumbi ostaju jedan ispod drugoga bez rupe.
#[test]
fn ispis_je_skriven_a_ostali_gumbi_su_redom() {
    let h = prozor(Aplikacija::nova(PostavkeAplikacije::default()));
    assert!(h.query_by_label("Ispis").is_none());
    assert!(
        !tekstovi(&h).contains(&"Ispis".to_owned()),
        "{:?}",
        tekstovi(&h)
    );
    let y = |naziv: &str| h.get_by_label(naziv).rect().min.y;
    let zaslon = y("Cijeli zaslon");
    assert_eq!(y("Sjever") - zaslon, 50.0);
    assert_eq!(y("Približi") - zaslon, 100.0);
    assert_eq!(y("Udalji") - zaslon, 141.0);
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
        // Nakon svakog objekta alat se vraća na Odaberi, pa se Točka bira ponovno.
        h.state_mut().stanje.alat = Alat::Tocka;
        h.run_steps(2);
        klikni(&mut h, m + vec2(pomak, 0.0));
        preskoci_naziv(&mut h);
        assert_eq!(h.state().stanje.alat, Alat::Odaberi);
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

#[test]
fn otvoren_dijalog_zaklanja_kartu_i_tipke() {
    use crate::dijalozi::{AktivniDijalog, NamjeraPotvrde};
    let mut app = Aplikacija::default();
    let s = novi_sloj(&mut app, VrstaGeometrije::Tocka);
    app.stanje.odabrano = Some(Odabir::Sloj(s));
    app.stanje.alat = Alat::Tocka;
    app.aktivni_dijalog = Some(AktivniDijalog::PotvrdaNespremljeno(NamjeraPotvrde::Novi));
    let mut h = prozor(app);
    // Daleko od dijaloga u sredini, ali na karti.
    klikni(&mut h, pos2(300.0, 760.0));
    assert_eq!(broj_objekata(h.state()), 0, "klik pored dijaloga ne crta");
    pritisni(&mut h, Key::Delete);
    assert!(
        h.state().uredjivac.projekt().sloj(s).is_some(),
        "Delete ne briše sloj ispod dijaloga"
    );
    h.state_mut().aktivni_dijalog = None;
    h.run_steps(2);
    klikni(&mut h, pos2(300.0, 760.0));
    assert_eq!(broj_objekata(h.state()), 1, "bez dijaloga klik opet crta");
}

#[test]
fn obuhvat_projekta_daje_sredinu_i_zum() {
    let mut app = Aplikacija::default();
    assert_eq!(obuhvat(app.uredjivac.projekt()), None);
    let s = novi_sloj(&mut app, VrstaGeometrije::Tocka);
    let dodaj = |u: &mut Uredjivac, d: f64, sir: f64| {
        u.izvrsi(NaredbaObjekta::Dodaj {
            sloj: s,
            naziv: "T".into(),
            geometrija: Geometrija::Tocka(Vrh::novi(d, sir)),
        })
        .expect("točka");
    };
    dodaj(&mut app.uredjivac, 16.0, 45.0);
    assert_eq!(obuhvat(app.uredjivac.projekt()), Some((16.0, 45.0, 16.0)));
    dodaj(&mut app.uredjivac, 18.0, 46.0);
    let (d, sir, z) = obuhvat(app.uredjivac.projekt()).expect("obuhvat");
    assert_eq!((d, sir), (17.0, 45.5));
    // Raspon 2° po dužini: 1200 / 2 = 600, log2 ≈ 9,2.
    assert_eq!(z, 9.0);
}

#[test]
fn novi_projekt_brise_odabir_karte_i_pomice_pogled_na_podatke() {
    let mut app = Aplikacija::default();
    let s = novi_sloj(&mut app, VrstaGeometrije::Linija);
    let Ok(Some(Novo::Objekt(o))) = app.uredjivac.izvrsi(NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: "Crta".into(),
        geometrija: Geometrija::Linija(vec![Vrh::novi(17.0, 45.0), Vrh::novi(17.01, 45.01)]),
    }) else {
        panic!("objekt");
    };
    app.stanje.odabrano = Some(Odabir::Objekt(s, o));
    let mut h = prozor(app);
    assert_eq!(
        h.state_mut().karta.karta().crtanje().odabran(),
        Some((s, o))
    );
    let projekt = h.state().uredjivac.projekt().clone();
    h.state_mut().karta.novi_projekt(&projekt);
    assert_eq!(h.state_mut().karta.karta().crtanje().odabran(), None);
    h.run_steps(2);
    let pogled = h.state().karta.status().expect("pogled");
    let [e, _] = pogled.koordinate.expect("koordinate");
    // Sredina crte, 17,005° E, je oko 39 km istočno od središnjeg meridijana 16,5°.
    assert!((e - 539_800.0).abs() < 1_000.0, "{e}");
}

#[test]
fn klik_izvan_otvorenog_izbornika_datoteka_ga_samo_zatvara() {
    let mut app = Aplikacija::default();
    app.stanje.alat = Alat::Tocka;
    let mut h = prozor(app);
    h.get_by_label("Datoteka").click();
    h.run_steps(2);
    assert!(h.state().izbornik_datoteka_otvoren);
    klikni(&mut h, sredina_karte());
    assert!(
        !h.state().izbornik_datoteka_otvoren,
        "klik izvan zatvara izbornik"
    );
    assert_eq!(broj_objekata(h.state()), 0, "isti klik ne crta točku");
    klikni(&mut h, sredina_karte());
    assert_eq!(broj_objekata(h.state()), 1, "sljedeći klik crta");
}

/// Odabran lik koji siječe sam sebe dobiva upozorenje ispod karte, a ispravan ne.
#[test]
fn odabrana_osmica_dobiva_upozorenje_ispod_karte() {
    let mut app = Aplikacija::nova(PostavkeAplikacije::default());
    let Ok(Some(Novo::Sloj(sloj))) = app.uredjivac.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: "Parcele".into(),
        vrsta: slojevi::VrstaGeometrije::Poligon,
    }) else {
        panic!("nema sloja");
    };
    let lik = |tocke: [(f64, f64); 4]| {
        Geometrija::Poligon(vec![tocke.iter().map(|&(x, y)| Vrh::novi(x, y)).collect()])
    };
    let mut objekt = |naziv: &str, g| {
        let Ok(Some(Novo::Objekt(o))) = app.uredjivac.izvrsi(NaredbaObjekta::Dodaj {
            sloj,
            naziv: naziv.into(),
            geometrija: g,
        }) else {
            panic!("nema objekta");
        };
        o
    };
    let osmica = objekt(
        "Osmica",
        lik([(18.0, 45.0), (18.01, 45.01), (18.01, 45.0), (18.0, 45.01)]),
    );
    let kvadrat = objekt(
        "Kvadrat",
        lik([(18.0, 45.0), (18.01, 45.0), (18.01, 45.01), (18.0, 45.01)]),
    );
    let upozorenje =
        "Rub lika siječe sam sebe, pa površina nije točna. Mjesto je označeno crvenim krugom.";
    let mut h = prozor(app);
    assert!(!tekstovi(&h).iter().any(|t| t == upozorenje));
    h.state_mut()
        .karta
        .karta()
        .crtanje()
        .postavi_odabrani(Some((sloj, osmica)));
    h.run_steps(2);
    assert!(
        tekstovi(&h).iter().any(|t| t == upozorenje),
        "{:?}",
        tekstovi(&h)
    );
    h.state_mut()
        .karta
        .karta()
        .crtanje()
        .postavi_odabrani(Some((sloj, kvadrat)));
    h.run_steps(2);
    assert!(!tekstovi(&h).iter().any(|t| t == upozorenje));
}

fn poligon_desnim_klikom(h: &mut Harness<'_, Aplikacija>, m: Pos2) {
    h.state_mut().stanje.alat = Alat::Poligon;
    h.run_steps(2);
    for pomak in [
        vec2(0.0, 0.0),
        vec2(120.0, 0.0),
        vec2(120.0, 90.0),
        vec2(0.0, 90.0),
    ] {
        klikni(h, m + pomak);
    }
    desni_klik(h, m + vec2(60.0, 140.0));
    h.run_steps(2);
}

fn ponisti(h: &mut Harness<'_, Aplikacija>) {
    for pressed in [true, false] {
        h.input_mut().events.push(Event::Key {
            key: Key::Z,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::COMMAND,
        });
    }
    h.run_steps(2);
}

fn jedini_objekt(app: &Aplikacija) -> slojevi::Objekt {
    app.uredjivac
        .projekt()
        .slojevi()
        .flat_map(|(_, s)| s.objekti.values())
        .next()
        .expect("objekt")
        .clone()
}

#[test]
fn nakon_crtanja_prozorcic_trazi_naziv_a_alat_se_vraca_na_odaberi() {
    let mut h = prozor(Aplikacija::default());
    poligon_desnim_klikom(&mut h, sredina_karte());
    assert_eq!(broj_objekata(h.state()), 1, "desni klik je završio lik");
    assert_eq!(h.state().stanje.alat, Alat::Odaberi);
    assert!(h.query_by_label("Naziv objekta").is_some());
    let predlozeni = jedini_objekt(h.state()).naziv;

    // Predloženi naziv je sav označen, pa ga upisani tekst zamijeni.
    h.input_mut().events.push(Event::Text("Parcela 9".into()));
    h.run_steps(1);
    pritisni(&mut h, Key::Enter);
    assert!(h.state().stanje.naziv_novog.is_none());
    assert_eq!(jedini_objekt(h.state()).naziv, "Parcela 9");

    ponisti(&mut h);
    assert_eq!(jedini_objekt(h.state()).naziv, predlozeni);
    assert_eq!(broj_objekata(h.state()), 1, "poništen je samo naziv");
}

#[test]
fn okvir_odabire_dva_objekta_a_delete_ih_brise_odjednom() {
    let mut h = prozor(Aplikacija::default());
    let m = sredina_karte();
    for pomak in [0.0, 90.0] {
        h.state_mut().stanje.alat = Alat::Tocka;
        h.run_steps(2);
        klikni(&mut h, m + vec2(pomak, 0.0));
        preskoci_naziv(&mut h);
    }
    pritisni(&mut h, Key::Escape);
    assert!(h.state().stanje.odabrani_na_karti.is_empty());

    klikni(&mut h, m + vec2(-30.0, -30.0));
    klikni(&mut h, m + vec2(150.0, 30.0));
    assert_eq!(h.state().stanje.odabrani_na_karti.len(), 2);
    assert_eq!(h.state().stanje.odabrano, None);

    pritisni(&mut h, Key::Delete);
    assert_eq!(broj_objekata(h.state()), 0);
    ponisti(&mut h);
    assert_eq!(broj_objekata(h.state()), 2, "brisanje je jedan korak");
}

#[test]
fn backspace_brise_odabranu_tocku_lika_a_ne_cijeli_lik() {
    let mut h = prozor(Aplikacija::default());
    let m = sredina_karte();
    poligon_desnim_klikom(&mut h, m);
    preskoci_naziv(&mut h);
    klikni(&mut h, m + vec2(120.0, 0.0));
    pritisni(&mut h, Key::Backspace);
    assert_eq!(broj_objekata(h.state()), 1);
    let Geometrija::Poligon(p) = jedini_objekt(h.state()).geometrija else {
        panic!("poligon");
    };
    assert_eq!(p[0].len(), 3);
    pritisni(&mut h, Key::Backspace);
    assert_eq!(
        broj_objekata(h.state()),
        1,
        "sam Backspace ne briše cijeli lik"
    );
    pritisni_s(&mut h, Modifiers::COMMAND, Key::Backspace);
    assert_eq!(broj_objekata(h.state()), 0, "⌘ + Backspace briše lik");
}

#[test]
fn cmd_backspace_brise_lik_i_kad_mu_je_odabrana_tocka() {
    let mut h = prozor(Aplikacija::default());
    let m = sredina_karte();
    poligon_desnim_klikom(&mut h, m);
    preskoci_naziv(&mut h);
    klikni(&mut h, m + vec2(120.0, 0.0));
    pritisni_s(&mut h, Modifiers::COMMAND, Key::Backspace);
    assert_eq!(broj_objekata(h.state()), 0);
    ponisti(&mut h);
    let Geometrija::Poligon(p) = jedini_objekt(h.state()).geometrija else {
        panic!("poligon");
    };
    assert_eq!(p[0].len(), 4, "poništavanje vraća cijeli lik");
}
