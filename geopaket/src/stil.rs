use crate::Greska;
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use rusqlite::types::Value;
use slojevi::{
    Boja, Natpis, OblikZnaka, Stil, StilObjekta, VrstaCrte, VrstaGeometrije, VrstaIspune,
};
use std::collections::BTreeMap;

pub const STUPCI: [&str; 9] = [
    "naziv",
    "opis",
    "stil_boja",
    "stil_debljina",
    "stil_ispuna",
    "stil_znak",
    "stil_velicina",
    "stil_crta",
    "stil_uzorak",
];
pub fn boja(b: Boja) -> String {
    format!("{},{},{},{}", b.r, b.g, b.b, b.a)
}
pub fn procitaj_boju(s: &str) -> Result<Boja, Greska> {
    if let Some(hex) = s.strip_prefix('#') {
        return hex_boja(hex);
    }
    let v = s
        .split(',')
        .take(4)
        .map(str::parse::<u8>)
        .collect::<Result<Vec<_>, _>>()?;
    let [r, g, b, a] = v.as_slice() else {
        return Err("Boja nema četiri sastavnice.".into());
    };
    Ok(Boja {
        r: *r,
        g: *g,
        b: *b,
        a: *a,
    })
}
fn hex_sastavnice(s: &str) -> Result<Boja, Greska> {
    let b = |i| u8::from_str_radix(&s[i..i + 2], 16);
    Ok(Boja {
        r: b(0)?,
        g: b(2)?,
        b: b(4)?,
        a: if s.len() == 8 { b(6)? } else { 255 },
    })
}
fn hex_boja(s: &str) -> Result<Boja, Greska> {
    if ![6, 8].contains(&s.len()) || !s.is_ascii() {
        return Err("Boja nije valjana.".into());
    }
    hex_sastavnice(s)
}

pub fn znak(z: OblikZnaka) -> &'static str {
    match z {
        OblikZnaka::Krug => "circle",
        OblikZnaka::Kvadrat => "square",
        OblikZnaka::Trokut => "triangle",
        OblikZnaka::Zvijezda => "star",
    }
}
pub fn procitaj_znak(s: &str) -> Result<OblikZnaka, Greska> {
    match s {
        "circle" => Ok(OblikZnaka::Krug),
        "square" => Ok(OblikZnaka::Kvadrat),
        "triangle" => Ok(OblikZnaka::Trokut),
        "star" => Ok(OblikZnaka::Zvijezda),
        _ => Err("Oblik znaka nije podržan.".into()),
    }
}
/// Nazivi QGIS-a za vrstu crte.
pub const fn crta(v: VrstaCrte) -> &'static str {
    match v {
        VrstaCrte::Puna => "solid",
        VrstaCrte::Isprekidana => "dash",
        VrstaCrte::Tockasta => "dot",
        VrstaCrte::CrtaTocka => "dash dot",
    }
}
/// Ostale crte QGIS-a, npr. `dash dot dot`, čitaju se kao najsličnije, a ne kao greška.
pub fn procitaj_crtu(s: &str) -> VrstaCrte {
    match s {
        "dash" => VrstaCrte::Isprekidana,
        "dot" => VrstaCrte::Tockasta,
        "dash dot" | "dash dot dot" => VrstaCrte::CrtaTocka,
        _ => VrstaCrte::Puna,
    }
}
/// Nazivi QGIS-a za ispunu. Šrafura je `f_diagonal`.
pub const fn uzorak(v: VrstaIspune) -> &'static str {
    match v {
        VrstaIspune::Puna => "solid",
        VrstaIspune::Srafura => "f_diagonal",
        VrstaIspune::Bez => "no",
    }
}
/// Svaki drugi uzorak QGIS-a, npr. križni, čita se kao šrafura.
pub fn procitaj_uzorak(s: &str) -> VrstaIspune {
    match s {
        "solid" => VrstaIspune::Puna,
        "no" => VrstaIspune::Bez,
        _ => VrstaIspune::Srafura,
    }
}
pub fn vrijednosti(naziv: &str, opis: &str, s: StilObjekta) -> Vec<Value> {
    vec![
        Value::Text(naziv.into()),
        Value::Text(opis.into()),
        s.boja_crte.map(boja).into(),
        s.debljina_mm.into(),
        s.boja_ispune.map(boja).into(),
        s.oblik_znaka.map(|z| znak(z).to_owned()).into(),
        s.velicina_znaka_mm.into(),
        s.vrsta_crte.map(|v| crta(v).to_owned()).into(),
        s.vrsta_ispune.map(|v| uzorak(v).to_owned()).into(),
    ]
}
fn tekst(v: Option<&Value>) -> Option<&str> {
    match v {
        Some(Value::Text(s)) => Some(s),
        _ => None,
    }
}
fn broj(v: Option<&Value>) -> Result<Option<f64>, Greska> {
    match v {
        Some(Value::Real(n)) => Ok(Some(*n)),
        Some(Value::Integer(n)) => Ok(Some(*n as f64)),
        Some(Value::Text(s)) => Ok(Some(s.parse()?)),
        _ => Ok(None),
    }
}
pub fn objekt(v: &BTreeMap<String, Value>) -> Result<StilObjekta, Greska> {
    Ok(StilObjekta {
        boja_crte: tekst(v.get("stil_boja")).map(procitaj_boju).transpose()?,
        debljina_mm: broj(v.get("stil_debljina"))?,
        boja_ispune: tekst(v.get("stil_ispuna")).map(procitaj_boju).transpose()?,
        oblik_znaka: tekst(v.get("stil_znak")).map(procitaj_znak).transpose()?,
        velicina_znaka_mm: broj(v.get("stil_velicina"))?,
        vrsta_crte: tekst(v.get("stil_crta")).map(procitaj_crtu),
        vrsta_ispune: tekst(v.get("stil_uzorak")).map(procitaj_uzorak),
    })
}
fn opcija(n: &str, v: &str) -> String {
    format!(r#"<Option name="{n}" value="{v}" type="QString"/>"#)
}
fn svojstvo(n: &str, stupac: &str, zamjena: Option<String>) -> String {
    let izvor = match zamjena {
        None => format!(
            r#"<Option name="field" value="{stupac}" type="QString"/><Option name="type" value="2" type="int"/>"#
        ),
        Some(z) => format!(
            r#"<Option name="expression" value="coalesce(&quot;{stupac}&quot;,{z})" type="QString"/><Option name="type" value="3" type="int"/>"#
        ),
    };
    format!(
        r#"<Option name="{n}" type="Map"><Option name="active" value="true" type="bool"/>{izvor}</Option>"#
    )
}
fn svojstva(v: VrstaGeometrije, s: Stil) -> String {
    let mut p =
        svojstvo("fillColor", "stil_ispuna", None) + &svojstvo("outlineColor", "stil_boja", None);
    match v {
        VrstaGeometrije::Tocka => {
            p = svojstvo("fillColor", "stil_boja", None)
                + &svojstvo("outlineColor", "stil_ispuna", None);
            p += &svojstvo("size", "stil_velicina", None);
            p += &svojstvo(
                "outlineWidth",
                "stil_debljina",
                Some(s.debljina_mm.to_string()),
            );
            p += &svojstvo(
                "name",
                "stil_znak",
                Some(format!("'{}'", znak(s.oblik_znaka))),
            );
        }
        VrstaGeometrije::Linija => {
            p += &svojstvo("outlineWidth", "stil_debljina", None);
            p += &svojstvo_crte(s);
        }
        VrstaGeometrije::Poligon => {
            p += &svojstvo(
                "outlineWidth",
                "stil_debljina",
                Some(s.debljina_mm.to_string()),
            );
            p += &svojstvo_crte(s);
            p += &svojstvo(
                "fillStyle",
                "stil_uzorak",
                Some(format!("'{}'", uzorak(s.vrsta_ispune))),
            );
        }
    }
    format!(
        r#"<data_defined_properties><Option type="Map"><Option name="name" value="" type="QString"/><Option name="properties" type="Map">{p}</Option><Option name="type" value="collection" type="QString"/></Option></data_defined_properties>"#
    )
}
fn svojstvo_crte(s: Stil) -> String {
    svojstvo(
        "outlineStyle",
        "stil_crta",
        Some(format!("'{}'", crta(s.vrsta_crte))),
    )
}
pub fn renderer(v: VrstaGeometrije, s: Stil) -> String {
    let (tip, klasa, o) = match v {
        VrstaGeometrije::Tocka => (
            "marker",
            "SimpleMarker",
            opcija("color", &boja(s.boja_crte))
                + &opcija("outline_color", &boja(s.boja_ispune))
                + &opcija("outline_width", &s.debljina_mm.to_string())
                + &opcija("size", &s.velicina_znaka_mm.to_string())
                + &opcija("name", znak(s.oblik_znaka)),
        ),
        VrstaGeometrije::Linija => (
            "line",
            "SimpleLine",
            opcija("line_color", &boja(s.boja_crte))
                + &opcija("line_width", &s.debljina_mm.to_string())
                + &opcija("line_style", crta(s.vrsta_crte)),
        ),
        VrstaGeometrije::Poligon => (
            "fill",
            "SimpleFill",
            opcija("color", &boja(s.boja_ispune))
                + &opcija("outline_color", &boja(s.boja_crte))
                + &opcija("outline_width", &s.debljina_mm.to_string())
                + &opcija("outline_style", crta(s.vrsta_crte))
                + &opcija("style", uzorak(s.vrsta_ispune)),
        ),
    };
    format!(
        r#"<renderer-v2 type="singleSymbol"><symbols><symbol name="0" type="{tip}"><layer class="{klasa}" enabled="1"><Option type="Map">{o}</Option>{}</layer></symbol></symbols></renderer-v2>"#,
        svojstva(v, s)
    )
}

/// Smještaj natpisa u QGIS-u: točka oko znaka, linija uz zavoj, lik u sredini.
const fn smjestaj(v: VrstaGeometrije) -> &'static str {
    match v {
        VrstaGeometrije::Tocka => r#"placement="6""#,
        VrstaGeometrije::Linija => {
            r#"placement="3" placementFlags="9" repeatDistance="130" repeatDistanceUnits="MM" maxCurvedCharAngleIn="25" maxCurvedCharAngleOut="-25""#
        }
        VrstaGeometrije::Poligon => r#"placement="1" centroidInside="1""#,
    }
}
/// Natpisi s nazivom objekta, kako ih crta i TerraSimple.
pub fn natpisi(v: VrstaGeometrije, s: Stil) -> String {
    let n = s.natpis;
    format!(
        r#"<labeling type="simple"><settings><text-style fieldName="naziv" isExpression="0" fontSize="{}" fontSizeUnit="Point" fontItalic="{}" textColor="{}" textOpacity="1"><text-buffer bufferDraw="{}" bufferSize="1" bufferSizeUnits="MM" bufferColor="255,255,255,255" bufferOpacity="1" bufferNoFill="1"/></text-style><placement {}/></settings></labeling>"#,
        n.velicina_pt,
        i32::from(n.kurziv),
        boja(n.boja),
        i32::from(n.obrub),
        smjestaj(v)
    )
}
/// Uključenost natpisa, kao atribut sloja u QML-u i u projektu.
pub fn ukljuceni_natpisi(s: Stil) -> &'static str {
    if s.natpis.prikazi { "1" } else { "0" }
}

pub fn qml(v: VrstaGeometrije, s: Stil) -> String {
    let rend = renderer(v, s) + &natpisi(v, s);
    let skriveni: String = STUPCI
        .iter()
        .map(|n| {
            format!(
                r#"<column name="{n}" type="field" hidden="{}" width="-1"/>"#,
                i32::from(n.starts_with("stil_"))
            )
        })
        .collect();
    format!(
        r#"<qgis version="3.44.0" styleCategories="Symbology|Labeling|AttributeTable" labelsEnabled="{}">{rend}<attributetableconfig><columns>{skriveni}</columns></attributetableconfig></qgis>"#,
        ukljuceni_natpisi(s)
    )
}
#[derive(Default)]
struct Cvor {
    ime: String,
    atributi: BTreeMap<String, String>,
    djeca: Vec<Cvor>,
}
fn cvor(e: &BytesStart<'_>) -> Result<Cvor, Greska> {
    let mut a = BTreeMap::new();
    for v in e.attributes() {
        let v = v?;
        a.insert(
            v.key.as_ref().to_owned(),
            v.normalized_value(quick_xml::XmlVersion::Implicit1_0)?
                .into_owned(),
        );
    }
    Ok(Cvor {
        ime: e.name().as_ref().to_owned(),
        atributi: a,
        djeca: Vec::new(),
    })
}
fn dodaj_cvor(red: &mut [Cvor], c: Cvor) -> Result<(), Greska> {
    red.last_mut().ok_or("XML nije uravnotežen.")?.djeca.push(c);
    Ok(())
}
fn zatvori(red: &mut Vec<Cvor>) -> Result<(), Greska> {
    let c = red.pop().ok_or("XML stablo je prazno.")?;
    dodaj_cvor(red, c)
}
fn prazni_element(red: &mut [Cvor], e: &BytesStart<'_>) -> Result<(), Greska> {
    dodaj_cvor(red, cvor(e)?)
}
fn otvori_element(red: &mut Vec<Cvor>, e: &BytesStart<'_>) -> Result<(), Greska> {
    red.push(cvor(e)?);
    Ok(())
}
fn dogadaj(red: &mut Vec<Cvor>, e: Event<'_>) -> Result<bool, Greska> {
    match e {
        Event::Start(e) => otvori_element(red, &e).map(|()| true),
        Event::Empty(e) => prazni_element(red, &e).map(|()| true),
        Event::End(_) => zatvori(red).map(|()| true),
        Event::Eof => Ok(false),
        _ => Ok(true),
    }
}

fn stablo(xml: &str) -> Result<Cvor, Greska> {
    let mut r = Reader::from_str(xml);
    let mut red = vec![Cvor::default()];
    while dogadaj(&mut red, r.read_event()?)? {}
    if red.len() != 1 {
        return Err("XML nije dovršen.".into());
    }
    Ok(red.remove(0))
}

fn dijete<'a>(c: &'a Cvor, n: &str) -> Result<&'a Cvor, Greska> {
    c.djeca
        .iter()
        .find(|v| v.ime == n)
        .ok_or_else(|| format!("QML nema dio {n}.").into())
}
fn opcije(c: &Cvor) -> BTreeMap<String, String> {
    c.djeca
        .iter()
        .filter(|c| c.ime == "Option")
        .filter_map(|c| {
            Some((
                c.atributi.get("name")?.clone(),
                c.atributi.get("value")?.clone(),
            ))
        })
        .collect()
}
fn iscrtavanje(q: &Cvor) -> Result<&Cvor, Greska> {
    let r = dijete(q, "renderer-v2")?;
    if r.atributi.get("type").map(String::as_str) != Some("singleSymbol") {
        return Err("Složeno iscrtavanje stila nije podržano; QML ostaje sačuvan.".into());
    }
    Ok(r)
}
fn prvi_sloj(r: &Cvor) -> Result<&Cvor, Greska> {
    dijete(dijete(dijete(r, "symbols")?, "symbol")?, "layer")
}
pub fn procitaj(xml: &str, v: VrstaGeometrije, s: Stil) -> Result<Stil, Greska> {
    let k = stablo(xml)?;
    let q = dijete(&k, "qgis")?;
    let simbol = procitaj_simbol(q, v, s)?;
    Ok(Stil {
        natpis: procitaj_natpis(q, s.natpis)?,
        ..simbol
    })
}
fn procitaj_simbol(q: &Cvor, v: VrstaGeometrije, s: Stil) -> Result<Stil, Greska> {
    let l = prvi_sloj(iscrtavanje(q)?)?;
    primijeni_opcije(&opcije(dijete(l, "Option")?), v, s)
}

fn da(s: &str) -> bool {
    matches!(s, "1" | "true")
}
/// QGIS veličinu slova zna zadati i u milimetrima ili pikselima.
fn u_tocke_slova(velicina: f64, jedinica: Option<&str>) -> f64 {
    match jedinica {
        Some("MM") => velicina * 72.0 / 25.4,
        Some("Pixel") => velicina * 0.75,
        _ => velicina,
    }
}
fn stil_teksta(q: &Cvor) -> Option<&Cvor> {
    let l = dijete(q, "labeling").ok()?;
    if l.atributi.get("type").map(String::as_str) != Some("simple") {
        return None;
    }
    dijete(dijete(l, "settings").ok()?, "text-style").ok()
}
fn ukljuceno(c: &Cvor, kljuc: &str, staro: bool) -> bool {
    c.atributi.get(kljuc).map_or(staro, |v| da(v))
}
fn velicina_slova(a: &BTreeMap<String, String>, staro: f64) -> Result<f64, Greska> {
    let jedinica = a.get("fontSizeUnit").map(String::as_str);
    uzmi(a, "fontSize", staro, |v| {
        Ok(u_tocke_slova(decimalni(v)?, jedinica))
    })
}
/// Datoteka bez natpisa u QML-u, npr. iz 0.0.5, zadržava natpis iz metapodataka.
fn procitaj_natpis(q: &Cvor, n: Natpis) -> Result<Natpis, Greska> {
    let mut n = n;
    n.prikazi = ukljuceno(q, "labelsEnabled", n.prikazi);
    let Some(t) = stil_teksta(q) else {
        return Ok(n);
    };
    n.velicina_pt = velicina_slova(&t.atributi, n.velicina_pt)?;
    n.boja = uzmi(&t.atributi, "textColor", n.boja, procitaj_boju)?;
    n.kurziv = ukljuceno(t, "fontItalic", n.kurziv);
    if let Ok(b) = dijete(t, "text-buffer") {
        n.obrub = ukljuceno(b, "bufferDraw", n.obrub);
    }
    Ok(n)
}

fn uzmi<T>(
    o: &BTreeMap<String, String>,
    n: &str,
    staro: T,
    citaj: impl FnOnce(&str) -> Result<T, Greska>,
) -> Result<T, Greska> {
    o.get(n)
        .map(|s| citaj(s))
        .transpose()
        .map(|v| v.unwrap_or(staro))
}
fn decimalni(s: &str) -> Result<f64, Greska> {
    Ok(s.parse()?)
}
fn linija(o: &BTreeMap<String, String>, mut s: Stil) -> Result<Stil, Greska> {
    s.boja_crte = uzmi(o, "line_color", s.boja_crte, procitaj_boju)?;
    s.debljina_mm = uzmi(o, "line_width", s.debljina_mm, decimalni)?;
    s.vrsta_crte = o
        .get("line_style")
        .map_or(s.vrsta_crte, |v| procitaj_crtu(v));
    Ok(s)
}
fn poligon(o: &BTreeMap<String, String>, mut s: Stil) -> Result<Stil, Greska> {
    s.boja_crte = uzmi(o, "outline_color", s.boja_crte, procitaj_boju)?;
    s.boja_ispune = uzmi(o, "color", s.boja_ispune, procitaj_boju)?;
    s.debljina_mm = uzmi(o, "outline_width", s.debljina_mm, decimalni)?;
    s.vrsta_crte = o
        .get("outline_style")
        .map_or(s.vrsta_crte, |v| procitaj_crtu(v));
    s.vrsta_ispune = o
        .get("style")
        .map_or(s.vrsta_ispune, |v| procitaj_uzorak(v));
    Ok(s)
}
fn tocka(o: &BTreeMap<String, String>, mut s: Stil) -> Result<Stil, Greska> {
    s.boja_crte = uzmi(o, "color", s.boja_crte, procitaj_boju)?;
    s.boja_ispune = uzmi(o, "outline_color", s.boja_ispune, procitaj_boju)?;
    s.debljina_mm = uzmi(o, "outline_width", s.debljina_mm, decimalni)?;
    s.velicina_znaka_mm = uzmi(o, "size", s.velicina_znaka_mm, decimalni)?;
    s.oblik_znaka = uzmi(o, "name", s.oblik_znaka, procitaj_znak)?;
    Ok(s)
}
fn primijeni_opcije(
    o: &BTreeMap<String, String>,
    v: VrstaGeometrije,
    s: Stil,
) -> Result<Stil, Greska> {
    match v {
        VrstaGeometrije::Tocka => tocka(o, s),
        VrstaGeometrije::Linija => linija(o, s),
        VrstaGeometrije::Poligon => poligon(o, s),
    }
}

// U metapodacima se čuvaju i svojstva koja pripadna vrsta simbola ne prikazuje.
pub fn json(s: Stil) -> String {
    serde_json::json!([
        boja(s.boja_crte),
        s.debljina_mm,
        boja(s.boja_ispune),
        znak(s.oblik_znaka),
        s.velicina_znaka_mm,
        crta(s.vrsta_crte),
        uzorak(s.vrsta_ispune),
        [
            serde_json::json!(s.natpis.prikazi),
            serde_json::json!(s.natpis.velicina_pt),
            serde_json::json!(boja(s.natpis.boja)),
            serde_json::json!(s.natpis.kurziv),
            serde_json::json!(s.natpis.obrub),
        ]
    ])
    .to_string()
}
/// Natpis iz metapodataka. Datoteke prije 0.0.6 ga nemaju, pa vrijedi zadani.
fn natpis_iz_json(v: Option<&serde_json::Value>) -> Result<Natpis, Greska> {
    let Some(v) = v else {
        return Ok(Natpis::default());
    };
    let (prikazi, velicina_pt, b, kurziv, obrub): (bool, f64, String, bool, bool) =
        serde_json::from_value(v.clone())?;
    Ok(Natpis {
        prikazi,
        velicina_pt,
        boja: procitaj_boju(&b)?,
        kurziv,
        obrub,
    })
}
/// Datoteke prije 0.0.5 imaju pet vrijednosti, pa vrsta crte i ispune ostaje zadana.
/// Natpis je osma vrijednost.
pub fn iz_json(s: &str) -> Result<Stil, Greska> {
    let sve: Vec<serde_json::Value> = serde_json::from_str(s)?;
    let dodatak = |i: usize| sve.get(i).and_then(serde_json::Value::as_str);
    Ok(Stil {
        vrsta_crte: dodatak(5).map_or(VrstaCrte::Puna, procitaj_crtu),
        vrsta_ispune: dodatak(6).map_or(VrstaIspune::Puna, procitaj_uzorak),
        natpis: natpis_iz_json(sve.get(7))?,
        ..prvih_pet(&sve)?
    })
}
/// Boja i debljina crte, ispuna, oblik i veličina znaka, kakve ima i datoteka prije 0.0.5.
fn prvih_pet(sve: &[serde_json::Value]) -> Result<Stil, Greska> {
    let prvih_pet = serde_json::Value::Array(sve.iter().take(5).cloned().collect());
    let (boja_crte, debljina, ispuna, oblik, velicina): (String, f64, String, String, f64) =
        serde_json::from_value(prvih_pet)?;
    Ok(Stil {
        boja_crte: procitaj_boju(&boja_crte)?,
        debljina_mm: debljina,
        boja_ispune: procitaj_boju(&ispuna)?,
        oblik_znaka: procitaj_znak(&oblik)?,
        velicina_znaka_mm: velicina,
        ..Stil::default()
    })
}

#[cfg(test)]
mod testovi {
    use super::*;
    #[test]
    fn nazivi_crta_i_uzoraka_kao_u_qgis_u() {
        for v in [
            VrstaCrte::Puna,
            VrstaCrte::Isprekidana,
            VrstaCrte::Tockasta,
            VrstaCrte::CrtaTocka,
        ] {
            assert_eq!(procitaj_crtu(crta(v)), v);
        }
        assert_eq!(procitaj_crtu("dash dot dot"), VrstaCrte::CrtaTocka);
        assert_eq!(procitaj_crtu("no"), VrstaCrte::Puna);
        for v in [VrstaIspune::Puna, VrstaIspune::Srafura, VrstaIspune::Bez] {
            assert_eq!(procitaj_uzorak(uzorak(v)), v);
        }
        assert_eq!(procitaj_uzorak("cross"), VrstaIspune::Srafura);
    }

    #[test]
    fn svi_stilovi_i_qml_odabira() {
        let s = Stil {
            boja_crte: Boja {
                r: 4,
                g: 8,
                b: 12,
                a: 99,
            },
            boja_ispune: Boja {
                r: 20,
                g: 30,
                b: 40,
                a: 80,
            },
            oblik_znaka: OblikZnaka::Zvijezda,
            vrsta_crte: VrstaCrte::Isprekidana,
            vrsta_ispune: VrstaIspune::Bez,
            ..Stil::default()
        };
        for v in [
            VrstaGeometrije::Tocka,
            VrstaGeometrije::Linija,
            VrstaGeometrije::Poligon,
        ] {
            let q = qml(v, s);
            assert_eq!(procitaj(&q, v, s).unwrap(), s);
            let izmijenjen=q.replace("</qgis>","<selection><symbol><layer><Option><Option name=\"color\" value=\"#ffffff\"/></Option></layer></symbol></selection></qgis>");
            assert_eq!(procitaj(&izmijenjen, v, s).unwrap(), s);
            if v != VrstaGeometrije::Linija {
                assert!(q.contains("coalesce"));
            }
        }
        assert_eq!(iz_json(&json(s)).unwrap(), s);
        assert!(iz_json("[]").is_err());
        // Datoteka prije 0.0.5 ima pet vrijednosti.
        let staro = r#"["1,2,3,4",0.8,"5,6,7,8","circle",3.0]"#;
        let st = iz_json(staro).unwrap();
        assert_eq!(
            (st.vrsta_crte, st.vrsta_ispune),
            (VrstaCrte::Puna, VrstaIspune::Puna)
        );
    }
    #[test]
    fn natpis_ide_u_qml_i_metapodatke_i_vraca_se() {
        let s = Stil {
            natpis: Natpis {
                prikazi: false,
                velicina_pt: 13.5,
                boja: Boja {
                    r: 21,
                    g: 101,
                    b: 192,
                    a: 255,
                },
                kurziv: true,
                obrub: false,
            },
            ..Stil::default()
        };
        for v in [
            VrstaGeometrije::Tocka,
            VrstaGeometrije::Linija,
            VrstaGeometrije::Poligon,
        ] {
            let q = qml(v, s);
            assert!(q.contains(r#"labelsEnabled="0""#));
            assert!(q.contains(r#"fieldName="naziv""#));
            assert_eq!(procitaj(&q, v, Stil::default()).unwrap(), s);
        }
        assert!(qml(VrstaGeometrije::Linija, s).contains(r#"placement="3""#));
        assert_eq!(iz_json(&json(s)).unwrap(), s);
        // Metapodaci prije 0.0.6 nemaju natpis.
        let staro = r#"["1,2,3,4",0.8,"5,6,7,8","circle",3.0,"dash","no"]"#;
        assert_eq!(iz_json(staro).unwrap().natpis, Natpis::default());
        assert!(iz_json(r#"["1,2,3,4",0.8,"5,6,7,8","circle",3.0,"dash","no",[true]]"#).is_err());
    }

    #[test]
    fn qml_bez_natpisa_ili_iz_qgis_a() {
        let s = Stil::default();
        let bez = qml(VrstaGeometrije::Tocka, s)
            .replace(r#" labelsEnabled="1""#, "")
            .replace(&natpisi(VrstaGeometrije::Tocka, s), "");
        let mut zadani = s;
        zadani.natpis.velicina_pt = 20.0;
        assert_eq!(
            procitaj(&bez, VrstaGeometrije::Tocka, zadani)
                .unwrap()
                .natpis,
            zadani.natpis
        );

        let qgis = r#"<qgis labelsEnabled="1"><renderer-v2 type="singleSymbol"><symbols><symbol><layer><Option/></layer></symbol></symbols></renderer-v2><labeling type="simple"><settings><text-style fontSize="5" fontSizeUnit="MM" textColor="10,20,30,255,rgb:0.1,0.2,0.3,1" fontItalic="true"><text-buffer bufferDraw="0"/></text-style></settings></labeling></qgis>"#;
        let n = procitaj(qgis, VrstaGeometrije::Tocka, s).unwrap().natpis;
        assert!(n.prikazi && n.kurziv && !n.obrub);
        assert!((n.velicina_pt - 5.0 * 72.0 / 25.4).abs() < 1e-9);
        assert_eq!((n.boja.r, n.boja.g, n.boja.b), (10, 20, 30));
        let piksel = qgis.replace(r#"fontSizeUnit="MM""#, r#"fontSizeUnit="Pixel""#);
        let n = procitaj(&piksel, VrstaGeometrije::Tocka, s).unwrap().natpis;
        assert!((n.velicina_pt - 3.75).abs() < 1e-9);
        let pravila = qgis.replace(r#"type="simple""#, r#"type="rule-based""#);
        assert_eq!(
            procitaj(&pravila, VrstaGeometrije::Tocka, s)
                .unwrap()
                .natpis
                .boja,
            s.natpis.boja
        );
        let bez_obruba = qgis.replace(r#"<text-buffer bufferDraw="0"/>"#, "");
        let n = procitaj(&bez_obruba, VrstaGeometrije::Tocka, s)
            .unwrap()
            .natpis;
        assert_eq!(n.obrub, s.natpis.obrub);
        let losa = qgis.replace(r#"fontSize="5""#, r#"fontSize="x""#);
        assert!(procitaj(&losa, VrstaGeometrije::Tocka, s).is_err());
    }

    #[test]
    fn vlastiti_stil_i_prazni_stupci() {
        let b = Boja {
            r: 1,
            g: 2,
            b: 3,
            a: 4,
        };
        let s = StilObjekta {
            boja_crte: Some(b),
            boja_ispune: Some(b),
            debljina_mm: Some(1.5),
            oblik_znaka: Some(OblikZnaka::Trokut),
            velicina_znaka_mm: Some(5.),
            vrsta_crte: Some(VrstaCrte::CrtaTocka),
            vrsta_ispune: Some(VrstaIspune::Srafura),
        };
        let v = vrijednosti("Naziv", "Opis", s);
        let mapa = STUPCI
            .iter()
            .zip(v)
            .map(|(k, v)| (k.to_string(), v))
            .collect();
        assert_eq!(objekt(&mapa).unwrap(), s);
        assert_eq!(objekt(&BTreeMap::new()).unwrap(), StilObjekta::default());
        assert_eq!(broj(Some(&Value::Integer(2))).unwrap(), Some(2.));
        assert_eq!(broj(Some(&Value::Text("2.5".into()))).unwrap(), Some(2.5));
        assert!(broj(Some(&Value::Text("krivo".into()))).is_err());
        for z in [
            OblikZnaka::Krug,
            OblikZnaka::Kvadrat,
            OblikZnaka::Trokut,
            OblikZnaka::Zvijezda,
        ] {
            assert_eq!(procitaj_znak(znak(z)).unwrap(), z);
        }
        assert!(procitaj_znak("nepoznato").is_err());
        assert_eq!(procitaj_boju("#01020304").unwrap(), b);
        assert_eq!(procitaj_boju("#010203").unwrap().a, 255);
        for s in ["", "1,2,3", "a,b,c,d", "#gggggg", "#a", "#ššš"] {
            assert!(procitaj_boju(s).is_err());
        }
    }
    #[test]
    fn pokvaren_i_slozen_qml_je_prijavljen() {
        for xml in [
            "",
            "<qgis>",
            "<qgis></krivo>",
            "<qgis><renderer-v2 type=\"categorizedSymbol\"/></qgis>",
            "<qgis><renderer-v2 type=\"singleSymbol\"/></qgis>",
        ] {
            assert!(procitaj(xml, VrstaGeometrije::Linija, Stil::default()).is_err());
        }
        assert!(stablo("<a></a></b>").is_err());
        let q = qml(VrstaGeometrije::Tocka, Stil::default());
        assert!(
            procitaj(
                &q.replace("circle", "nepoznato"),
                VrstaGeometrije::Tocka,
                Stil::default()
            )
            .is_err()
        );
        assert!(
            procitaj(
                &q.replace("value=\"3\"", "value=\"x\""),
                VrstaGeometrije::Tocka,
                Stil::default()
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod nevaljane_opcije {
    use super::*;
    #[test]
    fn pogreske_svih_svojstava_i_prazne_opcije() {
        for v in [
            VrstaGeometrije::Tocka,
            VrstaGeometrije::Linija,
            VrstaGeometrije::Poligon,
        ] {
            assert_eq!(
                primijeni_opcije(&BTreeMap::new(), v, Stil::default()).unwrap(),
                Stil::default()
            );
        }
        for (v, kljucevi) in [
            (
                VrstaGeometrije::Tocka,
                vec!["color", "outline_color", "size", "name"],
            ),
            (VrstaGeometrije::Linija, vec!["line_color", "line_width"]),
            (
                VrstaGeometrije::Poligon,
                vec!["outline_color", "color", "outline_width"],
            ),
        ] {
            for k in kljucevi {
                assert!(
                    primijeni_opcije(
                        &BTreeMap::from([(k.to_string(), "pogresno".into())]),
                        v,
                        Stil::default()
                    )
                    .is_err()
                );
            }
        }
        for s in [
            r#"["x",1,"1,2,3,4","circle",3]"#,
            r#"["1,2,3,4",1,"x","circle",3]"#,
            r#"["1,2,3,4",1,"1,2,3,4","x",3]"#,
        ] {
            assert!(iz_json(s).is_err());
        }
        for k in [
            "stil_boja",
            "stil_debljina",
            "stil_ispuna",
            "stil_znak",
            "stil_velicina",
        ] {
            assert!(objekt(&BTreeMap::from([(k.into(), Value::Text("x".into()))])).is_err());
        }
        for s in ["gg0000", "00gg00", "0000gg", "000000gg"] {
            assert!(hex_boja(s).is_err());
        }
        assert!(cvor(&BytesStart::from_content("qgis a", 4)).is_err());
    }
}

#[cfg(test)]
mod granice_xmla {
    use super::*;
    #[test]
    fn nepoznati_entitet_i_nepotpuna_struktura() {
        let e = BytesStart::from_content("qgis a='&nepoznato;'", 4);
        assert!(cvor(&e).is_err());
        assert!(otvori_element(&mut Vec::new(), &e).is_err());
        assert!(prazni_element(&mut [], &e).is_err());
        assert!(dodaj_cvor(&mut [], Cvor::default()).is_err());
        assert!(zatvori(&mut Vec::new()).is_err());
        for q in [
            "<qgis/>",
            "<qgis><renderer-v2 type='singleSymbol'><symbols/></renderer-v2></qgis>",
            "<qgis><renderer-v2 type='singleSymbol'><symbols><symbol/></symbols></renderer-v2></qgis>",
            "<qgis><renderer-v2 type='singleSymbol'><symbols><symbol><layer/></symbol></symbols></renderer-v2></qgis>",
            "<qgis a='&nepoznato;'/>",
            "<qgis a='&nepoznato;'></qgis>",
            "<qgis",
        ] {
            assert!(
                procitaj(q, VrstaGeometrije::Tocka, Stil::default()).is_err(),
                "{q}"
            );
        }
        let c = Cvor {
            djeca: vec![
                Cvor {
                    ime: "Option".into(),
                    ..Cvor::default()
                },
                Cvor {
                    ime: "Option".into(),
                    atributi: BTreeMap::from([("name".into(), "x".into())]),
                    ..Cvor::default()
                },
            ],
            ..Cvor::default()
        };
        assert!(opcije(&c).is_empty());
    }
}

#[cfg(test)]
mod zavrsetak_citanja {
    use super::*;
    #[test]
    fn kraj_dokumenta_zaustavlja_citanje_bez_promjene_stabla() {
        let mut red = vec![Cvor::default()];
        assert!(!dogadaj(&mut red, Event::Eof).unwrap());
        assert_eq!(red.len(), 1);
        assert!(red[0].djeca.is_empty());
    }
}
