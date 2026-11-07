//! Stil sloja u QML-u, onako kako ga QGIS čita iz tablice `layer_styles`, i čitanje natrag.
//!
//! Stil jednog objekta drži se u običnim stupcima `stil_*`. U QML-u su to pravila „boja iz
//! stupca”: gdje je stupac prazan, vrijedi stil sloja.

use quick_xml::Reader;
use quick_xml::events::Event;
use rusqlite::{Connection, OptionalExtension};
use serde_json::{Map, Value as Json, json};

use crate::paket::Rezultat;

pub enum Simbol {
    Tocka { boja: &'static str, velicina: f64, znak: &'static str },
    Linija { boja: &'static str, debljina: f64, crta: &'static str },
    Lik { ispuna: &'static str, rub: &'static str, debljina: f64 },
}

fn opcija(ime: &str, vrijednost: &str) -> String {
    format!(r#"<Option type="QString" name="{ime}" value="{vrijednost}"/>"#)
}

/// Svojstvo iz stupca. Za prazan stupac QGIS boju i debljinu linije uzme iz sloja, ali širinu
/// ruba lika i oblik znaka zadrži od prethodnog objekta, pa njima treba `zamjena`.
struct IzStupca {
    svojstvo: &'static str,
    stupac: &'static str,
    zamjena: Option<String>,
}

fn stupac(svojstvo: &'static str, stupac: &'static str) -> IzStupca {
    IzStupca { svojstvo, stupac, zamjena: None }
}

fn iz_stupca(svojstva: &[IzStupca]) -> String {
    let mut s = String::from(r#"<data_defined_properties><Option type="Map"><Option type="QString" name="name" value=""/><Option type="Map" name="properties">"#);
    for p in svojstva {
        let (svojstvo, stupac) = (p.svojstvo, p.stupac);
        let izvor = match &p.zamjena {
            None => format!(r#"<Option type="QString" name="field" value="{stupac}"/><Option type="int" name="type" value="2"/>"#),
            Some(z) => format!(r#"<Option type="QString" name="expression" value="coalesce(&quot;{stupac}&quot;,{z})"/><Option type="int" name="type" value="3"/>"#),
        };
        s += &format!(r#"<Option type="Map" name="{svojstvo}"><Option type="bool" name="active" value="true"/>{izvor}</Option>"#);
    }
    s + r#"</Option><Option type="QString" name="type" value="collection"/></Option></data_defined_properties>"#
}

fn sloj_simbola(s: &Simbol) -> (&'static str, String) {
    match s {
        Simbol::Tocka { boja, velicina, znak } => {
            let o = [opcija("name", znak), opcija("color", boja), opcija("outline_color", "#ffffff"), opcija("outline_width", "0.2"), opcija("size", &velicina.to_string())].concat();
            ("marker", format!(
                r#"<layer pass="0" class="SimpleMarker" enabled="1" locked="0"><Option type="Map">{o}</Option>{}</layer>"#,
                iz_stupca(&[stupac("fillColor", "stil_boja"), stupac("size", "stil_velicina"), IzStupca { svojstvo: "name", stupac: "stil_znak", zamjena: Some(format!("'{znak}'")) }])
            ))
        }
        Simbol::Linija { boja, debljina, crta } => {
            let o = [opcija("line_color", boja), opcija("line_width", &debljina.to_string()), opcija("line_style", crta), opcija("capstyle", "flat")].concat();
            ("line", format!(
                r#"<layer pass="0" class="SimpleLine" enabled="1" locked="0"><Option type="Map">{o}</Option>{}</layer>"#,
                iz_stupca(&[stupac("outlineColor", "stil_boja"), stupac("outlineWidth", "stil_debljina"), stupac("outlineStyle", "stil_crta")])
            ))
        }
        Simbol::Lik { ispuna, rub, debljina } => {
            let o = [opcija("color", ispuna), opcija("style", "solid"), opcija("outline_color", rub), opcija("outline_style", "solid"), opcija("outline_width", &debljina.to_string())].concat();
            ("fill", format!(
                r#"<layer pass="0" class="SimpleFill" enabled="1" locked="0"><Option type="Map">{o}</Option>{}</layer>"#,
                iz_stupca(&[stupac("fillColor", "stil_ispuna"), stupac("outlineColor", "stil_boja"), IzStupca { svojstvo: "outlineWidth", stupac: "stil_debljina", zamjena: Some(debljina.to_string()) }])
            ))
        }
    }
}

/// Najmanji QML koji stilizira sloj, ispisuje natpise i skriva stupce `stil_*` u tablici.
pub fn qml(simbol: &Simbol, stupci: &[&str], natpis: &str) -> String {
    let (vrsta, sloj) = sloj_simbola(simbol);
    // QGIS: 0 oko točke, 1 preko točke, 2 uz liniju.
    let polozaj = match simbol {
        Simbol::Tocka { .. } => 0,
        Simbol::Linija { .. } => 2,
        Simbol::Lik { .. } => 1,
    };
    let stupci: String = stupci
        .iter()
        .map(|s| format!(r#"<column name="{s}" type="field" hidden="{}" width="-1"/>"#, i32::from(s.starts_with("stil_"))))
        .collect();
    format!(
        r#"<!DOCTYPE qgis PUBLIC 'http://mrcc.com/qgis.dtd' 'SYSTEM'>
<qgis version="3.44.0" styleCategories="Symbology|Labeling|AttributeTable" labelsEnabled="1">
 <renderer-v2 type="singleSymbol" symbollevels="0" forceraster="0" enableorderby="0">
  <symbols><symbol type="{vrsta}" name="0" alpha="1" clip_to_extent="1" force_rhr="0">{sloj}</symbol></symbols>
 </renderer-v2>
 <labeling type="simple"><settings calloutType="simple"><text-style fieldName="{natpis}" fontSize="9" fontSizeUnit="Point" textColor="30,30,30,255" fontFamily="Arial" isExpression="0"/><placement placement="{polozaj}" dist="1" placementFlags="10"/></settings></labeling>
 <attributetableconfig sortOrder="0" sortExpression="" actionWidgetStyle="dropDown"><columns>{stupci}<column type="actions" hidden="1" width="-1"/></columns></attributetableconfig>
</qgis>
"#
    )
}

/// Zadani stil sloja, kako ga program može prikazati.
pub fn procitaj_zadani(c: &Connection, tablica: &str) -> Rezultat<Json> {
    let postoji: i64 = c.query_row("SELECT count(*) FROM sqlite_master WHERE name = 'layer_styles'", [], |r| r.get(0))?;
    if postoji == 0 {
        return Ok(Json::Null);
    }
    let broj: i64 = c.query_row("SELECT count(*) FROM layer_styles WHERE f_table_name = ?1", [tablica], |r| r.get(0))?;
    let red: Option<(String, String)> = c
        .query_row(
            "SELECT styleName, styleQML FROM layer_styles WHERE f_table_name = ?1 AND useAsDefault = 1 ORDER BY update_time DESC, id DESC LIMIT 1",
            [tablica],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    let Some((naziv, qml)) = red else { return Ok(json!({ "stilova": broj })) };
    let mut o = procitaj_qml(&qml)?;
    o.insert("naziv".into(), json!(naziv));
    o.insert("stilova".into(), json!(broj));
    Ok(Json::Object(o))
}

/// Čita prvi sloj simbola. Sve drugo osim jednog simbola program zasad ne zna prikazati.
pub fn procitaj_qml(qml: &str) -> Rezultat<Map<String, Json>> {
    let mut r = Reader::from_str(qml);
    let (mut o, mut opcije) = (Map::new(), Map::new());
    let (mut u_iscrtavanju, mut u_sloju, mut u_svojstvima, mut slojeva) = (false, false, 0_u32, 0_u32);
    let mut iz_stupaca = Vec::new();
    loop {
        match r.read_event()? {
            Event::Eof => break,
            // QGIS isti <layer> piše i za odabir i legendu, pa vrijedi samo onaj iz iscrtavanja.
            Event::Start(e) if e.name().as_ref() == b"renderer-v2" => {
                u_iscrtavanju = true;
                o.insert("iscrtavanje".into(), json!(atribut(&e, "type")));
            }
            Event::End(e) if e.name().as_ref() == b"renderer-v2" => u_iscrtavanju = false,
            Event::Start(e) if u_iscrtavanju && e.name().as_ref() == b"layer" => {
                slojeva += 1;
                if slojeva == 1 {
                    u_sloju = true;
                    o.insert("simbol".into(), json!(atribut(&e, "class")));
                }
            }
            Event::End(e) if e.name().as_ref() == b"layer" => u_sloju = false,
            Event::Start(e) if u_sloju && e.name().as_ref() == b"data_defined_properties" => u_svojstvima = 1,
            Event::End(e) if u_sloju && e.name().as_ref() == b"data_defined_properties" => u_svojstvima = 0,
            Event::Start(e) if u_svojstvima > 0 && e.name().as_ref() == b"Option" => {
                if let Some(ime) = atribut(&e, "name").filter(|i| i != "properties") {
                    iz_stupaca.push(ime);
                }
            }
            Event::Empty(e) if u_sloju && u_svojstvima == 0 && e.name().as_ref() == b"Option" => {
                if let (Some(i), Some(v)) = (atribut(&e, "name"), atribut(&e, "value")) {
                    opcije.insert(i, json!(v));
                }
            }
            _ => {}
        }
    }
    o.insert("slojeva_simbola".into(), json!(slojeva));
    o.insert("opcije".into(), Json::Object(opcije));
    o.insert("iz_stupaca".into(), json!(iz_stupaca));
    Ok(o)
}

fn atribut(e: &quick_xml::events::BytesStart<'_>, ime: &str) -> Option<String> {
    e.try_get_attribute(ime).ok().flatten().and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn nas_qml_se_cita_natrag() {
        let t = qml(&Simbol::Linija { boja: "#0f766e", debljina: 0.8, crta: "solid" }, &["naziv", "stil_boja"], "naziv");
        let o = procitaj_qml(&t).unwrap();
        assert_eq!(o["iscrtavanje"], "singleSymbol");
        assert_eq!(o["simbol"], "SimpleLine");
        assert_eq!(o["opcije"]["line_color"], "#0f766e");
        assert_eq!(o["opcije"]["line_width"], "0.8");
        assert!(o["iz_stupaca"].to_string().contains("outlineColor"));
    }
}
