//! Sustavi koje TerraSimple poznaje, zapisani onako kako ih QGIS 3.44 piše u projekt.
//!
//! QGIS sustav sloja čita samo iz punog zapisa; sam kod „EPSG:4326” ostavi sloj bez
//! sustava, pa se objekti ne preračunaju i nacrtaju se na krivom mjestu.

use koordinate::Sustav;

pub const WGS84: &str = r#"<spatialrefsys nativeFormat="Wkt"><wkt>GEOGCRS["WGS 84",ENSEMBLE["World Geodetic System 1984 ensemble",MEMBER["World Geodetic System 1984 (Transit)"],MEMBER["World Geodetic System 1984 (G730)"],MEMBER["World Geodetic System 1984 (G873)"],MEMBER["World Geodetic System 1984 (G1150)"],MEMBER["World Geodetic System 1984 (G1674)"],MEMBER["World Geodetic System 1984 (G1762)"],MEMBER["World Geodetic System 1984 (G2139)"],MEMBER["World Geodetic System 1984 (G2296)"],ELLIPSOID["WGS 84",6378137,298.257223563,LENGTHUNIT["metre",1]],ENSEMBLEACCURACY[2.0]],PRIMEM["Greenwich",0,ANGLEUNIT["degree",0.0174532925199433]],CS[ellipsoidal,2],AXIS["geodetic latitude (Lat)",north,ORDER[1],ANGLEUNIT["degree",0.0174532925199433]],AXIS["geodetic longitude (Lon)",east,ORDER[2],ANGLEUNIT["degree",0.0174532925199433]],USAGE[SCOPE["Horizontal component of 3D system."],AREA["World."],BBOX[-90,-180,90,180]],ID["EPSG",4326]]</wkt><proj4>+proj=longlat +datum=WGS84 +no_defs</proj4><srsid>3452</srsid><srid>4326</srid><authid>EPSG:4326</authid><description>WGS 84</description><projectionacronym>longlat</projectionacronym><ellipsoidacronym>EPSG:7030</ellipsoidacronym><geographicflag>true</geographicflag></spatialrefsys>"#;

pub const PSEUDO_MERKATOR: &str = r#"<spatialrefsys nativeFormat="Wkt"><wkt>PROJCRS["WGS 84 / Pseudo-Mercator",BASEGEOGCRS["WGS 84",ENSEMBLE["World Geodetic System 1984 ensemble",MEMBER["World Geodetic System 1984 (Transit)"],MEMBER["World Geodetic System 1984 (G730)"],MEMBER["World Geodetic System 1984 (G873)"],MEMBER["World Geodetic System 1984 (G1150)"],MEMBER["World Geodetic System 1984 (G1674)"],MEMBER["World Geodetic System 1984 (G1762)"],MEMBER["World Geodetic System 1984 (G2139)"],MEMBER["World Geodetic System 1984 (G2296)"],ELLIPSOID["WGS 84",6378137,298.257223563,LENGTHUNIT["metre",1]],ENSEMBLEACCURACY[2.0]],PRIMEM["Greenwich",0,ANGLEUNIT["degree",0.0174532925199433]],ID["EPSG",4326]],CONVERSION["Popular Visualisation Pseudo-Mercator",METHOD["Popular Visualisation Pseudo Mercator",ID["EPSG",1024]],PARAMETER["Latitude of natural origin",0,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8801]],PARAMETER["Longitude of natural origin",0,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8802]],PARAMETER["False easting",0,LENGTHUNIT["metre",1],ID["EPSG",8806]],PARAMETER["False northing",0,LENGTHUNIT["metre",1],ID["EPSG",8807]]],CS[Cartesian,2],AXIS["easting (X)",east,ORDER[1],LENGTHUNIT["metre",1]],AXIS["northing (Y)",north,ORDER[2],LENGTHUNIT["metre",1]],USAGE[SCOPE["Web mapping and visualisation."],AREA["World between 85.06°S and 85.06°N."],BBOX[-85.06,-180,85.06,180]],ID["EPSG",3857]]</wkt><proj4>+proj=merc +a=6378137 +b=6378137 +lat_ts=0 +lon_0=0 +x_0=0 +y_0=0 +k=1 +units=m +nadgrids=@null +wktext +no_defs</proj4><srsid>3857</srsid><srid>3857</srid><authid>EPSG:3857</authid><description>WGS 84 / Pseudo-Mercator</description><projectionacronym>merc</projectionacronym><ellipsoidacronym>EPSG:7030</ellipsoidacronym><geographicflag>false</geographicflag></spatialrefsys>"#;

pub const HTRS96_TM: &str = r#"<spatialrefsys nativeFormat="Wkt"><wkt>PROJCRS["HTRS96 / Croatia TM",BASEGEOGCRS["HTRS96",DATUM["Croatian Terrestrial Reference System",ELLIPSOID["GRS 1980",6378137,298.257222101,LENGTHUNIT["metre",1]]],PRIMEM["Greenwich",0,ANGLEUNIT["degree",0.0174532925199433]],ID["EPSG",4761]],CONVERSION["Croatia Transverse Mercator",METHOD["Transverse Mercator",ID["EPSG",9807]],PARAMETER["Latitude of natural origin",0,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8801]],PARAMETER["Longitude of natural origin",16.5,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8802]],PARAMETER["Scale factor at natural origin",0.9999,SCALEUNIT["unity",1],ID["EPSG",8805]],PARAMETER["False easting",500000,LENGTHUNIT["metre",1],ID["EPSG",8806]],PARAMETER["False northing",0,LENGTHUNIT["metre",1],ID["EPSG",8807]]],CS[Cartesian,2],AXIS["(E)",east,ORDER[1],LENGTHUNIT["metre",1]],AXIS["(N)",north,ORDER[2],LENGTHUNIT["metre",1]],USAGE[SCOPE["Cadastre, engineering survey, topographic mapping (large scale)."],AREA["Croatia - onshore."],BBOX[42.34,13.43,46.54,19.43]],ID["EPSG",3765]]</wkt><proj4>+proj=tmerc +lat_0=0 +lon_0=16.5 +k=0.9999 +x_0=500000 +y_0=0 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs</proj4><srsid>1723</srsid><srid>3765</srid><authid>EPSG:3765</authid><description>HTRS96 / Croatia TM</description><projectionacronym>tmerc</projectionacronym><ellipsoidacronym>EPSG:7019</ellipsoidacronym><geographicflag>false</geographicflag></spatialrefsys>"#;

pub const HDKS_ZONA_5: &str = r#"<spatialrefsys nativeFormat="Wkt"><wkt>PROJCRS["MGI 1901 / Balkans zone 5",BASEGEOGCRS["MGI 1901",DATUM["MGI 1901",ELLIPSOID["Bessel 1841",6377397.155,299.1528128,LENGTHUNIT["metre",1]]],PRIMEM["Greenwich",0,ANGLEUNIT["degree",0.0174532925199433]],ID["EPSG",3906]],CONVERSION["Balkans zone 5",METHOD["Transverse Mercator",ID["EPSG",9807]],PARAMETER["Latitude of natural origin",0,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8801]],PARAMETER["Longitude of natural origin",15,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8802]],PARAMETER["Scale factor at natural origin",0.9999,SCALEUNIT["unity",1],ID["EPSG",8805]],PARAMETER["False easting",5500000,LENGTHUNIT["metre",1],ID["EPSG",8806]],PARAMETER["False northing",0,LENGTHUNIT["metre",1],ID["EPSG",8807]]],CS[Cartesian,2],AXIS["easting (Y)",east,ORDER[1],LENGTHUNIT["metre",1]],AXIS["northing (X)",north,ORDER[2],LENGTHUNIT["metre",1]],USAGE[SCOPE["Cadastre, engineering survey, topographic mapping (large and medium scale)."],AREA["Bosnia and Herzegovina - west of 16°30'E; Croatia - onshore west of 16°30'E; Slovenia - onshore."],BBOX[42.95,13.38,46.88,16.5]],ID["EPSG",8677]]</wkt><proj4>+proj=tmerc +lat_0=0 +lon_0=15 +k=0.9999 +x_0=5500000 +y_0=0 +ellps=bessel +towgs84=682,-203,480,0,0,0,0 +units=m +no_defs</proj4><srsid>60313</srsid><srid>520000313</srid><authid>EPSG:8677</authid><description>MGI 1901 / Balkans zone 5</description><projectionacronym>tmerc</projectionacronym><ellipsoidacronym>EPSG:7004</ellipsoidacronym><geographicflag>false</geographicflag></spatialrefsys>"#;

pub const HDKS_ZONA_6: &str = r#"<spatialrefsys nativeFormat="Wkt"><wkt>PROJCRS["MGI 1901 / Balkans zone 6",BASEGEOGCRS["MGI 1901",DATUM["MGI 1901",ELLIPSOID["Bessel 1841",6377397.155,299.1528128,LENGTHUNIT["metre",1]]],PRIMEM["Greenwich",0,ANGLEUNIT["degree",0.0174532925199433]],ID["EPSG",3906]],CONVERSION["Balkans zone 6",METHOD["Transverse Mercator",ID["EPSG",9807]],PARAMETER["Latitude of natural origin",0,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8801]],PARAMETER["Longitude of natural origin",18,ANGLEUNIT["degree",0.0174532925199433],ID["EPSG",8802]],PARAMETER["Scale factor at natural origin",0.9999,SCALEUNIT["unity",1],ID["EPSG",8805]],PARAMETER["False easting",6500000,LENGTHUNIT["metre",1],ID["EPSG",8806]],PARAMETER["False northing",0,LENGTHUNIT["metre",1],ID["EPSG",8807]]],CS[Cartesian,2],AXIS["easting (Y)",east,ORDER[1],LENGTHUNIT["metre",1]],AXIS["northing (X)",north,ORDER[2],LENGTHUNIT["metre",1]],USAGE[SCOPE["Cadastre, engineering survey, topographic mapping (large and medium scale)."],AREA["Bosnia and Herzegovina - between 16°30'E and 19°30'E; Croatia - onshore east of 16°30'E; Montenegro - onshore west of 19°30'E; Serbia - west of 19°30'E."],BBOX[41.79,16.5,46.55,19.51]],ID["EPSG",8678]]</wkt><proj4>+proj=tmerc +lat_0=0 +lon_0=18 +k=0.9999 +x_0=6500000 +y_0=0 +ellps=bessel +towgs84=682,-203,480,0,0,0,0 +units=m +no_defs</proj4><srsid>60314</srsid><srid>520000314</srid><authid>EPSG:8678</authid><description>MGI 1901 / Balkans zone 6</description><projectionacronym>tmerc</projectionacronym><ellipsoidacronym>EPSG:7004</ellipsoidacronym><geographicflag>false</geographicflag></spatialrefsys>"#;

/// Puni zapis `<spatialrefsys>` za sustav.
pub fn zapis(sustav: Sustav) -> &'static str {
    match sustav {
        Sustav::Wgs84 => WGS84,
        Sustav::WebMercator => PSEUDO_MERKATOR,
        Sustav::Htrs96Tm => HTRS96_TM,
        Sustav::HdksZona5 => HDKS_ZONA_5,
        Sustav::HdksZona6 => HDKS_ZONA_6,
    }
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn svaki_sustav_ima_svoj_epsg_u_zapisu() {
        for sustav in [
            Sustav::Wgs84,
            Sustav::WebMercator,
            Sustav::Htrs96Tm,
            Sustav::HdksZona5,
            Sustav::HdksZona6,
        ] {
            let z = zapis(sustav);
            assert!(z.starts_with("<spatialrefsys"), "{sustav:?}");
            assert!(z.contains("<wkt>"), "{sustav:?}");
            assert!(
                z.contains(&format!("<authid>EPSG:{}</authid>", sustav.epsg())),
                "{sustav:?}"
            );
        }
    }
}
