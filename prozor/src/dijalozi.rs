//! Prikaz padajućeg izbornika Datoteka i modalnih dijaloga (potvrde, greške, upozorenja).

use crate::poruke::KatalogPoruka;
use crate::sastavnice::{tekst_lijevo, tekst_sredina, zaobljeni_okvir};
use crate::tema::{Boje, Razmaci, font_obicni, font_srednji};
use eframe::egui::{self, Color32, Key, Rect, Response, Role, Sense, WidgetInfo, pos2, vec2};
use slojevi::Stavka;

/// Stavka odabrana u padajućem izborniku Datoteka.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StavkaIzbornikaDatoteka {
    Novi,
    Otvori,
    Spremi,
    SpremiKao,
}

/// Svrha potvrde nespremljenih izmjena.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NamjeraPotvrde {
    Novi,
    Otvori,
    ZatvoriProzor,
}

/// Stanje trenutno otvorenog modalnog dijaloga.
#[derive(Clone, Debug, PartialEq)]
pub enum AktivniDijalog {
    PotvrdaNespremljeno(NamjeraPotvrde),
    UpozorenjeBrisanja {
        stavka: Stavka,
        naziv: String,
        broj_nepodrzanih: usize,
    },
    PorukaGreske {
        naslov: String,
        poruka: String,
    },
    UpozorenjaOtvaranja {
        upozorenja: Vec<String>,
        otvoren_popis: bool,
    },
}

/// Radnja koju je korisnik odabrao u dijalogu.
#[derive(Clone, Debug, PartialEq)]
pub enum IshodDijaloga {
    PotvrdaSpremi(NamjeraPotvrde),
    PotvrdaNeSpremaj(NamjeraPotvrde),
    PotvrdaOdustani,
    ObrisiPotvrdjeno(Stavka),
    ObrisiOdustani,
    UpozorenjaPreklopiPopis,
    ZatvoriPoruku,
}

/// Crta padajući izbornik "Datoteka" ispod gornje trake.
pub fn crtaj_padajuci_izbornik_datoteka(
    ui: &mut egui::Ui,
    otvoren: &mut bool,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<StavkaIzbornikaDatoteka> {
    if !*otvoren {
        return None;
    }

    if ui.input(|i| i.key_pressed(Key::Escape)) {
        *otvoren = false;
        return None;
    }

    let pocetak = pos2(40.0, Razmaci::VISINA_IZBORNIKA + 2.0);
    let stavke = [
        (
            StavkaIzbornikaDatoteka::Novi,
            katalog.tekst("izbornik-datoteka-novi"),
            katalog.tekst("precac-novi"),
        ),
        (
            StavkaIzbornikaDatoteka::Otvori,
            katalog.tekst("izbornik-datoteka-otvori"),
            katalog.tekst("precac-otvori"),
        ),
        (
            StavkaIzbornikaDatoteka::Spremi,
            katalog.tekst("izbornik-datoteka-spremi"),
            katalog.tekst("precac-spremi"),
        ),
        (
            StavkaIzbornikaDatoteka::SpremiKao,
            katalog.tekst("izbornik-datoteka-spremi-kao"),
            katalog.tekst("precac-spremi-kao"),
        ),
    ];

    let sirina = 190.0;
    let visina_retka = 28.0;
    let ukupna_visina = stavke.len() as f32 * visina_retka + 8.0;
    let okvir = Rect::from_min_size(pocetak, vec2(sirina, ukupna_visina));

    // Pozadina i obrub izbornika
    // Dok je izbornik otvoren, klik izvan njega samo ga zatvara i ne pada na kartu ili ploču.
    let mut ispod = ui.ctx().content_rect();
    ispod.min.y = Razmaci::VISINA_IZBORNIKA;
    crate::karta_prozora::zaklon(ui, ispod, "izbornik_datoteka");
    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 6.0);

    let mut odabrano = None;
    for (indeks, (vrsta, naziv, precac)) in stavke.into_iter().enumerate() {
        let ry = pocetak.y + 4.0 + indeks as f32 * visina_retka;
        let r_okvir =
            Rect::from_min_size(pos2(pocetak.x + 4.0, ry), vec2(sirina - 8.0, visina_retka));

        if crtaj_stavku_padajuceg_izbornika(ui, r_okvir, indeks, &naziv, &precac, boje) {
            odabrano = Some(vrsta);
            *otvoren = false;
        }
    }

    if ui.input(|i| i.pointer.any_click()) {
        let mis = ui.input(|i| i.pointer.hover_pos()).unwrap_or_default();
        if !okvir.contains(mis) && mis.y > Razmaci::VISINA_IZBORNIKA && odabrano.is_none() {
            *otvoren = false;
        }
    }

    odabrano
}

fn crtaj_stavku_padajuceg_izbornika(
    ui: &mut egui::Ui,
    r_okvir: Rect,
    indeks: usize,
    naziv: &str,
    precac: &str,
    boje: &Boje,
) -> bool {
    let id = ui.make_persistent_id(("padajuci_datoteka", indeks));
    let odg = ui.interact(r_okvir, id, Sense::click());
    odg.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), naziv));

    if odg.hovered() {
        zaobljeni_okvir(ui, r_okvir, boje.ploca_2, None, 4.0);
    }

    tekst_lijevo(
        ui,
        pos2(r_okvir.min.x + 8.0, r_okvir.center().y),
        naziv,
        font_obicni(12.0),
        boje.tekst,
    );
    crate::sastavnice::tekst_desno(
        ui,
        pos2(r_okvir.max.x - 8.0, r_okvir.center().y),
        precac,
        font_obicni(11.0),
        boje.tekst_3,
    );

    odg.clicked()
}

/// Crta aktivni modalni dijalog u središtu zaslona i vraća ishod korisničkog odabira.
pub fn crtaj_aktivni_dijalog(
    ui: &mut egui::Ui,
    dijalog: &mut AktivniDijalog,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let dostupan = ui.ctx().content_rect();
    // Zatamnjena pozadina zaklanja kartu i ploču, da klik pored dijaloga ne nacrta točku ili ne
    // promijeni odabir dok dijalog čeka odgovor.
    crate::karta_prozora::zaklon(ui, dostupan, "aktivni_dijalog");
    ui.painter()
        .rect_filled(dostupan, 0.0, Color32::from_black_alpha(110));

    match dijalog {
        AktivniDijalog::PotvrdaNespremljeno(namjera) => {
            crtaj_potvrdu_nespremljenog(ui, dostupan, *namjera, boje, katalog)
        }
        AktivniDijalog::UpozorenjeBrisanja {
            stavka,
            naziv,
            broj_nepodrzanih,
        } => crtaj_upozorenje_brisanja(
            ui,
            dostupan,
            *stavka,
            naziv,
            *broj_nepodrzanih,
            boje,
            katalog,
        ),
        AktivniDijalog::PorukaGreske { naslov, poruka } => {
            crtaj_poruku_greske(ui, dostupan, naslov, poruka, boje, katalog)
        }
        AktivniDijalog::UpozorenjaOtvaranja {
            upozorenja,
            otvoren_popis,
        } => crtaj_upozorenja_otvaranja(ui, dostupan, upozorenja, otvoren_popis, boje, katalog),
    }
}

fn crtaj_potvrdu_nespremljenog(
    ui: &mut egui::Ui,
    dostupan: Rect,
    namjera: NamjeraPotvrde,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let sirina = 420.0;
    let visina = 150.0;
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, visina));

    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 10.0);

    let naslov = katalog.tekst("potvrda-naslov");
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 24.0),
        &naslov,
        font_srednji(15.0),
        boje.tekst,
    );

    let poruka = katalog.tekst("potvrda-poruka");
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 60.0),
        &poruka,
        font_obicni(13.0),
        boje.tekst_2,
    );

    let by = okvir.max.y - 42.0;
    let sirina_gumba = 96.0;

    // Odustani
    let okvir_odustani =
        Rect::from_min_size(pos2(okvir.min.x + 20.0, by), vec2(sirina_gumba, 30.0));
    let odustani = gumb_dijaloga(
        ui,
        okvir_odustani,
        &katalog.tekst("potvrda-gumb-odustani"),
        false,
        false,
        boje,
    );

    // Ne spremaj
    let okvir_ne_spremaj = Rect::from_min_size(pos2(okvir.max.x - 224.0, by), vec2(104.0, 30.0));
    let ne_spremaj = gumb_dijaloga(
        ui,
        okvir_ne_spremaj,
        &katalog.tekst("potvrda-gumb-ne-spremaj"),
        false,
        false,
        boje,
    );

    // Spremi
    let okvir_spremi = Rect::from_min_size(pos2(okvir.max.x - 110.0, by), vec2(sirina_gumba, 30.0));
    let spremi = gumb_dijaloga(
        ui,
        okvir_spremi,
        &katalog.tekst("potvrda-gumb-spremi"),
        true,
        false,
        boje,
    );

    if odustani.clicked() {
        Some(IshodDijaloga::PotvrdaOdustani)
    } else if ne_spremaj.clicked() {
        Some(IshodDijaloga::PotvrdaNeSpremaj(namjera))
    } else if spremi.clicked() {
        Some(IshodDijaloga::PotvrdaSpremi(namjera))
    } else {
        None
    }
}

fn crtaj_upozorenje_brisanja(
    ui: &mut egui::Ui,
    dostupan: Rect,
    stavka: Stavka,
    naziv: &str,
    broj_nepodrzanih: usize,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let sirina = 460.0;
    let visina = 175.0;
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, visina));

    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 10.0);

    let je_mapa = matches!(stavka, Stavka::Mapa(_));
    let naslov = if je_mapa {
        "Upozorenje prije brisanja mape"
    } else {
        "Upozorenje prije brisanja sloja"
    };
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 24.0),
        naslov,
        font_srednji(15.0),
        boje.crvena,
    );

    let poruka = if je_mapa {
        katalog.upozorenje_brisanja_mape(naziv, broj_nepodrzanih)
    } else {
        katalog.upozorenje_brisanja_sloja(naziv, broj_nepodrzanih)
    };

    let tekst_okvir = Rect::from_min_size(
        pos2(okvir.min.x + 20.0, okvir.min.y + 50.0),
        vec2(sirina - 40.0, 60.0),
    );
    ui.put(
        tekst_okvir,
        egui::Label::new(
            egui::RichText::new(&poruka)
                .font(font_obicni(13.0))
                .color(boje.tekst_2),
        ),
    );

    let by = okvir.max.y - 42.0;

    let okvir_odustani = Rect::from_min_size(pos2(okvir.max.x - 220.0, by), vec2(96.0, 30.0));
    let odustani = gumb_dijaloga(
        ui,
        okvir_odustani,
        &katalog.tekst("brisanje-gumb-odustani"),
        false,
        false,
        boje,
    );

    let oznaka_obrisi = if je_mapa {
        katalog.tekst("brisanje-gumb-obrisi-mapu")
    } else {
        katalog.tekst("brisanje-gumb-obrisi-sloj")
    };
    let okvir_obrisi = Rect::from_min_size(pos2(okvir.max.x - 114.0, by), vec2(98.0, 30.0));
    let obrisi = gumb_dijaloga(ui, okvir_obrisi, &oznaka_obrisi, false, true, boje);

    if odustani.clicked() {
        Some(IshodDijaloga::ObrisiOdustani)
    } else if obrisi.clicked() {
        Some(IshodDijaloga::ObrisiPotvrdjeno(stavka))
    } else {
        None
    }
}

fn crtaj_poruku_greske(
    ui: &mut egui::Ui,
    dostupan: Rect,
    naslov: &str,
    poruka: &str,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let sirina = 440.0;
    let visina = 160.0;
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, visina));

    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 10.0);

    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 24.0),
        naslov,
        font_srednji(15.0),
        boje.crvena,
    );

    let tekst_okvir = Rect::from_min_size(
        pos2(okvir.min.x + 20.0, okvir.min.y + 50.0),
        vec2(sirina - 40.0, 55.0),
    );
    ui.put(
        tekst_okvir,
        egui::Label::new(
            egui::RichText::new(poruka)
                .font(font_obicni(13.0))
                .color(boje.tekst_2),
        ),
    );

    let by = okvir.max.y - 42.0;
    let okvir_u_redu = Rect::from_min_size(pos2(okvir.max.x - 106.0, by), vec2(86.0, 30.0));
    let u_redu = gumb_dijaloga(
        ui,
        okvir_u_redu,
        &katalog.tekst("greska-gumb-u-redu"),
        true,
        false,
        boje,
    );

    if u_redu.clicked() {
        Some(IshodDijaloga::ZatvoriPoruku)
    } else {
        None
    }
}

fn crtaj_upozorenja_otvaranja(
    ui: &mut egui::Ui,
    dostupan: Rect,
    upozorenja: &[String],
    otvoren_popis: &mut bool,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let dodatna_visina = if *otvoren_popis { 110.0 } else { 0.0 };
    let sirina = 460.0;
    let visina = 160.0 + dodatna_visina;
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, visina));

    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 10.0);

    let naslov = katalog.tekst("upozorenja-naslov");
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 24.0),
        &naslov,
        font_srednji(15.0),
        boje.narancasta,
    );

    let poruka = katalog.upozorenja_otvaranja_poruka(upozorenja.len());
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 58.0),
        &poruka,
        font_obicni(13.0),
        boje.tekst_2,
    );

    if *otvoren_popis {
        let okvir_popisa = Rect::from_min_size(
            pos2(okvir.min.x + 20.0, okvir.min.y + 80.0),
            vec2(sirina - 40.0, 95.0),
        );
        zaobljeni_okvir(ui, okvir_popisa, boje.ploca_2, Some(boje.obrub), 6.0);
        let mut py = okvir_popisa.min.y + 12.0;
        for u in upozorenja.iter().take(4) {
            tekst_lijevo(
                ui,
                pos2(okvir_popisa.min.x + 10.0, py),
                u,
                font_obicni(11.0),
                boje.tekst_2,
            );
            py += 20.0;
        }
    }

    let by = okvir.max.y - 42.0;

    let oznaka_popisa = if *otvoren_popis {
        katalog.tekst("upozorenja-sakrij-popis")
    } else {
        katalog.pokazi_popis_upozorenja(upozorenja.len())
    };

    let okvir_popis_gumb = Rect::from_min_size(pos2(okvir.min.x + 20.0, by), vec2(140.0, 30.0));
    let g_popis = gumb_dijaloga(ui, okvir_popis_gumb, &oznaka_popisa, false, false, boje);

    let okvir_u_redu = Rect::from_min_size(pos2(okvir.max.x - 106.0, by), vec2(86.0, 30.0));
    let u_redu = gumb_dijaloga(
        ui,
        okvir_u_redu,
        &katalog.tekst("upozorenja-gumb-u-redu"),
        true,
        false,
        boje,
    );

    if g_popis.clicked() {
        Some(IshodDijaloga::UpozorenjaPreklopiPopis)
    } else if u_redu.clicked() {
        Some(IshodDijaloga::ZatvoriPoruku)
    } else {
        None
    }
}

fn gumb_dijaloga(
    ui: &mut egui::Ui,
    okvir: Rect,
    tekst: &str,
    istaknut: bool,
    opasno: bool,
    boje: &Boje,
) -> Response {
    let (ispuna, obrub, boja_teksta) = if opasno {
        (boje.crvena, Some(boje.crvena), Color32::WHITE)
    } else if istaknut {
        (
            boje.primarna,
            Some(boje.primarna_tamna),
            boje.primarna_tekst,
        )
    } else {
        (boje.ploca_2, Some(boje.obrub), boje.tekst)
    };
    zaobljeni_okvir(ui, okvir, ispuna, obrub, Razmaci::ZAOBLJENJE_GUMBA);
    tekst_sredina(ui, okvir.center(), tekst, font_srednji(12.0), boja_teksta);
    let id = ui.make_persistent_id((
        "gumb_dijaloga",
        tekst,
        okvir.min.x.to_bits(),
        okvir.min.y.to_bits(),
    ));
    let odg = ui.interact(okvir, id, Sense::click());
    odg.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), tekst));
    odg
}
