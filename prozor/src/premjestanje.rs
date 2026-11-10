//! Premještanje mapa i slojeva u ploči Slojevi, izbornikom „Premjesti u” ili povlačenjem.

use crate::poruke::KatalogPoruka;
use crate::stanje::{Odabir, StanjePrikaza};
use slojevi::{IdMape, NaredbaStabla, Projekt, Stavka, Uredjivac};

/// Mjesto u podizborniku „Premjesti u”. `mapa: None` znači izvan svih mapa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Odrediste {
    pub mapa: Option<IdMape>,
    pub naziv: String,
    pub uvlaka: usize,
    /// Stavka već jest u toj mapi.
    pub trenutna: bool,
}

/// Mape i slojevi se premještaju, objekti ne.
#[must_use]
pub const fn stavka_odabira(odabir: Odabir) -> Option<Stavka> {
    match odabir {
        Odabir::Mapa(id) => Some(Stavka::Mapa(id)),
        Odabir::Sloj(id) => Some(Stavka::Sloj(id)),
        Odabir::Objekt(_, _) => None,
    }
}

const fn odabir_stavke(stavka: Stavka) -> Odabir {
    match stavka {
        Stavka::Mapa(id) => Odabir::Mapa(id),
        Stavka::Sloj(id) => Odabir::Sloj(id),
    }
}

fn roditelj_stavke(projekt: &Projekt, stavka: Stavka) -> Option<Option<IdMape>> {
    match stavka {
        Stavka::Mapa(id) => projekt.mapa(id).map(|m| m.roditelj),
        Stavka::Sloj(id) => projekt.sloj(id).map(|s| s.roditelj),
    }
}

fn djeca_u(projekt: &Projekt, u: Option<IdMape>) -> Option<&[Stavka]> {
    match u {
        None => Some(projekt.korijen()),
        Some(id) => projekt.mapa(id).map(|m| m.djeca.as_slice()),
    }
}

/// Je li `u` sama `mapa` ili neka njezina podmapa.
fn je_unutar(projekt: &Projekt, mapa: IdMape, u: Option<IdMape>) -> bool {
    let mut sada = u;
    while let Some(id) = sada {
        if id == mapa {
            return true;
        }
        sada = projekt.mapa(id).and_then(|m| m.roditelj);
    }
    false
}

/// Sva mjesta kamo se stavka može premjestiti, redom kao u stablu. Mapa ne nudi samu sebe ni
/// svoje podmape.
#[must_use]
pub fn odredista_premjestanja(
    projekt: &Projekt,
    stavka: Stavka,
    katalog: &KatalogPoruka,
) -> Vec<Odrediste> {
    let roditelj = roditelj_stavke(projekt, stavka).flatten();
    let mut odredista = vec![Odrediste {
        mapa: None,
        naziv: katalog.tekst("premjesti-izvan-mapa"),
        uvlaka: 0,
        trenutna: roditelj.is_none(),
    }];
    let mut za_obilazak: Vec<(Stavka, usize)> =
        projekt.korijen().iter().rev().map(|s| (*s, 0)).collect();
    while let Some((s, dubina)) = za_obilazak.pop() {
        let Stavka::Mapa(id) = s else {
            continue;
        };
        let Some(mapa) = projekt.mapa(id).filter(|_| s != stavka) else {
            continue;
        };
        odredista.push(Odrediste {
            mapa: Some(id),
            naziv: mapa.naziv.clone(),
            uvlaka: dubina,
            trenutna: roditelj == Some(id),
        });
        za_obilazak.extend(mapa.djeca.iter().rev().map(|d| (*d, dubina + 1)));
    }
    odredista
}

/// Kamo ide stavka ispuštena na redak `cilj`: u mapu na kraj, a na sloj ispred njega, u
/// njegovu mapu. Na objekt, na samu sebe ili mapu u vlastitu podmapu ne ide nikamo.
#[must_use]
pub fn odrediste_ispustanja(
    projekt: &Projekt,
    povuceno: Stavka,
    cilj: Odabir,
) -> Option<(Option<IdMape>, usize)> {
    if stavka_odabira(cilj) == Some(povuceno) {
        return None;
    }
    let (u, polozaj) = mjesto_cilja(projekt, cilj)?;
    if let Stavka::Mapa(mapa) = povuceno
        && je_unutar(projekt, mapa, u)
    {
        return None;
    }
    // Stavka se prvo vadi iz popisa, pa se mjesta iza nje pomaknu za jedno.
    let stari = djeca_u(projekt, u)?.iter().position(|s| *s == povuceno);
    let polozaj = match stari {
        Some(i) if i < polozaj && polozaj != usize::MAX => polozaj - 1,
        _ => polozaj,
    };
    Some((u, polozaj))
}

/// Mapa i mjesto u njoj na koje pokazuje redak `cilj`.
fn mjesto_cilja(projekt: &Projekt, cilj: Odabir) -> Option<(Option<IdMape>, usize)> {
    match cilj {
        Odabir::Mapa(id) => Some((Some(id), usize::MAX)),
        Odabir::Sloj(id) => {
            let roditelj = projekt.sloj(id)?.roditelj;
            let polozaj = djeca_u(projekt, roditelj)?
                .iter()
                .position(|s| *s == Stavka::Sloj(id))?;
            Some((roditelj, polozaj))
        }
        Odabir::Objekt(_, _) => None,
    }
}

/// Premješta stavku, ostavlja je odabranom i otvara mape do nje, da se vidi kamo je otišla.
pub fn premjesti_stavku(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    stavka: Stavka,
    u: Option<IdMape>,
    polozaj: usize,
) -> bool {
    let naredba = NaredbaStabla::Premjesti { stavka, u, polozaj };
    if uredjivac.izvrsi(naredba).is_err() {
        return false;
    }
    let mut mapa = u;
    while let Some(id) = mapa {
        stanje.postavi_otvorenost_mape(id, true);
        mapa = uredjivac.projekt().mapa(id).and_then(|m| m.roditelj);
    }
    stanje.odabrano = Some(odabir_stavke(stavka));
    true
}

/// Naziv mape ili sloja za natpis koji prati miš dok se redak povlači.
#[must_use]
pub fn naziv_stavke(projekt: &Projekt, stavka: Stavka) -> Option<&str> {
    match stavka {
        Stavka::Mapa(id) => projekt.mapa(id).map(|m| m.naziv.as_str()),
        Stavka::Sloj(id) => projekt.sloj(id).map(|s| s.naziv.as_str()),
    }
}

#[cfg(test)]
mod testovi {
    use super::*;
    use slojevi::{IdSloja, Novo, VrstaGeometrije};

    struct Stablo {
        u: Uredjivac,
        teren: IdMape,
        stare: IdMape,
        uvezeno: IdMape,
        parcele: IdSloja,
        trase: IdSloja,
        granice: IdSloja,
    }

    fn mapa(u: &mut Uredjivac, roditelj: Option<IdMape>, naziv: &str) -> IdMape {
        let Ok(Some(Novo::Mapa(id))) = u.izvrsi(NaredbaStabla::DodajMapu {
            u: roditelj,
            naziv: naziv.into(),
        }) else {
            panic!("mapa {naziv}")
        };
        id
    }

    fn sloj(u: &mut Uredjivac, roditelj: Option<IdMape>, naziv: &str) -> IdSloja {
        let Ok(Some(Novo::Sloj(id))) = u.izvrsi(NaredbaStabla::DodajSloj {
            u: roditelj,
            naziv: naziv.into(),
            vrsta: VrstaGeometrije::Poligon,
        }) else {
            panic!("sloj {naziv}")
        };
        id
    }

    /// Teren [Parcele, Trase, Stare [Granice]], Uvezeno.
    fn stablo() -> Stablo {
        let mut u = Uredjivac::novi();
        let teren = mapa(&mut u, None, "Teren");
        let parcele = sloj(&mut u, Some(teren), "Parcele");
        let trase = sloj(&mut u, Some(teren), "Trase");
        let stare = mapa(&mut u, Some(teren), "Stare");
        let granice = sloj(&mut u, Some(stare), "Granice");
        let uvezeno = mapa(&mut u, None, "Uvezeno");
        Stablo {
            u,
            teren,
            stare,
            uvezeno,
            parcele,
            trase,
            granice,
        }
    }

    fn djeca(s: &Stablo, mapa: IdMape) -> Vec<Stavka> {
        s.u.projekt().mapa(mapa).expect("mapa").djeca.clone()
    }

    #[test]
    fn odredista_su_redom_kao_stablo_bez_same_mape_i_njezinih_podmapa() {
        let s = stablo();
        let katalog = KatalogPoruka::hrvatski();
        let za_sloj = odredista_premjestanja(s.u.projekt(), Stavka::Sloj(s.granice), &katalog);
        let opis: Vec<_> = za_sloj
            .iter()
            .map(|o| (o.naziv.as_str(), o.uvlaka, o.trenutna))
            .collect();
        assert_eq!(
            opis,
            [
                ("Izvan mapa", 0, false),
                ("Teren", 0, false),
                ("Stare", 1, true),
                ("Uvezeno", 0, false),
            ]
        );

        let za_mapu = odredista_premjestanja(s.u.projekt(), Stavka::Mapa(s.teren), &katalog);
        let mape: Vec<_> = za_mapu.iter().map(|o| o.mapa).collect();
        assert_eq!(mape, [None, Some(s.uvezeno)]);
        assert!(za_mapu[0].trenutna);
    }

    #[test]
    fn ispustanje_na_mapu_ide_na_kraj_a_na_sloj_ispred_njega() {
        let s = stablo();
        let p = s.u.projekt();
        let trase = Stavka::Sloj(s.trase);
        assert_eq!(
            odrediste_ispustanja(p, trase, Odabir::Mapa(s.uvezeno)),
            Some((Some(s.uvezeno), usize::MAX))
        );
        assert_eq!(
            odrediste_ispustanja(p, trase, Odabir::Sloj(s.granice)),
            Some((Some(s.stare), 0))
        );
        // U istoj mapi Trase već stoji iza Parcela, pa „ispred Parcela” ostaje mjesto 0.
        assert_eq!(
            odrediste_ispustanja(p, trase, Odabir::Sloj(s.parcele)),
            Some((Some(s.teren), 0))
        );
        // Parcele ispred Trasa: nakon vađenja Parcela Trase su na mjestu 0.
        assert_eq!(
            odrediste_ispustanja(p, Stavka::Sloj(s.parcele), Odabir::Sloj(s.trase)),
            Some((Some(s.teren), 0))
        );
    }

    #[test]
    fn nedopusteno_ispustanje_ne_vodi_nikamo() {
        let s = stablo();
        let p = s.u.projekt();
        let teren = Stavka::Mapa(s.teren);
        assert_eq!(odrediste_ispustanja(p, teren, Odabir::Mapa(s.teren)), None);
        assert_eq!(odrediste_ispustanja(p, teren, Odabir::Mapa(s.stare)), None);
        assert_eq!(
            odrediste_ispustanja(p, teren, Odabir::Sloj(s.granice)),
            None
        );
        let objekt = Odabir::Objekt(s.parcele, slojevi::IdObjekta(1));
        assert_eq!(odrediste_ispustanja(p, teren, objekt), None);
        assert_eq!(
            odrediste_ispustanja(p, Stavka::Sloj(s.trase), Odabir::Sloj(s.trase)),
            None
        );
        assert_eq!(stavka_odabira(objekt), None);
    }

    #[test]
    fn premjestanje_otvara_mape_do_stavke_i_moze_se_ponistiti() {
        let mut s = stablo();
        let mut stanje = StanjePrikaza::novo();
        stanje.postavi_otvorenost_mape(s.teren, false);
        stanje.postavi_otvorenost_mape(s.stare, false);
        let uvezeno = Stavka::Mapa(s.uvezeno);

        assert!(premjesti_stavku(
            &mut s.u,
            &mut stanje,
            uvezeno,
            Some(s.stare),
            0
        ));
        assert_eq!(djeca(&s, s.stare), [uvezeno, Stavka::Sloj(s.granice)]);
        assert!(stanje.je_mapa_otvorena(s.teren));
        assert!(stanje.je_mapa_otvorena(s.stare));
        assert_eq!(stanje.odabrano, Some(Odabir::Mapa(s.uvezeno)));
        assert_eq!(naziv_stavke(s.u.projekt(), uvezeno), Some("Uvezeno"));

        // Mapa ne može u vlastitu podmapu.
        let teren = Stavka::Mapa(s.teren);
        assert!(!premjesti_stavku(
            &mut s.u,
            &mut stanje,
            teren,
            Some(s.uvezeno),
            0
        ));

        s.u.ponisti().expect("poništi");
        assert_eq!(djeca(&s, s.stare), [Stavka::Sloj(s.granice)]);
        assert_eq!(s.u.projekt().korijen(), [teren, uvezeno]);
    }
}
