//! Ponašanje naredbi i poništavanja, onako kako ih koriste sučelje i MCP.

use slojevi::{
    Boja, Geometrija, Greska, IdMape, IdObjekta, IdSloja, IzmjenaObjekta, NaredbaObjekta,
    NaredbaStabla, Novo, Projekt, StanjeSloja, Stavka, Stil, StilObjekta, Uredjivac, Vrh,
    VrstaGeometrije,
};

fn mapa(u: &mut Uredjivac, roditelj: Option<IdMape>, naziv: &str) -> IdMape {
    let n = NaredbaStabla::DodajMapu {
        u: roditelj,
        naziv: naziv.into(),
    };
    match u.izvrsi(n) {
        Ok(Some(Novo::Mapa(id))) => id,
        r => panic!("{r:?}"),
    }
}

fn sloj(u: &mut Uredjivac, roditelj: Option<IdMape>, vrsta: VrstaGeometrije) -> IdSloja {
    let n = NaredbaStabla::DodajSloj {
        u: roditelj,
        naziv: "Sloj".into(),
        vrsta,
    };
    match u.izvrsi(n) {
        Ok(Some(Novo::Sloj(id))) => id,
        r => panic!("{r:?}"),
    }
}

fn tocka() -> Geometrija {
    Geometrija::Tocka(Vrh::novi(18.69, 45.55))
}

fn linija(pomak: f64) -> Geometrija {
    Geometrija::Linija(vec![
        Vrh::novi(18.0 + pomak, 45.0),
        Vrh::novi(18.1 + pomak, 45.1),
    ])
}

fn dodaj(s: IdSloja, geometrija: Geometrija) -> NaredbaObjekta {
    NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: "Objekt".into(),
        geometrija,
    }
}

fn objekt(u: &mut Uredjivac, s: IdSloja, g: Geometrija) -> IdObjekta {
    match u.izvrsi(dodaj(s, g)) {
        Ok(Some(Novo::Objekt(id))) => id,
        r => panic!("{r:?}"),
    }
}

#[test]
fn prazan_projekt_nema_sto_ponistiti() {
    let mut u = Uredjivac::novi();
    assert_eq!(u.ponisti(), Ok(false));
    assert_eq!(u.ponovi(), Ok(false));
    assert!(!u.moze_natrag());
    assert!(!u.moze_naprijed());
}

#[test]
fn poredak_u_stablu_i_poznati_roditelji() {
    let mut u = Uredjivac::novi();
    let m = mapa(&mut u, None, "Teren 2026");
    let pod = mapa(&mut u, Some(m), "Parcele");
    let s = sloj(&mut u, Some(pod), VrstaGeometrije::Poligon);
    let p = u.projekt();
    assert_eq!(p.korijen(), &[Stavka::Mapa(m)]);
    assert_eq!(p.mapa(m).expect("mapa").djeca, vec![Stavka::Mapa(pod)]);
    assert_eq!(p.mapa(pod).expect("mapa").roditelj, Some(m));
    assert_eq!(p.sloj(s).expect("sloj").roditelj, Some(pod));
    assert_eq!(p.slojevi().count(), 1);
}

#[test]
fn nazivi_mape_i_sloja_ne_smiju_biti_prazni_a_objekta_smiju() {
    let mut u = Uredjivac::novi();
    let n = NaredbaStabla::DodajMapu {
        u: None,
        naziv: "  ".into(),
    };
    assert_eq!(u.izvrsi(n), Err(Greska::PrazanNaziv));
    let n = NaredbaStabla::DodajSloj {
        u: None,
        naziv: String::new(),
        vrsta: VrstaGeometrije::Tocka,
    };
    assert_eq!(u.izvrsi(n), Err(Greska::PrazanNaziv));
    let s = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let n = NaredbaStabla::Preimenuj {
        stavka: Stavka::Sloj(s),
        naziv: "\t".into(),
    };
    assert_eq!(u.izvrsi(n), Err(Greska::PrazanNaziv));
    let bez_naziva = NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: String::new(),
        geometrija: tocka(),
    };
    assert!(u.izvrsi(bez_naziva).is_ok());
}

#[test]
fn neuspjela_naredba_ne_mijenja_projekt_ni_povijest() {
    let mut u = Uredjivac::novi();
    let s = sloj(&mut u, None, VrstaGeometrije::Linija);
    let id = objekt(&mut u, s, linija(0.0));
    let prije = u.projekt().clone();
    let nema_mape = IdMape(999);
    let nema_sloja = IdSloja(999);
    let nema_objekta = IdObjekta(999);
    let losi: Vec<(Result<Option<Novo>, Greska>, Greska)> = vec![
        (
            u.izvrsi(NaredbaStabla::DodajMapu {
                u: Some(nema_mape),
                naziv: "X".into(),
            }),
            Greska::NepoznataMapa,
        ),
        (
            u.izvrsi(NaredbaStabla::DodajSloj {
                u: Some(nema_mape),
                naziv: "X".into(),
                vrsta: VrstaGeometrije::Tocka,
            }),
            Greska::NepoznataMapa,
        ),
        (
            u.izvrsi(NaredbaStabla::Obrisi {
                stavka: Stavka::Sloj(nema_sloja),
            }),
            Greska::NepoznatSloj,
        ),
        (
            u.izvrsi(NaredbaStabla::Obrisi {
                stavka: Stavka::Mapa(nema_mape),
            }),
            Greska::NepoznataMapa,
        ),
        (
            u.izvrsi(NaredbaStabla::Preimenuj {
                stavka: Stavka::Mapa(nema_mape),
                naziv: "X".into(),
            }),
            Greska::NepoznataMapa,
        ),
        (
            u.izvrsi(NaredbaStabla::Preimenuj {
                stavka: Stavka::Sloj(nema_sloja),
                naziv: "X".into(),
            }),
            Greska::NepoznatSloj,
        ),
        (
            u.izvrsi(NaredbaStabla::Premjesti {
                stavka: Stavka::Sloj(s),
                u: Some(nema_mape),
                polozaj: 0,
            }),
            Greska::NepoznataMapa,
        ),
        (
            u.izvrsi(NaredbaStabla::PostaviStil {
                sloj: nema_sloja,
                stil: Stil::default(),
            }),
            Greska::NepoznatSloj,
        ),
        (u.izvrsi(dodaj(nema_sloja, tocka())), Greska::NepoznatSloj),
        (
            u.izvrsi(dodaj(s, tocka())),
            Greska::KrivaVrsta {
                ocekivana: VrstaGeometrije::Linija,
                dobivena: VrstaGeometrije::Tocka,
            },
        ),
        (
            u.izvrsi(dodaj(s, Geometrija::Linija(vec![Vrh::novi(18.0, 45.0)]))),
            Greska::PremaloVrhova,
        ),
        (
            u.izvrsi(NaredbaObjekta::Izmijeni {
                sloj: nema_sloja,
                objekt: id,
                izmjena: IzmjenaObjekta::Naziv("X".into()),
            }),
            Greska::NepoznatSloj,
        ),
        (
            u.izvrsi(NaredbaObjekta::Izmijeni {
                sloj: s,
                objekt: nema_objekta,
                izmjena: IzmjenaObjekta::Naziv("X".into()),
            }),
            Greska::NepoznatObjekt,
        ),
        (
            u.izvrsi(NaredbaObjekta::Izmijeni {
                sloj: s,
                objekt: id,
                izmjena: IzmjenaObjekta::Geometrija(tocka()),
            }),
            Greska::KrivaVrsta {
                ocekivana: VrstaGeometrije::Linija,
                dobivena: VrstaGeometrije::Tocka,
            },
        ),
        (
            u.izvrsi(NaredbaObjekta::Obrisi {
                sloj: s,
                objekt: nema_objekta,
            }),
            Greska::NepoznatObjekt,
        ),
        (
            u.izvrsi(NaredbaObjekta::Obrisi {
                sloj: nema_sloja,
                objekt: id,
            }),
            Greska::NepoznatSloj,
        ),
        (
            u.izvrsi(NaredbaObjekta::Premjesti {
                iz: s,
                u: nema_sloja,
                objekt: id,
            }),
            Greska::NepoznatSloj,
        ),
        (
            u.izvrsi(NaredbaObjekta::Premjesti {
                iz: s,
                u: s,
                objekt: nema_objekta,
            }),
            Greska::NepoznatObjekt,
        ),
    ];
    for (rezultat, ocekivano) in losi {
        assert_eq!(rezultat, Err(ocekivano));
    }
    assert_eq!(u.projekt(), &prije);
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(
        u.projekt(),
        &Projekt::default(),
        "povijest je ostala od dvije naredbe"
    );
    assert_eq!(u.ponisti(), Ok(false));
}

#[test]
fn mapa_ne_moze_u_samu_sebe_ni_u_podmapu() {
    let mut u = Uredjivac::novi();
    let m = mapa(&mut u, None, "A");
    let pod = mapa(&mut u, Some(m), "B");
    let dublje = mapa(&mut u, Some(pod), "C");
    let prije = u.projekt().clone();
    for cilj in [m, pod, dublje] {
        let n = NaredbaStabla::Premjesti {
            stavka: Stavka::Mapa(m),
            u: Some(cilj),
            polozaj: 0,
        };
        assert_eq!(u.izvrsi(n), Err(Greska::MapaUSebe));
    }
    assert_eq!(u.projekt(), &prije);
    let n = NaredbaStabla::Premjesti {
        stavka: Stavka::Mapa(dublje),
        u: None,
        polozaj: 0,
    };
    assert_eq!(u.izvrsi(n), Ok(None));
    assert_eq!(
        u.projekt().korijen(),
        &[Stavka::Mapa(dublje), Stavka::Mapa(m)]
    );
}

#[test]
fn premjestanje_cuva_redoslijed_a_ponistavanje_ga_vraca() {
    let mut u = Uredjivac::novi();
    let a = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let b = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let c = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let m = mapa(&mut u, None, "M");
    let prije = u.projekt().clone();
    let n = NaredbaStabla::Premjesti {
        stavka: Stavka::Sloj(b),
        u: Some(m),
        polozaj: usize::MAX,
    };
    u.izvrsi(n).expect("premještanje");
    assert_eq!(
        u.projekt().korijen(),
        &[Stavka::Sloj(a), Stavka::Sloj(c), Stavka::Mapa(m)]
    );
    assert_eq!(u.projekt().sloj(b).expect("sloj").roditelj, Some(m));
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.projekt(), &prije, "b je opet u sredini");
}

#[test]
fn brisanje_mape_vadi_sav_sadrzaj_a_ponistavanje_ga_vraca_s_istim_brojevima() {
    let mut u = Uredjivac::novi();
    let m = mapa(&mut u, None, "M");
    let pod = mapa(&mut u, Some(m), "P");
    let s = sloj(&mut u, Some(pod), VrstaGeometrije::Tocka);
    let id = objekt(&mut u, s, tocka());
    let prije = u.projekt().clone();
    u.izvrsi(NaredbaStabla::Obrisi {
        stavka: Stavka::Mapa(m),
    })
    .expect("brisanje");
    assert!(u.projekt().mapa(pod).is_none());
    assert!(u.projekt().sloj(s).is_none());
    assert!(u.projekt().korijen().is_empty());
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.projekt(), &prije);
    assert!(u.projekt().objekt(s, id).is_some());
}

#[test]
fn nova_naredba_brise_ponavljanje() {
    let mut u = Uredjivac::novi();
    sloj(&mut u, None, VrstaGeometrije::Tocka);
    assert_eq!(u.ponisti(), Ok(true));
    assert!(u.moze_naprijed());
    sloj(&mut u, None, VrstaGeometrije::Tocka);
    assert!(!u.moze_naprijed());
    assert_eq!(u.ponovi(), Ok(false));
}

#[test]
fn ponovljeni_objekt_ima_isti_broj_a_novi_nikad_ne_koristi_stari() {
    let mut u = Uredjivac::novi();
    let s = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let id = objekt(&mut u, s, tocka());
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.ponovi(), Ok(true));
    assert!(u.projekt().objekt(s, id).is_some());
    assert_eq!(u.ponisti(), Ok(true));
    let novi = objekt(&mut u, s, tocka());
    assert_ne!(novi, id);
}

#[test]
fn visina_prezivi_izmjenu_naziva_i_ponistavanje() {
    let mut u = Uredjivac::novi();
    let s = sloj(&mut u, None, VrstaGeometrije::Linija);
    let visoka = Geometrija::Linija(vec![
        Vrh {
            visina: Some(525.4),
            ..Vrh::novi(18.0, 45.0)
        },
        Vrh {
            visina: Some(530.0),
            ..Vrh::novi(18.1, 45.0)
        },
    ]);
    let id = objekt(&mut u, s, visoka.clone());
    let n = NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: id,
        izmjena: IzmjenaObjekta::Naziv("Trasa".into()),
    };
    u.izvrsi(n).expect("naziv");
    let o = u.projekt().objekt(s, id).expect("objekt");
    assert_eq!(o.naziv, "Trasa");
    assert_eq!(o.geometrija, visoka);
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.projekt().objekt(s, id).expect("objekt").naziv, "Objekt");
}

#[test]
fn zakljucan_sloj_ne_prima_izmjene_objekata_i_ne_ponistava_ih() {
    let mut u = Uredjivac::novi();
    let s = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let id = objekt(&mut u, s, tocka());
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: id,
        izmjena: IzmjenaObjekta::Naziv("Novo".into()),
    })
    .expect("naziv");
    for stanje in [StanjeSloja::SamoCitanje, StanjeSloja::USukobu] {
        u.postavi_stanje_sloja(s, stanje).expect("stanje");
        let prije = u.projekt().clone();
        assert_eq!(u.izvrsi(dodaj(s, tocka())), Err(Greska::SlojZakljucan));
        assert_eq!(
            u.izvrsi(NaredbaObjekta::Obrisi {
                sloj: s,
                objekt: id
            }),
            Err(Greska::SlojZakljucan)
        );
        assert_eq!(u.ponisti(), Err(Greska::SlojZakljucan));
        assert_eq!(u.projekt(), &prije);
        assert!(u.moze_natrag(), "povijest nije izgubljena");
    }
    u.postavi_stanje_sloja(s, StanjeSloja::CekaSlanje)
        .expect("stanje");
    assert_eq!(u.ponisti(), Ok(true), "čim se otključa, poništavanje radi");
    assert_eq!(u.projekt().objekt(s, id).expect("objekt").naziv, "Objekt");
    assert_eq!(
        u.postavi_stanje_sloja(IdSloja(999), StanjeSloja::Uredivo),
        Err(Greska::NepoznatSloj)
    );
}

#[test]
fn zakljucan_sloj_mozes_preimenovati_premjestiti_i_maknuti_iz_projekta() {
    let mut u = Uredjivac::novi();
    let s = sloj(&mut u, None, VrstaGeometrije::Tocka);
    u.postavi_stanje_sloja(s, StanjeSloja::SamoCitanje)
        .expect("stanje");
    let stavka = Stavka::Sloj(s);
    assert!(
        u.izvrsi(NaredbaStabla::Preimenuj {
            stavka,
            naziv: "Nasipi".into()
        })
        .is_ok()
    );
    assert!(
        u.izvrsi(NaredbaStabla::PostaviStil {
            sloj: s,
            stil: Stil::default()
        })
        .is_ok()
    );
    assert!(u.izvrsi(NaredbaStabla::Obrisi { stavka }).is_ok());
    assert_eq!(u.ponisti(), Ok(true));
    let sloj_nakon = u.projekt().sloj(s).expect("sloj");
    assert_eq!(
        sloj_nakon.stanje,
        StanjeSloja::SamoCitanje,
        "stanje se vraća sa slojem"
    );
}

#[test]
fn premjestanje_objekta_trazi_istu_vrstu_i_oba_otkljucana_sloja() {
    let mut u = Uredjivac::novi();
    let a = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let b = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let linije = sloj(&mut u, None, VrstaGeometrije::Linija);
    let id = objekt(&mut u, a, tocka());
    let u_linije = NaredbaObjekta::Premjesti {
        iz: a,
        u: linije,
        objekt: id,
    };
    assert_eq!(
        u.izvrsi(u_linije),
        Err(Greska::KrivaVrsta {
            ocekivana: VrstaGeometrije::Linija,
            dobivena: VrstaGeometrije::Tocka
        })
    );
    u.postavi_stanje_sloja(b, StanjeSloja::SamoCitanje)
        .expect("stanje");
    let n = NaredbaObjekta::Premjesti {
        iz: a,
        u: b,
        objekt: id,
    };
    assert_eq!(u.izvrsi(n.clone()), Err(Greska::SlojZakljucan));
    u.postavi_stanje_sloja(b, StanjeSloja::Uredivo)
        .expect("stanje");
    assert_eq!(u.izvrsi(n), Ok(None));
    assert!(u.projekt().objekt(b, id).is_some());
    assert!(u.projekt().objekt(a, id).is_none());
    assert_eq!(u.ponisti(), Ok(true));
    assert!(u.projekt().objekt(a, id).is_some());
}

#[test]
fn skupina_je_jedan_korak_ili_nista() {
    let mut u = Uredjivac::novi();
    let (s, id) = u
        .skupina(|g| {
            let Some(Novo::Sloj(s)) = g.izvrsi(NaredbaStabla::DodajSloj {
                u: None,
                naziv: "Uvoz".into(),
                vrsta: VrstaGeometrije::Tocka,
            })?
            else {
                return Err(Greska::NepoznatSloj);
            };
            assert!(g.projekt().sloj(s).is_some(), "vidi se unutar skupine");
            let id = g.izvrsi(dodaj(s, tocka()))?;
            g.izvrsi(dodaj(s, tocka()))?;
            Ok((s, id))
        })
        .expect("skupina");
    let Some(Novo::Objekt(id)) = id else {
        panic!("{id:?}");
    };
    assert_eq!(u.projekt().sloj(s).expect("sloj").objekti.len(), 2);
    assert!(u.projekt().objekt(s, id).is_some());
    let nakon_uvoza = u.projekt().clone();

    let neuspjeh = u.skupina(|g| {
        g.izvrsi(dodaj(s, tocka()))?;
        g.izvrsi(NaredbaObjekta::Obrisi {
            sloj: s,
            objekt: id,
        })?;
        g.izvrsi(dodaj(s, linija(0.0)))
    });
    assert!(matches!(neuspjeh, Err(Greska::KrivaVrsta { .. })));
    assert_eq!(u.projekt(), &nakon_uvoza, "uspjele naredbe su vraćene");

    assert_eq!(u.skupina(|_| Ok(())), Ok(()));
    assert_eq!(u.ponisti(), Ok(true), "prazna skupina nije korak");
    assert_eq!(
        u.projekt(),
        &Projekt::default(),
        "jedan korak briše cijeli uvoz"
    );
    assert_eq!(u.ponovi(), Ok(true));
    assert_eq!(u.projekt(), &nakon_uvoza);
}

#[test]
fn stil_sloja_i_objekta_se_vracaju() {
    let mut u = Uredjivac::novi();
    let s = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let id = objekt(&mut u, s, tocka());
    let prije = u.projekt().clone();
    let plavo = Boja {
        r: 0,
        g: 0,
        b: 255,
        a: 255,
    };
    let stil = Stil {
        boja_crte: plavo,
        ..Stil::default()
    };
    u.izvrsi(NaredbaStabla::PostaviStil { sloj: s, stil })
        .expect("stil");
    let stil_objekta = StilObjekta {
        debljina_mm: Some(2.0),
        ..StilObjekta::default()
    };
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: id,
        izmjena: IzmjenaObjekta::Stil(stil_objekta),
    })
    .expect("stil objekta");
    let o = u.projekt().objekt(s, id).expect("objekt");
    let stvarni = o
        .stil
        .na_stilu_sloja(&u.projekt().sloj(s).expect("sloj").stil);
    assert_eq!(stvarni.boja_crte, plavo);
    assert_eq!(stvarni.debljina_mm, 2.0);
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.projekt(), &prije);
}

#[test]
fn inacica_sloja_raste_i_pri_ponistavanju_a_tudi_sloj_miruje() {
    let mut u = Uredjivac::novi();
    let a = sloj(&mut u, None, VrstaGeometrije::Tocka);
    let b = sloj(&mut u, None, VrstaGeometrije::Tocka);
    assert_eq!(u.inacica_sloja(a), 0);
    let ukupno = u.inacica();
    objekt(&mut u, a, tocka());
    let nakon = u.inacica_sloja(a);
    assert!(nakon > 0);
    assert!(u.inacica() > ukupno);
    assert_eq!(u.inacica_sloja(b), 0);
    assert_eq!(u.ponisti(), Ok(true));
    assert!(u.inacica_sloja(a) > nakon);
    let stil = u.inacica_sloja(b);
    u.izvrsi(NaredbaStabla::PostaviStil {
        sloj: b,
        stil: Stil::default(),
    })
    .expect("stil");
    assert!(u.inacica_sloja(b) > stil);
}

#[test]
fn preimenovanje_mape_i_sloja_se_vraca() {
    let mut u = Uredjivac::novi();
    let m = mapa(&mut u, None, "Staro");
    let s = sloj(&mut u, Some(m), VrstaGeometrije::Tocka);
    let prije = u.projekt().clone();
    for stavka in [Stavka::Mapa(m), Stavka::Sloj(s)] {
        let naziv = "Novo".to_string();
        u.izvrsi(NaredbaStabla::Preimenuj { stavka, naziv })
            .expect("naziv");
    }
    assert_eq!(u.projekt().mapa(m).expect("mapa").naziv, "Novo");
    assert_eq!(u.projekt().sloj(s).expect("sloj").naziv, "Novo");
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.ponisti(), Ok(true));
    assert_eq!(u.projekt(), &prije);
}

#[test]
fn poruke_postoje_za_svaku_gresku() {
    let sve = [
        (Greska::NepoznataMapa, "mapa"),
        (Greska::NepoznatSloj, "sloj"),
        (Greska::NepoznatObjekt, "objekt"),
        (Greska::PrazanNaziv, "prazan"),
        (Greska::SlojZakljucan, "mijenjati"),
        (Greska::PremaloVrhova, "vrhova"),
        (Greska::MapaUSebe, "samu sebe"),
        (Greska::Koordinata(geometrija::Greska::NijeBroj), "broj"),
        (
            Greska::KrivaVrsta {
                ocekivana: VrstaGeometrije::Tocka,
                dobivena: VrstaGeometrije::Linija,
            },
            "Linija",
        ),
    ];
    for (g, dio) in sve {
        assert!(g.to_string().contains(dio), "{g}");
    }
}

/// Mali generator, da niz bude isti u svakom pokretanju i bez dodatne ovisnosti.
struct Slucajno(u64);

impl Slucajno {
    fn broj(&mut self, do_: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % do_.max(1)
    }
}

/// Bira naredbu nad onim što trenutno postoji, a dio ih namjerno ne uspije.
fn slucajna_naredba(
    u: &mut Uredjivac,
    r: &mut Slucajno,
    mape: &mut Vec<IdMape>,
    slojevi: &mut Vec<IdSloja>,
) -> bool {
    mape.retain(|id| u.projekt().mapa(*id).is_some());
    slojevi.retain(|id| u.projekt().sloj(*id).is_some());
    let m = if mape.is_empty() || r.broj(3) == 0 {
        None
    } else {
        Some(mape[r.broj(mape.len())])
    };
    let s = slojevi.get(r.broj(slojevi.len())).copied();
    let objekti: Vec<IdObjekta> = s
        .and_then(|id| u.projekt().sloj(id))
        .map(|x| x.objekti.keys().copied().collect())
        .unwrap_or_default();
    let o = objekti.get(r.broj(objekti.len())).copied();
    let pomak = r.broj(1000) as f64 * 0.001;
    let ishod = match (r.broj(13), s, o) {
        (0, ..) => u.izvrsi(NaredbaStabla::DodajMapu {
            u: m,
            naziv: "M".into(),
        }),
        (1, ..) => u.izvrsi(NaredbaStabla::DodajSloj {
            u: m,
            naziv: "S".into(),
            vrsta: VrstaGeometrije::Linija,
        }),
        (2 | 3, Some(s), _) => u.izvrsi(dodaj(s, linija(pomak))),
        (4, Some(s), Some(o)) => u.izvrsi(NaredbaObjekta::Izmijeni {
            sloj: s,
            objekt: o,
            izmjena: IzmjenaObjekta::Geometrija(linija(pomak)),
        }),
        (5, Some(s), Some(o)) => u.izvrsi(NaredbaObjekta::Izmijeni {
            sloj: s,
            objekt: o,
            izmjena: IzmjenaObjekta::Naziv(format!("{pomak}")),
        }),
        (6, Some(s), Some(o)) => u.izvrsi(NaredbaObjekta::Obrisi { sloj: s, objekt: o }),
        (7, Some(s), Some(o)) => {
            let cilj = slojevi[r.broj(slojevi.len())];
            u.izvrsi(NaredbaObjekta::Premjesti {
                iz: s,
                u: cilj,
                objekt: o,
            })
        }
        (8, Some(s), _) => u.izvrsi(NaredbaStabla::Premjesti {
            stavka: Stavka::Sloj(s),
            u: m,
            polozaj: r.broj(5),
        }),
        (9, Some(s), _) => u.izvrsi(NaredbaStabla::Obrisi {
            stavka: Stavka::Sloj(s),
        }),
        (10, ..) if !mape.is_empty() => {
            let stavka = Stavka::Mapa(mape[r.broj(mape.len())]);
            u.izvrsi(NaredbaStabla::Premjesti {
                stavka,
                u: m,
                polozaj: r.broj(5),
            })
        }
        (11, ..) if !mape.is_empty() => {
            let stavka = Stavka::Mapa(mape[r.broj(mape.len())]);
            u.izvrsi(NaredbaStabla::Obrisi { stavka })
        }
        (12, Some(s), _) => u.izvrsi(NaredbaStabla::Preimenuj {
            stavka: Stavka::Sloj(s),
            naziv: format!("S{pomak}"),
        }),
        _ => return false,
    };
    match ishod {
        Ok(Some(Novo::Mapa(id))) => mape.push(id),
        Ok(Some(Novo::Sloj(id))) => slojevi.push(id),
        _ => {}
    }
    ishod.is_ok()
}

/// Stablo mora biti ispravno: svaka stavka jednom, roditelj se slaže s popisom, ništa izgubljeno.
fn provjeri_stablo(p: &Projekt) {
    let mut vidjene = 0usize;
    let mut stog: Vec<(Stavka, Option<IdMape>)> = p.korijen().iter().map(|s| (*s, None)).collect();
    while let Some((stavka, roditelj)) = stog.pop() {
        vidjene += 1;
        match stavka {
            Stavka::Mapa(id) => {
                let m = p.mapa(id).expect("mapa iz stabla postoji");
                assert_eq!(m.roditelj, roditelj);
                stog.extend(m.djeca.iter().map(|d| (*d, Some(id))));
            }
            Stavka::Sloj(id) => {
                assert_eq!(
                    p.sloj(id).expect("sloj iz stabla postoji").roditelj,
                    roditelj
                );
            }
        }
    }
    let ukupno = p.slojevi().count() + broj_mapa(p);
    assert_eq!(vidjene, ukupno, "stavka je izvan stabla ili u njemu dvaput");
}

fn broj_mapa(p: &Projekt) -> usize {
    let mut n = 0;
    let mut stog: Vec<Stavka> = p.korijen().to_vec();
    while let Some(s) = stog.pop() {
        if let Stavka::Mapa(id) = s {
            n += 1;
            stog.extend(&p.mapa(id).expect("mapa").djeca);
        }
    }
    n
}

#[test]
fn slucajni_nizovi_se_vracaju_tocno_unatrag_i_unaprijed() {
    for sjeme in 0..40_u64 {
        let mut r = Slucajno(sjeme.wrapping_mul(2_654_435_769).wrapping_add(7));
        let mut u = Uredjivac::novi();
        let (mut mape, mut slojevi) = (Vec::new(), Vec::new());
        let mut stanja = vec![u.projekt().clone()];
        for _ in 0..500 {
            let prije = u.projekt().clone();
            let uspjela = slucajna_naredba(&mut u, &mut r, &mut mape, &mut slojevi);
            provjeri_stablo(u.projekt());
            if uspjela {
                stanja.push(u.projekt().clone());
            } else {
                assert_eq!(
                    u.projekt(),
                    &prije,
                    "neuspjela naredba ne smije ništa promijeniti"
                );
            }
        }
        assert!(
            stanja.len() > 100,
            "niz je prekratak da bi išta dokazao: {}",
            stanja.len()
        );
        for ocekivano in stanja.iter().rev().skip(1) {
            assert_eq!(u.ponisti(), Ok(true));
            assert_eq!(u.projekt(), ocekivano, "poništavanje, sjeme {sjeme}");
            provjeri_stablo(u.projekt());
        }
        for ocekivano in stanja.iter().skip(1) {
            assert_eq!(u.ponovi(), Ok(true));
            assert_eq!(u.projekt(), ocekivano, "ponavljanje, sjeme {sjeme}");
        }
    }
}
