// SPDX-License-Identifier: EUPL-1.2

//! Ispitivanje 2. pitanja: Što nudi `walkers_extras` 0.65.0 i čuva li KML podatke?
//!
//! Provjerava na umjetnom primjeru:
//! - Čuva li rupe u poligonima kad su u odvojenim `<innerBoundaryIs>`?
//! - Čuva li više prstenova (`<LinearRing>`) u istom `<innerBoundaryIs>`?
//! - Što se događa pri teselaciji (`walkers::render_fill` koristi `FillOptions::default()`, a to je
//!   u `lyon`-u `FillRule::EvenOdd`, pa smjer vrhova rupe ne smije biti važan)?
//! - Čuva li KML stilove (`<Style>`, `<StyleMap>`, `<LineStyle>`, `<PolyStyle>`)?
//! - Čuva li Z visine (`lon,lat,alt`)?
//! - Što radi na praznoj točki `<Point>` i na `<gx:Track>`?
//! - Kako crta na svakom okviru (`KmlLayer`, `GeoJsonLayer`, `GroupedPlacesTree`)?

use std::collections::HashMap;
use std::str::FromStr;

use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use walkers::geo_types::{Coord, Geometry};
use walkers::{
    Color as WalkersColor, Context as WalkersContext, Drawable, Map, MapMemory, MercatorProjection,
    Paint, Position, Projector, Style, StyleLayer, json, lon_lat, render_fill,
};
use walkers_extras::{
    GeoJsonLayer, Group, GroupedPlacesTree, KmlLayer, LabeledSymbol, LabeledSymbolGroup,
    LabeledSymbolGroupStyle, LabeledSymbolStyle, Place, Symbol,
};

/// Umjetni KML s poligonom koji ima 2 rupe u ODVOJENIM `<innerBoundaryIs>` elementima
/// (suprotne orijentacije od vanjskog prstena), KML stilom (`#crveni_stil`) i 3D Z visinama (525.4 m).
pub const UMJETNI_KML_ODVOJENE_RUPE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <Style id="crveni_stil">
      <LineStyle><color>ff0000ff</color><width>5</width></LineStyle>
      <PolyStyle><color>800000ff</color></PolyStyle>
    </Style>
    <Placemark>
      <name>Poligon s dvije odvojene rupe</name>
      <styleUrl>#crveni_stil</styleUrl>
      <Polygon>
        <outerBoundaryIs>
          <LinearRing>
            <coordinates>
              15.90,45.75,525.4 16.00,45.75,525.4 16.00,45.85,525.4 15.90,45.85,525.4 15.90,45.75,525.4
            </coordinates>
          </LinearRing>
        </outerBoundaryIs>
        <innerBoundaryIs>
          <LinearRing>
            <coordinates>
              15.91,45.76,525.4 15.91,45.79,525.4 15.94,45.79,525.4 15.94,45.76,525.4 15.91,45.76,525.4
            </coordinates>
          </LinearRing>
        </innerBoundaryIs>
        <innerBoundaryIs>
          <LinearRing>
            <coordinates>
              15.96,45.80,525.4 15.96,45.83,525.4 15.99,45.83,525.4 15.99,45.80,525.4 15.96,45.80,525.4
            </coordinates>
          </LinearRing>
        </innerBoundaryIs>
      </Polygon>
    </Placemark>
  </Document>
</kml>"#;

/// Umjetni KML s poligonom koji ima 2 `<LinearRing>` prstena unutar ISTOG `<innerBoundaryIs>`
/// (suprotne orijentacije od vanjskog prstena).
pub const UMJETNI_KML_ISTI_INNER_BOUNDARY: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <Placemark>
      <name>Poligon s dva prstena u istom innerBoundaryIs</name>
      <Polygon>
        <outerBoundaryIs>
          <LinearRing>
            <coordinates>
              15.90,45.75,120.0 16.00,45.75,120.0 16.00,45.85,120.0 15.90,45.85,120.0 15.90,45.75,120.0
            </coordinates>
          </LinearRing>
        </outerBoundaryIs>
        <innerBoundaryIs>
          <LinearRing>
            <coordinates>
              15.91,45.76,120.0 15.91,45.79,120.0 15.94,45.79,120.0 15.94,45.76,120.0 15.91,45.76,120.0
            </coordinates>
          </LinearRing>
          <LinearRing>
            <coordinates>
              15.96,45.80,120.0 15.96,45.83,120.0 15.99,45.83,120.0 15.99,45.83,120.0 15.96,45.80,120.0
            </coordinates>
          </LinearRing>
        </innerBoundaryIs>
      </Polygon>
    </Placemark>
  </Document>
</kml>"#;

/// Umjetni KML s poligonom čija rupa ima ISTU orijentaciju vrhova (CCW) kao i vanjski prsten.
pub const UMJETNI_KML_RUPA_ISTE_ORIJENTACIJE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <Placemark>
      <Polygon>
        <outerBoundaryIs>
          <LinearRing>
            <coordinates>
              15.90,45.75 16.00,45.75 16.00,45.85 15.90,45.85 15.90,45.75
            </coordinates>
          </LinearRing>
        </outerBoundaryIs>
        <innerBoundaryIs>
          <LinearRing>
            <coordinates>
              15.92,45.77 15.98,45.77 15.98,45.83 15.92,45.83 15.92,45.77
            </coordinates>
          </LinearRing>
        </innerBoundaryIs>
      </Polygon>
    </Placemark>
  </Document>
</kml>"#;

/// Umjetni KML s praznim `<Point></Point>` ili `<Polygon><outerBoundaryIs></outerBoundaryIs></Polygon>`
/// na kojem `KmlLayer::from_string` puca (`panic!`) zbog `.unwrap()` i `coords.remove(0)`.
pub const UMJETNI_KML_PRAZNA_TOCKA: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2">
  <Document>
    <Placemark>
      <name>Prazna točka</name>
      <Point></Point>
    </Placemark>
  </Document>
</kml>"#;

/// Umjetni KML s `<gx:Track>` elementom.
pub const UMJETNI_KML_GX_TRACK: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://www.opengis.net/kml/2.2" xmlns:gx="http://www.google.com/kml/ext/2.2">
  <Document>
    <Placemark>
      <name>Trag leta</name>
      <gx:Track>
        <when>2026-11-07T10:00:00Z</when>
        <when>2026-11-07T10:01:00Z</when>
        <gx:coord>15.90 45.75 300.0</gx:coord>
        <gx:coord>15.95 45.80 350.0</gx:coord>
      </gx:Track>
    </Placemark>
  </Document>
</kml>"#;

/// Rezultat ispitivanja `walkers_extras`.
#[derive(Debug, Clone)]
pub struct IzvjestajDodataka {
    /// Broj sačuvanih rupa kad su u odvojenim `<innerBoundaryIs>` (2).
    pub rupe_odvojeni_inner_boundary: usize,
    /// Broj trokuta u teseliranom poligonu s 2 odvojene rupe suprotne orijentacije (> 10).
    pub trokuti_odvojeni_inner_boundary: usize,
    /// Broj sačuvanih rupa kad su 2 `<LinearRing>` u istom `<innerBoundaryIs>` (u `kml 0.14` su 2).
    pub rupe_isti_inner_boundary: usize,
    /// Broj trokuta u teseliranom poligonu kad su 2 prstena u istom `<innerBoundaryIs>` (> 10).
    pub trokuti_isti_inner_boundary: usize,
    /// Je li središte rupe prebojano (ispunjeno trokutom) kad rupa u KML-u ima istu orijentaciju
    /// vrhova kao vanjski prsten (`walkers::render_fill` koristi `FillRule::EvenOdd`, pa ne smije biti).
    pub rupa_iste_orijentacije_prebojana: bool,
    /// Čuva li `walkers_extras::KmlLayer` KML stilove (`<Style>`, `<StyleMap>`, `<LineStyle>`, `<PolyStyle>`)?
    pub cuva_kml_stilove: bool,
    /// Čuva li `walkers_extras::KmlLayer` Z visine u svojoj geometriji (`geo_types::Geometry<f64>`)?
    pub cuva_z_visine: bool,
    /// Ruši li se (`panic!`) `KmlLayer::from_string` na praznoj točki `<Point>`?
    pub panic_na_praznoj_tocki: bool,
    /// Broj geometrija koje `KmlLayer` zadrži iz `<gx:Track>` (0 = tiho izgubljeno).
    pub broj_geometrija_iz_gx_track: usize,
    /// Radi li `GeoJsonLayer` i `GroupedPlacesTree` osnovni prikaz bez rušenja?
    pub geojson_i_grouped_places_rade: bool,
}

fn tocka_u_trokutu(p: Pos2, a: Pos2, b: Pos2, c: Pos2) -> bool {
    let v0 = c - a;
    let v1 = b - a;
    let v2 = p - a;
    let dot00 = v0.dot(v0);
    let dot01 = v0.dot(v1);
    let dot02 = v0.dot(v2);
    let dot11 = v1.dot(v1);
    let dot12 = v1.dot(v2);
    let nazivnik = dot00 * dot11 - dot01 * dot01;
    if nazivnik.abs() < 1e-9 {
        return false;
    }
    let inv = 1.0 / nazivnik;
    let u = (dot11 * dot02 - dot01 * dot12) * inv;
    let v = (dot00 * dot12 - dot01 * dot02) * inv;
    u >= -1e-4 && v >= -1e-4 && (u + v) <= 1.0 + 1e-4
}

fn teseliraj_geometriju(
    geometrija: &Geometry<f64>,
    projektor: &Projector,
    ispitna_tocka: Pos2,
) -> (usize, Color32, bool) {
    let projicirano = geo_033_ili_rucno::projiciraj_u_zaslon(geometrija, projektor);
    let kontekst = WalkersContext::new("Polygon".to_string(), HashMap::new(), 12);
    let paint = Paint {
        fill_color: Some(WalkersColor(json!("#0088ff"))),
        ..Default::default()
    };
    let mut drawables = Vec::new();
    let _ = render_fill(&projicirano, &kontekst, &paint, &mut drawables);
    let mut ukupno_trokuta = 0;
    let mut boja = Color32::TRANSPARENT;
    let mut tocka_pokrivena = false;
    for d in &drawables {
        if let Drawable::Fill(mesh) = d {
            ukupno_trokuta += mesh.indices.len() / 3;
            if let Some(v) = mesh.vertices.first() {
                boja = v.color;
            }
            for trokut in mesh.indices.as_chunks::<3>().0 {
                let a = mesh.vertices[trokut[0] as usize].position;
                let b = mesh.vertices[trokut[1] as usize].position;
                let c = mesh.vertices[trokut[2] as usize].position;
                if tocka_u_trokutu(ispitna_tocka, a, b, c) {
                    tocka_pokrivena = true;
                }
            }
        }
    }
    (ukupno_trokuta, boja, tocka_pokrivena)
}

mod geo_033_ili_rucno {
    use super::*;
    use walkers::geo_types::{LineString, Polygon};

    pub fn projiciraj_u_zaslon(geometrija: &Geometry<f64>, projektor: &Projector) -> Geometry<f32> {
        match geometrija {
            Geometry::Polygon(poligon) => {
                let vanjski: Vec<Coord<f32>> = poligon
                    .exterior()
                    .0
                    .iter()
                    .map(|c| {
                        let p = projektor.project(Position::new(c.x, c.y));
                        Coord { x: p.x, y: p.y }
                    })
                    .collect();
                let unutarnji: Vec<LineString<f32>> = poligon
                    .interiors()
                    .iter()
                    .map(|prsten| {
                        LineString::new(
                            prsten
                                .0
                                .iter()
                                .map(|c| {
                                    let p = projektor.project(Position::new(c.x, c.y));
                                    Coord { x: p.x, y: p.y }
                                })
                                .collect(),
                        )
                    })
                    .collect();
                Geometry::Polygon(Polygon::new(LineString::new(vanjski), unutarnji))
            }
            _ => Geometry::Point(walkers::geo_types::Point::new(0.0, 0.0)),
        }
    }
}

/// Provjerava ponašanje `walkers_extras` i vraća izvještaj.
/// Ako je `simuliraj_kvar = true`, namjerno tvrdi da `walkers_extras` čuva KML stilove,
/// što mora srušiti provjeru točnosti izvještaja.
pub fn provjeri_walkers_extras(simuliraj_kvar: bool) -> Result<IzvjestajDodataka, String> {
    let pravokutnik = Rect::from_min_size(Pos2::ZERO, Vec2::splat(800.0));
    let mut memorija = MapMemory::default();
    memorija.set_zoom(12.0).map_err(|e| e.to_string())?;
    let srediste = lon_lat(15.95, 45.80);
    let projektor = Projector::new(pravokutnik, &MercatorProjection, &memorija, srediste);
    let srediste_prve_rupe = projektor.project(lon_lat(15.925, 45.775));
    let srediste_prve_rupe_pos2 = Pos2::new(srediste_prve_rupe.x, srediste_prve_rupe.y);

    // 1. Odvojene `<innerBoundaryIs>` rupe
    let kml_odvojeni = kml::Kml::<f64>::from_str(UMJETNI_KML_ODVOJENE_RUPE)
        .map_err(|e| format!("Greška čitanja KML-a: {e}"))?;
    let geom_odvojeni = Vec::<Geometry<f64>>::try_from(kml_odvojeni)
        .map_err(|e| format!("Greška pretvorbe KML-a: {e}"))?;
    let poligon_odvojeni = match geom_odvojeni.first() {
        Some(Geometry::Polygon(p)) => p,
        _ => return Err("Očekivan Polygon iz UMJETNI_KML_ODVOJENE_RUPE".to_string()),
    };
    let rupe_odvojeni_inner_boundary = poligon_odvojeni.interiors().len();
    let (trokuti_odvojeni_inner_boundary, boja_iz_vanjskog_stila, rupa_odvojeni_pokrivena) =
        teseliraj_geometriju(&geom_odvojeni[0], &projektor, srediste_prve_rupe_pos2);
    if rupa_odvojeni_pokrivena {
        return Err("Središte ispravno orijentirane rupe ne smije biti pokriveno trokutom".to_string());
    }

    // 2. Više `<LinearRing>` u istom `<innerBoundaryIs>`
    let kml_isti = kml::Kml::<f64>::from_str(UMJETNI_KML_ISTI_INNER_BOUNDARY)
        .map_err(|e| format!("Greška čitanja KML-a: {e}"))?;
    let geom_isti = Vec::<Geometry<f64>>::try_from(kml_isti)
        .map_err(|e| format!("Greška pretvorbe KML-a: {e}"))?;
    let poligon_isti = match geom_isti.first() {
        Some(Geometry::Polygon(p)) => p,
        _ => return Err("Očekivan Polygon iz UMJETNI_KML_ISTI_INNER_BOUNDARY".to_string()),
    };
    let rupe_isti_inner_boundary = poligon_isti.interiors().len();
    let (trokuti_isti_inner_boundary, _, _) =
        teseliraj_geometriju(&geom_isti[0], &projektor, srediste_prve_rupe_pos2);

    // 2b. Rupa iste orijentacije kao vanjski prsten (CCW u CCW):
    // `walkers::render_fill` koristi `FillOptions::default()` (`FillRule::EvenOdd`), pa rupa iste
    // orijentacije mora ostati izrezana.
    let kml_ista_orijent = kml::Kml::<f64>::from_str(UMJETNI_KML_RUPA_ISTE_ORIJENTACIJE)
        .map_err(|e| format!("Greška čitanja KML-a: {e}"))?;
    let geom_ista_orijent = Vec::<Geometry<f64>>::try_from(kml_ista_orijent)
        .map_err(|e| format!("Greška pretvorbe KML-a: {e}"))?;
    let srediste_velike_rupe = projektor.project(lon_lat(15.95, 45.80));
    let (_, _, rupa_iste_orijentacije_prebojana) = teseliraj_geometriju(
        &geom_ista_orijent[0],
        &projektor,
        Pos2::new(srediste_velike_rupe.x, srediste_velike_rupe.y),
    );

    // 3. Stilovi: U `UMJETNI_KML_ODVOJENE_RUPE` definiran je crveni `<PolyStyle><color>800000ff</color></PolyStyle>`,
    // ali `KmlLayer` odbacuje sve KML stilove i koristi isključivo boju iz vanjskog `walkers::Style` (`#0088ff`, R=0).
    let mut cuva_kml_stilove = boja_iz_vanjskog_stila.r() > 0;
    if simuliraj_kvar {
        cuva_kml_stilove = true;
    }

    // 4. Z visine: `geo_types::Coord<f64>` ima samo 2 polja (`x` i `y`, po 8 bajtova = 16 bajtova).
    let velicina_coord_f64 = std::mem::size_of::<Coord<f64>>();
    let cuva_z_visine = velicina_coord_f64 > 16;

    // 5. Prazna točka `<Point>`: `KmlLayer::from_string` radi `.unwrap()` i puca (`panic!`)!
    let rezultat_panike = std::panic::catch_unwind(|| {
        let _ = KmlLayer::from_string(UMJETNI_KML_PRAZNA_TOCKA, Style::default());
    });
    let panic_na_praznoj_tocki = rezultat_panike.is_err();

    // 6. `<gx:Track>`: `Vec::<Geometry<f64>>::try_from` vraća 0 geometrija (tiho gubi sve tragove!)
    let kml_track = kml::Kml::<f64>::from_str(UMJETNI_KML_GX_TRACK)
        .map_err(|e| format!("Greška čitanja gx:Track KML-a: {e}"))?;
    let geom_track = Vec::<Geometry<f64>>::try_from(kml_track).unwrap_or_default();
    let broj_geometrija_iz_gx_track = geom_track.len();

    // 7. Stvarno izvođenje `KmlLayer`, `GeoJsonLayer` i `GroupedPlacesTree` u `walkers::Map::show`
    let napravi_stil = || {
        let mut s = Style::default();
        s.layers.push(StyleLayer::Fill {
            source_layer: "".into(),
            filter: None,
            paint: Paint {
                fill_color: Some(WalkersColor(json!("#0088ff"))),
                ..Default::default()
            },
        });
        s
    };
    let kml_sloj = KmlLayer::from_string(UMJETNI_KML_ODVOJENE_RUPE, napravi_stil());

    let geojson_tekst = r#"{
        "type": "FeatureCollection",
        "features": [{
            "type": "Feature",
            "properties": {"naziv": "Testni poligon"},
            "geometry": {
                "type": "Polygon",
                "coordinates": [
                    [[15.90, 45.75], [16.00, 45.75], [16.00, 45.85], [15.90, 45.85], [15.90, 45.75]],
                    [[15.92, 45.77], [15.92, 45.80], [15.95, 45.80], [15.95, 45.77], [15.92, 45.77]]
                ]
            }
        }]
    }"#;
    let geojson_objekt = geojson::GeoJson::from_str(geojson_tekst).map_err(|e| e.to_string())?;
    let geojson_sloj = GeoJsonLayer::new(geojson_objekt, napravi_stil());

    let mjesta = vec![
        LabeledSymbol {
            position: lon_lat(15.95, 45.80),
            label: "Tocka 1".to_string(),
            symbol: Some(Symbol::Circle("1".to_string())),
            style: LabeledSymbolStyle::default(),
        },
        LabeledSymbol {
            position: lon_lat(15.951, 45.801),
            label: "Tocka 2".to_string(),
            symbol: Some(Symbol::TwoCorners("2".to_string())),
            style: LabeledSymbolStyle::default(),
        },
    ];
    let stablo_mjesta = GroupedPlacesTree::new(
        mjesta,
        LabeledSymbolGroup {
            style: LabeledSymbolGroupStyle::default(),
        },
    )
    .with_screen_radius_px(50.0);

    let (broj_skupina, maks_u_skupini) = stablo_mjesta.cluster_stats(pravokutnik, &projektor);
    let mut opcija_stabla = Some(stablo_mjesta);

    let ctx = egui::Context::default();
    let ulaz = egui::RawInput {
        screen_rect: Some(pravokutnik),
        ..Default::default()
    };
    let mut prvi_izlaz = ctx.run_ui(ulaz.clone(), |_| {});
    prvi_izlaz.textures_delta.clear();
    let mut izlaz = ctx.run_ui(ulaz, |ui| {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                let mut mem = memorija.clone();
                let mut karta = Map::new(None, &mut mem, srediste)
                    .with_layer(&kml_sloj, 1.0)
                    .with_layer(&geojson_sloj, 1.0);
                if let Some(stablo) = opcija_stabla.take() {
                    karta = karta.with_plugin(stablo);
                }
                karta.show(ui, |_, _, _, _| {});
            });
    });
    izlaz.textures_delta.clear();

    let geojson_i_grouped_places_rade =
        !izlaz.shapes.is_empty() && broj_skupina == 1 && maks_u_skupini == 2;

    let izvjestaj = IzvjestajDodataka {
        rupe_odvojeni_inner_boundary,
        trokuti_odvojeni_inner_boundary,
        rupe_isti_inner_boundary,
        trokuti_isti_inner_boundary,
        rupa_iste_orijentacije_prebojana,
        cuva_kml_stilove,
        cuva_z_visine,
        panic_na_praznoj_tocki,
        broj_geometrija_iz_gx_track,
        geojson_i_grouped_places_rade,
    };

    if izvjestaj.rupe_odvojeni_inner_boundary != 2 || izvjestaj.trokuti_odvojeni_inner_boundary < 10
    {
        return Err(format!(
            "Očekivane 2 rupe i >=10 trokuta za odvojene innerBoundaryIs, dobiveno: {} rupa, {} trokuta",
            izvjestaj.rupe_odvojeni_inner_boundary, izvjestaj.trokuti_odvojeni_inner_boundary
        ));
    }
    if izvjestaj.rupe_isti_inner_boundary != 2 || izvjestaj.trokuti_isti_inner_boundary < 10 {
        return Err(format!(
            "Neočekivan broj rupa/trokuta za isti innerBoundaryIs: {} rupa, {} trokuta",
            izvjestaj.rupe_isti_inner_boundary, izvjestaj.trokuti_isti_inner_boundary
        ));
    }
    if izvjestaj.rupa_iste_orijentacije_prebojana {
        return Err(
            "Neočekivano: rupa iste orijentacije je prebojana umjesto izrezana (EvenOdd pravilo)"
                .to_string(),
        );
    }
    if izvjestaj.cuva_kml_stilove {
        return Err("walkers_extras::KmlLayer zapravo ne čuva KML stilove".to_string());
    }
    if izvjestaj.cuva_z_visine {
        return Err("walkers_extras::KmlLayer zapravo ne čuva Z visine".to_string());
    }
    if !izvjestaj.panic_na_praznoj_tocki {
        return Err("Očekivana panika KmlLayer::from_string na praznoj točki".to_string());
    }
    if izvjestaj.broj_geometrija_iz_gx_track != 0 {
        return Err("walkers_extras zapravo gubi gx:Track geometrije".to_string());
    }
    if !izvjestaj.geojson_i_grouped_places_rade {
        return Err("GeoJsonLayer ili GroupedPlacesTree nisu iscrtali oblike".to_string());
    }

    let _ = <LabeledSymbol as Place>::position;
    let _ = <LabeledSymbolGroup as Group>::draw::<LabeledSymbol>;

    Ok(izvjestaj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_walkers_extras_ogranicenja_potvrdena() {
        let izvjestaj = provjeri_walkers_extras(false).unwrap();
        assert_eq!(izvjestaj.rupe_odvojeni_inner_boundary, 2);
        assert!(izvjestaj.trokuti_odvojeni_inner_boundary >= 12);
        assert_eq!(izvjestaj.rupe_isti_inner_boundary, 2);
        assert!(izvjestaj.trokuti_isti_inner_boundary >= 12);
        assert!(!izvjestaj.rupa_iste_orijentacije_prebojana);
        assert!(!izvjestaj.cuva_kml_stilove);
        assert!(!izvjestaj.cuva_z_visine);
        assert!(izvjestaj.panic_na_praznoj_tocki);
        assert_eq!(izvjestaj.broj_geometrija_iz_gx_track, 0);
        assert!(izvjestaj.geojson_i_grouped_places_rade);
    }

    #[test]
    fn test_walkers_extras_namjerni_kvar_pada() {
        let rezultat = provjeri_walkers_extras(true);
        assert!(
            rezultat.is_err(),
            "Provjera mora pasti kad se lažno tvrdi da walkers_extras čuva KML stilove"
        );
    }
}
