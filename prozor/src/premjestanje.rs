//! Premještanje u ploči Slojevi, izbornikom „Premjesti u” ili povlačenjem: mape i slojevi idu u
//! drugu mapu, a objekti u drugi sloj iste vrste.

use crate::poruke::KatalogPoruka;
use crate::stanje::{Odabir, StanjePrikaza};
use slojevi::{IdMape, IdSloja, NaredbaObjekta, NaredbaStabla, Projekt, Stavka, Uredjivac};

/// Kamo stavka ide. `u: None` znači izvan svih mapa, a položaj iza kraja znači na kraj.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mjesto {
    Mapa { u: Option<IdMape>, polozaj: usize },
    Sloj(IdSloja),
}

/// Redak podizbornika „Premjesti u”. Bez mjesta je samo natpis, npr. mapa iznad slojeva.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Odrediste {
    pub mjesto: Option<Mjesto>,
    pub naziv: String,
    pub uvlaka: usize,
    pub omoguceno: bool,
}

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

/// Može li objekt iz sloja `iz` u sloj `u`: drugi sloj, ista vrsta, oba se smiju mijenjati.
fn smije_u_sloj(projekt: &Projekt, iz: IdSloja, u: IdSloja) -> bool {
    match (projekt.sloj(iz), projekt.sloj(u)) {
        (Some(a), Some(b)) => {
            iz != u
                && a.vrsta == b.vrsta
                && a.stanje.smije_mijenjati()
                && b.stanje.smije_mijenjati()
        }
        _ => false,
    }
}

/// Sve stavke stabla redom kao u ploči, s dubinom.
fn obilazak(projekt: &Projekt) -> Vec<(Stavka, usize)> {
    let mut redom = Vec::new();
    let mut za_obilazak: Vec<(Stavka, usize)> =
        projekt.korijen().iter().rev().map(|s| (*s, 0)).collect();
    while let Some((s, dubina)) = za_obilazak.pop() {
        redom.push((s, dubina));
        if let Stavka::Mapa(id) = s
            && let Some(mapa) = projekt.mapa(id)
        {
            za_obilazak.extend(mapa.djeca.iter().rev().map(|d| (*d, dubina + 1)));
        }
    }
    redom
}

/// Retci podizbornika „Premjesti u”, redom kao u stablu.
#[must_use]
pub fn odredista_premjestanja(
    projekt: &Projekt,
    odabir: Odabir,
    katalog: &KatalogPoruka,
) -> Vec<Odrediste> {
    match (odabir, stavka_odabira(odabir)) {
        (_, Some(stavka)) => odredista_stavke(projekt, stavka, katalog),
        (Odabir::Objekt(sloj, _), None) => odredista_objekta(projekt, sloj),
        _ => Vec::new(),
    }
}

/// Za mapu ili sloj: „Izvan mapa” i sve mape. Mapa ne nudi samu sebe ni svoje podmape.
fn odredista_stavke(projekt: &Projekt, stavka: Stavka, katalog: &KatalogPoruka) -> Vec<Odrediste> {
    let roditelj = roditelj_stavke(projekt, stavka).flatten();
    let u_mapu = |u: Option<IdMape>, naziv: String, uvlaka: usize| Odrediste {
        mjesto: Some(Mjesto::Mapa {
            u,
            polozaj: usize::MAX,
        }),
        naziv,
        uvlaka,
        omoguceno: roditelj != u,
    };
    let mut odredista = vec![u_mapu(None, katalog.tekst("premjesti-izvan-mapa"), 0)];
    for (s, dubina) in obilazak(projekt) {
        let Stavka::Mapa(id) = s else {
            continue;
        };
        let Some(mapa) = projekt.mapa(id) else {
            continue;
        };
        let vlastita = matches!(stavka, Stavka::Mapa(m) if je_unutar(projekt, m, Some(id)));
        if !vlastita {
            odredista.push(u_mapu(Some(id), mapa.naziv.clone(), dubina));
        }
    }
    odredista
}

/// Za objekt: slojevi iste vrste, s mapama iznad njih kao natpisima.
fn odredista_objekta(projekt: &Projekt, iz: IdSloja) -> Vec<Odrediste> {
    let mut odredista = Vec::new();
    for (s, dubina) in obilazak(projekt) {
        let (mjesto, naziv, omoguceno) = match s {
            Stavka::Mapa(id) => match projekt.mapa(id) {
                Some(m) => (None, m.naziv.clone(), false),
                None => continue,
            },
            Stavka::Sloj(id) => match projekt.sloj(id) {
                Some(sloj) if id == iz || smije_u_sloj(projekt, iz, id) => {
                    (Some(Mjesto::Sloj(id)), sloj.naziv.clone(), id != iz)
                }
                _ => continue,
            },
        };
        odredista.push(Odrediste {
            mjesto,
            naziv,
            uvlaka: dubina,
            omoguceno,
        });
    }
    odredista
}

/// Kamo ide ono što je ispušteno na redak `cilj`. Mapa ili sloj ide u mapu na kraj, a na sloj
/// ispred njega; objekt ide u sloj na koji je ispušten ili u sloj objekta na koji je ispušten.
#[must_use]
pub fn odrediste_ispustanja(projekt: &Projekt, povuceno: Odabir, cilj: Odabir) -> Option<Mjesto> {
    match (povuceno, stavka_odabira(povuceno)) {
        (_, Some(stavka)) => mjesto_stavke(projekt, stavka, cilj),
        (Odabir::Objekt(iz, _), None) => {
            let u = match cilj {
                Odabir::Sloj(u) | Odabir::Objekt(u, _) => u,
                Odabir::Mapa(_) => return None,
            };
            smije_u_sloj(projekt, iz, u).then_some(Mjesto::Sloj(u))
        }
        _ => None,
    }
}

/// Na objekt, na samu sebe ili mapu u vlastitu podmapu stavka ne ide nikamo.
fn mjesto_stavke(projekt: &Projekt, povuceno: Stavka, cilj: Odabir) -> Option<Mjesto> {
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
    Some(Mjesto::Mapa { u, polozaj })
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

/// Premješta, ostavlja premješteno odabranim i otvara mape i sloj do njega, da se vidi kamo je
/// otišlo.
pub fn premjesti(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    povuceno: Odabir,
    mjesto: Mjesto,
) -> bool {
    let (izvrseno, novi_odabir, mapa) = match (povuceno, mjesto) {
        (Odabir::Objekt(iz, objekt), Mjesto::Sloj(u)) => {
            let naredba = NaredbaObjekta::Premjesti { iz, u, objekt };
            let roditelj = uredjivac.projekt().sloj(u).and_then(|s| s.roditelj);
            (
                uredjivac.izvrsi(naredba).is_ok(),
                Odabir::Objekt(u, objekt),
                roditelj,
            )
        }
        (_, Mjesto::Mapa { u, polozaj }) => {
            let Some(stavka) = stavka_odabira(povuceno) else {
                return false;
            };
            let naredba = NaredbaStabla::Premjesti { stavka, u, polozaj };
            (uredjivac.izvrsi(naredba).is_ok(), odabir_stavke(stavka), u)
        }
        (_, Mjesto::Sloj(_)) => return false,
    };
    if !izvrseno {
        return false;
    }
    if let Odabir::Objekt(sloj, _) = novi_odabir {
        stanje.postavi_otvorenost_sloja(sloj, true);
    }
    let mut mapa = mapa;
    while let Some(id) = mapa {
        stanje.postavi_otvorenost_mape(id, true);
        mapa = uredjivac.projekt().mapa(id).and_then(|m| m.roditelj);
    }
    stanje.odabrano = Some(novi_odabir);
    true
}

#[cfg(test)]
mod testovi {
    use super::*;
    use slojevi::{Geometrija, IdObjekta, Novo, StanjeSloja, VrstaGeometrije};

    struct Stablo {
        u: Uredjivac,
        teren: IdMape,
        stare: IdMape,
        uvezeno: IdMape,
        parcele: IdSloja,
        trase: IdSloja,
        granice: IdSloja,
        nasipi: IdSloja,
        parcela: IdObjekta,
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

    fn sloj(
        u: &mut Uredjivac,
        roditelj: Option<IdMape>,
        naziv: &str,
        vrsta: VrstaGeometrije,
    ) -> IdSloja {
        let Ok(Some(Novo::Sloj(id))) = u.izvrsi(NaredbaStabla::DodajSloj {
            u: roditelj,
            naziv: naziv.into(),
            vrsta,
        }) else {
            panic!("sloj {naziv}")
        };
        id
    }

    /// Teren [Parcele, Trase, Stare [Granice]], Uvezeno [Nasipi]. Parcele, Granice i Nasipi su
    /// poligoni, Trase linije.
    fn stablo() -> Stablo {
        let mut u = Uredjivac::novi();
        let poligon = VrstaGeometrije::Poligon;
        let teren = mapa(&mut u, None, "Teren");
        let parcele = sloj(&mut u, Some(teren), "Parcele", poligon);
        let trase = sloj(&mut u, Some(teren), "Trase", VrstaGeometrije::Linija);
        let stare = mapa(&mut u, Some(teren), "Stare");
        let granice = sloj(&mut u, Some(stare), "Granice", poligon);
        let uvezeno = mapa(&mut u, None, "Uvezeno");
        let nasipi = sloj(&mut u, Some(uvezeno), "Nasipi", poligon);
        let vrh = |duzina, sirina| slojevi::Vrh {
            duzina,
            sirina,
            visina: None,
        };
        let lik = vec![vec![
            vrh(0.0, 0.0),
            vrh(1.0, 0.0),
            vrh(1.0, 1.0),
            vrh(0.0, 0.0),
        ]];
        let Ok(Some(Novo::Objekt(parcela))) = u.izvrsi(NaredbaObjekta::Dodaj {
            sloj: parcele,
            naziv: "Parcela 1".into(),
            geometrija: Geometrija::Poligon(lik),
        }) else {
            panic!("parcela")
        };
        Stablo {
            u,
            teren,
            stare,
            uvezeno,
            parcele,
            trase,
            granice,
            nasipi,
            parcela,
        }
    }

    fn djeca(s: &Stablo, mapa: IdMape) -> Vec<Stavka> {
        s.u.projekt().mapa(mapa).expect("mapa").djeca.clone()
    }

    fn opis(odredista: &[Odrediste]) -> Vec<(&str, usize, bool)> {
        odredista
            .iter()
            .map(|o| (o.naziv.as_str(), o.uvlaka, o.omoguceno))
            .collect()
    }

    #[test]
    fn mapa_ili_sloj_nudi_sve_mape_osim_sebe_i_svojih_podmapa() {
        let s = stablo();
        let katalog = KatalogPoruka::hrvatski();
        let p = s.u.projekt();
        let za_sloj = odredista_premjestanja(p, Odabir::Sloj(s.granice), &katalog);
        assert_eq!(
            opis(&za_sloj),
            [
                ("Izvan mapa", 0, true),
                ("Teren", 0, true),
                ("Stare", 1, false),
                ("Uvezeno", 0, true),
            ]
        );

        let za_mapu = odredista_premjestanja(p, Odabir::Mapa(s.teren), &katalog);
        assert_eq!(
            opis(&za_mapu),
            [("Izvan mapa", 0, false), ("Uvezeno", 0, true)]
        );
        assert_eq!(
            za_mapu[1].mjesto,
            Some(Mjesto::Mapa {
                u: Some(s.uvezeno),
                polozaj: usize::MAX
            })
        );
    }

    #[test]
    fn objekt_nudi_slojeve_iste_vrste_a_mape_su_samo_natpisi() {
        let s = stablo();
        let katalog = KatalogPoruka::hrvatski();
        let objekt = Odabir::Objekt(s.parcele, s.parcela);
        let odredista = odredista_premjestanja(s.u.projekt(), objekt, &katalog);
        assert_eq!(
            opis(&odredista),
            [
                ("Teren", 0, false),
                ("Parcele", 1, false),
                ("Stare", 1, false),
                ("Granice", 2, true),
                ("Uvezeno", 0, false),
                ("Nasipi", 1, true),
            ]
        );
        assert_eq!(odredista[0].mjesto, None);
        assert_eq!(odredista[5].mjesto, Some(Mjesto::Sloj(s.nasipi)));
    }

    #[test]
    fn ispustanje_mape_ili_sloja_ide_u_mapu_na_kraj_a_na_sloj_ispred_njega() {
        let s = stablo();
        let p = s.u.projekt();
        let trase = Odabir::Sloj(s.trase);
        let u = |u, polozaj| Some(Mjesto::Mapa { u, polozaj });
        assert_eq!(
            odrediste_ispustanja(p, trase, Odabir::Mapa(s.uvezeno)),
            u(Some(s.uvezeno), usize::MAX)
        );
        assert_eq!(
            odrediste_ispustanja(p, trase, Odabir::Sloj(s.granice)),
            u(Some(s.stare), 0)
        );
        // Trase već stoji iza Parcela, pa „ispred Parcela” ostaje mjesto 0.
        assert_eq!(
            odrediste_ispustanja(p, trase, Odabir::Sloj(s.parcele)),
            u(Some(s.teren), 0)
        );
        // Parcele ispred Trasa: nakon vađenja Parcela Trase su na mjestu 0.
        assert_eq!(
            odrediste_ispustanja(p, Odabir::Sloj(s.parcele), Odabir::Sloj(s.trase)),
            u(Some(s.teren), 0)
        );
    }

    #[test]
    fn ispustanje_objekta_ide_u_sloj_iste_vrste() {
        let s = stablo();
        let p = s.u.projekt();
        let objekt = Odabir::Objekt(s.parcele, s.parcela);
        assert_eq!(
            odrediste_ispustanja(p, objekt, Odabir::Sloj(s.nasipi)),
            Some(Mjesto::Sloj(s.nasipi))
        );
        let u_nasipima = Odabir::Objekt(s.nasipi, IdObjekta(99));
        assert_eq!(
            odrediste_ispustanja(p, objekt, u_nasipima),
            Some(Mjesto::Sloj(s.nasipi))
        );
        assert_eq!(odrediste_ispustanja(p, objekt, Odabir::Sloj(s.trase)), None);
        assert_eq!(
            odrediste_ispustanja(p, objekt, Odabir::Sloj(s.parcele)),
            None
        );
        assert_eq!(
            odrediste_ispustanja(p, objekt, Odabir::Mapa(s.uvezeno)),
            None
        );
    }

    #[test]
    fn nedopusteno_ispustanje_ne_vodi_nikamo() {
        let s = stablo();
        let p = s.u.projekt();
        let teren = Odabir::Mapa(s.teren);
        assert_eq!(odrediste_ispustanja(p, teren, Odabir::Mapa(s.teren)), None);
        assert_eq!(odrediste_ispustanja(p, teren, Odabir::Mapa(s.stare)), None);
        assert_eq!(
            odrediste_ispustanja(p, teren, Odabir::Sloj(s.granice)),
            None
        );
        let objekt = Odabir::Objekt(s.parcele, s.parcela);
        assert_eq!(odrediste_ispustanja(p, teren, objekt), None);
        let trase = Odabir::Sloj(s.trase);
        assert_eq!(odrediste_ispustanja(p, trase, trase), None);
    }

    #[test]
    fn zakljucan_sloj_ne_prima_objekte() {
        let mut s = stablo();
        s.u.postavi_stanje_sloja(s.nasipi, StanjeSloja::SamoCitanje)
            .expect("zaključavanje");
        let objekt = Odabir::Objekt(s.parcele, s.parcela);
        let p = s.u.projekt();
        assert_eq!(
            odrediste_ispustanja(p, objekt, Odabir::Sloj(s.nasipi)),
            None
        );
    }

    #[test]
    fn premjestanje_otvara_put_do_premjestenog_i_moze_se_ponistiti() {
        let mut s = stablo();
        let mut stanje = StanjePrikaza::novo();
        stanje.postavi_otvorenost_mape(s.teren, false);
        stanje.postavi_otvorenost_mape(s.stare, false);
        let uvezeno = Stavka::Mapa(s.uvezeno);
        let u_stare = Mjesto::Mapa {
            u: Some(s.stare),
            polozaj: 0,
        };

        assert!(premjesti(
            &mut s.u,
            &mut stanje,
            Odabir::Mapa(s.uvezeno),
            u_stare
        ));
        assert_eq!(djeca(&s, s.stare), [uvezeno, Stavka::Sloj(s.granice)]);
        assert!(stanje.je_mapa_otvorena(s.teren));
        assert!(stanje.je_mapa_otvorena(s.stare));
        assert_eq!(stanje.odabrano, Some(Odabir::Mapa(s.uvezeno)));

        // Mapa ne može u vlastitu podmapu, a mapa nije sloj.
        let teren = Odabir::Mapa(s.teren);
        let u_uvezeno = Mjesto::Mapa {
            u: Some(s.uvezeno),
            polozaj: 0,
        };
        assert!(!premjesti(&mut s.u, &mut stanje, teren, u_uvezeno));
        assert!(!premjesti(
            &mut s.u,
            &mut stanje,
            teren,
            Mjesto::Sloj(s.nasipi)
        ));
        let objekt = Odabir::Objekt(s.parcele, s.parcela);
        assert!(!premjesti(&mut s.u, &mut stanje, objekt, u_stare));

        s.u.ponisti().expect("poništi");
        assert_eq!(djeca(&s, s.stare), [Stavka::Sloj(s.granice)]);
        assert_eq!(s.u.projekt().korijen(), [Stavka::Mapa(s.teren), uvezeno]);
    }

    #[test]
    fn objekt_prelazi_u_drugi_sloj_koji_se_otvori() {
        let mut s = stablo();
        let mut stanje = StanjePrikaza::novo();
        let objekt = Odabir::Objekt(s.parcele, s.parcela);
        assert!(premjesti(
            &mut s.u,
            &mut stanje,
            objekt,
            Mjesto::Sloj(s.nasipi)
        ));
        assert!(s.u.projekt().objekt(s.nasipi, s.parcela).is_some());
        assert!(s.u.projekt().objekt(s.parcele, s.parcela).is_none());
        assert!(stanje.je_sloj_otvoren(s.nasipi));
        assert!(stanje.je_mapa_otvorena(s.uvezeno));
        assert_eq!(stanje.odabrano, Some(Odabir::Objekt(s.nasipi, s.parcela)));
    }
}
