use chrono::NaiveDate;
use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
use unic_langid::LanguageIdentifier;

const KATALOG_HR: &str = include_str!("../prijevodi/hr.ftl");

pub struct KatalogPoruka {
    sveznik: FluentBundle<FluentResource>,
}

impl Default for KatalogPoruka {
    fn default() -> Self {
        Self::hrvatski()
    }
}

impl KatalogPoruka {
    #[must_use]
    pub fn hrvatski() -> Self {
        let izvor =
            FluentResource::try_new(KATALOG_HR.to_owned()).expect("neispravan hr.ftl katalog");
        let jezik: LanguageIdentifier = "hr".parse().expect("valjan jezik hr");
        let mut sveznik = FluentBundle::new(vec![jezik]);
        sveznik.set_use_isolating(false);
        sveznik
            .add_resource(izvor)
            .expect("neuspio unos hr.ftl kataloga");
        Self { sveznik }
    }

    #[must_use]
    pub fn tekst(&self, kljuc: &str) -> String {
        self.dohvati(kljuc, None)
    }

    #[must_use]
    pub fn broj_slojeva(&self, broj: usize) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("broj", broj);
        self.dohvati("broj-slojeva", Some(&argumenti))
    }

    #[must_use]
    pub fn broj_objekata(&self, broj: usize) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("broj", broj);
        self.dohvati("broj-objekata", Some(&argumenti))
    }

    /// Poruka s brojem, za oblike uz broj (1 dio, 2 dijela, 5 dijelova).
    #[must_use]
    pub fn s_brojem(&self, kljuc: &str, broj: usize) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("broj", broj);
        self.dohvati(kljuc, Some(&argumenti))
    }

    /// Poruka s gotovim tekstovima, npr. već oblikovanim mjerama.
    #[must_use]
    pub fn s_tekstovima(&self, kljuc: &str, tekstovi: &[(&str, String)]) -> String {
        let mut argumenti = FluentArgs::new();
        for (ime, vrijednost) in tekstovi {
            argumenti.set(*ime, vrijednost.clone());
        }
        self.dohvati(kljuc, Some(&argumenti))
    }

    #[must_use]
    pub fn s_rednim(&self, kljuc: &str, redni: usize) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("redni", redni);
        self.dohvati(kljuc, Some(&argumenti))
    }

    #[must_use]
    pub fn status_koordinate(&self, istok: f64, sjever: f64, decimale: usize) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("istok", broj_s_razmakom(istok, decimale));
        argumenti.set("sjever", broj_s_razmakom(sjever, decimale));
        self.dohvati("status-koordinate", Some(&argumenti))
    }

    #[must_use]
    pub fn broj_nepodrzanih_zapisa(&self, broj: usize) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("broj", broj);
        self.dohvati("broj-nepodrzanih-zapisa", Some(&argumenti))
    }

    #[must_use]
    pub fn upozorenje_brisanja_sloja(&self, naziv: &str, broj: usize) -> String {
        let mut argumenti = FluentArgs::new();
        let broj_tekst = self.broj_nepodrzanih_zapisa(broj);
        argumenti.set("naziv", naziv);
        argumenti.set("broj_zapisa", broj_tekst);
        self.dohvati("brisanje-upozorenje-sloj", Some(&argumenti))
    }

    #[must_use]
    pub fn upozorenje_brisanja_mape(&self, naziv: &str, broj: usize) -> String {
        let mut argumenti = FluentArgs::new();
        let broj_tekst = self.broj_nepodrzanih_zapisa(broj);
        argumenti.set("naziv", naziv);
        argumenti.set("broj_zapisa", broj_tekst);
        self.dohvati("brisanje-upozorenje-mapa", Some(&argumenti))
    }

    #[must_use]
    pub fn upozorenja_otvaranja_poruka(&self, broj: usize) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("broj", broj);
        self.dohvati("upozorenja-broj", Some(&argumenti))
    }

    #[must_use]
    pub fn pokazi_popis_upozorenja(&self, broj: usize) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("broj", broj);
        self.dohvati("upozorenja-pokazi-popis", Some(&argumenti))
    }

    #[must_use]
    pub fn status_datoteke_spremljeno(&self, naziv: &str, vrijeme: &str) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("naziv", naziv);
        argumenti.set("vrijeme", vrijeme);
        self.dohvati("datoteka-status-spremljeno", Some(&argumenti))
    }

    #[must_use]
    pub fn status_datoteke_izmijenjeno(&self, naziv: &str) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("naziv", naziv);
        self.dohvati("datoteka-status-izmijenjeno", Some(&argumenti))
    }

    #[must_use]
    pub fn status_datoteke_spremljeno_izmijenjeno(&self, naziv: &str, vrijeme: &str) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("naziv", naziv);
        argumenti.set("vrijeme", vrijeme);
        self.dohvati("datoteka-status-spremljeno-izmijenjeno", Some(&argumenti))
    }

    #[must_use]
    pub fn greska_spremanja(
        &self,
        greska: &(dyn std::error::Error + Send + Sync + 'static),
    ) -> String {
        if greska.is::<geopaket::DatotekaPromijenjena>() {
            self.tekst("greska-sukob-promjena")
        } else {
            let mut argumenti = FluentArgs::new();
            argumenti.set("detalji", greska.to_string());
            self.dohvati("greska-spremanja-opcenito", Some(&argumenti))
        }
    }

    #[must_use]
    pub fn greska_otvaranja(&self, detalji: &str) -> String {
        let mut argumenti = FluentArgs::new();
        argumenti.set("detalji", detalji);
        self.dohvati("greska-otvaranja-opcenito", Some(&argumenti))
    }

    fn dohvati(&self, kljuc: &str, argumenti: Option<&FluentArgs<'_>>) -> String {
        let poruka = self
            .sveznik
            .get_message(kljuc)
            .unwrap_or_else(|| panic!("nedostaje poruka u katalogu: {kljuc}"));
        let uzorak = poruka
            .value()
            .unwrap_or_else(|| panic!("poruka nema vrijednost: {kljuc}"));
        let mut greske = Vec::new();
        let rezultat = self
            .sveznik
            .format_pattern(uzorak, argumenti, &mut greske)
            .into_owned();
        assert!(
            greske.is_empty(),
            "greška pri oblikovanju poruke {kljuc}: {greske:?}"
        );
        rezultat
    }
}

#[must_use]
pub fn broj(vrijednost: f64, decimale: usize) -> String {
    let sirovo = format!("{vrijednost:.decimale$}");
    if decimale == 0 {
        return sirovo;
    }
    sirovo.replace('.', ",")
}

#[must_use]
pub fn broj_s_razmakom(vrijednost: f64, decimale: usize) -> String {
    let osnovno = broj(vrijednost.abs(), decimale);
    let predznak = if vrijednost.is_sign_negative() {
        "-"
    } else {
        ""
    };
    let (cijeli, ostatak) = osnovno
        .split_once(',')
        .map_or((osnovno.as_str(), None), |(c, d)| (c, Some(d)));
    let grupirani = grupiraj_tisucice(cijeli);
    match ostatak {
        Some(dec) => format!("{predznak}{grupirani},{dec}"),
        None => format!("{predznak}{grupirani}"),
    }
}

fn grupiraj_tisucice(znamenke: &str) -> String {
    let duljina = znamenke.len();
    let mut izlaz = String::with_capacity(duljina + duljina / 3);
    for (indeks, znak) in znamenke.chars().enumerate() {
        if indeks > 0 && (duljina - indeks).is_multiple_of(3) {
            izlaz.push(' ');
        }
        izlaz.push(znak);
    }
    izlaz
}

#[must_use]
pub fn mjera_povrsine(cetvorni_metri: f64) -> String {
    if cetvorni_metri >= 10_000.0 {
        let hektari = cetvorni_metri / 10_000.0;
        format!("{} ha", broj(hektari, 2))
    } else {
        format!("{} m²", broj(cetvorni_metri, 0))
    }
}

#[must_use]
pub fn mjera_duljine(metri: f64) -> String {
    if metri >= 1_000.0 {
        format!("{} km", broj(metri / 1_000.0, 2))
    } else {
        format!("{} m", broj(metri, 0))
    }
}

#[must_use]
pub fn postotak(vrijednost: u32) -> String {
    format!("{vrijednost} %")
}

#[must_use]
pub fn datum(d: NaiveDate) -> String {
    d.format("%d.%m.%Y.").to_string()
}

#[must_use]
pub fn vrijeme(sat: u32, minuta: u32) -> String {
    format!("{sat:02}:{minuta:02}")
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn mnozina_za_objekte_i_slojeve_slijedi_hrvatska_pravila() {
        let katalog = KatalogPoruka::hrvatski();
        assert_eq!(katalog.broj_objekata(1), "1 objekt");
        assert_eq!(katalog.broj_objekata(2), "2 objekta");
        assert_eq!(katalog.broj_objekata(4), "4 objekta");
        assert_eq!(katalog.broj_objekata(5), "5 objekata");
        assert_eq!(katalog.broj_objekata(11), "11 objekata");
        assert_eq!(katalog.broj_objekata(12), "12 objekata");
        assert_eq!(katalog.broj_objekata(21), "21 objekt");
        assert_eq!(katalog.broj_objekata(23), "23 objekta");

        assert_eq!(katalog.broj_slojeva(1), "1 sloj");
        assert_eq!(katalog.broj_slojeva(2), "2 sloja");
        assert_eq!(katalog.broj_slojeva(5), "5 slojeva");
    }

    #[test]
    fn oblikovanje_brojeva_koristi_decimalni_zarez_i_razmake() {
        assert_eq!(broj(4.83, 2), "4,83");
        assert_eq!(broj(827.0, 0), "827");
        assert_eq!(broj_s_razmakom(459_382.14, 2), "459 382,14");
        assert_eq!(broj_s_razmakom(5_074_918.62, 2), "5 074 918,62");
        assert_eq!(broj_s_razmakom(655_120.0, 0), "655 120");
        assert_eq!(broj_s_razmakom(5_046_380.0, 0), "5 046 380");
        assert_eq!(broj_s_razmakom(-1_250.0, 0), "-1 250");
        assert_eq!(mjera_povrsine(48_300.0), "4,83 ha");
        assert_eq!(mjera_povrsine(827.0), "827 m²");
        assert_eq!(mjera_duljine(128.0), "128 m");
        assert_eq!(mjera_duljine(2_500.0), "2,50 km");
        assert_eq!(postotak(100), "100 %");
    }

    #[test]
    fn oblikovanje_datuma_vremena_i_statusne_trake() {
        let katalog = KatalogPoruka::default();
        let d = NaiveDate::from_ymd_opt(2026, 11, 7).unwrap();
        assert_eq!(datum(d), "07.11.2026.");
        assert_eq!(vrijeme(14, 32), "14:32");
        assert_eq!(
            katalog.status_koordinate(655_120.0, 5_046_380.0, 0),
            "E 655 120   N 5 046 380"
        );
        assert_eq!(katalog.s_rednim("novi-sloj-naziv", 3), "Sloj 3");
        assert_eq!(katalog.tekst("naslov-aplikacije"), "TerraSimple");
    }

    #[test]
    fn oblikovanje_poruka_za_datoteku_i_brisanje() {
        let katalog = KatalogPoruka::hrvatski();
        assert_eq!(katalog.broj_nepodrzanih_zapisa(1), "1 zapis");
        assert_eq!(katalog.broj_nepodrzanih_zapisa(2), "2 zapisa");
        assert_eq!(katalog.broj_nepodrzanih_zapisa(5), "5 zapisa");
        assert_eq!(
            katalog.upozorenje_brisanja_sloja("Parcele", 5),
            "Sloj „Parcele” ima 5 zapisa koje program ne prikazuje, npr. krivulje ili prazne geometrije. Kad spremiš projekt, nestat će i oni."
        );
        assert_eq!(
            katalog.upozorenje_brisanja_mape("Uvezeno", 1),
            "Mapa „Uvezeno” ima 1 zapis koje program ne prikazuje, npr. krivulje ili prazne geometrije. Kad spremiš projekt, nestat će i oni."
        );
        assert_eq!(
            katalog.upozorenja_otvaranja_poruka(1),
            "Pri otvaranju je zabilježeno 1 upozorenje."
        );
        assert_eq!(
            katalog.upozorenja_otvaranja_poruka(3),
            "Pri otvaranju su zabilježena 3 upozorenja."
        );
        assert_eq!(
            katalog.upozorenja_otvaranja_poruka(5),
            "Pri otvaranju je zabilježeno 5 upozorenja."
        );
        assert_eq!(
            katalog.status_datoteke_spremljeno("teren.gpkg", "14:20"),
            "teren.gpkg · spremljeno 14:20"
        );
        assert_eq!(
            katalog.status_datoteke_izmijenjeno("teren.gpkg"),
            "teren.gpkg · nespremljeno"
        );
        assert_eq!(
            katalog.status_datoteke_spremljeno_izmijenjeno("teren.gpkg", "14:20"),
            "teren.gpkg · spremljeno 14:20 (nespremljeno)"
        );
        assert_eq!(
            katalog.greska_spremanja(&geopaket::DatotekaPromijenjena),
            "Projekt nije spremljen: datoteku je u međuvremenu promijenio drugi program, npr. QGIS. Spremi svoj crtež kao novu datoteku. Datoteka koju je promijenio drugi program ostaje kakva jest."
        );
        assert!(
            katalog
                .greska_spremanja(&std::io::Error::other("disk je pun"))
                .contains("disk je pun")
        );
        assert!(
            katalog
                .greska_otvaranja("kriva datoteka")
                .contains("kriva datoteka")
        );
    }
}
