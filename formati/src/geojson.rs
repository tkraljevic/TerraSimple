//! GeoJSON. Uvoz ga pretvara u isti oblik kao KML, pa dalje ide istim putem: sve u novu mapu, a
//! objekti u slojeve Točke, Linije i Poligoni. Izvoz piše jednu zbirku objekata u WGS84 (RFC 7946),
//! a mapa i sloj objekta idu u svojstva `mapa` i `sloj`.
//!
//! Stari GeoJSON smije zadati drugi sustav (`crs`), npr. HTRS96/TM. Takve koordinate se preračunaju,
//! a sustav koji program ne zna je greška, da se ništa ne nacrta na krivom mjestu.

use crate::citac::GreskaKml;
use crate::izvoz::OpsegIzvoza;
use crate::model::{
    DokumentKml, GeometrijaKml, Gubici, MapaKml, ObjektKml, StilKml, ispravan, otvoren_prsten,
};
use koordinate::{Pretvornik, Sustav};
use serde_json::{Map, Value, json};
use slojevi::{
    Boja, Geometrija, IdSloja, Objekt, Projekt, Sloj, Stavka, Stil, Vrh, Vrijednost, VrstaIspune,
};

/// Svojstva iz kojih objekt dobiva naziv, redom.
const NAZIVI: [&str; 5] = ["name", "naziv", "ime", "title", "oznaka"];
const OPISI: [&str; 2] = ["description", "opis"];
/// Stil po dogovoru `simplestyle`, koji čitaju geojson.io i GitHub.
const STIL: [&str; 7] = [
    "stroke",
    "stroke-width",
    "stroke-opacity",
    "fill",
    "fill-opacity",
    "marker-color",
    "marker-size",
];

fn svojstvo<'a>(p: &'a Map<String, Value>, kljucevi: &[&str]) -> Option<&'a str> {
    kljucevi.iter().find_map(|k| {
        p.iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(k))
            .and_then(|(_, v)| v.as_str())
    })
}

/// Sustav iz `crs`. Bez njega vrijedi WGS84.
fn sustav(v: &Value) -> Result<Option<Pretvornik>, GreskaKml> {
    let Some(naziv) = v.pointer("/crs/properties/name").and_then(Value::as_str) else {
        return Ok(None);
    };
    let kraj = naziv.rsplit(':').next().unwrap_or_default();
    let s = if kraj.eq_ignore_ascii_case("CRS84") {
        Some(Sustav::Wgs84)
    } else {
        kraj.parse().ok().and_then(Sustav::iz_epsg)
    };
    match s {
        None => Err(GreskaKml::NepoznatSustav),
        Some(Sustav::Wgs84) => Ok(None),
        Some(s) => Pretvornik::novi(s, Sustav::Wgs84)
            .map(Some)
            .map_err(|_| GreskaKml::NepoznatSustav),
    }
}

/// Čita GeoJSON i pretvara ga u isti oblik kao KML.
pub(crate) fn procitaj(tekst: &[u8]) -> Result<DokumentKml, GreskaKml> {
    let v: Value = serde_json::from_slice(tekst).map_err(|_| GreskaKml::NeispravanJson)?;
    let pretvornik = sustav(&v)?;
    let znacajke: Vec<Value> = match v.get("type").and_then(Value::as_str) {
        Some("FeatureCollection") => v
            .get("features")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        Some("Feature") => vec![v.clone()],
        Some(_) => vec![json!({"type": "Feature", "geometry": v, "properties": {}})],
        None => return Err(GreskaKml::NeispravanJson),
    };
    let mut c = Citac {
        pretvornik,
        gubici: Gubici::default(),
    };
    let objekti = znacajke.iter().filter_map(|z| c.objekt(z)).collect();
    let naziv = v.get("name").and_then(Value::as_str).unwrap_or_default();
    Ok(DokumentKml {
        korijen: MapaKml {
            naziv: naziv.to_owned(),
            objekti,
            ..MapaKml::default()
        },
        gubici: c.gubici,
    })
}

struct Citac {
    pretvornik: Option<Pretvornik>,
    gubici: Gubici,
}

impl Citac {
    fn objekt(&mut self, z: &Value) -> Option<ObjektKml> {
        let prazna = Map::new();
        let p = z
            .get("properties")
            .and_then(Value::as_object)
            .unwrap_or(&prazna);
        let mut geometrije = Vec::new();
        if let Some(g) = z.get("geometry") {
            self.geometrija(g, &mut geometrije);
        }
        if geometrije.is_empty() {
            self.gubici.bez_geometrije += 1;
            return None;
        }
        let poznato = |k: &String| {
            [NAZIVI.as_slice(), &OPISI, &STIL]
                .concat()
                .iter()
                .any(|p| k.eq_ignore_ascii_case(p))
        };
        let ostala = p.keys().filter(|k| !poznato(k)).cloned();
        self.gubici.izostavljena_svojstva.extend(ostala);
        Some(ObjektKml {
            naziv: svojstvo(p, &NAZIVI).unwrap_or_default().to_owned(),
            opis: svojstvo(p, &OPISI).unwrap_or_default().to_owned(),
            stil: stil(p),
            geometrije,
            ..ObjektKml::default()
        })
    }

    fn vrh(&self, v: &Value) -> Option<Vrh> {
        let b = v.as_array()?;
        let broj = |i: usize| b.get(i).and_then(Value::as_f64);
        broj(0)
            .zip(broj(1))
            .and_then(|(x, y)| self.pretvori([x, y]))
            .and_then(|[d, s]| ispravan(d, s, broj(2)))
    }

    fn pretvori(&self, t: [f64; 2]) -> Option<[f64; 2]> {
        match &self.pretvornik {
            Some(p) => p.pretvori(t).ok(),
            None => Some(t),
        }
    }

    /// Niz točaka. Jedna loša točka odbacuje cijeli niz.
    fn vrhovi(&mut self, v: &Value) -> Option<Vec<Vrh>> {
        let ishod = v
            .as_array()
            .and_then(|a| a.iter().map(|t| self.vrh(t)).collect::<Option<Vec<_>>>());
        if ishod.is_none() {
            self.gubici.neispravne_koordinate += 1;
        }
        ishod
    }

    fn dodaj(&mut self, g: Option<GeometrijaKml>, izlaz: &mut Vec<GeometrijaKml>) {
        match g {
            Some(g) => izlaz.push(g),
            None => self.gubici.prazne_geometrije += 1,
        }
    }

    fn tocka(&mut self, k: &Value, izlaz: &mut Vec<GeometrijaKml>) {
        let v = self.vrhovi(&json!([k]));
        if let Some(v) = v {
            izlaz.extend(v.into_iter().map(GeometrijaKml::Tocka));
        }
    }

    fn linija(&mut self, k: &Value, izlaz: &mut Vec<GeometrijaKml>) {
        if let Some(v) = self.vrhovi(k) {
            let g = (v.len() >= 2).then_some(GeometrijaKml::Linija(v));
            self.dodaj(g, izlaz);
        }
    }

    fn poligon(&mut self, k: &Value, izlaz: &mut Vec<GeometrijaKml>) {
        let prstenovi: Option<Vec<Vec<Vrh>>> = k
            .as_array()
            .map(|a| a.iter().filter_map(|r| self.vrhovi(r)).collect());
        let mut prstenovi = prstenovi
            .unwrap_or_default()
            .into_iter()
            .map(otvoren_prsten);
        let g = prstenovi.next().flatten().map(|vanjski| {
            let mut svi = vec![vanjski];
            svi.extend(prstenovi.flatten());
            GeometrijaKml::Poligon(svi)
        });
        self.dodaj(g, izlaz);
    }

    fn dijelovi(
        &mut self,
        k: &Value,
        izlaz: &mut Vec<GeometrijaKml>,
        dio: fn(&mut Self, &Value, &mut Vec<GeometrijaKml>),
    ) {
        for d in k.as_array().into_iter().flatten() {
            dio(self, d, izlaz);
        }
    }

    /// Skupna geometrija daje po jedan objekt na karti za svaki dio, kao `MultiGeometry` u KML-u.
    fn geometrija(&mut self, g: &Value, izlaz: &mut Vec<GeometrijaKml>) {
        let vrsta = g.get("type").and_then(Value::as_str).unwrap_or_default();
        if vrsta == "GeometryCollection" {
            let dijelovi = g.get("geometries").cloned().unwrap_or_default();
            return self.dijelovi(&dijelovi, izlaz, Self::geometrija);
        }
        let k = g.get("coordinates").unwrap_or(&Value::Null);
        match VRSTE.iter().find(|(n, _, _)| *n == vrsta) {
            Some((_, dio, true)) => self.dijelovi(k, izlaz, *dio),
            Some((_, dio, false)) => dio(self, k, izlaz),
            None => self.gubici.prazne_geometrije += 1,
        }
    }
}

type Dio = fn(&mut Citac, &Value, &mut Vec<GeometrijaKml>);

/// Vrste geometrije: naziv, kako se čita jedan dio i je li skupna.
const VRSTE: [(&str, Dio, bool); 6] = [
    ("Point", Citac::tocka, false),
    ("MultiPoint", Citac::tocka, true),
    ("LineString", Citac::linija, false),
    ("MultiLineString", Citac::linija, true),
    ("Polygon", Citac::poligon, false),
    ("MultiPolygon", Citac::poligon, true),
];

/// `#rrggbb` i prozirnost od 0 do 1.
fn boja(tekst: Option<&str>, prozirnost: Option<f64>) -> Option<Boja> {
    let t = tekst?.strip_prefix('#')?;
    let bajt = |i: usize| t.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok());
    Some(Boja {
        r: bajt(0)?,
        g: bajt(2)?,
        b: bajt(4)?,
        a: (prozirnost.unwrap_or(1.0).clamp(0.0, 1.0) * 255.0).round() as u8,
    })
}

fn stil(p: &Map<String, Value>) -> StilKml {
    let broj = |k: &str| p.get(k).and_then(Value::as_f64);
    let tekst = |k: &str| p.get(k).and_then(Value::as_str);
    let mjerilo = tekst("marker-size").and_then(|v| match v {
        "small" => Some(0.75),
        "large" => Some(1.5),
        "medium" => Some(1.0),
        _ => None,
    });
    StilKml {
        boja_crte: boja(tekst("stroke"), broj("stroke-opacity")),
        sirina_px: broj("stroke-width"),
        boja_ispune: boja(tekst("fill"), broj("fill-opacity")),
        boja_ikone: boja(tekst("marker-color"), None),
        mjerilo_ikone: mjerilo,
        ..StilKml::default()
    }
}

fn polozaji(vrhovi: &[Vrh]) -> Value {
    Value::Array(
        vrhovi
            .iter()
            .map(|v| match v.visina {
                Some(h) => json!([v.duzina, v.sirina, h]),
                None => json!([v.duzina, v.sirina]),
            })
            .collect(),
    )
}

/// GeoJSON prsten se zatvara ponovljenom prvom točkom.
fn prsten(r: &[Vrh]) -> Value {
    let mut z = r.to_vec();
    if z.first() != z.last() {
        z.extend(r.first().copied());
    }
    polozaji(&z)
}

fn u_geometriju(g: &Geometrija) -> Value {
    match g {
        Geometrija::Tocka(v) => json!({"type": "Point", "coordinates": polozaji(&[*v])[0]}),
        Geometrija::Linija(v) => json!({"type": "LineString", "coordinates": polozaji(v)}),
        Geometrija::Poligon(p) => {
            let prstenovi: Vec<Value> = p.iter().map(|r| prsten(r)).collect();
            json!({"type": "Polygon", "coordinates": prstenovi})
        }
    }
}

fn hex(b: Boja) -> String {
    format!("#{:02x}{:02x}{:02x}", b.r, b.g, b.b)
}

fn u_vrijednost(v: &Vrijednost) -> Option<Value> {
    match v {
        Vrijednost::Prazno => Some(Value::Null),
        Vrijednost::Cijeli(n) => Some(json!(n)),
        Vrijednost::Broj(n) => Some(json!(n)),
        Vrijednost::Tekst(t) => Some(json!(t)),
        Vrijednost::Bajtovi(_) => None,
    }
}

struct Pisac<'a> {
    p: &'a Projekt,
    znacajke: Vec<Value>,
}

impl Pisac<'_> {
    fn stavka(&mut self, s: Stavka, mapa: &str) {
        match s {
            Stavka::Mapa(m) => {
                if let Some(m) = self.p.mapa(m) {
                    let put = if mapa.is_empty() {
                        m.naziv.clone()
                    } else {
                        format!("{mapa} / {}", m.naziv)
                    };
                    m.djeca.iter().for_each(|d| self.stavka(*d, &put));
                }
            }
            Stavka::Sloj(id) => {
                if let Some(s) = self.p.sloj(id) {
                    s.objekti.values().for_each(|o| self.objekt(id, s, o, mapa));
                }
            }
        }
    }

    fn objekt(&mut self, id: IdSloja, s: &Sloj, o: &Objekt, mapa: &str) {
        let mut p = Map::new();
        p.insert("name".into(), json!(o.naziv));
        if !o.opis.is_empty() {
            p.insert("description".into(), json!(o.opis));
        }
        p.insert("mapa".into(), json!(mapa));
        p.insert("sloj".into(), json!(s.naziv));
        svojstva_stila(&mut p, &o.geometrija, &o.stil.na_stilu_sloja(&s.stil));
        let zapis = self
            .p
            .zapisi()
            .slojevi
            .get(&id)
            .and_then(|z| z.objekti.get(&o.id));
        let stupci = zapis.iter().flat_map(|z| &z.nepoznati);
        for (naziv, v) in stupci.filter_map(|(n, v)| Some((n, u_vrijednost(v)?))) {
            p.entry(naziv.clone()).or_insert(v);
        }
        self.znacajke.push(json!({
            "type": "Feature",
            "properties": p,
            "geometry": u_geometriju(&o.geometrija),
        }));
    }
}

/// Stil po dogovoru `simplestyle`: crta za sve, ispuna za lik, boja znaka za točku.
fn svojstva_stila(p: &mut Map<String, Value>, g: &Geometrija, stil: &Stil) {
    let udio = |b: Boja| f64::from(b.a) / 255.0;
    p.insert("stroke".into(), json!(hex(stil.boja_crte)));
    p.insert("stroke-opacity".into(), json!(udio(stil.boja_crte)));
    let px = (stil.debljina_mm * 96.0 / 25.4 * 100.0).round() / 100.0;
    p.insert("stroke-width".into(), json!(px));
    match g {
        Geometrija::Poligon(_) => {
            let bez = stil.vrsta_ispune == VrstaIspune::Bez;
            let a = if bez { 0.0 } else { udio(stil.boja_ispune) };
            p.insert("fill".into(), json!(hex(stil.boja_ispune)));
            p.insert("fill-opacity".into(), json!(a));
        }
        Geometrija::Tocka(_) => {
            p.insert("marker-color".into(), json!(hex(stil.boja_crte)));
        }
        Geometrija::Linija(_) => {}
    }
}

/// Cijela zbirka objekata kao tekst.
pub(crate) fn napisi(p: &Projekt, opseg: OpsegIzvoza, naziv: &str) -> String {
    let mut pisac = Pisac {
        p,
        znacajke: Vec::new(),
    };
    match opseg {
        OpsegIzvoza::Projekt => p.korijen().iter().for_each(|s| pisac.stavka(*s, "")),
        OpsegIzvoza::Stavka(s) => pisac.stavka(s, ""),
    }
    let zbirka = json!({"type": "FeatureCollection", "name": naziv, "features": pisac.znacajke});
    zbirka.to_string()
}

#[cfg(test)]
#[path = "geojson_testovi.rs"]
mod testovi;
