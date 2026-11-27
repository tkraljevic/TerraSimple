//! Prozorčić „Nadogradnja” iz izbornika Pomoć (Odluke (27. 11. 2026.)). GitHub se pita samo
//! kad korisnik to zatraži. Provjera i preuzimanje rade u zasebnoj niti, pa prozor ne zastaje.

use crate::poruke::KatalogPoruka;
use eframe::egui;
use nadogradnja::{Greska, Inacica, Ishod, Izvor, Mjesto, Ponuda, Sustav};
use std::sync::mpsc::{Receiver, channel};

/// Što prozorčić trenutno pokazuje.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StanjeNadogradnje {
    Provjera,
    Najnovija(Inacica),
    /// Novije izdanje; `moze_sam` kaže može li se program sam zamijeniti.
    Dostupna {
        ponuda: Ponuda,
        moze_sam: bool,
    },
    BezPaketa {
        inacica: Inacica,
        stranica: String,
    },
    Preuzimanje(Ponuda),
    Instalirana(Inacica),
    Greska {
        poruka: String,
        stranica: Option<String>,
    },
}

/// Gumbi prozorčića, s lijeva na desno; zadnji je istaknut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GumbNadogradnje {
    Zatvori,
    Kasnije,
    StranicaIzdanja,
    Instaliraj,
    PonovnoPokreni,
}

enum Poruka {
    Provjera(Result<Ishod, Greska>),
    Ugradnja(Ponuda, Result<(), Greska>),
}

/// Nit koja radi i program koji se pokreće nakon zatvaranja.
pub struct Nadogradnja {
    izvor: Izvor,
    trenutna: Inacica,
    mjesto: Option<Mjesto>,
    primatelj: Option<Receiver<Poruka>>,
    pokreni_nakon_izlaska: Option<Mjesto>,
}

pub fn trenutna_inacica() -> Inacica {
    Inacica::procitaj(env!("CARGO_PKG_VERSION")).unwrap_or(Inacica(0, 0, 0))
}

impl Nadogradnja {
    /// Program iz paketa izdanja, s izvorom iz `TERRASIMPLE_NADOGRADNJA_IZVOR` ako je zadan.
    #[must_use]
    pub fn za_program() -> Self {
        let okolina = std::env::var("TERRASIMPLE_NADOGRADNJA_IZVOR").ok();
        let mjesto = std::env::current_exe()
            .ok()
            .and_then(|exe| Mjesto::za_program(&exe, Sustav::trenutni()).ok());
        Self::nova(
            Izvor::za_program(okolina.as_deref()),
            trenutna_inacica(),
            mjesto,
        )
    }

    #[must_use]
    pub fn nova(izvor: Izvor, trenutna: Inacica, mjesto: Option<Mjesto>) -> Self {
        Self {
            izvor,
            trenutna,
            mjesto,
            primatelj: None,
            pokreni_nakon_izlaska: None,
        }
    }

    fn pokreni(&mut self, ctx: &egui::Context, posao: impl FnOnce() -> Poruka + Send + 'static) {
        let (posiljatelj, primatelj) = channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = posiljatelj.send(posao()); // prozor je možda već zatvoren
            ctx.request_repaint();
        });
        self.primatelj = Some(primatelj);
    }

    /// Pokreće provjeru i vraća stanje koje prozorčić pokazuje dok čeka.
    pub fn provjeri(&mut self, ctx: &egui::Context) -> StanjeNadogradnje {
        let (izvor, trenutna) = (self.izvor.clone(), self.trenutna);
        self.pokreni(ctx, move || {
            Poruka::Provjera(nadogradnja::provjeri(&izvor, trenutna, Sustav::trenutni()))
        });
        StanjeNadogradnje::Provjera
    }

    /// Preuzima i ugrađuje ponudu. Bez mjesta programa ne radi ništa.
    pub fn ugradi(&mut self, ctx: &egui::Context, ponuda: Ponuda) -> Option<StanjeNadogradnje> {
        let mjesto = self.mjesto.clone()?;
        let izvor = self.izvor.clone();
        let stanje = StanjeNadogradnje::Preuzimanje(ponuda.clone());
        self.pokreni(ctx, move || {
            let ishod = nadogradnja::preuzmi_i_ugradi(&izvor, &ponuda, &mjesto);
            Poruka::Ugradnja(ponuda, ishod)
        });
        Some(stanje)
    }

    /// Novo stanje kad nit završi; inače `None`.
    pub fn primi(&mut self, katalog: &KatalogPoruka) -> Option<StanjeNadogradnje> {
        let poruka = self.primatelj.as_ref()?.try_recv().ok()?;
        self.primatelj = None;
        Some(match poruka {
            Poruka::Provjera(ishod) => self.stanje_provjere(ishod, katalog),
            Poruka::Ugradnja(ponuda, Ok(())) => StanjeNadogradnje::Instalirana(ponuda.inacica),
            Poruka::Ugradnja(ponuda, Err(e)) => greska(&e, Some(ponuda.stranica), katalog),
        })
    }

    fn stanje_provjere(
        &self,
        ishod: Result<Ishod, Greska>,
        katalog: &KatalogPoruka,
    ) -> StanjeNadogradnje {
        match ishod {
            Ok(Ishod::Najnovija(v)) => StanjeNadogradnje::Najnovija(v),
            Ok(Ishod::Dostupna(ponuda)) => StanjeNadogradnje::Dostupna {
                ponuda,
                moze_sam: self.mjesto.is_some(),
            },
            Ok(Ishod::BezPaketa { inacica, stranica }) => {
                StanjeNadogradnje::BezPaketa { inacica, stranica }
            }
            Err(e) => greska(&e, None, katalog),
        }
    }

    /// Nakon „Ponovno pokreni” novi program kreće tek kad se ovaj zatvori, pa pitanje o
    /// nespremljenim izmjenama ostaje kakvo jest.
    pub fn pokreni_nakon_izlaska(&mut self) {
        self.pokreni_nakon_izlaska.clone_from(&self.mjesto);
    }

    /// Poziva se pri izlasku iz programa.
    pub fn na_izlasku(&mut self) {
        if let Some(mjesto) = self.pokreni_nakon_izlaska.take() {
            let _ = nadogradnja::naredba_ponovnog_pokretanja(&mjesto).spawn(); // korisnik ga može pokrenuti i sam
        }
    }

    /// Briše ostatke prethodne nadogradnje (na Windowsu stari program, koji se nije mogao obrisati).
    pub fn pocisti(&self) {
        if let Some(mjesto) = &self.mjesto {
            nadogradnja::pocisti_ostatke(mjesto);
        }
    }
}

fn velicina(bajtova: u64) -> String {
    format!(
        "{} MB",
        crate::poruke::broj(bajtova as f64 / 1_000_000.0, 1)
    )
}

fn greska(e: &Greska, stranica: Option<String>, katalog: &KatalogPoruka) -> StanjeNadogradnje {
    let poruka = match e {
        Greska::Veza(_) => katalog.tekst("nadogradnja-greska-veza"),
        Greska::Odgovor(status) => katalog.s_tekstovima(
            "nadogradnja-greska-odgovor",
            &[("status", status.to_string())],
        ),
        Greska::NeocekivanOdgovor(_) => katalog.tekst("nadogradnja-greska-neocekivano"),
        Greska::Prevelika { ime, .. } => {
            katalog.s_tekstovima("nadogradnja-greska-prevelika", &[("ime", ime.clone())])
        }
        Greska::TudaAdresa(_) => katalog.tekst("nadogradnja-greska-adresa"),
        Greska::KrivPotpis => katalog.tekst("nadogradnja-greska-potpis"),
        Greska::OstecenPaket(_) => katalog.tekst("nadogradnja-greska-paket"),
        Greska::NijeIzPaketa(_) => katalog.tekst("nadogradnja-greska-nije-paket"),
        Greska::Zamjena { mjesto, detalji } => katalog.s_tekstovima(
            "nadogradnja-greska-zamjena",
            &[
                ("mjesto", mjesto.display().to_string()),
                ("detalji", detalji.clone()),
            ],
        ),
    };
    StanjeNadogradnje::Greska { poruka, stranica }
}

/// Tekst prozorčića za stanje.
pub fn poruka(stanje: &StanjeNadogradnje, trenutna: Inacica, katalog: &KatalogPoruka) -> String {
    let nova = |v: &Inacica| ("nova", v.to_string());
    match stanje {
        StanjeNadogradnje::Provjera => katalog.tekst("nadogradnja-provjera"),
        StanjeNadogradnje::Najnovija(v) => {
            katalog.s_tekstovima("nadogradnja-najnovija", &[("inacica", v.to_string())])
        }
        StanjeNadogradnje::Dostupna { ponuda, moze_sam } => katalog.s_tekstovima(
            if *moze_sam {
                "nadogradnja-dostupna"
            } else {
                "nadogradnja-dostupna-rucno"
            },
            &[
                nova(&ponuda.inacica),
                ("trenutna", trenutna.to_string()),
                ("velicina", velicina(ponuda.paket.velicina)),
            ],
        ),
        StanjeNadogradnje::BezPaketa { inacica, .. } => {
            katalog.s_tekstovima("nadogradnja-bez-paketa", &[nova(inacica)])
        }
        StanjeNadogradnje::Preuzimanje(p) => {
            katalog.s_tekstovima("nadogradnja-preuzimanje", &[nova(&p.inacica)])
        }
        StanjeNadogradnje::Instalirana(v) => {
            katalog.s_tekstovima("nadogradnja-instalirana", &[nova(v)])
        }
        StanjeNadogradnje::Greska { poruka, .. } => poruka.clone(),
    }
}

/// Gumbi za stanje. Dok nit radi, gumba nema: prekinuta zamjena bila bi gora od čekanja.
#[must_use]
pub fn gumbi(stanje: &StanjeNadogradnje) -> &'static [GumbNadogradnje] {
    use GumbNadogradnje as G;
    match stanje {
        StanjeNadogradnje::Provjera | StanjeNadogradnje::Preuzimanje(_) => &[],
        StanjeNadogradnje::Najnovija(_) => &[G::Zatvori],
        StanjeNadogradnje::Dostupna { moze_sam: true, .. } => {
            &[G::Kasnije, G::StranicaIzdanja, G::Instaliraj]
        }
        StanjeNadogradnje::Dostupna { .. }
        | StanjeNadogradnje::BezPaketa { .. }
        | StanjeNadogradnje::Greska {
            stranica: Some(_), ..
        } => &[G::Zatvori, G::StranicaIzdanja],
        StanjeNadogradnje::Greska { .. } => &[G::Zatvori],
        StanjeNadogradnje::Instalirana(_) => &[G::Kasnije, G::PonovnoPokreni],
    }
}

/// Stranica izdanja za gumb „Stranica izdanja”.
#[must_use]
pub fn stranica(stanje: &StanjeNadogradnje) -> Option<&str> {
    match stanje {
        StanjeNadogradnje::Dostupna { ponuda, .. } => Some(&ponuda.stranica),
        StanjeNadogradnje::BezPaketa { stranica, .. } => Some(stranica),
        StanjeNadogradnje::Greska { stranica, .. } => stranica.as_deref(),
        _ => None,
    }
}

#[must_use]
pub const fn kljuc_gumba(g: GumbNadogradnje) -> &'static str {
    match g {
        GumbNadogradnje::Zatvori => "nadogradnja-zatvori",
        GumbNadogradnje::Kasnije => "nadogradnja-kasnije",
        GumbNadogradnje::StranicaIzdanja => "nadogradnja-stranica",
        GumbNadogradnje::Instaliraj => "nadogradnja-instaliraj",
        GumbNadogradnje::PonovnoPokreni => "nadogradnja-ponovno",
    }
}

/// Otvara stranicu izdanja u pregledniku. Prima samo web adrese.
pub fn otvori_stranicu(adresa: &str) {
    if !(adresa.starts_with("https://") || adresa.starts_with("http://127.0.0.1")) {
        return;
    }
    let mut naredba = naredba_preglednika();
    let _ = naredba.arg(adresa).spawn(); // bez preglednika adresa ostaje u poruci izdanja
}

#[cfg(target_os = "macos")]
fn naredba_preglednika() -> std::process::Command {
    std::process::Command::new("open")
}

#[cfg(windows)]
fn naredba_preglednika() -> std::process::Command {
    let mut n = std::process::Command::new("rundll32");
    n.arg("url.dll,FileProtocolHandler");
    n
}

#[cfg(not(any(target_os = "macos", windows)))]
fn naredba_preglednika() -> std::process::Command {
    std::process::Command::new("xdg-open")
}

#[cfg(test)]
#[path = "nadogradnja_prozora_testovi.rs"]
mod testovi;
