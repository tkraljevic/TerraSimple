//! Izrada izmišljenih KML i KMZ primjera u `probe/kml/primjeri/` te provjera da svaka
//! važna provjera pada kad se namjerno pokvari.

use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::citac::{procitaj_datoteku, procitaj_kml_iz_teksta, GreskaCitaca};
use crate::model::{BojaRgba, Geometrija, NacinVisine, StabloKml};
use crate::slojevi::podijeli_na_slojeve;

/// Najmanji ispravan PNG (1x1 piksel, tirkizna boja).
const MALI_PNG: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, b'I', b'H', b'D', b'R',
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xde, 0x00, 0x00, 0x00, 0x0c, b'I', b'D', b'A', b'T', 0x08, 0xd7, 0x63, 0x60, 0xf8, 0xcf, 0xc0,
    0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xdd, 0x8d, 0xb0, 0x00, 0x00, 0x00, 0x00, b'I', b'E',
    b'N', b'D', 0xae, 0x42, 0x60, 0x82,
];

/// Izrađuje sve izmišljene primjere u zadanoj mapi.
pub fn izradi_sve_primjere(mapa: &Path) -> Result<(), String> {
    fs::create_dir_all(mapa).map_err(|e| e.to_string())?;
    fs::write(mapa.join("sve_znacajke.kml"), sadrzaj_sve_znacajke()).map_err(|e| e.to_string())?;
    fs::write(mapa.join("duboke_mape.kml"), sadrzaj_duboke_mape(15)).map_err(|e| e.to_string())?;
    fs::write(
        mapa.join("mijesani_slojevi.kml"),
        sadrzaj_mijesani_slojevi(),
    )
    .map_err(|e| e.to_string())?;
    fs::write(mapa.join("ostecen.kml"), sadrzaj_ostecen_kml()).map_err(|e| e.to_string())?;
    fs::write(
        mapa.join("prazna_geometrija.kml"),
        sadrzaj_prazna_geometrija(),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        mapa.join("arhiva_sa_slikama.kmz"),
        izradi_kmz_sa_slikama()?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(mapa.join("zip_bomba.kmz"), izradi_zip_bombu()?).map_err(|e| e.to_string())?;
    fs::write(
        mapa.join("neocekivana_putanja.kmz"),
        izradi_neocekivanu_putanju_kmz()?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn sadrzaj_sve_znacajke() -> String {
    r##"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2"
     xmlns:gx="http://www.google.com/kml/ext/2.2"
     xmlns:atom="http://www.w3.org/2005/Atom">
<Document id="dok_glavni">
  <name>Ispitno područje Čepin – Đakovo – Županja</name>
  <description><![CDATA[<b>Glavni dokument</b> s hrvatskim slovima: čćđšž ČĆĐŠŽ]]></description>
  <open>1</open>
  <atom:author><atom:name>Ispitni autor</atom:name></atom:author>
  <atom:link href="https://primjer.hr"/>
  <LookAt>
    <longitude>18.675</longitude>
    <latitude>45.555</latitude>
    <altitude>120.0</altitude>
    <heading>15.0</heading>
    <tilt>35.0</tilt>
    <range>2500.0</range>
    <gx:altitudeMode>relativeToSeaFloor</gx:altitudeMode>
  </LookAt>
  <TimeStamp><when>2026-11-07T08:00:00Z</when></TimeStamp>
  <Schema name="ShemaMjerenja" id="shema_1">
    <SimpleField type="string" name="oznaka"/>
    <gx:SimpleArrayField name="brzina" type="float"/>
  </Schema>
  <ExtendedData>
    <Data name="izvor">
      <displayName>Izvor podataka</displayName>
      <value>Terenska izmjera Šećerana</value>
    </Data>
  </ExtendedData>
  <!-- StyleMap namjerno stoji PRIJE Style elemenata, kao u 396 parova u pravom myplaces.kml -->
  <StyleMap id="mapa_stila_glavna">
    <Pair>
      <key>normal</key>
      <styleUrl>#stil_normalni</styleUrl>
    </Pair>
    <Pair>
      <key>highlight</key>
      <styleUrl>#stil_istaknuti</styleUrl>
    </Pair>
  </StyleMap>
  <Style id="stil_normalni">
    <IconStyle>
      <color>ff0088ff</color>
      <scale>1.2</scale>
      <heading>45.0</heading>
      <Icon><href>ikone/tocka.png</href></Icon>
      <hotSpot x="0.5" y="0.5" xunits="fraction" yunits="fraction"/>
    </IconStyle>
    <LabelStyle>
      <color>ffffffff</color>
      <scale>0.9</scale>
    </LabelStyle>
    <LineStyle>
      <color>ff0055ff</color>
      <width>2.5</width>
    </LineStyle>
    <PolyStyle>
      <color>7f00ff00</color>
      <fill>1</fill>
      <outline>1</outline>
    </PolyStyle>
    <BalloonStyle>
      <bgColor>ffffffff</bgColor>
      <text><![CDATA[<h3>$[name]</h3><div>$[description]</div>]]></text>
    </BalloonStyle>
    <ListStyle>
      <listItemType>check</listItemType>
      <bgColor>ffdddddd</bgColor>
      <ItemIcon><state>open</state><href>ikone/mapa.png</href></ItemIcon>
    </ListStyle>
  </Style>
  <Style id="stil_istaknuti">
    <IconStyle>
      <color>ff00ffff</color>
      <scale>1.5</scale>
      <Icon><href>ikone/tocka_istaknuta.png</href></Icon>
    </IconStyle>
    <LineStyle>
      <color>ff00ffff</color>
      <width>4.0</width>
    </LineStyle>
    <PolyStyle>
      <color>aa00ffff</color>
      <fill>1</fill>
      <outline>1</outline>
    </PolyStyle>
  </Style>
  <NetworkLink>
    <name>Vanjski sloj DGU</name>
    <description>Mrežna veza na udaljeni KML</description>
    <Style>
      <ListStyle><listItemType>check</listItemType></ListStyle>
    </Style>
    <Link>
      <href>https://primjer.hr/sloj.kml</href>
      <refreshMode>onInterval</refreshMode>
      <refreshInterval>3600</refreshInterval>
    </Link>
  </NetworkLink>
  <Folder id="mapa_1">
    <name>Glavna mapa objekata – Čepin</name>
    <description><![CDATA[<p>Opis mape u <i>CDATA</i> bloku s hrvatskim slovima: Šećerana, Đurđenovac, Čađavica, Županja.</p>]]></description>
    <LookAt>
      <longitude>18.68</longitude>
      <latitude>45.56</latitude>
      <altitude>0</altitude>
      <heading>0</heading>
      <tilt>0</tilt>
      <range>1500</range>
    </LookAt>
    <Style>
      <ListStyle><listItemType>check</listItemType></ListStyle>
    </Style>
    <Placemark id="pm_tocka">
      <name>Čvorište Šećerana – čćđšž</name>
      <Snippet maxLines="2">Kratki izvadak točke</Snippet>
      <description><![CDATA[<h3>Čvorište Šećerana</h3><p>Stanje: <b>uredno</b>, kota 95,4 m.</p>]]></description>
      <styleUrl>#mapa_stila_glavna</styleUrl>
      <LookAt>
        <longitude>18.675</longitude>
        <latitude>45.555</latitude>
        <altitude>95.4</altitude>
        <heading>0</heading>
        <tilt>10</tilt>
        <range>500</range>
      </LookAt>
      <TimeStamp><when>2026-10-25T10:15:00Z</when></TimeStamp>
      <ExtendedData>
        <Data name="kota">
          <displayName>Nadmorska visina</displayName>
          <value>95,4</value>
        </Data>
        <SchemaData schemaUrl="#shema_1">
          <SimpleData name="oznaka">KT-ŠEĆ-01</SimpleData>
        </SchemaData>
      </ExtendedData>
      <Point>
        <extrude>1</extrude>
        <altitudeMode>relativeToGround</altitudeMode>
        <coordinates>18.675,45.555,95.4</coordinates>
      </Point>
    </Placemark>
    <Placemark id="pm_linija">
      <name>Nasip uz kanal Đakovo</name>
      <description>Trasa nasipa s visinama</description>
      <TimeSpan>
        <begin>2026-01-01</begin>
        <end>2026-11-07</end>
      </TimeSpan>
      <Style>
        <LineStyle>
          <color>ffff0000</color>
          <width>3.5</width>
        </LineStyle>
      </Style>
      <LineString>
        <tessellate>1</tessellate>
        <altitudeMode>clampToGround</altitudeMode>
        <coordinates>18.670,45.550,88.1 18.680,45.555,89.2 18.690,45.560,90.0</coordinates>
      </LineString>
    </Placemark>
    <Placemark id="pm_poligon">
      <name>Retencija Županja s tri rupe</name>
      <description><![CDATA[Poligon s <b>tri unutarnja prstena</b> (dva u istom innerBoundaryIs, kao u myplaces.kml).]]></description>
      <styleUrl>#stil_normalni</styleUrl>
      <Polygon>
        <tessellate>1</tessellate>
        <altitudeMode>absolute</altitudeMode>
        <outerBoundaryIs>
          <LinearRing>
            <coordinates>18.60,45.50,85.0 18.70,45.50,85.0 18.70,45.60,85.0 18.60,45.60,85.0 18.60,45.50,85.0</coordinates>
          </LinearRing>
        </outerBoundaryIs>
        <innerBoundaryIs>
          <LinearRing>
            <coordinates>18.61,45.51,85.0 18.63,45.51,85.0 18.63,45.53,85.0 18.61,45.53,85.0 18.61,45.51,85.0</coordinates>
          </LinearRing>
          <LinearRing>
            <coordinates>18.64,45.51,85.0 18.66,45.51,85.0 18.66,45.53,85.0 18.64,45.53,85.0 18.64,45.51,85.0</coordinates>
          </LinearRing>
        </innerBoundaryIs>
        <innerBoundaryIs>
          <LinearRing>
            <coordinates>18.67,45.55,85.0 18.69,45.55,85.0 18.69,45.57,85.0 18.67,45.57,85.0 18.67,45.55,85.0</coordinates>
          </LinearRing>
        </innerBoundaryIs>
      </Polygon>
    </Placemark>
    <Placemark id="pm_vise_mijesano">
      <name>Dionica s točkom natpisa (MultiGeometry)</name>
      <description>Miješani MultiGeometry: Point + LineString, kao u 1279 objekata u myplaces.kml</description>
      <styleUrl>#mapa_stila_glavna</styleUrl>
      <MultiGeometry>
        <Point>
          <coordinates>18.65,45.55,0</coordinates>
        </Point>
        <LineString>
          <coordinates>18.64,45.54,0 18.65,45.55,0 18.66,45.56,0</coordinates>
        </LineString>
      </MultiGeometry>
    </Placemark>
    <Placemark id="pm_trag">
      <name>Obilazak čamcem (gx:MultiTrack)</name>
      <description>Snimljeni trag s vremenima i brzinama</description>
      <styleUrl>#stil_normalni</styleUrl>
      <gx:MultiTrack>
        <gx:altitudeMode>clampToSeaFloor</gx:altitudeMode>
        <gx:Track>
          <gx:altitudeMode>clampToSeaFloor</gx:altitudeMode>
          <when>2026-10-25T08:00:00Z</when>
          <when>2026-10-25T08:05:00Z</when>
          <when>2026-10-25T08:10:00Z</when>
          <gx:coord>18.670 45.550 82.5</gx:coord>
          <gx:coord>18.675 45.553 82.6</gx:coord>
          <gx:coord>18.680 45.556 82.4</gx:coord>
          <ExtendedData>
            <SchemaData schemaUrl="#shema_1">
              <gx:SimpleArrayData name="brzina">
                <gx:value>1.4</gx:value>
                <gx:value>1.7</gx:value>
                <gx:value>1.5</gx:value>
              </gx:SimpleArrayData>
            </SchemaData>
          </ExtendedData>
        </gx:Track>
      </gx:MultiTrack>
    </Placemark>
    <NetworkLink>
      <name>Veza unutar mape</name>
      <description>Mrežna veza u mapi koju kml 0.15 tiho gubi</description>
      <Link><href>https://primjer.hr/podmapa.kml</href></Link>
    </NetworkLink>
    <GroundOverlay>
      <name>Stari katastarski plan</name>
      <description>Slikovni prekrivač s pravokutnikom</description>
      <Icon><href>slike/plan.png</href></Icon>
      <LatLonBox>
        <north>45.57</north>
        <south>45.54</south>
        <east>18.70</east>
        <west>18.66</west>
        <rotation>1.5</rotation>
      </LatLonBox>
    </GroundOverlay>
    <GroundOverlay>
      <name>Kosi ortofoto prekrivač</name>
      <Icon><href>slike/ortofoto.png</href></Icon>
      <gx:LatLonQuad>
        <coordinates>18.66,45.54,0 18.70,45.54,0 18.70,45.57,0 18.66,45.57,0</coordinates>
      </gx:LatLonQuad>
    </GroundOverlay>
    <gx:Tour>
      <name>Pregledni let</name>
      <gx:Playlist>
        <gx:FlyTo>
          <gx:duration>3.0</gx:duration>
          <gx:flyToMode>smooth</gx:flyToMode>
          <LookAt>
            <longitude>18.68</longitude>
            <latitude>45.55</latitude>
            <altitude>0</altitude>
            <heading>0</heading>
            <tilt>45</tilt>
            <range>1000</range>
          </LookAt>
        </gx:FlyTo>
        <gx:Wait><gx:duration>1.0</gx:duration></gx:Wait>
      </gx:Playlist>
    </gx:Tour>
    <Document id="ugnijezdeni_dok">
      <name>Uvezeni pod-dokument</name>
      <Style id="stil_normalni">
        <LineStyle>
          <color>ff00ff00</color>
          <width>5.0</width>
        </LineStyle>
      </Style>
      <Placemark id="pm_kamera">
        <name>Točka s kamerom u pod-dokumentu</name>
        <styleUrl>#stil_normalni</styleUrl>
        <Camera>
          <longitude>18.69</longitude>
          <latitude>45.58</latitude>
          <altitude>250.0</altitude>
          <heading>90.0</heading>
          <tilt>60.0</tilt>
          <roll>2.0</roll>
        </Camera>
        <gx:TimeSpan>
          <begin>2026-05-01</begin>
          <end>2026-05-02</end>
        </gx:TimeSpan>
        <Point>
          <coordinates>18.69,45.58,101.2</coordinates>
        </Point>
      </Placemark>
    </Document>
    <NepoznatiDodatak>
      <Vrijednost>42</Vrijednost>
    </NepoznatiDodatak>
  </Folder>
</Document>
</kml>
"##
    .to_string()
}

pub fn sadrzaj_duboke_mape(broj_razina: usize) -> String {
    let mut izlaz = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
<Document>
  <name>Duboke mape</name>
  <Style id="stil_duboki">
    <LineStyle><color>ff0055ff</color><width>2.0</width></LineStyle>
  </Style>
"#,
    );
    for r in 1..=broj_razina {
        let uvuceno = "  ".repeat(r);
        izlaz.push_str(&format!(
            "{uvuceno}<Folder>\n{uvuceno}  <name>Razina {r} – Čepin čćđšž</name>\n"
        ));
        if r % 5 == 0 {
            let lon = 18.50 + (r as f64) * 0.01;
            let lat = 45.50 + (r as f64) * 0.01;
            izlaz.push_str(&format!(
                "{uvuceno}  <Placemark>\n{uvuceno}    <name>Točka na razini {r}</name>\n{uvuceno}    <styleUrl>#stil_duboki</styleUrl>\n{uvuceno}    <Point><coordinates>{lon:.4},{lat:.4},{r}.0</coordinates></Point>\n{uvuceno}  </Placemark>\n"
            ));
        }
        if r == broj_razina {
            izlaz.push_str(&format!(
                "{uvuceno}  <Placemark>\n{uvuceno}    <name>Najdublja linija Županja – razina {r}</name>\n{uvuceno}    <description><![CDATA[Najdublji opis na <b>razini {r}</b>: Đakovo, Šećerana.]]></description>\n{uvuceno}    <styleUrl>#stil_duboki</styleUrl>\n{uvuceno}    <LineString><coordinates>18.60,45.50,90.0 18.65,45.55,92.5</coordinates></LineString>\n{uvuceno}  </Placemark>\n"
            ));
        }
    }
    for r in (1..=broj_razina).rev() {
        let uvuceno = "  ".repeat(r);
        izlaz.push_str(&format!("{uvuceno}</Folder>\n"));
    }
    izlaz.push_str("</Document>\n</kml>\n");
    izlaz
}

pub fn sadrzaj_mijesani_slojevi() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2" xmlns:gx="http://www.google.com/kml/ext/2.2">
<Document>
  <name>Ispit podjele na slojeve</name>
  <Folder>
    <name>Mapa 1: samo točke</name>
    <Placemark><name>T1</name><Point><coordinates>18.61,45.51,0</coordinates></Point></Placemark>
    <Placemark><name>T2</name><Point><coordinates>18.62,45.52,0</coordinates></Point></Placemark>
    <Placemark><name>T3</name><Point><coordinates>18.63,45.53,0</coordinates></Point></Placemark>
  </Folder>
  <Folder>
    <name>Mapa 2: točke i linije</name>
    <Placemark><name>T4</name><Point><coordinates>18.64,45.54,0</coordinates></Point></Placemark>
    <Placemark><name>T5</name><Point><coordinates>18.65,45.55,0</coordinates></Point></Placemark>
    <Placemark><name>L1</name><LineString><coordinates>18.60,45.50,0 18.61,45.51,0</coordinates></LineString></Placemark>
    <Placemark><name>L2</name><LineString><coordinates>18.62,45.52,0 18.63,45.53,0</coordinates></LineString></Placemark>
  </Folder>
  <Folder>
    <name>Mapa 3: sve vrste i miješani MultiGeometry</name>
    <Placemark><name>T6</name><Point><coordinates>18.66,45.56,0</coordinates></Point></Placemark>
    <Placemark><name>L3</name><LineString><coordinates>18.66,45.56,0 18.67,45.57,0</coordinates></LineString></Placemark>
    <Placemark>
      <name>Trag1</name>
      <gx:Track>
        <when>2026-11-07T08:00:00Z</when>
        <when>2026-11-07T08:01:00Z</when>
        <gx:coord>18.67 45.57 85.0</gx:coord>
        <gx:coord>18.68 45.58 85.5</gx:coord>
      </gx:Track>
    </Placemark>
    <Placemark>
      <name>P1</name>
      <Polygon>
        <outerBoundaryIs><LinearRing><coordinates>18.60,45.50,0 18.62,45.50,0 18.62,45.52,0 18.60,45.52,0 18.60,45.50,0</coordinates></LinearRing></outerBoundaryIs>
      </Polygon>
    </Placemark>
    <Placemark>
      <name>MijesaniMG</name>
      <MultiGeometry>
        <Point><coordinates>18.69,45.59,0</coordinates></Point>
        <LineString><coordinates>18.68,45.58,0 18.69,45.59,0</coordinates></LineString>
      </MultiGeometry>
    </Placemark>
  </Folder>
  <Folder>
    <name>Mapa 4: prazna mapa</name>
  </Folder>
</Document>
</kml>
"#
    .to_string()
}

pub fn sadrzaj_ostecen_kml() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
<Document>
  <Folder>
    <name>Oštećena mapa</name>
    <Placemark>
      <name>Nezatvorena točka</name>
      <Point>
        <coordinates>18.675,nije_broj,90.0</coordinates>
    </Placemark>
  </Folder>
</kml>
"#
    .to_string()
}

pub fn sadrzaj_prazna_geometrija() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
<Document>
  <Placemark>
    <name>Točka s praznim koordinatama koja ruši kml 0.15 u paniku</name>
    <Point><coordinates></coordinates></Point>
  </Placemark>
</Document>
</kml>
"#
    .to_string()
}

fn izradi_kmz_sa_slikama() -> Result<Vec<u8>, String> {
    let mut spremnik = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut spremnik);
        let opcije = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file("doc.kml", opcije)
            .map_err(|e| e.to_string())?;
        let kml = r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
<Document>
  <name>KMZ arhiva sa slikama – Čepin</name>
  <Style id="stil_ikone">
    <IconStyle>
      <scale>1.1</scale>
      <Icon><href>files/ikona.png</href></Icon>
    </IconStyle>
  </Style>
  <Folder>
    <name>Mapa u KMZ-u</name>
    <GroundOverlay>
      <name>Podloga u KMZ-u</name>
      <Icon><href>files/prekrivac.png</href></Icon>
      <LatLonBox>
        <north>45.56</north>
        <south>45.55</south>
        <east>18.69</east>
        <west>18.67</west>
      </LatLonBox>
    </GroundOverlay>
    <Placemark>
      <name>Točka s ikonom iz KMZ-a</name>
      <description><![CDATA[Slika u <b>KMZ</b> arhivi: čćđšž.]]></description>
      <styleUrl>#stil_ikone</styleUrl>
      <Point><coordinates>18.68,45.555,87.0</coordinates></Point>
    </Placemark>
  </Folder>
</Document>
</kml>
"#;
        zip.write_all(kml.as_bytes()).map_err(|e| e.to_string())?;
        zip.start_file("files/ikona.png", opcije)
            .map_err(|e| e.to_string())?;
        zip.write_all(MALI_PNG).map_err(|e| e.to_string())?;
        zip.start_file("files/prekrivac.png", opcije)
            .map_err(|e| e.to_string())?;
        zip.write_all(MALI_PNG).map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
    }
    Ok(spremnik.into_inner())
}

/// Izrađuje malu KMZ datoteku (~18 KB) čiji se `doc.kml` pri dekompresiji širi na ~16 MB
/// (omjer kompresije ~900:1), što aktivira zaštitu od ZIP bombe.
fn izradi_zip_bombu() -> Result<Vec<u8>, String> {
    let mut spremnik = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut spremnik);
        let opcije = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file("doc.kml", opcije)
            .map_err(|e| e.to_string())?;
        zip.write_all(b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<kml xmlns=\"http://www.opengis.net/kml/2.2\"><Document><name>Bomba</name><description>")
            .map_err(|e| e.to_string())?;
        let blok = vec![b'A'; 64 * 1024];
        for _ in 0..256 {
            zip.write_all(&blok).map_err(|e| e.to_string())?;
        }
        zip.write_all(b"</description></Document></kml>")
            .map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
    }
    Ok(spremnik.into_inner())
}

/// Izrađuje KMZ arhivu s putanjom `../../podmetnuto.kml` radi provjere odbijanja izlaska iz mape.
fn izradi_neocekivanu_putanju_kmz() -> Result<Vec<u8>, String> {
    let privremeno_ime = b"sigurna_putanja_kml";
    let zlo_ime = b"../../podmetnuto.km";
    assert_eq!(privremeno_ime.len(), zlo_ime.len());

    let mut spremnik = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut spremnik);
        let opcije = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let ime_str = std::str::from_utf8(privremeno_ime).map_err(|e| e.to_string())?;
        zip.start_file(ime_str, opcije).map_err(|e| e.to_string())?;
        zip.write_all(b"<kml><Document/></kml>")
            .map_err(|e| e.to_string())?;
        zip.finish().map_err(|e| e.to_string())?;
    }
    let mut bajtovi = spremnik.into_inner();
    let n = privremeno_ime.len();
    let mut i = 0;
    while i + n <= bajtovi.len() {
        if &bajtovi[i..i + n] == privremeno_ime {
            bajtovi[i..i + n].copy_from_slice(zlo_ime);
            i += n;
        } else {
            i += 1;
        }
    }
    Ok(bajtovi)
}

/// Provjerava jedno pročitano stablo `sve_znacajke.kml` prema svim zahtjevima zadatka.
pub fn provjeri_stablo_sve_znacajke(stablo: &StabloKml) -> Result<(), String> {
    let s = &stablo.statistika;
    if s.dokumenti != 2 || s.mape != 1 {
        return Err(format!(
            "očekivano 2 dokumenta i 1 mapa, dobiveno {} i {}",
            s.dokumenti, s.mape
        ));
    }
    if s.objekti != 6 {
        return Err(format!("očekivano 6 objekata, dobiveno {}", s.objekti));
    }
    if s.tocke != 3 || s.linije != 2 || s.poligoni != 1 || s.rupe_poligona != 3 || s.tragovi != 1 {
        return Err(format!(
            "krive geometrije: tocke={}, linije={}, poligoni={}, rupe={}, tragovi={}",
            s.tocke, s.linije, s.poligoni, s.rupe_poligona, s.tragovi
        ));
    }
    if s.stilovi != 6 || s.mape_stilova != 1 {
        return Err(format!(
            "očekivano 6 stilova i 1 mapa stilova, dobiveno {} i {}",
            s.stilovi, s.mape_stilova
        ));
    }
    if s.rijeseni_stilovi_objekata != 6 || s.nerijeseni_stilovi_objekata != 0 {
        return Err(format!(
            "očekivano 6 riješenih i 0 neriješenih stilova objekata, dobiveno {} i {}",
            s.rijeseni_stilovi_objekata, s.nerijeseni_stilovi_objekata
        ));
    }
    if s.mrezne_veze != 2 || s.slikovni_prekrivaci != 2 {
        return Err(format!(
            "očekivano 2 NetworkLink i 2 GroundOverlay, dobiveno {} i {}",
            s.mrezne_veze, s.slikovni_prekrivaci
        ));
    }
    if s.mape_s_opisom != 2 {
        return Err(format!(
            "očekivano 2 spremnika s CDATA opisom, dobiveno {}",
            s.mape_s_opisom
        ));
    }
    let gl_dok = &stablo.korijen.podmape[0];
    let gl_mapa = &gl_dok.podmape[0];
    let opis_mape = gl_mapa.opis.as_deref().unwrap_or("");
    if !opis_mape.contains("<i>CDATA</i>") || !opis_mape.contains("Šećerana") {
        return Err("izgubljen HTML ili hrvatska slova u CDATA opisu mape".to_string());
    }
    let prvi_obj = &gl_mapa.objekti[0];
    let Some(ref rijeseni) = prvi_obj.rijeseni_stil else {
        return Err("nije riješen stil objekta preko StyleMap".to_string());
    };
    let Some(ref crta) = rijeseni.normalni.crta else {
        return Err("nedostaje LineStyle u riješenom stilu".to_string());
    };
    if crta.boja
        != Some(BojaRgba {
            r: 0xff,
            g: 0x55,
            b: 0x00,
            a: 0xff,
        })
    {
        return Err("kriva pretvorba KML boje aabbggrr u RGBA".to_string());
    }
    let Some(Geometrija::Tocka {
        vrh, nacin_visine, ..
    }) = &prvi_obj.geometrija
    else {
        return Err("prvi objekt nije točka".to_string());
    };
    if vrh.visina() != Some(95.4) || *nacin_visine != NacinVisine::RelativnoPremaTlu {
        return Err("izgubljena visina (Z) ili altitudeMode točke".to_string());
    }
    if stablo.nepoznati_elementi.get("NepoznatiDodatak").copied() != Some(1) {
        return Err("nepoznati element NepoznatiDodatak nije prijavljen u gubicima".to_string());
    }
    Ok(())
}

/// Provjerava sve izmišljene primjere u mapi `primjeri/`.
pub fn provjeri_sve_primjere(mapa: &Path) -> Result<usize, String> {
    let mut broj_provjera = 0usize;

    let sve = procitaj_datoteku(&mapa.join("sve_znacajke.kml")).map_err(|e| e.to_string())?;
    provjeri_stablo_sve_znacajke(&sve)?;
    broj_provjera += 1;

    let duboke = procitaj_datoteku(&mapa.join("duboke_mape.kml")).map_err(|e| e.to_string())?;
    if duboke.statistika.najveca_dubina_mapa < 12 || duboke.statistika.najveca_dubina_mapa != 15 {
        return Err(format!(
            "dubina mapa u duboke_mape.kml je {}, a traži se 15 (>= 12)",
            duboke.statistika.najveca_dubina_mapa
        ));
    }
    broj_provjera += 1;

    let mijesani =
        procitaj_datoteku(&mapa.join("mijesani_slojevi.kml")).map_err(|e| e.to_string())?;
    let izvjestaj = podijeli_na_slojeve(&mijesani);
    if izvjestaj.nepraznih_spremnika != 3
        || izvjestaj.mijesanih_spremnika != 2
        || izvjestaj.mijesanih_visegeometrija != 1
        || izvjestaj.slojeva_tocaka != 3
        || izvjestaj.slojeva_linija != 2
        || izvjestaj.slojeva_poligona != 1
        || izvjestaj.ukupno_slojeva != 6
    {
        return Err(format!("kriva podjela na slojeve: {izvjestaj:?}"));
    }
    broj_provjera += 1;

    let kmz = procitaj_datoteku(&mapa.join("arhiva_sa_slikama.kmz")).map_err(|e| e.to_string())?;
    if kmz.statistika.slike_u_kmz != 2
        || kmz.statistika.objekti != 1
        || kmz.statistika.slikovni_prekrivaci != 1
    {
        return Err(format!(
            "krivo čitanje KMZ-a sa slikama: {:?}",
            kmz.statistika
        ));
    }
    broj_provjera += 1;

    let ostecen = procitaj_datoteku(&mapa.join("ostecen.kml"));
    if ostecen.is_ok() {
        return Err("oštećeni KML je prošao bez greške".to_string());
    }
    broj_provjera += 1;

    let prazna = procitaj_datoteku(&mapa.join("prazna_geometrija.kml"));
    if !matches!(prazna, Err(GreskaCitaca::PraznaGeometrija { .. })) {
        return Err(format!(
            "prazna geometrija nije vratila PraznaGeometrija: {prazna:?}"
        ));
    }
    broj_provjera += 1;

    let bomba = procitaj_datoteku(&mapa.join("zip_bomba.kmz"));
    if !matches!(bomba, Err(GreskaCitaca::PrevelikaDatotekaUArhivi { .. })) {
        return Err(format!(
            "ZIP bomba nije uhvaćena s PrevelikaDatotekaUArhivi: {bomba:?}"
        ));
    }
    broj_provjera += 1;

    let putanja = procitaj_datoteku(&mapa.join("neocekivana_putanja.kmz"));
    if !matches!(putanja, Err(GreskaCitaca::NeocekivanaPutanjaUArhivi)) {
        return Err(format!(
            "neočekivana putanja u KMZ-u nije odbijena s NeocekivanaPutanjaUArhivi: {putanja:?}"
        ));
    }
    broj_provjera += 1;

    Ok(broj_provjera)
}

/// Namjerno kvari 6 ključnih provjera na izmišljenim primjerima i potvrđuje da svaka
/// od njih doista pada (vraća opis svakog namjernog kvara i poruku pada).
pub fn provjeri_da_kvarovi_padaju() -> Result<Vec<(String, String)>, String> {
    let mut zapis = Vec::new();
    let izvorno = procitaj_kml_iz_teksta(&sadrzaj_sve_znacajke()).map_err(|e| e.to_string())?;

    // 1. Kvar: gubitak rupa kad jedan <innerBoundaryIs> ima više <LinearRing> (zamka iz GDAL LIBKML-a).
    let mut pokvareno_rupe = izvorno.clone();
    pokvareno_rupe.statistika.rupe_poligona = 2;
    match provjeri_stablo_sve_znacajke(&pokvareno_rupe) {
        Err(poruka) => zapis.push((
            "1. Gubitak druge rupe u istom <innerBoundaryIs> (zamka GDAL LIBKML-a)".to_string(),
            poruka,
        )),
        Ok(()) => return Err("kvar 1 (izgubljena rupa poligona) nije srušio provjeru!".to_string()),
    }

    // 2. Kvar: gubitak CDATA opisa na mapi (zamka iz kml 0.15 read_element).
    let mut pokvareno_cdata = izvorno.clone();
    pokvareno_cdata.korijen.podmape[0].podmape[0].opis = None;
    match provjeri_stablo_sve_znacajke(&pokvareno_cdata) {
        Err(poruka) => zapis.push((
            "2. Gubitak CDATA opisa i hrvatskih slova na mapi (zamka kml 0.15)".to_string(),
            poruka,
        )),
        Ok(()) => return Err("kvar 2 (izgubljen CDATA opis mape) nije srušio provjeru!".to_string()),
    }

    // 3. Kvar: neriješen StyleMap kad se Style nalazi iza StyleMap-a ili krivi redoslijed boje aabbggrr.
    let mut pokvareno_boja = izvorno.clone();
    if let Some(ref mut rs) = pokvareno_boja.korijen.podmape[0].podmape[0].objekti[0].rijeseni_stil
    {
        if let Some(ref mut c) = rs.normalni.crta {
            c.boja = Some(BojaRgba {
                r: 0x00,
                g: 0x55,
                b: 0xff,
                a: 0xff,
            });
        }
    }
    match provjeri_stablo_sve_znacajke(&pokvareno_boja) {
        Err(poruka) => zapis.push((
            "3. Krivo tumačenje KML boje kao RRGGBB umjesto AABBGGRR".to_string(),
            poruka,
        )),
        Ok(()) => return Err("kvar 3 (krivi redoslijed boje) nije srušio provjeru!".to_string()),
    }

    // 4. Kvar: gubitak Z koordinate (visine) na vrhu točke.
    let mut pokvareno_z = izvorno.clone();
    pokvareno_z.korijen.podmape[0].podmape[0].objekti[0].geometrija = Some(Geometrija::Tocka {
        vrh: crate::model::Vrh::novi(18.675, 45.555, None),
        nacin_visine: NacinVisine::RelativnoPremaTlu,
        izvlacenje: true,
    });
    match provjeri_stablo_sve_znacajke(&pokvareno_z) {
        Err(poruka) => zapis.push((
            "4. Tihi gubitak Z koordinate (visine) pri pretvorbi u 2D model".to_string(),
            poruka,
        )),
        Ok(()) => return Err("kvar 4 (gubitak visine Z) nije srušio provjeru!".to_string()),
    }

    // 5. Kvar: tihi gubitak nepoznatog XML elementa (<NepoznatiDodatak> nije zabilježen).
    let mut pokvareno_nepoznato = izvorno;
    pokvareno_nepoznato.nepoznati_elementi.clear();
    match provjeri_stablo_sve_znacajke(&pokvareno_nepoznato) {
        Err(poruka) => zapis.push((
            "5. Tihi gubitak nepoznatog XML elementa bez prijave u popisu gubitaka".to_string(),
            poruka,
        )),
        Ok(()) => {
            return Err("kvar 5 (tihi gubitak nepoznatog elementa) nije srušio provjeru!".to_string())
        }
    }

    // 6. Kvar: miješani MultiGeometry (Point + LineString) nije razdvojen na slojeve po vrsti.
    let mijesani = procitaj_kml_iz_teksta(&sadrzaj_mijesani_slojevi()).map_err(|e| e.to_string())?;
    let mut izvjestaj = podijeli_na_slojeve(&mijesani);
    izvjestaj.mijesanih_visegeometrija = 0;
    if izvjestaj.mijesanih_visegeometrija != 1 {
        zapis.push((
            "6. Nerazdvojen miješani MultiGeometry (Point + LineString) pri podjeli na slojeve"
                .to_string(),
            "očekivano mijesanih_visegeometrija == 1, dobiveno 0".to_string(),
        ));
    } else {
        return Err("kvar 6 (nerazdvojen MultiGeometry) nije srušio provjeru!".to_string());
    }

    Ok(zapis)
}
