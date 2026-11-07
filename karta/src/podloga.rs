//! Podloge izdanja 0.0.1 i što mora pisati uz njih (docs/podloge.md).

use eframe::egui;
use walkers::sources::{Attribution, OpenFreeMap, TileSource};
use walkers::{HeaderValue, HttpOptions, HttpTiles, Style, TileId, Tiles};

const KORISNIK: &str = concat!("TerraSimple/", env!("CARGO_PKG_VERSION"));
const POLA_SVIJETA_M: f64 = 20_037_508.342_789_244;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListDgu {
    Dof,
    Hok,
    Tk25,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Podloga {
    OpenFreeMap { tamna: bool },
    Dgu(ListDgu),
}

impl ListDgu {
    fn sloj(self) -> &'static str {
        match self {
            Self::Dof => "DOF",
            Self::Hok => "HOK",
            Self::Tk25 => "TK25",
        }
    }
}

/// DGU WMS kao pločice. Traži se EPSG:3857, koji radi iako ga opis usluge ne navodi. EPSG:3785, koji
/// navodi, vraća krivo mjesto (6. 11. 2026.).
struct DguWms(ListDgu);

impl TileSource for DguWms {
    fn tile_url(&self, t: TileId) -> String {
        let [x1, y1, x2, y2] = okvir_plocice(t);
        format!(
            "https://geoportal.dgu.hr/wms?SERVICE=WMS&VERSION=1.3.0&REQUEST=GetMap&LAYERS={}&STYLES=&CRS=EPSG:3857&BBOX={x1},{y1},{x2},{y2}&WIDTH=256&HEIGHT=256&FORMAT=image/jpeg",
            self.0.sloj()
        )
    }

    fn attribution(&self) -> Attribution {
        Attribution {
            text: "",
            url: "https://geoportal.dgu.hr",
            logo_light: None,
            logo_dark: None,
        }
    }
}

fn okvir_plocice(t: TileId) -> [f64; 4] {
    let velicina = 2.0 * POLA_SVIJETA_M / 2.0_f64.powi(i32::from(t.zoom));
    let x1 = -POLA_SVIJETA_M + f64::from(t.x) * velicina;
    let y2 = POLA_SVIJETA_M - f64::from(t.y) * velicina;
    [x1, y2 - velicina, x1 + velicina, y2]
}

impl Podloga {
    /// Tekst koji mora biti vidljiv na karti, prema uvjetima izvora. Ne prevodi se.
    pub fn navod(self) -> &'static str {
        match self {
            Self::OpenFreeMap { .. } => "OpenFreeMap © OpenMapTiles Data from OpenStreetMap",
            Self::Dgu(ListDgu::Dof) => {
                "Sadrži podatke Državne geodetske uprave, DOF, u skladu s Otvorenom dozvolom"
            }
            Self::Dgu(ListDgu::Hok) => {
                "Sadrži podatke Državne geodetske uprave, HOK, u skladu s Otvorenom dozvolom"
            }
            Self::Dgu(ListDgu::Tk25) => {
                "Sadrži podatke Državne geodetske uprave, TK25, u skladu s Otvorenom dozvolom"
            }
        }
    }

    /// Program se predstavlja svojim imenom, kako traže OpenFreeMap i OpenStreetMap.
    pub(crate) fn izvor(self, ctx: egui::Context) -> Box<dyn Tiles> {
        let opcije = HttpOptions {
            user_agent: Some(HeaderValue::from_static(KORISNIK)),
            ..HttpOptions::default()
        };
        match self {
            Self::OpenFreeMap { tamna } => {
                let stil = if tamna {
                    Style::openmaptiles_basemap_dark()
                } else {
                    Style::openmaptiles_basemap_light()
                };
                Box::new(HttpTiles::with_options_and_style(
                    OpenFreeMap,
                    opcije,
                    stil,
                    ctx,
                ))
            }
            Self::Dgu(list) => Box::new(HttpTiles::with_options(DguWms(list), opcije, ctx)),
        }
    }
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn okvir_cijelog_svijeta_i_cetvrtine() {
        let svijet = okvir_plocice(TileId {
            x: 0,
            y: 0,
            zoom: 0,
        });
        assert_eq!(
            svijet,
            [
                -POLA_SVIJETA_M,
                -POLA_SVIJETA_M,
                POLA_SVIJETA_M,
                POLA_SVIJETA_M
            ]
        );
        let sjeveroistok = okvir_plocice(TileId {
            x: 1,
            y: 0,
            zoom: 1,
        });
        assert_eq!(sjeveroistok, [0.0, 0.0, POLA_SVIJETA_M, POLA_SVIJETA_M]);
        let jugozapad = okvir_plocice(TileId {
            x: 0,
            y: 1,
            zoom: 1,
        });
        assert_eq!(jugozapad, [-POLA_SVIJETA_M, -POLA_SVIJETA_M, 0.0, 0.0]);
    }

    #[test]
    fn adresa_trazi_pravi_sloj_i_sustav_koji_radi() {
        for (list, ime) in [
            (ListDgu::Dof, "DOF"),
            (ListDgu::Hok, "HOK"),
            (ListDgu::Tk25, "TK25"),
        ] {
            let url = DguWms(list).tile_url(TileId {
                x: 0,
                y: 0,
                zoom: 0,
            });
            assert!(url.contains(&format!("LAYERS={ime}&")), "{url}");
            assert!(url.contains("CRS=EPSG:3857"), "{url}");
            assert_eq!(url.matches("EPSG:3785").count(), 0);
        }
    }

    #[test]
    fn svaka_podloga_daje_izvor_s_pravom_velicinom_plocice() {
        let ctx = egui::Context::default();
        let velicina = |p: Podloga| p.izvor(ctx.clone()).tile_size();
        assert_eq!(velicina(Podloga::OpenFreeMap { tamna: false }), 512);
        assert_eq!(velicina(Podloga::OpenFreeMap { tamna: true }), 512);
        assert_eq!(velicina(Podloga::Dgu(ListDgu::Dof)), 256);
    }

    #[test]
    fn svaka_podloga_ima_propisani_navod() {
        let ofm = Podloga::OpenFreeMap { tamna: true }.navod();
        assert!(
            ofm.contains("OpenMapTiles") && ofm.contains("OpenStreetMap"),
            "{ofm}"
        );
        for (list, ime) in [
            (ListDgu::Dof, "DOF"),
            (ListDgu::Hok, "HOK"),
            (ListDgu::Tk25, "TK25"),
        ] {
            let n = Podloga::Dgu(list).navod();
            assert!(
                n.contains("Sadrži podatke Državne geodetske uprave") && n.contains(ime),
                "{n}"
            );
        }
    }
}
