use super::*;

fn tekst(n: usize) -> Tekst {
    Tekst {
        lijevo: (0..n).map(|i| i as f32 * 8.0).collect(),
        sirine: vec![8.0; n],
        sirina: n as f32 * 8.0,
        visina: 14.0,
    }
}

fn ekran() -> Rect {
    Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 800.0))
}

fn ravna(od: [f64; 2], do_: [f64; 2]) -> Put {
    Put::novi(vec![od, do_])
}

#[test]
fn mjesta_na_liniji_su_u_sredini_i_jednako_razmaknuta() {
    assert_eq!(sredista(300.0, 500.0), vec![150.0]);
    assert_eq!(sredista(1000.0, 500.0), vec![250.0, 750.0]);
    let s = sredista(1700.0, 500.0);
    assert_eq!(s.len(), 3);
    assert!((s[0] - (1700.0 - s[2])).abs() < 1e-9, "{s:?}");
}

#[test]
fn razmak_na_tlu_stoji_unutar_iste_razine_zuma() {
    let na_tlu = |mpp: f64| razmak_na_ekranu(mpp) * mpp;
    assert_eq!(na_tlu(3.0), na_tlu(3.9));
    assert!(na_tlu(4.1) > na_tlu(3.9));
    for mpp in [0.3, 1.0, 7.0, 1500.0] {
        let px = razmak_na_ekranu(mpp);
        assert!((500.0..=1000.0).contains(&px), "{mpp} → {px}");
    }
}

#[test]
fn kratka_linija_nema_natpis_a_duga_ga_ima_vodoravno() {
    let t = tekst(10);
    let mut z = Zauzeto::default();
    let kratka = ravna([100.0, 100.0], [170.0, 100.0]);
    assert!(smjesti_uz_liniju(&kratka, 500.0, &t, ekran(), &mut z).is_empty());

    let duga = ravna([100.0, 100.0], [400.0, 100.0]);
    let n = smjesti_uz_liniju(&duga, 500.0, &t, ekran(), &mut z);
    assert_eq!(n.len(), 1);
    assert!(n[0].iter().all(|s| s.kut.abs() < 1e-6));
    assert!((n[0][0].sredina.x - 214.0).abs() < 1e-3);
}

#[test]
fn natpis_nikad_nije_naopako() {
    let t = tekst(6);
    let mut z = Zauzeto::default();
    // Linija nacrtana zdesna nalijevo.
    let ulijevo = ravna([600.0, 300.0], [100.0, 320.0]);
    let n = smjesti_uz_liniju(&ulijevo, 1000.0, &t, ekran(), &mut z);
    let slova = &n[0];
    assert!(slova.iter().all(|s| s.kut.abs() < 0.1), "{slova:?}");
    assert!(slova[0].sredina.x < slova[5].sredina.x);
}

#[test]
fn natpis_se_mice_s_ostrog_zavoja_ili_izostaje() {
    let t = tekst(8);
    let mut z = Zauzeto::default();
    // Pravi kut točno u sredini linije.
    let v = Put::novi(vec![[100.0, 400.0], [300.0, 400.0], [300.0, 200.0]]);
    let n = smjesti_uz_liniju(&v, 1000.0, &t, ekran(), &mut z);
    assert_eq!(n.len(), 1);
    let kutovi: Vec<f32> = n[0].iter().map(|s| s.kut).collect();
    assert!(
        kutovi.windows(2).all(|k| (k[1] - k[0]).abs() < 0.45),
        "{kutovi:?}"
    );

    let mut z = Zauzeto::default();
    let cik_cak: Vec<[f64; 2]> = (0..20)
        .map(|i| {
            [
                100.0 + f64::from(i) * 6.0,
                if i % 2 == 0 { 300.0 } else { 306.0 },
            ]
        })
        .collect();
    assert!(smjesti_uz_liniju(&Put::novi(cik_cak), 1000.0, &t, ekran(), &mut z).is_empty());
}

#[test]
fn duga_linija_dobiva_natpis_vise_puta_ali_samo_na_ekranu() {
    let t = tekst(8);
    let mut z = Zauzeto::default();
    let duga = ravna([-2000.0, 400.0], [3000.0, 400.0]);
    let n = smjesti_uz_liniju(&duga, 500.0, &t, ekran(), &mut z);
    assert_eq!(n.len(), 2);
    assert!(n.iter().flatten().all(|s| ekran().contains(s.sredina)));
}

#[test]
fn natpis_tocke_trazi_slobodnu_stranu() {
    let t = tekst(5);
    let zid = |z: &mut Zauzeto, x: f32, y: f32| {
        z.zauzmi(Rect::from_center_size(pos2(x, y), vec2(30.0, 10.0)));
    };
    let mut z = Zauzeto::default();
    let desno = smjesti_uz_tocku(pos2(300.0, 300.0), 5.0, &t, ekran(), &mut z).unwrap();
    assert!(desno.min.x > 305.0);
    // Isti natpis još jednom više ne stane desno.
    let lijevo = smjesti_uz_tocku(pos2(300.0, 300.0), 5.0, &t, ekran(), &mut z).unwrap();
    assert!(lijevo.max.x < 295.0, "{lijevo:?}");

    zid(&mut z, 530.0, 500.0);
    zid(&mut z, 470.0, 500.0);
    let gore = smjesti_uz_tocku(pos2(500.0, 500.0), 5.0, &t, ekran(), &mut z).unwrap();
    assert!(gore.max.y < 495.0, "{gore:?}");
    let dolje = smjesti_uz_tocku(pos2(500.0, 500.0), 5.0, &t, ekran(), &mut z).unwrap();
    assert!(dolje.min.y > 505.0, "{dolje:?}");
    assert!(smjesti_uz_tocku(pos2(500.0, 500.0), 5.0, &t, ekran(), &mut z).is_none());
}

#[test]
fn natpis_tocke_na_rubu_ekrana_ide_na_drugu_stranu() {
    let t = tekst(5);
    let mut z = Zauzeto::default();
    let r = smjesti_uz_tocku(pos2(990.0, 300.0), 5.0, &t, ekran(), &mut z).unwrap();
    assert!(r.max.x < 990.0);
}

#[test]
fn natpis_lika_samo_kad_stane() {
    let t = tekst(10);
    let mut z = Zauzeto::default();
    assert!(smjesti_u_lik(pos2(300.0, 300.0), 60.0, 100.0, &t, ekran(), &mut z).is_none());
    assert!(smjesti_u_lik(pos2(300.0, 300.0), 200.0, 10.0, &t, ekran(), &mut z).is_none());
    let r = smjesti_u_lik(pos2(300.0, 300.0), 200.0, 100.0, &t, ekran(), &mut z).unwrap();
    assert_eq!(r.center(), pos2(300.0, 300.0));
}

fn unutra(prstenovi: &[Vec<[f64; 2]>], t: [f64; 2]) -> bool {
    sjecista(prstenovi, t[1])
        .iter()
        .filter(|x| **x < t[0])
        .count()
        % 2
        == 1
}

#[test]
fn tocka_lika_je_unutra_i_kod_udubljenog_lika_i_rupe() {
    // Slovo U: sredina okvira pada u prazninu.
    let u = vec![vec![
        [0.0, 0.0],
        [30.0, 0.0],
        [30.0, 30.0],
        [20.0, 30.0],
        [20.0, 10.0],
        [10.0, 10.0],
        [10.0, 30.0],
        [0.0, 30.0],
    ]];
    let (t, sirina) = tocka_unutar(&u).unwrap();
    assert!(unutra(&u, t), "{t:?}");
    assert!(sirina > 0.0);

    let s_rupom = vec![
        vec![[0.0, 0.0], [100.0, 0.0], [100.0, 100.0], [0.0, 100.0]],
        vec![[20.0, 20.0], [80.0, 20.0], [80.0, 80.0], [20.0, 80.0]],
    ];
    let (t, sirina) = tocka_unutar(&s_rupom).unwrap();
    assert!(unutra(&s_rupom, t), "{t:?}");
    assert!((sirina - 20.0).abs() < 1e-6);
    assert!(tocka_unutar(&[]).is_none());
}

#[test]
fn zauzeto_mjesto_pamti_se_preko_celija() {
    let mut z = Zauzeto::default();
    let r = Rect::from_min_max(pos2(50.0, 50.0), pos2(200.0, 80.0));
    assert!(z.slobodno(r));
    z.zauzmi(r);
    assert!(!z.slobodno(Rect::from_min_size(pos2(190.0, 70.0), vec2(5.0, 5.0))));
    assert!(z.slobodno(Rect::from_min_size(pos2(210.0, 70.0), vec2(5.0, 5.0))));
    assert!(z.slobodno(Rect::from_min_size(pos2(-80.0, -80.0), vec2(5.0, 5.0))));
}

#[test]
fn obrub_je_suprotan_boji_slova() {
    let tamna = Boja {
        r: 20,
        g: 20,
        b: 20,
        a: 255,
    };
    let svijetla = Boja {
        r: 250,
        g: 250,
        b: 240,
        a: 255,
    };
    assert_eq!(boja_obruba(tamna).to_srgba_unmultiplied()[0], 255);
    assert!(boja_obruba(svijetla).r() < 100);
}

mod priprema {
    use super::super::*;
    use slojevi::{NaredbaObjekta, NaredbaStabla, Novo, Uredjivac, VrstaGeometrije};

    fn sloj(vrsta: VrstaGeometrije, objekti: Vec<(&str, Geometrija)>) -> (Sloj, Vec<IdObjekta>) {
        let mut u = Uredjivac::novi();
        let Ok(Some(Novo::Sloj(s))) = u.izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: "S".into(),
            vrsta,
        }) else {
            panic!("sloj");
        };
        let mut id = Vec::new();
        for (naziv, geometrija) in objekti {
            if let Ok(Some(Novo::Objekt(o))) = u.izvrsi(NaredbaObjekta::Dodaj {
                sloj: s,
                naziv: naziv.into(),
                geometrija,
            }) {
                id.push(o);
            }
        }
        (u.projekt().sloj(s).cloned().expect("sloj"), id)
    }

    #[test]
    fn linija_bez_naziva_nema_natpis_a_tocka_ostaje_kao_prepreka() {
        let linija = Geometrija::Linija(vec![Vrh::novi(18.0, 45.0), Vrh::novi(18.1, 45.0)]);
        let (s, id) = sloj(
            VrstaGeometrije::Linija,
            vec![("  ", linija.clone()), (" Kanal ", linija)],
        );
        let n = pripremi_natpise(&s, None);
        assert_eq!(n.objekti.len(), 1);
        assert_eq!(n.objekti[0].naziv, "Kanal");
        assert!(pripremi_natpise(&s, Some(id[1])).objekti.is_empty());
        assert_eq!(n.natpis, s.stil.natpis);

        let (t, _) = sloj(
            VrstaGeometrije::Tocka,
            vec![("", Geometrija::Tocka(Vrh::novi(18.0, 45.0)))],
        );
        let n = pripremi_natpise(&t, None);
        assert!(matches!(n.objekti[0].mjesto, Mjesto::Tocka { polumjer, .. } if polumjer > 5.0));
    }

    #[test]
    fn lik_dobiva_tocku_unutra() {
        let kvadrat = vec![
            Vrh::novi(18.0, 45.0),
            Vrh::novi(18.1, 45.0),
            Vrh::novi(18.1, 45.1),
            Vrh::novi(18.0, 45.1),
        ];
        let (s, _) = sloj(
            VrstaGeometrije::Poligon,
            vec![("Jezero", Geometrija::Poligon(vec![kvadrat]))],
        );
        let n = pripremi_natpise(&s, None);
        let Mjesto::Lik {
            sredina,
            sirina_m,
            visina_m,
        } = n.objekti[0].mjesto
        else {
            panic!("lik");
        };
        let o = n.objekti[0].okvir;
        assert!(sredina[0] > o[0] && sredina[0] < o[2]);
        assert!(sirina_m > 7_000.0 && visina_m > 7_000.0);
    }
}
