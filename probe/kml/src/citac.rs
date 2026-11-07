//! Čitač KML i KMZ datoteka temeljen na `quick-xml` 0.42 i `zip` 9.4.
//!
//! Sigurnosna pravila (7. 11. 2026.):
//! - Poruke o greškama nikad ne sadrže tekst, nazive, opise ni koordinate iz datoteke.
//! - KMZ čitač odbija putanje s `..`, vodećim `/` ili `\` te prekida čitanje čim
//!   dekomprimirana veličina ili omjer kompresije prijeđu granicu (zaštita od ZIP bombe).
//! - Svaki element koji nije dio 2D vektorskih slojeva ili nije prepoznat broji se po imenu.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::model::{
    AtributPodatka, BojaRgba, Geometrija, Mapa, MapaStila, MreznaVeza, NacinVisine, Objekt,
    ParStila, PogledKamere, RijeseniStil, SlikaArhive, SlikovniPrekrivac, StabloKml, StatistikaKml,
    Stil, StilBalona, StilCrte, StilIkone, StilNatpisa, StilPoligona, Vrh, VrijemeKml, VrstaMape,
};

/// Ograničenja za sigurno čitanje nepouzdanih KML i KMZ datoteka.
#[derive(Clone, Copy, Debug)]
pub struct OgranicenjaCitaca {
    pub najveca_dubina: usize,
    pub najvise_stavki_u_arhivi: usize,
    pub najveca_velicina_iz_arhive: u64,
    pub najveci_omjer_kompresije: u64,
}

impl Default for OgranicenjaCitaca {
    fn default() -> Self {
        Self {
            najveca_dubina: 64,
            najvise_stavki_u_arhivi: 10_000,
            najveca_velicina_iz_arhive: 256 * 1024 * 1024,
            najveci_omjer_kompresije: 100,
        }
    }
}

/// Greška čitanja KML/KMZ datoteke — bez citiranja sadržaja datoteke radi privatnosti.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GreskaCitaca {
    UlazIzlaz,
    NeispravanXml {
        pozicija_bajta: u64,
    },
    NeocekivanZavrsetakOznake {
        pozicija_bajta: u64,
    },
    PrevelikaDubinaStabla {
        dopusteno: usize,
    },
    NeispravnaKoordinata {
        pozicija_bajta: u64,
    },
    PraznaGeometrija {
        oznaka: &'static str,
        pozicija_bajta: u64,
    },
    NeispravnaArhiva,
    NemaKmlUArhivi,
    NeocekivanaPutanjaUArhivi,
    PrevelikaDatotekaUArhivi {
        dopusteno_bajta: u64,
    },
    PreviseStavkiUArhivi {
        dopusteno: usize,
    },
}

impl fmt::Display for GreskaCitaca {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UlazIzlaz => write!(f, "greška čitanja sa spremnika"),
            Self::NeispravanXml { pozicija_bajta } => {
                write!(f, "neispravan XML na bajtu {pozicija_bajta}")
            }
            Self::NeocekivanZavrsetakOznake { pozicija_bajta } => {
                write!(f, "neočekivan završetak XML oznake na bajtu {pozicija_bajta}")
            }
            Self::PrevelikaDubinaStabla { dopusteno } => {
                write!(f, "dubina stabla prelazi dopuštenih {dopusteno} razina")
            }
            Self::NeispravnaKoordinata { pozicija_bajta } => {
                write!(f, "neispravna koordinata na bajtu {pozicija_bajta}")
            }
            Self::PraznaGeometrija {
                oznaka,
                pozicija_bajta,
            } => write!(f, "prazna geometrija {oznaka} na bajtu {pozicija_bajta}"),
            Self::NeispravnaArhiva => write!(f, "neispravna KMZ (ZIP) arhiva"),
            Self::NemaKmlUArhivi => write!(f, "u KMZ arhivi nema KML datoteke"),
            Self::NeocekivanaPutanjaUArhivi => {
                write!(f, "KMZ arhiva sadrži nedopuštenu putanju")
            }
            Self::PrevelikaDatotekaUArhivi { dopusteno_bajta } => write!(
                f,
                "sadržaj KMZ arhive prelazi sigurnosnu granicu ({dopusteno_bajta} B ili omjer kompresije)"
            ),
            Self::PreviseStavkiUArhivi { dopusteno } => {
                write!(f, "KMZ arhiva ima više od {dopusteno} datoteka")
            }
        }
    }
}

/// Pomoćni čitač koji prati broj dekomprimiranih bajtova i omjer kompresije tijekom čitanja iz ZIP-a.
struct OgraniceniUnos<'a, R: Read> {
    unutarnji: R,
    komprimirano_bajta: u64,
    procitano_stavke: u64,
    ukupno_procitano: &'a mut u64,
    ogranicenja: OgranicenjaCitaca,
    prekoraceno: bool,
}

impl<R: Read> Read for OgraniceniUnos<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.unutarnji.read(buf)?;
        self.procitano_stavke += n as u64;
        *self.ukupno_procitano += n as u64;
        let preveliko = *self.ukupno_procitano > self.ogranicenja.najveca_velicina_iz_arhive;
        let prevelik_omjer = self.procitano_stavke > 1_048_576
            && self.procitano_stavke / self.komprimirano_bajta.max(1)
                > self.ogranicenja.najveci_omjer_kompresije;
        if preveliko || prevelik_omjer {
            self.prekoraceno = true;
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "prekoračena granica dekompresije",
            ));
        }
        Ok(n)
    }
}

fn je_sigurna_putanja_u_arhivi(ime: &str) -> bool {
    if ime.is_empty() || ime.starts_with('/') || ime.starts_with('\\') || ime.contains('\\') {
        return false;
    }
    if ime.contains(':') {
        return false;
    }
    for dio in ime.split('/') {
        if dio == ".." || (dio.is_empty() && ime.ends_with("//")) {
            return false;
        }
    }
    true
}

fn je_slika(ime: &str) -> bool {
    let malo = ime.to_ascii_lowercase();
    [
        ".png", ".jpg", ".jpeg", ".gif", ".bmp", ".webp", ".svg", ".tif", ".tiff",
    ]
    .iter()
    .any(|nastavak| malo.ends_with(nastavak))
}

/// Pročita KML ili KMZ datoteku sa zadanim sigurnosnim ograničenjima.
pub fn procitaj_datoteku(put: &Path) -> Result<StabloKml, GreskaCitaca> {
    procitaj_datoteku_uz_ogranicenja(put, OgranicenjaCitaca::default())
}

pub fn procitaj_datoteku_uz_ogranicenja(
    put: &Path,
    ogranicenja: OgranicenjaCitaca,
) -> Result<StabloKml, GreskaCitaca> {
    let je_kmz_po_imenu = put
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("kmz"));
    let mut dat = File::open(put).map_err(|_| GreskaCitaca::UlazIzlaz)?;
    let mut zaglavlje = [0u8; 4];
    let n = dat.read(&mut zaglavlje).map_err(|_| GreskaCitaca::UlazIzlaz)?;
    let je_zip_potpis = n == 4 && &zaglavlje == b"PK\x03\x04";
    if je_kmz_po_imenu || je_zip_potpis {
        procitaj_kmz(put, ogranicenja)
    } else {
        let dat = File::open(put).map_err(|_| GreskaCitaca::UlazIzlaz)?;
        let citac = BufReader::with_capacity(64 * 1024, dat);
        procitaj_kml_iz_citaca(citac, ogranicenja)
    }
}

fn procitaj_kmz(put: &Path, ogranicenja: OgranicenjaCitaca) -> Result<StabloKml, GreskaCitaca> {
    let dat = File::open(put).map_err(|_| GreskaCitaca::UlazIzlaz)?;
    let mut arhiva = zip::ZipArchive::new(dat).map_err(|_| GreskaCitaca::NeispravnaArhiva)?;
    if arhiva.len() > ogranicenja.najvise_stavki_u_arhivi {
        return Err(GreskaCitaca::PreviseStavkiUArhivi {
            dopusteno: ogranicenja.najvise_stavki_u_arhivi,
        });
    }

    let mut ukupno_procitano = 0u64;
    let mut kml_indeks: Option<usize> = None;
    let mut slike = Vec::new();

    for i in 0..arhiva.len() {
        let mut stavka = arhiva
            .by_index(i)
            .map_err(|_| GreskaCitaca::NeispravnaArhiva)?;
        let sirovo_ime = stavka
            .name()
            .map_err(|_| GreskaCitaca::NeocekivanaPutanjaUArhivi)?
            .into_owned();
        if !je_sigurna_putanja_u_arhivi(&sirovo_ime) || stavka.enclosed_name().is_none() {
            return Err(GreskaCitaca::NeocekivanaPutanjaUArhivi);
        }
        if stavka.is_dir() {
            continue;
        }
        let komprimirano = stavka.compressed_size();
        let malo = sirovo_ime.to_ascii_lowercase();
        if malo.ends_with(".kml") {
            if kml_indeks.is_none() || malo == "doc.kml" || malo.ends_with("/doc.kml") {
                kml_indeks = Some(i);
            }
        } else if je_slika(&malo) {
            let mut ogr_citac = OgraniceniUnos {
                unutarnji: &mut stavka,
                komprimirano_bajta: komprimirano,
                procitano_stavke: 0,
                ukupno_procitano: &mut ukupno_procitano,
                ogranicenja,
                prekoraceno: false,
            };
            let mut spremnik = Vec::new();
            if ogr_citac.read_to_end(&mut spremnik).is_err() {
                if ogr_citac.prekoraceno {
                    return Err(GreskaCitaca::PrevelikaDatotekaUArhivi {
                        dopusteno_bajta: ogranicenja.najveca_velicina_iz_arhive,
                    });
                }
                return Err(GreskaCitaca::NeispravnaArhiva);
            }
            slike.push(SlikaArhive {
                putanja: sirovo_ime,
                velicina_bajta: spremnik.len(),
            });
        }
    }

    let Some(indeks) = kml_indeks else {
        return Err(GreskaCitaca::NemaKmlUArhivi);
    };
    let mut kml_stavka = arhiva
        .by_index(indeks)
        .map_err(|_| GreskaCitaca::NeispravnaArhiva)?;
    let komprimirano = kml_stavka.compressed_size();
    let ogr_citac = OgraniceniUnos {
        unutarnji: &mut kml_stavka,
        komprimirano_bajta: komprimirano,
        procitano_stavke: 0,
        ukupno_procitano: &mut ukupno_procitano,
        ogranicenja,
        prekoraceno: false,
    };
    let buf_citac = BufReader::with_capacity(64 * 1024, ogr_citac);
    let mut stablo = match procitaj_kml_iz_citaca(buf_citac, ogranicenja) {
        Ok(s) => s,
        Err(GreskaCitaca::NeispravanXml { .. } | GreskaCitaca::UlazIzlaz)
            if ukupno_procitano > ogranicenja.najveca_velicina_iz_arhive
                || (ukupno_procitano > 1_048_576
                    && ukupno_procitano / komprimirano.max(1)
                        > ogranicenja.najveci_omjer_kompresije) =>
        {
            return Err(GreskaCitaca::PrevelikaDatotekaUArhivi {
                dopusteno_bajta: ogranicenja.najveca_velicina_iz_arhive,
            });
        }
        Err(g) => return Err(g),
    };
    stablo.statistika.slike_u_kmz = slike.len();
    stablo.slike_u_arhivi = slike;
    Ok(stablo)
}

pub fn procitaj_kml_iz_teksta(tekst: &str) -> Result<StabloKml, GreskaCitaca> {
    procitaj_kml_iz_citaca(tekst.as_bytes(), OgranicenjaCitaca::default())
}

struct KontekstCitaca<B: BufRead> {
    xml: Reader<B>,
    buf: Vec<u8>,
    tekst_buf: String,
    ogranicenja: OgranicenjaCitaca,
    izgubljeno_za_2d: BTreeMap<String, usize>,
    nepoznati_elementi: BTreeMap<String, usize>,
    statistika: StatistikaKml,
}

impl<B: BufRead> KontekstCitaca<B> {
    fn novi(mut xml: Reader<B>, ogranicenja: OgranicenjaCitaca) -> Self {
        xml.config_mut().trim_text(false);
        Self {
            xml,
            buf: Vec::with_capacity(4096),
            tekst_buf: String::with_capacity(4096),
            ogranicenja,
            izgubljeno_za_2d: BTreeMap::new(),
            nepoznati_elementi: BTreeMap::new(),
            statistika: StatistikaKml::default(),
        }
    }

    fn pozicija(&self) -> u64 {
        self.xml.buffer_position()
    }

    fn sljedeci_dogadaj(&mut self) -> Result<Event<'static>, GreskaCitaca> {
        self.buf.clear();
        let poz = self.xml.buffer_position();
        let dog = self
            .xml
            .read_event_into(&mut self.buf)
            .map_err(|_| GreskaCitaca::NeispravanXml {
                pozicija_bajta: poz,
            })?;
        Ok(dog.into_owned())
    }

    fn zabiljezi_gubitak_za_2d(&mut self, oznaka: &str) {
        *self.izgubljeno_za_2d.entry(oznaka.to_string()).or_insert(0) += 1;
    }

    fn zabiljezi_nepoznato(&mut self, oznaka: &str) {
        *self
            .nepoznati_elementi
            .entry(oznaka.to_string())
            .or_insert(0) += 1;
        *self.izgubljeno_za_2d.entry(oznaka.to_string()).or_insert(0) += 1;
    }

    /// Čita tekstualni sadržaj do odgovarajuće završne oznake `kraj_oznake`, uključujući CDATA,
    /// XML entitete i eventualne ugniježđene HTML oznake u opisu.
    fn procitaj_tekst(&mut self, kraj_oznake: &str) -> Result<String, GreskaCitaca> {
        let mut izlaz = String::new();
        let mut dubina = 0usize;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            let poz = self.pozicija();
            match dogadaj {
                Event::Text(e) => {
                    izlaz.push_str(e.as_ref());
                }
                Event::CData(e) => {
                    izlaz.push_str(e.as_ref());
                }
                Event::GeneralRef(e) => {
                    if let Some(znak) =
                        e.resolve_char_ref()
                            .map_err(|_| GreskaCitaca::NeispravanXml {
                                pozicija_bajta: poz,
                            })?
                    {
                        izlaz.push(znak);
                    } else {
                        let naziv: &str = e.as_ref();
                        if let Some(rijeseno) = quick_xml::escape::resolve_xml_entity(naziv) {
                            izlaz.push_str(rijeseno);
                        } else if naziv == "nbsp" {
                            izlaz.push('\u{00a0}');
                        } else {
                            izlaz.push('&');
                            izlaz.push_str(naziv);
                            izlaz.push(';');
                        }
                    }
                }
                Event::Start(e) => {
                    dubina += 1;
                    let ime = e.name().0;
                    izlaz.push('<');
                    izlaz.push_str(ime);
                    izlaz.push('>');
                }
                Event::Empty(e) => {
                    let ime = e.name().0;
                    izlaz.push('<');
                    izlaz.push_str(ime);
                    izlaz.push_str("/>");
                }
                Event::End(e) => {
                    let ime = e.name().0;
                    if dubina == 0 && ime == kraj_oznake {
                        break;
                    }
                    if dubina == 0 {
                        return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                            pozicija_bajta: poz,
                        });
                    }
                    dubina -= 1;
                    izlaz.push_str("</");
                    izlaz.push_str(ime);
                    izlaz.push('>');
                }
                Event::Comment(_) => {}
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: poz,
                    });
                }
                _ => {}
            }
        }
        Ok(izlaz)
    }

    fn procitaj_broj(&mut self, kraj_oznake: &str) -> Result<f64, GreskaCitaca> {
        let s = self.procitaj_tekst(kraj_oznake)?;
        s.trim()
            .parse::<f64>()
            .map_err(|_| GreskaCitaca::NeispravnaKoordinata {
                pozicija_bajta: self.pozicija(),
            })
    }

    fn procitaj_bool(&mut self, kraj_oznake: &str) -> Result<bool, GreskaCitaca> {
        let s = self.procitaj_tekst(kraj_oznake)?;
        let cisto = s.trim();
        Ok(cisto == "1" || cisto.eq_ignore_ascii_case("true"))
    }

    /// Preskače podstablo trenutne oznake `kraj_oznake`.
    fn preskoci_element(&mut self, kraj_oznake: &str) -> Result<(), GreskaCitaca> {
        let mut dubina = 0usize;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            let poz = self.pozicija();
            match dogadaj {
                Event::Start(_) => dubina += 1,
                Event::End(e) => {
                    let ime = e.name().0;
                    if dubina == 0 {
                        if ime == kraj_oznake {
                            return Ok(());
                        }
                        return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                            pozicija_bajta: poz,
                        });
                    }
                    dubina -= 1;
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: poz,
                    });
                }
                _ => {}
            }
        }
    }

    /// Obradi oznaku koja se ne prenosi u 2D vektorske slojeve ili je potpuno nepoznata.
    fn obradi_ostalu_oznaku(&mut self, naziv: &str, prazna: bool) -> Result<(), GreskaCitaca> {
        if je_poznata_ne_2d_oznaka(naziv) {
            self.zabiljezi_gubitak_za_2d(naziv);
        } else if !je_poznata_pomocna_oznaka(naziv) {
            self.zabiljezi_nepoznato(naziv);
        }
        if !prazna {
            self.preskoci_element(naziv)?;
        }
        Ok(())
    }

    /// Čita niz koordinata iz `<coordinates>` oznake uz ponovno korištenje spremnika teksta.
    fn procitaj_koordinate(
        &mut self,
        kraj_oznake: &str,
        broji_u_statistiku: bool,
    ) -> Result<Vec<Vrh>, GreskaCitaca> {
        self.tekst_buf.clear();
        loop {
            self.buf.clear();
            let poz = self.xml.buffer_position();
            let dogadaj = self
                .xml
                .read_event_into(&mut self.buf)
                .map_err(|_| GreskaCitaca::NeispravanXml {
                    pozicija_bajta: poz,
                })?;
            match dogadaj {
                Event::Text(e) => {
                    self.tekst_buf.push_str(e.as_ref());
                }
                Event::CData(e) => {
                    self.tekst_buf.push_str(e.as_ref());
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: poz,
                    });
                }
                Event::Comment(_) => {}
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: poz,
                    });
                }
                _ => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: poz,
                    });
                }
            }
        }
        let poz = self.pozicija();
        let mut vrhovi = Vec::new();
        for zapis in self.tekst_buf.split_whitespace() {
            let mut dijelovi = zapis.split(',');
            let Some(duz_s) = dijelovi.next() else {
                continue;
            };
            let Some(sir_s) = dijelovi.next() else {
                return Err(GreskaCitaca::NeispravnaKoordinata {
                    pozicija_bajta: poz,
                });
            };
            let duzina = duz_s
                .parse::<f64>()
                .map_err(|_| GreskaCitaca::NeispravnaKoordinata {
                    pozicija_bajta: poz,
                })?;
            let sirina = sir_s
                .parse::<f64>()
                .map_err(|_| GreskaCitaca::NeispravnaKoordinata {
                    pozicija_bajta: poz,
                })?;
            let visina = match dijelovi.next() {
                Some(vis_s) if !vis_s.is_empty() => Some(vis_s.parse::<f64>().map_err(|_| {
                    GreskaCitaca::NeispravnaKoordinata {
                        pozicija_bajta: poz,
                    }
                })?),
                _ => None,
            };
            let vrh = Vrh::novi(duzina, sirina, visina);
            if broji_u_statistiku {
                self.statistika.ukupno_vrhova += 1;
                if vrh.visina().is_some() {
                    self.statistika.vrhovi_s_visinom += 1;
                    if vrh.ima_nenultu_visinu() {
                        self.statistika.vrhovi_s_nenultom_visinom += 1;
                    }
                }
            }
            vrhovi.push(vrh);
        }
        Ok(vrhovi)
    }

    fn procitaj_gx_coord(&mut self, kraj_oznake: &str) -> Result<Vrh, GreskaCitaca> {
        let s = self.procitaj_tekst(kraj_oznake)?;
        let poz = self.pozicija();
        let mut dijelovi = s.split_whitespace();
        let duz_s = dijelovi
            .next()
            .ok_or(GreskaCitaca::NeispravnaKoordinata {
                pozicija_bajta: poz,
            })?;
        let sir_s = dijelovi
            .next()
            .ok_or(GreskaCitaca::NeispravnaKoordinata {
                pozicija_bajta: poz,
            })?;
        let duzina = duz_s
            .parse::<f64>()
            .map_err(|_| GreskaCitaca::NeispravnaKoordinata {
                pozicija_bajta: poz,
            })?;
        let sirina = sir_s
            .parse::<f64>()
            .map_err(|_| GreskaCitaca::NeispravnaKoordinata {
                pozicija_bajta: poz,
            })?;
        let visina = match dijelovi.next() {
            Some(vis_s) => Some(vis_s.parse::<f64>().map_err(|_| {
                GreskaCitaca::NeispravnaKoordinata {
                    pozicija_bajta: poz,
                }
            })?),
            None => None,
        };
        let vrh = Vrh::novi(duzina, sirina, visina);
        self.statistika.ukupno_vrhova += 1;
        if vrh.visina().is_some() {
            self.statistika.vrhovi_s_visinom += 1;
            if vrh.ima_nenultu_visinu() {
                self.statistika.vrhovi_s_nenultom_visinom += 1;
            }
        }
        Ok(vrh)
    }
}

fn je_poznata_ne_2d_oznaka(ime: &str) -> bool {
    matches!(
        ime,
        "NetworkLink"
            | "GroundOverlay"
            | "LookAt"
            | "Camera"
            | "TimeStamp"
            | "TimeSpan"
            | "gx:TimeSpan"
            | "gx:Tour"
            | "Tour"
            | "ScreenOverlay"
            | "PhotoOverlay"
            | "Model"
            | "Region"
            | "BalloonStyle"
            | "ListStyle"
            | "Snippet"
            | "snippet"
            | "atom:author"
            | "atom:link"
    )
}

fn je_poznata_pomocna_oznaka(ime: &str) -> bool {
    matches!(
        ime,
        "open"
            | "visibility"
            | "address"
            | "phoneNumber"
            | "Schema"
            | "SimpleField"
            | "gx:SimpleArrayField"
            | "gx:drawOrder"
            | "drawOrder"
            | "colorMode"
            | "refreshVisibility"
            | "flyToView"
            | "gx:balloonVisibility"
            | "gx:CascadingStyle"
            | "ItemIcon"
            | "state"
            | "listItemType"
            | "maxSnippetLines"
            | "bgColor"
            | "textColor"
            | "displayMode"
    )
}

fn atribut_id(e: &BytesStart<'_>) -> Option<String> {
    procitaj_atribut(e, "id")
}

fn procitaj_atribut(e: &BytesStart<'_>, kljuc: &str) -> Option<String> {
    for a in e.attributes().flatten() {
        if a.key.0 == kljuc {
            if let Ok(odkodirano) = quick_xml::escape::unescape(&a.value) {
                return Some(odkodirano.into_owned());
            }
            return Some(a.value.into_owned());
        }
    }
    None
}

impl<B: BufRead> KontekstCitaca<B> {
    fn procitaj_korijen(&mut self) -> Result<Mapa, GreskaCitaca> {
        let mut korijen = Mapa::nova(VrstaMape::Korijen);
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    let id = atribut_id(&e);
                    match ime.as_str() {
                        "kml" => self.procitaj_sadrzaj_mape(&mut korijen, "kml", 0, 0)?,
                        "Document" => {
                            let pod =
                                self.procitaj_mapu(VrstaMape::Dokument, id, "Document", 0, 1)?;
                            korijen.podmape.push(pod);
                        }
                        "Folder" => {
                            let pod = self.procitaj_mapu(VrstaMape::Mapa, id, "Folder", 1, 1)?;
                            korijen.podmape.push(pod);
                        }
                        "Placemark" => {
                            let obj = self.procitaj_objekt(id, "Placemark")?;
                            korijen.objekti.push(obj);
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    if ime != "kml" {
                        self.obradi_ostalu_oznaku(&ime, true)?;
                    }
                }
                Event::End(_) => {
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => break,
                _ => {}
            }
        }
        Ok(korijen)
    }

    fn procitaj_mapu(
        &mut self,
        vrsta: VrstaMape,
        id: Option<String>,
        kraj_oznake: &str,
        dubina_mapa: usize,
        dubina_spremnika: usize,
    ) -> Result<Mapa, GreskaCitaca> {
        if dubina_spremnika > self.ogranicenja.najveca_dubina {
            return Err(GreskaCitaca::PrevelikaDubinaStabla {
                dopusteno: self.ogranicenja.najveca_dubina,
            });
        }
        match vrsta {
            VrstaMape::Dokument => self.statistika.dokumenti += 1,
            VrstaMape::Mapa => self.statistika.mape += 1,
            VrstaMape::Korijen => {}
        }
        if dubina_mapa > self.statistika.najveca_dubina_mapa {
            self.statistika.najveca_dubina_mapa = dubina_mapa;
        }
        if dubina_spremnika > self.statistika.najveca_dubina_spremnika {
            self.statistika.najveca_dubina_spremnika = dubina_spremnika;
        }
        let mut mapa = Mapa::nova(vrsta);
        mapa.id = id;
        self.procitaj_sadrzaj_mape(&mut mapa, kraj_oznake, dubina_mapa, dubina_spremnika)?;
        if mapa.opis.as_ref().is_some_and(|o| !o.trim().is_empty()) {
            self.statistika.mape_s_opisom += 1;
        }
        Ok(mapa)
    }

    fn procitaj_sadrzaj_mape(
        &mut self,
        mapa: &mut Mapa,
        kraj_oznake: &str,
        dubina_mapa: usize,
        dubina_spremnika: usize,
    ) -> Result<(), GreskaCitaca> {
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    let id = atribut_id(&e);
                    let kaskada_id = procitaj_atribut(&e, "kml:id").or_else(|| id.clone());
                    match ime.as_str() {
                        "Document" => {
                            let pod = self.procitaj_mapu(
                                VrstaMape::Dokument,
                                id,
                                "Document",
                                dubina_mapa,
                                dubina_spremnika + 1,
                            )?;
                            mapa.podmape.push(pod);
                        }
                        "Folder" => {
                            let pod = self.procitaj_mapu(
                                VrstaMape::Mapa,
                                id,
                                "Folder",
                                dubina_mapa + 1,
                                dubina_spremnika + 1,
                            )?;
                            mapa.podmape.push(pod);
                        }
                        "Placemark" => {
                            let obj = self.procitaj_objekt(id, "Placemark")?;
                            mapa.objekti.push(obj);
                        }
                        "name" => mapa.naziv = Some(self.procitaj_tekst("name")?),
                        "description" => mapa.opis = Some(self.procitaj_tekst("description")?),
                        "visibility" => mapa.vidljivo = self.procitaj_bool("visibility")?,
                        "open" => mapa.otvoreno = self.procitaj_bool("open")?,
                        "styleUrl" => mapa.stil_url = Some(self.procitaj_tekst("styleUrl")?),
                        "Style" => {
                            let stil = self.procitaj_stil(id, "Style")?;
                            if stil.id.is_some() {
                                mapa.stilovi.push(stil);
                            } else {
                                mapa.izravni_stil = Some(stil);
                            }
                        }
                        "gx:CascadingStyle" => {
                            if let Some(mut stil) =
                                self.procitaj_kaskadni_stil("gx:CascadingStyle")?
                            {
                                if stil.id.is_none() {
                                    stil.id = kaskada_id;
                                }
                                mapa.stilovi.push(stil);
                            }
                        }
                        "StyleMap" => {
                            let ms = self.procitaj_mapu_stila(id, "StyleMap")?;
                            mapa.mape_stilova.push(ms);
                        }
                        "NetworkLink" => {
                            let mv = self.procitaj_mreznu_vezu("NetworkLink")?;
                            mapa.mrezne_veze.push(mv);
                        }
                        "GroundOverlay" => {
                            let sp = self.procitaj_slikovni_prekrivac("GroundOverlay")?;
                            mapa.prekrivaci.push(sp);
                        }
                        "LookAt" => {
                            mapa.pogled = Some(self.procitaj_pogled(false, "LookAt")?);
                        }
                        "Camera" => {
                            mapa.pogled = Some(self.procitaj_pogled(true, "Camera")?);
                        }
                        "TimeStamp" => {
                            mapa.vrijeme = Some(self.procitaj_trenutak("TimeStamp")?);
                        }
                        "TimeSpan" | "gx:TimeSpan" => {
                            mapa.vrijeme = Some(self.procitaj_raspon_vremena(&ime)?);
                        }
                        "ExtendedData" => {
                            self.procitaj_prosirene_podatke(&mut mapa.atributi, "ExtendedData")?;
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    self.obradi_ostalu_oznaku(&ime, true)?;
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn procitaj_objekt(
        &mut self,
        id: Option<String>,
        kraj_oznake: &str,
    ) -> Result<Objekt, GreskaCitaca> {
        self.statistika.objekti += 1;
        let mut obj = Objekt {
            id,
            vidljivo: true,
            ..Objekt::default()
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    let pod_id = atribut_id(&e);
                    match ime.as_str() {
                        "name" => obj.naziv = Some(self.procitaj_tekst("name")?),
                        "description" => obj.opis = Some(self.procitaj_tekst("description")?),
                        "Snippet" | "snippet" => {
                            self.zabiljezi_gubitak_za_2d("Snippet");
                            obj.kratki_opis = Some(self.procitaj_tekst(&ime)?);
                        }
                        "visibility" => obj.vidljivo = self.procitaj_bool("visibility")?,
                        "styleUrl" => obj.stil_url = Some(self.procitaj_tekst("styleUrl")?),
                        "Style" => obj.izravni_stil = Some(self.procitaj_stil(pod_id, "Style")?),
                        "Point" => obj.geometrija = Some(self.procitaj_tocku("Point")?),
                        "LineString" => obj.geometrija = Some(self.procitaj_liniju("LineString")?),
                        "LinearRing" => obj.geometrija = Some(self.procitaj_liniju("LinearRing")?),
                        "Polygon" => obj.geometrija = Some(self.procitaj_poligon("Polygon")?),
                        "MultiGeometry" => {
                            obj.geometrija = Some(self.procitaj_vise_geometrija("MultiGeometry")?)
                        }
                        "gx:Track" | "Track" => {
                            obj.geometrija = Some(self.procitaj_trag(&ime, &mut obj.atributi)?)
                        }
                        "gx:MultiTrack" | "MultiTrack" => {
                            obj.geometrija =
                                Some(self.procitaj_vise_tragova(&ime, &mut obj.atributi)?)
                        }
                        "LookAt" => obj.pogled = Some(self.procitaj_pogled(false, "LookAt")?),
                        "Camera" => obj.pogled = Some(self.procitaj_pogled(true, "Camera")?),
                        "TimeStamp" => obj.vrijeme = Some(self.procitaj_trenutak("TimeStamp")?),
                        "TimeSpan" | "gx:TimeSpan" => {
                            obj.vrijeme = Some(self.procitaj_raspon_vremena(&ime)?)
                        }
                        "ExtendedData" => {
                            self.procitaj_prosirene_podatke(&mut obj.atributi, "ExtendedData")?
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    if matches!(
                        ime.as_str(),
                        "Point" | "LineString" | "LinearRing" | "Polygon"
                    ) {
                        return Err(GreskaCitaca::PraznaGeometrija {
                            oznaka: "Geometrija",
                            pozicija_bajta: self.pozicija(),
                        });
                    }
                    self.obradi_ostalu_oznaku(&ime, true)?;
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        if obj.naziv.as_ref().is_some_and(|n| !n.trim().is_empty()) {
            self.statistika.objekti_s_nazivom += 1;
        }
        if obj.opis.as_ref().is_some_and(|o| !o.trim().is_empty()) {
            self.statistika.objekti_s_opisom += 1;
        }
        if obj.geometrija.is_none() {
            self.statistika.objekti_bez_geometrije += 1;
        }
        Ok(obj)
    }

    fn procitaj_tocku(&mut self, kraj_oznake: &str) -> Result<Geometrija, GreskaCitaca> {
        let mut vrhovi = Vec::new();
        let mut nacin_visine = NacinVisine::UzTlo;
        let mut izvlacenje = false;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "coordinates" => vrhovi = self.procitaj_koordinate("coordinates", true)?,
                        "altitudeMode" | "gx:altitudeMode" => {
                            let s = self.procitaj_tekst(&ime)?;
                            if let Some(nv) = NacinVisine::iz_teksta(&s) {
                                nacin_visine = nv;
                            }
                        }
                        "extrude" => izvlacenje = self.procitaj_bool("extrude")?,
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    if ime != "coordinates" {
                        self.obradi_ostalu_oznaku(&ime, true)?;
                    }
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        let Some(vrh) = vrhovi.into_iter().next() else {
            return Err(GreskaCitaca::PraznaGeometrija {
                oznaka: "Point",
                pozicija_bajta: self.pozicija(),
            });
        };
        self.statistika.tocke += 1;
        Ok(Geometrija::Tocka {
            vrh,
            nacin_visine,
            izvlacenje,
        })
    }

    fn procitaj_liniju(&mut self, kraj_oznake: &str) -> Result<Geometrija, GreskaCitaca> {
        let mut vrhovi = Vec::new();
        let mut nacin_visine = NacinVisine::UzTlo;
        let mut izvlacenje = false;
        let mut prati_teren = false;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "coordinates" => vrhovi = self.procitaj_koordinate("coordinates", true)?,
                        "altitudeMode" | "gx:altitudeMode" => {
                            let s = self.procitaj_tekst(&ime)?;
                            if let Some(nv) = NacinVisine::iz_teksta(&s) {
                                nacin_visine = nv;
                            }
                        }
                        "extrude" => izvlacenje = self.procitaj_bool("extrude")?,
                        "tessellate" => prati_teren = self.procitaj_bool("tessellate")?,
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    if ime != "coordinates" {
                        self.obradi_ostalu_oznaku(&ime, true)?;
                    }
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        // U pravom `myplaces.kml` Google Earth Pro zapisuje 2 `<LineString>` s praznim `<coordinates/>`.
        if vrhovi.is_empty() {
            self.zabiljezi_gubitak_za_2d("prazan_LineString");
        }
        self.statistika.linije += 1;
        Ok(Geometrija::Linija {
            vrhovi,
            nacin_visine,
            izvlacenje,
            prati_teren,
        })
    }

    /// Čita sve `<LinearRing>` prstenove unutar `<outerBoundaryIs>` ili `<innerBoundaryIs>`.
    /// U stvarnom `myplaces.kml` jedan `<innerBoundaryIs>` sadrži više `<LinearRing>` elemenata!
    fn procitaj_granicu_poligona(
        &mut self,
        kraj_oznake: &str,
    ) -> Result<Vec<Vec<Vrh>>, GreskaCitaca> {
        let mut prstenovi = Vec::new();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    if ime == "LinearRing" {
                        let prsten = self.procitaj_prsten("LinearRing")?;
                        if !prsten.is_empty() {
                            prstenovi.push(prsten);
                        }
                    } else {
                        self.obradi_ostalu_oznaku(&ime, false)?;
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    self.obradi_ostalu_oznaku(&ime, true)?;
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(prstenovi)
    }

    fn procitaj_prsten(&mut self, kraj_oznake: &str) -> Result<Vec<Vrh>, GreskaCitaca> {
        let mut vrhovi = Vec::new();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    if ime == "coordinates" {
                        vrhovi = self.procitaj_koordinate("coordinates", true)?;
                    } else if matches!(
                        ime.as_str(),
                        "altitudeMode" | "gx:altitudeMode" | "extrude" | "tessellate"
                    ) {
                        self.preskoci_element(&ime)?;
                    } else {
                        self.obradi_ostalu_oznaku(&ime, false)?;
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    self.obradi_ostalu_oznaku(&ime, true)?;
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(vrhovi)
    }

    fn procitaj_poligon(&mut self, kraj_oznake: &str) -> Result<Geometrija, GreskaCitaca> {
        let mut vanjski = Vec::new();
        let mut rupe = Vec::new();
        let mut nacin_visine = NacinVisine::UzTlo;
        let mut izvlacenje = false;
        let mut prati_teren = false;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "outerBoundaryIs" => {
                            let mut prstenovi =
                                self.procitaj_granicu_poligona("outerBoundaryIs")?;
                            if !prstenovi.is_empty() {
                                vanjski = prstenovi.remove(0);
                            }
                        }
                        "innerBoundaryIs" => {
                            let mut unutarnji =
                                self.procitaj_granicu_poligona("innerBoundaryIs")?;
                            rupe.append(&mut unutarnji);
                        }
                        "altitudeMode" | "gx:altitudeMode" => {
                            let s = self.procitaj_tekst(&ime)?;
                            if let Some(nv) = NacinVisine::iz_teksta(&s) {
                                nacin_visine = nv;
                            }
                        }
                        "extrude" => izvlacenje = self.procitaj_bool("extrude")?,
                        "tessellate" => prati_teren = self.procitaj_bool("tessellate")?,
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    self.obradi_ostalu_oznaku(&ime, true)?;
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        if vanjski.is_empty() {
            return Err(GreskaCitaca::PraznaGeometrija {
                oznaka: "Polygon",
                pozicija_bajta: self.pozicija(),
            });
        }
        self.statistika.poligoni += 1;
        self.statistika.rupe_poligona += rupe.len();
        Ok(Geometrija::Poligon {
            vanjski,
            rupe,
            nacin_visine,
            izvlacenje,
            prati_teren,
        })
    }

    fn procitaj_vise_geometrija(&mut self, kraj_oznake: &str) -> Result<Geometrija, GreskaCitaca> {
        self.statistika.vise_geometrija += 1;
        let mut geometrije = Vec::new();
        let mut pisi_atribute = Vec::new();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "Point" => geometrije.push(self.procitaj_tocku("Point")?),
                        "LineString" => geometrije.push(self.procitaj_liniju("LineString")?),
                        "LinearRing" => geometrije.push(self.procitaj_liniju("LinearRing")?),
                        "Polygon" => geometrije.push(self.procitaj_poligon("Polygon")?),
                        "MultiGeometry" => {
                            geometrije.push(self.procitaj_vise_geometrija("MultiGeometry")?)
                        }
                        "gx:Track" | "Track" => {
                            geometrije.push(self.procitaj_trag(&ime, &mut pisi_atribute)?)
                        }
                        "gx:MultiTrack" | "MultiTrack" => {
                            geometrije.push(self.procitaj_vise_tragova(&ime, &mut pisi_atribute)?)
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    self.obradi_ostalu_oznaku(&ime, true)?;
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(Geometrija::ViseGeometrija(geometrije))
    }

    fn procitaj_trag(
        &mut self,
        kraj_oznake: &str,
        atributi: &mut Vec<AtributPodatka>,
    ) -> Result<Geometrija, GreskaCitaca> {
        self.statistika.tragovi += 1;
        let mut vrhovi = Vec::new();
        let mut vremena = Vec::new();
        let mut nacin_visine = NacinVisine::UzTlo;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "gx:coord" | "coord" => vrhovi.push(self.procitaj_gx_coord(&ime)?),
                        "when" => vremena.push(self.procitaj_tekst("when")?),
                        "altitudeMode" | "gx:altitudeMode" => {
                            let s = self.procitaj_tekst(&ime)?;
                            if let Some(nv) = NacinVisine::iz_teksta(&s) {
                                nacin_visine = nv;
                            }
                        }
                        "ExtendedData" => {
                            self.procitaj_prosirene_podatke(atributi, "ExtendedData")?
                        }
                        "gx:angles" | "angles" => self.preskoci_element(&ime)?,
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    self.obradi_ostalu_oznaku(&ime, true)?;
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(Geometrija::Trag {
            vrhovi,
            vremena,
            nacin_visine,
        })
    }

    fn procitaj_vise_tragova(
        &mut self,
        kraj_oznake: &str,
        atributi: &mut Vec<AtributPodatka>,
    ) -> Result<Geometrija, GreskaCitaca> {
        let mut tragovi = Vec::new();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "gx:Track" | "Track" => {
                            tragovi.push(self.procitaj_trag(&ime, atributi)?)
                        }
                        "altitudeMode" | "gx:altitudeMode" | "gx:interpolate" => {
                            self.preskoci_element(&ime)?
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    self.obradi_ostalu_oznaku(&ime, true)?;
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        if tragovi.len() == 1 {
            Ok(tragovi.remove(0))
        } else {
            self.statistika.vise_geometrija += 1;
            Ok(Geometrija::ViseGeometrija(tragovi))
        }
    }

    fn procitaj_stil(
        &mut self,
        id: Option<String>,
        kraj_oznake: &str,
    ) -> Result<Stil, GreskaCitaca> {
        self.statistika.stilovi += 1;
        let mut stil = Stil {
            id,
            ..Stil::default()
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "IconStyle" => stil.ikona = Some(self.procitaj_stil_ikone("IconStyle")?),
                        "LineStyle" => stil.crta = Some(self.procitaj_stil_crte("LineStyle")?),
                        "PolyStyle" => {
                            stil.poligon = Some(self.procitaj_stil_poligona("PolyStyle")?)
                        }
                        "LabelStyle" => {
                            stil.natpis = Some(self.procitaj_stil_natpisa("LabelStyle")?)
                        }
                        "BalloonStyle" => {
                            self.zabiljezi_gubitak_za_2d("BalloonStyle");
                            stil.balon = Some(self.procitaj_stil_balona("BalloonStyle")?);
                        }
                        "ListStyle" => {
                            self.zabiljezi_gubitak_za_2d("ListStyle");
                            self.preskoci_element("ListStyle")?;
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    if ime == "ListStyle" || ime == "BalloonStyle" {
                        self.zabiljezi_gubitak_za_2d(&ime);
                    }
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(stil)
    }

    fn procitaj_kaskadni_stil(
        &mut self,
        kraj_oznake: &str,
    ) -> Result<Option<Stil>, GreskaCitaca> {
        let mut izlaz = None;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    let id = atribut_id(&e);
                    if ime == "Style" {
                        izlaz = Some(self.procitaj_stil(id, "Style")?);
                    } else {
                        self.obradi_ostalu_oznaku(&ime, false)?;
                    }
                }
                Event::Empty(e) => {
                    let ime = e.name().0.to_string();
                    self.obradi_ostalu_oznaku(&ime, true)?;
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(izlaz)
    }

    fn procitaj_stil_ikone(&mut self, kraj_oznake: &str) -> Result<StilIkone, GreskaCitaca> {
        let mut ikona = StilIkone {
            mjerilo: 1.0,
            ..StilIkone::default()
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    let hs_x = procitaj_atribut(&e, "x").and_then(|v| v.parse::<f64>().ok());
                    let hs_y = procitaj_atribut(&e, "y").and_then(|v| v.parse::<f64>().ok());
                    match ime.as_str() {
                        "color" => {
                            let s = self.procitaj_tekst("color")?;
                            ikona.boja = BojaRgba::iz_kml_heksa(&s);
                        }
                        "scale" => ikona.mjerilo = self.procitaj_broj("scale").unwrap_or(1.0),
                        "heading" => ikona.smjer = self.procitaj_broj("heading").unwrap_or(0.0),
                        "Icon" => ikona.putanja_ikone = self.procitaj_ikona_href("Icon")?,
                        "hotSpot" => {
                            ikona.sidriste = Some((hs_x.unwrap_or(0.5), hs_y.unwrap_or(0.5)));
                            self.preskoci_element("hotSpot")?;
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(e) => {
                    if e.name().0 == "hotSpot" {
                        let x = procitaj_atribut(&e, "x")
                            .and_then(|v| v.parse::<f64>().ok())
                            .unwrap_or(0.5);
                        let y = procitaj_atribut(&e, "y")
                            .and_then(|v| v.parse::<f64>().ok())
                            .unwrap_or(0.5);
                        ikona.sidriste = Some((x, y));
                    }
                }
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(ikona)
    }

    fn procitaj_ikona_href(&mut self, kraj_oznake: &str) -> Result<Option<String>, GreskaCitaca> {
        let mut putanja = None;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    if ime == "href" {
                        putanja = Some(self.procitaj_tekst("href")?);
                    } else {
                        self.preskoci_element(&ime)?;
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(putanja)
    }

    fn procitaj_stil_crte(&mut self, kraj_oznake: &str) -> Result<StilCrte, GreskaCitaca> {
        let mut crta = StilCrte {
            sirina: 1.0,
            ..StilCrte::default()
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "color" => {
                            let s = self.procitaj_tekst("color")?;
                            crta.boja = BojaRgba::iz_kml_heksa(&s);
                        }
                        "width" => crta.sirina = self.procitaj_broj("width").unwrap_or(1.0),
                        "gx:physicalWidth" | "gx:outerColor" | "gx:outerWidth"
                        | "gx:labelVisibility" => self.preskoci_element(&ime)?,
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(crta)
    }

    fn procitaj_stil_poligona(&mut self, kraj_oznake: &str) -> Result<StilPoligona, GreskaCitaca> {
        let mut poligon = StilPoligona {
            ispuna: true,
            obrub: true,
            ..StilPoligona::default()
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "color" => {
                            let s = self.procitaj_tekst("color")?;
                            poligon.boja = BojaRgba::iz_kml_heksa(&s);
                        }
                        "fill" => poligon.ispuna = self.procitaj_bool("fill")?,
                        "outline" => poligon.obrub = self.procitaj_bool("outline")?,
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(poligon)
    }

    fn procitaj_stil_natpisa(&mut self, kraj_oznake: &str) -> Result<StilNatpisa, GreskaCitaca> {
        let mut natpis = StilNatpisa {
            mjerilo: 1.0,
            ..StilNatpisa::default()
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "color" => {
                            let s = self.procitaj_tekst("color")?;
                            natpis.boja = BojaRgba::iz_kml_heksa(&s);
                        }
                        "scale" => natpis.mjerilo = self.procitaj_broj("scale").unwrap_or(1.0),
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(natpis)
    }

    fn procitaj_stil_balona(&mut self, kraj_oznake: &str) -> Result<StilBalona, GreskaCitaca> {
        let mut balon = StilBalona::default();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "bgColor" => {
                            let s = self.procitaj_tekst("bgColor")?;
                            balon.boja_pozadine = BojaRgba::iz_kml_heksa(&s);
                        }
                        "textColor" => {
                            let s = self.procitaj_tekst("textColor")?;
                            balon.boja_teksta = BojaRgba::iz_kml_heksa(&s);
                        }
                        "text" => balon.tekst = Some(self.procitaj_tekst("text")?),
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(balon)
    }

    fn procitaj_mapu_stila(
        &mut self,
        id: Option<String>,
        kraj_oznake: &str,
    ) -> Result<MapaStila, GreskaCitaca> {
        self.statistika.mape_stilova += 1;
        let mut ms = MapaStila {
            id,
            parovi: Vec::new(),
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    if ime == "Pair" {
                        ms.parovi.push(self.procitaj_par_stila("Pair")?);
                    } else {
                        self.obradi_ostalu_oznaku(&ime, false)?;
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(ms)
    }

    fn procitaj_par_stila(&mut self, kraj_oznake: &str) -> Result<ParStila, GreskaCitaca> {
        let mut par = ParStila::default();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "key" => par.kljuc = self.procitaj_tekst("key")?.trim().to_string(),
                        "styleUrl" => {
                            par.stil_url = self.procitaj_tekst("styleUrl")?.trim().to_string()
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(par)
    }

    fn procitaj_mreznu_vezu(&mut self, kraj_oznake: &str) -> Result<MreznaVeza, GreskaCitaca> {
        self.statistika.mrezne_veze += 1;
        self.zabiljezi_gubitak_za_2d("NetworkLink");
        let mut mv = MreznaVeza {
            vidljivo: true,
            ..MreznaVeza::default()
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    let pod_id = atribut_id(&e);
                    match ime.as_str() {
                        "name" => mv.naziv = Some(self.procitaj_tekst("name")?),
                        "description" => mv.opis = Some(self.procitaj_tekst("description")?),
                        "visibility" => mv.vidljivo = self.procitaj_bool("visibility")?,
                        "styleUrl" => {
                            let _ = self.procitaj_tekst("styleUrl")?;
                        }
                        "Style" => mv.stil = Some(self.procitaj_stil(pod_id, "Style")?),
                        "Link" | "Url" => {
                            let (put, osvj) = self.procitaj_link(&ime)?;
                            mv.putanja = put;
                            mv.nacin_osvjezavanja = osvj;
                        }
                        "LookAt" => {
                            let _ = self.procitaj_pogled(false, "LookAt")?;
                        }
                        "Camera" => {
                            let _ = self.procitaj_pogled(true, "Camera")?;
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(mv)
    }

    fn procitaj_link(
        &mut self,
        kraj_oznake: &str,
    ) -> Result<(Option<String>, Option<String>), GreskaCitaca> {
        let mut putanja = None;
        let mut osvjezavanje = None;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "href" => putanja = Some(self.procitaj_tekst("href")?),
                        "refreshMode" => osvjezavanje = Some(self.procitaj_tekst("refreshMode")?),
                        _ => self.preskoci_element(&ime)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok((putanja, osvjezavanje))
    }

    fn procitaj_slikovni_prekrivac(
        &mut self,
        kraj_oznake: &str,
    ) -> Result<SlikovniPrekrivac, GreskaCitaca> {
        self.statistika.slikovni_prekrivaci += 1;
        self.zabiljezi_gubitak_za_2d("GroundOverlay");
        let mut sp = SlikovniPrekrivac {
            vidljivo: true,
            ..SlikovniPrekrivac::default()
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "name" => sp.naziv = Some(self.procitaj_tekst("name")?),
                        "description" => sp.opis = Some(self.procitaj_tekst("description")?),
                        "visibility" => sp.vidljivo = self.procitaj_bool("visibility")?,
                        "Icon" => sp.putanja_slike = self.procitaj_ikona_href("Icon")?,
                        "LatLonBox" => sp.pravokutnik = Some(self.procitaj_lat_lon_box()?),
                        "gx:LatLonQuad" | "LatLonQuad" => {
                            sp.cetverokut = self.procitaj_lat_lon_quad(&ime)?
                        }
                        "LookAt" => {
                            let _ = self.procitaj_pogled(false, "LookAt")?;
                        }
                        "Camera" => {
                            let _ = self.procitaj_pogled(true, "Camera")?;
                        }
                        "TimeStamp" => {
                            let _ = self.procitaj_trenutak("TimeStamp")?;
                        }
                        "TimeSpan" | "gx:TimeSpan" => {
                            let _ = self.procitaj_raspon_vremena(&ime)?;
                        }
                        "color" | "altitude" | "altitudeMode" | "gx:altitudeMode" => {
                            self.preskoci_element(&ime)?
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(sp)
    }

    fn procitaj_lat_lon_box(&mut self) -> Result<(f64, f64, f64, f64, f64), GreskaCitaca> {
        let (mut sjever, mut jug, mut istok, mut zapad, mut kut) = (0.0, 0.0, 0.0, 0.0, 0.0);
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "north" => sjever = self.procitaj_broj("north")?,
                        "south" => jug = self.procitaj_broj("south")?,
                        "east" => istok = self.procitaj_broj("east")?,
                        "west" => zapad = self.procitaj_broj("west")?,
                        "rotation" => kut = self.procitaj_broj("rotation")?,
                        _ => self.preskoci_element(&ime)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == "LatLonBox" {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok((sjever, jug, istok, zapad, kut))
    }

    fn procitaj_lat_lon_quad(&mut self, kraj_oznake: &str) -> Result<Vec<Vrh>, GreskaCitaca> {
        let mut vrhovi = Vec::new();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    if ime == "coordinates" {
                        vrhovi = self.procitaj_koordinate("coordinates", false)?;
                    } else {
                        self.preskoci_element(&ime)?;
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(vrhovi)
    }

    fn procitaj_pogled(
        &mut self,
        je_kamera: bool,
        kraj_oznake: &str,
    ) -> Result<PogledKamere, GreskaCitaca> {
        self.statistika.pogledi_kamere += 1;
        self.zabiljezi_gubitak_za_2d(if je_kamera { "Camera" } else { "LookAt" });
        let mut p = PogledKamere {
            je_kamera,
            ..PogledKamere::default()
        };
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "longitude" => p.duzina = self.procitaj_broj("longitude").unwrap_or(0.0),
                        "latitude" => p.sirina = self.procitaj_broj("latitude").unwrap_or(0.0),
                        "altitude" => p.visina = self.procitaj_broj("altitude").unwrap_or(0.0),
                        "heading" => p.smjer = self.procitaj_broj("heading").unwrap_or(0.0),
                        "tilt" => p.nagib = self.procitaj_broj("tilt").unwrap_or(0.0),
                        "range" | "roll" => {
                            p.udaljenost_ili_rotacija = self.procitaj_broj(&ime).unwrap_or(0.0)
                        }
                        "altitudeMode" | "gx:altitudeMode" => {
                            let s = self.procitaj_tekst(&ime)?;
                            if let Some(nv) = NacinVisine::iz_teksta(&s) {
                                p.nacin_visine = nv;
                            }
                        }
                        "gx:TimeStamp" | "TimeStamp" => {
                            let _ = self.procitaj_trenutak(&ime)?;
                        }
                        "gx:TimeSpan" | "TimeSpan" => {
                            let _ = self.procitaj_raspon_vremena(&ime)?;
                        }
                        "gx:horizFov" => self.preskoci_element(&ime)?,
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(p)
    }

    fn procitaj_trenutak(&mut self, kraj_oznake: &str) -> Result<VrijemeKml, GreskaCitaca> {
        self.statistika.vremenski_zapisi += 1;
        self.zabiljezi_gubitak_za_2d(kraj_oznake);
        let mut kada = String::new();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    if ime == "when" {
                        kada = self.procitaj_tekst("when")?;
                    } else {
                        self.preskoci_element(&ime)?;
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(VrijemeKml::Trenutak(kada))
    }

    fn procitaj_raspon_vremena(&mut self, kraj_oznake: &str) -> Result<VrijemeKml, GreskaCitaca> {
        self.statistika.vremenski_zapisi += 1;
        self.zabiljezi_gubitak_za_2d(kraj_oznake);
        let (mut pocetak, mut kraj) = (None, None);
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "begin" => pocetak = Some(self.procitaj_tekst("begin")?),
                        "end" => kraj = Some(self.procitaj_tekst("end")?),
                        _ => self.preskoci_element(&ime)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(VrijemeKml::Raspon { pocetak, kraj })
    }

    fn procitaj_prosirene_podatke(
        &mut self,
        atributi: &mut Vec<AtributPodatka>,
        kraj_oznake: &str,
    ) -> Result<(), GreskaCitaca> {
        self.statistika.prosireni_podaci += 1;
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    let naziv = procitaj_atribut(&e, "name").unwrap_or_default();
                    match ime.as_str() {
                        "Data" => {
                            atributi.push(self.procitaj_data(naziv, "Data")?);
                        }
                        "SchemaData" => {
                            self.procitaj_schema_data(atributi, "SchemaData")?;
                        }
                        _ => self.obradi_ostalu_oznaku(&ime, false)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn procitaj_data(
        &mut self,
        naziv: String,
        kraj_oznake: &str,
    ) -> Result<AtributPodatka, GreskaCitaca> {
        let mut prikazni_naziv = None;
        let mut vrijednost = String::new();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    match ime.as_str() {
                        "displayName" => {
                            prikazni_naziv = Some(self.procitaj_tekst("displayName")?)
                        }
                        "value" => vrijednost = self.procitaj_tekst("value")?,
                        _ => self.preskoci_element(&ime)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(AtributPodatka {
            naziv,
            prikazni_naziv,
            vrijednosti: vec![vrijednost],
        })
    }

    fn procitaj_schema_data(
        &mut self,
        atributi: &mut Vec<AtributPodatka>,
        kraj_oznake: &str,
    ) -> Result<(), GreskaCitaca> {
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    let naziv = procitaj_atribut(&e, "name").unwrap_or_default();
                    match ime.as_str() {
                        "SimpleData" => {
                            let v = self.procitaj_tekst("SimpleData")?;
                            atributi.push(AtributPodatka {
                                naziv,
                                prikazni_naziv: None,
                                vrijednosti: vec![v],
                            });
                        }
                        "gx:SimpleArrayData" | "SimpleArrayData" => {
                            let niz = self.procitaj_niz_podataka(naziv, &ime)?;
                            atributi.push(niz);
                        }
                        _ => self.preskoci_element(&ime)?,
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn procitaj_niz_podataka(
        &mut self,
        naziv: String,
        kraj_oznake: &str,
    ) -> Result<AtributPodatka, GreskaCitaca> {
        let mut vrijednosti = Vec::new();
        loop {
            let dogadaj = self.sljedeci_dogadaj()?;
            match dogadaj {
                Event::Start(e) => {
                    let ime = e.name().0.to_string();
                    if ime == "gx:value" || ime == "value" {
                        vrijednosti.push(self.procitaj_tekst(&ime)?);
                    } else {
                        self.preskoci_element(&ime)?;
                    }
                }
                Event::Empty(_) => {}
                Event::End(e) => {
                    if e.name().0 == kraj_oznake {
                        break;
                    }
                    return Err(GreskaCitaca::NeocekivanZavrsetakOznake {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                Event::Eof => {
                    return Err(GreskaCitaca::NeispravanXml {
                        pozicija_bajta: self.pozicija(),
                    });
                }
                _ => {}
            }
        }
        Ok(AtributPodatka {
            naziv,
            prikazni_naziv: None,
            vrijednosti,
        })
    }
}

fn cisti_kljuc_stila(url: &str) -> &str {
    url.trim().trim_start_matches('#')
}

/// Rješava `styleUrl` reference nakon što je cijela mapa pročitana, jer Google Earth Pro
/// često zapisuje `<StyleMap>` prije `<Style>` elemenata na koje se `<Pair>` poziva.
fn rijesi_stilove_u_mapi(
    mapa: &mut Mapa,
    naslijedeni_stilovi: &HashMap<String, Stil>,
    naslijedene_mape_stilova: &HashMap<String, MapaStila>,
    stat: &mut StatistikaKml,
) {
    let mut lokalni_stilovi = naslijedeni_stilovi.clone();
    let mut lokalne_mape = naslijedene_mape_stilova.clone();

    for s in &mapa.stilovi {
        if let Some(ref id) = s.id {
            lokalni_stilovi.insert(cisti_kljuc_stila(id).to_string(), s.clone());
        }
    }
    for ms in &mapa.mape_stilova {
        if let Some(ref id) = ms.id {
            lokalne_mape.insert(cisti_kljuc_stila(id).to_string(), ms.clone());
        }
    }

    for obj in &mut mapa.objekti {
        if let Some(ref izravni) = obj.izravni_stil {
            obj.rijeseni_stil = Some(RijeseniStil {
                normalni: izravni.clone(),
                istaknuti: None,
            });
            stat.rijeseni_stilovi_objekata += 1;
            continue;
        }
        let Some(ref url) = obj.stil_url else {
            continue;
        };
        let kljuc = cisti_kljuc_stila(url);
        if let Some(ms) = lokalne_mape.get(kljuc) {
            let mut normalni = None;
            let mut istaknuti = None;
            for par in &ms.parovi {
                let ciljni_kljuc = cisti_kljuc_stila(&par.stil_url);
                if let Some(s) = lokalni_stilovi.get(ciljni_kljuc) {
                    if par.kljuc == "normal" {
                        normalni = Some(s.clone());
                    } else if par.kljuc == "highlight" {
                        istaknuti = Some(s.clone());
                    }
                }
            }
            if let Some(norm) = normalni {
                obj.rijeseni_stil = Some(RijeseniStil {
                    normalni: norm,
                    istaknuti,
                });
                stat.rijeseni_stilovi_objekata += 1;
            } else {
                stat.nerijeseni_stilovi_objekata += 1;
            }
        } else if let Some(s) = lokalni_stilovi.get(kljuc) {
            obj.rijeseni_stil = Some(RijeseniStil {
                normalni: s.clone(),
                istaknuti: None,
            });
            stat.rijeseni_stilovi_objekata += 1;
        } else {
            stat.nerijeseni_stilovi_objekata += 1;
        }
    }

    for pod in &mut mapa.podmape {
        rijesi_stilove_u_mapi(pod, &lokalni_stilovi, &lokalne_mape, stat);
    }
}

fn procitaj_kml_iz_citaca<B: BufRead>(
    citac: B,
    ogranicenja: OgranicenjaCitaca,
) -> Result<StabloKml, GreskaCitaca> {
    let xml = Reader::from_reader(citac);
    let mut ktx = KontekstCitaca::novi(xml, ogranicenja);
    let mut korijen = ktx.procitaj_korijen()?;
    let prazni_stilovi = HashMap::new();
    let prazne_mape = HashMap::new();
    rijesi_stilove_u_mapi(
        &mut korijen,
        &prazni_stilovi,
        &prazne_mape,
        &mut ktx.statistika,
    );
    Ok(StabloKml {
        korijen,
        slike_u_arhivi: Vec::new(),
        izgubljeno_za_2d: ktx.izgubljeno_za_2d,
        nepoznati_elementi: ktx.nepoznati_elementi,
        statistika: ktx.statistika,
    })
}
