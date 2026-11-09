//! Tražilica koordinata, slojeva i objekata u karti i projektu.

use crate::karta_prozora::{KartaProzora, obuhvat_vrhova};
use crate::poruke::KatalogPoruka;
use crate::sastavnice::zaobljeni_okvir;
use crate::stanje::{Odabir, StanjePrikaza};
use crate::tema::{Boje, font_obicni, font_srednji};
use eframe::egui::{self, Key, Painter, Pos2, Rect, Sense, Stroke, TextEdit, Ui, pos2, vec2};
use koordinate::{
    IshodPrepoznavanja, PrepoznataKoordinata, RazlogRaspona, Sustav, prepoznaj_koordinate,
};
use slojevi::{Geometrija, IdMape, IdObjekta, IdSloja, Objekt, Projekt, Sloj, Uredjivac, Vrh};

/// Stanje tražilice unutar prozora.
#[derive(Clone, Debug, Default)]
pub struct StanjeTrazilice {
    pub tekst: String,
    pub fokusirana: bool,
    pub fokusiraj: bool,
    pub otvorena: bool,
    pub privremena_oznaka: Option<[f64; 2]>,
    zadnji_upit: String,
    zadnja_inacica: u64,
    spremljeni_rezultati: Vec<RezultatPretrage>,
    zadnja_greska: Option<RazlogRaspona>,
}

impl StanjeTrazilice {
    pub fn ponisti_ili_zatvori(&mut self) -> bool {
        let ima_stanja =
            self.privremena_oznaka.is_some() || self.fokusirana || !self.tekst.is_empty();
        if ima_stanja {
            self.tekst.clear();
            self.privremena_oznaka = None;
            self.otvorena = false;
            self.fokusirana = false;
            self.zadnji_upit.clear();
            self.spremljeni_rezultati.clear();
            self.zadnja_greska = None;
            true
        } else {
            false
        }
    }

    /// Vraća spremljene rezultate ili ih izračunava ako se promijenio tekst ili projekt.
    pub fn rezultati(
        &mut self,
        uredjivac: &Uredjivac,
        katalog: &KatalogPoruka,
    ) -> (Vec<RezultatPretrage>, Option<RazlogRaspona>) {
        let upit = self.tekst.trim();
        let inacica = uredjivac.inacica();
        if self.zadnji_upit != upit || self.zadnja_inacica != inacica {
            let (rez, greska) = pronadji_rezultate(upit, uredjivac.projekt(), katalog);
            self.spremljeni_rezultati = rez;
            self.zadnja_greska = greska;
            self.zadnji_upit = upit.to_string();
            self.zadnja_inacica = inacica;
        }
        (self.spremljeni_rezultati.clone(), self.zadnja_greska)
    }
}

/// Rezultat pretrage koordinata, sloja ili objekta.
#[derive(Clone, Debug, PartialEq)]
pub enum RezultatPretrage {
    Koordinata {
        duzina: f64,
        sirina: f64,
        tekst: String,
    },
    Sloj {
        id: IdSloja,
        naziv: String,
    },
    Objekt {
        sloj_id: IdSloja,
        objekt_id: IdObjekta,
        naziv_objekta: String,
        naziv_sloja: String,
    },
}

fn zamijeni_kvacicu(c: char) -> Option<char> {
    for &(iz, u) in &[
        ('č', 'c'),
        ('ć', 'c'),
        ('Č', 'c'),
        ('Ć', 'c'),
        ('đ', 'd'),
        ('Đ', 'd'),
        ('š', 's'),
        ('Š', 's'),
        ('ž', 'z'),
        ('Ž', 'z'),
    ] {
        if c == iz {
            return Some(u);
        }
    }
    None
}

/// Normalizira tekst uklanjanjem kvačica i pretvaranjem u mala slova.
#[must_use]
pub fn normaliziraj_za_trazenje(tekst: &str) -> String {
    let mut rez = String::with_capacity(tekst.len());
    for c in tekst.chars() {
        if let Some(zamjena) = zamijeni_kvacicu(c) {
            rez.push(zamjena);
        } else {
            for lc in c.to_lowercase() {
                rez.push(lc);
            }
        }
    }
    rez
}

fn formatiraj_wgs(katalog: &KatalogPoruka, tocka: &PrepoznataKoordinata) -> String {
    let s_slovo = if tocka.sirina >= 0.0 { 'S' } else { 'J' };
    let d_slovo = if tocka.duzina >= 0.0 { 'I' } else { 'Z' };
    let s_val = format!("{:.4}", tocka.sirina.abs()).replace('.', ",");
    let d_val = format!("{:.4}", tocka.duzina.abs()).replace('.', ",");
    let baza = format!("{s_val}° {s_slovo} {d_val}° {d_slovo}");
    if tocka.zamijenjen_redoslijed {
        let napomena = katalog.tekst("trazi-zamijenjen-redoslijed");
        format!("{baza} ({napomena})")
    } else {
        baza
    }
}

fn formatiraj_htrs(katalog: &KatalogPoruka, tocka: &PrepoznataKoordinata) -> String {
    let Some([istok, sjever]) = tocka.izvorni_htrs else {
        return String::new();
    };
    let i_val = format!("{:.2}", istok).replace('.', ",");
    let s_val = format!("{:.2}", sjever).replace('.', ",");
    if tocka.zamijenjen_redoslijed {
        let napomena = katalog.tekst("trazi-zamijenjen-htrs");
        format!("{s_val} m N {i_val} m E ({napomena})")
    } else {
        format!("{i_val} m E {s_val} m N (HTRS96/TM)")
    }
}

/// Formatira prepoznatu koordinatu prema hrvatskim pravilima prikaza.
#[must_use]
pub fn formatiraj_koordinatu(katalog: &KatalogPoruka, tocka: &PrepoznataKoordinata) -> String {
    match tocka.sustav {
        Sustav::Wgs84 => formatiraj_wgs(katalog, tocka),
        Sustav::Htrs96Tm => formatiraj_htrs(katalog, tocka),
        _ => String::new(),
    }
}

const fn kljuc_greske_raspona(razlog: RazlogRaspona) -> &'static str {
    match razlog {
        RazlogRaspona::Sirina => "trazi-greska-sirina",
        RazlogRaspona::Duzina => "trazi-greska-duzina",
        RazlogRaspona::Minute => "trazi-greska-minute",
        RazlogRaspona::Sekunde => "trazi-greska-sekunde",
        RazlogRaspona::Htrs => "trazi-greska-htrs",
        RazlogRaspona::PreviseBrojeva => "trazi-greska-previse-brojeva",
    }
}

/// Vraća lokalizirani tekst greške raspona koordinata.
#[must_use]
pub fn tekst_greske_raspona(katalog: &KatalogPoruka, razlog: RazlogRaspona) -> String {
    katalog.tekst(kljuc_greske_raspona(razlog))
}

/// Pronalazi do 8 rezultata za upisani tekst uz moguću grešku raspona.
#[must_use]
pub fn pronadji_rezultate(
    tekst: &str,
    projekt: &Projekt,
    katalog: &KatalogPoruka,
) -> (Vec<RezultatPretrage>, Option<RazlogRaspona>) {
    let t = tekst.trim();
    if t.is_empty() {
        return (Vec::new(), None);
    }
    let mut rezultati = Vec::new();
    let greska = dodaj_koordinate(&mut rezultati, t, katalog);
    dodaj_slojeve_i_objekte(&mut rezultati, t, projekt);
    rezultati.truncate(8);
    (rezultati, greska)
}

fn dodaj_koordinate(
    rezultati: &mut Vec<RezultatPretrage>,
    tekst: &str,
    katalog: &KatalogPoruka,
) -> Option<RazlogRaspona> {
    match prepoznaj_koordinate(tekst) {
        IshodPrepoznavanja::Prepoznato(tocke) => {
            for tocka in tocke {
                rezultati.push(RezultatPretrage::Koordinata {
                    duzina: tocka.duzina,
                    sirina: tocka.sirina,
                    tekst: formatiraj_koordinatu(katalog, &tocka),
                });
            }
            None
        }
        IshodPrepoznavanja::IzvanRaspona(r) => Some(r),
        IshodPrepoznavanja::NijeKoordinata => None,
    }
}

fn dodaj_slojeve_i_objekte(rezultati: &mut Vec<RezultatPretrage>, tekst: &str, projekt: &Projekt) {
    let upit = normaliziraj_za_trazenje(tekst);
    for (id, sloj) in projekt.slojevi() {
        if normaliziraj_za_trazenje(&sloj.naziv).contains(&upit) {
            rezultati.push(RezultatPretrage::Sloj {
                id,
                naziv: sloj.naziv.clone(),
            });
        }
        for objekt in sloj.objekti.values() {
            if normaliziraj_za_trazenje(&objekt.naziv).contains(&upit) {
                rezultati.push(RezultatPretrage::Objekt {
                    sloj_id: id,
                    objekt_id: objekt.id,
                    naziv_objekta: objekt.naziv.clone(),
                    naziv_sloja: sloj.naziv.clone(),
                });
            }
        }
    }
}

/// Računa obuhvat jednog objekta za zumiranje karte.
#[must_use]
pub fn obuhvat_objekta(objekt: &Objekt) -> Option<(f64, f64, f64)> {
    match &objekt.geometrija {
        Geometrija::Tocka(v) => Some((v.duzina, v.sirina, 16.0)),
        Geometrija::Linija(v) => obuhvat_vrhova(v),
        Geometrija::Poligon(p) => obuhvat_vrhova(p.iter().flatten()),
    }
}

/// Računa obuhvat svih objekata unutar jednog sloja.
#[must_use]
pub fn obuhvat_sloja(sloj: &Sloj) -> Option<(f64, f64, f64)> {
    let vrhovi = sloj.objekti.values().flat_map(|o| match &o.geometrija {
        Geometrija::Tocka(v) => vec![v],
        Geometrija::Linija(v) => v.iter().collect(),
        Geometrija::Poligon(p) => p.iter().flatten().collect(),
    });
    obuhvat_vrhova(vrhovi)
}

fn otvori_roditeljske_mape(projekt: &Projekt, stanje: &mut StanjePrikaza, sloj_id: IdSloja) {
    let mut mapa: Option<IdMape> = projekt.sloj(sloj_id).and_then(|s| s.roditelj);
    while let Some(m) = mapa {
        stanje.postavi_otvorenost_mape(m, true);
        mapa = projekt.mapa(m).and_then(|m| m.roditelj);
    }
}

/// Izvršava odabir stavke iz rezultata pretrage (pomak karte, selekcija u stablu).
pub fn primijeni_odabir(
    rezultat: &RezultatPretrage,
    karta: &mut KartaProzora,
    projekt: &Projekt,
    stanje: &mut StanjePrikaza,
    trazilica: &mut StanjeTrazilice,
) {
    match rezultat {
        RezultatPretrage::Koordinata { duzina, sirina, .. } => {
            karta.karta().idi_na(*duzina, *sirina, 16.0);
            trazilica.privremena_oznaka = Some([*duzina, *sirina]);
        }
        RezultatPretrage::Objekt {
            sloj_id, objekt_id, ..
        } => {
            if let Some(sloj) = projekt.sloj(*sloj_id)
                && let Some(objekt) = sloj.objekti.get(objekt_id)
            {
                if let Some((d, s, zum)) = obuhvat_objekta(objekt) {
                    karta.karta().idi_na(d, s, zum);
                }
                stanje.postavi_otvorenost_sloja(*sloj_id, true);
                otvori_roditeljske_mape(projekt, stanje, *sloj_id);
                stanje.odabrano = Some(Odabir::Objekt(*sloj_id, *objekt_id));
                karta
                    .karta()
                    .crtanje()
                    .postavi_odabrani(Some((*sloj_id, *objekt_id)));
            }
        }
        RezultatPretrage::Sloj { id, .. } => {
            if let Some(sloj) = projekt.sloj(*id) {
                if let Some((d, s, zum)) = obuhvat_sloja(sloj) {
                    karta.karta().idi_na(d, s, zum);
                }
                otvori_roditeljske_mape(projekt, stanje, *id);
                stanje.odabrano = Some(Odabir::Sloj(*id));
            }
        }
    }
    trazilica.otvorena = false;
    trazilica.fokusirana = false;
}

/// Crta polje tražilice i upravlja interakcijom.
pub fn crtaj_polje(
    ui: &mut Ui,
    min: Pos2,
    stanje: &mut StanjeTrazilice,
    katalog: &KatalogPoruka,
    boje: &Boje,
) -> (Rect, bool) {
    let okvir = Rect::from_min_size(min, vec2(310.0, 40.0));
    zaobljeni_okvir(ui, okvir, boje.lebdeca, Some(boje.obrub), 10.0);

    let id_ikone = ui.make_persistent_id("trazilica_ikona");
    let podrucje_ikone = Rect::from_min_max(okvir.min, pos2(okvir.min.x + 36.0, okvir.max.y));
    let odgovor_ikone = ui.interact(podrucje_ikone, id_ikone, Sense::click());
    if odgovor_ikone.clicked() {
        stanje.fokusiraj = true;
        stanje.otvorena = true;
    }

    let id_polja = ui.make_persistent_id("trazilica_polje");
    let ima_fokus = ui.memory(|m| m.has_focus(id_polja));
    let pritisnut_enter = ima_fokus && ui.input(|i| i.key_pressed(Key::Enter));
    if ima_fokus && ui.input(|i| i.key_pressed(Key::Escape)) {
        stanje.tekst.clear();
        stanje.privremena_oznaka = None;
        stanje.otvorena = false;
        ui.memory_mut(|m| m.surrender_focus(id_polja));
    }

    let mut podrucje = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_max(
        pos2(okvir.min.x + 36.0, okvir.min.y + 10.0),
        okvir.max,
    )));

    let polje = podrucje.add(
        TextEdit::singleline(&mut stanje.tekst)
            .id(id_polja)
            .hint_text(katalog.tekst("trazi-mjesto"))
            .font(font_obicni(13.0))
            .text_color(boje.tekst)
            .frame(egui::Frame::NONE)
            .desired_width(260.0),
    );

    if stanje.fokusiraj {
        polje.request_focus();
        stanje.fokusiraj = false;
    }

    stanje.fokusirana = ui.memory(|m| m.has_focus(id_polja));

    if polje.changed() {
        stanje.privremena_oznaka = None;
        stanje.otvorena = true;
    }
    if polje.clicked() {
        stanje.otvorena = true;
    }

    (okvir, pritisnut_enter)
}

/// Crta padajući popis rezultata pretrage ispod polja.
pub fn crtaj_popis_rezultata(
    ui: &mut Ui,
    trazilica_okvir: Rect,
    rezultati: &[RezultatPretrage],
    greska: Option<RazlogRaspona>,
    tekst_upita: &str,
    katalog: &KatalogPoruka,
    boje: &Boje,
) -> Option<RezultatPretrage> {
    let prazan_upit = tekst_upita.trim().is_empty();
    if prazan_upit {
        return None;
    }
    let broj_redaka = if rezultati.is_empty() {
        1
    } else {
        rezultati.len()
    };
    let visina = (broj_redaka as f32) * 36.0 + 8.0;
    let okvir = Rect::from_min_size(
        pos2(trazilica_okvir.min.x, trazilica_okvir.max.y + 4.0),
        vec2(310.0, visina),
    );

    zaobljeni_okvir(ui, okvir, boje.lebdeca, Some(boje.obrub), 8.0);

    if rezultati.is_empty() {
        let tekst = if let Some(razlog) = greska {
            tekst_greske_raspona(katalog, razlog)
        } else {
            katalog.tekst("trazi-nema-rezultata")
        };
        ui.painter().text(
            pos2(okvir.min.x + 12.0, okvir.center().y),
            egui::Align2::LEFT_CENTER,
            tekst,
            font_obicni(12.0),
            boje.tekst_3,
        );
        return None;
    }

    let mut odabrano = None;
    for (i, rez) in rezultati.iter().enumerate() {
        let redak_min = pos2(okvir.min.x + 4.0, okvir.min.y + 4.0 + (i as f32) * 36.0);
        let redak_okvir = Rect::from_min_size(redak_min, vec2(302.0, 34.0));
        let odgovor = ui.allocate_rect(redak_okvir, Sense::click());
        if odgovor.hovered() {
            ui.painter().rect_filled(redak_okvir, 6.0, boje.odabir);
        }
        if odgovor.clicked() {
            odabrano = Some(rez.clone());
        }
        nacrtaj_redak_rezultata(ui.painter(), redak_okvir, rez, katalog, boje);
    }
    odabrano
}

fn nacrtaj_redak_rezultata(
    s: &Painter,
    okvir: Rect,
    rez: &RezultatPretrage,
    katalog: &KatalogPoruka,
    boje: &Boje,
) {
    match rez {
        RezultatPretrage::Koordinata { tekst, .. } => {
            s.text(
                pos2(okvir.min.x + 8.0, okvir.center().y),
                egui::Align2::LEFT_CENTER,
                tekst,
                font_obicni(13.0),
                boje.tekst,
            );
        }
        RezultatPretrage::Sloj { naziv, .. } => {
            let tekst = katalog.s_tekstovima("trazi-sloj", &[("naziv", naziv.clone())]);
            s.text(
                pos2(okvir.min.x + 8.0, okvir.center().y),
                egui::Align2::LEFT_CENTER,
                tekst,
                font_srednji(13.0),
                boje.tekst,
            );
        }
        RezultatPretrage::Objekt {
            naziv_objekta,
            naziv_sloja,
            ..
        } => {
            s.text(
                pos2(okvir.min.x + 8.0, okvir.center().y - 7.0),
                egui::Align2::LEFT_CENTER,
                naziv_objekta,
                font_obicni(13.0),
                boje.tekst,
            );
            let u_sloju = katalog.s_tekstovima("trazi-u-sloju", &[("sloj", naziv_sloja.clone())]);
            s.text(
                pos2(okvir.min.x + 8.0, okvir.center().y + 8.0),
                egui::Align2::LEFT_CENTER,
                u_sloju,
                font_obicni(10.0),
                boje.tekst_3,
            );
        }
    }
}

/// Crta privremenu oznaku pretražene koordinate na karti koristeći funkcije paketa `karta`.
pub fn crtaj_privremenu_oznaku(
    painter: &Painter,
    okvir: Rect,
    karta: &KartaProzora,
    tocka: [f64; 2],
    boje: &Boje,
) {
    if let Some(pos) = karta.tocka_u_ekran(Vrh::novi(tocka[0], tocka[1]), okvir) {
        painter.circle(pos, 8.0, boje.crvena, Stroke::new(2.0, boje.lebdeca));
        painter.circle_filled(pos, 3.0, boje.lebdeca);
    }
}
