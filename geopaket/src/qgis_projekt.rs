//! Pohrana QGIS projekta unutar tablice `qgis_projects` u GeoPackageu.
//!
//! Omogućuje da QGIS pri otvaranju prepozna nazive slojeva, mape i stilove
//! identično kao u TerraSimpleu.

use crate::qgis_stari::{self, SlojQgisa};
use crate::qgis_sustavi::{HTRS96_TM, PSEUDO_MERKATOR};
use crate::{Greska, qgis_sustavi, stil};
use koordinate::{Pretvornik, Sustav};
use rusqlite::Connection;
use slojevi::{Geometrija, IdSloja, Projekt, Stavka, VrstaGeometrije};
use std::collections::BTreeMap;
use std::path::Path;

/// Naziv QGIS projekta koji sprema TerraSimple.
pub const NAZIV_PROJEKTA: &str = "TerraSimple";

/// Sprema ili zamjenjuje projekt `TerraSimple` u tablici `qgis_projects`.
///
/// Ostali projekti u tablici ostaju netaknuti. Sloj kojem stil nije mijenjan od `prije`
/// zadržava svoj element iz projekta koji je već u datoteci (Odluke (9. 11. 2026.)).
pub fn spremi(c: &Connection, prije: &Projekt, p: &Projekt) -> Result<(), Greska> {
    stvori_tablicu_ako_ne_postoji(c)?;
    let ime_datoteke = dohvati_ime_datoteke(c);
    let sacuvani = sacuvani_slojevi(c, prije, p);
    let xml = generiraj_qgs_sa(p, &ime_datoteke, &sacuvani);
    let zip_bajti = zapakiraj_qgz(xml.as_bytes())?;
    let heks_sadrzaj = heksadecimalni_niz(&zip_bajti);

    c.execute(
        "INSERT INTO qgis_projects(name, metadata, content)
         VALUES (?1, json_object('last_modified_time', strftime('%Y-%m-%dT%H:%M:%fZ','now'), 'last_modified_user', 'TerraSimple'), ?2)
         ON CONFLICT(name) DO UPDATE SET metadata=excluded.metadata, content=excluded.content",
        rusqlite::params![NAZIV_PROJEKTA, heks_sadrzaj],
    )?;
    Ok(())
}

fn stvori_tablicu_ako_ne_postoji(c: &Connection) -> Result<(), Greska> {
    c.execute_batch(
        "CREATE TABLE IF NOT EXISTS qgis_projects(name TEXT PRIMARY KEY, metadata BLOB, content BLOB);",
    )?;
    Ok(())
}

fn dohvati_ime_datoteke(c: &Connection) -> String {
    let staza: Result<String, _> = c.query_row("PRAGMA database_list", [], |r| r.get(2));
    match staza {
        Ok(s) => match Path::new(&s).file_name().and_then(|n| n.to_str()) {
            Some(ime) if !ime.is_empty() => ime.to_string(),
            _ => "projekt.gpkg".into(),
        },
        Err(_) => "projekt.gpkg".into(),
    }
}

/// Slojevi kojima TerraSimple nije mijenjao stil, s elementom iz starog projekta.
fn sacuvani_slojevi(c: &Connection, prije: &Projekt, p: &Projekt) -> BTreeMap<IdSloja, SlojQgisa> {
    let mut stari = qgis_stari::slojevi(c, NAZIV_PROJEKTA);
    p.slojevi()
        .filter(|(id, s)| prije.sloj(*id).is_some_and(|ps| ps.stil == s.stil))
        .filter_map(|(id, _)| stari.remove(&id_sloja(p, id)).map(|x| (id, x)))
        .collect()
}

#[cfg(test)]
pub fn generiraj_qgs(p: &Projekt, ime_datoteke: &str) -> String {
    generiraj_qgs_sa(p, ime_datoteke, &BTreeMap::new())
}

fn generiraj_qgs_sa(
    p: &Projekt,
    ime_datoteke: &str,
    sacuvani: &BTreeMap<IdSloja, SlojQgisa>,
) -> String {
    let (min_x, min_y, max_x, max_y) = opseg_projekta(p);
    let mut xml = String::with_capacity(4096);
    xml.push_str("<!DOCTYPE qgis PUBLIC 'http://mrcc.com/qgis.dtd' 'SYSTEM'>\n");
    xml.push_str("<qgis version=\"3.44.15\" projectname=\"TerraSimple\">\n");
    xml.push_str("  <homePath path=\"\"/>\n");
    xml.push_str("  <title>TerraSimple</title>\n");
    dodaj_crs_xml(&mut xml);
    dodaj_stablo_xml(&mut xml, p, ime_datoteke);
    dodaj_slojeve_xml(&mut xml, p, ime_datoteke, sacuvani);
    dodaj_redoslijed_xml(&mut xml, p);
    dodaj_pogled_xml(&mut xml, min_x, min_y, max_x, max_y);
    xml.push_str("</qgis>\n");
    xml
}

fn dodaj_crs_xml(xml: &mut String) {
    xml.push_str(&format!("  <projectCrs>{HTRS96_TM}</projectCrs>\n"));
}

fn dodaj_stablo_xml(xml: &mut String, p: &Projekt, ime_datoteke: &str) {
    xml.push_str("  <layer-tree-group>\n");
    xml.push_str("    <customproperties><Option/></customproperties>\n");
    for stavka in p.korijen() {
        stablo_stavka_xml(xml, p, stavka, ime_datoteke, 2);
    }
    xml.push_str(
        "    <layer-tree-layer id=\"osm_podloga\" name=\"OpenStreetMap\" source=\"type=xyz&amp;zmin=0&amp;zmax=19&amp;url=https://tile.openstreetmap.org/{z}/{x}/{y}.png\" providerKey=\"wms\" checked=\"Qt::Checked\" expanded=\"1\">\n\
         \x20     <customproperties><Option/></customproperties>\n\
         \x20   </layer-tree-layer>\n",
    );
    xml.push_str("    <custom-order enabled=\"0\">\n");
    for id in slojevi_po_stablu(p) {
        xml.push_str(&format!(
            "      <item>{}</item>\n",
            xml_bijeg(&id_sloja(p, id))
        ));
    }
    xml.push_str("      <item>osm_podloga</item>\n");
    xml.push_str("    </custom-order>\n");
    xml.push_str("  </layer-tree-group>\n");
}

fn stablo_stavka_xml(
    xml: &mut String,
    p: &Projekt,
    stavka: &Stavka,
    ime_datoteke: &str,
    uvlaka: usize,
) {
    let razmak = " ".repeat(uvlaka * 2);
    match stavka {
        Stavka::Mapa(id) => {
            if let Some(m) = p.mapa(*id) {
                xml.push_str(&format!(
                    "{razmak}<layer-tree-group name=\"{}\" checked=\"Qt::Checked\" expanded=\"1\">\n",
                    xml_bijeg(&m.naziv)
                ));
                xml.push_str(&format!(
                    "{razmak}  <customproperties><Option/></customproperties>\n"
                ));
                for dijete in &m.djeca {
                    stablo_stavka_xml(xml, p, dijete, ime_datoteke, uvlaka + 1);
                }
                xml.push_str(&format!("{razmak}</layer-tree-group>\n"));
            }
        }
        Stavka::Sloj(id) => {
            if let Some(s) = p.sloj(*id) {
                let id_tekst = id_sloja(p, *id);
                let tablica = crate::pisanje::tablica(p, *id);
                xml.push_str(&format!(
                    "{razmak}<layer-tree-layer id=\"{}\" name=\"{}\" source=\"./{}|layername={}\" providerKey=\"ogr\" checked=\"Qt::Checked\" expanded=\"1\">\n",
                    xml_bijeg(&id_tekst),
                    xml_bijeg(&s.naziv),
                    xml_bijeg(ime_datoteke),
                    xml_bijeg(&tablica)
                ));
                xml.push_str(&format!(
                    "{razmak}  <customproperties><Option/></customproperties>\n"
                ));
                xml.push_str(&format!("{razmak}</layer-tree-layer>\n"));
            }
        }
    }
}

fn dodaj_slojeve_xml(
    xml: &mut String,
    p: &Projekt,
    ime_datoteke: &str,
    sacuvani: &BTreeMap<IdSloja, SlojQgisa>,
) {
    // `<srs>` mora biti sustav u kojem je sloj stvarno spremljen (WGS84, HTRS96/TM, HDKS).
    // QGIS ga sluša i bez njega sloj ostaje bez sustava, a s krivim crta objekte na krivom mjestu.
    xml.push_str("  <projectlayers>\n");
    for (id, s) in p.slojevi() {
        let id_tekst = id_sloja(p, id);
        let tablica = crate::pisanje::tablica(p, id);
        if let Some(stari) = sacuvani.get(&id) {
            let izvor = format!("./{}|layername={}", ime_datoteke, tablica);
            xml.push_str("    ");
            xml.push_str(&stari.prilagodi(&xml_bijeg(&izvor), &xml_bijeg(&s.naziv)));
            xml.push('\n');
            continue;
        }
        let (geom_tip, wkb_tip) = geometrija_oznake(s.vrsta);
        let renderer = stil::renderer(s.vrsta, s.stil);
        let srs = qgis_sustavi::zapis(sustav_sloja(p, id));

        xml.push_str(&format!(
            "    <maplayer type=\"vector\" geometry=\"{geom_tip}\" wkbType=\"{wkb_tip}\" styleCategories=\"AllStyleCategories\">\n\
             \x20     <id>{}</id>\n\
             \x20     <datasource>./{}|layername={}</datasource>\n\
             \x20     <layername>{}</layername>\n\
             \x20     <srs>{srs}</srs>\n\
             \x20     <provider>ogr</provider>\n\
             \x20     {renderer}\n\
             \x20   </maplayer>\n",
            xml_bijeg(&id_tekst),
            xml_bijeg(ime_datoteke),
            xml_bijeg(&tablica),
            xml_bijeg(&s.naziv)
        ));
    }

    // Podloga OpenStreetMap na dnu
    xml.push_str(&format!(
        "    <maplayer type=\"raster\" styleCategories=\"AllStyleCategories\">\n\
         \x20     <id>osm_podloga</id>\n\
         \x20     <datasource>type=xyz&amp;zmin=0&amp;zmax=19&amp;url=https://tile.openstreetmap.org/{{z}}/{{x}}/{{y}}.png</datasource>\n\
         \x20     <layername>OpenStreetMap</layername>\n\
         \x20     <srs>{PSEUDO_MERKATOR}</srs>\n\
         \x20     <provider>wms</provider>\n\
         \x20   </maplayer>\n",
    ));
    xml.push_str("  </projectlayers>\n");
}

fn dodaj_redoslijed_xml(xml: &mut String, p: &Projekt) {
    xml.push_str("  <layerorder>\n");
    for id in slojevi_po_stablu(p) {
        xml.push_str(&format!(
            "    <layer id=\"{}\"/>\n",
            xml_bijeg(&id_sloja(p, id))
        ));
    }
    xml.push_str("    <layer id=\"osm_podloga\"/>\n");
    xml.push_str("  </layerorder>\n");
}

/// Početni pogled. QGIS ga čita iz `mapcanvas` s imenom svog glavnog platna, a
/// `DefaultViewExtent` je pogled za „Prikaži cijeli projekt”.
fn dodaj_pogled_xml(xml: &mut String, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
    xml.push_str(&format!(
        "  <mapcanvas annotationsVisible=\"1\" name=\"theMapCanvas\">\n\
         \x20   <units>meters</units>\n\
         \x20   <extent><xmin>{min_x:.3}</xmin><ymin>{min_y:.3}</ymin><xmax>{max_x:.3}</xmax><ymax>{max_y:.3}</ymax></extent>\n\
         \x20   <rotation>0</rotation>\n\
         \x20   <destinationsrs>{HTRS96_TM}</destinationsrs>\n\
         \x20 </mapcanvas>\n\
         \x20 <ProjectViewSettings rotation=\"0\" UseProjectScales=\"0\">\n\
         \x20   <Scales/>\n\
         \x20   <DefaultViewExtent xmin=\"{min_x:.3}\" ymin=\"{min_y:.3}\" xmax=\"{max_x:.3}\" ymax=\"{max_y:.3}\">{HTRS96_TM}</DefaultViewExtent>\n\
         \x20 </ProjectViewSettings>\n\
         \x20 <properties>\n\
         \x20   <SpatialRefSys>\n\
         \x20     <ProjectionsEnabled type=\"int\">1</ProjectionsEnabled>\n\
         \x20   </SpatialRefSys>\n\
         \x20 </properties>\n"
    ));
    // Bez `ProjectionsEnabled` QGIS 3.44 projekt otvori bez sustava, iako `projectCrs` postoji.
}

/// Sustav u kojem je sloj spremljen; novi slojevi spremaju se u WGS84.
fn sustav_sloja(p: &Projekt, id: IdSloja) -> Sustav {
    p.zapisi()
        .slojevi
        .get(&id)
        .map_or(Sustav::Wgs84, |z| z.sustav)
}

/// Oznaka sloja u projektu. QGIS oznaku od 10 ili manje znakova zanemari i sloju dade
/// novu, pa ga stavka u stablu više ne nađe; zato oznaka uvijek počinje s „TerraSimple_”.
fn id_sloja(p: &Projekt, id: IdSloja) -> String {
    format!("TerraSimple_{}_{}", crate::pisanje::tablica(p, id), id.0)
}

/// Slojevi redom kako stoje u stablu, od vrha prema dnu, i oni u mapama.
fn slojevi_po_stablu(p: &Projekt) -> Vec<IdSloja> {
    let mut redom = Vec::new();
    dodaj_po_stablu(p, p.korijen(), &mut redom);
    redom
}

fn dodaj_po_stablu(p: &Projekt, stavke: &[Stavka], redom: &mut Vec<IdSloja>) {
    for stavka in stavke {
        match stavka {
            Stavka::Sloj(id) => redom.push(*id),
            Stavka::Mapa(id) => p
                .mapa(*id)
                .iter()
                .for_each(|m| dodaj_po_stablu(p, &m.djeca, redom)),
        }
    }
}

fn geometrija_oznake(vrsta: VrstaGeometrije) -> (&'static str, &'static str) {
    match vrsta {
        VrstaGeometrije::Tocka => ("Point", "Point"),
        VrstaGeometrije::Linija => ("Line", "LineString"),
        VrstaGeometrije::Poligon => ("Polygon", "Polygon"),
    }
}

pub fn opseg_projekta(p: &Projekt) -> (f64, f64, f64, f64) {
    let mut opseg = (
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    );

    for (s_id, s) in p.slojevi() {
        let sustav = sustav_sloja(p, s_id);
        let pretvornik = if sustav == Sustav::Htrs96Tm {
            None
        } else {
            Pretvornik::novi(sustav, Sustav::Htrs96Tm).ok()
        };
        for obj in s.objekti.values() {
            obradi_geometriju_opseg(&obj.geometrija, sustav, &pretvornik, &mut opseg);
        }
    }

    let (min_x, min_y, max_x, max_y) = opseg;
    if min_x.is_infinite() {
        // Zadani opseg Republike Hrvatske u EPSG:3765
        (200_000.0, 5_000_000.0, 750_000.0, 5_150_000.0)
    } else {
        let dx = ((max_x - min_x).max(100.0)) * 0.05;
        let dy = ((max_y - min_y).max(100.0)) * 0.05;
        (min_x - dx, min_y - dy, max_x + dx, max_y + dy)
    }
}

fn obradi_geometriju_opseg(
    g: &Geometrija,
    sustav: Sustav,
    pretvornik: &Option<Pretvornik>,
    opseg: &mut (f64, f64, f64, f64),
) {
    match g {
        Geometrija::Tocka(v) => prosiri_tockom(v.duzina, v.sirina, sustav, pretvornik, opseg),
        Geometrija::Linija(vrhovi) => {
            for v in vrhovi {
                prosiri_tockom(v.duzina, v.sirina, sustav, pretvornik, opseg);
            }
        }
        Geometrija::Poligon(prstenovi) => {
            for prsten in prstenovi {
                for v in prsten {
                    prosiri_tockom(v.duzina, v.sirina, sustav, pretvornik, opseg);
                }
            }
        }
    }
}

fn prosiri_tockom(
    x: f64,
    y: f64,
    sustav: Sustav,
    pretvornik: &Option<Pretvornik>,
    opseg: &mut (f64, f64, f64, f64),
) {
    let tocka = match sustav {
        Sustav::Htrs96Tm => Some([x, y]),
        _ => pretvornik.as_ref().and_then(|p| p.pretvori([x, y]).ok()),
    };
    if let Some([e, n]) = tocka {
        opseg.0 = opseg.0.min(e);
        opseg.1 = opseg.1.min(n);
        opseg.2 = opseg.2.max(e);
        opseg.3 = opseg.3.max(n);
    }
}

pub fn zapakiraj_qgz(qgs_xml: &[u8]) -> Result<Vec<u8>, Greska> {
    use flate2::{Compression, Crc, write::DeflateEncoder};
    use std::io::Write;

    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(qgs_xml)?;
    let komprimirano = encoder.finish()?;

    let mut crc = Crc::new();
    crc.update(qgs_xml);
    let crc32 = crc.sum();

    let ime = b"TerraSimple.qgs";
    let vel_nekompr = qgs_xml.len() as u32;
    let vel_kompr = komprimirano.len() as u32;
    let duljina_imena = ime.len() as u16;

    let mut zip = Vec::with_capacity(128 + komprimirano.len());

    // 1. Lokalno zaglavlje datoteke
    zip.extend_from_slice(&0x04034b50_u32.to_le_bytes());
    zip.extend_from_slice(&20_u16.to_le_bytes());
    zip.extend_from_slice(&0x0800_u16.to_le_bytes());
    zip.extend_from_slice(&8_u16.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&crc32.to_le_bytes());
    zip.extend_from_slice(&vel_kompr.to_le_bytes());
    zip.extend_from_slice(&vel_nekompr.to_le_bytes());
    zip.extend_from_slice(&duljina_imena.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(ime);

    // 2. Komprimirani podaci
    zip.extend_from_slice(&komprimirano);

    // 3. Središnji direktorij
    let pocetak_sredisnjeg = zip.len() as u32;
    zip.extend_from_slice(&0x02014b50_u32.to_le_bytes());
    zip.extend_from_slice(&20_u16.to_le_bytes());
    zip.extend_from_slice(&20_u16.to_le_bytes());
    zip.extend_from_slice(&0x0800_u16.to_le_bytes());
    zip.extend_from_slice(&8_u16.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&crc32.to_le_bytes());
    zip.extend_from_slice(&vel_kompr.to_le_bytes());
    zip.extend_from_slice(&vel_nekompr.to_le_bytes());
    zip.extend_from_slice(&duljina_imena.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&0_u32.to_le_bytes());
    zip.extend_from_slice(&0_u32.to_le_bytes());
    zip.extend_from_slice(ime);

    // 4. Kraj središnjeg direktorija
    let velicina_sredisnjeg = (zip.len() as u32) - pocetak_sredisnjeg;
    zip.extend_from_slice(&0x06054b50_u32.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());
    zip.extend_from_slice(&1_u16.to_le_bytes());
    zip.extend_from_slice(&1_u16.to_le_bytes());
    zip.extend_from_slice(&velicina_sredisnjeg.to_le_bytes());
    zip.extend_from_slice(&pocetak_sredisnjeg.to_le_bytes());
    zip.extend_from_slice(&0_u16.to_le_bytes());

    Ok(zip)
}

fn heksadecimalni_niz(bajti: &[u8]) -> String {
    let mut s = String::with_capacity(bajti.len() * 2);
    for b in bajti {
        use std::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

fn xml_bijeg(tekst: &str) -> String {
    let mut izlaz = String::with_capacity(tekst.len());
    for znak in tekst.chars() {
        match znak {
            '&' => izlaz.push_str("&amp;"),
            '<' => izlaz.push_str("&lt;"),
            '>' => izlaz.push_str("&gt;"),
            '"' => izlaz.push_str("&quot;"),
            '\'' => izlaz.push_str("&apos;"),
            ostalo => izlaz.push(ostalo),
        }
    }
    izlaz
}

#[cfg(test)]
mod testovi {
    use super::*;
    use flate2::read::DeflateDecoder;
    use std::io::Read;

    #[test]
    fn zip_okrugli_put() {
        let izvorni = b"Pozdrav iz TerraSimple QGIS projekta!";
        let zip = zapakiraj_qgz(izvorni).unwrap();
        assert_eq!(&zip[0..4], b"PK\x03\x04");

        // Provjeri lokalno zaglavlje
        let duljina_imena = u16::from_le_bytes([zip[26], zip[27]]) as usize;
        let vel_kompr = u32::from_le_bytes([zip[18], zip[19], zip[20], zip[21]]) as usize;
        let podaci_pocetak = 30 + duljina_imena;
        let kompr = &zip[podaci_pocetak..podaci_pocetak + vel_kompr];

        let mut decoder = DeflateDecoder::new(kompr);
        let mut dekomprimirano = Vec::new();
        decoder.read_to_end(&mut dekomprimirano).unwrap();
        assert_eq!(dekomprimirano, izvorni);
    }

    #[test]
    fn heks_ispravan() {
        let bajti = [0x50, 0x4b, 0x03, 0x04];
        assert_eq!(heksadecimalni_niz(&bajti), "504b0304");
    }

    #[test]
    fn xml_bijeg_ispravan() {
        assert_eq!(
            xml_bijeg("A & B < C > 'D' \"E\""),
            "A &amp; B &lt; C &gt; &apos;D&apos; &quot;E&quot;"
        );
    }

    #[test]
    fn ime_datoteke_u_memoriji_vraca_zadanu_vrijednost() {
        let c = Connection::open_in_memory().unwrap();
        assert_eq!(dohvati_ime_datoteke(&c), "projekt.gpkg");
    }

    #[test]
    fn ime_datoteke_s_diska_vraca_naziv() {
        let mapa = std::env::temp_dir().join(format!("ts_qgis_test_{}", std::process::id()));
        std::fs::create_dir_all(&mapa).unwrap();
        let staza = mapa.join("moja_karta.gpkg");
        let c = Connection::open(&staza).unwrap();
        assert_eq!(dohvati_ime_datoteke(&c), "moja_karta.gpkg");
        drop(c);
        let _ = std::fs::remove_dir_all(&mapa);
    }

    #[test]
    fn spremi_zapisuje_qgis_projekt_i_cuva_ostale() {
        let c = Connection::open_in_memory().unwrap();
        c.execute(
            "CREATE TABLE qgis_projects(name TEXT PRIMARY KEY, metadata BLOB, content BLOB)",
            [],
        )
        .unwrap();
        c.execute(
            "INSERT INTO qgis_projects VALUES ('DrugiProjekt', '{\"user\":\"drugi\"}', '1234')",
            [],
        )
        .unwrap();

        let mut u = slojevi::Uredjivac::novi();
        let s1 = match u
            .izvrsi(slojevi::NaredbaStabla::DodajSloj {
                u: None,
                naziv: "Tocke".into(),
                vrsta: slojevi::VrstaGeometrije::Tocka,
            })
            .unwrap()
        {
            Some(slojevi::Novo::Sloj(id)) => id,
            _ => panic!("nema sloja"),
        };
        u.izvrsi(slojevi::NaredbaObjekta::Dodaj {
            sloj: s1,
            naziv: "T1".into(),
            geometrija: slojevi::Geometrija::Tocka(slojevi::Vrh::novi(18.0, 45.0)),
        })
        .unwrap();

        spremi(&c, &Projekt::default(), u.projekt()).unwrap();

        // Tuđi projekt ostaje
        let drugi: String = c
            .query_row(
                "SELECT content FROM qgis_projects WHERE name = 'DrugiProjekt'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(drugi, "1234");

        // TerraSimple projekt je zapisan
        let (meta, content): (String, String) = c
            .query_row(
                "SELECT metadata, content FROM qgis_projects WHERE name = 'TerraSimple'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(meta.contains("TerraSimple"));
        assert!(!content.is_empty());
    }

    #[test]
    fn generiraj_qgs_sadrzi_sve_elemente() {
        let mut u = slojevi::Uredjivac::novi();
        let m = match u
            .izvrsi(slojevi::NaredbaStabla::DodajMapu {
                u: None,
                naziv: "Skupina".into(),
            })
            .unwrap()
        {
            Some(slojevi::Novo::Mapa(id)) => id,
            _ => panic!("nema mape"),
        };
        let s = match u
            .izvrsi(slojevi::NaredbaStabla::DodajSloj {
                u: Some(m),
                naziv: "Linije".into(),
                vrsta: slojevi::VrstaGeometrije::Linija,
            })
            .unwrap()
        {
            Some(slojevi::Novo::Sloj(id)) => id,
            _ => panic!("nema sloja"),
        };
        u.izvrsi(slojevi::NaredbaObjekta::Dodaj {
            sloj: s,
            naziv: "L1".into(),
            geometrija: slojevi::Geometrija::Linija(vec![
                slojevi::Vrh::novi(18.0, 45.0),
                slojevi::Vrh::novi(18.1, 45.1),
            ]),
        })
        .unwrap();

        let qgs = generiraj_qgs(u.projekt(), "karta.gpkg");
        assert!(qgs.contains("<projectCrs>"));
        assert!(qgs.contains("EPSG:3765"));
        assert!(qgs.contains("<mapcanvas annotationsVisible=\"1\" name=\"theMapCanvas\">"));
        assert!(qgs.contains("<DefaultViewExtent xmin="));
        assert!(qgs.contains("<ProjectionsEnabled type=\"int\">1</ProjectionsEnabled>"));
        assert!(qgs.contains(
            "<srs><spatialrefsys nativeFormat=\"Wkt\"><wkt>PROJCRS[\"WGS 84 / Pseudo-Mercator\""
        ));
        assert!(qgs.contains("<layer-tree-group name=\"Skupina\""));
        assert!(qgs.contains("name=\"Linije\""));
        assert!(qgs.contains("name=\"OpenStreetMap\""));
        assert!(qgs.contains("<maplayer type=\"vector\""));
        assert!(qgs.contains("<maplayer type=\"raster\""));
        assert!(qgs.contains("<custom-order enabled=\"0\">"));
    }

    fn dodaj_sloj(u: &mut slojevi::Uredjivac, naziv: &str) -> IdSloja {
        match u
            .izvrsi(slojevi::NaredbaStabla::DodajSloj {
                u: None,
                naziv: naziv.into(),
                vrsta: slojevi::VrstaGeometrije::Tocka,
            })
            .unwrap()
        {
            Some(slojevi::Novo::Sloj(id)) => id,
            _ => panic!("nema sloja"),
        }
    }

    #[test]
    fn oznaka_sloja_je_dovoljno_duga_za_qgis() {
        let mut u = slojevi::Uredjivac::novi();
        let id = dodaj_sloj(&mut u, "A");
        let oznaka = id_sloja(u.projekt(), id);
        assert!(oznaka.len() > 10, "{oznaka}");
        assert!(oznaka.starts_with("TerraSimple_"), "{oznaka}");
    }

    #[test]
    fn redoslijed_crtanja_prati_stablo_a_ne_redoslijed_stvaranja() {
        let mut u = slojevi::Uredjivac::novi();
        let prvi = dodaj_sloj(&mut u, "Prvi");
        let drugi = dodaj_sloj(&mut u, "Drugi");
        let m = match u
            .izvrsi(slojevi::NaredbaStabla::DodajMapu {
                u: None,
                naziv: "Mapa".into(),
            })
            .unwrap()
        {
            Some(slojevi::Novo::Mapa(id)) => id,
            _ => panic!("nema mape"),
        };
        // Prvi sloj ide u mapu, a drugi na vrh korijena.
        for (stavka, u_mapu, polozaj) in [
            (Stavka::Sloj(prvi), Some(m), 0),
            (Stavka::Sloj(drugi), None, 0),
        ] {
            u.izvrsi(slojevi::NaredbaStabla::Premjesti {
                stavka,
                u: u_mapu,
                polozaj,
            })
            .unwrap();
        }
        let p = u.projekt();
        assert_eq!(p.korijen()[0], Stavka::Sloj(drugi));
        assert_eq!(slojevi_po_stablu(p), vec![drugi, prvi]);

        let qgs = generiraj_qgs(p, "karta.gpkg");
        let (d, pr) = (id_sloja(p, drugi), id_sloja(p, prvi));
        for oznaka in ["<item>", "<layer id=\""] {
            let mjesto = |id: &str| qgs.find(&format!("{oznaka}{id}")).unwrap();
            assert!(mjesto(&d) < mjesto(&pr), "{oznaka}");
            assert!(mjesto(&pr) < mjesto("osm_podloga"), "{oznaka}");
        }
    }

    #[test]
    fn opseg_projekta_prazan_i_s_geometrijom() {
        let u_prazan = slojevi::Uredjivac::novi();
        let opseg_zadano = opseg_projekta(u_prazan.projekt());
        assert_eq!(
            opseg_zadano,
            (200_000.0, 5_000_000.0, 750_000.0, 5_150_000.0)
        );

        let mut u = slojevi::Uredjivac::novi();
        let s = match u
            .izvrsi(slojevi::NaredbaStabla::DodajSloj {
                u: None,
                naziv: "Poligoni".into(),
                vrsta: slojevi::VrstaGeometrije::Poligon,
            })
            .unwrap()
        {
            Some(slojevi::Novo::Sloj(id)) => id,
            _ => panic!("nema sloja"),
        };
        u.izvrsi(slojevi::NaredbaObjekta::Dodaj {
            sloj: s,
            naziv: "P1".into(),
            geometrija: slojevi::Geometrija::Poligon(vec![vec![
                slojevi::Vrh::novi(16.0, 45.0),
                slojevi::Vrh::novi(16.5, 45.5),
                slojevi::Vrh::novi(16.0, 45.5),
            ]]),
        })
        .unwrap();

        let opseg = opseg_projekta(u.projekt());
        assert!(opseg.0 < opseg.2);
        assert!(opseg.1 < opseg.3);
    }
}
