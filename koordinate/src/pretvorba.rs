//! Pretvorba točke iz jednog sustava u drugi.

use crate::sustav::Sustav;
use proj4rs::proj::Proj;
use proj4rs::transform::transform;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Greska {
    /// Definicija sustava nije valjana. Na popisu to nije moguće, ali kod to ne pretpostavlja.
    Definicija,
    /// Ulazna ili izlazna vrijednost nije konačan broj.
    NijeBroj,
    /// Točka je izvan onoga što sustav može prikazati, npr. pol u Mercatoru.
    IzvanSustava,
}

impl std::fmt::Display for Greska {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Definicija => "definicija koordinatnog sustava nije valjana",
            Self::NijeBroj => "koordinata nije konačan broj",
            Self::IzvanSustava => "točka je izvan koordinatnog sustava",
        })
    }
}

impl std::error::Error for Greska {}

fn proj_iz_definicije(definicija: &str) -> Result<Proj, Greska> {
    Proj::from_proj_string(definicija).map_err(|_| Greska::Definicija)
}

/// Pretvara točke iz jednog sustava u drugi. Stupnjevi su redom dužina i širina.
pub struct Pretvornik {
    iz: Proj,
    u: Proj,
    iz_geo: bool,
    u_geo: bool,
}

fn konacna(t: [f64; 2]) -> Result<[f64; 2], Greska> {
    if t[0].is_finite() && t[1].is_finite() {
        Ok(t)
    } else {
        Err(Greska::NijeBroj)
    }
}

/// Biblioteka za geografske koordinate traži i vraća radijane, a mi radimo u stupnjevima.
fn mnozitelj(geografski: bool, u_stupnju: f64) -> f64 {
    if geografski { u_stupnju } else { 1.0 }
}

impl Pretvornik {
    pub fn novi(iz: Sustav, u: Sustav) -> Result<Self, Greska> {
        Self::iz_definicija(
            iz.definicija(),
            u.definicija(),
            iz.je_geografski(),
            u.je_geografski(),
        )
    }

    fn iz_definicija(iz: &str, u: &str, iz_geo: bool, u_geo: bool) -> Result<Self, Greska> {
        Ok(Self {
            iz: proj_iz_definicije(iz)?,
            u: proj_iz_definicije(u)?,
            iz_geo,
            u_geo,
        })
    }

    pub fn pretvori(&self, t: [f64; 2]) -> Result<[f64; 2], Greska> {
        let t = konacna(t)?;
        let k = mnozitelj(self.iz_geo, 1.0_f64.to_radians());
        let mut tocka = (t[0] * k, t[1] * k, 0.0);
        transform(&self.iz, &self.u, &mut tocka).map_err(|_| Greska::IzvanSustava)?;
        let k = mnozitelj(self.u_geo, 1.0_f64.to_degrees());
        konacna([tocka.0 * k, tocka.1 * k])
    }
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn neispravna_definicija_je_greska_a_ne_pad() {
        let dobra = Sustav::Wgs84.definicija();
        let losa = "+proj=nepostoji";
        let greska = |iz, u| Pretvornik::iz_definicija(iz, u, true, true).err();
        assert_eq!(greska(losa, dobra), Some(Greska::Definicija));
        assert_eq!(greska(dobra, losa), Some(Greska::Definicija));
        assert_eq!(greska(dobra, dobra), None);
    }

    #[test]
    fn konacnost_gleda_obje_koordinate() {
        assert_eq!(konacna([1.0, 2.0]), Ok([1.0, 2.0]));
        assert_eq!(konacna([f64::NAN, 2.0]), Err(Greska::NijeBroj));
        assert_eq!(konacna([1.0, f64::INFINITY]), Err(Greska::NijeBroj));
    }

    #[test]
    fn nije_broj_se_odbija_u_obje_koordinate() {
        let p = Pretvornik::novi(Sustav::Wgs84, Sustav::Htrs96Tm).expect("pretvornik");
        assert_eq!(p.pretvori([f64::NAN, 45.0]), Err(Greska::NijeBroj));
        assert_eq!(p.pretvori([18.0, f64::INFINITY]), Err(Greska::NijeBroj));
    }

    #[test]
    fn pol_nije_u_mercatoru() {
        let p = Pretvornik::novi(Sustav::Wgs84, Sustav::WebMercator).expect("pretvornik");
        let r = p.pretvori([0.0, 90.0]);
        assert!(
            matches!(r, Err(Greska::IzvanSustava | Greska::NijeBroj)),
            "{r:?}"
        );
        assert!(p.pretvori([0.0, 100.0]).is_err());
    }

    #[test]
    fn prevelik_rezultat_nije_tiho_prihvacen() {
        let p = Pretvornik::novi(Sustav::Htrs96Tm, Sustav::WebMercator).expect("pretvornik");
        assert!(p.pretvori([1e300, 1e300]).is_err());
    }

    #[test]
    fn poruke_postoje_za_svaku_gresku() {
        assert!(Greska::Definicija.to_string().contains("definicija"));
        assert!(Greska::NijeBroj.to_string().contains("broj"));
        assert!(Greska::IzvanSustava.to_string().contains("izvan"));
    }
}
