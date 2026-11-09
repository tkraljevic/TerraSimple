use crate::aplikacija::{Aplikacija, PostavkeAplikacije};
use crate::poruke::KatalogPoruka;
use crate::stanje::{Alat, Odabir};
use crate::tema::Razmaci;
use crate::trazilica::*;
use eframe::egui::{Event, Key, Modifiers, PointerButton, Pos2, Rect, pos2, vec2};
use egui_kittest::Harness;
use koordinate::RazlogRaspona;
use slojevi::{
    Geometrija, IdObjekta, IdSloja, NaredbaObjekta, NaredbaStabla, Novo, Vrh, VrstaGeometrije,
};

fn sredina_karte() -> Pos2 {
    pos2(
        Razmaci::SIRINA_PROZORA / 2.0,
        (Razmaci::VISINA_IZBORNIKA + Razmaci::VISINA_PROZORA - Razmaci::VISINA_STATUSA) / 2.0,
    )
}

fn okvir_karte_ekran() -> Rect {
    Rect::from_min_max(
        pos2(0.0, Razmaci::VISINA_IZBORNIKA),
        pos2(
            Razmaci::SIRINA_PROZORA,
            Razmaci::VISINA_PROZORA - Razmaci::VISINA_STATUSA,
        ),
    )
}

fn stvori_harness(app: Aplikacija) -> Harness<'static, Aplikacija> {
    let mut h = Harness::builder()
        .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
        .with_step_dt(0.05)
        .with_context_setup(Aplikacija::postavi_kontekst)
        .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
    h.run_steps(3);
    h
}

fn pritisni_precac(h: &mut Harness<'_, Aplikacija>, key: Key, command: bool) {
    let modifiers = if command {
        Modifiers::COMMAND
    } else {
        Modifiers::NONE
    };
    h.input_mut().events.push(Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    });
    h.run_steps(1);
    h.input_mut().events.push(Event::Key {
        key,
        physical_key: None,
        pressed: false,
        repeat: false,
        modifiers,
    });
    h.run_steps(1);
}

fn tipkaj(h: &mut Harness<'_, Aplikacija>, tekst: &str) {
    h.input_mut().events.push(Event::Text(tekst.to_string()));
    h.run_steps(2);
}

fn klikni_misem(h: &mut Harness<'_, Aplikacija>, m: Pos2) {
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
    h.run_steps(5);
}

fn dodaj_testni_sloj(app: &mut Aplikacija, naziv: &str, vrsta: VrstaGeometrije) -> IdSloja {
    let Ok(Some(Novo::Sloj(s))) = app.uredjivac.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: naziv.to_string(),
        vrsta,
    }) else {
        panic!("greška pri dodavanju sloja");
    };
    s
}

fn dodaj_testni_objekt(
    app: &mut Aplikacija,
    sloj: IdSloja,
    naziv: &str,
    geometrija: Geometrija,
) -> IdObjekta {
    let Ok(Some(Novo::Objekt(o))) = app.uredjivac.izvrsi(NaredbaObjekta::Dodaj {
        sloj,
        naziv: naziv.to_string(),
        geometrija,
    }) else {
        panic!("greška pri dodavanju objekta");
    };
    o
}

#[test]
fn pretraga_koordinata_pomice_kartu_i_postavlja_privremenu_oznaku() {
    let app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });
    let mut h = stvori_harness(app);

    let sredina_prije = h.state().karta.pogled().expect("pogled prije").sredina;

    // Otvaranje tražilice preko Cmd+F
    pritisni_precac(&mut h, Key::F, true);
    assert!(h.state().trazilica.otvorena);

    // Upis koordinata Osijeka
    tipkaj(&mut h, "45.5547, 18.6955");
    assert_eq!(h.state().trazilica.tekst, "45.5547, 18.6955");

    // Enter primjenjuje prvi rezultat
    pritisni_precac(&mut h, Key::Enter, false);
    h.run_steps(3);

    // Karta je stvarno premještena: sredina pogleda prije i poslije se razlikuju
    let sredina_poslije = h.state().karta.pogled().expect("pogled poslije").sredina;
    assert!(
        (sredina_poslije.duzina - 18.6955).abs() < 1e-4,
        "sredina duzina {}",
        sredina_poslije.duzina
    );
    assert!(
        (sredina_poslije.sirina - 45.5547).abs() < 1e-4,
        "sredina sirina {}",
        sredina_poslije.sirina
    );
    assert!(
        (sredina_prije.duzina - sredina_poslije.duzina).abs() > 0.01
            || (sredina_prije.sirina - sredina_poslije.sirina).abs() > 0.01,
        "karta se morala pomaknuti"
    );

    // Privremena oznaka je postavljena
    let oznaka = h.state().trazilica.privremena_oznaka;
    assert!(oznaka.is_some(), "privremena oznaka mora biti postavljena");
    let [d, s] = oznaka.unwrap();
    assert!((d - 18.6955).abs() < 1e-4);
    assert!((s - 45.5547).abs() < 1e-4);

    // Escape uklanja oznaku
    pritisni_precac(&mut h, Key::Escape, false);
    assert!(h.state().trazilica.privremena_oznaka.is_none());
}

#[test]
fn pretraga_objekta_po_nazivu_bez_kvacica_odabire_objekt_i_objekt_cijeli_stane() {
    let mut app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });

    let sloj_id = dodaj_testni_sloj(&mut app, "Česta", VrstaGeometrije::Linija);
    let linija_vrhovi = vec![Vrh::novi(15.98, 45.81), Vrh::novi(16.02, 45.85)];
    let objekt_id = dodaj_testni_objekt(
        &mut app,
        sloj_id,
        "Kuća prizemnica",
        Geometrija::Linija(linija_vrhovi.clone()),
    );

    let mut h = stvori_harness(app);

    pritisni_precac(&mut h, Key::F, true);
    tipkaj(&mut h, "kuca");
    pritisni_precac(&mut h, Key::Enter, false);
    h.run_steps(3);

    assert_eq!(
        h.state().stanje.odabrano,
        Some(Odabir::Objekt(sloj_id, objekt_id))
    );

    // Provjera da je sredina pogleda sredina objekta
    let sredina = h.state().karta.pogled().expect("pogled").sredina;
    let ocekivana_duzina = (15.98 + 16.02) / 2.0;
    let ocekivana_sirina = (45.81 + 45.85) / 2.0;
    assert!(
        (sredina.duzina - ocekivana_duzina).abs() < 1e-3,
        "sredina duzina {}, ocekivano {}",
        sredina.duzina,
        ocekivana_duzina
    );
    assert!(
        (sredina.sirina - ocekivana_sirina).abs() < 1e-3,
        "sredina sirina {}, ocekivano {}",
        sredina.sirina,
        ocekivana_sirina
    );

    // Svi vrhovi objekta moraju stati unutar okvira karte
    let okvir = okvir_karte_ekran();
    for v in &linija_vrhovi {
        let tocka = h.state().karta.tocka_u_ekran(*v, okvir);
        assert!(tocka.is_some(), "vrh {v:?} mora biti unutar okvira karte");
    }
}

#[test]
fn pretraga_poligona_cijeli_stane_u_okvir_karte() {
    let mut app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });

    let sloj_id = dodaj_testni_sloj(&mut app, "Parcele", VrstaGeometrije::Poligon);
    let poligon_vrhovi = vec![
        Vrh::novi(16.40, 43.50),
        Vrh::novi(16.50, 43.50),
        Vrh::novi(16.45, 43.55),
    ];
    let objekt_id = dodaj_testni_objekt(
        &mut app,
        sloj_id,
        "Poligon Split",
        Geometrija::Poligon(vec![poligon_vrhovi.clone()]),
    );

    let mut h = stvori_harness(app);

    pritisni_precac(&mut h, Key::F, true);
    tipkaj(&mut h, "split");
    pritisni_precac(&mut h, Key::Enter, false);
    h.run_steps(3);

    assert_eq!(
        h.state().stanje.odabrano,
        Some(Odabir::Objekt(sloj_id, objekt_id))
    );

    let sredina = h.state().karta.pogled().expect("pogled").sredina;
    let ocekivana_duzina = (16.40 + 16.50) / 2.0;
    let ocekivana_sirina = (43.50 + 43.55) / 2.0;
    assert!((sredina.duzina - ocekivana_duzina).abs() < 1e-3);
    assert!((sredina.sirina - ocekivana_sirina).abs() < 1e-3);

    let okvir = okvir_karte_ekran();
    for v in &poligon_vrhovi {
        let tocka = h.state().karta.tocka_u_ekran(*v, okvir);
        assert!(
            tocka.is_some(),
            "vrh poligona {v:?} mora biti unutar okvira karte"
        );
    }
}

#[test]
fn pretraga_sloja_po_nazivu_odabire_sloj_i_pomice_kartu() {
    let mut app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });

    let sloj_id = dodaj_testni_sloj(&mut app, "Vodovodne cijevi", VrstaGeometrije::Linija);
    dodaj_testni_objekt(
        &mut app,
        sloj_id,
        "Cijev 1",
        Geometrija::Linija(vec![Vrh::novi(18.68, 45.54), Vrh::novi(18.70, 45.56)]),
    );

    let mut h = stvori_harness(app);
    let sredina_prije = h.state().karta.pogled().expect("pogled prije").sredina;

    pritisni_precac(&mut h, Key::F, true);
    tipkaj(&mut h, "vodovod");
    pritisni_precac(&mut h, Key::Enter, false);
    h.run_steps(3);

    assert_eq!(h.state().stanje.odabrano, Some(Odabir::Sloj(sloj_id)));

    // Provjera da se karta pomaknula na obuhvat sloja u Osijeku
    let sredina_poslije = h.state().karta.pogled().expect("pogled poslije").sredina;
    assert!((sredina_poslije.duzina - 18.69).abs() < 1e-2);
    assert!((sredina_poslije.sirina - 45.55).abs() < 1e-2);
    assert!(
        (sredina_prije.duzina - sredina_poslije.duzina).abs() > 0.01
            || (sredina_prije.sirina - sredina_poslije.sirina).abs() > 0.01,
        "karta se morala pomaknuti na sloj"
    );
}

#[test]
fn backspace_u_trazilici_ne_brise_odabrani_objekt_ni_tocku_crteza() {
    let mut app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });

    let sloj_id = dodaj_testni_sloj(&mut app, "Sloj za test", VrstaGeometrije::Linija);
    let objekt_id = dodaj_testni_objekt(
        &mut app,
        sloj_id,
        "Objekt 1",
        Geometrija::Linija(vec![Vrh::novi(15.98, 45.81), Vrh::novi(15.99, 45.82)]),
    );

    app.stanje.odabrano = Some(Odabir::Objekt(sloj_id, objekt_id));
    app.stanje.alat = Alat::Linija;

    let mut h = stvori_harness(app);

    // Kliknemo dvije točke za crtanje linije na karti
    let centar = sredina_karte();
    klikni_misem(&mut h, centar);
    klikni_misem(&mut h, centar + vec2(40.0, 30.0));
    assert!(h.state_mut().karta.u_tijeku(), "crtanje mora biti u tijeku");
    assert_eq!(
        h.state_mut()
            .karta
            .karta()
            .crtanje()
            .vrhovi_crteza()
            .map(|v| v.len()),
        Some(2),
        "crtež mora imati točno 2 točke prije otvaranja tražilice"
    );

    // Otvorimo i fokusiramo tražilicu
    pritisni_precac(&mut h, Key::F, true);
    tipkaj(&mut h, "abc");
    assert_eq!(h.state().trazilica.tekst, "abc");

    // Pritisak Backspace u polju tražilice
    pritisni_precac(&mut h, Key::Backspace, false);

    // Provjere:
    // 1. Objekt i dalje postoji (nije obrisan!)
    let projekt = h.state().uredjivac.projekt();
    let sloj = projekt.sloj(sloj_id).expect("sloj postoji");
    assert!(
        sloj.objekti.contains_key(&objekt_id),
        "objekt ne smije biti obrisan pritiskom na Backspace u tražilici"
    );

    // 2. Crtanje je i dalje u tijeku s točno obje točke (točka crteža nije obrisana!)
    assert!(
        h.state_mut().karta.u_tijeku(),
        "crtanje na karti ne smije biti prekinuto"
    );
    assert_eq!(
        h.state_mut()
            .karta
            .karta()
            .crtanje()
            .vrhovi_crteza()
            .map(|v| v.len()),
        Some(2),
        "crtež i dalje mora imati obje točke nakon Backspacea u tražilici"
    );
}

#[test]
fn pretraga_nepostojeceg_pojma_ne_rusi_program() {
    let app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });
    let mut h = stvori_harness(app);

    pritisni_precac(&mut h, Key::F, true);
    tipkaj(&mut h, "nepostojeci pojam 12345");

    let katalog = KatalogPoruka::hrvatski();
    let (rezultati, greska) = pronadji_rezultate(
        &h.state().trazilica.tekst,
        h.state().uredjivac.projekt(),
        &katalog,
    );
    assert!(rezultati.is_empty());
    assert!(greska.is_none());

    // Enter ne radi ništa
    pritisni_precac(&mut h, Key::Enter, false);
    assert!(h.state().stanje.odabrano.is_none());
    assert!(h.state().trazilica.privremena_oznaka.is_none());
}

#[test]
fn prikaz_razloga_greske_raspona() {
    let katalog = KatalogPoruka::hrvatski();
    let app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });

    // 95, 18 -> širina preko 90
    let (rez1, greska1) = pronadji_rezultate("95, 18", app.uredjivac.projekt(), &katalog);
    assert!(rez1.is_empty());
    assert_eq!(greska1, Some(RazlogRaspona::Sirina));
    assert_eq!(
        tekst_greske_raspona(&katalog, greska1.unwrap()),
        "Zemljopisna širina mora biti između -90° i 90°."
    );

    // 45 65 00 N 18 00 00 E -> minute preko 60
    let (rez2, greska2) =
        pronadji_rezultate("45 65 00 N 18 00 00 E", app.uredjivac.projekt(), &katalog);
    assert!(rez2.is_empty());
    assert_eq!(greska2, Some(RazlogRaspona::Minute));
    assert_eq!(
        tekst_greske_raspona(&katalog, greska2.unwrap()),
        "Minute moraju biti manje od 60."
    );

    // Provjera preostalih varijanti greške raspona
    assert_eq!(
        tekst_greske_raspona(&katalog, RazlogRaspona::Duzina),
        "Zemljopisna dužina mora biti između -180° i 180°."
    );
    assert_eq!(
        tekst_greske_raspona(&katalog, RazlogRaspona::Sekunde),
        "Sekunde moraju biti manje od 60."
    );
    assert_eq!(
        tekst_greske_raspona(&katalog, RazlogRaspona::Htrs),
        "Koordinate HTRS96/TM su izvan raspona za Hrvatsku."
    );
    assert_eq!(
        tekst_greske_raspona(&katalog, RazlogRaspona::PreviseBrojeva),
        "Previše brojeva u koordinati."
    );
}

#[test]
fn normalizacija_slova_i_kvacica() {
    assert_eq!(
        normaliziraj_za_trazenje("ČeŠĆa ĐaKa ŽaLa"),
        "cesca daka zala"
    );
    assert_eq!(normaliziraj_za_trazenje("ZAGREB"), "zagreb");
}

#[test]
fn pretraga_htrs96_tm_koordinata() {
    let app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });
    let mut h = stvori_harness(app);

    pritisni_precac(&mut h, Key::F, true);
    tipkaj(&mut h, "676543 5047321");

    let katalog = KatalogPoruka::hrvatski();
    let (rezultati, _) = pronadji_rezultate(
        &h.state().trazilica.tekst,
        h.state().uredjivac.projekt(),
        &katalog,
    );
    assert!(!rezultati.is_empty());

    pritisni_precac(&mut h, Key::Enter, false);
    assert!(h.state().trazilica.privremena_oznaka.is_some());
}

#[test]
fn zamjena_redoslijeda_nudi_obje_opcije() {
    let app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });
    let katalog = KatalogPoruka::hrvatski();
    let (rezultati, _) = pronadji_rezultate("18.6955 45.5547", app.uredjivac.projekt(), &katalog);
    assert!(rezultati.len() >= 2);
}

#[test]
fn klik_na_ikonu_otvara_trazilicu() {
    let app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });
    let mut h = stvori_harness(app);

    let x = Razmaci::RUB + 15.0;
    let y = Razmaci::VISINA_IZBORNIKA + 12.0 + 20.0;
    klikni_misem(&mut h, pos2(x, y));

    assert!(h.state().trazilica.otvorena);
}

#[test]
fn predmemoriranje_rezultata_trazilice() {
    let mut app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });
    let sloj_id = dodaj_testni_sloj(&mut app, "Veliki sloj", VrstaGeometrije::Tocka);

    // Dodajemo nekoliko tisuća objekata za provjeru predmemoriranja
    for i in 0..1000 {
        let _ = dodaj_testni_objekt(
            &mut app,
            sloj_id,
            &format!("Tocka {i}"),
            Geometrija::Tocka(Vrh::novi(16.0, 45.0)),
        );
    }

    let katalog = KatalogPoruka::hrvatski();
    let mut trazilica = StanjeTrazilice::default();
    trazilica.tekst = "tocka".to_string();

    // Prvi poziv izračunava rezultate
    let (rez1, _) = trazilica.rezultati(&app.uredjivac, &katalog);
    assert_eq!(rez1.len(), 8);

    // Drugi poziv s istim tekstom i projektom vraća spremljene rezultate odmah
    let (rez2, _) = trazilica.rezultati(&app.uredjivac, &katalog);
    assert_eq!(rez2.len(), 8);

    // Poništavanje čisti predmemoriju
    assert!(trazilica.ponisti_ili_zatvori());
    assert_eq!(trazilica.tekst, "");
}

/// Odluke (9. 11. 2026.): prikaz koordinate u popisu mora se pročitati natrag u istu točku,
/// i na jugu i na zapadu. Provjerava se funkcija koju prozor stvarno koristi.
#[test]
fn prikaz_koordinate_cita_se_natrag_u_istu_tocku() {
    let katalog = KatalogPoruka::hrvatski();
    let wgs = |sirina, duzina| koordinate::PrepoznataKoordinata {
        duzina,
        sirina,
        sustav: koordinate::Sustav::Wgs84,
        zamijenjen_redoslijed: false,
        izvorni_htrs: None,
    };
    for (sirina, duzina, ocekivano) in [
        (45.5547, 18.6955, "45,5547° S 18,6955° I"),
        (-33.86, 151.2, "33,8600° J 151,2000° I"),
        (51.5, -0.12, "51,5000° S 0,1200° Z"),
        (-34.6, -58.38, "34,6000° J 58,3800° Z"),
    ] {
        let tekst = formatiraj_koordinatu(&katalog, &wgs(sirina, duzina));
        assert_eq!(tekst, ocekivano);
        let koordinate::IshodPrepoznavanja::Prepoznato(t) =
            koordinate::prepoznaj_koordinate(&tekst)
        else {
            panic!("prikaz {tekst} se ne čita");
        };
        assert_eq!(t.len(), 1, "{tekst}");
        assert!((t[0].sirina - sirina).abs() < 1e-4, "{tekst}");
        assert!((t[0].duzina - duzina).abs() < 1e-4, "{tekst}");
    }
    let zamijenjena = koordinate::PrepoznataKoordinata {
        zamijenjen_redoslijed: true,
        ..wgs(45.5547, 18.6955)
    };
    assert_eq!(
        formatiraj_koordinatu(&katalog, &zamijenjena),
        format!(
            "45,5547° S 18,6955° I ({})",
            katalog.tekst("trazi-zamijenjen-redoslijed")
        )
    );
    let htrs = koordinate::PrepoznataKoordinata {
        sustav: koordinate::Sustav::Htrs96Tm,
        izvorni_htrs: Some([676_543.0, 5_047_321.5]),
        ..wgs(45.5, 18.7)
    };
    assert_eq!(
        formatiraj_koordinatu(&katalog, &htrs),
        "676543,00 m E 5047321,50 m N (HTRS96/TM)"
    );
    assert_eq!(
        formatiraj_koordinatu(
            &katalog,
            &koordinate::PrepoznataKoordinata {
                zamijenjen_redoslijed: true,
                ..htrs
            }
        ),
        format!(
            "5047321,50 m N 676543,00 m E ({})",
            katalog.tekst("trazi-zamijenjen-htrs")
        )
    );
    let bez_izvora = koordinate::PrepoznataKoordinata {
        izvorni_htrs: None,
        ..htrs
    };
    assert_eq!(formatiraj_koordinatu(&katalog, &bez_izvora), "");
    let hdks = koordinate::PrepoznataKoordinata {
        sustav: koordinate::Sustav::HdksZona5,
        ..htrs
    };
    assert_eq!(formatiraj_koordinatu(&katalog, &hdks), "");
}

/// Spremljeni rezultati vrijede samo dok se ne promijeni projekt.
#[test]
fn rezultati_se_osvjeze_kad_se_projekt_promijeni() {
    let mut app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });
    let sloj = dodaj_testni_sloj(&mut app, "Bunari", VrstaGeometrije::Tocka);
    let objekt = dodaj_testni_objekt(
        &mut app,
        sloj,
        "Stari bunar",
        Geometrija::Tocka(Vrh::novi(18.0, 45.0)),
    );
    let katalog = KatalogPoruka::hrvatski();
    let mut trazilica = StanjeTrazilice::default();
    trazilica.tekst = "stari".into();
    assert_eq!(trazilica.rezultati(&app.uredjivac, &katalog).0.len(), 1);
    app.uredjivac
        .izvrsi(NaredbaObjekta::Izmijeni {
            sloj,
            objekt,
            izmjena: slojevi::IzmjenaObjekta::Naziv("Novi bunar".into()),
        })
        .unwrap();
    assert!(trazilica.rezultati(&app.uredjivac, &katalog).0.is_empty());
    trazilica.tekst = "novi".into();
    assert_eq!(trazilica.rezultati(&app.uredjivac, &katalog).0.len(), 1);
}

/// Escape briše i samo oznaku na karti ili samo upisani tekst.
#[test]
fn escape_brise_oznaku_ili_tekst_i_kad_je_samo_jedno() {
    let mut samo_oznaka = StanjeTrazilice::default();
    samo_oznaka.privremena_oznaka = Some([18.0, 45.0]);
    assert!(samo_oznaka.ponisti_ili_zatvori());
    assert_eq!(samo_oznaka.privremena_oznaka, None);
    let mut samo_tekst = StanjeTrazilice::default();
    samo_tekst.tekst = "x".into();
    assert!(samo_tekst.ponisti_ili_zatvori());
    assert_eq!(samo_tekst.tekst, "");
    assert!(!StanjeTrazilice::default().ponisti_ili_zatvori());
}

/// Objekt u sloju unutar dvije zatvorene mape: odabir ih obje otvori, da se vidi u ploči.
#[test]
fn odabir_objekta_otvara_sve_mape_iznad_njega() {
    let mut app = Aplikacija::nova(PostavkeAplikacije {
        podloga_s_mreze: false,
        ..Default::default()
    });
    let Ok(Some(Novo::Mapa(vanjska))) = app.uredjivac.izvrsi(NaredbaStabla::DodajMapu {
        u: None,
        naziv: "Vanjska".into(),
    }) else {
        panic!("nema mape");
    };
    let Ok(Some(Novo::Mapa(unutarnja))) = app.uredjivac.izvrsi(NaredbaStabla::DodajMapu {
        u: Some(vanjska),
        naziv: "Unutarnja".into(),
    }) else {
        panic!("nema mape");
    };
    let Ok(Some(Novo::Sloj(sloj))) = app.uredjivac.izvrsi(NaredbaStabla::DodajSloj {
        u: Some(unutarnja),
        naziv: "Duboko".into(),
        vrsta: VrstaGeometrije::Tocka,
    }) else {
        panic!("nema sloja");
    };
    dodaj_testni_objekt(
        &mut app,
        sloj,
        "Skriveni zdenac",
        Geometrija::Tocka(Vrh::novi(18.0, 45.0)),
    );
    app.stanje.postavi_otvorenost_mape(vanjska, false);
    app.stanje.postavi_otvorenost_mape(unutarnja, false);
    let mut h = stvori_harness(app);

    pritisni_precac(&mut h, Key::F, true);
    tipkaj(&mut h, "zdenac");
    pritisni_precac(&mut h, Key::Enter, false);
    h.run_steps(3);

    assert!(h.state().stanje.je_mapa_otvorena(vanjska));
    assert!(h.state().stanje.je_mapa_otvorena(unutarnja));
}
