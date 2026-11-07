//! Popis sustava i njihovo prepoznavanje.

/// Sustavi koje program poznaje.
///
/// HDKS ima isti prijenos prema WGS-u kao QGIS bez izbora, operaciju MGI 1901 to WGS 84 (1)
/// s točnošću oko 5 m, da se isti sloj u oba programa crta na istom mjestu (7. 11. 2026.).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sustav {
    Wgs84,
    WebMercator,
    Htrs96Tm,
    HdksZona5,
    HdksZona6,
}

const POPIS_EPSG: [(i64, Sustav); 7] = [
    (4326, Sustav::Wgs84),
    (3857, Sustav::WebMercator),
    (3765, Sustav::Htrs96Tm),
    (3907, Sustav::HdksZona5),
    (8677, Sustav::HdksZona5),
    (3908, Sustav::HdksZona6),
    (8678, Sustav::HdksZona6),
];

impl Sustav {
    /// Stari brojevi 3907 i 3908 čitaju se, jer ih ima u datotekama, a pišu se novi.
    ///
    /// 31275 i 31276 namjerno nisu ovdje: isti brojevi uz njih u QGIS-u završe oko 310 m dalje.
    pub fn iz_epsg(broj: i64) -> Option<Self> {
        POPIS_EPSG.iter().find(|(b, _)| *b == broj).map(|(_, s)| *s)
    }

    /// Broj koji program zapisuje. Za HDKS je to novi broj, a ne zastarjeli.
    pub fn epsg(self) -> i64 {
        match self {
            Self::Wgs84 => 4326,
            Self::WebMercator => 3857,
            Self::Htrs96Tm => 3765,
            Self::HdksZona5 => 8677,
            Self::HdksZona6 => 8678,
        }
    }

    /// Prepoznaje sustav iz retka `gpkg_spatial_ref_sys`.
    ///
    /// QGIS sloj u 3907 ili 3908 sprema pod brojem 100000 i organizacijom `NONE`, pa pravi broj
    /// piše samo na kraju definicije.
    pub fn iz_retka_geopaketa(organizacija: &str, broj: i64, definicija: &str) -> Option<Self> {
        if organizacija.eq_ignore_ascii_case("EPSG") {
            return Self::iz_epsg(broj);
        }
        broj_na_kraju_definicije(definicija).and_then(Self::iz_epsg)
    }

    /// Procjena točnosti prijenosa prema WGS-u u metrima, za natpis uz koordinate.
    pub fn tocnost_prema_wgs_m(self) -> f64 {
        match self {
            Self::Wgs84 | Self::WebMercator => 0.0,
            Self::Htrs96Tm => 1.0,
            Self::HdksZona5 | Self::HdksZona6 => 5.0,
        }
    }

    pub(crate) fn je_geografski(self) -> bool {
        self == Self::Wgs84
    }

    pub(crate) fn definicija(self) -> &'static str {
        match self {
            Self::Wgs84 => "+proj=longlat +datum=WGS84 +no_defs",
            Self::WebMercator => {
                "+proj=merc +a=6378137 +b=6378137 +lat_ts=0 +lon_0=0 +x_0=0 +y_0=0 +k=1 +units=m +nadgrids=@null +no_defs"
            }
            Self::Htrs96Tm => {
                "+proj=tmerc +lat_0=0 +lon_0=16.5 +k=0.9999 +x_0=500000 +y_0=0 +ellps=GRS80 +towgs84=0,0,0,0,0,0,0 +units=m +no_defs"
            }
            Self::HdksZona5 => {
                "+proj=tmerc +lat_0=0 +lon_0=15 +k=0.9999 +x_0=5500000 +y_0=0 +ellps=bessel +towgs84=682,-203,480,0,0,0,0 +units=m +no_defs"
            }
            Self::HdksZona6 => {
                "+proj=tmerc +lat_0=0 +lon_0=18 +k=0.9999 +x_0=6500000 +y_0=0 +ellps=bessel +towgs84=682,-203,480,0,0,0,0 +units=m +no_defs"
            }
        }
    }
}

/// Zadnji `AUTHORITY["EPSG","n"]` (WKT 1) ili `ID["EPSG",n]` (WKT 2). Zadnji je onaj cijelog sustava.
fn broj_na_kraju_definicije(definicija: &str) -> Option<i64> {
    let polozaj = definicija.rfind("\"EPSG\"")?;
    let ostatak = definicija[polozaj + 6..].trim_start_matches([',', ' ', '"']);
    let kraj = ostatak.find(|z: char| !z.is_ascii_digit())?;
    ostatak[..kraj].parse().ok()
}

#[cfg(test)]
mod testovi {
    use super::*;

    const WKT_3908_IZ_QGISA: &str = r#"PROJCS["MGI 1901 / Balkans zone 6",GEOGCS["MGI 1901",DATUM["MGI_1901",SPHEROID["Bessel 1841",6377397.155,299.1528128]],PRIMEM["Greenwich",0],UNIT["degree",0.0174532925199433,AUTHORITY["EPSG","9122"]],AUTHORITY["EPSG","3906"]],PROJECTION["Transverse_Mercator"],PARAMETER["central_meridian",18],UNIT["metre",1],AXIS["Northing",NORTH],AXIS["Easting",EAST],AUTHORITY["EPSG","3908"]]"#;

    #[test]
    fn stari_i_novi_broj_hdks_su_isti_sustav_a_pise_se_novi() {
        assert_eq!(Sustav::iz_epsg(3907), Some(Sustav::HdksZona5));
        assert_eq!(Sustav::iz_epsg(8677), Some(Sustav::HdksZona5));
        assert_eq!(Sustav::iz_epsg(3908), Some(Sustav::HdksZona6));
        assert_eq!(Sustav::iz_epsg(8678), Some(Sustav::HdksZona6));
        assert_eq!(Sustav::HdksZona5.epsg(), 8677);
        assert_eq!(Sustav::HdksZona6.epsg(), 8678);
    }

    #[test]
    fn svaki_sustav_cita_broj_koji_sam_pise() {
        for s in [
            Sustav::Wgs84,
            Sustav::WebMercator,
            Sustav::Htrs96Tm,
            Sustav::HdksZona5,
            Sustav::HdksZona6,
        ] {
            assert_eq!(Sustav::iz_epsg(s.epsg()), Some(s));
        }
    }

    #[test]
    fn mgi_31276_nije_hdks() {
        // Isti brojevi uz 31276 u QGIS-u završe oko 310 m dalje nego uz 3908.
        assert_eq!(Sustav::iz_epsg(31275), None);
        assert_eq!(Sustav::iz_epsg(31276), None);
        assert_eq!(Sustav::iz_epsg(0), None);
    }

    #[test]
    fn broj_100000_iz_qgisa_prepoznaje_se_iz_definicije() {
        assert_eq!(
            Sustav::iz_retka_geopaketa("NONE", 100_000, WKT_3908_IZ_QGISA),
            Some(Sustav::HdksZona6)
        );
    }

    #[test]
    fn epsg_organizacija_gleda_broj_a_ne_definiciju() {
        assert_eq!(
            Sustav::iz_retka_geopaketa("EPSG", 3765, WKT_3908_IZ_QGISA),
            Some(Sustav::Htrs96Tm)
        );
        assert_eq!(
            Sustav::iz_retka_geopaketa("epsg", 3765, ""),
            Some(Sustav::Htrs96Tm)
        );
    }

    #[test]
    fn wkt2_id_se_cita() {
        assert_eq!(
            Sustav::iz_retka_geopaketa("NONE", 1, r#"PROJCRS["x",ID["EPSG",3765]]"#),
            Some(Sustav::Htrs96Tm)
        );
    }

    #[test]
    fn definicija_bez_broja_nije_nagadanje() {
        assert_eq!(
            Sustav::iz_retka_geopaketa("NONE", 1, "PROJCS[\"lokalni\"]"),
            None
        );
        assert_eq!(
            Sustav::iz_retka_geopaketa("NONE", 1, "AUTHORITY[\"EPSG\",\"1234567"),
            None
        );
        assert_eq!(Sustav::iz_retka_geopaketa("NONE", 1, "ID[\"EPSG\",]"), None);
    }

    #[test]
    fn tocnost_hdks_je_grublja_od_htrs() {
        assert_eq!(Sustav::HdksZona5.tocnost_prema_wgs_m(), 5.0);
        assert_eq!(Sustav::HdksZona6.tocnost_prema_wgs_m(), 5.0);
        assert_eq!(Sustav::Htrs96Tm.tocnost_prema_wgs_m(), 1.0);
        assert_eq!(Sustav::Wgs84.tocnost_prema_wgs_m(), 0.0);
        assert_eq!(Sustav::WebMercator.tocnost_prema_wgs_m(), 0.0);
    }
}
