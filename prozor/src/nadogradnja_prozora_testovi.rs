use super::*;
use nadogradnja::Datoteka;
use std::time::{Duration, Instant};

fn ponuda() -> Ponuda {
    let datoteka = |ime: &str, velicina| Datoteka {
        ime: ime.into(),
        velicina,
        adresa: format!("https://github.com/tkraljevic/TerraSimple/releases/download/v0.0.9/{ime}"),
    };
    Ponuda {
        inacica: Inacica(0, 0, 9),
        stranica: "https://github.com/tkraljevic/TerraSimple/releases/tag/v0.0.9".into(),
        paket: datoteka("TerraSimple-0.0.9-macos-universal.zip", 24_652_928),
        potpis: datoteka("TerraSimple-0.0.9-macos-universal.zip.sig", 89),
    }
}

/// Izvor na koji se ne može spojiti: provjera završi greškom veze, bez izlaska na mrežu.
fn izvor_bez_veze() -> Izvor {
    Izvor {
        popis: "http://127.0.0.1:1/releases".into(),
        prefiks: "http://127.0.0.1:1/download/".into(),
        javni_kljuc: Vec::new(),
    }
}

fn cekaj(n: &mut Nadogradnja, k: &KatalogPoruka) -> StanjeNadogradnje {
    let pocetak = Instant::now();
    loop {
        if let Some(s) = n.primi(k) {
            return s;
        }
        assert!(
            pocetak.elapsed() < Duration::from_secs(20),
            "nit nije završila"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn tekst_i_gumbi_za_svako_stanje() {
    use GumbNadogradnje as G;
    let k = KatalogPoruka::hrvatski();
    let t = Inacica(0, 0, 8);
    let slucajevi = [
        (StanjeNadogradnje::Provjera, "Provjeravam", &[][..]),
        (
            StanjeNadogradnje::Najnovija(t),
            "najnovije izdanje, TerraSimple 0.0.8",
            &[G::Zatvori],
        ),
        (
            StanjeNadogradnje::Dostupna {
                ponuda: ponuda(),
                moze_sam: true,
            },
            "Dostupno je izdanje 0.0.9 alfa (24,7 MB), a instalirano je 0.0.8.",
            &[G::Kasnije, G::StranicaIzdanja, G::Instaliraj],
        ),
        (
            StanjeNadogradnje::Dostupna {
                ponuda: ponuda(),
                moze_sam: false,
            },
            "nije pokrenut iz paketa izdanja",
            &[G::Zatvori, G::StranicaIzdanja],
        ),
        (
            StanjeNadogradnje::BezPaketa {
                inacica: Inacica(0, 0, 9),
                stranica: "https://x".into(),
            },
            "nema potpisanog paketa",
            &[G::Zatvori, G::StranicaIzdanja],
        ),
        (
            StanjeNadogradnje::Preuzimanje(ponuda()),
            "Preuzimam i provjeravam izdanje 0.0.9",
            &[],
        ),
        (
            StanjeNadogradnje::Instalirana(Inacica(0, 0, 9)),
            "Izdanje 0.0.9 je instalirano.",
            &[G::Kasnije, G::PonovnoPokreni],
        ),
        (
            StanjeNadogradnje::Greska {
                poruka: "Ne ide.".into(),
                stranica: None,
            },
            "Ne ide.",
            &[G::Zatvori],
        ),
    ];
    for (stanje, dio, ocekivani) in slucajevi {
        let tekst = poruka(&stanje, t, &k);
        assert!(tekst.contains(dio), "{tekst}");
        assert_eq!(gumbi(&stanje), ocekivani, "{stanje:?}");
        for g in gumbi(&stanje) {
            assert!(
                !k.tekst(kljuc_gumba(*g)).starts_with("nadogradnja-"),
                "{g:?}"
            );
        }
    }
    assert_eq!(k.tekst(kljuc_gumba(G::Zatvori)), "Zatvori");
}

#[test]
fn stranica_izdanja_po_stanju() {
    let p = ponuda();
    assert_eq!(
        stranica(&StanjeNadogradnje::Dostupna {
            ponuda: p.clone(),
            moze_sam: true
        }),
        Some(p.stranica.as_str())
    );
    assert_eq!(
        stranica(&StanjeNadogradnje::BezPaketa {
            inacica: p.inacica,
            stranica: "https://s".into()
        }),
        Some("https://s")
    );
    assert_eq!(
        stranica(&StanjeNadogradnje::Greska {
            poruka: String::new(),
            stranica: Some("https://g".into())
        }),
        Some("https://g")
    );
    assert_eq!(stranica(&StanjeNadogradnje::Provjera), None);
    // Adresa koja nije web stranica se ne otvara.
    otvori_stranicu("file:///etc/passwd");
}

#[test]
fn svaka_greska_ima_svoju_recenicu() {
    let k = KatalogPoruka::hrvatski();
    let greske = [
        Greska::Veza("x".into()),
        Greska::Odgovor(503),
        Greska::NeocekivanOdgovor("x".into()),
        Greska::Prevelika {
            ime: "paket.zip".into(),
            najvise: 1,
        },
        Greska::TudaAdresa("x".into()),
        Greska::KrivPotpis,
        Greska::OstecenPaket("x".into()),
        Greska::NijeIzPaketa("x".into()),
        Greska::Zamjena {
            mjesto: "/Programi".into(),
            detalji: "nema dozvole".into(),
        },
    ];
    let mut recenice = std::collections::BTreeSet::new();
    for e in &greske {
        let StanjeNadogradnje::Greska { poruka, stranica } =
            greska(e, Some("https://s".into()), &k)
        else {
            panic!("treba greška");
        };
        assert!(poruka.ends_with('.'), "{poruka}");
        assert!(!poruka.contains("nadogradnja-greska"), "{poruka}");
        assert_eq!(stranica.as_deref(), Some("https://s"));
        recenice.insert(poruka);
    }
    assert_eq!(recenice.len(), greske.len());
    assert!(recenice.iter().any(|r| r.contains("503")));
    assert!(recenice.iter().any(|r| r.contains("paket.zip")));
    assert!(
        recenice
            .iter()
            .any(|r| r.contains("nema dozvole") && r.contains("/Programi"))
    );
}

#[test]
fn provjera_bez_veze_javlja_gresku_veze() {
    let k = KatalogPoruka::hrvatski();
    let mut n = Nadogradnja::nova(izvor_bez_veze(), Inacica(0, 0, 8), None);
    assert!(n.primi(&k).is_none(), "bez niti nema poruke");
    let ctx = egui::Context::default();
    assert_eq!(n.provjeri(&ctx), StanjeNadogradnje::Provjera);
    assert_eq!(
        cekaj(&mut n, &k),
        StanjeNadogradnje::Greska {
            poruka: k.tekst("nadogradnja-greska-veza"),
            stranica: None
        }
    );
}

#[test]
fn ishodi_provjere_postaju_stanja() {
    let k = KatalogPoruka::hrvatski();
    let n = Nadogradnja::nova(izvor_bez_veze(), Inacica(0, 0, 8), None);
    assert_eq!(
        n.stanje_provjere(Ok(Ishod::Najnovija(Inacica(0, 0, 8))), &k),
        StanjeNadogradnje::Najnovija(Inacica(0, 0, 8))
    );
    assert_eq!(
        n.stanje_provjere(Ok(Ishod::Dostupna(ponuda())), &k),
        StanjeNadogradnje::Dostupna {
            ponuda: ponuda(),
            moze_sam: false
        }
    );
    let bez = Ishod::BezPaketa {
        inacica: Inacica(0, 0, 9),
        stranica: "https://s".into(),
    };
    assert!(matches!(
        n.stanje_provjere(Ok(bez), &k),
        StanjeNadogradnje::BezPaketa { .. }
    ));
}

#[test]
fn bez_mjesta_programa_nema_ugradnje_ni_ponovnog_pokretanja() {
    let mut n = Nadogradnja::nova(izvor_bez_veze(), Inacica(0, 0, 8), None);
    assert!(n.ugradi(&egui::Context::default(), ponuda()).is_none());
    n.pokreni_nakon_izlaska();
    assert!(n.pokreni_nakon_izlaska.is_none());
    n.na_izlasku();
    n.pocisti();
    let za_program = Nadogradnja::za_program();
    assert!(
        za_program.mjesto.is_none(),
        "test ne radi iz paketa izdanja"
    );
    assert_eq!(za_program.trenutna, trenutna_inacica());
    assert_eq!(trenutna_inacica().to_string(), env!("CARGO_PKG_VERSION"));
}

#[test]
fn ugradnja_koja_ne_uspije_vraca_gresku_sa_stranicom() {
    let k = KatalogPoruka::hrvatski();
    let mapa = std::env::temp_dir().join(format!("terrasimple-ugradnja-{}", std::process::id()));
    let exe = mapa.join("TerraSimple.app/Contents/MacOS/TerraSimple");
    std::fs::create_dir_all(exe.parent().expect("mapa")).expect("mapa");
    std::fs::write(&exe, b"stari").expect("program");
    let mjesto = Mjesto::za_program(&exe, Sustav::MacOs).expect("mjesto");
    let mut n = Nadogradnja::nova(izvor_bez_veze(), Inacica(0, 0, 8), Some(mjesto));
    let mut p = ponuda();
    p.paket.adresa = "http://127.0.0.1:1/download/p.zip".into();
    p.potpis.adresa = "http://127.0.0.1:1/download/p.zip.sig".into();
    let ctx = egui::Context::default();
    assert_eq!(
        n.ugradi(&ctx, p.clone()),
        Some(StanjeNadogradnje::Preuzimanje(p.clone()))
    );
    assert_eq!(
        cekaj(&mut n, &k),
        StanjeNadogradnje::Greska {
            poruka: k.tekst("nadogradnja-greska-veza"),
            stranica: Some(p.stranica)
        }
    );
    assert_eq!(std::fs::read(&exe).expect("program"), b"stari");
    n.pokreni_nakon_izlaska();
    assert!(n.pokreni_nakon_izlaska.is_some());
    let _ = std::fs::remove_dir_all(&mapa); // čišćenje nakon testa
}

#[test]
fn zavrsena_ugradnja_trazi_ponovno_pokretanje() {
    let k = KatalogPoruka::hrvatski();
    let mut n = Nadogradnja::nova(izvor_bez_veze(), Inacica(0, 0, 8), None);
    n.pokreni(&egui::Context::default(), || {
        Poruka::Ugradnja(ponuda(), Ok(()))
    });
    assert_eq!(
        cekaj(&mut n, &k),
        StanjeNadogradnje::Instalirana(Inacica(0, 0, 9))
    );
}
