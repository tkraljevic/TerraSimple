//! Čitanje pomoću biblioteke `kml` 0.15 iz plana, radi izravne usporedbe s našim čitačem.
//!
//! Pokazuje što `kml` 0.15 tiho gubi ili odbija na stvarnim i izmišljenim datotekama.

use std::path::Path;

use kml::types::{Coord, Element, Geometry, Kml};
use kml::KmlReader;

use crate::model::StatistikaKml;

fn zbroji_vrh(c: &Coord<f64>, stat: &mut StatistikaKml) {
    stat.ukupno_vrhova += 1;
    if let Some(z) = c.z {
        stat.vrhovi_s_visinom += 1;
        if z.abs() > 1e-9 {
            stat.vrhovi_s_nenultom_visinom += 1;
        }
    }
}

fn zbroji_geometriju(g: &Geometry<f64>, stat: &mut StatistikaKml) {
    match g {
        Geometry::Point(p) => {
            stat.tocke += 1;
            zbroji_vrh(&p.coord, stat);
        }
        Geometry::LineString(l) => {
            stat.linije += 1;
            for c in &l.coords {
                zbroji_vrh(c, stat);
            }
        }
        Geometry::LinearRing(r) => {
            stat.linije += 1;
            for c in &r.coords {
                zbroji_vrh(c, stat);
            }
        }
        Geometry::Polygon(poly) => {
            stat.poligoni += 1;
            stat.rupe_poligona += poly.inner.len();
            for c in &poly.outer.coords {
                zbroji_vrh(c, stat);
            }
            for rupa in &poly.inner {
                for c in &rupa.coords {
                    zbroji_vrh(c, stat);
                }
            }
        }
        Geometry::MultiGeometry(mg) => {
            stat.vise_geometrija += 1;
            for pod in &mg.geometries {
                zbroji_geometriju(pod, stat);
            }
        }
        _ => {}
    }
}

fn zbroji_genericki_element(el: &Element, stat: &mut StatistikaKml) {
    match el.name.as_str() {
        "NetworkLink" => stat.mrezne_veze += 1,
        "GroundOverlay" => stat.slikovni_prekrivaci += 1,
        "LookAt" | "Camera" => stat.pogledi_kamere += 1,
        "TimeStamp" | "TimeSpan" => stat.vremenski_zapisi += 1,
        "ExtendedData" => stat.prosireni_podaci += 1,
        _ => {}
    }
    for d in &el.children {
        zbroji_genericki_element(d, stat);
    }
}

fn obidi_kml(
    k: &Kml<f64>,
    dubina_mapa: usize,
    dubina_spremnika: usize,
    stat: &mut StatistikaKml,
) {
    if dubina_mapa > stat.najveca_dubina_mapa {
        stat.najveca_dubina_mapa = dubina_mapa;
    }
    if dubina_spremnika > stat.najveca_dubina_spremnika {
        stat.najveca_dubina_spremnika = dubina_spremnika;
    }
    match k {
        Kml::KmlDocument(d) => {
            for e in &d.elements {
                obidi_kml(e, dubina_mapa, dubina_spremnika, stat);
            }
        }
        Kml::Document { elements, .. } => {
            stat.dokumenti += 1;
            let nova_spremnika = dubina_spremnika + 1;
            if nova_spremnika > stat.najveca_dubina_spremnika {
                stat.najveca_dubina_spremnika = nova_spremnika;
            }
            for e in elements {
                if let Kml::Element(el) = e {
                    if el.name == "description"
                        && el.content.as_ref().is_some_and(|s| !s.trim().is_empty())
                    {
                        stat.mape_s_opisom += 1;
                    }
                }
                obidi_kml(e, dubina_mapa, nova_spremnika, stat);
            }
        }
        Kml::Folder(f) => {
            stat.mape += 1;
            let nova_mapa = dubina_mapa + 1;
            let nova_spremnika = dubina_spremnika + 1;
            if nova_mapa > stat.najveca_dubina_mapa {
                stat.najveca_dubina_mapa = nova_mapa;
            }
            if nova_spremnika > stat.najveca_dubina_spremnika {
                stat.najveca_dubina_spremnika = nova_spremnika;
            }
            if f.description.as_ref().is_some_and(|s| !s.trim().is_empty()) {
                stat.mape_s_opisom += 1;
            }
            for e in &f.elements {
                obidi_kml(e, nova_mapa, nova_spremnika, stat);
            }
        }
        Kml::Placemark(p) => {
            stat.objekti += 1;
            if p.name.as_ref().is_some_and(|s| !s.trim().is_empty()) {
                stat.objekti_s_nazivom += 1;
            }
            if p.description.as_ref().is_some_and(|s| !s.trim().is_empty()) {
                stat.objekti_s_opisom += 1;
            }
            if let Some(ref g) = p.geometry {
                zbroji_geometriju(g, stat);
            } else {
                stat.objekti_bez_geometrije += 1;
            }
            for el in &p.children {
                zbroji_genericki_element(el, stat);
            }
        }
        Kml::Style(_) => stat.stilovi += 1,
        Kml::StyleMap(_) => stat.mape_stilova += 1,
        Kml::Element(el) => zbroji_genericki_element(el, stat),
        _ => {}
    }
}

/// Sigurni naziv vrste greške iz `kml` 0.15, bez ispisa teksta iz datoteke.
fn naziv_greske_kml(e: &kml::Error) -> &'static str {
    match e {
        kml::Error::InvalidInput(_) => "InvalidInput",
        kml::Error::MalformedXml(_) => "MalformedXml",
        kml::Error::InvalidXmlEvent(_) => "InvalidXmlEvent",
        kml::Error::CoordEmpty => "CoordEmpty",
        kml::Error::NoElements => "NoElements",
        kml::Error::NumParse(_) => "NumParse",
        kml::Error::InvalidKmlVersion(_) => "InvalidKmlVersion",
        kml::Error::InvalidKmlElement(_) => "InvalidKmlElement",
        kml::Error::InvalidGeometry(_) => "InvalidGeometry",
        kml::Error::InvalidAltitudeMode(_) => "InvalidAltitudeMode",
        kml::Error::InvalidColorMode(_) => "InvalidColorMode",
        kml::Error::InvalidListItemType(_) => "InvalidListItemType",
        kml::Error::InvalidRefreshMode(_) => "InvalidRefreshMode",
        kml::Error::InvalidViewRefreshMode(_) => "InvalidViewRefreshMode",
        kml::Error::IoError(_) => "IoError",
        kml::Error::ZipError(_) => "ZipError",
        kml::Error::InvalidUnits(_) => "InvalidUnits",
    }
}

pub fn procitaj_s_kml_crateom(put: &Path) -> Result<StatistikaKml, &'static str> {
    let put_buf = put.to_path_buf();
    let je_kmz = put
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("kmz"));
    let rezultat = std::panic::catch_unwind(move || {
        if je_kmz {
            let mut r = KmlReader::<_, f64>::from_kmz_path(&put_buf)?;
            r.read()
        } else {
            let mut r = KmlReader::<_, f64>::from_path(&put_buf)?;
            r.read()
        }
    });
    match rezultat {
        Ok(Ok(kml_stablo)) => {
            let mut stat = StatistikaKml::default();
            obidi_kml(&kml_stablo, 0, 0, &mut stat);
            Ok(stat)
        }
        Ok(Err(e)) => Err(naziv_greske_kml(&e)),
        Err(_) => Err("PANIKA_U_KML_CRATEU"),
    }
}
