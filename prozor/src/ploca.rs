use crate::ikone::{SkladisteIkona, okvir_ikone_u_sredini};
use crate::poruke::{KatalogPoruka, mjera_duljine, mjera_povrsine};
use crate::premjestanje::{Mjesto, odredista_premjestanja, odrediste_ispustanja, premjesti};
use crate::primjer::ukupno_slojeva_u_mapi;
use crate::sastavnice::{
    OpisGumbaIkone, OpisRetkaStabla, PodizbornikRetka, StavkaPodizbornika, cip, gumb_ikona,
    gumb_obrub, redak_stabla, tekst_lijevo, zaobljeni_okvir,
};
use crate::stanje::{NoviSloj, Odabir, StanjePrikaza, UredjivanjeStila};
use crate::tema::{Boje, Razmaci, font_obicni, font_srednji};
use eframe::egui::{
    self, CursorIcon, Id, LayerId, Order, Pos2, Rect, Role, Sense, Stroke, StrokeKind, WidgetInfo,
    pos2, vec2,
};
use slojevi::{
    IdMape, IdSloja, IzmjenaObjekta, Mjere, NaredbaObjekta, NaredbaStabla, Novo, Objekt, Projekt,
    Sloj, StanjeSloja, Stavka, Uredjivac, VrstaGeometrije,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedakZaPrikaz {
    pub odabir: Odabir,
    pub id_kljuc: String,
    pub uvlaka: usize,
    pub strelica: Option<bool>,
    pub rezerviraj_strelicu: bool,
    pub ikona: String,
    pub stanje_ikone: String,
    pub naslov: String,
    pub naglasen_naslov: bool,
    pub podnaslov: String,
    pub oznaka_formata: Option<String>,
    pub zakljucano: bool,
    pub ukljuceno: bool,
}

#[must_use]
pub fn izgradi_retke_stabla(
    projekt: &Projekt,
    stanje: &StanjePrikaza,
    katalog: &KatalogPoruka,
) -> Vec<RedakZaPrikaz> {
    let mut redci = Vec::new();
    for stavka in projekt.korijen() {
        dodaj_retke_stavke(projekt, stanje, katalog, *stavka, 0, &mut redci);
    }
    redci
}

fn dodaj_retke_stavke(
    projekt: &Projekt,
    stanje: &StanjePrikaza,
    katalog: &KatalogPoruka,
    stavka: Stavka,
    uvlaka: usize,
    redci: &mut Vec<RedakZaPrikaz>,
) {
    match stavka {
        Stavka::Mapa(id_mape) => {
            dodaj_retke_mape(projekt, stanje, katalog, id_mape, uvlaka, redci);
        }
        Stavka::Sloj(id_sloja) => {
            if let Some(sloj) = projekt.sloj(id_sloja) {
                dodaj_retke_sloja(stanje, katalog, id_sloja, sloj, uvlaka, redci);
            }
        }
    }
}

fn dodaj_retke_mape(
    projekt: &Projekt,
    stanje: &StanjePrikaza,
    katalog: &KatalogPoruka,
    id_mape: IdMape,
    uvlaka: usize,
    redci: &mut Vec<RedakZaPrikaz>,
) {
    let Some(mapa) = projekt.mapa(id_mape) else {
        return;
    };
    let otvorena = stanje.je_mapa_otvorena(id_mape);
    let podnaslov = if stanje.skrij_podnaslov_mape.contains(&id_mape) {
        String::new()
    } else {
        let ukupno = ukupno_slojeva_u_mapi(projekt, id_mape);
        katalog.broj_slojeva(ukupno)
    };
    let odabir = Odabir::Mapa(id_mape);
    redci.push(RedakZaPrikaz {
        odabir,
        id_kljuc: format!("mapa_{}", id_mape.0),
        uvlaka,
        strelica: Some(otvorena),
        rezerviraj_strelicu: false,
        ikona: "mapa".to_owned(),
        stanje_ikone: "obicno".to_owned(),
        naslov: aktivni_naslov(stanje, odabir, &mapa.naziv),
        naglasen_naslov: true,
        podnaslov,
        oznaka_formata: None,
        zakljucano: false,
        ukljuceno: stanje.je_mapa_vidljiva(id_mape),
    });
    if !otvorena {
        return;
    }
    for dijete in &mapa.djeca {
        dodaj_retke_stavke(projekt, stanje, katalog, *dijete, uvlaka + 1, redci);
    }
}

fn dodaj_retke_sloja(
    stanje: &StanjePrikaza,
    katalog: &KatalogPoruka,
    id: IdSloja,
    sloj: &Sloj,
    uvlaka: usize,
    redci: &mut Vec<RedakZaPrikaz>,
) {
    let otvoren = stanje.je_sloj_otvoren(id);
    let (ikona, stanje_ikone) = ikona_za_sloj(stanje, id, sloj);
    let podnaslov = match sloj.stanje {
        StanjeSloja::SamoCitanje => katalog.tekst("sloj-samo-citanje"),
        _ => katalog.broj_objekata(sloj.objekti.len()),
    };
    let odabir = Odabir::Sloj(id);
    redci.push(RedakZaPrikaz {
        odabir,
        id_kljuc: format!("sloj_{}", id.0),
        uvlaka,
        strelica: Some(otvoren),
        rezerviraj_strelicu: false,
        ikona,
        stanje_ikone,
        naslov: aktivni_naslov(stanje, odabir, &sloj.naziv),
        naglasen_naslov: true,
        podnaslov,
        oznaka_formata: stanje.oznake_slojeva.get(&id).cloned(),
        zakljucano: !sloj.stanje.smije_mijenjati(),
        ukljuceno: stanje.je_sloj_vidljiv(id),
    });
    if !otvoren {
        return;
    }
    let granica = stanje
        .ogranicenje_objekata
        .get(&id)
        .copied()
        .unwrap_or(sloj.objekti.len());
    for objekt in sloj.objekti.values().take(granica) {
        dodaj_redak_objekta(stanje, id, objekt, uvlaka + 1, redci);
    }
}

fn dodaj_redak_objekta(
    stanje: &StanjePrikaza,
    id_sloja: IdSloja,
    objekt: &Objekt,
    uvlaka: usize,
    redci: &mut Vec<RedakZaPrikaz>,
) {
    let (ikona, stanje_ikone) = ikona_za_vrstu(objekt.geometrija.vrsta());
    let odabir = Odabir::Objekt(id_sloja, objekt.id);
    redci.push(RedakZaPrikaz {
        odabir,
        id_kljuc: format!("objekt_{}_{}", id_sloja.0, objekt.id.0),
        uvlaka,
        strelica: None,
        rezerviraj_strelicu: true,
        ikona: ikona.to_owned(),
        stanje_ikone: stanje_ikone.to_owned(),
        naslov: aktivni_naslov(stanje, odabir, &objekt.naziv),
        naglasen_naslov: false,
        podnaslov: oblikuj_mjeru_objekta(objekt),
        oznaka_formata: None,
        zakljucano: false,
        ukljuceno: stanje.je_objekt_vidljiv(id_sloja, objekt.id),
    });
}

fn aktivni_naslov(stanje: &StanjePrikaza, odabir: Odabir, izvorni: &str) -> String {
    if let Some((cilj, tekst)) = &stanje.preimenovanje
        && *cilj == odabir
    {
        return tekst.clone();
    }
    izvorni.to_owned()
}

fn ikona_za_sloj(stanje: &StanjePrikaza, id: IdSloja, sloj: &Sloj) -> (String, String) {
    if let Some(par) = stanje.ikone_slojeva.get(&id) {
        return par.clone();
    }
    let (i, s) = ikona_za_vrstu(sloj.vrsta);
    (i.to_owned(), s.to_owned())
}

const fn ikona_za_vrstu(vrsta: VrstaGeometrije) -> (&'static str, &'static str) {
    match vrsta {
        VrstaGeometrije::Tocka => ("tocka", "crvena_tocka"),
        VrstaGeometrije::Linija => ("linija", "plava_linija"),
        VrstaGeometrije::Poligon => ("poligon", "narancasti_poligon"),
    }
}

fn oblikuj_mjeru_objekta(objekt: &Objekt) -> String {
    match objekt.geometrija.mjere() {
        Ok(Mjere::Tocka) | Err(_) => objekt.opis.clone(),
        Ok(Mjere::Linija(m)) => mjera_duljine(m.duljina_m),
        Ok(Mjere::Poligon(m)) => {
            if m.povrsina_m2 >= 10_000.0 {
                mjera_povrsine(m.povrsina_m2)
            } else {
                mjera_duljine(m.opseg_m)
            }
        }
    }
}

pub fn dodaj_novu_mapu(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
) -> Option<IdMape> {
    let u = odredi_roditelja_za_novu_mapu(uredjivac.projekt(), stanje.odabrano);
    let naziv = katalog.s_rednim("nova-mapa-naziv", stanje.redni_nove_mape);
    let Some(Novo::Mapa(novi_id)) = uredjivac
        .izvrsi(NaredbaStabla::DodajMapu { u, naziv })
        .ok()?
    else {
        return None;
    };
    stanje.redni_nove_mape += 1;
    stanje.odabrano = Some(Odabir::Mapa(novi_id));
    Some(novi_id)
}

/// Otvara prozorčić Novi sloj. Vrsta se predlaže prema alatu, a mapa prema odabiru u ploči.
pub fn otvori_novi_sloj(projekt: &Projekt, stanje: &mut StanjePrikaza, katalog: &KatalogPoruka) {
    stanje.novi_sloj = Some(NoviSloj {
        naziv: katalog.s_rednim("novi-sloj-naziv", stanje.redni_novog_sloja),
        vrsta: stanje.alat.vrsta().unwrap_or(VrstaGeometrije::Poligon),
        mapa: odredi_roditelja_za_novi_sloj(projekt, stanje.odabrano),
        prazan_naziv: false,
    });
}

/// Gumb „Napravi sloj”: sloj nastaje s nazivom, vrstom i mapom iz prozorčića i postaje odabran.
pub fn napravi_novi_sloj(uredjivac: &mut Uredjivac, stanje: &mut StanjePrikaza) -> Option<IdSloja> {
    let novi = stanje.novi_sloj.as_mut()?;
    let naziv = novi.naziv.trim().to_owned();
    if naziv.is_empty() {
        novi.prazan_naziv = true;
        return None;
    }
    let naredba = NaredbaStabla::DodajSloj {
        u: novi.mapa,
        naziv,
        vrsta: novi.vrsta,
    };
    let Ok(Some(Novo::Sloj(novi_id))) = uredjivac.izvrsi(naredba) else {
        return None;
    };
    stanje.novi_sloj = None;
    stanje.redni_novog_sloja += 1;
    stanje.odabrano = Some(Odabir::Sloj(novi_id));
    Some(novi_id)
}

/// Novi sloj s predloženim vrijednostima, kao da je korisnik odmah kliknuo „Napravi sloj”.
pub fn dodaj_novi_sloj(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
) -> Option<IdSloja> {
    otvori_novi_sloj(uredjivac.projekt(), stanje, katalog);
    napravi_novi_sloj(uredjivac, stanje)
}

pub fn potvrdi_preimenovanje(uredjivac: &mut Uredjivac, stanje: &mut StanjePrikaza) -> bool {
    let Some((cilj, sirovi_tekst)) = stanje.preimenovanje.take() else {
        return false;
    };
    let naziv = sirovi_tekst.trim().to_owned();
    if naziv.is_empty() {
        return false;
    }
    match cilj {
        Odabir::Mapa(id) => uredjivac
            .izvrsi(NaredbaStabla::Preimenuj {
                stavka: Stavka::Mapa(id),
                naziv,
            })
            .is_ok(),
        Odabir::Sloj(id) => uredjivac
            .izvrsi(NaredbaStabla::Preimenuj {
                stavka: Stavka::Sloj(id),
                naziv,
            })
            .is_ok(),
        Odabir::Objekt(sloj, objekt) => uredjivac
            .izvrsi(NaredbaObjekta::Izmijeni {
                sloj,
                objekt,
                izmjena: IzmjenaObjekta::Naziv(naziv),
            })
            .is_ok(),
    }
}

pub fn obrisi_odabrano(uredjivac: &mut Uredjivac, stanje: &mut StanjePrikaza) -> bool {
    let Some(cilj) = stanje.odabrano else {
        return false;
    };
    let uspjeh = match cilj {
        Odabir::Mapa(id) => uredjivac
            .izvrsi(NaredbaStabla::Obrisi {
                stavka: Stavka::Mapa(id),
            })
            .is_ok(),
        Odabir::Sloj(id) => uredjivac
            .izvrsi(NaredbaStabla::Obrisi {
                stavka: Stavka::Sloj(id),
            })
            .is_ok(),
        Odabir::Objekt(sloj, objekt) => uredjivac
            .izvrsi(NaredbaObjekta::Obrisi { sloj, objekt })
            .is_ok(),
    };
    if uspjeh {
        stanje.odabrano = None;
        stanje.preimenovanje = None;
    }
    uspjeh
}

pub fn pokreni_preimenovanje_odabranog(projekt: &Projekt, stanje: &mut StanjePrikaza) -> bool {
    let Some(cilj) = stanje.odabrano else {
        return false;
    };
    let Some(naziv) = naziv_za_odabir(projekt, cilj) else {
        return false;
    };
    stanje.zapocni_preimenovanje(cilj, &naziv);
    true
}

#[must_use]
pub fn naziv_za_odabir(projekt: &Projekt, odabir: Odabir) -> Option<String> {
    match odabir {
        Odabir::Mapa(id) => Some(projekt.mapa(id)?.naziv.clone()),
        Odabir::Sloj(id) => Some(projekt.sloj(id)?.naziv.clone()),
        Odabir::Objekt(sloj, objekt) => Some(projekt.objekt(sloj, objekt)?.naziv.clone()),
    }
}

fn odredi_roditelja_za_novu_mapu(projekt: &Projekt, odabrano: Option<Odabir>) -> Option<IdMape> {
    match odabrano {
        Some(Odabir::Mapa(id)) if projekt.mapa(id).is_some() => Some(id),
        _ => None,
    }
}

fn odredi_roditelja_za_novi_sloj(projekt: &Projekt, odabrano: Option<Odabir>) -> Option<IdMape> {
    match odabrano {
        Some(Odabir::Mapa(id)) if projekt.mapa(id).is_some() => Some(id),
        Some(Odabir::Sloj(id) | Odabir::Objekt(id, _)) => {
            projekt.sloj(id).and_then(|sloj| sloj.roditelj)
        }
        _ => projekt.korijen().iter().find_map(|stavka| match *stavka {
            Stavka::Mapa(id) => Some(id),
            Stavka::Sloj(_) => None,
        }),
    }
}

pub fn crtaj_plocu_slojeva(
    ui: &mut egui::Ui,
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    ikone: &mut SkladisteIkona,
    katalog: &KatalogPoruka,
    okvir_karte: Rect,
    boje: &Boje,
) {
    if !stanje.ploca_otvorena {
        return;
    }
    let pw = Razmaci::SIRINA_PLOCE;
    let ph = okvir_karte.height() - 24.0;
    let px = if stanje.ploca_lijevo {
        okvir_karte.min.x + Razmaci::RUB
    } else {
        okvir_karte.max.x - pw - Razmaci::RUB
    };
    let py = okvir_karte.min.y + Razmaci::RUB;
    let okvir_ploce = Rect::from_min_size(pos2(px, py), vec2(pw, ph));

    zaobljeni_okvir(
        ui,
        okvir_ploce,
        boje.ploca,
        Some(boje.obrub),
        Razmaci::ZAOBLJENJE_PLOCE,
    );
    if stanje.stil.is_some() {
        crate::stil::crtaj_plocu_stila(ui, uredjivac, stanje, ikone, katalog, okvir_ploce, boje);
        return;
    }
    crtaj_zaglavlje_ploce(
        ui,
        stanje,
        ikone,
        katalog,
        okvir_ploce,
        okvir_karte.center().x,
        boje,
    );
    crtaj_akcijske_gumbe_ploce(
        ui,
        uredjivac,
        stanje,
        ikone,
        katalog,
        pos2(px, py + 52.0),
        boje,
    );
    let visina_podloge = if stanje.prikazi_primjer { 146.0 } else { 114.0 };
    crtaj_stablo_u_ploci(
        ui,
        uredjivac,
        stanje,
        ikone,
        katalog,
        Rect::from_min_size(
            pos2(px + 8.0, py + 94.0),
            vec2(pw - 16.0, ph - visina_podloge - 96.0),
        ),
        boje,
    );
    crtaj_odjeljak_podloge(
        ui,
        stanje,
        ikone,
        katalog,
        pos2(px, py + ph - visina_podloge),
        pw,
        boje,
    );
}

fn crtaj_zaglavlje_ploce(
    ui: &mut egui::Ui,
    stanje: &mut StanjePrikaza,
    ikone: &mut SkladisteIkona,
    katalog: &KatalogPoruka,
    okvir_ploce: Rect,
    sredina_prozora_x: f32,
    boje: &Boje,
) {
    let px = okvir_ploce.min.x;
    let py = okvir_ploce.min.y;
    let pw = okvir_ploce.width();
    let tamna = stanje.tema_tamna;

    ikone.crtaj(
        ui,
        okvir_ikone_u_sredini(pos2(px + 19.0, py + 23.0), 18),
        "hvataljka",
        tamna,
        "priguseno",
        18,
    );
    tekst_lijevo(
        ui,
        pos2(px + 34.0, py + 22.0),
        &katalog.tekst("ploca-slojevi"),
        font_srednji(20.0),
        boje.tekst,
    );

    let okvir_povlacenja = Rect::from_min_size(pos2(px, py), vec2(pw - 44.0, 44.0));
    let id_povlacenja = ui.make_persistent_id("povlacenje_ploce_slojeva");
    let odg_povlacenja = ui.interact(okvir_povlacenja, id_povlacenja, Sense::click_and_drag());
    let labela_premjesti = katalog.tekst("ploca-premjesti");
    odg_povlacenja
        .widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &labela_premjesti));
    if odg_povlacenja.clicked() {
        stanje.ploca_lijevo = !stanje.ploca_lijevo;
    } else if (odg_povlacenja.dragged() || odg_povlacenja.drag_stopped())
        && let Some(pozicija) = ui.input(|i| i.pointer.interact_pos())
    {
        stanje.ploca_lijevo = pozicija.x < sredina_prozora_x;
    }

    let opis_zatvori = OpisGumbaIkone {
        okvir: Rect::from_min_size(pos2(px + pw - 36.0, py + 11.0), vec2(24.0, 24.0)),
        ikona: "x",
        stanje: "obicno",
        crtez: 20,
        obrubljen: false,
        aktivno: false,
        labela: &katalog.tekst("ploca-zatvori"),
    };
    if gumb_ikona(ui, ikone, &opis_zatvori, tamna, boje).clicked() {
        stanje.ploca_otvorena = false;
    }
    ui.painter().line_segment(
        [pos2(px, py + 44.0), pos2(px + pw, py + 44.0)],
        egui::Stroke::new(1.0, boje.obrub),
    );
}

fn crtaj_akcijske_gumbe_ploce(
    ui: &mut egui::Ui,
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    ikone: &mut SkladisteIkona,
    katalog: &KatalogPoruka,
    pocetak: Pos2,
    boje: &Boje,
) {
    let tamna = stanje.tema_tamna;
    let okvir_novi = Rect::from_min_size(pos2(pocetak.x + 12.0, pocetak.y), vec2(112.0, 32.0));
    if gumb_obrub(
        ui,
        ikone,
        okvir_novi,
        "plus",
        &katalog.tekst("gumb-novi-sloj"),
        tamna,
        boje,
    )
    .clicked()
    {
        otvori_novi_sloj(uredjivac.projekt(), stanje, katalog);
    }

    let okvir_uvezi = Rect::from_min_size(pos2(pocetak.x + 130.0, pocetak.y), vec2(112.0, 32.0));
    if gumb_obrub(
        ui,
        ikone,
        okvir_uvezi,
        "uvezi",
        &katalog.tekst("gumb-uvezi-sloj"),
        tamna,
        boje,
    )
    .clicked()
    {
        stanje.zatrazen_uvoz = true;
    }

    let okvir_mapa = Rect::from_min_size(pos2(pocetak.x + 248.0, pocetak.y), vec2(100.0, 32.0));
    if gumb_obrub(
        ui,
        ikone,
        okvir_mapa,
        "nova_mapa",
        &katalog.tekst("gumb-nova-mapa"),
        tamna,
        boje,
    )
    .clicked()
    {
        dodaj_novu_mapu(uredjivac, stanje, katalog);
    }
}

fn crtaj_stablo_u_ploci(
    ui: &mut egui::Ui,
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    ikone: &mut SkladisteIkona,
    katalog: &KatalogPoruka,
    podrucje: Rect,
    boje: &Boje,
) {
    let redci = izgradi_retke_stabla(uredjivac.projekt(), stanje, katalog);
    let labela_strelice = katalog.tekst("oznaka-prosiri");
    let labela_vidljivosti = katalog.tekst("oznaka-vidljivost");
    let preimenuj = katalog.tekst("gumb-preimenuj");
    let obrisi = katalog.tekst("gumb-obrisi");
    let stil = katalog.tekst("izbornik-stil");
    let stavke_mape = [preimenuj.as_str(), obrisi.as_str()];
    let stavke_sloja = [preimenuj.as_str(), stil.as_str(), obrisi.as_str()];
    let premjesti_u = katalog.tekst("izbornik-premjesti-u");
    let tamna = stanje.tema_tamna;
    let visina_retka = crate::tema::Razmaci::VISINA_RETKA_STABLA;
    let maks_redaka = ((podrucje.height() / visina_retka).floor() as usize).max(1);

    let vidljivi = &redci[..redci.len().min(maks_redaka)];
    for (indeks, redak) in vidljivi.iter().enumerate() {
        let y = podrucje.min.y + indeks as f32 * visina_retka;
        let projekt = uredjivac.projekt();
        let odredista = || stavke_podizbornika(projekt, redak.odabir, katalog);
        let podizbornik = Some(PodizbornikRetka {
            naslov: &premjesti_u,
            iza_stavke: STAVKA_PREIMENUJ,
            stavke: &odredista,
        });
        let okvir = Rect::from_min_size(
            pos2(podrucje.min.x, y),
            vec2(podrucje.width(), visina_retka - 2.0),
        );
        let opis = OpisRetkaStabla {
            id_kljuc: &redak.id_kljuc,
            okvir,
            uvlaka: redak.uvlaka,
            strelica: redak.strelica,
            rezerviraj_strelicu: redak.rezerviraj_strelicu,
            ikona: &redak.ikona,
            stanje_ikone: &redak.stanje_ikone,
            naslov: &redak.naslov,
            naglasen_naslov: redak.naglasen_naslov,
            podnaslov: &redak.podnaslov,
            oznaka_formata: redak.oznaka_formata.as_deref(),
            zakljucano: redak.zakljucano,
            ukljuceno: Some(redak.ukljuceno),
            odabrano: je_odabran_redak(stanje, redak.odabir),
            labela_strelice: &labela_strelice,
            labela_vidljivosti: &labela_vidljivosti,
            uredjivanje: uredjivanje_retka(stanje, redak.odabir),
            stavke_izbornika: if matches!(redak.odabir, Odabir::Mapa(_)) {
                &stavke_mape
            } else {
                &stavke_sloja
            },
            podizbornik,
            povlacenje: stanje.preimenovanje.is_none(),
        };
        let odg = redak_stabla(ui, ikone, &opis, tamna, boje);
        obradi_odgovor_retka(uredjivac.projekt(), stanje, redak.odabir, odg);
        obradi_premjestanje_retka(uredjivac, stanje, katalog, redak.odabir, odg);
    }
    obradi_ispustanje(ui, uredjivac, stanje, vidljivi, podrucje, boje);
}

/// Redak je odabran u ploči ili je njegov objekt jedan od više odabranih na karti.
fn je_odabran_redak(stanje: &StanjePrikaza, odabir: Odabir) -> bool {
    stanje.odabrano == Some(odabir)
        || matches!(odabir, Odabir::Objekt(s, o) if stanje.odabrani_na_karti.contains(&(s, o)))
}

fn stavke_podizbornika(
    projekt: &Projekt,
    odabir: Odabir,
    katalog: &KatalogPoruka,
) -> Vec<StavkaPodizbornika> {
    odredista_premjestanja(projekt, odabir, katalog)
        .into_iter()
        .map(|o| StavkaPodizbornika {
            naziv: o.naziv,
            uvlaka: o.uvlaka,
            omogucena: o.omoguceno,
        })
        .collect()
}

/// Početak povlačenja retka i odabir mjesta u podizborniku „Premjesti u”.
fn obradi_premjestanje_retka(
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
    odabir: Odabir,
    odgovor: crate::sastavnice::OdgovorRetkaStabla,
) {
    if odgovor.povlacenje_pocelo {
        stanje.povuceno = Some(odabir);
    }
    let Some(redni) = odgovor.stavka_podizbornika else {
        return;
    };
    let mjesto = odredista_premjestanja(uredjivac.projekt(), odabir, katalog)
        .into_iter()
        .nth(redni)
        .filter(|o| o.omoguceno)
        .and_then(|o| o.mjesto);
    if let Some(mjesto) = mjesto {
        let _ = premjesti(uredjivac, stanje, odabir, mjesto);
    }
}

/// Dok se redak povlači, označava kamo bi otišao i ispisuje njegov naziv uz miš. Kad korisnik
/// pusti tipku, stavka se premješta, a izvan stabla ili na nedopušteno mjesto ostaje gdje je bila.
fn obradi_ispustanje(
    ui: &egui::Ui,
    uredjivac: &mut Uredjivac,
    stanje: &mut StanjePrikaza,
    redci: &[RedakZaPrikaz],
    podrucje: Rect,
    boje: &Boje,
) {
    let Some(povuceno) = stanje.povuceno else {
        return;
    };
    let (pokazivac, pusteno) = ui.input(|i| (i.pointer.interact_pos(), !i.pointer.primary_down()));
    let cilj = pokazivac.and_then(|p| redak_pod_pokazivacem(redci, podrucje, p));
    let odrediste = cilj.and_then(|(indeks, odabir)| {
        odrediste_ispustanja(uredjivac.projekt(), povuceno, odabir).map(|m| (indeks, m))
    });
    if pusteno {
        stanje.povuceno = None;
        if let Some((_, mjesto)) = odrediste {
            let _ = premjesti(uredjivac, stanje, povuceno, mjesto);
        }
        return;
    }
    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
    if let Some((indeks, mjesto)) = odrediste {
        crtaj_cilj_ispustanja(ui, podrucje, indeks, mjesto, boje);
    }
    if let (Some(p), Some(naziv)) = (pokazivac, naziv_za_odabir(uredjivac.projekt(), povuceno)) {
        crtaj_natpis_povlacenja(ui, p, &naziv, boje);
    }
}

fn redak_pod_pokazivacem(
    redci: &[RedakZaPrikaz],
    podrucje: Rect,
    pokazivac: Pos2,
) -> Option<(usize, Odabir)> {
    if !podrucje.contains(pokazivac) {
        return None;
    }
    let indeks = ((pokazivac.y - podrucje.min.y) / Razmaci::VISINA_RETKA_STABLA).floor() as usize;
    redci.get(indeks).map(|r| (indeks, r.odabir))
}

/// Mapa ili sloj u koji nešto ide dobije okvir, a kad mapa ili sloj ide ispred sloja, iznad njega se
/// crta crta.
fn crtaj_cilj_ispustanja(
    ui: &egui::Ui,
    podrucje: Rect,
    indeks: usize,
    mjesto: Mjesto,
    boje: &Boje,
) {
    let y = podrucje.min.y + indeks as f32 * Razmaci::VISINA_RETKA_STABLA;
    let okvir = Rect::from_min_size(
        pos2(podrucje.min.x, y),
        vec2(podrucje.width(), Razmaci::VISINA_RETKA_STABLA - 2.0),
    );
    let potez = Stroke::new(2.0, boje.primarna);
    let ispred = matches!(mjesto, Mjesto::Mapa { polozaj, .. } if polozaj != usize::MAX);
    if !ispred {
        ui.painter()
            .rect_stroke(okvir, Razmaci::ZAOBLJENJE_RETKA, potez, StrokeKind::Inside);
    } else {
        ui.painter().line_segment(
            [
                pos2(okvir.min.x + 4.0, y - 1.0),
                pos2(okvir.max.x - 4.0, y - 1.0),
            ],
            potez,
        );
    }
}

fn crtaj_natpis_povlacenja(ui: &egui::Ui, pokazivac: Pos2, naziv: &str, boje: &Boje) {
    let slikar = ui
        .ctx()
        .layer_painter(LayerId::new(Order::Tooltip, Id::unique("povlacenje_retka")));
    let tocka = pokazivac + vec2(14.0, 6.0);
    let natpis = slikar.layout_no_wrap(naziv.to_owned(), font_obicni(13.0), boje.tekst);
    let okvir = Rect::from_min_size(tocka, natpis.size()).expand(4.0);
    slikar.rect_filled(okvir, 4.0, boje.ploca);
    slikar.rect_stroke(okvir, 4.0, Stroke::new(1.0, boje.obrub), StrokeKind::Inside);
    slikar.galley(tocka, natpis, boje.tekst);
}

fn obradi_odgovor_retka(
    projekt: &Projekt,
    stanje: &mut StanjePrikaza,
    odabir: Odabir,
    odgovor: crate::sastavnice::OdgovorRetkaStabla,
) {
    if odgovor.kliknuta_strelica {
        obradi_klik_strelice(stanje, odabir);
    }
    if odgovor.kliknut_prekidac {
        obradi_klik_prekidaca(stanje, odabir);
    }
    obradi_klik_retka(projekt, stanje, odabir, odgovor);
    obradi_izbornik_retka(projekt, stanje, odabir, odgovor);
}

/// Redak čiji se naziv upravo mijenja crta se kao polje.
fn uredjivanje_retka(stanje: &StanjePrikaza, odabir: Odabir) -> Option<bool> {
    match &stanje.preimenovanje {
        Some((cilj, _)) if *cilj == odabir => Some(stanje.naziv_oznacen),
        _ => None,
    }
}

const STAVKA_PREIMENUJ: usize = 0;
/// Mapa nema stil, pa joj je „Obriši” odmah iza „Preimenuj”.
const STAVKA_STIL: usize = 1;

/// Desni klik odabire redak; izbornik ga zatim preimenuje ili traži brisanje.
fn obradi_izbornik_retka(
    projekt: &Projekt,
    stanje: &mut StanjePrikaza,
    odabir: Odabir,
    odgovor: crate::sastavnice::OdgovorRetkaStabla,
) {
    if odgovor.desni_klik {
        stanje.odabrano = Some(odabir);
    }
    let stil = !matches!(odabir, Odabir::Mapa(_));
    match odgovor.stavka_izbornika {
        Some(STAVKA_PREIMENUJ) => {
            stanje.odabrano = Some(odabir);
            let _ = pokreni_preimenovanje_odabranog(projekt, stanje);
        }
        Some(STAVKA_STIL) if stil => {
            stanje.odabrano = Some(odabir);
            stanje.preimenovanje = None;
            stanje.stil = UredjivanjeStila::za(odabir);
        }
        Some(_) => {
            stanje.odabrano = Some(odabir);
            stanje.preimenovanje = None;
            stanje.zatrazeno_brisanje = true;
        }
        None => {}
    }
}

fn obradi_klik_strelice(stanje: &mut StanjePrikaza, odabir: Odabir) {
    match odabir {
        Odabir::Mapa(id) => stanje.prebaci_otvorenost_mape(id),
        Odabir::Sloj(id) => stanje.prebaci_otvorenost_sloja(id),
        Odabir::Objekt(_, _) => {}
    }
}

fn obradi_klik_prekidaca(stanje: &mut StanjePrikaza, odabir: Odabir) {
    match odabir {
        Odabir::Mapa(id) => stanje.prebaci_vidljivost_mape(id),
        Odabir::Sloj(id) => stanje.prebaci_vidljivost_sloja(id),
        Odabir::Objekt(sloj, objekt) => stanje.prebaci_vidljivost_objekta(sloj, objekt),
    }
}

fn obradi_klik_retka(
    projekt: &Projekt,
    stanje: &mut StanjePrikaza,
    odabir: Odabir,
    odgovor: crate::sastavnice::OdgovorRetkaStabla,
) {
    if odgovor.dvokliknut_redak {
        stanje.odabrano = Some(odabir);
        let _ = pokreni_preimenovanje_odabranog(projekt, stanje);
    } else if odgovor.kliknut_redak {
        stanje.odabrano = Some(odabir);
    }
}

fn crtaj_odjeljak_podloge(
    ui: &mut egui::Ui,
    stanje: &mut StanjePrikaza,
    ikone: &mut SkladisteIkona,
    katalog: &KatalogPoruka,
    pocetak: Pos2,
    pw: f32,
    boje: &Boje,
) {
    let px = pocetak.x;
    let pod_y = pocetak.y;
    let tamna = stanje.tema_tamna;

    ui.painter().line_segment(
        [pos2(px, pod_y), pos2(px + pw, pod_y)],
        egui::Stroke::new(1.0, boje.obrub),
    );
    tekst_lijevo(
        ui,
        pos2(px + 12.0, pod_y + 18.0),
        &katalog.tekst("podloga-naslov"),
        font_srednji(12.0),
        boje.tekst_2,
    );

    let cipovi: &[(usize, &str, f32, f32, f32)] = if stanje.prikazi_primjer {
        &[
            (0usize, "podloga-karta", 12.0, 34.0, 74.0),
            (1usize, "podloga-dgu", 92.0, 34.0, 114.0),
            (2usize, "podloga-lokalna", 12.0, 66.0, 136.0),
        ]
    } else {
        &[
            (0usize, "podloga-karta", 12.0, 34.0, 74.0),
            (1usize, "podloga-dgu", 92.0, 34.0, 114.0),
        ]
    };
    for &(indeks, kljuc, pomak_x, pomak_y, sirina) in cipovi {
        let okvir = Rect::from_min_size(pos2(px + pomak_x, pod_y + pomak_y), vec2(sirina, 26.0));
        let naziv = katalog.tekst(kljuc);
        if cip(
            ui,
            ikone,
            okvir,
            &naziv,
            stanje.podloga == indeks,
            tamna,
            boje,
        )
        .clicked()
        {
            stanje.podloga = indeks;
        }
    }

    let gumbi_y = if stanje.prikazi_primjer {
        pod_y + 104.0
    } else {
        pod_y + 72.0
    };

    let ucitaj = katalog.tekst("gumb-ucitaj-datoteku");
    ikone.crtaj(
        ui,
        okvir_ikone_u_sredini(pos2(px + 21.0, gumbi_y), 18),
        "uvezi",
        tamna,
        "obicno",
        18,
    );
    tekst_lijevo(
        ui,
        pos2(px + 34.0, gumbi_y),
        &ucitaj,
        font_srednji(12.0),
        boje.primarna,
    );

    let wms = katalog.tekst("gumb-dodaj-wms");
    ikone.crtaj(
        ui,
        okvir_ikone_u_sredini(pos2(px + 157.0, gumbi_y), 18),
        "mreza",
        tamna,
        "obicno",
        18,
    );
    tekst_lijevo(
        ui,
        pos2(px + 170.0, gumbi_y),
        &wms,
        font_srednji(12.0),
        boje.primarna,
    );
}

#[cfg(test)]
mod testovi {
    use super::*;
    use crate::primjer::izgradi_primjer_zaslona_a;

    #[test]
    fn izmjene_stabla_idu_kroz_uredjivac_i_mogu_se_ponistiti_i_ponoviti() {
        let mut uredjivac = Uredjivac::novi();
        let mut stanje = StanjePrikaza::novo();
        let katalog = KatalogPoruka::hrvatski();

        let id_mape = dodaj_novu_mapu(&mut uredjivac, &mut stanje, &katalog).unwrap();
        assert_eq!(uredjivac.projekt().mapa(id_mape).unwrap().naziv, "Mapa 1");

        let id_sloja = dodaj_novi_sloj(&mut uredjivac, &mut stanje, &katalog).unwrap();
        assert_eq!(uredjivac.projekt().sloj(id_sloja).unwrap().naziv, "Sloj 1");

        assert!(pokreni_preimenovanje_odabranog(
            uredjivac.projekt(),
            &mut stanje
        ));
        stanje.preimenovanje = Some((Odabir::Sloj(id_sloja), "Katastar".into()));
        assert!(potvrdi_preimenovanje(&mut uredjivac, &mut stanje));
        assert_eq!(
            uredjivac.projekt().sloj(id_sloja).unwrap().naziv,
            "Katastar"
        );

        assert!(uredjivac.ponisti().unwrap());
        assert_eq!(uredjivac.projekt().sloj(id_sloja).unwrap().naziv, "Sloj 1");
        assert!(uredjivac.ponovi().unwrap());
        assert_eq!(
            uredjivac.projekt().sloj(id_sloja).unwrap().naziv,
            "Katastar"
        );

        assert!(obrisi_odabrano(&mut uredjivac, &mut stanje));
        assert!(uredjivac.projekt().sloj(id_sloja).is_none());
        assert!(uredjivac.ponisti().unwrap());
        assert!(uredjivac.projekt().sloj(id_sloja).is_some());
    }

    #[test]
    fn stablo_primjera_zaslona_a_daje_jedanaest_redaka() {
        let mut uredjivac = Uredjivac::novi();
        let mut stanje = StanjePrikaza::novo();
        let katalog = KatalogPoruka::hrvatski();
        izgradi_primjer_zaslona_a(&mut uredjivac, &mut stanje, &katalog);

        let redci = izgradi_retke_stabla(uredjivac.projekt(), &stanje, &katalog);
        assert_eq!(redci.len(), 11);
        assert_eq!(redci[0].naslov, "Teren 2026");
        assert_eq!(redci[0].podnaslov, "3 sloja");
        assert_eq!(redci[1].naslov, "Parcele");
        assert_eq!(redci[1].podnaslov, "12 objekata");
        assert_eq!(redci[2].podnaslov, "2,31 ha");
        assert_eq!(redci[3].podnaslov, "4,83 ha");
        assert_eq!(redci[4].podnaslov, "1,87 ha");
        assert_eq!(redci[0].ikona, "mapa");
        assert_eq!(redci[1].ikona, "poligon");
        assert_eq!(redci[1].stanje_ikone, "narancasti_poligon");
        assert_eq!(redci[5].ikona, "linija");
        assert_eq!(redci[5].stanje_ikone, "plava_linija");
        assert_eq!(redci[6].ikona, "tocka");
        assert_eq!(redci[6].stanje_ikone, "crvena_tocka");
        assert_eq!(redci[9].oznaka_formata.as_deref(), Some("KML"));
        assert_eq!(redci[10].oznaka_formata.as_deref(), Some("GPKG"));
        assert!(redci[10].zakljucano);

        let mapa_id = match redci[0].odabir {
            Odabir::Mapa(id) => id,
            _ => panic!("očekivana mapa"),
        };
        let sloj_id = match redci[1].odabir {
            Odabir::Sloj(id) => id,
            _ => panic!("očekivan sloj"),
        };
        let objekt_id = match redci[2].odabir {
            Odabir::Objekt(_, id) => id,
            _ => panic!("očekivan objekt"),
        };

        assert_eq!(
            naziv_za_odabir(uredjivac.projekt(), Odabir::Mapa(mapa_id)).as_deref(),
            Some("Teren 2026")
        );
        assert_eq!(
            naziv_za_odabir(uredjivac.projekt(), Odabir::Sloj(sloj_id)).as_deref(),
            Some("Parcele")
        );
        assert_eq!(
            naziv_za_odabir(uredjivac.projekt(), Odabir::Objekt(sloj_id, objekt_id)).as_deref(),
            Some("Parcela 1118/2")
        );
        assert_eq!(
            naziv_za_odabir(uredjivac.projekt(), Odabir::Mapa(IdMape(9999))),
            None
        );

        let odg_dvoklik = crate::sastavnice::OdgovorRetkaStabla {
            dvokliknut_redak: true,
            ..Default::default()
        };
        obradi_odgovor_retka(
            uredjivac.projekt(),
            &mut stanje,
            Odabir::Objekt(sloj_id, objekt_id),
            odg_dvoklik,
        );
        assert_eq!(
            stanje.preimenovanje,
            Some((Odabir::Objekt(sloj_id, objekt_id), "Parcela 1118/2".into()))
        );

        let odg_prekidac = crate::sastavnice::OdgovorRetkaStabla {
            kliknut_prekidac: true,
            ..Default::default()
        };
        obradi_odgovor_retka(
            uredjivac.projekt(),
            &mut stanje,
            Odabir::Mapa(mapa_id),
            odg_prekidac,
        );
        assert!(!stanje.je_mapa_vidljiva(mapa_id));
        obradi_odgovor_retka(
            uredjivac.projekt(),
            &mut stanje,
            Odabir::Objekt(sloj_id, objekt_id),
            odg_prekidac,
        );
        assert!(!stanje.je_objekt_vidljiv(sloj_id, objekt_id));
    }

    #[test]
    fn dodavanje_slojeva_i_mapa_inkrementira_redne_brojeve_i_postuje_roditelja() {
        let mut uredjivac = Uredjivac::novi();
        let mut stanje = StanjePrikaza::novo();
        let katalog = KatalogPoruka::hrvatski();

        let mapa1 = dodaj_novu_mapu(&mut uredjivac, &mut stanje, &katalog).unwrap();
        assert_eq!(stanje.redni_nove_mape, 2);
        assert_eq!(uredjivac.projekt().mapa(mapa1).unwrap().naziv, "Mapa 1");

        let mapa2 = dodaj_novu_mapu(&mut uredjivac, &mut stanje, &katalog).unwrap();
        assert_eq!(stanje.redni_nove_mape, 3);
        assert_eq!(uredjivac.projekt().mapa(mapa2).unwrap().naziv, "Mapa 2");

        stanje.odabrano = Some(Odabir::Mapa(mapa1));
        let sloj1 = dodaj_novi_sloj(&mut uredjivac, &mut stanje, &katalog).unwrap();
        assert_eq!(stanje.redni_novog_sloja, 2);
        assert_eq!(uredjivac.projekt().sloj(sloj1).unwrap().naziv, "Sloj 1");
        assert!(
            uredjivac
                .projekt()
                .mapa(mapa1)
                .unwrap()
                .djeca
                .contains(&Stavka::Sloj(sloj1))
        );

        let sloj2 = dodaj_novi_sloj(&mut uredjivac, &mut stanje, &katalog).unwrap();
        assert_eq!(stanje.redni_novog_sloja, 3);
        assert_eq!(uredjivac.projekt().sloj(sloj2).unwrap().naziv, "Sloj 2");
        assert!(
            uredjivac
                .projekt()
                .mapa(mapa1)
                .unwrap()
                .djeca
                .contains(&Stavka::Sloj(sloj2))
        );

        stanje.odabrano = Some(Odabir::Sloj(sloj1));
        let sloj3 = dodaj_novi_sloj(&mut uredjivac, &mut stanje, &katalog).unwrap();
        assert!(
            uredjivac
                .projekt()
                .mapa(mapa1)
                .unwrap()
                .djeca
                .contains(&Stavka::Sloj(sloj3))
        );

        pokreni_preimenovanje_odabranog(uredjivac.projekt(), &mut stanje);
        assert!(stanje.preimenovanje.is_some());
        if let Some((_, tekst)) = &mut stanje.preimenovanje {
            *tekst = "Novi naziv".into();
        }
        assert!(potvrdi_preimenovanje(&mut uredjivac, &mut stanje));
        assert_eq!(uredjivac.projekt().sloj(sloj3).unwrap().naziv, "Novi naziv");
        assert!(stanje.preimenovanje.is_none());
    }

    use slojevi::{Geometrija, Vrh};

    fn mapa_u(u: &mut Uredjivac, roditelj: Option<IdMape>, naziv: &str) -> IdMape {
        let Ok(Some(Novo::Mapa(id))) = u.izvrsi(NaredbaStabla::DodajMapu {
            u: roditelj,
            naziv: naziv.into(),
        }) else {
            panic!("mapa {naziv}")
        };
        id
    }

    fn sloj_u(
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

    fn objekt_u(
        u: &mut Uredjivac,
        sloj: IdSloja,
        naziv: &str,
        geometrija: Geometrija,
    ) -> slojevi::IdObjekta {
        let Ok(Some(Novo::Objekt(id))) = u.izvrsi(NaredbaObjekta::Dodaj {
            sloj,
            naziv: naziv.into(),
            geometrija,
        }) else {
            panic!("objekt {naziv}")
        };
        id
    }

    fn obrisi_mapu(u: &mut Uredjivac, mapa: IdMape) {
        u.izvrsi(NaredbaStabla::Obrisi {
            stavka: Stavka::Mapa(mapa),
        })
        .expect("brisanje mape");
    }

    fn je_u_mapi(u: &Uredjivac, mapa: IdMape, stavka: Stavka) -> bool {
        u.projekt()
            .mapa(mapa)
            .is_some_and(|m| m.djeca.contains(&stavka))
    }

    #[test]
    fn nova_mapa_ide_u_odabranu_mapu_a_bez_valjanog_odabira_na_vrh() {
        let mut u = Uredjivac::novi();
        let mut stanje = StanjePrikaza::novo();
        let katalog = KatalogPoruka::hrvatski();
        let prva = mapa_u(&mut u, None, "Prva");

        stanje.odabrano = Some(Odabir::Mapa(prva));
        let unutra = dodaj_novu_mapu(&mut u, &mut stanje, &katalog).expect("mapa u mapi");
        assert!(je_u_mapi(&u, prva, Stavka::Mapa(unutra)));

        let obrisana = mapa_u(&mut u, None, "Obrisana");
        obrisi_mapu(&mut u, obrisana);
        stanje.odabrano = Some(Odabir::Mapa(obrisana));
        let na_vrhu = dodaj_novu_mapu(&mut u, &mut stanje, &katalog).expect("mapa na vrhu");
        assert!(u.projekt().korijen().contains(&Stavka::Mapa(na_vrhu)));
    }

    #[test]
    fn novi_sloj_ide_u_mapu_odabrane_mape_sloja_ili_objekta() {
        let mut u = Uredjivac::novi();
        let mut stanje = StanjePrikaza::novo();
        let katalog = KatalogPoruka::hrvatski();
        let prva = mapa_u(&mut u, None, "Prva");
        let druga = mapa_u(&mut u, None, "Druga");

        stanje.odabrano = Some(Odabir::Mapa(druga));
        let iz_mape = dodaj_novi_sloj(&mut u, &mut stanje, &katalog).expect("sloj");
        assert!(
            je_u_mapi(&u, druga, Stavka::Sloj(iz_mape)),
            "odabrana druga mapa"
        );

        stanje.odabrano = Some(Odabir::Sloj(iz_mape));
        let iz_sloja = dodaj_novi_sloj(&mut u, &mut stanje, &katalog).expect("sloj");
        assert!(
            je_u_mapi(&u, druga, Stavka::Sloj(iz_sloja)),
            "mapa odabranog sloja"
        );

        let tocke = sloj_u(&mut u, Some(druga), "Točke", VrstaGeometrije::Tocka);
        let zdenac = objekt_u(
            &mut u,
            tocke,
            "Zdenac",
            Geometrija::Tocka(Vrh::novi(16.0, 45.8)),
        );
        stanje.odabrano = Some(Odabir::Objekt(tocke, zdenac));
        let iz_objekta = dodaj_novi_sloj(&mut u, &mut stanje, &katalog).expect("sloj");
        assert!(
            je_u_mapi(&u, druga, Stavka::Sloj(iz_objekta)),
            "mapa odabranog objekta"
        );

        let obrisana = mapa_u(&mut u, None, "Obrisana");
        obrisi_mapu(&mut u, obrisana);
        stanje.odabrano = Some(Odabir::Mapa(obrisana));
        let bez_odabira = dodaj_novi_sloj(&mut u, &mut stanje, &katalog).expect("sloj");
        assert!(
            je_u_mapi(&u, prva, Stavka::Sloj(bez_odabira)),
            "bez valjanog odabira prva mapa"
        );
    }

    #[test]
    fn redci_stabla_imaju_uvlaku_ikonu_naslov_i_stanje_iz_projekta() {
        let mut u = Uredjivac::novi();
        let mut stanje = StanjePrikaza::novo();
        let katalog = KatalogPoruka::hrvatski();
        let vanjska = mapa_u(&mut u, None, "Vanjska");
        let unutarnja = mapa_u(&mut u, Some(vanjska), "Unutarnja");
        let tocke = sloj_u(&mut u, Some(unutarnja), "Točke", VrstaGeometrije::Tocka);
        objekt_u(
            &mut u,
            tocke,
            "Zdenac",
            Geometrija::Tocka(Vrh::novi(16.0, 45.8)),
        );
        let crte = sloj_u(&mut u, Some(unutarnja), "Crte", VrstaGeometrije::Linija);
        let crta = vec![Vrh::novi(16.0, 45.8), Vrh::novi(16.01, 45.8)];
        objekt_u(&mut u, crte, "Trasa", Geometrija::Linija(crta));
        sloj_u(&mut u, Some(unutarnja), "Likovi", VrstaGeometrije::Poligon);
        let tudi = sloj_u(&mut u, None, "Tuđi", VrstaGeometrije::Tocka);
        u.postavi_stanje_sloja(tudi, StanjeSloja::SamoCitanje)
            .expect("stanje sloja");
        stanje.postavi_otvorenost_sloja(tocke, true);
        stanje.postavi_otvorenost_sloja(crte, true);

        let redci = izgradi_retke_stabla(u.projekt(), &stanje, &katalog);
        let uvlake: Vec<(&str, usize)> = redci
            .iter()
            .map(|r| (r.naslov.as_str(), r.uvlaka))
            .collect();
        assert_eq!(
            uvlake,
            vec![
                ("Vanjska", 0),
                ("Unutarnja", 1),
                ("Točke", 2),
                ("Zdenac", 3),
                ("Crte", 2),
                ("Trasa", 3),
                ("Likovi", 2),
                ("Tuđi", 0),
            ]
        );

        let ikona = |naslov: &str| {
            redci
                .iter()
                .find(|r| r.naslov == naslov)
                .map(|r| (r.ikona.as_str(), r.stanje_ikone.as_str()))
        };
        assert_eq!(ikona("Točke"), Some(("tocka", "crvena_tocka")));
        assert_eq!(ikona("Zdenac"), Some(("tocka", "crvena_tocka")));
        assert_eq!(ikona("Crte"), Some(("linija", "plava_linija")));
        assert_eq!(ikona("Trasa"), Some(("linija", "plava_linija")));
        assert_eq!(ikona("Likovi"), Some(("poligon", "narancasti_poligon")));

        let tudi_redak = redci
            .iter()
            .find(|r| r.naslov == "Tuđi")
            .expect("tuđi sloj");
        assert_eq!(tudi_redak.podnaslov, "samo za čitanje");
        assert!(tudi_redak.zakljucano);
        let tocke_redak = redci.iter().find(|r| r.naslov == "Točke").expect("točke");
        assert_eq!(tocke_redak.podnaslov, katalog.broj_objekata(1));

        stanje.preimenovanje = Some((Odabir::Sloj(crte), "Nove crte".into()));
        let naslovi: Vec<String> = izgradi_retke_stabla(u.projekt(), &stanje, &katalog)
            .into_iter()
            .map(|r| r.naslov)
            .collect();
        assert!(naslovi.contains(&"Nove crte".to_owned()));
        assert!(
            naslovi.contains(&"Točke".to_owned()),
            "ostali redci zadržavaju naziv"
        );
        assert!(!naslovi.contains(&"Crte".to_owned()));
    }
}
