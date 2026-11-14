use super::*;
use slojevi::{Geometrija, NaredbaStabla, Novo, StanjeSloja, Vrh, VrstaGeometrije};

struct Stol {
    u: Uredjivac,
    zdenci: IdSloja,
    crpke: IdSloja,
    kanali: IdSloja,
    zdenac: IdObjekta,
    crpka: IdObjekta,
    kanal: IdObjekta,
}

fn sloj(u: &mut Uredjivac, naziv: &str, vrsta: VrstaGeometrije) -> IdSloja {
    let Ok(Some(Novo::Sloj(s))) = u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: naziv.into(),
        vrsta,
    }) else {
        panic!("sloj");
    };
    s
}

fn objekt(u: &mut Uredjivac, sloj: IdSloja, geometrija: Geometrija) -> IdObjekta {
    let Ok(Some(Novo::Objekt(o))) = u.izvrsi(NaredbaObjekta::Dodaj {
        sloj,
        naziv: "Pokus".into(),
        geometrija,
    }) else {
        panic!("objekt");
    };
    o
}

fn stol() -> Stol {
    let mut u = Uredjivac::novi();
    let zdenci = sloj(&mut u, "Zdenci", VrstaGeometrije::Tocka);
    let crpke = sloj(&mut u, "Crpke", VrstaGeometrije::Tocka);
    let kanali = sloj(&mut u, "Kanali", VrstaGeometrije::Linija);
    let tocka = || Geometrija::Tocka(Vrh::novi(16.0, 45.0));
    let zdenac = objekt(&mut u, zdenci, tocka());
    let crpka = objekt(&mut u, crpke, tocka());
    let kanal = objekt(
        &mut u,
        kanali,
        Geometrija::Linija(vec![Vrh::novi(16.0, 45.0), Vrh::novi(16.1, 45.0)]),
    );
    Stol {
        u,
        zdenci,
        crpke,
        kanali,
        zdenac,
        crpka,
        kanal,
    }
}

fn omoguceni(o: &[Odrediste]) -> Vec<Option<Mjesto>> {
    o.iter().filter(|x| x.omoguceno).map(|x| x.mjesto).collect()
}

#[test]
fn jedan_objekt_ima_sve_stavke_a_vise_samo_premjestanje_i_brisanje() {
    assert!(stavke(0).is_empty());
    assert_eq!(stavke(1).len(), 5);
    assert_eq!(stavke(3), [Stavka::Premjesti, Stavka::Obrisi]);
    let k = KatalogPoruka::hrvatski();
    let nazivi: Vec<String> = stavke(1).iter().map(|s| naziv_stavke(*s, 1, &k)).collect();
    assert_eq!(
        nazivi,
        [
            "Preimenuj",
            "Stil",
            "Premjesti u",
            "Zumiraj na objekt",
            "Obriši"
        ]
    );
    assert_eq!(
        naziv_stavke(Stavka::Obrisi, 3, &k),
        "Obriši 3 odabrana objekta"
    );
    assert_eq!(radnja_stavke(Stavka::Premjesti), None);
    assert_eq!(radnja_stavke(Stavka::Zumiraj), Some(RadnjaKarte::Zumiraj));
}

#[test]
fn odredista_nude_slojeve_iste_vrste_osim_onog_u_kojem_su_svi() {
    let s = stol();
    let k = KatalogPoruka::hrvatski();
    let p = s.u.projekt();
    assert_eq!(
        omoguceni(&odredista(p, &[(s.zdenci, s.zdenac)], &k)),
        [Some(Mjesto::Sloj(s.crpke))]
    );
    // Objekti iz dva sloja: svaki sloj prima onaj koji nije u njemu.
    assert_eq!(
        omoguceni(&odredista(
            p,
            &[(s.zdenci, s.zdenac), (s.crpke, s.crpka)],
            &k
        )),
        [Some(Mjesto::Sloj(s.zdenci)), Some(Mjesto::Sloj(s.crpke))]
    );
    assert!(odredista(p, &[(s.zdenci, s.zdenac), (s.kanali, s.kanal)], &k).is_empty());
    assert!(odredista(p, &[], &k).is_empty());
}

#[test]
fn premjestanje_vise_objekata_je_jedan_korak() {
    let mut s = stol();
    let odabrani = [(s.zdenci, s.zdenac), (s.crpke, s.crpka)];
    let novi = premjesti_objekte(&mut s.u, &odabrani, s.crpke).expect("premješteno");
    assert_eq!(novi, [(s.crpke, s.zdenac), (s.crpke, s.crpka)]);
    assert!(s.u.projekt().objekt(s.crpke, s.zdenac).is_some());
    s.u.ponisti().expect("poništavanje");
    assert!(s.u.projekt().objekt(s.zdenci, s.zdenac).is_some());
}

#[test]
fn zakljucan_sloj_ne_prima_premjestanje() {
    let mut s = stol();
    s.u.postavi_stanje_sloja(s.crpke, StanjeSloja::SamoCitanje)
        .expect("zaključavanje");
    assert_eq!(
        premjesti_objekte(&mut s.u, &[(s.zdenci, s.zdenac)], s.crpke),
        None
    );
    assert!(s.u.projekt().objekt(s.zdenci, s.zdenac).is_some());
}
