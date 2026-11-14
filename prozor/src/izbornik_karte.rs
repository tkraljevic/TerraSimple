//! Izbornik na desni klik na objekt na karti (Odluke (14. 11. 2026.)). Za jedan objekt nudi isto
//! što i njegov redak u ploči Slojevi, uz zumiranje na objekt. Kad je odabrano više objekata,
//! nudi samo ono što vrijedi za sve: premještanje u sloj iste vrste i brisanje.

use crate::poruke::KatalogPoruka;
use crate::premjestanje::{Mjesto, Odrediste, odredista_premjestanja};
use crate::stanje::Odabir;
use eframe::egui::{self, Popup, Response, SetOpenCommand};
use slojevi::{IdObjekta, IdSloja, NaredbaObjekta, Projekt, Uredjivac};

/// Što je korisnik odabrao u izborniku.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadnjaKarte {
    Preimenuj,
    Stil,
    Zumiraj,
    PremjestiU(IdSloja),
    Obrisi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stavka {
    Preimenuj,
    Stil,
    Premjesti,
    Zumiraj,
    Obrisi,
}

const ZA_JEDAN: [Stavka; 5] = [
    Stavka::Preimenuj,
    Stavka::Stil,
    Stavka::Premjesti,
    Stavka::Zumiraj,
    Stavka::Obrisi,
];
const ZA_VISE: [Stavka; 2] = [Stavka::Premjesti, Stavka::Obrisi];

const fn stavke(broj_odabranih: usize) -> &'static [Stavka] {
    match broj_odabranih {
        0 => &[],
        1 => &ZA_JEDAN,
        _ => &ZA_VISE,
    }
}

fn naziv_stavke(s: Stavka, broj_odabranih: usize, katalog: &KatalogPoruka) -> String {
    match s {
        Stavka::Preimenuj => katalog.tekst("gumb-preimenuj"),
        Stavka::Stil => katalog.tekst("izbornik-stil"),
        Stavka::Premjesti => katalog.tekst("izbornik-premjesti-u"),
        Stavka::Zumiraj => katalog.tekst("izbornik-zumiraj"),
        Stavka::Obrisi if broj_odabranih > 1 => {
            katalog.s_brojem("izbornik-obrisi-odabrane", broj_odabranih)
        }
        Stavka::Obrisi => katalog.tekst("gumb-obrisi"),
    }
}

/// Slojevi u koje odabrani objekti smiju, redom kao u stablu, s mapama kao natpisima. Objekti
/// različitih vrsta ne mogu u isti sloj, pa za njih nema ničega. Sloj je ponuđen ako barem jedan
/// odabrani objekt nije već u njemu.
#[must_use]
pub fn odredista(
    projekt: &Projekt,
    odabrani: &[(IdSloja, IdObjekta)],
    katalog: &KatalogPoruka,
) -> Vec<Odrediste> {
    let vrsta = |s: &IdSloja| projekt.sloj(*s).map(|x| x.vrsta);
    let Some(&(prvi_sloj, prvi)) = odabrani.first() else {
        return Vec::new();
    };
    if odabrani.iter().any(|(s, _)| vrsta(s) != vrsta(&prvi_sloj)) {
        return Vec::new();
    }
    let mut popis = odredista_premjestanja(projekt, Odabir::Objekt(prvi_sloj, prvi), katalog);
    for o in &mut popis {
        if let Some(Mjesto::Sloj(cilj)) = o.mjesto {
            o.omoguceno = odabrani.iter().any(|(s, _)| *s != cilj);
        }
    }
    popis
}

/// Premješta odabrane objekte u sloj `cilj` u jednom koraku poništavanja i vraća novi odabir.
/// Objekti koji su već u tom sloju ostaju gdje jesu.
pub fn premjesti_objekte(
    u: &mut Uredjivac,
    odabrani: &[(IdSloja, IdObjekta)],
    cilj: IdSloja,
) -> Option<Vec<(IdSloja, IdObjekta)>> {
    let ishod = u.skupina(|g| {
        for &(iz, objekt) in odabrani.iter().filter(|(s, _)| *s != cilj) {
            g.izvrsi(NaredbaObjekta::Premjesti {
                iz,
                u: cilj,
                objekt,
            })?;
        }
        Ok(())
    });
    ishod
        .ok()
        .map(|()| odabrani.iter().map(|&(_, o)| (cilj, o)).collect())
}

/// Desni klik na objekt otvara izbornik uz miš; klik ili povlačenje karte, ili prazan odabir, ga
/// zatvaraju.
fn naredba_otvaranja(odgovor: &Response, otvori: bool, prazno: bool) -> Option<SetOpenCommand> {
    if otvori {
        Some(SetOpenCommand::Bool(true))
    } else if prazno || odgovor.clicked() || odgovor.secondary_clicked() || odgovor.drag_started() {
        Some(SetOpenCommand::Bool(false))
    } else {
        None
    }
}

/// Crta izbornik uz kartu i vraća odabranu radnju. `otvori` je istina u okviru desnog klika.
pub fn crtaj(
    odgovor: &Response,
    otvori: bool,
    projekt: &Projekt,
    odabrani: &[(IdSloja, IdObjekta)],
    katalog: &KatalogPoruka,
) -> Option<RadnjaKarte> {
    Popup::menu(odgovor)
        .open_memory(naredba_otvaranja(odgovor, otvori, odabrani.is_empty()))
        .at_pointer_fixed()
        .show(|ui| sadrzaj(ui, projekt, odabrani, katalog))
        .and_then(|r| r.inner)
}

fn sadrzaj(
    ui: &mut egui::Ui,
    projekt: &Projekt,
    odabrani: &[(IdSloja, IdObjekta)],
    katalog: &KatalogPoruka,
) -> Option<RadnjaKarte> {
    let mut radnja = None;
    for &s in stavke(odabrani.len()) {
        let naziv = naziv_stavke(s, odabrani.len(), katalog);
        let odabrana = match s {
            Stavka::Premjesti => podizbornik(ui, &naziv, &odredista(projekt, odabrani, katalog)),
            _ => ui
                .button(naziv)
                .clicked()
                .then(|| radnja_stavke(s))
                .flatten(),
        };
        if odabrana.is_some() {
            radnja = odabrana;
            ui.close();
        }
    }
    radnja
}

const fn radnja_stavke(s: Stavka) -> Option<RadnjaKarte> {
    match s {
        Stavka::Preimenuj => Some(RadnjaKarte::Preimenuj),
        Stavka::Stil => Some(RadnjaKarte::Stil),
        Stavka::Zumiraj => Some(RadnjaKarte::Zumiraj),
        Stavka::Obrisi => Some(RadnjaKarte::Obrisi),
        Stavka::Premjesti => None,
    }
}

/// „Premjesti u” sa slojevima; bez ijednog dopuštenog sloja stavka je siva.
fn podizbornik(ui: &mut egui::Ui, naziv: &str, odredista: &[Odrediste]) -> Option<RadnjaKarte> {
    if !odredista.iter().any(|o| o.omoguceno) {
        let _ = ui.add_enabled(false, egui::Button::new(naziv));
        return None;
    }
    ui.menu_button(naziv, |ui| {
        let mut odabrano = None;
        for o in odredista {
            let gumb = ui
                .horizontal(|ui| {
                    ui.add_space(o.uvlaka as f32 * crate::tema::Razmaci::UVLAKA_PODIZBORNIKA);
                    ui.add_enabled(o.omoguceno, egui::Button::new(&o.naziv))
                })
                .inner;
            if gumb.clicked()
                && let Some(Mjesto::Sloj(cilj)) = o.mjesto
            {
                odabrano = Some(RadnjaKarte::PremjestiU(cilj));
            }
        }
        odabrano
    })
    .inner
    .flatten()
}

#[cfg(test)]
#[path = "izbornik_karte_testovi.rs"]
mod testovi;
