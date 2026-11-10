//! Karta u sredini prozora: podloga, crtanje i mjerenje, traka mjerila, navod izvora i podaci
//! za statusnu traku. Alat, odabir i vidljivost dolaze iz ploče Slojevi.

use crate::ikone::SkladisteIkona;
use crate::poruke::{KatalogPoruka, broj_s_razmakom, mjera_duljine, mjera_povrsine};
use crate::sastavnice::{
    OpisGumbaIkone, cip, gumb_ikona, tekst_lijevo, tekst_sredina, zaobljeni_okvir,
};
use crate::stanje::{Alat, NazivNovog, Odabir, StanjePrikaza};
use crate::tema::{Boje, font_obicni, font_srednji};
use eframe::egui::{self, Pos2, Rect, Role, Sense, Stroke, WidgetInfo, pos2, vec2};
use karta::{Karta, ListDgu, Okolina, Podloga, Pogled, Poruka};
use koordinate::{Pretvornik, Sustav};
use slojevi::{Geometrija, IdObjekta, IdSloja, Mjere, Projekt, Uredjivac, VrstaGeometrije};
use std::collections::BTreeSet;

/// Sredina Hrvatske i zum na kojem se vidi cijela, dok projekt nema ništa.
const POCETAK: (f64, f64, f64) = (16.45, 44.6, 7.0);
/// Logičkih točaka po metru, uz uobičajenih 96 točaka po inču.
const TOCAKA_PO_METRU: f64 = 96.0 / 0.0254;
/// Najveća širina trake mjerila u točkama.
const NAJSIRA_TRAKA: f64 = 100.0;
const SIRINA_PLOCE_MJERENJA: f32 = 252.0;

/// Ono što statusna traka prikazuje o karti.
#[derive(Clone, Debug, PartialEq)]
pub struct StatusKarte {
    /// Istok i sjever u HTRS96/TM, pod mišem ili u sredini karte.
    pub koordinate: Option<[f64; 2]>,
    /// Nazivnik mjerila, npr. 4500 za 1 : 4 500.
    pub nazivnik: Option<f64>,
    pub podloga: Option<Podloga>,
}

pub struct KartaProzora {
    karta: Karta,
    /// Podloga se traži s mreže samo u pravom programu, a testovi rade bez nje.
    s_mreze: bool,
    postavljena: Option<Podloga>,
    /// Zadnji odabir koji su karta i ploča vidjele, da se zna tko ga je promijenio.
    zadnje_odabrano: Option<Odabir>,
    zadnji_na_karti: Vec<(IdSloja, IdObjekta)>,
    /// Alat koji je karta zadnji put dobila iz prozora, da se zna tko ga je promijenio.
    zadnji_alat: Alat,
    u_htrs: Option<Pretvornik>,
}

impl KartaProzora {
    #[must_use]
    pub fn nova(s_mreze: bool) -> Self {
        let mut karta = Karta::bez_podloge(POCETAK.0, POCETAK.1);
        karta.idi_na(POCETAK.0, POCETAK.1, POCETAK.2);
        // Ploča pokriva desni dio karte, pa navod crta prozor, lijevo od ploče.
        karta.crtaj_navod(false);
        Self {
            karta,
            s_mreze,
            postavljena: None,
            zadnje_odabrano: None,
            zadnji_na_karti: Vec::new(),
            zadnji_alat: Alat::Odaberi,
            u_htrs: Pretvornik::novi(Sustav::Wgs84, Sustav::Htrs96Tm).ok(),
        }
    }

    pub fn karta(&mut self) -> &mut Karta {
        &mut self.karta
    }

    /// Crta li se ili mjeri upravo nešto. Tada Backspace pripada karti, a ne ploči.
    pub fn u_tijeku(&mut self) -> bool {
        self.karta.crtanje().u_tijeku()
    }

    /// Koliko je objekata odabrano na karti.
    pub fn broj_odabranih(&mut self) -> usize {
        self.karta.crtanje().odabrani().len()
    }

    /// Briše sve objekte odabrane na karti, u jednom koraku poništavanja.
    pub fn obrisi_odabrane(&mut self, u: &mut Uredjivac) -> bool {
        self.karta.crtanje().obrisi_odabrane(u)
    }

    /// Novi ili otvoreni projekt. Karta zaboravlja stari projekt i prikazuje sve objekte novoga.
    pub fn novi_projekt(&mut self, projekt: &Projekt) {
        self.karta.novi_projekt();
        self.zadnje_odabrano = None;
        self.zadnji_na_karti.clear();
        if let Some((duzina, sirina, zum)) = obuhvat(projekt) {
            self.karta.idi_na(duzina, sirina, zum);
        }
    }

    pub fn priblizi(&mut self) {
        // Na najvećem zumu walkers odbija daljnje, a to nije greška za korisnika.
        let _ = self.karta.memorija().zoom_in();
    }

    pub fn udalji(&mut self) {
        let _ = self.karta.memorija().zoom_out();
    }

    #[must_use]
    pub fn pogled(&self) -> Option<Pogled> {
        self.karta.pogled()
    }

    #[must_use]
    pub fn tocka_u_ekran(&self, tocka: slojevi::Vrh, okvir: Rect) -> Option<Pos2> {
        self.karta.tocka_u_ekran(tocka, okvir)
    }

    #[must_use]
    pub fn status(&self) -> Option<StatusKarte> {
        let pogled = self.karta.pogled()?;
        let mjesto = pogled.mis.unwrap_or(pogled.sredina);
        let koordinate = self
            .u_htrs
            .as_ref()
            .and_then(|p| p.pretvori([mjesto.duzina, mjesto.sirina]).ok());
        Some(StatusKarte {
            koordinate,
            nazivnik: nazivnik_mjerila(pogled.metara_po_tocki),
            podloga: self.postavljena,
        })
    }

    /// Prikazuje kartu u okviru i usklađuje je s pločom Slojevi.
    pub fn prikazi(
        &mut self,
        ui: &mut egui::Ui,
        okvir: Rect,
        u: &mut Uredjivac,
        stanje: &mut StanjePrikaza,
        katalog: &KatalogPoruka,
    ) {
        self.uskladi_podlogu(ui.ctx(), stanje);
        self.uskladi_alat(stanje);
        self.odabir_iz_ploce(u.projekt(), stanje);
        let vidljivi = vidljivi_slojevi(u.projekt(), stanje);
        let mut dijete = ui.new_child(egui::UiBuilder::new().max_rect(okvir));
        dijete.set_clip_rect(okvir);
        s_okolinom(katalog, &vidljivi, |ok| {
            self.karta.prikazi(&mut dijete, u, ok);
        });
        self.odabir_s_karte(stanje);
        self.alat_s_karte(stanje);
        if let Some((sloj, objekt)) = self.karta.crtanje().uzmi_novi()
            && let Some(o) = u.projekt().objekt(sloj, objekt)
        {
            stanje.naziv_novog = Some(NazivNovog::novi(sloj, objekt, &o.naziv));
            ui.ctx().request_repaint();
        }
    }

    fn uskladi_podlogu(&mut self, ctx: &egui::Context, stanje: &StanjePrikaza) {
        let zeljena = zeljena_podloga(stanje);
        if self.s_mreze && self.postavljena != Some(zeljena) {
            self.karta.postavi_podlogu(zeljena, ctx.clone());
            self.postavljena = Some(zeljena);
        }
    }

    /// Alat odabran u prozoru ide na kartu samo kad ga je korisnik promijenio. Inače bi karta
    /// izgubila produljenje linije, koje sama prebacuje na alat Linija.
    fn uskladi_alat(&mut self, stanje: &StanjePrikaza) {
        if stanje.alat == self.zadnji_alat {
            return;
        }
        let zeljeni = alat_karte(stanje.alat);
        if self.karta.crtanje().alat() != zeljeni {
            self.karta.crtanje().postavi_alat(zeljeni);
        }
        self.zadnji_alat = stanje.alat;
    }

    /// Karta nakon nacrtanog objekta sama prelazi na Odaberi, a gumbi alata to moraju pokazati.
    fn alat_s_karte(&mut self, stanje: &mut StanjePrikaza) {
        stanje.alat = alat_prozora(self.karta.crtanje().alat());
        self.zadnji_alat = stanje.alat;
    }

    /// Odabrani sloj prima nove objekte svoje vrste, a odabrani objekt karta ističe.
    fn odabir_iz_ploce(&mut self, projekt: &Projekt, stanje: &mut StanjePrikaza) {
        let sloj = match stanje.odabrano {
            Some(Odabir::Sloj(s) | Odabir::Objekt(s, _)) => Some(s),
            _ => None,
        };
        if let Some((s, vrsta)) = sloj.and_then(|s| projekt.sloj(s).map(|sl| (s, sl.vrsta))) {
            self.karta.crtanje().postavi_aktivni_sloj(vrsta, s);
        }
        if stanje.odabrano == self.zadnje_odabrano {
            return;
        }
        // Odabir u ploči zamjenjuje odabir na karti, pa i kad je odabran sloj ili mapa.
        let objekt = match stanje.odabrano {
            Some(Odabir::Objekt(s, o)) => Some((s, o)),
            _ => None,
        };
        self.karta.crtanje().postavi_odabrani(objekt);
        self.zadnje_odabrano = stanje.odabrano;
        self.zadnji_na_karti = objekt.into_iter().collect();
        stanje.odabrani_na_karti.clone_from(&self.zadnji_na_karti);
    }

    /// Jedan objekt odabran na karti odabire se i u ploči. Više njih ploča samo ističe.
    fn odabir_s_karte(&mut self, stanje: &mut StanjePrikaza) {
        let na_karti = self.karta.crtanje().odabrani();
        if na_karti == self.zadnji_na_karti.as_slice() {
            return;
        }
        self.zadnji_na_karti = na_karti.to_vec();
        stanje.odabrano = match na_karti {
            [(s, o)] => Some(Odabir::Objekt(*s, *o)),
            _ => None,
        };
        stanje.odabrani_na_karti.clone_from(&self.zadnji_na_karti);
        self.zadnje_odabrano = stanje.odabrano;
    }

    /// Ploča mjerenja po zaslonu D, desno od trake alata.
    #[allow(clippy::too_many_arguments)]
    pub fn crtaj_plocu_mjerenja(
        &mut self,
        ui: &mut egui::Ui,
        min: Pos2,
        u: &mut Uredjivac,
        stanje: &mut StanjePrikaza,
        katalog: &KatalogPoruka,
        ikone: &mut SkladisteIkona,
        boje: &Boje,
    ) {
        if stanje.alat != Alat::Mjeri {
            return;
        }
        let mjerenje = self.karta.crtanje().mjerenje();
        let visina = if mjerenje.is_some() { 126.0 } else { 78.0 };
        let okvir = Rect::from_min_size(min, vec2(SIRINA_PLOCE_MJERENJA, visina));
        zaklon(ui, okvir, "ploca_mjerenja");
        zaobljeni_okvir(ui, okvir, boje.lebdeca, Some(boje.obrub), 12.0);
        self.crtaj_zaglavlje_mjerenja(ui, min, stanje, katalog, ikone, boje);
        match mjerenje {
            Some(m) => self.crtaj_rezultat_mjerenja(ui, min, &m, u, stanje, katalog, boje),
            None => tekst_lijevo(
                ui,
                min + vec2(12.0, 56.0),
                &katalog.tekst("mjerenje-pocetak"),
                font_obicni(12.0),
                boje.tekst_2,
            ),
        }
    }

    /// Izbor Udaljenost ili Površina i gumb za zatvaranje.
    fn crtaj_zaglavlje_mjerenja(
        &mut self,
        ui: &mut egui::Ui,
        min: Pos2,
        stanje: &mut StanjePrikaza,
        katalog: &KatalogPoruka,
        ikone: &mut SkladisteIkona,
        boje: &Boje,
    ) {
        let tamna = stanje.tema_tamna;
        let povrsina = self.karta.crtanje().mjerenje_povrsine();
        let udaljenost = katalog.tekst("mjerenje-udaljenost");
        let okvir_u = Rect::from_min_size(min + vec2(10.0, 10.0), vec2(108.0, 26.0));
        if cip(ui, ikone, okvir_u, &udaljenost, !povrsina, tamna, boje).clicked() {
            self.karta.crtanje().mjeri_povrsinu(false);
        }
        let tekst_p = katalog.tekst("mjerenje-povrsina");
        let okvir_p = Rect::from_min_size(min + vec2(124.0, 10.0), vec2(82.0, 26.0));
        if cip(ui, ikone, okvir_p, &tekst_p, povrsina, tamna, boje).clicked() {
            self.karta.crtanje().mjeri_povrsinu(true);
        }
        let zatvori = katalog.tekst("mjerenje-zatvori");
        let opis_x = OpisGumbaIkone {
            okvir: Rect::from_min_size(
                min + vec2(SIRINA_PLOCE_MJERENJA - 34.0, 11.0),
                vec2(24.0, 24.0),
            ),
            ikona: "x",
            stanje: "obicno",
            crtez: 20,
            obrubljen: false,
            aktivno: false,
            labela: &zatvori,
        };
        if gumb_ikona(ui, ikone, &opis_x, tamna, boje).clicked() {
            self.karta.crtanje().ocisti_mjerenje();
            stanje.alat = Alat::Odaberi;
        }
    }

    /// Mjera, broj dijelova i gumbi Spremi i Očisti.
    #[allow(clippy::too_many_arguments)]
    fn crtaj_rezultat_mjerenja(
        &mut self,
        ui: &mut egui::Ui,
        min: Pos2,
        m: &karta::Mjerenje,
        u: &mut Uredjivac,
        stanje: &StanjePrikaza,
        katalog: &KatalogPoruka,
        boje: &Boje,
    ) {
        tekst_lijevo(
            ui,
            min + vec2(12.0, 62.0),
            &opis_mjere(katalog, &m.mjere),
            font_srednji(20.0),
            boje.tekst,
        );
        tekst_lijevo(
            ui,
            min + vec2(12.0, 86.0),
            &katalog.s_brojem("mjerenje-dijelova", m.dijelova),
            font_obicni(11.0),
            boje.tekst_2,
        );
        let spremi = if matches!(m.mjere, Mjere::Poligon(_)) {
            katalog.tekst("mjerenje-spremi-poligon")
        } else {
            katalog.tekst("mjerenje-spremi-liniju")
        };
        let okvir_s = Rect::from_min_size(min + vec2(10.0, 98.0), vec2(150.0, 24.0));
        if gumb_teksta(ui, okvir_s, &spremi, &spremi, boje).clicked() {
            let vidljivi = vidljivi_slojevi(u.projekt(), stanje);
            let crtanje = self.karta.crtanje();
            s_okolinom(katalog, &vidljivi, |ok| crtanje.spremi_mjerenje(u, ok));
        }
        let ocisti = katalog.tekst("mjerenje-ocisti");
        let okvir_o = Rect::from_min_size(
            min + vec2(SIRINA_PLOCE_MJERENJA - 74.0, 98.0),
            vec2(64.0, 24.0),
        );
        if gumb_teksta(ui, okvir_o, &ocisti, &ocisti, boje).clicked() {
            self.karta.crtanje().ocisti_mjerenje();
        }
    }

    /// Kratka poruka iznad donjeg ruba karte: greška crtanja, npr. premalo točaka, ili upozorenje da
    /// novi objekt ne ide u odabrani sloj.
    pub fn crtaj_poruku(
        &mut self,
        ui: &egui::Ui,
        dno: Pos2,
        projekt: &Projekt,
        stanje: &StanjePrikaza,
        katalog: &KatalogPoruka,
        boje: &Boje,
    ) {
        let crtanje = self.karta.crtanje();
        let presjek = crtanje
            .presjek(projekt)
            .map(|_| katalog.tekst("poruka-presjek"));
        let Some(tekst) = crtanje
            .poruka()
            .map(|poruka| tekst_poruke(katalog, poruka))
            .or(presjek)
            .or_else(|| poruka_drugog_sloja(projekt, stanje, katalog))
        else {
            return;
        };
        let sirina = tekst.chars().count() as f32 * 6.6 + 28.0;
        let okvir = Rect::from_center_size(dno - vec2(0.0, 20.0), vec2(sirina, 30.0));
        zaobljeni_okvir(ui, okvir, boje.lebdeca, Some(boje.obrub), 10.0);
        tekst_sredina(ui, okvir.center(), &tekst, font_obicni(12.0), boje.tekst);
    }

    /// Traka mjerila u donjem lijevom kutu karte, prema stvarnom zumu.
    pub fn crtaj_traku_mjerila(&self, ui: &egui::Ui, mjesto: Pos2, boje: &Boje) {
        let Some((duljina, sirina)) = self.karta.pogled().and_then(traka_mjerila) else {
            return;
        };
        let y = mjesto.y + 18.0;
        let potez = Stroke::new(1.5, boje.tekst);
        let (x0, x1) = (mjesto.x, mjesto.x + sirina);
        ui.painter().line_segment([pos2(x0, y), pos2(x1, y)], potez);
        ui.painter()
            .line_segment([pos2(x0, y - 5.0), pos2(x0, y)], potez);
        ui.painter()
            .line_segment([pos2(x1, y - 5.0), pos2(x1, y)], potez);
        tekst_lijevo(
            ui,
            pos2(x0, y - 9.0),
            &oznaka_mjerila(duljina),
            font_srednji(11.0),
            boje.tekst,
        );
    }

    /// Navod izvora podloge, desno dolje, lijevo od ploče.
    pub fn crtaj_navod(&self, ui: &egui::Ui, desno_dolje: Pos2, boje: &Boje) {
        let Some(p) = self.postavljena else {
            return;
        };
        let galija =
            ui.painter()
                .layout_no_wrap(p.navod().to_owned(), font_obicni(10.0), boje.tekst_2);
        let velicina = galija.size();
        let okvir = Rect::from_min_size(
            desno_dolje - velicina - vec2(8.0, 4.0),
            velicina + vec2(8.0, 4.0),
        );
        ui.painter()
            .rect_filled(okvir, 4.0, boje.lebdeca.gamma_multiply(0.85));
        ui.painter()
            .galley(okvir.min + vec2(4.0, 2.0), galija, boje.tekst_2);
    }
}

/// Površina koja pokriva kartu, da klik na ploču ili gumb ne padne na kartu ispod.
/// Mora biti prije gumba u istoj ploči, jer kasnije dodano u `egui`-ju leži iznad.
pub fn zaklon(ui: &egui::Ui, okvir: Rect, ime: &str) {
    let id = ui.make_persistent_id(("zaklon", ime));
    let _ = ui.interact(okvir, id, Sense::click_and_drag());
}

fn gumb_teksta(ui: &egui::Ui, okvir: Rect, tekst: &str, opis: &str, boje: &Boje) -> egui::Response {
    let id = ui.make_persistent_id(("gumb_teksta", opis));
    let odgovor = ui.interact(okvir, id, Sense::click());
    if odgovor.hovered() {
        zaobljeni_okvir(ui, okvir, boje.ploca_2, None, 6.0);
    }
    tekst_sredina(
        ui,
        okvir.center(),
        tekst,
        font_srednji(12.0),
        boje.primarna_tamna,
    );
    odgovor.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), opis));
    odgovor
}

fn s_okolinom<R>(
    katalog: &KatalogPoruka,
    vidljivi: &BTreeSet<IdSloja>,
    posao: impl FnOnce(&Okolina<'_>) -> R,
) -> R {
    let vidljiv = |s: IdSloja| vidljivi.contains(&s);
    let naziv = |v: VrstaGeometrije| katalog.tekst(kljuc_novog_sloja(v));
    let opis = |m: &Mjere| opis_mjere(katalog, m);
    let duljina = |d: f64| mjera_duljine(d);
    let naziv_objekta =
        |v: VrstaGeometrije, redni: usize| katalog.s_rednim(kljuc_novog_objekta(v), redni);
    posao(&Okolina {
        vidljiv: &vidljiv,
        naziv_sloja: &naziv,
        opis_mjere: &opis,
        opis_duljine: &duljina,
        naziv_objekta: &naziv_objekta,
    })
}

/// Računa sredinu i zum iz niza vrhova.
#[must_use]
pub fn obuhvat_vrhova<'a>(
    vrhovi: impl IntoIterator<Item = &'a slojevi::Vrh>,
) -> Option<(f64, f64, f64)> {
    let mut granice: Option<[f64; 4]> = None;
    for v in vrhovi {
        let g = granice.get_or_insert([v.duzina, v.sirina, v.duzina, v.sirina]);
        *g = [
            g[0].min(v.duzina),
            g[1].min(v.sirina),
            g[2].max(v.duzina),
            g[3].max(v.sirina),
        ];
    }
    let [z, j, i, s] = granice?;
    let raspon = (i - z).max((s - j) * 1.5);
    let zum = if raspon > 0.0 {
        (1200.0 / raspon).log2().floor().clamp(3.0, 17.0)
    } else {
        16.0
    };
    Some(((z + i) / 2.0, (j + s) / 2.0, zum))
}

/// Sredina i zum na kojem se vide svi objekti projekta. Zum je približan: karta široka oko tisuću
/// točaka, s malo ruba. Bez objekata nema obuhvata.
#[must_use]
pub fn obuhvat(projekt: &Projekt) -> Option<(f64, f64, f64)> {
    let vrhovi = projekt
        .slojevi()
        .flat_map(|(_, s)| s.objekti.values())
        .flat_map(|o| match &o.geometrija {
            Geometrija::Tocka(v) => vec![v],
            Geometrija::Linija(v) => v.iter().collect(),
            Geometrija::Poligon(p) => p.iter().flatten().collect(),
        });
    obuhvat_vrhova(vrhovi)
}

/// Sloj se crta kad su uključeni i on i sve mape iznad njega.
#[must_use]
pub fn vidljivi_slojevi(projekt: &Projekt, stanje: &StanjePrikaza) -> BTreeSet<IdSloja> {
    projekt
        .slojevi()
        .filter(|(id, sloj)| {
            let mut mapa = sloj.roditelj;
            let mut vidljiv = stanje.je_sloj_vidljiv(*id);
            while let (true, Some(m)) = (vidljiv, mapa) {
                vidljiv = stanje.je_mapa_vidljiva(m);
                mapa = projekt.mapa(m).and_then(|m| m.roditelj);
            }
            vidljiv
        })
        .map(|(id, _)| id)
        .collect()
}

#[must_use]
pub fn zeljena_podloga(stanje: &StanjePrikaza) -> Podloga {
    match stanje.podloga {
        1 => Podloga::Dgu(ListDgu::Dof),
        _ => Podloga::OpenFreeMap {
            tamna: stanje.tema_tamna,
        },
    }
}

#[must_use]
const fn alat_prozora(alat: karta::Alat) -> Alat {
    match alat {
        karta::Alat::Odaberi => Alat::Odaberi,
        karta::Alat::Tocka => Alat::Tocka,
        karta::Alat::Linija => Alat::Linija,
        karta::Alat::Poligon => Alat::Poligon,
        karta::Alat::Mjeri => Alat::Mjeri,
    }
}

pub fn alat_karte(alat: Alat) -> karta::Alat {
    match alat {
        Alat::Odaberi => karta::Alat::Odaberi,
        Alat::Tocka => karta::Alat::Tocka,
        Alat::Linija => karta::Alat::Linija,
        Alat::Poligon => karta::Alat::Poligon,
        Alat::Mjeri => karta::Alat::Mjeri,
    }
}

const fn kljuc_novog_sloja(vrsta: VrstaGeometrije) -> &'static str {
    match vrsta {
        VrstaGeometrije::Tocka => "sloj-tocke",
        VrstaGeometrije::Linija => "sloj-linije",
        VrstaGeometrije::Poligon => "sloj-poligoni",
    }
}

const fn kljuc_novog_objekta(vrsta: VrstaGeometrije) -> &'static str {
    match vrsta {
        VrstaGeometrije::Tocka => "objekt-tocka",
        VrstaGeometrije::Linija => "objekt-linija",
        VrstaGeometrije::Poligon => "objekt-poligon",
    }
}

#[must_use]
pub fn opis_mjere(katalog: &KatalogPoruka, m: &Mjere) -> String {
    match m {
        Mjere::Tocka => String::new(),
        Mjere::Linija(c) => mjera_duljine(c.duljina_m),
        Mjere::Poligon(l) => katalog.s_tekstovima(
            "mjera-lika",
            &[
                ("povrsina", mjera_povrsine(l.povrsina_m2)),
                ("opseg", mjera_duljine(l.opseg_m)),
            ],
        ),
    }
}

/// Kad je u ploči odabran sloj druge vrste od alata, novi objekt ide u drugi sloj, pa to treba reći.
#[must_use]
pub fn poruka_drugog_sloja(
    projekt: &Projekt,
    stanje: &StanjePrikaza,
    katalog: &KatalogPoruka,
) -> Option<String> {
    let vrsta_alata = stanje.alat.vrsta()?;
    let sloj = match stanje.odabrano? {
        Odabir::Sloj(s) | Odabir::Objekt(s, _) => projekt.sloj(s)?,
        Odabir::Mapa(_) => return None,
    };
    if sloj.vrsta == vrsta_alata {
        return None;
    }
    let vrsta = |v| katalog.tekst(kljuc_vrste_sloja(v));
    Some(katalog.s_tekstovima(
        "poruka-drugi-sloj",
        &[
            ("sloj", sloj.naziv.clone()),
            ("vrsta_sloja", vrsta(sloj.vrsta)),
            ("vrsta_alata", vrsta(vrsta_alata)),
        ],
    ))
}

const fn kljuc_vrste_sloja(vrsta: VrstaGeometrije) -> &'static str {
    match vrsta {
        VrstaGeometrije::Tocka => "vrsta-sloja-tocka",
        VrstaGeometrije::Linija => "vrsta-sloja-linija",
        VrstaGeometrije::Poligon => "vrsta-sloja-poligon",
    }
}

#[must_use]
pub fn tekst_poruke(katalog: &KatalogPoruka, poruka: Poruka) -> String {
    match poruka {
        Poruka::Premalo { najmanje } => katalog.s_brojem("poruka-premalo", najmanje),
        Poruka::NeMozeBrisati { .. } => katalog.tekst("poruka-ne-moze-brisati"),
        Poruka::Naredba(g) => katalog.s_tekstovima("poruka-naredba", &[("razlog", g.to_string())]),
    }
}

/// Nazivnik mjerila zaokružen na dvije znamenke, kao 1 : 4 500.
#[must_use]
pub fn nazivnik_mjerila(metara_po_tocki: f64) -> Option<f64> {
    let n = metara_po_tocki * TOCAKA_PO_METRU;
    if !n.is_finite() || n < 1.0 {
        return None;
    }
    let korak = 10_f64.powi(n.log10().floor() as i32 - 1).max(1.0);
    Some((n / korak).round() * korak)
}

#[must_use]
pub fn tekst_mjerila(katalog: &KatalogPoruka, nazivnik: f64) -> String {
    katalog.s_tekstovima(
        "status-mjerilo",
        &[("nazivnik", broj_s_razmakom(nazivnik, 0))],
    )
}

/// Najdulja okrugla duljina (1, 2 ili 5 puta potencija desetke) koja stane u traku. Vraća metre i
/// širinu u točkama.
#[must_use]
pub fn traka_mjerila(pogled: Pogled) -> Option<(f64, f32)> {
    let m = pogled.metara_po_tocki;
    if !m.is_finite() || m <= 0.0 {
        return None;
    }
    let najvise = m * NAJSIRA_TRAKA;
    let potencija = 10_f64.powi(najvise.log10().floor() as i32);
    let duljina = [5.0, 2.0, 1.0]
        .into_iter()
        .map(|k| k * potencija)
        .find(|d| *d <= najvise)?;
    Some((duljina, (duljina / m) as f32))
}

/// Okrugla duljina bez decimala: 500 m, 2 km.
#[must_use]
pub fn oznaka_mjerila(metri: f64) -> String {
    if metri >= 1000.0 {
        format!("{} km", broj_s_razmakom(metri / 1000.0, 0))
    } else if metri >= 1.0 {
        format!("{} m", broj_s_razmakom(metri, 0))
    } else {
        format!("{} cm", broj_s_razmakom(metri * 100.0, 0))
    }
}

#[cfg(test)]
#[path = "karta_prozora_testovi.rs"]
mod testovi;
