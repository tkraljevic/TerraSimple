//! Čitač KML-a na `quick-xml`. Čita jednim prolazom: prati put oznaka od korijena, a mape, objekte,
//! geometrije i stilove slaže kad im oznaka završi. Što se radi na kojoj oznaci, piše u tablicama
//! `OTVARANJE` i `ZATVARANJE`. Stilovi se pridružuju objektima tek na kraju, jer Google Earth
//! `StyleMap` često zapiše prije stila na koji pokazuje.

use crate::kmz;
use crate::model::{
    DokumentKml, GeometrijaKml, Gubici, MapaKml, ObjektKml, StilKml, boja_iz_kml, koordinata_traga,
    koordinate, otvoren_prsten,
};
use quick_xml::Reader;
use quick_xml::events::{BytesRef, BytesStart, Event};
use slojevi::Vrh;
use std::collections::BTreeMap;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Najdublje gniježđenje oznaka. `myplaces.kml` ima 8 razina mapa, a pravi KML rijetko više od 20.
const NAJVECA_DUBINA: usize = 256;

/// Oznake koje 2D karta ne prikazuje. Preskaču se cijele i broje u gubicima.
const PRESKOCI_I_BROJI: [&str; 6] = [
    "NetworkLink",
    "GroundOverlay",
    "ScreenOverlay",
    "PhotoOverlay",
    "Tour",
    "Model",
];

/// Oznake koje ne nose ništa za kartu, a mogle bi imati vlastiti `name` ili `coordinates`.
const PRESKOCI: [&str; 14] = [
    "LookAt",
    "Camera",
    "ExtendedData",
    "Snippet",
    "Region",
    "TimeSpan",
    "TimeStamp",
    "BalloonStyle",
    "ListStyle",
    "LabelStyle",
    "Schema",
    "author",
    "link",
    "Metadata",
];

/// Greška čitanja. Ne citira sadržaj datoteke, jer on može biti privatan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GreskaKml {
    Citanje,
    NeispravanXml {
        polozaj: u64,
    },
    PreDuboko,
    NeispravnaArhiva,
    NemaKmlUArhivi,
    NedopustenaPutanja,
    PrevelikaArhiva,
    NeispravanJson,
    /// GeoJSON je u koordinatnom sustavu koji program ne zna.
    NepoznatSustav,
}

impl GreskaKml {
    /// Opis greške u KMZ arhivi, a za ostale ništa.
    const fn opis_arhive(self) -> Option<&'static str> {
        match self {
            Self::NeispravnaArhiva => Some("KMZ nije ispravna ZIP arhiva"),
            Self::NemaKmlUArhivi => Some("u KMZ arhivi nema KML datoteke"),
            Self::NedopustenaPutanja => Some("KMZ arhiva ima nedopuštenu putanju"),
            Self::PrevelikaArhiva => Some("KMZ arhiva se raspakira u preveliku datoteku"),
            _ => None,
        }
    }
}

impl fmt::Display for GreskaKml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NeispravanXml { polozaj } => write!(f, "neispravan XML na bajtu {polozaj}"),
            Self::Citanje => f.write_str("datoteka se ne može pročitati"),
            Self::PreDuboko => f.write_str("oznake su ugniježđene predubok"),
            Self::NeispravanJson => f.write_str("GeoJSON nije ispravan"),
            Self::NepoznatSustav => f.write_str("koordinatni sustav nije poznat"),
            ostalo => f.write_str(ostalo.opis_arhive().unwrap_or_default()),
        }
    }
}

impl std::error::Error for GreskaKml {}

/// Čita KML, KMZ ili GeoJSON. Vrsta se prepoznaje po sadržaju, pa ni krivi nastavak ne smeta: KMZ
/// po potpisu ZIP-a, a GeoJSON po prvoj vitičastoj zagradi.
pub fn procitaj(staza: &Path) -> Result<DokumentKml, GreskaKml> {
    let datoteka = File::open(staza).map_err(|_| GreskaKml::Citanje)?;
    let mut ulaz = BufReader::with_capacity(64 * 1024, datoteka);
    let pocetak = ulaz.fill_buf().map(<[u8]>::to_vec).unwrap_or_default();
    // ZIP sam traži svoj popis na kraju datoteke, pa mu ne smeta što je početak već pročitan.
    if pocetak.starts_with(b"PK\x03\x04") {
        return kmz::procitaj(ulaz.into_inner());
    }
    if je_json(&pocetak) {
        return procitaj_json(ulaz);
    }
    procitaj_iz(ulaz)
}

/// Prvi znak nakon razmaka i oznake UTF-8 (BOM) je vitičasta zagrada.
fn je_json(pocetak: &[u8]) -> bool {
    let bez_bom = pocetak.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(pocetak);
    bez_bom.iter().find(|b| !b.is_ascii_whitespace()) == Some(&b'{')
}

fn procitaj_json(mut ulaz: impl std::io::Read) -> Result<DokumentKml, GreskaKml> {
    let mut sve = Vec::new();
    ulaz.read_to_end(&mut sve).map_err(|_| GreskaKml::Citanje)?;
    let bez_bom = sve.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&sve);
    crate::geojson::procitaj(bez_bom)
}

pub fn procitaj_tekst(tekst: &str) -> Result<DokumentKml, GreskaKml> {
    procitaj_iz(tekst.as_bytes())
}

pub(crate) fn procitaj_iz<B: BufRead>(ulaz: B) -> Result<DokumentKml, GreskaKml> {
    let mut xml = Reader::from_reader(ulaz);
    xml.config_mut().trim_text(false);
    let mut citac = Citac::default();
    let mut buf = Vec::with_capacity(4096);
    loop {
        buf.clear();
        let polozaj = xml.buffer_position();
        let pokvaren = |_| GreskaKml::NeispravanXml { polozaj };
        let dogadaj = xml.read_event_into(&mut buf).map_err(pokvaren)?;
        if !citac
            .dogadaj(&mut xml, dogadaj)
            .map_err(|g| g.uz(polozaj))?
        {
            return Ok(citac.zavrsi());
        }
    }
}

impl GreskaKml {
    /// Greška XML-a dobiva položaj tek ovdje, gdje se zna.
    const fn uz(self, polozaj: u64) -> Self {
        match self {
            Self::NeispravanXml { .. } => Self::NeispravanXml { polozaj },
            g => g,
        }
    }
}

type Otvaranje = fn(&mut Citac, &BytesStart<'_>);
type Zatvaranje = fn(&mut Citac, &str);

const OTVARANJE: [(&str, Otvaranje); 9] = [
    ("Document", |c, _| c.otvori_mapu(true)),
    ("Folder", |c, _| c.otvori_mapu(false)),
    ("Placemark", |c, _| c.otvori_objekt()),
    ("Polygon", |c, _| c.otvori_poligon()),
    ("Track", |c, _| c.trag = Some(Vec::new())),
    ("Style", |c, e| {
        c.stil = Some((atribut_id(e), StilKml::default()))
    }),
    ("StyleMap", |c, e| {
        c.mapa_stila = Some((atribut_id(e), None))
    }),
    ("Pair", |c, _| c.par = (None, None)),
    ("description", |c, _| c.u_opisu = 1),
];

const ZATVARANJE: [(&str, Zatvaranje); 19] = [
    ("name", |c, t| c.upisi_naziv(t, false)),
    ("description", |c, t| c.upisi_naziv(t, true)),
    ("coordinates", Citac::koordinate),
    ("coord", Citac::tocka_traga),
    ("Polygon", |c, _| c.zatvori_poligon()),
    ("Track", |c, _| c.zatvori_trag()),
    ("Placemark", |c, _| c.zatvori_objekt()),
    ("Document", |c, _| c.zatvori_dokument()),
    ("Folder", |c, _| c.zatvori_mapu()),
    ("styleUrl", Citac::adresa_stila),
    ("key", |c, t| c.par.0 = Some(t.to_owned())),
    ("Pair", |c, _| c.zatvori_par()),
    ("Style", |c, _| c.zatvori_stil()),
    ("StyleMap", |c, _| c.zatvori_mapu_stila()),
    ("color", |c, t| c.polje_stila("color", t)),
    ("width", |c, t| c.polje_stila("width", t)),
    ("fill", |c, t| c.polje_stila("fill", t)),
    ("outline", |c, t| c.polje_stila("outline", t)),
    ("scale", |c, t| c.polje_stila("scale", t)),
];

#[derive(Default)]
struct Citac {
    /// Put lokalnih naziva oznaka od korijena do trenutne.
    put: Vec<String>,
    /// Za svaki otvoreni `Document` ili `Folder`: je li otvorio novu mapu. Najviši `Document` je korijen.
    otvorene: Vec<bool>,
    korijen_zauzet: bool,
    mape: Vec<MapaKml>,
    objekt: Option<ObjektKml>,
    vanjski: Option<Vec<Vrh>>,
    rupe: Vec<Vec<Vrh>>,
    poligon_pokvaren: bool,
    trag: Option<Vec<Vrh>>,
    stil: Option<(String, StilKml)>,
    mapa_stila: Option<(String, Option<String>)>,
    par: (Option<String>, Option<String>),
    stilovi: BTreeMap<(usize, String), StilKml>,
    mape_stilova: BTreeMap<(usize, String), String>,
    /// Za svaki `Document` redni broj onoga u kojem je, a korijen datoteke je 0.
    nad: Vec<Option<usize>>,
    /// Otvoreni `Document`-i, od najvišeg.
    dosezi: Vec<usize>,
    /// Dubina oznaka unutar `description`, koje se čuvaju kao tekst.
    u_opisu: usize,
    tekst: String,
    gubici: Gubici,
}

impl Citac {
    /// Obrađuje jedan događaj. Vraća `false` na kraju datoteke.
    fn dogadaj<B: BufRead>(
        &mut self,
        xml: &mut Reader<B>,
        dogadaj: Event<'_>,
    ) -> Result<bool, GreskaKml> {
        match dogadaj {
            Event::Start(e) => self.otvorena_oznaka(xml, &e)?,
            Event::Empty(e) => self.prazna_oznaka(&e)?,
            Event::End(_) => self.kraj(),
            Event::Eof => return Ok(false),
            ostalo => self.sadrzaj(&ostalo),
        }
        Ok(true)
    }

    /// Preskočena oznaka čita se do svog kraja bez obrade.
    fn otvorena_oznaka<B: BufRead>(
        &mut self,
        xml: &mut Reader<B>,
        e: &BytesStart<'_>,
    ) -> Result<(), GreskaKml> {
        if !self.preskoci(e) {
            return self.pocetak(e);
        }
        let mut ostatak = Vec::new();
        xml.read_to_end_into(e.name(), &mut ostatak)
            .map(|_| ())
            .map_err(|_| GreskaKml::NeispravanXml { polozaj: 0 })
    }

    fn sadrzaj(&mut self, dogadaj: &Event<'_>) {
        match dogadaj {
            Event::Text(t) => self.tekst.push_str(t),
            Event::CData(t) => self.tekst.push_str(t),
            Event::GeneralRef(r) => self.entitet(r),
            _ => {}
        }
    }

    fn prazna_oznaka(&mut self, e: &BytesStart<'_>) -> Result<(), GreskaKml> {
        if !self.preskoci(e) {
            self.pocetak(e)?;
            self.kraj();
        }
        Ok(())
    }

    fn preskoci(&mut self, e: &BytesStart<'_>) -> bool {
        let ime = e.local_name();
        let ime = ime.as_ref();
        if self.u_opisu > 0 {
            return false;
        }
        if PRESKOCI_I_BROJI.contains(&ime) {
            *self.gubici.preskoceno.entry(ime.to_owned()).or_insert(0) += 1;
            return true;
        }
        PRESKOCI.contains(&ime)
    }

    fn roditelj(&self) -> &str {
        self.predak(1)
    }

    /// Naziv oznake `razina` mjesta iznad trenutne.
    fn predak(&self, razina: usize) -> &str {
        self.put
            .len()
            .checked_sub(razina + 1)
            .and_then(|i| self.put.get(i))
            .map_or("", String::as_str)
    }

    fn pocetak(&mut self, e: &BytesStart<'_>) -> Result<(), GreskaKml> {
        if self.put.len() >= NAJVECA_DUBINA {
            return Err(GreskaKml::PreDuboko);
        }
        let ime = e.local_name().as_ref().to_owned();
        if self.u_opisu > 0 {
            self.u_opisu += 1;
            self.tekst.push('<');
            self.tekst.push_str(e);
            self.tekst.push('>');
        } else {
            self.tekst.clear();
            if let Some((_, otvori)) = OTVARANJE.iter().find(|(n, _)| *n == ime) {
                otvori(self, e);
            }
        }
        self.put.push(ime);
        Ok(())
    }

    fn kraj(&mut self) {
        if self.u_opisu > 1 {
            self.u_opisu -= 1;
            let ime = self.put.pop().unwrap_or_default();
            self.tekst.push_str("</");
            self.tekst.push_str(&ime);
            self.tekst.push('>');
            return;
        }
        let tekst = std::mem::take(&mut self.tekst);
        let ime = self.put.last().cloned().unwrap_or_default();
        if let Some((_, zatvori)) = ZATVARANJE.iter().find(|(n, _)| *n == ime) {
            zatvori(self, tekst.trim());
        }
        self.put.pop();
    }

    fn entitet(&mut self, r: &BytesRef<'_>) {
        let naziv: &str = r;
        let znak = r.resolve_char_ref().ok().flatten().map(String::from);
        let rijeseno = znak
            .or_else(|| quick_xml::escape::resolve_xml_entity(naziv).map(str::to_owned))
            .unwrap_or_else(|| format!("&{naziv};"));
        self.tekst.push_str(&rijeseno);
    }

    fn doseg(&self) -> usize {
        self.dosezi.last().copied().unwrap_or(0)
    }

    fn otvori_mapu(&mut self, dokument: bool) {
        if self.mape.is_empty() {
            self.mape.push(MapaKml::default());
            self.nad.push(None);
        }
        if dokument {
            self.nad.push(Some(self.doseg()));
            self.dosezi.push(self.nad.len() - 1);
        }
        let korijen = dokument && !self.korijen_zauzet;
        self.korijen_zauzet = true;
        if !korijen {
            self.mape.push(MapaKml::default());
        }
        self.otvorene.push(!korijen);
    }

    fn otvori_objekt(&mut self) {
        self.objekt = Some(ObjektKml {
            doseg: self.doseg(),
            ..ObjektKml::default()
        });
    }

    fn otvori_poligon(&mut self) {
        self.vanjski = None;
        self.rupe.clear();
        self.poligon_pokvaren = false;
    }

    fn upisi_naziv(&mut self, tekst: &str, opis: bool) {
        self.u_opisu = 0;
        let cilj = match self.roditelj() {
            "Placemark" => self.objekt.as_mut().map(|o| (&mut o.naziv, &mut o.opis)),
            "Document" | "Folder" => self.mape.last_mut().map(|m| (&mut m.naziv, &mut m.opis)),
            _ => None,
        };
        if let Some((naziv, o)) = cilj {
            tekst.clone_into(if opis { o } else { naziv });
        }
    }

    fn koordinate(&mut self, tekst: &str) {
        let Some(vrhovi) = koordinate(tekst) else {
            self.gubici.neispravne_koordinate += 1;
            self.poligon_pokvaren |= self.predak(1) == "LinearRing";
            return;
        };
        match (self.predak(1), self.predak(2)) {
            ("Point", _) => self.dodaj_tocku(vrhovi),
            ("LinearRing", "outerBoundaryIs") => self.vanjski = Some(vrhovi),
            ("LinearRing", "innerBoundaryIs") => self.rupe.push(vrhovi),
            ("LineString" | "LinearRing", _) => self.dodaj_liniju(vrhovi),
            _ => {}
        }
    }

    /// Prazna geometrija se broji, a ispravna ide u objekt koji se upravo čita.
    fn dodaj_geometriju(&mut self, g: Option<GeometrijaKml>) {
        match g {
            Some(g) => self
                .objekt
                .iter_mut()
                .for_each(|o| o.geometrije.push(g.clone())),
            None => self.gubici.prazne_geometrije += 1,
        }
    }

    fn dodaj_tocku(&mut self, vrhovi: Vec<Vrh>) {
        let g = vrhovi.into_iter().next().map(GeometrijaKml::Tocka);
        self.dodaj_geometriju(g);
    }

    fn dodaj_liniju(&mut self, vrhovi: Vec<Vrh>) {
        let g = (vrhovi.len() >= 2).then_some(GeometrijaKml::Linija(vrhovi));
        self.dodaj_geometriju(g);
    }

    /// Jedna loša točka traga odbacuje cijeli trag.
    fn tocka_traga(&mut self, tekst: &str) {
        let Some(trag) = self.trag.as_mut() else {
            return;
        };
        match koordinata_traga(tekst) {
            Some(v) => trag.push(v),
            None => {
                self.gubici.neispravne_koordinate += 1;
                self.trag = None;
            }
        }
    }

    fn zatvori_trag(&mut self) {
        let trag = self.trag.take();
        trag.into_iter().for_each(|t| self.dodaj_liniju(t));
    }

    fn zatvori_poligon(&mut self) {
        if std::mem::take(&mut self.poligon_pokvaren) {
            return;
        }
        let vanjski = self.vanjski.take().and_then(otvoren_prsten);
        let rupe: Vec<_> = std::mem::take(&mut self.rupe)
            .into_iter()
            .map(otvoren_prsten)
            .collect();
        self.gubici.prazne_geometrije += rupe.iter().filter(|r| r.is_none()).count();
        let g = vanjski.map(|v| {
            let mut prstenovi = vec![v];
            prstenovi.extend(rupe.into_iter().flatten());
            GeometrijaKml::Poligon(prstenovi)
        });
        self.dodaj_geometriju(g);
    }

    fn zatvori_objekt(&mut self) {
        let objekt = self.objekt.take().unwrap_or_default();
        match self.mape.last_mut() {
            Some(mapa) if !objekt.geometrije.is_empty() => mapa.objekti.push(objekt),
            _ => self.gubici.bez_geometrije += 1,
        }
    }

    fn zatvori_dokument(&mut self) {
        self.dosezi.pop();
        self.zatvori_mapu();
    }

    fn zatvori_mapu(&mut self) {
        if self.otvorene.pop() == Some(true)
            && let Some(mapa) = self.mape.pop()
            && let Some(roditelj) = self.mape.last_mut()
        {
            roditelj.mape.push(mapa);
        }
    }

    fn adresa_stila(&mut self, tekst: &str) {
        let adresa = Some(tekst.to_owned());
        match self.roditelj() {
            "Pair" => self.par.1 = adresa,
            _ => self.postavi_adresu(adresa),
        }
    }

    fn postavi_adresu(&mut self, adresa: Option<String>) {
        self.objekt
            .iter_mut()
            .for_each(|o| o.adresa_stila.clone_from(&adresa));
    }

    fn zatvori_par(&mut self) {
        let (kljuc, adresa) = std::mem::take(&mut self.par);
        if kljuc.as_deref() == Some("normal")
            && let Some((_, normal)) = self.mapa_stila.as_mut()
        {
            *normal = adresa;
        }
    }

    fn zatvori_mapu_stila(&mut self) {
        let (id, normal) = self.mapa_stila.take().unwrap_or_default();
        if self.roditelj() == "Placemark" {
            self.postavi_adresu(normal);
        } else if let Some(normal) = normal {
            self.mape_stilova.insert((self.doseg(), id), normal);
        }
    }

    fn zatvori_stil(&mut self) {
        let (id, stil) = self.stil.take().unwrap_or_default();
        if self.roditelj() == "Placemark" {
            self.objekt.iter_mut().for_each(|o| o.stil = stil);
        } else {
            self.stilovi.insert((self.doseg(), id), stil);
        }
    }

    fn polje_stila(&mut self, ime: &str, tekst: &str) {
        let roditelj = self.roditelj().to_owned();
        if let Some((_, s)) = self.stil.as_mut() {
            upisi_polje(s, &roditelj, ime, tekst);
        }
    }

    fn zavrsi(mut self) -> DokumentKml {
        while !self.otvorene.is_empty() {
            self.zatvori_mapu();
        }
        let mut korijen = self.mape.into_iter().next().unwrap_or_default();
        let stilovi = Stilovi {
            stilovi: &self.stilovi,
            mape: &self.mape_stilova,
            nad: &self.nad,
        };
        pridruzi_stilove(&mut korijen, &stilovi);
        DokumentKml {
            korijen,
            gubici: self.gubici,
        }
    }
}

fn atribut_id(e: &BytesStart<'_>) -> String {
    let vrijednost = e
        .attributes()
        .flatten()
        .find(|a| a.key.local_name().as_ref() == "id")
        .map(|a| a.value.into_owned())
        .unwrap_or_default();
    quick_xml::escape::unescape(&vrijednost).map_or(vrijednost.clone(), |v| v.into_owned())
}

fn broj(tekst: &str) -> Option<f64> {
    tekst
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite() && *v >= 0.0)
}

type PoljeStila = fn(&mut StilKml, &str);

const POLJA_STILA: [(&str, &str, PoljeStila); 7] = [
    ("LineStyle", "color", |s, t| s.boja_crte = boja_iz_kml(t)),
    ("LineStyle", "width", |s, t| s.sirina_px = broj(t)),
    ("PolyStyle", "color", |s, t| s.boja_ispune = boja_iz_kml(t)),
    ("PolyStyle", "fill", |s, t| s.bez_ispune = t == "0"),
    ("PolyStyle", "outline", |s, t| s.bez_ruba = t == "0"),
    ("IconStyle", "color", |s, t| s.boja_ikone = boja_iz_kml(t)),
    ("IconStyle", "scale", |s, t| s.mjerilo_ikone = broj(t)),
];

fn upisi_polje(s: &mut StilKml, roditelj: &str, ime: &str, tekst: &str) {
    if let Some((_, _, upisi)) = POLJA_STILA
        .iter()
        .find(|(r, i, _)| *r == roditelj && *i == ime)
    {
        upisi(s, tekst);
    }
}

struct Stilovi<'a> {
    stilovi: &'a BTreeMap<(usize, String), StilKml>,
    mape: &'a BTreeMap<(usize, String), String>,
    nad: &'a [Option<usize>],
}

enum Nadjeno<'a> {
    Stil(StilKml),
    Mapa(&'a str),
}

/// Naziv stila iz `#id` ili `datoteka.kml#id`.
fn id_iz_adrese(adresa: &str) -> &str {
    adresa.rsplit('#').next().unwrap_or(adresa)
}

impl<'a> Stilovi<'a> {
    /// `StyleMap` može pokazivati na drugi `StyleMap`, ali ne beskonačno.
    fn rijesi(&self, doseg: usize, adresa: &str) -> Option<StilKml> {
        let (mut id, mut doseg) = (id_iz_adrese(adresa), doseg);
        for _ in 0..4 {
            match self.nadji(doseg, id)? {
                (Nadjeno::Stil(s), _) => return Some(s),
                (Nadjeno::Mapa(adresa), gdje) => (id, doseg) = (id_iz_adrese(adresa), gdje),
            }
        }
        None
    }

    /// Traži naziv u `Document`-u objekta, pa u onima oko njega.
    fn nadji(&self, doseg: usize, id: &str) -> Option<(Nadjeno<'a>, usize)> {
        let mut d = Some(doseg);
        while let Some(s) = d {
            let kljuc = (s, id.to_owned());
            let stil = self.stilovi.get(&kljuc).map(|st| Nadjeno::Stil(*st));
            let nadjeno = stil.or_else(|| self.mape.get(&kljuc).map(|a| Nadjeno::Mapa(a)));
            if let Some(n) = nadjeno {
                return Some((n, s));
            }
            d = self.nad.get(s).copied().flatten();
        }
        None
    }
}

/// Stil objekta je zajednički stil, a ono što zadaje vlastiti stil ima prednost.
fn spoji(zajednicki: StilKml, vlastiti: StilKml) -> StilKml {
    StilKml {
        boja_crte: vlastiti.boja_crte.or(zajednicki.boja_crte),
        sirina_px: vlastiti.sirina_px.or(zajednicki.sirina_px),
        boja_ispune: vlastiti.boja_ispune.or(zajednicki.boja_ispune),
        bez_ispune: vlastiti.bez_ispune || zajednicki.bez_ispune,
        bez_ruba: vlastiti.bez_ruba || zajednicki.bez_ruba,
        boja_ikone: vlastiti.boja_ikone.or(zajednicki.boja_ikone),
        mjerilo_ikone: vlastiti.mjerilo_ikone.or(zajednicki.mjerilo_ikone),
    }
}

/// Obilazi stablo bez rekurzije, da duboko stablo ne napuni stog.
fn pridruzi_stilove(mapa: &mut MapaKml, stilovi: &Stilovi<'_>) {
    let mut stog = vec![mapa];
    while let Some(m) = stog.pop() {
        for objekt in &mut m.objekti {
            let zajednicki = objekt
                .adresa_stila
                .as_deref()
                .and_then(|a| stilovi.rijesi(objekt.doseg, a))
                .unwrap_or_default();
            objekt.stil = spoji(zajednicki, objekt.stil);
        }
        stog.extend(m.mape.iter_mut());
    }
}

#[cfg(test)]
#[path = "citac_testovi.rs"]
mod testovi;
