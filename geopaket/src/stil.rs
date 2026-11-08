use crate::Greska;
use quick_xml::{
    Reader,
    events::{BytesStart, Event},
};
use rusqlite::types::Value;
use slojevi::{Boja, OblikZnaka, Stil, StilObjekta, VrstaGeometrije};
use std::collections::BTreeMap;

pub const STUPCI: [&str; 7] = [
    "naziv",
    "opis",
    "stil_boja",
    "stil_debljina",
    "stil_ispuna",
    "stil_znak",
    "stil_velicina",
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
pub fn vrijednosti(naziv: &str, opis: &str, s: StilObjekta) -> Vec<Value> {
    vec![
        Value::Text(naziv.into()),
        Value::Text(opis.into()),
        s.boja_crte.map(boja).into(),
        s.debljina_mm.into(),
        s.boja_ispune.map(boja).into(),
        s.oblik_znaka.map(|z| znak(z).to_owned()).into(),
        s.velicina_znaka_mm.into(),
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
        VrstaGeometrije::Linija => p += &svojstvo("outlineWidth", "stil_debljina", None),
        VrstaGeometrije::Poligon => {
            p += &svojstvo(
                "outlineWidth",
                "stil_debljina",
                Some(s.debljina_mm.to_string()),
            )
        }
    }
    format!(
        r#"<data_defined_properties><Option type="Map"><Option name="name" value="" type="QString"/><Option name="properties" type="Map">{p}</Option><Option name="type" value="collection" type="QString"/></Option></data_defined_properties>"#
    )
}
pub fn qml(v: VrstaGeometrije, s: Stil) -> String {
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
                + &opcija("line_width", &s.debljina_mm.to_string()),
        ),
        VrstaGeometrije::Poligon => (
            "fill",
            "SimpleFill",
            opcija("color", &boja(s.boja_ispune))
                + &opcija("outline_color", &boja(s.boja_crte))
                + &opcija("outline_width", &s.debljina_mm.to_string()),
        ),
    };
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
        r#"<qgis version="3.44.0" styleCategories="Symbology|AttributeTable"><renderer-v2 type="singleSymbol"><symbols><symbol name="0" type="{tip}"><layer class="{klasa}" enabled="1"><Option type="Map">{o}</Option>{}</layer></symbol></symbols></renderer-v2><attributetableconfig><columns>{skriveni}</columns></attributetableconfig></qgis>"#,
        svojstva(v, s)
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
fn iscrtavanje(k: &Cvor) -> Result<&Cvor, Greska> {
    let r = dijete(dijete(k, "qgis")?, "renderer-v2")?;
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
    let r = iscrtavanje(&k)?;
    let l = prvi_sloj(r)?;
    let o = opcije(dijete(l, "Option")?);
    primijeni_opcije(&o, v, s)
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
    Ok(s)
}
fn poligon(o: &BTreeMap<String, String>, mut s: Stil) -> Result<Stil, Greska> {
    s.boja_crte = uzmi(o, "outline_color", s.boja_crte, procitaj_boju)?;
    s.boja_ispune = uzmi(o, "color", s.boja_ispune, procitaj_boju)?;
    s.debljina_mm = uzmi(o, "outline_width", s.debljina_mm, decimalni)?;
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
        s.velicina_znaka_mm
    ])
    .to_string()
}
pub fn iz_json(s: &str) -> Result<Stil, Greska> {
    let (crta, debljina, ispuna, oblik, velicina): (String, f64, String, String, f64) =
        serde_json::from_str(s)?;
    Ok(Stil {
        boja_crte: procitaj_boju(&crta)?,
        debljina_mm: debljina,
        boja_ispune: procitaj_boju(&ispuna)?,
        oblik_znaka: procitaj_znak(&oblik)?,
        velicina_znaka_mm: velicina,
    })
}

#[cfg(test)]
mod testovi {
    use super::*;
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
