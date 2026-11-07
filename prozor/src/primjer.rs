use crate::poruke::KatalogPoruka;
use crate::stanje::{Odabir, StanjePrikaza};
use slojevi::{
    Geometrija, IdMape, IdObjekta, IdSloja, NaredbaObjekta, NaredbaStabla, Novo, Projekt,
    StanjeSloja, Stavka, Uredjivac, Vrh, VrstaGeometrije,
};

pub fn izgradi_primjer_zaslona_a(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
) {
    stanje.prikazi_primjer = true;
    let teren = dodaj_mapu(uredjivac, None, &katalog.tekst("primjer-teren-2026"));
    let (parcele, p2) = dodaj_sloj_parcele(uredjivac, stanje, katalog, teren);
    dodaj_sloj_trase(uredjivac, stanje, katalog, teren);
    dodaj_sloj_tocke(uredjivac, stanje, katalog, teren);
    dodaj_podmapu_parcele_2025(uredjivac, stanje, katalog, teren);
    dodaj_mapu_uvezeno(uredjivac, stanje, katalog);

    stanje.odabrano = Some(Odabir::Objekt(parcele, p2));
    stanje.redni_novog_sloja = 8;
    stanje.redni_nove_mape = 4;
}

fn dodaj_sloj_parcele(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
    roditelj: IdMape,
) -> (IdSloja, IdObjekta) {
    let parcele = dodaj_sloj(
        uredjivac,
        Some(roditelj),
        &katalog.tekst("primjer-parcele"),
        VrstaGeometrije::Poligon,
    );
    stanje.postavi_otvorenost_sloja(parcele, true);
    stanje.ogranicenje_objekata.insert(parcele, 3);
    stanje.ikone_slojeva.insert(
        parcele,
        ("poligon".to_owned(), "narancasti_poligon".to_owned()),
    );

    dodaj_pravokutnik_wgs84(
        uredjivac,
        parcele,
        &katalog.tekst("primjer-parcela-1"),
        0.001_353_7,
        0.001_975_4,
    );
    let p2 = dodaj_pravokutnik_wgs84(
        uredjivac,
        parcele,
        &katalog.tekst("primjer-parcela-2"),
        0.002_830_5,
        0.001_975_4,
    );
    dodaj_pravokutnik_wgs84(
        uredjivac,
        parcele,
        &katalog.tekst("primjer-parcela-3"),
        0.001_095_9,
        0.001_975_4,
    );

    for redni in 4..=12 {
        let naziv = katalog.s_rednim("primjer-parcela-ostalo", redni);
        dodaj_pravokutnik_wgs84(uredjivac, parcele, &naziv, 0.001_5, 0.001_2);
    }
    (parcele, p2)
}

fn dodaj_sloj_trase(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
    roditelj: IdMape,
) {
    let trase = dodaj_sloj(
        uredjivac,
        Some(roditelj),
        &katalog.tekst("primjer-trase"),
        VrstaGeometrije::Linija,
    );
    stanje
        .ikone_slojeva
        .insert(trase, ("linija".to_owned(), "plava_linija".to_owned()));
    let geometrija = Geometrija::Linija(vec![Vrh::novi(16.0, 45.8), Vrh::novi(16.003, 45.8)]);
    uredjivac
        .izvrsi(NaredbaObjekta::Dodaj {
            sloj: trase,
            naziv: katalog.tekst("primjer-trasa-1"),
            geometrija,
        })
        .expect("dodavanje trase");
}

fn dodaj_sloj_tocke(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
    roditelj: IdMape,
) {
    let tocke = dodaj_sloj(
        uredjivac,
        Some(roditelj),
        &katalog.tekst("primjer-tocke"),
        VrstaGeometrije::Tocka,
    );
    stanje
        .ikone_slojeva
        .insert(tocke, ("tocka".to_owned(), "crvena_tocka".to_owned()));
    for redni in 1..=2 {
        let naziv = katalog.s_rednim("primjer-tocka", redni);
        dodaj_tocku(uredjivac, tocke, &naziv, redni as f64 * 0.001);
    }
}

fn dodaj_podmapu_parcele_2025(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
    roditelj: IdMape,
) {
    let p2025 = dodaj_mapu(
        uredjivac,
        Some(roditelj),
        &katalog.tekst("primjer-parcele-2025"),
    );
    stanje.postavi_otvorenost_mape(p2025, false);
    stanje.postavi_vidljivost_mape(p2025, false);
    dodaj_sloj(
        uredjivac,
        Some(p2025),
        &katalog.tekst("primjer-stare-parcele"),
        VrstaGeometrije::Poligon,
    );
    dodaj_sloj(
        uredjivac,
        Some(p2025),
        &katalog.tekst("primjer-granice"),
        VrstaGeometrije::Linija,
    );
}

fn dodaj_mapu_uvezeno(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
) {
    let uvezeno = dodaj_mapu(uredjivac, None, &katalog.tekst("primjer-uvezeno"));

    let obilazak = dodaj_sloj(
        uredjivac,
        Some(uvezeno),
        &katalog.tekst("primjer-obilazak"),
        VrstaGeometrije::Tocka,
    );
    stanje
        .oznake_slojeva
        .insert(obilazak, katalog.tekst("primjer-oznaka-kml"));
    stanje
        .ikone_slojeva
        .insert(obilazak, ("datoteka".to_owned(), "obicno".to_owned()));
    for redni in 1..=23 {
        let naziv = katalog.s_rednim("primjer-obilazak-tocka", redni);
        dodaj_tocku(uredjivac, obilazak, &naziv, redni as f64 * 0.0005);
    }

    let nasipi = dodaj_sloj(
        uredjivac,
        Some(uvezeno),
        &katalog.tekst("primjer-nasipi"),
        VrstaGeometrije::Poligon,
    );
    stanje
        .oznake_slojeva
        .insert(nasipi, katalog.tekst("primjer-oznaka-gpkg"));
    stanje
        .ikone_slojeva
        .insert(nasipi, ("baza".to_owned(), "obicno".to_owned()));
    stanje.postavi_vidljivost_sloja(nasipi, false);
    uredjivac
        .postavi_stanje_sloja(nasipi, StanjeSloja::SamoCitanje)
        .expect("zaključavanje sloja nasipi");
}

pub fn izgradi_stablo_opterecenja(uredjivac: &mut Uredjivac, katalog: &KatalogPoruka) {
    let mut trenutna_mapa = None;
    for razina in 1..=8 {
        let naziv = katalog.s_rednim("nova-mapa-naziv", razina);
        let id = dodaj_mapu(uredjivac, trenutna_mapa, &naziv);
        for indeks_sloja in 0..26 {
            let redni = (razina - 1) * 26 + indeks_sloja + 1;
            let naziv_sloja = katalog.s_rednim("novi-sloj-naziv", redni);
            let sloj = dodaj_sloj(uredjivac, Some(id), &naziv_sloja, VrstaGeometrije::Tocka);
            let naziv_tocke = katalog.s_rednim("primjer-tocka", redni);
            dodaj_tocku(uredjivac, sloj, &naziv_tocke, (redni as f64) * 0.0001);
        }
        trenutna_mapa = Some(id);
    }
}

#[must_use]
pub fn ukupno_slojeva_u_mapi(projekt: &Projekt, id_mape: IdMape) -> usize {
    let Some(mapa) = projekt.mapa(id_mape) else {
        return 0;
    };
    mapa.djeca
        .iter()
        .filter(|d| matches!(*d, Stavka::Sloj(_)))
        .count()
}

#[must_use]
pub fn ukupno_slojeva_u_projektu(projekt: &Projekt) -> usize {
    projekt.slojevi().count()
}

#[must_use]
pub fn najveca_dubina_mapa(projekt: &Projekt) -> usize {
    projekt
        .korijen()
        .iter()
        .filter_map(|stavka| match *stavka {
            Stavka::Mapa(id) => Some(dubina_mape(projekt, id)),
            Stavka::Sloj(_) => None,
        })
        .max()
        .unwrap_or(0)
}

fn dubina_mape(projekt: &Projekt, id_mape: IdMape) -> usize {
    let Some(mapa) = projekt.mapa(id_mape) else {
        return 0;
    };
    let dubina_djece = mapa
        .djeca
        .iter()
        .filter_map(|stavka| match *stavka {
            Stavka::Mapa(podmapa) => Some(dubina_mape(projekt, podmapa)),
            Stavka::Sloj(_) => None,
        })
        .max()
        .unwrap_or(0);
    1 + dubina_djece
}

fn dodaj_mapu(uredjivac: &mut Uredjivac, u: Option<IdMape>, naziv: &str) -> IdMape {
    let Some(Novo::Mapa(id)) = uredjivac
        .izvrsi(NaredbaStabla::DodajMapu {
            u,
            naziv: naziv.to_owned(),
        })
        .expect("dodavanje mape")
    else {
        panic!("očekivana nova mapa");
    };
    id
}

fn dodaj_sloj(
    uredjivac: &mut Uredjivac,
    u: Option<IdMape>,
    naziv: &str,
    vrsta: VrstaGeometrije,
) -> IdSloja {
    let Some(Novo::Sloj(id)) = uredjivac
        .izvrsi(NaredbaStabla::DodajSloj {
            u,
            naziv: naziv.to_owned(),
            vrsta,
        })
        .expect("dodavanje sloja")
    else {
        panic!("očekivan novi sloj");
    };
    id
}

fn dodaj_pravokutnik_wgs84(
    uredjivac: &mut Uredjivac,
    sloj: IdSloja,
    naziv: &str,
    d_duzina: f64,
    d_sirina: f64,
) -> IdObjekta {
    let duz0 = 16.0;
    let sir0 = 45.8;
    let geometrija = Geometrija::Poligon(vec![vec![
        Vrh::novi(duz0, sir0),
        Vrh::novi(duz0 + d_duzina, sir0),
        Vrh::novi(duz0 + d_duzina, sir0 + d_sirina),
        Vrh::novi(duz0, sir0 + d_sirina),
    ]]);
    let Some(Novo::Objekt(id)) = uredjivac
        .izvrsi(NaredbaObjekta::Dodaj {
            sloj,
            naziv: naziv.to_owned(),
            geometrija,
        })
        .expect("dodavanje poligona")
    else {
        panic!("očekivan novi objekt");
    };
    id
}

fn dodaj_tocku(uredjivac: &mut Uredjivac, sloj: IdSloja, naziv: &str, pomak: f64) {
    let geometrija = Geometrija::Tocka(Vrh::novi(16.0 + pomak, 45.8 + pomak));
    uredjivac
        .izvrsi(NaredbaObjekta::Dodaj {
            sloj,
            naziv: naziv.to_owned(),
            geometrija,
        })
        .expect("dodavanje točke");
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn primjer_zaslona_a_gradi_tocan_broj_slojeva_i_objekata() {
        let mut uredjivac = Uredjivac::novi();
        let mut stanje = StanjePrikaza::novo();
        let katalog = KatalogPoruka::hrvatski();

        izgradi_primjer_zaslona_a(&mut uredjivac, &mut stanje, &katalog);
        let projekt = uredjivac.projekt();
        assert_eq!(projekt.korijen().len(), 2);

        let Stavka::Mapa(teren) = projekt.korijen()[0] else {
            panic!("očekivana mapa");
        };
        let Stavka::Mapa(uvezeno) = projekt.korijen()[1] else {
            panic!("očekivana mapa");
        };
        assert_eq!(ukupno_slojeva_u_mapi(projekt, teren), 3);
        assert_eq!(ukupno_slojeva_u_mapi(projekt, uvezeno), 2);
        assert_eq!(ukupno_slojeva_u_projektu(projekt), 7);
        assert_eq!(najveca_dubina_mapa(projekt), 2);
    }

    #[test]
    fn stablo_opterecenja_ima_208_slojeva_i_dubinu_8() {
        let mut uredjivac = Uredjivac::novi();
        let katalog = KatalogPoruka::hrvatski();

        izgradi_stablo_opterecenja(&mut uredjivac, &katalog);
        let projekt = uredjivac.projekt();
        assert_eq!(ukupno_slojeva_u_projektu(projekt), 208);
        assert_eq!(najveca_dubina_mapa(projekt), 8);
    }
}
