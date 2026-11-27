use crate::izdanja::{Izdanje, na_stranici_izdanja};
use crate::{Datoteka, Greska};
use base64::Engine;
use std::io::Read;
use std::time::Duration;

pub(crate) const NAJVECI_PAKET: u64 = 200 << 20;
pub(crate) const NAJVECI_POTPIS: u64 = 1 << 10;
const NAJVECI_POPIS: u64 = 4 << 20;
const ROK_POPISA: Duration = Duration::from_secs(20);
/// Ukupni rok preuzimanja dovoljan je i za sporu vezu (25 MB uz 0,5 Mbit/s je oko 7 minuta).
const ROK_PREUZIMANJA: Duration = Duration::from_secs(15 * 60);

/// Javni par ključa kojim se potpisuju paketi izdanja. Tajni je samo na računalu s kojeg se
/// izdaje (vidi docs/izdanje.md).
const JAVNI_KLJUC: &str = "MRvgiTjC7VdQQtizAtcTNYifEccmExC+g93LQDRE2H4=";

/// Odakle se čitaju izdanja i preuzimaju paketi. Testovi koriste lokalni poslužitelj.
#[derive(Clone, Debug)]
pub struct Izvor {
    pub popis: String,
    pub prefiks: String,
    pub javni_kljuc: Vec<u8>,
}

impl Izvor {
    /// Izdanja TerraSimplea na GitHubu.
    #[must_use]
    pub fn github() -> Self {
        Self {
            popis: "https://api.github.com/repos/tkraljevic/TerraSimple/releases?per_page=20"
                .into(),
            prefiks: "https://github.com/tkraljevic/TerraSimple/releases/download/".into(),
            javni_kljuc: base64::engine::general_purpose::STANDARD
                .decode(JAVNI_KLJUC)
                .unwrap_or_default(),
        }
    }

    /// Izvor za program. `TERRASIMPLE_NADOGRADNJA_IZVOR` (npr. `http://127.0.0.1:8000`) služi
    /// za probu nadogradnje bez objave izdanja; ključ ostaje naš, pa paket i dalje mora biti
    /// potpisan.
    #[must_use]
    pub fn za_program(okolina: Option<&str>) -> Self {
        match okolina {
            Some(osnova) => Self {
                popis: format!("{osnova}/releases"),
                prefiks: format!("{osnova}/download/"),
                ..Self::github()
            },
            None => Self::github(),
        }
    }

    pub(crate) fn izdanja(&self) -> Result<Vec<Izdanje>, Greska> {
        let tijelo = dohvati(&self.popis, NAJVECI_POPIS, ROK_POPISA)?;
        serde_json::from_slice(&tijelo).map_err(crate::greska::neocekivano)
    }

    pub(crate) fn preuzmi(&self, datoteka: &Datoteka, najvise: u64) -> Result<Vec<u8>, Greska> {
        if !na_stranici_izdanja(&datoteka.adresa, &self.prefiks) {
            return Err(Greska::TudaAdresa(datoteka.adresa.clone()));
        }
        if datoteka.velicina > najvise {
            return Err(prevelika(datoteka, najvise));
        }
        dohvati(&datoteka.adresa, najvise, ROK_PREUZIMANJA).map_err(|e| match e {
            Greska::Prevelika { .. } => prevelika(datoteka, najvise),
            e => e,
        })
    }
}

fn prevelika(datoteka: &Datoteka, najvise: u64) -> Greska {
    Greska::Prevelika {
        ime: datoteka.ime.clone(),
        najvise,
    }
}

fn veza(e: impl std::fmt::Display) -> Greska {
    Greska::Veza(e.to_string())
}

/// GitHub traži `User-Agent`; bez njega odbija zahtjev.
fn dohvati(adresa: &str, najvise: u64, rok: Duration) -> Result<Vec<u8>, Greska> {
    let odgovor = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_secs(15))
        .timeout(rok)
        .user_agent("TerraSimple-nadogradnja")
        .build()
        .map_err(veza)?
        .get(adresa)
        .send()
        .map_err(veza)?;
    if !odgovor.status().is_success() {
        return Err(Greska::Odgovor(odgovor.status().as_u16()));
    }
    let mut tijelo = Vec::new();
    odgovor
        .take(najvise + 1)
        .read_to_end(&mut tijelo)
        .map_err(veza)?;
    if tijelo.len() as u64 > najvise {
        return Err(Greska::Prevelika {
            ime: adresa.into(),
            najvise,
        });
    }
    Ok(tijelo)
}
