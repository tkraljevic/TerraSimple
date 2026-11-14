//! KML kakav je pročitan, prije nego što postane mape i slojevi.

use slojevi::{Boja, Vrh};
use std::collections::BTreeMap;

/// Pročitana datoteka. Korijen je najviši `Document`, a njegove mape su mape Google Eartha.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DokumentKml {
    pub korijen: MapaKml,
    pub gubici: Gubici,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapaKml {
    pub naziv: String,
    pub opis: String,
    pub mape: Vec<MapaKml>,
    pub objekti: Vec<ObjektKml>,
}

impl MapaKml {
    /// Broj objekata u mapi i svim podmapama.
    #[must_use]
    pub fn broj_objekata(&self) -> usize {
        self.objekti.len() + self.mape.iter().map(Self::broj_objekata).sum::<usize>()
    }
}

/// `Placemark`. Jedan objekt može imati više geometrija (`MultiGeometry`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObjektKml {
    pub naziv: String,
    pub opis: String,
    pub stil: StilKml,
    /// `styleUrl` iz datoteke. Čitač po njemu na kraju upiše zajednički stil u `stil`.
    pub adresa_stila: Option<String>,
    pub geometrije: Vec<GeometrijaKml>,
    /// `Document` u kojem je objekt, jer ugniježđeni `Document` smije ponoviti naziv stila.
    pub(crate) doseg: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GeometrijaKml {
    Tocka(Vrh),
    Linija(Vec<Vrh>),
    /// Prvi prsten je vanjski rub, a ostali su rupe. Prsten nije zatvoren ponovljenom točkom.
    Poligon(Vec<Vec<Vrh>>),
}

/// Stil iz `LineStyle`, `PolyStyle` i `IconStyle`. Prazno znači da ga KML ne zadaje.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StilKml {
    pub boja_crte: Option<Boja>,
    pub sirina_px: Option<f64>,
    pub boja_ispune: Option<Boja>,
    pub bez_ispune: bool,
    pub bez_ruba: bool,
    pub boja_ikone: Option<Boja>,
    pub mjerilo_ikone: Option<f64>,
}

/// Što iz datoteke nije postalo objekt na karti.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Gubici {
    /// Elementi koje 2D karta ne prikazuje, po nazivu oznake, npr. `NetworkLink` ili `GroundOverlay`.
    pub preskoceno: BTreeMap<String, usize>,
    /// Geometrije bez dovoljno točaka, npr. prazna linija iz nedovršenog crtanja.
    pub prazne_geometrije: usize,
    /// Geometrije s koordinatom koja nije broj ili je izvan raspona.
    pub neispravne_koordinate: usize,
    /// Objekti bez ijedne geometrije.
    pub bez_geometrije: usize,
    /// Svojstva iz GeoJSON-a koja projekt ne pamti, po nazivu.
    pub izostavljena_svojstva: std::collections::BTreeSet<String>,
}

impl Gubici {
    #[must_use]
    pub fn prazno(&self) -> bool {
        self == &Self::default()
    }
}

/// Boja u KML-u je zapisana kao `aabbggrr`.
#[must_use]
pub fn boja_iz_kml(tekst: &str) -> Option<Boja> {
    let t = tekst.trim();
    if t.len() != 8 || !t.is_ascii() {
        return None;
    }
    let bajt = |i: usize| u8::from_str_radix(&t[i..i + 2], 16).ok();
    Some(Boja {
        a: bajt(0)?,
        b: bajt(2)?,
        g: bajt(4)?,
        r: bajt(6)?,
    })
}

/// `lon,lat[,visina]` odvojeno razmacima. Ništa ako ijedna koordinata nije ispravna.
pub fn koordinate(tekst: &str) -> Option<Vec<Vrh>> {
    tekst.split_whitespace().map(vrh).collect()
}

/// `gx:coord` ima `lon lat visina` odvojeno razmacima.
pub fn koordinata_traga(tekst: &str) -> Option<Vrh> {
    let mut dijelovi = tekst.split_whitespace();
    let d = dijelovi.next()?.parse().ok()?;
    let s = dijelovi.next()?.parse().ok()?;
    let v = dijelovi.next().map(str::parse).transpose().ok()?;
    ispravan(d, s, v)
}

fn vrh(tekst: &str) -> Option<Vrh> {
    let mut dijelovi = tekst.split(',');
    let d = dijelovi.next()?.trim().parse().ok()?;
    let s = dijelovi.next()?.trim().parse().ok()?;
    let v = dijelovi
        .next()
        .filter(|t| !t.trim().is_empty())
        .map(|t| t.trim().parse())
        .transpose()
        .ok()?;
    ispravan(d, s, v)
}

pub(crate) fn ispravan(duzina: f64, sirina: f64, visina: Option<f64>) -> Option<Vrh> {
    let u_rasponu = (-180.0..=180.0).contains(&duzina) && (-90.0..=90.0).contains(&sirina);
    let visina_broj = visina.is_none_or(f64::is_finite);
    (u_rasponu && visina_broj).then_some(Vrh {
        duzina,
        sirina,
        visina,
    })
}

/// Prsten bez ponovljene zadnje točke. Ništa ako ostanu manje od tri točke.
#[must_use]
pub fn otvoren_prsten(mut prsten: Vec<Vrh>) -> Option<Vec<Vrh>> {
    if prsten.len() > 1 && prsten.first() == prsten.last() {
        prsten.pop();
    }
    (prsten.len() >= 3).then_some(prsten)
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn boja_je_aabbggrr() {
        let b = boja_iz_kml("ff0000ff").expect("boja");
        assert_eq!((b.r, b.g, b.b, b.a), (255, 0, 0, 255));
        let b = boja_iz_kml(" 7f00ff00 ").expect("boja");
        assert_eq!((b.r, b.g, b.b, b.a), (0, 255, 0, 127));
        assert_eq!(boja_iz_kml("fff"), None);
        assert_eq!(boja_iz_kml("gg0000ff"), None);
        assert_eq!(boja_iz_kml("00gg00ff"), None);
        assert_eq!(boja_iz_kml("0000gg00"), None);
        assert_eq!(boja_iz_kml("000000gg"), None);
        assert_eq!(boja_iz_kml("ččččč"), None);
    }

    #[test]
    fn koordinate_s_visinom_i_bez_nje() {
        let v = koordinate(" 18.6,45.5,12 18.7,45.6\n18.8,45.7, ").expect("koordinate");
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].visina, Some(12.0));
        assert_eq!(v[1].visina, None);
        assert_eq!(v[2].visina, None);
        assert_eq!(koordinate(""), Some(Vec::new()));
        assert_eq!(koordinate("18.6"), None);
        assert_eq!(koordinate("18.6,x"), None);
        assert_eq!(koordinate("x,45"), None);
        assert_eq!(koordinate("200,45"), None);
        assert_eq!(koordinate("18,95"), None);
        assert_eq!(koordinate("18,45,nema"), None);
        assert_eq!(koordinate("18,45,inf"), None);
    }

    #[test]
    fn koordinata_traga_ima_razmake() {
        let v = koordinata_traga("18.6 45.5 100").expect("vrh");
        assert_eq!((v.duzina, v.sirina, v.visina), (18.6, 45.5, Some(100.0)));
        assert_eq!(koordinata_traga("18.6 45.5").expect("vrh").visina, None);
        assert_eq!(koordinata_traga("18.6"), None);
        assert_eq!(koordinata_traga("x 45"), None);
        assert_eq!(koordinata_traga("18 y"), None);
        assert_eq!(koordinata_traga("18 45 z"), None);
    }

    #[test]
    fn prsten_gubi_ponovljenu_tocku() {
        let v = |x: f64| Vrh::novi(x, x);
        let p = otvoren_prsten(vec![v(1.0), v(2.0), v(3.0), v(1.0)]).expect("prsten");
        assert_eq!(p.len(), 3);
        assert_eq!(otvoren_prsten(vec![v(1.0), v(2.0), v(1.0)]), None);
        assert_eq!(otvoren_prsten(vec![v(1.0)]), None);
        assert!(otvoren_prsten(vec![v(1.0), v(2.0), v(3.0)]).is_some());
    }

    #[test]
    fn broj_objekata_i_prazni_gubici() {
        let mut m = MapaKml::default();
        m.objekti.push(ObjektKml::default());
        m.mape.push(MapaKml {
            objekti: vec![ObjektKml::default(), ObjektKml::default()],
            ..MapaKml::default()
        });
        assert_eq!(m.broj_objekata(), 3);
        assert!(Gubici::default().prazno());
        let g = Gubici {
            bez_geometrije: 1,
            ..Gubici::default()
        };
        assert!(!g.prazno());
    }
}
