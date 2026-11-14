use super::*;
use crate::{NaziviSlojeva, procitaj, procitaj_tekst, uvezi};
use slojevi::{
    IzmjenaObjekta, NaredbaObjekta, NaredbaStabla, Natpis, Novo, Uredjivac, ZapisObjekta,
};

fn nazivi() -> NaziviSlojeva {
    NaziviSlojeva {
        tocke: "Točke".into(),
        linije: "Linije".into(),
        poligoni: "Poligoni".into(),
        bez_naziva: "Bez naziva".into(),
    }
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

fn objekt(u: &mut Uredjivac, sloj: IdSloja, naziv: &str, g: Geometrija) -> slojevi::IdObjekta {
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

const PLAVA: Boja = Boja {
    r: 0x15,
    g: 0x65,
    b: 0xC0,
    a: 255,
};

/// Mapa „Teren” s kanalom (svoj stil i opis), točkom s visinom i likom s rupom.
fn projekt() -> Uredjivac {
    let mut u = Uredjivac::novi();
    let Ok(Some(Novo::Mapa(m))) = u.izvrsi(NaredbaStabla::DodajMapu {
        u: None,
        naziv: "Teren & okolica".into(),
    }) else {
        panic!("mapa");
    };
    let kanali = sloj(&mut u, Some(m), "Kanali", VrstaGeometrije::Linija);
    let linija = vec![
        Vrh::novi(18.6, 45.5),
        Vrh::novi(18.61, 45.51),
        Vrh::novi(18.63, 45.5),
    ];
    let k = objekt(&mut u, kanali, "Kanal <Bobota>", Geometrija::Linija(linija));
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: kanali,
        objekt: k,
        izmjena: IzmjenaObjekta::Opis("Glavni kanal".into()),
    })
    .unwrap();
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: kanali,
        objekt: k,
        izmjena: IzmjenaObjekta::Stil(StilObjekta {
            boja_crte: Some(PLAVA),
            vrsta_crte: Some(VrstaCrte::Isprekidana),
            ..StilObjekta::default()
        }),
    })
    .unwrap();
    let tocke = sloj(&mut u, Some(m), "Točke", VrstaGeometrije::Tocka);
    let mut t = Vrh::novi(18.62, 45.52);
    t.visina = Some(88.5);
    objekt(&mut u, tocke, "Crpka", Geometrija::Tocka(t));
    let likovi = sloj(&mut u, None, "Obuhvati", VrstaGeometrije::Poligon);
    let vanjski = vec![
        Vrh::novi(18.0, 45.0),
        Vrh::novi(18.1, 45.0),
        Vrh::novi(18.1, 45.1),
        Vrh::novi(18.0, 45.1),
    ];
    let rupa = vec![
        Vrh::novi(18.04, 45.04),
        Vrh::novi(18.06, 45.04),
        Vrh::novi(18.06, 45.06),
    ];
    objekt(
        &mut u,
        likovi,
        "Retencija",
        Geometrija::Poligon(vec![vanjski, rupa]),
    );
    let mut stil = Stil {
        vrsta_ispune: VrstaIspune::Srafura,
        ..Stil::default()
    };
    stil.natpis.kurziv = true;
    u.izvrsi(NaredbaStabla::PostaviStil { sloj: likovi, stil })
        .unwrap();
    u
}

#[test]
fn pregled_broji_objekte_i_gubitke() {
    let u = projekt();
    let p = pregledaj(u.projekt(), OpsegIzvoza::Projekt, &nazivi());
    assert_eq!((p.tocaka, p.linija, p.poligona, p.mapa), (1, 1, 1, 1));
    assert_eq!(p.objekata(), 3);
    assert_eq!((p.isprekidane, p.srafure), (1, 1));
    assert_eq!(p.natpisi_bez_tocaka, 2, "kanali i obuhvati");
    assert_eq!(p.kurziv, 1);
    assert!(!p.drugi_sustav);
    assert!(p.atributi.is_empty());
}

#[test]
fn izvoz_i_ponovni_uvoz_daju_isto() {
    let u = projekt();
    let tekst = kml(u.projekt(), OpsegIzvoza::Projekt, "Projekt", &nazivi());
    assert!(tekst.contains("Kanal &lt;Bobota&gt;"));
    assert!(tekst.contains("<Folder><name>Teren &amp; okolica</name><Folder><name>Kanali</name>"));
    assert!(
        !tekst.contains("<name>Točke</name>"),
        "sloj iz uvoza ide ravno u mapu"
    );
    assert!(tekst.contains("18.62,45.52,88.5"));
    assert!(tekst.contains(&format!("<color>{}</color>", boja_u_kml(PLAVA))));

    let d = procitaj_tekst(&tekst).expect("KML");
    assert!(d.gubici.prazno(), "{:?}", d.gubici);
    let mut novi = Uredjivac::novi();
    let s = uvezi(&mut novi, &d, "proba", &nazivi()).expect("uvoz");
    assert_eq!(s.objekata, 3);
    let projekt = novi.projekt();
    let objekti: Vec<&Objekt> = projekt
        .slojevi()
        .flat_map(|(_, s)| s.objekti.values())
        .collect();
    let kanal = objekti
        .iter()
        .find(|o| o.naziv == "Kanal <Bobota>")
        .unwrap();
    assert_eq!(kanal.opis, "Glavni kanal");
    let (izvorni, _) = u
        .projekt()
        .slojevi()
        .find(|(_, s)| s.naziv == "Kanali")
        .unwrap();
    let izvorni = &u
        .projekt()
        .sloj(izvorni)
        .unwrap()
        .objekti
        .values()
        .next()
        .unwrap()
        .geometrija;
    assert_eq!(&kanal.geometrija, izvorni);
    let (sid, sloj_kanala) = projekt
        .slojevi()
        .find(|(_, s)| s.objekti.values().any(|o| o.naziv == "Kanal <Bobota>"))
        .unwrap();
    assert_eq!(
        kanal.stil.na_stilu_sloja(&sloj_kanala.stil).boja_crte,
        PLAVA
    );
    let _ = sid;
    let crpka = objekti.iter().find(|o| o.naziv == "Crpka").unwrap();
    assert!(matches!(crpka.geometrija, Geometrija::Tocka(v) if v.visina == Some(88.5)));
    let lik = objekti.iter().find(|o| o.naziv == "Retencija").unwrap();
    assert!(matches!(&lik.geometrija, Geometrija::Poligon(p) if p.len() == 2));
}

#[test]
fn stil_tocke_i_lika_ide_u_kml() {
    let mut s = Stil {
        oblik_znaka: OblikZnaka::Zvijezda,
        velicina_znaka_mm: 6.0,
        vrsta_ispune: VrstaIspune::Bez,
        ..Stil::default()
    };
    s.boja_crte.a = 0;
    s.natpis = Natpis {
        prikazi: false,
        ..Natpis::default()
    };
    let t = tijelo_stila(&s, VrstaGeometrije::Tocka);
    assert!(t.contains("star.png") && t.contains("<scale>2</scale>"));
    assert!(t.contains("<LabelStyle><color>ff242120</color><scale>0</scale>"));
    let p = tijelo_stila(&s, VrstaGeometrije::Poligon);
    assert!(p.contains("<fill>0</fill><outline>0</outline>"));
    for o in [OblikZnaka::Krug, OblikZnaka::Kvadrat, OblikZnaka::Trokut] {
        assert!(ikona(o).ends_with(".png"));
    }
    assert_eq!(boja_u_kml(PLAVA), "ffc06515");
}

fn mape(p: &Projekt) -> std::collections::BTreeMap<IdMape, slojevi::Mapa> {
    let mut izlaz = std::collections::BTreeMap::new();
    let mut stog: Vec<Stavka> = p.korijen().to_vec();
    while let Some(s) = stog.pop() {
        if let Stavka::Mapa(id) = s {
            let m = p.mapa(id).unwrap().clone();
            stog.extend(m.djeca.iter().copied());
            izlaz.insert(id, m);
        }
    }
    izlaz
}

#[test]
fn jedan_sloj_i_atributi_iz_geopackagea() {
    let u = projekt();
    let (id, _) = u
        .projekt()
        .slojevi()
        .find(|(_, s)| s.naziv == "Obuhvati")
        .unwrap();
    let mut projekt = u.projekt().clone();
    let o = *projekt.sloj(id).unwrap().objekti.keys().next().unwrap();
    let mut zapis = ZapisObjekta {
        geometrija: None,
        fid: 1,
        nepoznati: Default::default(),
    };
    zapis
        .nepoznati
        .insert("povrsina".into(), Vrijednost::Broj(12.5));
    zapis
        .nepoznati
        .insert("oznaka".into(), Vrijednost::Tekst("R&1".into()));
    zapis.nepoznati.insert("broj".into(), Vrijednost::Cijeli(7));
    zapis.nepoznati.insert("prazno".into(), Vrijednost::Prazno);
    zapis
        .nepoznati
        .insert("slika".into(), Vrijednost::Bajtovi(vec![1, 2]));
    let mut zapisi = projekt.zapisi().clone();
    zapisi.slojevi.insert(
        id,
        slojevi::IzvorSloja {
            srs_id: 3765,
            sql: String::new(),
            stupci: vec![],
            indeksi: vec![],
            okidaci: vec![],
            stil: Stil::default(),
            sustav: Sustav::Htrs96Tm,
            tablica: "obuhvati".into(),
            fid_stupac: "fid".into(),
            geometrija_stupac: "geom".into(),
            objekti: [(o, zapis)].into(),
            nepodrzani: 0,
        },
    );
    projekt = Projekt::iz_zapisa(
        projekt.korijen().to_vec(),
        mape(&projekt),
        projekt.slojevi().map(|(i, s)| (i, s.clone())).collect(),
        zapisi,
    )
    .unwrap();
    let opseg = OpsegIzvoza::Stavka(Stavka::Sloj(id));
    let p = pregledaj(&projekt, opseg, &nazivi());
    assert_eq!(p.objekata(), 1);
    assert!(p.drugi_sustav);
    assert_eq!(p.atributi.len(), 4);
    assert_eq!(p.izostavljene_vrijednosti, 1);
    let tekst = kml(&projekt, opseg, "Obuhvati", &nazivi());
    assert!(tekst.contains("<Folder><name>Obuhvati</name>"));
    assert!(tekst.contains("<Data name=\"oznaka\"><value>R&amp;1</value></Data>"));
    assert!(tekst.contains("<Data name=\"povrsina\"><value>12.5</value></Data>"));
    assert!(!tekst.contains("Kanal"));
}

#[test]
fn datoteka_kml_i_kmz_se_ponovno_cita() {
    let u = projekt();
    let mapa = std::env::temp_dir().join(format!("terrasimple-izvoz-{}", std::process::id()));
    std::fs::create_dir_all(&mapa).unwrap();
    for ime in ["teren.kml", "teren.KMZ"] {
        let staza = mapa.join(ime);
        let p = izvezi(
            u.projekt(),
            OpsegIzvoza::Projekt,
            "Teren",
            &nazivi(),
            &staza,
        )
        .unwrap();
        assert_eq!(p.objekata(), 3);
        let d = procitaj(&staza).expect("ponovno čitanje");
        assert_eq!(d.korijen.naziv, "Teren");
        assert_eq!(d.korijen.broj_objekata(), 3);
        assert!(!staza.with_extension("izvoz-u-tijeku").exists());
    }
    let bajtovi = std::fs::read(mapa.join("teren.KMZ")).unwrap();
    assert!(bajtovi.starts_with(b"PK"));
    let nema = mapa.join("nema").join("x.kml");
    assert!(matches!(
        izvezi(u.projekt(), OpsegIzvoza::Projekt, "T", &nazivi(), &nema),
        Err(GreskaIzvoza::Pisanje)
    ));
    // Mapa na mjestu datoteke: pisanje uspije, a preimenovanje ne, pa privremene ne smije ostati.
    let zauzeto = mapa.join("zauzeto.kml");
    std::fs::create_dir_all(&zauzeto).unwrap();
    assert!(izvezi(u.projekt(), OpsegIzvoza::Projekt, "T", &nazivi(), &zauzeto).is_err());
    assert!(!zauzeto.with_extension("izvoz-u-tijeku").exists());
    std::fs::remove_dir_all(&mapa).unwrap();
}

#[test]
fn posebni_znakovi_zatvoren_prsten_i_nepostojeca_stavka() {
    assert_eq!(bijeg("a&<>\"'b"), "a&amp;&lt;&gt;&quot;&apos;b");
    let zatvoren = vec![
        Vrh::novi(18.0, 45.0),
        Vrh::novi(18.1, 45.0),
        Vrh::novi(18.1, 45.1),
        Vrh::novi(18.0, 45.0),
    ];
    assert_eq!(prsten(&zatvoren).matches("18,45").count(), 2);
    let u = projekt();
    for s in [Stavka::Mapa(IdMape(999)), Stavka::Sloj(IdSloja(999))] {
        assert_eq!(
            pregledaj(u.projekt(), OpsegIzvoza::Stavka(s), &nazivi()),
            PregledIzvoza::default()
        );
    }
    assert!(matches!(
        GreskaIzvoza::from(zip::result::ZipError::FileNotFound),
        GreskaIzvoza::Pisanje
    ));
}
