//! Prikaz padajućeg izbornika Datoteka i modalnih dijaloga (potvrde, greške, upozorenja).

use crate::izvoz::{Prolaz, StavkaPregleda};
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
    Uvezi,
    Izvezi,
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
    /// Obavijest o uspjehu, npr. sažetak uvoza.
    Obavijest {
        naslov: String,
        poruka: String,
    },
    UpozorenjaOtvaranja {
        upozorenja: Vec<String>,
        otvoren_popis: bool,
    },
    /// Spremanje je odbijeno jer je datoteku promijenio drugi program; nudi se Spremi kao.
    SukobSpremanja,
    /// Inačica, licenca i izvori karte.
    OProgramu,
    /// Prečaci tipkovnice i miša.
    Precaci,
    /// Pregled prije izvoza u KML (zaslon K).
    Izvoz(crate::izvoz::PripremaIzvoza),
    /// Kopija rada koju je ostavio program koji se srušio.
    Oporavak(crate::oporavak::Ostatak),
    /// Odabir slojeva iz GeoPackagea koji se kopiraju u projekt (zaslon G).
    UvozGpkg(crate::uvoz_gpkg::PripremaUvoza),
}

/// Radnja koju je korisnik odabrao u dijalogu.
#[derive(Clone, Debug, PartialEq)]
pub enum IshodDijaloga {
    PotvrdaSpremi(NamjeraPotvrde),
    PotvrdaNeSpremaj(NamjeraPotvrde),
    ObrisiPotvrdjeno(Stavka),
    UpozorenjaPreklopiPopis,
    ZatvoriPoruku,
    SukobSpremiKao,
    /// Glavni gumb prozorčića izvoza ili uvoza: izvoz bira datoteku, a uvoz kopira slojeve.
    Prihvati,
    /// Vrati (`true`) ili odbaci kopiju nespremljenog rada.
    Oporavak(bool),
}

/// Oznaka prečaca kako je pišu programi na tom sustavu: na Macu `⌘S`, drugdje `Ctrl+S`.
pub fn oznaka_precaca(
    katalog: &KatalogPoruka,
    sustav: egui::os::OperatingSystem,
    kljuc: &str,
) -> String {
    let naziv = if sustav == egui::os::OperatingSystem::Mac {
        "mac"
    } else {
        "ostali"
    };
    katalog.s_tekstovima(kljuc, &[("sustav", naziv.to_owned())])
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
    let precac = |kljuc: &str| oznaka_precaca(katalog, ui.ctx().os(), kljuc);
    let stavke = [
        (
            StavkaIzbornikaDatoteka::Novi,
            katalog.tekst("izbornik-datoteka-novi"),
            precac("precac-novi"),
        ),
        (
            StavkaIzbornikaDatoteka::Otvori,
            katalog.tekst("izbornik-datoteka-otvori"),
            precac("precac-otvori"),
        ),
        (
            StavkaIzbornikaDatoteka::Spremi,
            katalog.tekst("izbornik-datoteka-spremi"),
            precac("precac-spremi"),
        ),
        (
            StavkaIzbornikaDatoteka::SpremiKao,
            katalog.tekst("izbornik-datoteka-spremi-kao"),
            precac("precac-spremi-kao"),
        ),
        (
            StavkaIzbornikaDatoteka::Uvezi,
            katalog.tekst("izbornik-datoteka-uvezi"),
            String::new(),
        ),
        (
            StavkaIzbornikaDatoteka::Izvezi,
            katalog.tekst("izbornik-datoteka-izvezi"),
            String::new(),
        ),
    ];

    let sirina = 220.0;
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
            crtaj_poruku(ui, dostupan, (naslov, boje.crvena), poruka, boje, katalog)
        }
        AktivniDijalog::Obavijest { naslov, poruka } => {
            crtaj_poruku(ui, dostupan, (naslov, boje.tekst), poruka, boje, katalog)
        }
        AktivniDijalog::UpozorenjaOtvaranja {
            upozorenja,
            otvoren_popis,
        } => crtaj_upozorenja_otvaranja(ui, dostupan, upozorenja, otvoren_popis, boje, katalog),
        AktivniDijalog::SukobSpremanja => crtaj_sukob_spremanja(ui, dostupan, boje, katalog),
        ostali => crtaj_ostale(ui, dostupan, ostali, boje, katalog),
    }
}

/// Kućica, ikona vrste, naziv i opis jednog sloja. Klik mijenja odabir.
fn redak_sloja_uvoza(
    ui: &mut egui::Ui,
    sloj: &mut crate::uvoz_gpkg::SlojIzvora,
    okvir: Rect,
    boje: &Boje,
    katalog: &KatalogPoruka,
) {
    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 8.0);
    let kucica =
        Rect::from_center_size(pos2(okvir.min.x + 22.0, okvir.center().y), vec2(18.0, 18.0));
    if sloj.odabran {
        ui.painter().rect_filled(kucica, 4.0, boje.primarna);
        tekst_sredina(
            ui,
            kucica.center(),
            "✓",
            font_srednji(12.0),
            boje.primarna_tekst,
        );
    } else {
        ui.painter().rect_stroke(
            kucica,
            4.0,
            egui::Stroke::new(2.0, boje.obrub_2),
            egui::StrokeKind::Inside,
        );
    }
    let x = okvir.min.x + 44.0;
    tekst_lijevo(
        ui,
        pos2(x, okvir.min.y + 15.0),
        &sloj.naziv,
        font_obicni(14.0),
        boje.tekst,
    );
    let opis = katalog.s_tekstovima(
        "uvoz-gpkg-opis-sloja",
        &[
            ("objekti", katalog.broj_objekata(sloj.objekata)),
            ("sustav", sloj.sustav.to_owned()),
        ],
    );
    tekst_lijevo(
        ui,
        pos2(x, okvir.min.y + 33.0),
        &opis,
        font_obicni(12.0),
        boje.tekst_2,
    );
    let id = ui.make_persistent_id(("uvoz_gpkg_sloj", sloj.id.0));
    let odgovor = ui.interact(okvir, id, Sense::click());
    odgovor.widget_info(|| {
        WidgetInfo::selected(Role::CheckBox, ui.is_enabled(), sloj.odabran, &sloj.naziv)
    });
    if odgovor.clicked() {
        sloj.odabran = !sloj.odabran;
    }
}

/// Zaslon G: slojevi iz datoteke s kućicama, napomena i gumbi.
fn crtaj_uvoz_gpkg(
    ui: &mut egui::Ui,
    dostupan: Rect,
    priprema: &mut crate::uvoz_gpkg::PripremaUvoza,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let sirina = 560.0;
    let vidljivih = priprema.slojevi.len().min(6) as f32;
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, 240.0 + 56.0 * vidljivih));
    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 12.0);
    let x = okvir.min.x + 24.0;
    tekst_lijevo(
        ui,
        pos2(x, okvir.min.y + 30.0),
        &katalog.tekst("uvoz-gpkg-naslov"),
        font_srednji(18.0),
        boje.tekst,
    );
    let datoteka = katalog.s_tekstovima(
        "uvoz-gpkg-datoteka",
        &[("datoteka", priprema.naziv.clone())],
    );
    tekst_lijevo(
        ui,
        pos2(x, okvir.min.y + 60.0),
        &datoteka,
        font_obicni(13.0),
        boje.tekst_2,
    );
    tekst_lijevo(
        ui,
        pos2(x, okvir.min.y + 90.0),
        &katalog.tekst("uvoz-gpkg-slojevi"),
        font_obicni(13.0),
        boje.tekst_2,
    );
    let popis = Rect::from_min_size(
        pos2(x, okvir.min.y + 104.0),
        vec2(sirina - 48.0, 56.0 * vidljivih),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(popis), |ui| {
        egui::ScrollArea::vertical()
            .max_height(popis.height())
            .show(ui, |ui| {
                for sloj in &mut priprema.slojevi {
                    let (r, _) =
                        ui.allocate_exact_size(vec2(popis.width() - 12.0, 50.0), Sense::hover());
                    redak_sloja_uvoza(ui, sloj, r, boje, katalog);
                    ui.add_space(6.0);
                }
            });
    });
    let mut napomena = katalog.tekst("uvoz-gpkg-kopija");
    let polja = priprema.izostavljena_polja();
    if !polja.is_empty() {
        napomena.push(' ');
        napomena.push_str(&katalog.s_tekstovima("uvoz-gpkg-polja", &[("popis", polja.join(", "))]));
    }
    let galija = ui
        .painter()
        .layout(napomena, font_obicni(12.5), boje.tekst_2, sirina - 48.0);
    ui.painter()
        .galley(pos2(x, popis.max.y + 14.0), galija, boje.tekst_2);
    let by = okvir.max.y - 48.0;
    let uvezi = Rect::from_min_size(pos2(okvir.max.x - 164.0, by), vec2(140.0, 32.0));
    let odustani = Rect::from_min_size(pos2(uvezi.min.x - 108.0, by), vec2(100.0, 32.0));
    let odustani = gumb_dijaloga(
        ui,
        odustani,
        &katalog.tekst("izvoz-odustani"),
        false,
        false,
        boje,
    );
    let broj = priprema.odabranih();
    let gumb = katalog.s_tekstovima("uvoz-gpkg-gumb", &[("slojevi", katalog.broj_slojeva(broj))]);
    let uvezi = gumb_dijaloga(ui, uvezi, &gumb, true, false, boje);
    if uvezi.clicked() && broj > 0 {
        Some(IshodDijaloga::Prihvati)
    } else {
        odustani.clicked().then_some(IshodDijaloga::ZatvoriPoruku)
    }
}

/// Prozorčići s vlastitim sadržajem: o programu, prečaci, izvoz, uvoz i oporavak.
fn crtaj_ostale(
    ui: &mut egui::Ui,
    dostupan: Rect,
    dijalog: &mut AktivniDijalog,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    match dijalog {
        AktivniDijalog::OProgramu => crtaj_o_programu(ui, dostupan, boje, katalog),
        AktivniDijalog::Precaci => crtaj_precace(ui, dostupan, boje, katalog),
        AktivniDijalog::Izvoz(priprema) => crtaj_izvoz(ui, dostupan, priprema, boje, katalog),
        AktivniDijalog::Oporavak(ostatak) => crtaj_oporavak(ui, dostupan, ostatak, boje, katalog),
        AktivniDijalog::UvozGpkg(priprema) => {
            crtaj_uvoz_gpkg(ui, dostupan, priprema, boje, katalog)
        }
        _ => None,
    }
}

/// Ponuda da se vrati rad koji nije spremljen prije pada programa.
fn crtaj_oporavak(
    ui: &mut egui::Ui,
    dostupan: Rect,
    ostatak: &crate::oporavak::Ostatak,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let sirina = 460.0;
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, 190.0));
    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 10.0);
    let naslov = katalog.tekst("oporavak-naslov");
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 24.0),
        &naslov,
        font_srednji(15.0),
        boje.tekst,
    );
    let tekstovi = crate::oporavak::tekstovi_ostatka(katalog, ostatak);
    let poruka = katalog.s_tekstovima("oporavak-poruka", &tekstovi);
    let galija = ui
        .painter()
        .layout(poruka, font_obicni(13.0), boje.tekst_2, sirina - 40.0);
    ui.painter().galley(
        pos2(okvir.min.x + 20.0, okvir.min.y + 48.0),
        galija,
        boje.tekst_2,
    );
    let by = okvir.max.y - 46.0;
    let vrati = Rect::from_min_size(pos2(okvir.max.x - 116.0, by), vec2(96.0, 30.0));
    let odbaci = Rect::from_min_size(pos2(vrati.min.x - 104.0, by), vec2(96.0, 30.0));
    let odbaci = gumb_dijaloga(
        ui,
        odbaci,
        &katalog.tekst("oporavak-odbaci"),
        false,
        false,
        boje,
    );
    let vrati = gumb_dijaloga(
        ui,
        vrati,
        &katalog.tekst("oporavak-vrati"),
        true,
        false,
        boje,
    );
    if vrati.clicked() {
        Some(IshodDijaloga::Oporavak(true))
    } else {
        odbaci.clicked().then_some(IshodDijaloga::Oporavak(false))
    }
}

/// Znak uz stavku pregleda: kvačica, uskličnik ili križić u krugu.
fn znak_prolaza(ui: &egui::Ui, srediste: egui::Pos2, prolaz: Prolaz, boje: &Boje) {
    let (boja, znak) = match prolaz {
        Prolaz::Da => (boje.zelena, "✓"),
        Prolaz::Pazi => (boje.upozorenje, "!"),
        Prolaz::Ne => (boje.crvena, "×"),
    };
    ui.painter().circle_filled(srediste, 9.0, boja);
    tekst_sredina(ui, srediste, znak, font_srednji(12.0), Color32::WHITE);
}

/// Vraća visinu stavke.
fn crtaj_stavku_pregleda(
    ui: &mut egui::Ui,
    stavka: &StavkaPregleda,
    pocetak: egui::Pos2,
    sirina: f32,
    boje: &Boje,
) -> f32 {
    znak_prolaza(
        ui,
        pos2(pocetak.x + 9.0, pocetak.y + 10.0),
        stavka.prolaz,
        boje,
    );
    let x = pocetak.x + 30.0;
    tekst_lijevo(
        ui,
        pos2(x, pocetak.y + 10.0),
        &stavka.naslov,
        font_obicni(14.0),
        boje.tekst,
    );
    let opis = ui.painter().layout(
        stavka.opis.clone(),
        font_obicni(12.5),
        boje.tekst_2,
        sirina - 30.0,
    );
    let visina = opis.size().y;
    ui.painter()
        .galley(pos2(x, pocetak.y + 20.0), opis, boje.tekst_2);
    visina + 34.0
}

/// Gumbi za format izvoza. Klik na drugi format mijenja pregled.
fn izbor_formata(
    ui: &mut egui::Ui,
    priprema: &mut crate::izvoz::PripremaIzvoza,
    pocetak: egui::Pos2,
    boje: &Boje,
) {
    for (i, f) in crate::izvoz::FormatIzvoza::SVI.into_iter().enumerate() {
        let okvir = Rect::from_min_size(
            pos2(pocetak.x + i as f32 * 96.0, pocetak.y),
            vec2(88.0, 28.0),
        );
        if gumb_dijaloga(ui, okvir, f.naziv(), priprema.format == f, false, boje).clicked() {
            priprema.format = f;
        }
    }
}

/// Zaslon K: format, datoteka, popis onoga što ide u datoteku i gumbi.
fn crtaj_izvoz(
    ui: &mut egui::Ui,
    dostupan: Rect,
    priprema: &mut crate::izvoz::PripremaIzvoza,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let stavke = priprema.stavke(katalog);
    let sirina = 560.0;
    let visina = 210.0 + 60.0 * stavke.len() as f32;
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, visina));
    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 12.0);
    let x = okvir.min.x + 24.0;
    let naslov = katalog.s_tekstovima(
        "izvoz-naslov",
        &[("format", priprema.format.naziv().to_owned())],
    );
    tekst_lijevo(
        ui,
        pos2(x, okvir.min.y + 30.0),
        &naslov,
        font_srednji(18.0),
        boje.tekst,
    );
    izbor_formata(ui, priprema, pos2(x, okvir.min.y + 52.0), boje);
    let tekstovi = [
        ("naziv", priprema.naziv.clone()),
        ("nastavak", priprema.format.nastavak().to_owned()),
    ];
    let datoteka = katalog.s_tekstovima("izvoz-datoteka", &tekstovi);
    tekst_lijevo(
        ui,
        pos2(x, okvir.min.y + 100.0),
        &datoteka,
        font_obicni(13.0),
        boje.tekst_2,
    );
    let crta = okvir.min.y + 120.0;
    ui.painter().line_segment(
        [pos2(x, crta), pos2(okvir.max.x - 24.0, crta)],
        egui::Stroke::new(1.0, boje.obrub),
    );
    tekst_lijevo(
        ui,
        pos2(x, crta + 20.0),
        &katalog.tekst("izvoz-sto-ide"),
        font_obicni(13.0),
        boje.tekst_2,
    );
    let mut y = crta + 38.0;
    for stavka in &stavke {
        y += crtaj_stavku_pregleda(ui, stavka, pos2(x, y), sirina - 48.0, boje);
    }
    let by = okvir.max.y - 48.0;
    let izvezi = Rect::from_min_size(pos2(okvir.max.x - 124.0, by), vec2(100.0, 32.0));
    let odustani = Rect::from_min_size(pos2(izvezi.min.x - 108.0, by), vec2(100.0, 32.0));
    let odustani = gumb_dijaloga(
        ui,
        odustani,
        &katalog.tekst("izvoz-odustani"),
        false,
        false,
        boje,
    );
    let gumb = katalog.tekst("izvoz-gumb");
    let moze = priprema.moze();
    let izvezi = ui
        .add_enabled_ui(moze, |ui| {
            gumb_dijaloga(ui, izvezi, &gumb, true, false, boje)
        })
        .inner;
    if izvezi.clicked() && moze {
        Some(IshodDijaloga::Prihvati)
    } else {
        odustani.clicked().then_some(IshodDijaloga::ZatvoriPoruku)
    }
}

/// Retci prozorčića „O programu”, redom.
#[must_use]
pub fn retci_o_programu(katalog: &KatalogPoruka) -> [String; 4] {
    [
        katalog.tekst("o-programu-opis"),
        katalog.tekst("o-programu-licenca"),
        katalog.tekst("o-programu-karta"),
        katalog.tekst("o-programu-ostalo"),
    ]
}

fn crtaj_o_programu(
    ui: &mut egui::Ui,
    dostupan: Rect,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let sirina = 460.0;
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, 290.0));
    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 10.0);
    let naslov = katalog.s_tekstovima(
        "o-programu-naslov",
        &[("inacica", env!("CARGO_PKG_VERSION").to_string())],
    );
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 26.0),
        &naslov,
        font_srednji(17.0),
        boje.tekst,
    );
    let tekst_okvir = Rect::from_min_size(
        pos2(okvir.min.x + 20.0, okvir.min.y + 52.0),
        vec2(sirina - 40.0, 180.0),
    );
    ui.scope_builder(egui::UiBuilder::new().max_rect(tekst_okvir), |ui| {
        for redak in retci_o_programu(katalog) {
            ui.label(
                egui::RichText::new(redak)
                    .font(font_obicni(13.0))
                    .color(boje.tekst_2),
            );
            ui.add_space(6.0);
        }
    });
    let zatvori = gumb_dijaloga(
        ui,
        Rect::from_min_size(
            pos2(okvir.max.x - 106.0, okvir.max.y - 42.0),
            vec2(86.0, 30.0),
        ),
        &katalog.tekst("o-programu-zatvori"),
        true,
        false,
        boje,
    );
    zatvori.clicked().then_some(IshodDijaloga::ZatvoriPoruku)
}

/// Skupine prečaca: naslov skupine i parovi (radnja, ključ prečaca). Lijevi stupac su Datoteka i
/// Uređivanje, a desni crtanje, odabir i karta.
const PRECACI: [(&str, &[(&str, &str)]); 5] = [
    (
        "precaci-datoteka",
        &[
            ("izbornik-datoteka-novi", "precac-novi"),
            ("izbornik-datoteka-otvori", "precac-otvori"),
            ("izbornik-datoteka-spremi", "precac-spremi"),
            ("izbornik-datoteka-spremi-kao", "precac-spremi-kao"),
        ],
    ),
    (
        "precaci-uredjivanje",
        &[
            ("gumb-ponisti", "precac-ponisti"),
            ("gumb-ponovi", "precac-ponovi"),
            ("precaci-trazi", "precac-trazi"),
            ("precaci-preimenuj", "precac-preimenuj"),
            ("precaci-obrisi", "precac-obrisi"),
            ("precaci-odustani", "precac-odustani"),
            ("precaci-pomoc", "precac-pomoc"),
        ],
    ),
    (
        "precaci-crtanje",
        &[
            ("precaci-dodaj-tocku", "precac-klik"),
            ("precaci-zavrsi", "precac-desni-klik"),
            ("precaci-obrisi-tocku", "precac-obrisi-tocku"),
            ("precaci-pomakni-tocku", "precac-povuci"),
        ],
    ),
    (
        "precaci-odabir",
        &[
            ("precaci-dodaj-u-odabir", "precac-klik"),
            ("precaci-izbaci-iz-odabira", "precac-shift-klik"),
            ("precaci-okvir", "precac-okvir"),
            ("precaci-okvir-smjer", "precac-okvir-smjer"),
        ],
    ),
    (
        "precaci-karta",
        &[
            ("precaci-priblizi", "precac-kotacic"),
            ("precaci-pomakni-kartu", "precac-povuci"),
        ],
    ),
];

/// Redak popisa prečaca: naslov skupine ili radnja s prečacem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RedakPrecaca {
    Skupina(String),
    Precac { radnja: String, tipke: String },
}

/// Prečaci kako ih piše sustav: na Macu `⌘Z`, drugdje `Ctrl+Z`.
#[must_use]
pub fn retci_precaca(
    katalog: &KatalogPoruka,
    sustav: egui::os::OperatingSystem,
) -> [Vec<RedakPrecaca>; 2] {
    let stupac = |skupine: &[(&str, &[(&str, &str)])]| {
        let mut retci = Vec::new();
        for (naslov, parovi) in skupine {
            retci.push(RedakPrecaca::Skupina(katalog.tekst(naslov)));
            retci.extend(parovi.iter().map(|(radnja, tipke)| RedakPrecaca::Precac {
                radnja: katalog.tekst(radnja),
                tipke: oznaka_precaca(katalog, sustav, tipke),
            }));
        }
        retci
    };
    [stupac(&PRECACI[..2]), stupac(&PRECACI[2..])]
}

fn crtaj_precace(
    ui: &mut egui::Ui,
    dostupan: Rect,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let okvir = Rect::from_center_size(dostupan.center(), vec2(900.0, 430.0));
    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 10.0);
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 26.0),
        &katalog.tekst("precaci-naslov"),
        font_srednji(17.0),
        boje.tekst,
    );
    let stupci = retci_precaca(katalog, ui.ctx().os());
    for (i, stupac) in stupci.iter().enumerate() {
        let x = okvir.min.x + 20.0 + i as f32 * 440.0;
        let mut y = okvir.min.y + 62.0;
        for redak in stupac {
            y += crtaj_redak_precaca(ui, redak, x, y, boje);
        }
    }
    let zatvori = gumb_dijaloga(
        ui,
        Rect::from_min_size(
            pos2(okvir.max.x - 106.0, okvir.max.y - 42.0),
            vec2(86.0, 30.0),
        ),
        &katalog.tekst("o-programu-zatvori"),
        true,
        false,
        boje,
    );
    zatvori.clicked().then_some(IshodDijaloga::ZatvoriPoruku)
}

/// Vraća visinu retka.
fn crtaj_redak_precaca(ui: &egui::Ui, redak: &RedakPrecaca, x: f32, y: f32, boje: &Boje) -> f32 {
    match redak {
        RedakPrecaca::Skupina(naslov) => {
            tekst_lijevo(ui, pos2(x, y + 8.0), naslov, font_srednji(13.0), boje.tekst);
            28.0
        }
        RedakPrecaca::Precac { radnja, tipke } => {
            tekst_lijevo(ui, pos2(x, y), radnja, font_obicni(12.5), boje.tekst_2);
            tekst_lijevo(
                ui,
                pos2(x + 212.0, y),
                tipke,
                font_srednji(12.5),
                boje.tekst,
            );
            21.0
        }
    }
}

/// Odluke (9. 11. 2026.): nakon sukoba nudi se samo Spremi kao. Gumba koji bi pregazio izmjene
/// drugog programa nema.
fn crtaj_sukob_spremanja(
    ui: &mut egui::Ui,
    dostupan: Rect,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let sirina = 440.0;
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, 180.0));
    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 10.0);
    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 24.0),
        &katalog.tekst("sukob-naslov"),
        font_srednji(15.0),
        boje.crvena,
    );
    let tekst_okvir = Rect::from_min_size(
        pos2(okvir.min.x + 20.0, okvir.min.y + 50.0),
        vec2(sirina - 40.0, 75.0),
    );
    ui.put(
        tekst_okvir,
        egui::Label::new(
            egui::RichText::new(katalog.tekst("greska-sukob-promjena"))
                .font(font_obicni(13.0))
                .color(boje.tekst_2),
        ),
    );
    let by = okvir.max.y - 42.0;
    let odustani = gumb_dijaloga(
        ui,
        Rect::from_min_size(pos2(okvir.max.x - 230.0, by), vec2(96.0, 30.0)),
        &katalog.tekst("sukob-gumb-odustani"),
        false,
        false,
        boje,
    );
    let spremi_kao = gumb_dijaloga(
        ui,
        Rect::from_min_size(pos2(okvir.max.x - 124.0, by), vec2(104.0, 30.0)),
        &katalog.tekst("sukob-gumb-spremi-kao"),
        true,
        false,
        boje,
    );
    if spremi_kao.clicked() {
        Some(IshodDijaloga::SukobSpremiKao)
    } else if odustani.clicked() {
        Some(IshodDijaloga::ZatvoriPoruku)
    } else {
        None
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
        Some(IshodDijaloga::ZatvoriPoruku)
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
        Some(IshodDijaloga::ZatvoriPoruku)
    } else if obrisi.clicked() {
        Some(IshodDijaloga::ObrisiPotvrdjeno(stavka))
    } else {
        None
    }
}

/// Poruka s naslovom i gumbom „U redu”. Visina prati duljinu teksta.
fn crtaj_poruku(
    ui: &mut egui::Ui,
    dostupan: Rect,
    (naslov, boja_naslova): (&str, egui::Color32),
    poruka: &str,
    boje: &Boje,
    katalog: &KatalogPoruka,
) -> Option<IshodDijaloga> {
    let sirina = 440.0;
    let visina_teksta = ui
        .painter()
        .layout(
            poruka.to_owned(),
            font_obicni(13.0),
            boje.tekst_2,
            sirina - 40.0,
        )
        .size()
        .y;
    let visina = 105.0 + visina_teksta.max(55.0);
    let okvir = Rect::from_center_size(dostupan.center(), vec2(sirina, visina));

    zaobljeni_okvir(ui, okvir, boje.ploca, Some(boje.obrub), 10.0);

    tekst_lijevo(
        ui,
        pos2(okvir.min.x + 20.0, okvir.min.y + 24.0),
        naslov,
        font_srednji(15.0),
        boja_naslova,
    );

    let tekst_okvir = Rect::from_min_size(
        pos2(okvir.min.x + 20.0, okvir.min.y + 50.0),
        vec2(sirina - 40.0, visina - 105.0),
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

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn ostali_prozorcici_ne_crtaju_one_s_vlastitom_granom() {
        let ctx = egui::Context::default();
        let katalog = KatalogPoruka::hrvatski();
        let mut izlaz = ctx.run_ui(egui::RawInput::default(), |ui| {
            let mut d = AktivniDijalog::SukobSpremanja;
            let boje = Boje::za_temu(false);
            assert!(crtaj_ostale(ui, ui.max_rect(), &mut d, &boje, &katalog).is_none());
        });
        izlaz.textures_delta.clear();
    }
}
