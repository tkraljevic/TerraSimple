use crate::datoteka::{
    BiracDatoteka, GreskaSpremanja, PodaciDatoteke, SustavskiBirac, radnja_novi, radnja_otvori,
    radnja_spremi, radnja_spremi_kao, tekst_statusa_datoteke,
};
use crate::dijalozi::{
    AktivniDijalog, IshodDijaloga, NamjeraPotvrde, StavkaIzbornikaDatoteka, crtaj_aktivni_dijalog,
    crtaj_padajuci_izbornik_datoteka,
};
use crate::ikone::{SkladisteIkona, okvir_ikone_u_sredini};
use crate::karta_prozora::{KartaProzora, StatusKarte, tekst_mjerila, zaklon};
use crate::naziv_objekta::crtaj_prozorcic_naziva;
use crate::novi_sloj::crtaj_prozorcic_novog_sloja;
use crate::ploca::{
    crtaj_plocu_slojeva, obrisi_odabrano, pokreni_preimenovanje_odabranog, potvrdi_preimenovanje,
};
use crate::poruke::KatalogPoruka;
use crate::primjer::izgradi_primjer_zaslona_a;
use crate::sastavnice::{
    OpisGumbaAlata, OpisGumbaIkone, gumb_alat, gumb_ikona, tekst_desno, tekst_lijevo,
    tekst_sredina, zaobljeni_okvir,
};
use crate::stanje::{Alat, Odabir, StanjePrikaza};
use crate::tema::{Boje, Razmaci, font_obicni, font_srednji, fontovi, primijeni_stil};
use crate::trazilica::{self, StanjeTrazilice};
use eframe::egui::{self, Event, Key, Pos2, Rect, Role, Sense, Stroke, WidgetInfo, pos2, vec2};
use slojevi::{NaredbaStabla, Stavka, Uredjivac};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PostavkeAplikacije {
    pub primjer: bool,
    pub tamna: bool,
    pub ploca_lijevo: bool,
    pub snimka: Option<PathBuf>,
    /// Podloga s mreže. Pravi program je uključuje, a testovi rade bez mreže.
    pub podloga_s_mreze: bool,
    pub dijalog: Option<String>,
    /// Mapa za automatsko spremanje u kopiju. Bez nje kopije nema, npr. u testovima.
    pub oporavak: Option<PathBuf>,
}

#[must_use]
pub fn rasclani_argumente<I, S>(argumenti: I) -> PostavkeAplikacije
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut postavke = PostavkeAplikacije::default();
    let mut iter = argumenti.into_iter();
    while let Some(arg) = iter.next() {
        obradi_jedan_argument(&mut postavke, arg.as_ref(), &mut iter);
    }
    postavke
}

fn obradi_jedan_argument<S: AsRef<str>>(
    postavke: &mut PostavkeAplikacije,
    arg: &str,
    iter: &mut impl Iterator<Item = S>,
) {
    match arg {
        "--primjer" => postavke.primjer = true,
        "--tamna" => postavke.tamna = true,
        "--lijevo" => postavke.ploca_lijevo = true,
        "--snimka" => postavke.snimka = iter.next().map(|p| PathBuf::from(p.as_ref())),
        "--dijalog" => postavke.dijalog = iter.next().map(|n| n.as_ref().to_string()),
        _ => {}
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StavkeStatusneTrake {
    pub sustav: String,
    pub koordinate: Option<String>,
    pub mjerilo: Option<String>,
    pub izvor_podloge: Option<String>,
    pub mreza: Option<String>,
}

impl StavkeStatusneTrake {
    #[must_use]
    pub fn sve_vrijednosti(&self) -> Vec<&str> {
        let mut v = vec![self.sustav.as_str()];
        if let Some(k) = &self.koordinate {
            v.push(k.as_str());
        }
        if let Some(m) = &self.mjerilo {
            v.push(m.as_str());
        }
        if let Some(i) = &self.izvor_podloge {
            v.push(i.as_str());
        }
        if let Some(mreza) = &self.mreza {
            v.push(mreza.as_str());
        }
        v
    }
}

#[must_use]
pub fn odredi_stavke_statusne_trake(
    stanje: &StanjePrikaza,
    katalog: &KatalogPoruka,
    karta: Option<&StatusKarte>,
) -> StavkeStatusneTrake {
    let sustav = katalog.tekst("status-sustav");
    if let Some(k) = karta {
        StavkeStatusneTrake {
            sustav,
            koordinate: k
                .koordinate
                .map(|[istok, sjever]| katalog.status_koordinate(istok, sjever, 0)),
            mjerilo: k.nazivnik.map(|n| tekst_mjerila(katalog, n)),
            izvor_podloge: k.podloga.map(|p| match p {
                karta::Podloga::OpenFreeMap { .. } => katalog.tekst("status-izvor"),
                karta::Podloga::Dgu(_) => katalog.tekst("status-izvor-dgu"),
            }),
            // Stanje mreže program još ne prati, pa ga ne izmišlja.
            mreza: None,
        }
    } else if stanje.prikazi_primjer {
        StavkeStatusneTrake {
            sustav,
            koordinate: Some(katalog.status_koordinate(655_120.0, 5_046_380.0, 0)),
            mjerilo: Some(katalog.tekst("status-primjer-mjerilo")),
            izvor_podloge: Some(katalog.tekst("status-izvor")),
            mreza: Some(katalog.tekst("status-mreza")),
        }
    } else {
        StavkeStatusneTrake {
            sustav,
            koordinate: None,
            mjerilo: None,
            izvor_podloge: None,
            mreza: None,
        }
    }
}

#[must_use]
pub fn naziv_datoteke_status(stanje: &StanjePrikaza, katalog: &KatalogPoruka) -> Option<String> {
    if stanje.prikazi_primjer {
        Some(katalog.tekst("primjer-datoteka-status"))
    } else {
        None
    }
}

/// Okvir ploče Slojevi, isti kao u `crtaj_plocu_slojeva`.
fn okvir_ploce(stanje: &StanjePrikaza, okvir_karte: Rect) -> Option<Rect> {
    if !stanje.ploca_otvorena {
        return None;
    }
    let x = if stanje.ploca_lijevo {
        okvir_karte.min.x + Razmaci::RUB
    } else {
        okvir_karte.max.x - Razmaci::SIRINA_PLOCE - Razmaci::RUB
    };
    Some(Rect::from_min_size(
        pos2(x, okvir_karte.min.y + Razmaci::RUB),
        vec2(Razmaci::SIRINA_PLOCE, okvir_karte.height() - 24.0),
    ))
}

/// Kotačić iznad ploče pomiče ploču, a ne kartu: karta ga ispod ploče ne smije dobiti.
fn preuzmi_klizanje(ui: &egui::Ui, okvir: Option<Rect>) -> f32 {
    let pokazivac = ui.input(|i| i.pointer.hover_pos());
    let nad_plocom = matches!((okvir, pokazivac), (Some(o), Some(p)) if o.contains(p));
    if !nad_plocom {
        return 0.0;
    }
    ui.input_mut(|i| std::mem::take(&mut i.smooth_scroll_delta).y)
}

pub struct Aplikacija {
    pub uredjivac: Uredjivac,
    pub stanje: StanjePrikaza,
    pub karta: KartaProzora,
    pub ikone: SkladisteIkona,
    pub katalog: KatalogPoruka,
    pub datoteka: PodaciDatoteke,
    pub birac: Box<dyn BiracDatoteka>,
    desni_klik_maca: crate::unos::DesniKlikMaca,
    pub qgis: Box<dyn crate::qgis_veza::PokretacQgisa>,
    pub aktivni_dijalog: Option<AktivniDijalog>,
    /// Korisnik je već odgovorio na pitanje pri zatvaranju, pa se prozor zatvara bez novog pitanja.
    pub zatvaranje_potvrdjeno: bool,
    pub izbornik_datoteka_otvoren: bool,
    pub trazilica: StanjeTrazilice,
    pub snimka: Option<PathBuf>,
    pub okvir_brojac: u32,
    /// Snimka čeka da stignu pločice podloge. Bez mreže nema što čekati.
    pub cekanje_snimke: Duration,
    pocetak: Instant,
    pub zadnje_vrijeme_okvira: Duration,
    pub pocetak_rada: Instant,
    pub oporavak: Option<crate::oporavak::Oporavak>,
}

impl Default for Aplikacija {
    fn default() -> Self {
        Self::nova(PostavkeAplikacije::default())
    }
}

/// Prozorčić koji se otvori sam, za snimke i pregled izgleda (`--dijalog <naziv>`).
/// Stavke gornjeg izbornika koje nešto rade; ostale su zasad samo natpis.
const STAVKE_S_RADNJOM: [&str; 3] = ["izbornik-datoteka", "izbornik-slojevi", "izbornik-pomoc"];

/// Popis upozorenja nakon otvaranja otvara se i zatvara istim gumbom.
fn preklopi_popis(dijalog: &mut AktivniDijalog) {
    if let AktivniDijalog::UpozorenjaOtvaranja { otvoren_popis, .. } = dijalog {
        *otvoren_popis = !*otvoren_popis;
    }
}

pub(crate) fn dijalog_za_pregled(
    naziv: Option<&str>,
    katalog: &KatalogPoruka,
) -> Option<AktivniDijalog> {
    match naziv {
        Some("potvrda") => Some(AktivniDijalog::PotvrdaNespremljeno(NamjeraPotvrde::Novi)),
        Some("greska") => Some(AktivniDijalog::PorukaGreske {
            naslov: katalog.tekst("greska-spremanja-naslov"),
            poruka: katalog.greska_spremanja(&std::io::Error::other("disk je pun")),
        }),
        Some("sukob") => Some(AktivniDijalog::SukobSpremanja),
        Some("o-programu") => Some(AktivniDijalog::OProgramu),
        Some("upozorenje") => Some(AktivniDijalog::UpozorenjeBrisanja {
            stavka: slojevi::Stavka::Sloj(slojevi::IdSloja(1)),
            naziv: "Građevine".to_string(),
            broj_nepodrzanih: 5,
        }),
        Some("upozorenja-otvaranja") => Some(AktivniDijalog::UpozorenjaOtvaranja {
            upozorenja: vec![
                "Sloj „Katastar” sadrži krivulje koje nisu podržane".to_string(),
                "Sloj „Točke” sadrži nepodržane M koordinate".to_string(),
                "Sloj „Granice” sadrži nepoznati SRID 9999".to_string(),
            ],
            otvoren_popis: false,
        }),
        _ => None,
    }
}

impl Aplikacija {
    #[must_use]
    pub fn nova(postavke: PostavkeAplikacije) -> Self {
        let mut uredjivac = Uredjivac::novi();
        let mut stanje = StanjePrikaza::novo();
        let katalog = KatalogPoruka::hrvatski();
        stanje.tema_tamna = postavke.tamna;
        stanje.ploca_lijevo = postavke.ploca_lijevo;
        let mut karta = KartaProzora::nova(postavke.podloga_s_mreze);
        if postavke.primjer {
            izgradi_primjer_zaslona_a(&mut uredjivac, &mut stanje, &katalog);
            // Izmišljeni likovi primjera su kod 16,0° E i 45,8° N.
            karta.karta().idi_na(16.004, 45.802, 16.5);
        }
        let izvoz = || {
            let opseg = formati::OpsegIzvoza::Projekt;
            AktivniDijalog::Izvoz(crate::izvoz::pripremi(
                uredjivac.projekt(),
                opseg,
                None,
                &katalog,
            ))
        };
        let oporavak = postavke
            .oporavak
            .and_then(|m| crate::oporavak::Oporavak::novi(m).ok());
        let ostatak = oporavak
            .as_ref()
            .and_then(|o| o.ostaci().into_iter().next());
        let aktivni_dijalog = dijalog_za_pregled(postavke.dijalog.as_deref(), &katalog)
            .or_else(|| (postavke.dijalog.as_deref() == Some("izvoz")).then(izvoz))
            .or_else(|| ostatak.map(AktivniDijalog::Oporavak));
        Self {
            uredjivac,
            stanje,
            karta,
            ikone: SkladisteIkona::novo(),
            katalog,
            datoteka: PodaciDatoteke::nova(),
            birac: Box::new(SustavskiBirac),
            desni_klik_maca: crate::unos::DesniKlikMaca::default(),
            qgis: Box::new(crate::qgis_veza::SustavskiQgis),
            aktivni_dijalog,
            zatvaranje_potvrdjeno: false,
            izbornik_datoteka_otvoren: false,
            trazilica: StanjeTrazilice::default(),
            snimka: postavke.snimka,
            okvir_brojac: 0,
            cekanje_snimke: if postavke.podloga_s_mreze {
                Duration::from_secs(8)
            } else {
                Duration::ZERO
            },
            pocetak: Instant::now(),
            zadnje_vrijeme_okvira: Duration::ZERO,
            pocetak_rada: Instant::now(),
            oporavak,
        }
    }

    /// Kopija nespremljenog rada. Dok ima izmjena, okvir se crta barem svakih 5 s, da kopija
    /// nastane i kad korisnik ništa ne dira.
    fn automatsko_spremanje(&mut self, ctx: &egui::Context) {
        let Some(o) = self.oporavak.as_mut() else {
            return;
        };
        o.tik(
            &self.uredjivac,
            self.datoteka.staza.as_deref(),
            Instant::now(),
        );
        if self.uredjivac.je_li_izmijenjen() {
            ctx.request_repaint_after(Duration::from_secs(5));
        }
    }

    fn obradi_oporavak(&mut self, vrati: bool, dijalog: &AktivniDijalog) {
        self.aktivni_dijalog = None;
        let (AktivniDijalog::Oporavak(ostatak), Some(o)) = (dijalog, self.oporavak.as_mut()) else {
            return;
        };
        if !vrati {
            o.odbaci(ostatak);
            self.aktivni_dijalog = o.ostaci().into_iter().next().map(AktivniDijalog::Oporavak);
            return;
        }
        let otvoreno = o
            .preuzmi(ostatak)
            .map_err(|e| e.to_string())
            .and_then(|staza| {
                radnja_otvori(
                    &mut self.uredjivac,
                    &mut self.datoteka,
                    &mut self.stanje,
                    &self.katalog,
                    staza,
                )
            });
        self.aktivni_dijalog = Some(match otvoreno {
            Ok(_) => self.vraceni_rad(ostatak),
            Err(_) => AktivniDijalog::PorukaGreske {
                naslov: self.katalog.tekst("oporavak-greska-naslov"),
                poruka: self.katalog.tekst("oporavak-greska"),
            },
        });
    }

    /// Vraćeni rad nema svoju datoteku, pa Spremi pita kamo, i ostaje nespremljen dok se ne spremi.
    fn vraceni_rad(&mut self, ostatak: &crate::oporavak::Ostatak) -> AktivniDijalog {
        self.datoteka.staza = None;
        self.uredjivac.oznaci_nespremljeno();
        self.karta.novi_projekt(self.uredjivac.projekt());
        let tekstovi = crate::oporavak::tekstovi_ostatka(&self.katalog, ostatak);
        AktivniDijalog::Obavijest {
            naslov: self.katalog.tekst("oporavak-vraceno-naslov"),
            poruka: self.katalog.s_tekstovima("oporavak-vraceno", &tekstovi),
        }
    }

    pub fn postavi_birac(&mut self, birac: Box<dyn BiracDatoteka>) {
        self.birac = birac;
    }

    #[must_use]
    pub fn odredi_naziv_datoteke(&self) -> Option<String> {
        tekst_statusa_datoteke(&self.stanje, &self.datoteka, &self.uredjivac, &self.katalog)
    }

    pub fn postavi_kontekst(ctx: &egui::Context) {
        ctx.set_fonts(fontovi());
    }

    pub fn crtaj_sucelje(&mut self, ui: &mut egui::Ui) {
        if self.treba_pitati_prije_zatvaranja(ui) {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.aktivni_dijalog = Some(AktivniDijalog::PotvrdaNespremljeno(
                NamjeraPotvrde::ZatvoriProzor,
            ));
        }
        let pocetak = Instant::now();
        let boje = Boje::za_temu(self.stanje.tema_tamna);
        primijeni_stil(ui.ctx(), &boje, self.stanje.tema_tamna);
        if self.stanje.novi_sloj.is_some() {
            // Prozorčić se crta prvi, da polje za naziv dobije Backspace prije karte i ploče.
            crtaj_prozorcic_novog_sloja(
                ui,
                &mut self.uredjivac,
                &mut self.stanje,
                &self.katalog,
                &mut self.ikone,
                &boje,
            );
        } else if self.stanje.naziv_novog.is_some() {
            crtaj_prozorcic_naziva(
                ui,
                &mut self.uredjivac,
                &mut self.stanje,
                &self.katalog,
                &boje,
            );
        } else if self.aktivni_dijalog.is_none() {
            // Dok dijalog čeka odgovor, Delete i prečaci ne smiju mijenjati projekt ispod njega.
            self.obradi_tipkovnicu(ui);
        }

        let dostupan = ui.ctx().content_rect();
        let w = dostupan.width().max(640.0);
        let h = dostupan.height().max(480.0);

        self.crtaj_izbornik(ui, w, &boje);
        let okvir_karte = Rect::from_min_max(
            pos2(0.0, Razmaci::VISINA_IZBORNIKA),
            pos2(w, h - Razmaci::VISINA_STATUSA),
        );
        self.crtaj_podrucje_karte(ui, okvir_karte, &boje);
        crtaj_plocu_slojeva(
            ui,
            &mut self.uredjivac,
            &mut self.stanje,
            &mut self.ikone,
            &self.katalog,
            okvir_karte,
            &boje,
        );
        if std::mem::take(&mut self.stanje.zatrazeno_brisanje) {
            self.pokreni_brisanje();
        }
        if std::mem::take(&mut self.stanje.zatrazen_uvoz) {
            self.izvrsi_uvoz();
        }
        if let Some(stavka) = self.stanje.zatrazen_izvoz.take() {
            self.pokreni_izvoz(formati::OpsegIzvoza::Stavka(stavka));
        }
        self.crtaj_statusnu_traku(ui, w, h, &boje);
        self.crtaj_slojeve_dijaloga(ui, &boje);

        self.automatsko_spremanje(ui.ctx());
        self.zadnje_vrijeme_okvira = pocetak.elapsed();
    }

    /// Nakon „Ne spremaj” zatvaranje stiže ponovno, a izmjene su još tu, pa se ne smije pitati opet.
    fn treba_pitati_prije_zatvaranja(&self, ui: &egui::Ui) -> bool {
        self.snimka.is_none()
            && !self.zatvaranje_potvrdjeno
            && ui.input(|i| i.viewport().close_requested())
            && self.uredjivac.je_li_izmijenjen()
    }

    fn obradi_tipkovnicu(&mut self, ui: &egui::Ui) {
        let dogadjaji = ui.input(|i| i.events.clone());
        for dogadjaj in &dogadjaji {
            if self.stanje.preimenovanje.is_some() {
                self.obradi_tipku_preimenovanja(dogadjaj);
            } else {
                self.obradi_tipku_prechaca(dogadjaj);
            }
        }
    }

    fn obradi_tipku_preimenovanja(&mut self, dogadjaj: &Event) {
        match dogadjaj {
            Event::Text(uneseno) => self.stanje.upisi_u_naziv(uneseno),
            Event::Key {
                key: Key::Backspace,
                pressed: true,
                ..
            } => self.stanje.obrisi_znak_naziva(),
            Event::Key {
                key: Key::ArrowLeft | Key::ArrowRight | Key::Home | Key::End,
                pressed: true,
                ..
            } => self.stanje.naziv_oznacen = false,
            Event::Key {
                key: Key::Enter,
                pressed: true,
                ..
            } => {
                let _ = potvrdi_preimenovanje(&mut self.uredjivac, &mut self.stanje);
            }
            Event::Key {
                key: Key::Escape,
                pressed: true,
                ..
            } => {
                self.stanje.otkazi_preimenovanje();
            }
            _ => {}
        }
    }

    fn obradi_tipku_prechaca(&mut self, dogadjaj: &Event) {
        if let Event::Key {
            key,
            pressed: true,
            modifiers,
            ..
        } = dogadjaj
        {
            self.izvrsi_precac(*key, modifiers);
        }
    }

    fn izvrsi_precac(&mut self, key: Key, modifiers: &egui::Modifiers) {
        // `command` je ⌘ na Macu, a Ctrl drugdje, pa Ctrl na Macu nije prečac (Odluke (9. 11. 2026.)).
        if je_precac_pomoci(key, modifiers.command) {
            self.aktivni_dijalog = Some(AktivniDijalog::Precaci);
            return;
        }
        if self.smije_brisati_tipkom(key, modifiers.command) {
            self.pokreni_brisanje();
            return;
        }
        if modifiers.command {
            self.obradi_cmd_precac(key, modifiers.shift);
            return;
        }
        match key {
            Key::F2 => {
                let _ = pokreni_preimenovanje_odabranog(self.uredjivac.projekt(), &mut self.stanje);
            }
            Key::Escape => {
                let _ = self.trazilica.ponisti_ili_zatvori();
            }
            _ => {}
        }
    }

    /// Objekte, mape i slojeve briše Delete ili ⌘ + Backspace, kao u Finderu (Odluke (10. 11. 2026.)).
    /// Sam Backspace briše samo točku, a to radi karta.
    fn smije_brisati_tipkom(&mut self, key: Key, cmd: bool) -> bool {
        let tipka_brisanja = match key {
            Key::Delete => true,
            Key::Backspace => cmd,
            _ => false,
        };
        tipka_brisanja && !self.karta.u_tijeku() && !self.trazilica.fokusirana
    }

    fn obradi_cmd_precac(&mut self, key: Key, shift: bool) {
        match (key, shift) {
            (Key::F, false) => {
                self.trazilica.fokusiraj = true;
                self.trazilica.otvorena = true;
            }
            (Key::Z, false) => {
                let _ = self.uredjivac.ponisti();
            }
            (Key::Y, _) | (Key::Z, true) => {
                let _ = self.uredjivac.ponovi();
            }
            (Key::N, false) => self.pokreni_radnju_datoteke(StavkaIzbornikaDatoteka::Novi),
            (Key::O, false) => self.pokreni_radnju_datoteke(StavkaIzbornikaDatoteka::Otvori),
            (Key::S, false) => self.pokreni_radnju_datoteke(StavkaIzbornikaDatoteka::Spremi),
            (Key::S, true) => self.pokreni_radnju_datoteke(StavkaIzbornikaDatoteka::SpremiKao),
            _ => {}
        }
    }

    fn crtaj_izbornik(&mut self, ui: &mut egui::Ui, w: f32, boje: &Boje) {
        let okvir = Rect::from_min_size(pos2(0.0, 0.0), vec2(w, Razmaci::VISINA_IZBORNIKA));
        ui.painter().rect_filled(okvir, 0.0, boje.izbornik);
        ui.painter().line_segment(
            [
                pos2(0.0, Razmaci::VISINA_IZBORNIKA),
                pos2(w, Razmaci::VISINA_IZBORNIKA),
            ],
            Stroke::new(1.0, boje.obrub),
        );

        let okvir_znaka = Rect::from_min_size(pos2(12.0, 9.0), vec2(22.0, 22.0));
        self.ikone.crtaj_znak_aplikacije(ui, okvir_znaka);

        self.crtaj_stavke_izbornika(ui, boje);
        self.crtaj_gumbe_izbornika(ui, w, boje);
    }

    fn crtaj_stavke_izbornika(&mut self, ui: &mut egui::Ui, boje: &Boje) {
        let stavke = [
            "izbornik-datoteka",
            "izbornik-uredi",
            "izbornik-crtanje",
            "izbornik-slojevi",
            "izbornik-prikaz",
            "izbornik-alati",
            "izbornik-pomoc",
        ];
        let mut x = 44.0;
        for kljuc in stavke {
            let naziv = self.katalog.tekst(kljuc);
            let sirina = naziv.chars().count() as f32 * 7.2 + 14.0;
            tekst_lijevo(ui, pos2(x, 20.0), &naziv, font_obicni(13.0), boje.tekst_2);
            if STAVKE_S_RADNJOM.contains(&kljuc) {
                let inter = Rect::from_min_size(pos2(x - 4.0, 6.0), vec2(sirina, 28.0));
                let odg = ui.interact(inter, ui.make_persistent_id(kljuc), Sense::click());
                odg.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &naziv));
                if odg.clicked() {
                    self.otvori_stavku_izbornika(kljuc);
                }
            }
            x += sirina;
        }
    }

    /// Pomoć zasad ima samo jednu stavku, pa odmah otvara „O programu”.
    pub(crate) fn otvori_stavku_izbornika(&mut self, kljuc: &str) {
        match kljuc {
            "izbornik-datoteka" => self.izbornik_datoteka_otvoren = !self.izbornik_datoteka_otvoren,
            "izbornik-slojevi" => self.stanje.ploca_otvorena = !self.stanje.ploca_otvorena,
            _ => self.aktivni_dijalog = Some(AktivniDijalog::OProgramu),
        }
    }

    fn crtaj_gumbe_izbornika(&mut self, ui: &mut egui::Ui, w: f32, boje: &Boje) {
        let tamna = self.stanje.tema_tamna;

        let okvir_qgis = Rect::from_min_size(pos2(w - 264.0, 7.0), vec2(116.0, 26.0));
        zaobljeni_okvir(
            ui,
            okvir_qgis,
            boje.ploca_2,
            Some(boje.obrub),
            Razmaci::ZAOBLJENJE_GUMBA,
        );
        let tekst_qgis = self.katalog.tekst("gumb-qgis");
        tekst_sredina(
            ui,
            okvir_qgis.center(),
            &tekst_qgis,
            font_srednji(12.0),
            boje.tekst,
        );
        let id_qgis = ui.make_persistent_id("gumb_qgis");
        let odg_qgis = ui.interact(okvir_qgis, id_qgis, Sense::click());
        odg_qgis.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), &tekst_qgis));
        if odg_qgis.clicked() {
            self.otvori_u_qgisu();
        }

        if let Some(datoteka_status) = self.odredi_naziv_datoteke() {
            tekst_desno(
                ui,
                pos2(w - 276.0, 20.0),
                &datoteka_status,
                font_obicni(12.0),
                boje.tekst_2,
            );
        }

        let stanje_ponisti = if self.uredjivac.moze_natrag() {
            "obicno"
        } else {
            "priguseno"
        };
        let opis_ponisti = OpisGumbaIkone {
            okvir: Rect::from_min_size(pos2(w - 140.0, 4.0), vec2(32.0, 32.0)),
            ikona: "ponisti",
            stanje: stanje_ponisti,
            crtez: 18,
            obrubljen: false,
            aktivno: false,
            labela: &self.katalog.tekst("gumb-ponisti"),
        };
        if gumb_ikona(ui, &mut self.ikone, &opis_ponisti, tamna, boje).clicked() {
            let _ = self.uredjivac.ponisti();
        }

        let stanje_ponovi = if self.uredjivac.moze_naprijed() {
            "obicno"
        } else {
            "priguseno"
        };
        let opis_ponovi = OpisGumbaIkone {
            okvir: Rect::from_min_size(pos2(w - 104.0, 4.0), vec2(32.0, 32.0)),
            ikona: "ponovi",
            stanje: stanje_ponovi,
            crtez: 18,
            obrubljen: false,
            aktivno: false,
            labela: &self.katalog.tekst("gumb-ponovi"),
        };
        if gumb_ikona(ui, &mut self.ikone, &opis_ponovi, tamna, boje).clicked() {
            let _ = self.uredjivac.ponovi();
        }

        let opis_tema = OpisGumbaIkone {
            okvir: Rect::from_min_size(pos2(w - 68.0, 4.0), vec2(32.0, 32.0)),
            ikona: "mjesec",
            stanje: "obicno",
            crtez: 18,
            obrubljen: false,
            aktivno: false,
            labela: &self.katalog.tekst("gumb-tema"),
        };
        if gumb_ikona(ui, &mut self.ikone, &opis_tema, tamna, boje).clicked() {
            self.stanje.tema_tamna = !self.stanje.tema_tamna;
        }

        let opis_pomoc = OpisGumbaIkone {
            okvir: Rect::from_min_size(pos2(w - 36.0, 4.0), vec2(32.0, 32.0)),
            ikona: "pomoc",
            stanje: "obicno",
            crtez: 18,
            obrubljen: false,
            aktivno: false,
            labela: &self.katalog.tekst("gumb-pomoc"),
        };
        if gumb_ikona(ui, &mut self.ikone, &opis_pomoc, tamna, boje).clicked() {
            self.aktivni_dijalog = Some(AktivniDijalog::Precaci);
        }
    }

    fn crtaj_podrucje_karte(&mut self, ui: &mut egui::Ui, okvir_karte: Rect, boje: &Boje) {
        ui.painter().rect_filled(okvir_karte, 0.0, boje.k_zemlja);
        self.stanje.klizanje_ploce = preuzmi_klizanje(ui, okvir_ploce(&self.stanje, okvir_karte));
        self.karta.prikazi(
            ui,
            okvir_karte,
            &mut self.uredjivac,
            &mut self.stanje,
            &self.katalog,
        );
        if let Some(oznaka) = self.trazilica.privremena_oznaka {
            trazilica::crtaj_privremenu_oznaku(
                ui.painter(),
                okvir_karte,
                &self.karta,
                oznaka,
                boje,
            );
        }
        if let Some(okvir_ploce) = okvir_ploce(&self.stanje, okvir_karte) {
            zaklon(ui, okvir_ploce, "ploca_slojeva");
        }

        let pomak_lijevo = if self.stanje.ploca_otvorena && self.stanje.ploca_lijevo {
            Razmaci::SIRINA_PLOCE + Razmaci::RUB
        } else {
            0.0
        };
        let desni_rub = if self.stanje.ploca_otvorena && !self.stanje.ploca_lijevo {
            okvir_karte.max.x - Razmaci::SIRINA_PLOCE - Razmaci::RUB
        } else {
            okvir_karte.max.x
        };
        let lijevi_x = okvir_karte.min.x + pomak_lijevo + Razmaci::RUB;

        // Tražilica ide poslije alata i ploče mjerenja, da popis rezultata leži iznad njih.
        self.crtaj_alatnu_traku(ui, pos2(lijevi_x, okvir_karte.min.y + 68.0), boje);
        self.karta.crtaj_plocu_mjerenja(
            ui,
            pos2(lijevi_x + 76.0, okvir_karte.min.y + 68.0),
            &mut self.uredjivac,
            &mut self.stanje,
            &self.katalog,
            &mut self.ikone,
            boje,
        );
        self.crtaj_trazilicu(ui, pos2(lijevi_x, okvir_karte.min.y + 12.0), boje);
        let sredina = (lijevi_x + desni_rub) / 2.0;
        self.karta.crtaj_poruku(
            ui,
            pos2(sredina, okvir_karte.max.y - 12.0),
            self.uredjivac.projekt(),
            &self.stanje,
            &self.katalog,
            boje,
        );
        self.crtaj_lebdece_gumbe(ui, pos2(desni_rub - 52.0, okvir_karte.min.y + 12.0), boje);
        self.crtaj_mjerilo_i_atribuciju(
            ui,
            pos2(lijevi_x, okvir_karte.max.y - 38.0),
            pos2(desni_rub - 60.0, okvir_karte.max.y - 8.0),
            boje,
        );
    }

    fn crtaj_trazilicu(&mut self, ui: &mut egui::Ui, min: Pos2, boje: &Boje) {
        let (okvir, enter) =
            trazilica::crtaj_polje(ui, min, &mut self.trazilica, &self.katalog, boje);
        let ikona_okvir =
            Rect::from_center_size(pos2(okvir.min.x + 21.0, okvir.center().y), vec2(18.0, 18.0));
        self.ikone.crtaj(
            ui,
            ikona_okvir,
            "trazi",
            self.stanje.tema_tamna,
            "obicno",
            18,
        );

        if self.trazilica.otvorena {
            let (rezultati, greska) = self.trazilica.rezultati(&self.uredjivac, &self.katalog);
            if enter {
                if let Some(prvi) = rezultati.first() {
                    trazilica::primijeni_odabir(
                        prvi,
                        &mut self.karta,
                        self.uredjivac.projekt(),
                        &mut self.stanje,
                        &mut self.trazilica,
                    );
                }
            } else if !self.trazilica.tekst.trim().is_empty() {
                let visina = if rezultati.is_empty() {
                    36.0 + 8.0
                } else {
                    (rezultati.len() as f32) * 36.0 + 8.0
                };
                let okvir_popisa =
                    Rect::from_min_size(pos2(okvir.min.x, okvir.max.y + 4.0), vec2(310.0, visina));
                zaklon(ui, okvir_popisa, "trazilica_popis");
                if let Some(odabir) = trazilica::crtaj_popis_rezultata(
                    ui,
                    okvir,
                    &rezultati,
                    greska,
                    &self.trazilica.tekst,
                    &self.katalog,
                    boje,
                ) {
                    trazilica::primijeni_odabir(
                        &odabir,
                        &mut self.karta,
                        self.uredjivac.projekt(),
                        &mut self.stanje,
                        &mut self.trazilica,
                    );
                }
            }
        }
    }

    fn crtaj_alatnu_traku(&mut self, ui: &mut egui::Ui, min: Pos2, boje: &Boje) {
        let okvir_alata = Rect::from_min_size(min, vec2(68.0, 259.0));
        zaklon(ui, okvir_alata, "alati");
        zaobljeni_okvir(ui, okvir_alata, boje.lebdeca, Some(boje.obrub), 12.0);
        let tamna = self.stanje.tema_tamna;
        for (indeks, alat) in Alat::SVI.into_iter().enumerate() {
            let gy = min.y + 6.0 + indeks as f32 * 50.0;
            let g_okvir = Rect::from_min_size(pos2(min.x + 6.0, gy), vec2(56.0, 47.0));
            let naziv = self.katalog.tekst(alat.kljuc_poruke());
            let opis = OpisGumbaAlata {
                okvir: g_okvir,
                ikona: alat.ikona(),
                tekst: &naziv,
                odabran: self.stanje.alat == alat,
            };
            if gumb_alat(ui, &mut self.ikone, &opis, tamna, boje).clicked() {
                self.stanje.alat = alat;
            }
        }
    }

    fn crtaj_lebdece_gumbe(&mut self, ui: &mut egui::Ui, pocetak: Pos2, boje: &Boje) {
        let kx = pocetak.x;
        let ky = pocetak.y;
        let tamna = self.stanje.tema_tamna;

        // Odluke (9. 11. 2026.): gumb Ispis je skriven dok ispis ne proradi (faza 6).
        let k1 = Rect::from_min_size(pos2(kx, ky), vec2(40.0, 40.0));
        zaklon(ui, k1, "lebdeci_k1");
        zaobljeni_okvir(ui, k1, boje.lebdeca, Some(boje.obrub), 10.0);
        let zaslon = gumb_ikona(
            ui,
            &mut self.ikone,
            &OpisGumbaIkone {
                okvir: Rect::from_min_size(pos2(kx, ky), vec2(40.0, 40.0)),
                ikona: "zaslon",
                stanje: "obicno",
                crtez: 20,
                obrubljen: false,
                aktivno: false,
                labela: &self.katalog.tekst("lebdeci-zaslon"),
            },
            tamna,
            boje,
        );

        let k2 = Rect::from_min_size(pos2(kx, ky + 50.0), vec2(40.0, 40.0));
        zaklon(ui, k2, "lebdeci_k2");
        zaobljeni_okvir(ui, k2, boje.lebdeca, Some(boje.obrub), 10.0);
        let _ = gumb_ikona(
            ui,
            &mut self.ikone,
            &OpisGumbaIkone {
                okvir: k2,
                ikona: "sjever",
                stanje: "obicno",
                crtez: 20,
                obrubljen: false,
                aktivno: false,
                labela: &self.katalog.tekst("lebdeci-sjever"),
            },
            tamna,
            boje,
        );

        let k3 = Rect::from_min_size(pos2(kx, ky + 100.0), vec2(40.0, 81.0));
        zaklon(ui, k3, "lebdeci_k3");
        zaobljeni_okvir(ui, k3, boje.lebdeca, Some(boje.obrub), 10.0);
        let plus = gumb_ikona(
            ui,
            &mut self.ikone,
            &OpisGumbaIkone {
                okvir: Rect::from_min_size(pos2(kx, ky + 100.0), vec2(40.0, 40.0)),
                ikona: "plus",
                stanje: "obicno",
                crtez: 20,
                obrubljen: false,
                aktivno: false,
                labela: &self.katalog.tekst("lebdeci-plus"),
            },
            tamna,
            boje,
        );
        ui.painter().line_segment(
            [pos2(kx + 6.0, ky + 140.0), pos2(kx + 34.0, ky + 140.0)],
            Stroke::new(1.0, boje.obrub),
        );
        let minus = gumb_ikona(
            ui,
            &mut self.ikone,
            &OpisGumbaIkone {
                okvir: Rect::from_min_size(pos2(kx, ky + 141.0), vec2(40.0, 40.0)),
                ikona: "minus",
                stanje: "obicno",
                crtez: 20,
                obrubljen: false,
                aktivno: false,
                labela: &self.katalog.tekst("lebdeci-minus"),
            },
            tamna,
            boje,
        );
        if plus.clicked() {
            self.karta.priblizi();
        }
        if minus.clicked() {
            self.karta.udalji();
        }
        if zaslon.clicked() {
            let cijeli = ui.ctx().input(|i| i.viewport().fullscreen.unwrap_or(false));
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!cijeli));
        }
    }

    fn crtaj_mjerilo_i_atribuciju(
        &self,
        ui: &egui::Ui,
        pozicija_mjerila: Pos2,
        desno_dolje: Pos2,
        boje: &Boje,
    ) {
        self.karta.crtaj_traku_mjerila(ui, pozicija_mjerila, boje);
        self.karta.crtaj_navod(ui, desno_dolje, boje);
    }

    fn crtaj_statusnu_traku(&mut self, ui: &mut egui::Ui, w: f32, h: f32, boje: &Boje) {
        let vrh = h - Razmaci::VISINA_STATUSA;
        let okvir = Rect::from_min_max(pos2(0.0, vrh), pos2(w, h));
        ui.painter().rect_filled(okvir, 0.0, boje.status);
        ui.painter()
            .line_segment([pos2(0.0, vrh), pos2(w, vrh)], Stroke::new(1.0, boje.obrub));

        let sy = vrh + Razmaci::VISINA_STATUSA * 0.5;
        let tamna = self.stanje.tema_tamna;
        let status_karte = self.karta.status();
        let stavke =
            odredi_stavke_statusne_trake(&self.stanje, &self.katalog, status_karte.as_ref());

        if let Some(koordinate) = &stavke.koordinate {
            tekst_lijevo(
                ui,
                pos2(12.0, sy),
                koordinate,
                font_obicni(11.0),
                boje.tekst,
            );
            tekst_lijevo(
                ui,
                pos2(180.0, sy),
                &stavke.sustav,
                font_srednji(11.0),
                boje.tekst_2,
            );
        } else {
            tekst_lijevo(
                ui,
                pos2(12.0, sy),
                &stavke.sustav,
                font_srednji(11.0),
                boje.tekst_2,
            );
        }

        if let Some(mjerilo) = &stavke.mjerilo {
            tekst_lijevo(
                ui,
                pos2(275.0, sy),
                mjerilo,
                font_srednji(11.0),
                boje.tekst_2,
            );
        }

        if let Some(mreza) = &stavke.mreza {
            let okvir_mreza = okvir_ikone_u_sredini(pos2(w - 82.0, sy), 14);
            self.ikone
                .crtaj(ui, okvir_mreza, "mreza", tamna, "obicno", 14);
            tekst_lijevo(
                ui,
                pos2(w - 70.0, sy),
                mreza,
                font_obicni(11.0),
                boje.tekst_2,
            );
        }

        if let Some(izvor) = &stavke.izvor_podloge {
            let okvir_karta = okvir_ikone_u_sredini(pos2(w - 200.0, sy), 14);
            self.ikone
                .crtaj(ui, okvir_karta, "karta", tamna, "obicno", 14);
            tekst_lijevo(
                ui,
                pos2(w - 188.0, sy),
                izvor,
                font_obicni(11.0),
                boje.tekst_2,
            );
        }
    }

    pub fn pokreni_radnju_datoteke(&mut self, stavka: StavkaIzbornikaDatoteka) {
        match stavka {
            StavkaIzbornikaDatoteka::Novi => self.pokreni_novi(),
            StavkaIzbornikaDatoteka::Otvori => self.pokreni_otvori(),
            StavkaIzbornikaDatoteka::Spremi => {
                if let Err(greska) = self.izvrsi_spremanje() {
                    self.postavi_gresku_spremanja(greska);
                }
            }
            StavkaIzbornikaDatoteka::SpremiKao => self.izvrsi_spremi_kao(),
            StavkaIzbornikaDatoteka::Uvezi => self.izvrsi_uvoz(),
            StavkaIzbornikaDatoteka::Izvezi => self.pokreni_izvoz(formati::OpsegIzvoza::Projekt),
        }
    }

    /// Prvo pregled onoga što ide u KML, a datoteka se bira tek nakon toga.
    pub fn pokreni_izvoz(&mut self, opseg: formati::OpsegIzvoza) {
        let priprema = crate::izvoz::pripremi(
            self.uredjivac.projekt(),
            opseg,
            self.datoteka.staza.as_deref(),
            &self.katalog,
        );
        self.aktivni_dijalog = Some(AktivniDijalog::Izvoz(priprema));
    }

    fn izvrsi_izvoz(&mut self, dijalog: &AktivniDijalog) {
        let AktivniDijalog::Izvoz(priprema) = dijalog else {
            return;
        };
        let nastavak = priprema.format.nastavak();
        let ime = format!("{}.{nastavak}", priprema.naziv);
        let Some(staza) = self.birac.izvezi_datoteku(&ime, nastavak) else {
            return;
        };
        let ishod = crate::izvoz::izvezi(self.uredjivac.projekt(), priprema, &staza, &self.katalog);
        self.aktivni_dijalog = Some(match ishod {
            Ok(poruka) => AktivniDijalog::Obavijest {
                naslov: self.katalog.tekst("izvoz-gotov-naslov"),
                poruka,
            },
            Err(poruka) => AktivniDijalog::PorukaGreske {
                naslov: self.katalog.tekst("izvoz-greska-naslov"),
                poruka,
            },
        });
    }

    /// KML ili KMZ ide u novu mapu, a karta se pomiče na uvezeno.
    fn izvrsi_uvoz(&mut self) {
        let Some(staza) = self.birac.uvezi_datoteku() else {
            return;
        };
        let gpkg = staza
            .extension()
            .is_some_and(|n| n.eq_ignore_ascii_case("gpkg"));
        if gpkg {
            self.aktivni_dijalog = Some(match crate::uvoz_gpkg::pripremi(&staza, &self.katalog) {
                Ok(priprema) => AktivniDijalog::UvozGpkg(priprema),
                Err(poruka) => AktivniDijalog::PorukaGreske {
                    naslov: self.katalog.tekst("uvoz-greska-naslov"),
                    poruka,
                },
            });
            return;
        }
        let ishod = crate::uvoz::uvezi_datoteku(
            &mut self.uredjivac,
            &mut self.stanje,
            &self.katalog,
            &staza,
        );
        self.aktivni_dijalog = Some(match ishod {
            Ok(sazetak) => {
                if let Some((d, s, zum)) =
                    crate::uvoz::obuhvat_mape(self.uredjivac.projekt(), sazetak.mapa)
                {
                    self.karta.karta().idi_na(d, s, zum);
                }
                AktivniDijalog::Obavijest {
                    naslov: self.katalog.tekst("uvoz-naslov"),
                    poruka: crate::uvoz::poruka_sazetka(&self.katalog, &sazetak),
                }
            }
            Err(poruka) => AktivniDijalog::PorukaGreske {
                naslov: self.katalog.tekst("uvoz-greska-naslov"),
                poruka,
            },
        });
    }

    /// Odabrani slojevi idu u novu mapu, a karta se pomiče na njih.
    fn izvrsi_uvoz_gpkg(&mut self, dijalog: &AktivniDijalog) {
        let AktivniDijalog::UvozGpkg(priprema) = dijalog else {
            return;
        };
        let ishod = crate::uvoz_gpkg::uvezi(&mut self.uredjivac, priprema);
        self.aktivni_dijalog = Some(match ishod {
            Ok(s) => {
                self.stanje.postavi_otvorenost_mape(s.mapa, true);
                self.stanje.odabrano = Some(Odabir::Mapa(s.mapa));
                if let Some((d, sirina, zum)) =
                    crate::uvoz::obuhvat_mape(self.uredjivac.projekt(), s.mapa)
                {
                    self.karta.karta().idi_na(d, sirina, zum);
                }
                let tekstovi = [
                    ("objekti", self.katalog.broj_objekata(s.objekata)),
                    ("slojevi", self.katalog.broj_slojeva(s.slojeva)),
                ];
                AktivniDijalog::Obavijest {
                    naslov: self.katalog.tekst("uvoz-naslov"),
                    poruka: self.katalog.s_tekstovima("uvoz-gpkg-sazetak", &tekstovi),
                }
            }
            Err(g) => AktivniDijalog::PorukaGreske {
                naslov: self.katalog.tekst("uvoz-greska-naslov"),
                poruka: self
                    .katalog
                    .s_tekstovima("uvoz-greska-projekt", &[("opis", g.to_string())]),
            },
        });
    }

    fn pokreni_novi(&mut self) {
        if self.uredjivac.je_li_izmijenjen() {
            self.aktivni_dijalog = Some(AktivniDijalog::PotvrdaNespremljeno(NamjeraPotvrde::Novi));
        } else {
            radnja_novi(&mut self.uredjivac, &mut self.datoteka, &mut self.stanje);
            self.karta.novi_projekt(self.uredjivac.projekt());
        }
    }

    fn pokreni_otvori(&mut self) {
        if self.uredjivac.je_li_izmijenjen() {
            self.aktivni_dijalog =
                Some(AktivniDijalog::PotvrdaNespremljeno(NamjeraPotvrde::Otvori));
        } else {
            self.izvrsi_otvaranje();
        }
    }

    fn izvrsi_spremi_kao(&mut self) {
        if let Some(staza) = self.birac.spremi_datoteku(None)
            && let Err(poruka) = radnja_spremi_kao(
                &mut self.uredjivac,
                &mut self.datoteka,
                &self.katalog,
                staza,
            )
        {
            self.postavi_gresku_spremanja(GreskaSpremanja::Poruka(poruka));
        }
    }

    /// Gumb „Otvori u QGIS-u”: projekt se najprije sprema, jer QGIS čita datoteku, a ne ovaj prozor.
    /// Nespremljen projekt traži mjesto za datoteku; ako korisnik odustane, ne događa se ništa.
    pub fn otvori_u_qgisu(&mut self) {
        match self.izvrsi_spremanje() {
            Err(greska) => return self.postavi_gresku_spremanja(greska),
            Ok(false) => return,
            Ok(true) => {}
        }
        let Some(staza) = self.datoteka.staza.clone() else {
            return;
        };
        if let Err(poruka) = self.qgis.otvori(&staza) {
            self.aktivni_dijalog = Some(AktivniDijalog::PorukaGreske {
                naslov: self.katalog.tekst("greska-qgis-naslov"),
                poruka,
            });
        }
    }

    fn postavi_gresku_spremanja(&mut self, greska: GreskaSpremanja) {
        self.aktivni_dijalog = Some(match greska {
            GreskaSpremanja::Sukob => AktivniDijalog::SukobSpremanja,
            GreskaSpremanja::Poruka(poruka) => AktivniDijalog::PorukaGreske {
                naslov: self.katalog.tekst("greska-spremanja-naslov"),
                poruka,
            },
        });
    }

    fn izvrsi_otvaranje(&mut self) {
        if let Some(staza) = self.birac.otvori_datoteku() {
            match radnja_otvori(
                &mut self.uredjivac,
                &mut self.datoteka,
                &mut self.stanje,
                &self.katalog,
                staza,
            ) {
                Ok(upozorenja) => {
                    self.karta.novi_projekt(self.uredjivac.projekt());
                    if !upozorenja.is_empty() {
                        self.aktivni_dijalog = Some(AktivniDijalog::UpozorenjaOtvaranja {
                            upozorenja,
                            otvoren_popis: false,
                        });
                    }
                }
                Err(poruka) => {
                    self.aktivni_dijalog = Some(AktivniDijalog::PorukaGreske {
                        naslov: self.katalog.tekst("greska-otvaranja-naslov"),
                        poruka,
                    });
                }
            }
        }
    }

    fn izvrsi_spremanje(&mut self) -> Result<bool, GreskaSpremanja> {
        let spremljeno = radnja_spremi(&mut self.uredjivac, &mut self.datoteka, &self.katalog)?;
        if !spremljeno {
            if let Some(staza) = self.birac.spremi_datoteku(None) {
                radnja_spremi_kao(
                    &mut self.uredjivac,
                    &mut self.datoteka,
                    &self.katalog,
                    staza,
                )
                .map_err(GreskaSpremanja::Poruka)?;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(true)
        }
    }

    fn pokreni_brisanje(&mut self) {
        if self.karta.broj_odabranih() > 1 {
            let _ = self.karta.obrisi_odabrane(&mut self.uredjivac);
            return;
        }
        let Some(cilj) = self.stanje.odabrano else {
            return;
        };
        match cilj {
            Odabir::Sloj(id) => {
                let stavka = Stavka::Sloj(id);
                let nepodrzani = self.uredjivac.projekt().nepodrzani_zapisi(stavka);
                if nepodrzani > 0 {
                    let naziv = self
                        .uredjivac
                        .projekt()
                        .sloj(id)
                        .map_or("".into(), |s| s.naziv.clone());
                    self.aktivni_dijalog = Some(AktivniDijalog::UpozorenjeBrisanja {
                        stavka,
                        naziv,
                        broj_nepodrzanih: nepodrzani,
                    });
                    return;
                }
            }
            Odabir::Mapa(id) => {
                let stavka = Stavka::Mapa(id);
                let nepodrzani = self.uredjivac.projekt().nepodrzani_zapisi(stavka);
                if nepodrzani > 0 {
                    let naziv = self
                        .uredjivac
                        .projekt()
                        .mapa(id)
                        .map_or("".into(), |m| m.naziv.clone());
                    self.aktivni_dijalog = Some(AktivniDijalog::UpozorenjeBrisanja {
                        stavka,
                        naziv,
                        broj_nepodrzanih: nepodrzani,
                    });
                    return;
                }
            }
            Odabir::Objekt(_, _) => {}
        }
        let _ = obrisi_odabrano(&mut self.uredjivac, &mut self.stanje);
    }

    fn crtaj_slojeve_dijaloga(&mut self, ui: &mut egui::Ui, boje: &Boje) {
        if let Some(odabrana_stavka) = crtaj_padajuci_izbornik_datoteka(
            ui,
            &mut self.izbornik_datoteka_otvoren,
            boje,
            &self.katalog,
        ) {
            self.pokreni_radnju_datoteke(odabrana_stavka);
        }

        if let Some(mut dijalog) = self.aktivni_dijalog.take() {
            let ishod = crtaj_aktivni_dijalog(ui, &mut dijalog, boje, &self.katalog);
            match ishod {
                Some(ishod) => self.obradi_ishod_dijaloga(ui, ishod, dijalog),
                None => {
                    self.aktivni_dijalog = Some(dijalog);
                }
            }
        }
    }

    fn obradi_ishod_dijaloga(
        &mut self,
        ui: &mut egui::Ui,
        ishod: IshodDijaloga,
        mut dijalog: AktivniDijalog,
    ) {
        match ishod {
            IshodDijaloga::PotvrdaSpremi(namjera) => {
                self.obradi_potvrdu_spremi(ui, namjera, dijalog);
            }
            IshodDijaloga::PotvrdaNeSpremaj(namjera) => {
                self.aktivni_dijalog = None;
                self.nastavi_nakon_potvrde(ui, namjera);
            }
            IshodDijaloga::ZatvoriPoruku => self.aktivni_dijalog = None,
            IshodDijaloga::ObrisiPotvrdjeno(stavka) => self.obradi_potvrdu_brisanja(stavka),
            IshodDijaloga::UpozorenjaPreklopiPopis => {
                preklopi_popis(&mut dijalog);
                self.aktivni_dijalog = Some(dijalog);
            }
            IshodDijaloga::SukobSpremiKao => {
                self.aktivni_dijalog = None;
                self.izvrsi_spremi_kao();
            }
            IshodDijaloga::Prihvati => {
                self.aktivni_dijalog = None;
                self.izvrsi_izvoz(&dijalog);
                self.izvrsi_uvoz_gpkg(&dijalog);
            }
            IshodDijaloga::Oporavak(vrati) => self.obradi_oporavak(vrati, &dijalog),
        }
    }

    fn obradi_potvrdu_spremi(
        &mut self,
        ui: &mut egui::Ui,
        namjera: NamjeraPotvrde,
        dijalog: AktivniDijalog,
    ) {
        match self.izvrsi_spremanje() {
            Ok(true) => {
                self.aktivni_dijalog = None;
                self.nastavi_nakon_potvrde(ui, namjera);
            }
            Ok(false) => {
                self.aktivni_dijalog = Some(dijalog);
            }
            Err(greska) => {
                self.postavi_gresku_spremanja(greska);
            }
        }
    }

    fn obradi_potvrdu_brisanja(&mut self, stavka: Stavka) {
        let _ = self.uredjivac.izvrsi(NaredbaStabla::Obrisi { stavka });
        self.stanje.odabrano = None;
        self.aktivni_dijalog = None;
    }

    fn nastavi_nakon_potvrde(&mut self, ui: &mut egui::Ui, namjera: NamjeraPotvrde) {
        match namjera {
            NamjeraPotvrde::Novi => {
                radnja_novi(&mut self.uredjivac, &mut self.datoteka, &mut self.stanje);
                self.karta.novi_projekt(self.uredjivac.projekt());
            }
            NamjeraPotvrde::Otvori => {
                self.izvrsi_otvaranje();
            }
            NamjeraPotvrde::ZatvoriProzor => {
                self.zatvaranje_potvrdjeno = true;
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
    }

    fn obradi_snimanje_zaslona(&mut self, ui: &egui::Ui) {
        let Some(putanja) = self.snimka.clone() else {
            return;
        };
        if self.pocetak_rada.elapsed() > Duration::from_secs(60) {
            eprintln!("Isteklo vrijeme za stvaranje snimke zaslona (60 s)");
            std::process::exit(1);
        }
        self.okvir_brojac += 1;
        let ceka = self.pocetak.elapsed() < self.cekanje_snimke;
        if ceka {
            // Brojač stoji, pa se snimka traži točno jednom, u drugom okviru nakon čekanja.
            self.okvir_brojac = 1;
        }
        if self.okvir_brojac == 2 {
            ui.ctx().request_screenshot(move |slika| {
                spremi_sliku_zaslona(&slika, &putanja);
                std::process::exit(0);
            });
        }
        ui.ctx().request_repaint();
    }
}

impl eframe::App for Aplikacija {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.crtaj_sucelje(ui);
        self.obradi_snimanje_zaslona(ui);
    }

    /// Program sam prepoznaje sustav: na Macu je ⌃ + klik desni klik.
    fn raw_input_hook(&mut self, ctx: &egui::Context, unos: &mut egui::RawInput) {
        self.desni_klik_maca.obradi(ctx.os(), unos);
    }
}

pub fn spremi_sliku_zaslona(slika: &egui::ColorImage, putanja: &Path) {
    if let Some(roditelj) = putanja.parent() {
        let _ = std::fs::create_dir_all(roditelj);
    }
    let rgba: Vec<u8> = slika
        .pixels
        .iter()
        .flat_map(|p| p.to_srgba_unmultiplied())
        .collect();
    let _ = image::save_buffer(
        putanja,
        &rgba,
        slika.width() as u32,
        slika.height() as u32,
        image::ColorType::Rgba8,
    );
}

/// F1 ili ⌘/ (Ctrl+/ drugdje) otvara popis prečaca.
const fn je_precac_pomoci(key: Key, cmd: bool) -> bool {
    matches!(key, Key::F1) || (cmd && matches!(key, Key::Slash))
}

#[cfg(test)]
mod testovi {
    use super::*;
    use crate::primjer::{
        izgradi_stablo_opterecenja, najveca_dubina_mapa, ukupno_slojeva_u_projektu,
    };
    use crate::stanje::Odabir;
    use eframe::egui::Modifiers;
    use egui_kittest::{
        Harness,
        kittest::{NodeT, Queryable},
    };

    fn nacrtani_tekstovi<S>(harness: &Harness<'_, S>) -> Vec<String> {
        harness
            .output()
            .shapes
            .iter()
            .filter_map(|s| match &s.shape {
                egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
                _ => None,
            })
            .collect()
    }

    fn je_oznaceno<S>(harness: &Harness<'_, S>, labela: &str) -> Option<bool> {
        harness
            .get_by_label(labela)
            .accesskit_node()
            .toggled()
            .map(|t| t == egui::accesskit::Toggled::True)
    }

    #[test]
    fn zaglavlje_premjesta_plocu_a_odabrani_redak_i_podloga_su_oznaceni() {
        let app = Aplikacija::nova(PostavkeAplikacije {
            primjer: true,
            ..Default::default()
        });
        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));

        let tekstovi = nacrtani_tekstovi(&harness);
        assert!(
            tekstovi.iter().any(|t| t == "100 m"),
            "primjer crta traku mjerila"
        );

        assert_eq!(je_oznaceno(&harness, "Parcela 1124/3"), Some(true));
        assert_eq!(je_oznaceno(&harness, "Parcela 1126"), Some(false));
        assert_eq!(je_oznaceno(&harness, "Karta"), Some(true));
        assert_eq!(je_oznaceno(&harness, "DGU ortofoto"), Some(false));

        harness.get_by_label("DGU ortofoto").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert_eq!(harness.state().stanje.podloga, 1);
        assert_eq!(je_oznaceno(&harness, "DGU ortofoto"), Some(true));
        assert_eq!(je_oznaceno(&harness, "Karta"), Some(false));

        assert!(!harness.state().stanje.ploca_lijevo);
        harness.get_by_label("Premjesti ploču Slojevi").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(
            harness.state().stanje.ploca_lijevo,
            "klik premješta ploču lijevo"
        );
        harness.get_by_label("Premjesti ploču Slojevi").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(
            !harness.state().stanje.ploca_lijevo,
            "drugi klik je vraća desno"
        );
    }

    fn cekaj_smirenje<S>(harness: &mut Harness<'_, S>, najvise: Duration) {
        let pocetak = Instant::now();
        while pocetak.elapsed() < najvise {
            if harness.try_run().is_ok() {
                return;
            }
        }
        panic!("sučelje se nije smirilo unutar {najvise:?}");
    }

    #[test]
    fn rasclamba_argumenata_i_prazan_pocetni_projekt_bez_zastavice() {
        let postavke =
            rasclani_argumente(["--primjer", "--tamna", "--lijevo", "--snimka", "a.png"]);
        assert_eq!(
            postavke,
            PostavkeAplikacije {
                primjer: true,
                tamna: true,
                ploca_lijevo: true,
                snimka: Some(PathBuf::from("a.png")),
                podloga_s_mreze: false,
                dijalog: None,
                oporavak: None,
            }
        );

        let prazna = Aplikacija::default();
        assert!(prazna.uredjivac.projekt().korijen().is_empty());
        assert_eq!(prazna.uredjivac.projekt().slojevi().count(), 0);
    }

    fn izbornik_retka<'a>(harness: &mut Harness<'a, Aplikacija>, redak: &str, stavka: &str) {
        harness.get_by_label(redak).click_secondary();
        cekaj_smirenje(harness, Duration::from_millis(500));
        harness.get_by_label(stavka).click();
        cekaj_smirenje(harness, Duration::from_millis(500));
    }

    #[test]
    fn desni_klik_preimenuje_i_brise_mapu_i_objekt() {
        let app = Aplikacija::nova(PostavkeAplikacije {
            primjer: true,
            ..Default::default()
        });
        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

        // Objekt: upisani naziv zamijeni cijeli stari.
        izbornik_retka(&mut harness, "Parcela 1118/2", "Preimenuj");
        assert!(harness.state().stanje.naziv_oznacen);
        harness.event(Event::Text("Parcela 7".into()));
        harness.key_press(Key::Enter);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Parcela 7").is_some());
        assert!(harness.query_by_label("Parcela 1118/2").is_none());

        // Strelica skida oznaku, pa se upis dopisuje na kraj.
        harness.key_press(Key::F2);
        harness.key_press(Key::ArrowRight);
        harness.event(Event::Text("a".into()));
        harness.key_press(Key::Enter);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Parcela 7a").is_some());

        // Mapa: preimenovanje pa brisanje iz istog izbornika.
        izbornik_retka(&mut harness, "Teren 2026", "Preimenuj");
        harness.key_press(Key::Backspace);
        harness.event(Event::Text("Kanali".into()));
        harness.key_press(Key::Enter);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Kanali").is_some());

        izbornik_retka(&mut harness, "Kanali", "Obriši");
        assert!(harness.query_by_label("Kanali").is_none());
        assert!(harness.query_by_label("Parcela 7a").is_none());
        assert!(harness.state().uredjivac.je_li_izmijenjen());
    }

    /// Mapa na vrhu stabla.
    fn mapa_po_nazivu(app: &Aplikacija, naziv: &str) -> slojevi::IdMape {
        let projekt = app.uredjivac.projekt();
        projekt
            .korijen()
            .iter()
            .find_map(|s| match *s {
                Stavka::Mapa(id) if projekt.mapa(id).is_some_and(|m| m.naziv == naziv) => Some(id),
                _ => None,
            })
            .expect("mapa")
    }

    fn mapa_sloja(app: &Aplikacija, naziv: &str) -> Option<slojevi::IdMape> {
        app.uredjivac
            .projekt()
            .slojevi()
            .find(|(_, s)| s.naziv == naziv)
            .and_then(|(_, s)| s.roditelj)
    }

    #[test]
    fn desni_klik_premjesta_sloj_u_drugu_mapu() {
        let app = Aplikacija::nova(PostavkeAplikacije {
            primjer: true,
            ..Default::default()
        });
        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
        let uvezeno = mapa_po_nazivu(harness.state(), "Uvezeno");

        izbornik_retka(&mut harness, "Trase", "Premjesti u");
        // „Uvezeno” je i redak u stablu, pa se uzima onaj u stupcu podizbornika, uz „Izvan mapa”.
        let stupac = harness.get_by_label("Izvan mapa").rect().min.x;
        harness
            .get_all_by_label("Uvezeno")
            .find(|n| (n.rect().min.x - stupac).abs() < 1.0)
            .expect("Uvezeno u podizborniku")
            .click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));

        assert_eq!(mapa_sloja(harness.state(), "Trase"), Some(uvezeno));
        assert!(harness.query_by_label("Izvan mapa").is_none());
        assert!(harness.state().uredjivac.je_li_izmijenjen());
    }

    fn sloj_objekta(app: &Aplikacija, naziv: &str) -> Option<String> {
        app.uredjivac
            .projekt()
            .slojevi()
            .find(|(_, s)| s.objekti.values().any(|o| o.naziv == naziv))
            .map(|(_, s)| s.naziv.clone())
    }

    #[test]
    fn objekt_prelazi_u_drugi_sloj_izbornikom_i_povlacenjem() {
        let app = Aplikacija::nova(PostavkeAplikacije {
            primjer: true,
            ..Default::default()
        });
        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

        // Mapa Parcele 2025 je zatvorena, pa „Stare parcele” postoji samo u podizborniku.
        izbornik_retka(&mut harness, "Parcela 1126", "Premjesti u");
        harness.get_by_label("Stare parcele").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        let stare = Some("Stare parcele".to_owned());
        assert_eq!(sloj_objekta(harness.state(), "Parcela 1126"), stare);

        // Na sloj linija poligon ne ide, a na sloj poligona ide.
        let parcela = harness.get_by_label("Parcela 1118/2").rect().center();
        let trase = harness.get_by_label("Trase").rect().center();
        harness.drag_at(parcela);
        harness.hover_at(parcela + vec2(0.0, 20.0));
        harness.drop_at(trase);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        let parcele = Some("Parcele".to_owned());
        assert_eq!(sloj_objekta(harness.state(), "Parcela 1118/2"), parcele);

        let parcela = harness.get_by_label("Parcela 1118/2").rect().center();
        let cilj = harness.get_by_label("Stare parcele").rect().center();
        harness.drag_at(parcela);
        harness.hover_at(parcela + vec2(0.0, 20.0));
        harness.hover_at(cilj);
        harness.drop_at(cilj);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert_eq!(sloj_objekta(harness.state(), "Parcela 1118/2"), stare);
    }

    #[test]
    fn povlacenje_premjesta_sloj_a_mapu_ne_pusta_u_vlastitu_podmapu() {
        let app = Aplikacija::nova(PostavkeAplikacije {
            primjer: true,
            ..Default::default()
        });
        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);
        let uvezeno = mapa_po_nazivu(harness.state(), "Uvezeno");
        let teren = mapa_po_nazivu(harness.state(), "Teren 2026");

        let pocetak = harness.get_by_label("Točke").rect().center();
        let cilj = harness.get_by_label("Uvezeno").rect().center();
        harness.drag_at(pocetak);
        harness.hover_at(pocetak + vec2(0.0, 20.0));
        harness.hover_at(cilj);
        harness.run();
        assert!(harness.state().stanje.povuceno.is_some());
        harness.drop_at(cilj);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert_eq!(mapa_sloja(harness.state(), "Točke"), Some(uvezeno));
        assert!(harness.state().stanje.povuceno.is_none());

        // Teren 2026 ispušten na svoj sloj Parcele ostaje gdje je bio.
        let pocetak = harness.get_by_label("Teren 2026").rect().center();
        let cilj = harness.get_by_label("Parcele").rect().center();
        harness.drag_at(pocetak);
        harness.hover_at(pocetak + vec2(0.0, 20.0));
        harness.hover_at(cilj);
        harness.drop_at(cilj);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        let projekt = harness.state().uredjivac.projekt();
        assert_eq!(projekt.mapa(teren).and_then(|m| m.roditelj), None);
        assert_eq!(mapa_sloja(harness.state(), "Parcele"), Some(teren));
    }

    #[test]
    fn sucelje_podrzava_dodavanje_preimenovanje_brisanje_ponisti_i_ponovi() {
        let app = Aplikacija::nova(PostavkeAplikacije {
            primjer: true,
            ..Default::default()
        });
        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

        harness.get_by_label("Novi sloj").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        harness.get_by_label("Napravi sloj").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));

        harness.key_press(Key::F2);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        for _ in 0..10 {
            harness.key_press(Key::Backspace);
        }
        harness.event(Event::Text("Vodovod".into()));
        harness.key_press(Key::Enter);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));

        assert!(harness.query_by_label("Vodovod").is_some());

        harness.get_by_label("Poništi").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Vodovod").is_none());
        assert!(harness.query_by_label("Sloj 8").is_some());

        harness.get_by_label("Ponovi").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Vodovod").is_some());

        harness.key_press(Key::Delete);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Vodovod").is_none());

        // Ctrl bez ⌘ je Ctrl na Macu i ne smije poništiti.
        harness.key_press_modifiers(Modifiers::CTRL, Key::Z);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Vodovod").is_none());

        harness.key_press_modifiers(Modifiers::COMMAND, Key::Z);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Vodovod").is_some());

        harness.key_press_modifiers(Modifiers::COMMAND, Key::Y);
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Vodovod").is_none());
    }

    #[test]
    fn sucelje_premjesta_plocu_mijenja_temu_alate_i_cuva_stanje_prikaza() {
        let app = Aplikacija::nova(PostavkeAplikacije {
            primjer: true,
            ..Default::default()
        });
        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

        let desni_x = harness
            .get_by_label("Premjesti ploču Slojevi")
            .rect()
            .center()
            .x;
        assert!(desni_x > 900.0);

        harness.drag_at(pos2(desni_x, 62.0));
        harness.hover_at(pos2(180.0, 62.0));
        harness.drop_at(pos2(180.0, 62.0));
        cekaj_smirenje(&mut harness, Duration::from_millis(500));

        let lijevi_x = harness
            .get_by_label("Premjesti ploču Slojevi")
            .rect()
            .center()
            .x;
        assert!(lijevi_x < 400.0);

        harness.get_by_label("Poligon").click();
        harness.get_by_label("Promijeni temu").click();
        harness.get_by_label("DGU ortofoto").click();
        harness.get_by_label("Vidljivost: Trase").click();
        harness
            .get_by_label("Proširi ili skupi: Parcele 2025")
            .click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));

        assert!(harness.query_by_label("Stare parcele").is_some());

        harness.get_by_label("Poništi").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Stare parcele").is_some());

        harness.get_by_label("Zatvori ploču Slojevi").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Premjesti ploču Slojevi").is_none());

        harness.get_by_label("Slojevi").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        assert!(harness.query_by_label("Premjesti ploču Slojevi").is_some());
    }

    #[test]
    fn rad_s_208_slojeva_i_dubinom_8_mjeri_vrijeme_naseg_koda() {
        let mut app = Aplikacija::default();
        izgradi_stablo_opterecenja(&mut app.uredjivac, &app.katalog);
        assert_eq!(ukupno_slojeva_u_projektu(app.uredjivac.projekt()), 208);
        assert_eq!(najveca_dubina_mapa(app.uredjivac.projekt()), 8);

        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

        harness.get_by_label("Mapa").click();
        cekaj_smirenje(&mut harness, Duration::from_millis(500));
        let zadnje = harness.state().zadnje_vrijeme_okvira;
        // Granica vremena ne ide u obični test: pod opterećenjem pada sama od sebe.
        assert!(
            zadnje > Duration::ZERO,
            "vrijeme okvira mora biti izmjereno"
        );
        assert!(matches!(
            harness.state().stanje.odabrano,
            Some(Odabir::Mapa(_))
        ));
    }

    #[test]
    fn spremanje_slike_zaslona_i_obrada_snimanja() {
        let temp_dir = std::env::temp_dir().join("terrasimple_test_snimka");
        let temp_datoteka = temp_dir.join("test.png");
        let slika = egui::ColorImage::new([2, 2], vec![egui::Color32::RED; 4]);
        spremi_sliku_zaslona(&slika, &temp_datoteka);
        assert!(temp_datoteka.exists());
        let _ = std::fs::remove_dir_all(&temp_dir);

        let app_snimka = Aplikacija::nova(PostavkeAplikacije {
            snimka: Some(temp_datoteka),
            ..Default::default()
        });
        let mut harness = Harness::builder()
            .with_size(vec2(100.0, 100.0))
            .build_ui_state(
                |ui, app: &mut Aplikacija| {
                    app.obradi_snimanje_zaslona(ui);
                },
                app_snimka,
            );
        harness.run_steps(2);
        assert!(harness.state().okvir_brojac >= 2);
    }

    #[test]
    fn prazan_projekt_bez_primjera_nema_izmisljene_podatke() {
        let app = Aplikacija::default();
        assert!(!app.stanje.prikazi_primjer);

        let stavke = odredi_stavke_statusne_trake(&app.stanje, &app.katalog, None);
        assert!(
            stavke.koordinate.is_none(),
            "prazan projekt ne smije imati koordinate"
        );
        assert!(
            stavke.mjerilo.is_none(),
            "prazan projekt ne smije imati mjerilo"
        );
        assert!(
            stavke.izvor_podloge.is_none(),
            "prazan projekt ne smije imati izvor podloge"
        );
        assert!(
            stavke.mreza.is_none(),
            "prazan projekt ne smije imati status mreže"
        );
        assert_eq!(
            stavke.sve_vrijednosti(),
            vec!["HTRS96/TM ˅"],
            "prazan projekt u statusnoj traci prikazuje samo koordinatni sustav"
        );
        assert!(
            naziv_datoteke_status(&app.stanje, &app.katalog).is_none(),
            "prazan projekt ne smije imati naziv datoteke u gornjoj traci"
        );

        let mut harness = Harness::builder()
            .with_size(vec2(Razmaci::SIRINA_PROZORA, Razmaci::VISINA_PROZORA))
            .with_step_dt(0.05)
            .with_context_setup(Aplikacija::postavi_kontekst)
            .build_ui_state(|ui, app: &mut Aplikacija| app.crtaj_sucelje(ui), app);

        cekaj_smirenje(&mut harness, Duration::from_millis(500));

        let tekstovi = nacrtani_tekstovi(&harness);
        for izmisljeno in ["100 m", "© OpenStreetMap suradnici", "Mreža radi"] {
            assert!(
                !tekstovi.iter().any(|t| t == izmisljeno),
                "prazan projekt ne smije crtati „{izmisljeno}”"
            );
        }

        // Čip lokalnog ortofota ne smije postojati u praznom projektu
        assert!(harness.query_by_label("ortofoto_2024.tif").is_none());
        assert!(harness.query_by_label("Karta").is_some());
        assert!(harness.query_by_label("DGU ortofoto").is_some());

        // Provjeri stanje primjera
        assert!(!harness.state().stanje.prikazi_primjer);
    }

    #[test]
    fn primjer_zaslona_a_ima_sve_podatke_statusne_i_gornje_trake() {
        let app = Aplikacija::nova(PostavkeAplikacije {
            primjer: true,
            ..Default::default()
        });
        assert!(app.stanje.prikazi_primjer);

        let stavke = odredi_stavke_statusne_trake(&app.stanje, &app.katalog, None);
        assert_eq!(
            stavke.koordinate.as_deref(),
            Some("E 655 120   N 5 046 380")
        );
        assert_eq!(stavke.sustav, "HTRS96/TM ˅");
        assert_eq!(stavke.mjerilo.as_deref(), Some("1 : 4 500 ˅"));
        assert_eq!(stavke.izvor_podloge.as_deref(), Some("OpenFreeMap"));
        assert_eq!(stavke.mreza.as_deref(), Some("Mreža radi"));
        assert_eq!(
            stavke.sve_vrijednosti(),
            vec![
                "HTRS96/TM ˅",
                "E 655 120   N 5 046 380",
                "1 : 4 500 ˅",
                "OpenFreeMap",
                "Mreža radi"
            ]
        );
        assert_eq!(
            naziv_datoteke_status(&app.stanje, &app.katalog).as_deref(),
            Some("Teren 2026.gpkg · spremljeno 20:41")
        );
    }
}
