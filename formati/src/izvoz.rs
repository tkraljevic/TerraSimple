//! Izvoz u KML i KMZ za Google Earth. Mapa postaje mapa (`Folder`), a sloj također, osim sloja s
//! nazivom koji daje uvoz (Točke, Linije, Poligoni): njegovi objekti idu ravno u mapu, pa se uvezena
//! datoteka vraća u istom obliku. Stil sloja je zajednički stil, a vlastiti stil objekta piše se uz
//! objekt. Stupci koje je donio GeoPackage idu u `ExtendedData`.
//!
//! KML ne zna sve što zna projekt, pa [`pregledaj`] prije izvoza kaže što se mijenja ili gubi.

use crate::uvoz::NaziviSlojeva;
use koordinate::Sustav;
use slojevi::{
    Boja, Geometrija, IdMape, IdSloja, Objekt, OblikZnaka, Projekt, Sloj, Stavka, Stil,
    StilObjekta, Vrh, Vrijednost, VrstaCrte, VrstaGeometrije, VrstaIspune,
};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;

/// Što se izvozi: cijeli projekt ili jedna mapa ili sloj sa svime u sebi.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpsegIzvoza {
    Projekt,
    Stavka(Stavka),
}

/// Što ide u datoteku i što se usput mijenja ili gubi.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PregledIzvoza {
    pub tocaka: usize,
    pub linija: usize,
    pub poligona: usize,
    pub mapa: usize,
    /// Nazivi stupaca koji idu u `ExtendedData`.
    pub atributi: BTreeSet<String>,
    /// Vrijednosti koje nisu tekst ni broj, npr. slika u stupcu. KML ih ne može zapisati.
    pub izostavljene_vrijednosti: usize,
    /// Neki sloj je u datoteci bio u drugom sustavu, npr. HTRS96/TM. KML zna samo WGS84.
    pub drugi_sustav: bool,
    /// Objekti s isprekidanom, točkastom ili crta-točka crtom.
    pub isprekidane: usize,
    pub srafure: usize,
    /// Slojevi linija ili likova s uključenim natpisom. Google Earth naziv pokazuje samo uz točke.
    pub natpisi_bez_tocaka: usize,
    /// Slojevi s natpisom u kurzivu.
    pub kurziv: usize,
    /// Slojevi koji imaju barem jedan objekt. U SHP svaki ide u svoju datoteku.
    pub slojeva: usize,
    /// Objekti s visinom. SHP ih zapisuje bez nje.
    pub visine: usize,
}

impl PregledIzvoza {
    #[must_use]
    pub const fn objekata(&self) -> usize {
        self.tocaka + self.linija + self.poligona
    }
}

#[derive(Debug)]
pub enum GreskaIzvoza {
    /// Datoteka se ne može napisati, npr. nema prava ili mjesta.
    Pisanje,
}

impl From<std::io::Error> for GreskaIzvoza {
    fn from(_: std::io::Error) -> Self {
        Self::Pisanje
    }
}

impl From<zip::result::ZipError> for GreskaIzvoza {
    fn from(_: zip::result::ZipError) -> Self {
        Self::Pisanje
    }
}

const ZAMJENE: [(char, &str); 5] = [
    ('&', "&amp;"),
    ('<', "&lt;"),
    ('>', "&gt;"),
    ('"', "&quot;"),
    ('\'', "&apos;"),
];

fn bijeg(t: &str) -> String {
    t.chars().fold(String::with_capacity(t.len()), |mut s, z| {
        match ZAMJENE.iter().find(|(c, _)| *c == z) {
            Some((_, zamjena)) => s.push_str(zamjena),
            None => s.push(z),
        }
        s
    })
}

/// Boja u KML-u je zapisana kao `aabbggrr`.
#[must_use]
pub fn boja_u_kml(b: Boja) -> String {
    format!("{:02x}{:02x}{:02x}{:02x}", b.a, b.b, b.g, b.r)
}

/// Širina crte u pikselima zaslona, na 96 točaka po inču, kako je čita i uvoz.
fn pikseli(mm: f64) -> f64 {
    (mm * 96.0 / 25.4 * 100.0).round() / 100.0
}

/// Ikone iz zbirke Google Eartha, najsličnije obliku znaka.
const fn ikona(o: OblikZnaka) -> &'static str {
    match o {
        OblikZnaka::Krug => "http://maps.google.com/mapfiles/kml/shapes/placemark_circle.png",
        OblikZnaka::Kvadrat => "http://maps.google.com/mapfiles/kml/shapes/square.png",
        OblikZnaka::Trokut => "http://maps.google.com/mapfiles/kml/shapes/triangle.png",
        OblikZnaka::Zvijezda => "http://maps.google.com/mapfiles/kml/shapes/star.png",
    }
}

/// Sadržaj `Style` za vrstu sloja. Uvoz iz mjerila ikone računa veličinu kao 3 mm po jedinici.
fn tijelo_stila(s: &Stil, vrsta: VrstaGeometrije) -> String {
    let natpis = if s.natpis.prikazi {
        s.natpis.velicina_pt / 10.0
    } else {
        0.0
    };
    let mut x = format!(
        "<LabelStyle><color>{}</color><scale>{natpis}</scale></LabelStyle>",
        boja_u_kml(s.natpis.boja)
    );
    match vrsta {
        VrstaGeometrije::Tocka => {
            let _ = write!(
                x,
                "<IconStyle><color>{}</color><scale>{}</scale><Icon><href>{}</href></Icon></IconStyle>",
                boja_u_kml(s.boja_crte),
                s.velicina_znaka_mm / 3.0,
                ikona(s.oblik_znaka)
            );
        }
        VrstaGeometrije::Linija => x.push_str(&stil_crte(s)),
        VrstaGeometrije::Poligon => {
            x.push_str(&stil_crte(s));
            let _ = write!(
                x,
                "<PolyStyle><color>{}</color><fill>{}</fill><outline>{}</outline></PolyStyle>",
                boja_u_kml(s.boja_ispune),
                u8::from(s.vrsta_ispune != VrstaIspune::Bez),
                u8::from(s.boja_crte.a > 0)
            );
        }
    }
    x
}

fn stil_crte(s: &Stil) -> String {
    format!(
        "<LineStyle><color>{}</color><width>{}</width></LineStyle>",
        boja_u_kml(s.boja_crte),
        pikseli(s.debljina_mm)
    )
}

fn koordinate(vrhovi: &[Vrh]) -> String {
    let mut s = String::with_capacity(vrhovi.len() * 24);
    for v in vrhovi {
        if !s.is_empty() {
            s.push(' ');
        }
        let _ = write!(s, "{},{}", v.duzina, v.sirina);
        if let Some(h) = v.visina {
            let _ = write!(s, ",{h}");
        }
    }
    s
}

/// KML prsten se zatvara ponovljenom prvom točkom.
fn prsten(vrhovi: &[Vrh]) -> String {
    let mut zatvoren = vrhovi.to_vec();
    if zatvoren.first() != zatvoren.last() {
        zatvoren.extend(vrhovi.first().copied());
    }
    format!(
        "<LinearRing><coordinates>{}</coordinates></LinearRing>",
        koordinate(&zatvoren)
    )
}

fn geometrija(g: &Geometrija) -> String {
    match g {
        Geometrija::Tocka(v) => format!(
            "<Point><coordinates>{}</coordinates></Point>",
            koordinate(&[*v])
        ),
        Geometrija::Linija(v) => format!(
            "<LineString><tessellate>1</tessellate><coordinates>{}</coordinates></LineString>",
            koordinate(v)
        ),
        Geometrija::Poligon(p) => {
            let mut x = String::from("<Polygon><tessellate>1</tessellate>");
            for (i, r) in p.iter().enumerate() {
                let rub = if i == 0 { "outer" } else { "inner" };
                let _ = write!(x, "<{rub}BoundaryIs>{}</{rub}BoundaryIs>", prsten(r));
            }
            x.push_str("</Polygon>");
            x
        }
    }
}

fn vrijednost(v: &Vrijednost) -> Option<String> {
    match v {
        Vrijednost::Prazno => Some(String::new()),
        Vrijednost::Cijeli(n) => Some(n.to_string()),
        Vrijednost::Broj(n) => Some(n.to_string()),
        Vrijednost::Tekst(t) => Some(bijeg(t)),
        Vrijednost::Bajtovi(_) => None,
    }
}

/// Piše dokument i usput broji ono što ide u pregled.
struct Pisac<'a> {
    p: &'a Projekt,
    nazivi: &'a NaziviSlojeva,
    stilovi: String,
    tijelo: String,
    pregled: PregledIzvoza,
}

impl Pisac<'_> {
    fn stavka(&mut self, s: Stavka, u_mapi: bool) {
        match s {
            Stavka::Mapa(m) => self.mapa(m),
            Stavka::Sloj(id) => self.sloj(id, u_mapi),
        }
    }

    fn mapa(&mut self, id: IdMape) {
        let Some(m) = self.p.mapa(id) else {
            return;
        };
        self.pregled.mapa += 1;
        let _ = write!(self.tijelo, "<Folder><name>{}</name>", bijeg(&m.naziv));
        for d in &m.djeca {
            self.stavka(*d, true);
        }
        self.tijelo.push_str("</Folder>");
    }

    /// Sloj s nazivom iz uvoza unutar mape ne dobiva svoju mapu.
    fn vlastita_mapa(&self, s: &Sloj, u_mapi: bool) -> bool {
        let n = self.nazivi;
        !u_mapi || ![&n.tocke, &n.linije, &n.poligoni].contains(&&s.naziv)
    }

    fn sloj(&mut self, id: IdSloja, u_mapi: bool) {
        let Some(s) = self.p.sloj(id) else {
            return;
        };
        self.zabiljezi_sloj(id, s);
        let _ = write!(
            self.stilovi,
            "<Style id=\"sloj{}\">{}</Style>",
            id.0,
            tijelo_stila(&s.stil, s.vrsta)
        );
        let mapa = self.vlastita_mapa(s, u_mapi);
        if mapa {
            let _ = write!(self.tijelo, "<Folder><name>{}</name>", bijeg(&s.naziv));
        }
        for o in s.objekti.values() {
            self.objekt(id, s, o);
        }
        if mapa {
            self.tijelo.push_str("</Folder>");
        }
    }

    fn zabiljezi_sloj(&mut self, id: IdSloja, s: &Sloj) {
        let n = &s.stil.natpis;
        let ima = !s.objekti.is_empty();
        let pr = &mut self.pregled;
        pr.slojeva += usize::from(ima);
        pr.natpisi_bez_tocaka += usize::from(ima && n.prikazi && s.vrsta != VrstaGeometrije::Tocka);
        pr.kurziv += usize::from(ima && n.prikazi && n.kurziv);
        let izvor = self.p.zapisi().slojevi.get(&id);
        pr.drugi_sustav |= ima && izvor.is_some_and(|z| z.sustav != Sustav::Wgs84);
    }

    fn zabiljezi_objekt(&mut self, s: &Sloj, o: &Objekt, stil: &Stil) {
        let pr = &mut self.pregled;
        let visina = crate::shp_izvoz::vrhovi(&o.geometrija).any(|v| v.visina.is_some());
        pr.visine += usize::from(visina);
        match s.vrsta {
            VrstaGeometrije::Tocka => pr.tocaka += 1,
            VrstaGeometrije::Linija => pr.linija += 1,
            VrstaGeometrije::Poligon => pr.poligona += 1,
        }
        let crta = s.vrsta != VrstaGeometrije::Tocka;
        pr.isprekidane += usize::from(crta && stil.vrsta_crte != VrstaCrte::Puna);
        pr.srafure += usize::from(
            stil.vrsta_ispune == VrstaIspune::Srafura && s.vrsta == VrstaGeometrije::Poligon,
        );
    }

    fn objekt(&mut self, id: IdSloja, s: &Sloj, o: &Objekt) {
        let stil = o.stil.na_stilu_sloja(&s.stil);
        self.zabiljezi_objekt(s, o, &stil);
        let _ = write!(
            self.tijelo,
            "<Placemark><name>{}</name><styleUrl>#sloj{}</styleUrl>",
            bijeg(&o.naziv),
            id.0
        );
        if !o.opis.is_empty() {
            let _ = write!(self.tijelo, "<description>{}</description>", bijeg(&o.opis));
        }
        if o.stil != StilObjekta::default() {
            let _ = write!(
                self.tijelo,
                "<Style>{}</Style>",
                tijelo_stila(&stil, s.vrsta)
            );
        }
        self.atributi(id, o);
        self.tijelo.push_str(&geometrija(&o.geometrija));
        self.tijelo.push_str("</Placemark>");
    }

    fn atributi(&mut self, id: IdSloja, o: &Objekt) {
        let Some(z) = self
            .p
            .zapisi()
            .slojevi
            .get(&id)
            .and_then(|z| z.objekti.get(&o.id))
        else {
            return;
        };
        if z.nepoznati.is_empty() {
            return;
        }
        self.tijelo.push_str("<ExtendedData>");
        for (naziv, v) in &z.nepoznati {
            let Some(tekst) = vrijednost(v) else {
                self.pregled.izostavljene_vrijednosti += 1;
                continue;
            };
            self.pregled.atributi.insert(naziv.clone());
            let _ = write!(
                self.tijelo,
                "<Data name=\"{}\"><value>{tekst}</value></Data>",
                bijeg(naziv)
            );
        }
        self.tijelo.push_str("</ExtendedData>");
    }
}

fn napisi<'a>(p: &'a Projekt, opseg: OpsegIzvoza, nazivi: &'a NaziviSlojeva) -> Pisac<'a> {
    let mut pisac = Pisac {
        p,
        nazivi,
        stilovi: String::new(),
        tijelo: String::new(),
        pregled: PregledIzvoza::default(),
    };
    match opseg {
        OpsegIzvoza::Projekt => p.korijen().iter().for_each(|s| pisac.stavka(*s, false)),
        OpsegIzvoza::Stavka(s) => pisac.stavka(s, false),
    }
    pisac
}

/// Što bi izvoz zapisao i što bi se izgubilo, bez pisanja datoteke.
#[must_use]
pub fn pregledaj(p: &Projekt, opseg: OpsegIzvoza, nazivi: &NaziviSlojeva) -> PregledIzvoza {
    napisi(p, opseg, nazivi).pregled
}

/// Cijeli KML dokument. `naziv` je naziv dokumenta, npr. naziv projekta.
#[must_use]
pub fn kml(p: &Projekt, opseg: OpsegIzvoza, naziv: &str, nazivi: &NaziviSlojeva) -> String {
    dokument(&napisi(p, opseg, nazivi), naziv)
}

fn dokument(pisac: &Pisac<'_>, naziv: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<kml xmlns=\"http://www.opengis.net/kml/2.2\"><Document><name>{}</name>{}{}</Document></kml>\n",
        bijeg(naziv),
        pisac.stilovi,
        pisac.tijelo
    )
}

fn nastavak(staza: &Path, n: &[&str]) -> bool {
    staza
        .extension()
        .is_some_and(|e| n.iter().any(|n| e.eq_ignore_ascii_case(n)))
}

/// KMZ je ZIP s jednom datotekom `doc.kml`.
fn zapakiraj<W: std::io::Write + std::io::Seek>(
    izlaz: W,
    sadrzaj: &[u8],
) -> Result<W, GreskaIzvoza> {
    let mut zip = zip::ZipWriter::new(izlaz);
    let opcije = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("doc.kml", opcije)
        .and_then(|()| zip.write_all(sadrzaj).map_err(zip::result::ZipError::from))
        .and_then(|()| zip.finish())
        .map_err(GreskaIzvoza::from)
}

/// Piše KML, KMZ ili GeoJSON, prema nastavku datoteke (`.kmz`, `.geojson` ili `.json`).
/// Piše se u privremenu datoteku pa se preimenuje, da prekid usred pisanja ne ostavi pola datoteke pod pravim imenom.
pub fn izvezi(
    p: &Projekt,
    opseg: OpsegIzvoza,
    naziv: &str,
    nazivi: &NaziviSlojeva,
    staza: &Path,
) -> Result<PregledIzvoza, GreskaIzvoza> {
    let pisac = napisi(p, opseg, nazivi);
    let bajtovi = if nastavak(staza, &["geojson", "json"]) {
        crate::geojson::napisi(p, opseg, naziv).into_bytes()
    } else if nastavak(staza, &["kmz"]) {
        let kml = dokument(&pisac, naziv);
        zapakiraj(std::io::Cursor::new(Vec::new()), kml.as_bytes())?.into_inner()
    } else {
        dokument(&pisac, naziv).into_bytes()
    };
    let privremena = staza.with_extension("izvoz-u-tijeku");
    let ishod = std::fs::File::create(&privremena)
        .and_then(|mut d| d.write_all(&bajtovi).and_then(|()| d.sync_all()))
        .and_then(|()| std::fs::rename(&privremena, staza));
    if ishod.is_err() {
        let _ = std::fs::remove_file(&privremena);
    }
    ishod.map(|()| pisac.pregled).map_err(GreskaIzvoza::from)
}

#[cfg(test)]
#[path = "izvoz_testovi.rs"]
mod testovi;
