//! Podatkovni model stabla KML dokumenta, objekata, stilova i statistike.

use std::collections::BTreeMap;

/// Vrsta spremnika u KML hijerarhiji.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VrstaMape {
    Korijen,
    Dokument,
    Mapa,
}

/// Način tumačenja visine (Z koordinate) prema KML i `gx:` specifikaciji.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NacinVisine {
    #[default]
    UzTlo,
    RelativnoPremaTlu,
    Apsolutno,
    UzMorskoDno,
    RelativnoPremaMorskomDnu,
}

impl NacinVisine {
    pub fn iz_teksta(tekst: &str) -> Option<Self> {
        match tekst.trim() {
            "clampToGround" => Some(Self::UzTlo),
            "relativeToGround" => Some(Self::RelativnoPremaTlu),
            "absolute" => Some(Self::Apsolutno),
            "clampToSeaFloor" => Some(Self::UzMorskoDno),
            "relativeToSeaFloor" => Some(Self::RelativnoPremaMorskomDnu),
            _ => None,
        }
    }
}

/// Jedan vrh geometrije. Visina se čuva kao `f64` (NaN kad nije zadana) radi manjeg zauzeća memorije
/// na milijunima vrhova (24 bajta umjesto 32 bajta).
#[derive(Clone, Copy, Debug)]
pub struct Vrh {
    pub duzina: f64,
    pub sirina: f64,
    visina_m: f64,
}

impl Vrh {
    pub fn novi(duzina: f64, sirina: f64, visina: Option<f64>) -> Self {
        Self {
            duzina,
            sirina,
            visina_m: visina.unwrap_or(f64::NAN),
        }
    }

    pub fn visina(&self) -> Option<f64> {
        if self.visina_m.is_nan() {
            None
        } else {
            Some(self.visina_m)
        }
    }

    pub fn ima_nenultu_visinu(&self) -> bool {
        !self.visina_m.is_nan() && self.visina_m.abs() > 1e-9
    }
}

impl PartialEq for Vrh {
    fn eq(&self, other: &Self) -> bool {
        self.duzina == other.duzina
            && self.sirina == other.sirina
            && self.visina() == other.visina()
    }
}

/// Geometrija objekta u KML-u.
#[derive(Clone, Debug, PartialEq)]
pub enum Geometrija {
    Tocka {
        vrh: Vrh,
        nacin_visine: NacinVisine,
        izvlacenje: bool,
    },
    Linija {
        vrhovi: Vec<Vrh>,
        nacin_visine: NacinVisine,
        izvlacenje: bool,
        prati_teren: bool,
    },
    Poligon {
        vanjski: Vec<Vrh>,
        rupe: Vec<Vec<Vrh>>,
        nacin_visine: NacinVisine,
        izvlacenje: bool,
        prati_teren: bool,
    },
    Trag {
        vrhovi: Vec<Vrh>,
        vremena: Vec<String>,
        nacin_visine: NacinVisine,
    },
    ViseGeometrija(Vec<Geometrija>),
}

/// RGBA boja pretvorena iz KML zapisa `aabbggrr`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BojaRgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl BojaRgba {
    /// KML zapisuje boju heksadecimalno obrnutim redoslijedom: `aabbggrr`.
    pub fn iz_kml_heksa(tekst: &str) -> Option<Self> {
        let cisto = tekst.trim().trim_start_matches('#');
        if cisto.len() != 8 {
            return None;
        }
        let a = u8::from_str_radix(&cisto[0..2], 16).ok()?;
        let b = u8::from_str_radix(&cisto[2..4], 16).ok()?;
        let g = u8::from_str_radix(&cisto[4..6], 16).ok()?;
        let r = u8::from_str_radix(&cisto[6..8], 16).ok()?;
        Some(Self { r, g, b, a })
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StilIkone {
    pub boja: Option<BojaRgba>,
    pub mjerilo: f64,
    pub smjer: f64,
    pub putanja_ikone: Option<String>,
    pub sidriste: Option<(f64, f64)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StilCrte {
    pub boja: Option<BojaRgba>,
    pub sirina: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StilPoligona {
    pub boja: Option<BojaRgba>,
    pub ispuna: bool,
    pub obrub: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StilNatpisa {
    pub boja: Option<BojaRgba>,
    pub mjerilo: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StilBalona {
    pub boja_pozadine: Option<BojaRgba>,
    pub boja_teksta: Option<BojaRgba>,
    pub tekst: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stil {
    pub id: Option<String>,
    pub ikona: Option<StilIkone>,
    pub crta: Option<StilCrte>,
    pub poligon: Option<StilPoligona>,
    pub natpis: Option<StilNatpisa>,
    pub balon: Option<StilBalona>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ParStila {
    pub kljuc: String,
    pub stil_url: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapaStila {
    pub id: Option<String>,
    pub parovi: Vec<ParStila>,
}

/// Riješeni stil objekta (normalni + istaknuti ako dolazi iz `StyleMap`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RijeseniStil {
    pub normalni: Stil,
    pub istaknuti: Option<Stil>,
}

/// Pogled kamere (`LookAt` ili `Camera`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PogledKamere {
    pub je_kamera: bool,
    pub duzina: f64,
    pub sirina: f64,
    pub visina: f64,
    pub smjer: f64,
    pub nagib: f64,
    pub udaljenost_ili_rotacija: f64,
    pub nacin_visine: NacinVisine,
}

/// Vremenski podatak (`TimeStamp` ili `TimeSpan`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VrijemeKml {
    Trenutak(String),
    Raspon {
        pocetak: Option<String>,
        kraj: Option<String>,
    },
}

/// Jedan atribut iz `<ExtendedData>` (`Data`, `SimpleData` ili `gx:SimpleArrayData`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AtributPodatka {
    pub naziv: String,
    pub prikazni_naziv: Option<String>,
    pub vrijednosti: Vec<String>,
}

/// Mrežna veza (`NetworkLink`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MreznaVeza {
    pub naziv: Option<String>,
    pub opis: Option<String>,
    pub vidljivo: bool,
    pub putanja: Option<String>,
    pub nacin_osvjezavanja: Option<String>,
    pub stil: Option<Stil>,
}

/// Slikovni prekrivač (`GroundOverlay`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SlikovniPrekrivac {
    pub naziv: Option<String>,
    pub opis: Option<String>,
    pub vidljivo: bool,
    pub putanja_slike: Option<String>,
    pub pravokutnik: Option<(f64, f64, f64, f64, f64)>,
    pub cetverokut: Vec<Vrh>,
}

/// Vektorski objekt (`Placemark`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Objekt {
    pub id: Option<String>,
    pub naziv: Option<String>,
    pub opis: Option<String>,
    pub kratki_opis: Option<String>,
    pub vidljivo: bool,
    pub stil_url: Option<String>,
    pub izravni_stil: Option<Stil>,
    pub rijeseni_stil: Option<RijeseniStil>,
    pub geometrija: Option<Geometrija>,
    pub pogled: Option<PogledKamere>,
    pub vrijeme: Option<VrijemeKml>,
    pub atributi: Vec<AtributPodatka>,
}

/// Mapa ili dokument u stablu KML-a.
#[derive(Clone, Debug, PartialEq)]
pub struct Mapa {
    pub vrsta: VrstaMape,
    pub id: Option<String>,
    pub naziv: Option<String>,
    pub opis: Option<String>,
    pub vidljivo: bool,
    pub otvoreno: bool,
    pub stil_url: Option<String>,
    pub izravni_stil: Option<Stil>,
    pub stilovi: Vec<Stil>,
    pub mape_stilova: Vec<MapaStila>,
    pub pogled: Option<PogledKamere>,
    pub vrijeme: Option<VrijemeKml>,
    pub atributi: Vec<AtributPodatka>,
    pub podmape: Vec<Mapa>,
    pub objekti: Vec<Objekt>,
    pub mrezne_veze: Vec<MreznaVeza>,
    pub prekrivaci: Vec<SlikovniPrekrivac>,
}

impl Default for Mapa {
    fn default() -> Self {
        Self::nova(VrstaMape::Korijen)
    }
}

impl Mapa {
    pub fn nova(vrsta: VrstaMape) -> Self {
        Self {
            vrsta,
            id: None,
            naziv: None,
            opis: None,
            vidljivo: true,
            otvoreno: false,
            stil_url: None,
            izravni_stil: None,
            stilovi: Vec::new(),
            mape_stilova: Vec::new(),
            pogled: None,
            vrijeme: None,
            atributi: Vec::new(),
            podmape: Vec::new(),
            objekti: Vec::new(),
            mrezne_veze: Vec::new(),
            prekrivaci: Vec::new(),
        }
    }
}

/// Slika pronađena unutar KMZ arhive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlikaArhive {
    pub putanja: String,
    pub velicina_bajta: usize,
}

/// Zbirni brojevi pročitanog KML stabla (sigurni za ispis, bez privatnih tekstova).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatistikaKml {
    pub dokumenti: usize,
    pub mape: usize,
    pub najveca_dubina_mapa: usize,
    pub najveca_dubina_spremnika: usize,
    pub objekti: usize,
    pub objekti_s_nazivom: usize,
    pub objekti_s_opisom: usize,
    pub mape_s_opisom: usize,
    pub tocke: usize,
    pub linije: usize,
    pub poligoni: usize,
    pub rupe_poligona: usize,
    pub vise_geometrija: usize,
    pub tragovi: usize,
    pub objekti_bez_geometrije: usize,
    pub ukupno_vrhova: usize,
    pub vrhovi_s_visinom: usize,
    pub vrhovi_s_nenultom_visinom: usize,
    pub stilovi: usize,
    pub mape_stilova: usize,
    pub rijeseni_stilovi_objekata: usize,
    pub nerijeseni_stilovi_objekata: usize,
    pub mrezne_veze: usize,
    pub slikovni_prekrivaci: usize,
    pub pogledi_kamere: usize,
    pub vremenski_zapisi: usize,
    pub prosireni_podaci: usize,
    pub slike_u_kmz: usize,
}

/// Cijeli rezultat čitanja KML ili KMZ datoteke.
#[derive(Clone, Debug, Default)]
pub struct StabloKml {
    pub korijen: Mapa,
    pub slike_u_arhivi: Vec<SlikaArhive>,
    /// Poznati KML elementi koji se pri uvozu u čiste 2D vektorske slojeve ne prenose kao geometrija.
    pub izgubljeno_za_2d: BTreeMap<String, usize>,
    /// XML oznake koje čitač uopće ne poznaje (ništa ne smije proći tiho).
    pub nepoznati_elementi: BTreeMap<String, usize>,
    pub statistika: StatistikaKml,
}
