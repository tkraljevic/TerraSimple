//! Upravljanje datotekama projekta (Novi, Otvori, Spremi, Spremi kao) i birač datoteka.

use crate::poruke::{KatalogPoruka, vrijeme};
use crate::stanje::StanjePrikaza;
use chrono::Timelike;
use slojevi::Uredjivac;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Sučelje za odabir datoteka koje testovi mogu zamijeniti lažnim biračem.
pub trait BiracDatoteka: Send + Sync {
    fn otvori_datoteku(&self) -> Option<PathBuf>;
    fn spremi_datoteku(&self, predlozeno_ime: Option<&str>) -> Option<PathBuf>;
    /// KML ili KMZ iz Google Eartha ili GeoPackage, npr. iz QGIS-a.
    fn uvezi_datoteku(&self) -> Option<PathBuf>;
    /// Kamo ide izvoz u KML ili KMZ. Vrsta ovisi o nastavku koji korisnik odabere.
    fn izvezi_datoteku(&self, predlozeno_ime: &str) -> Option<PathBuf>;
}

/// Sustavski birač datoteka preko biblioteke `rfd`.
///
/// Na Linuxu su uključene mogućnosti `xdg-portal` i `wayland` jer koriste standardni XDG Desktop Portal
/// preko D-Busa za moderan odabir datoteka bez vezivanja za C biblioteke GTK3/GTK4, kompatibilno s Flatpakom,
/// Waylandom i svim radnim okruženjima (GNOME, KDE), uz `wayland` podršku za roditeljski prozor.
#[derive(Debug, Default)]
pub struct SustavskiBirac;

impl BiracDatoteka for SustavskiBirac {
    fn otvori_datoteku(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("GeoPackage", &["gpkg"])
            .pick_file()
    }

    fn spremi_datoteku(&self, predlozeno_ime: Option<&str>) -> Option<PathBuf> {
        let mut d = rfd::FileDialog::new().add_filter("GeoPackage", &["gpkg"]);
        if let Some(ime) = predlozeno_ime {
            d = d.set_file_name(ime);
        }
        d.save_file()
    }

    fn uvezi_datoteku(&self) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("KML, KMZ, GeoPackage", &["kml", "kmz", "gpkg"])
            .add_filter("Google Earth (KML, KMZ)", &["kml", "kmz"])
            .add_filter("GeoPackage", &["gpkg"])
            .pick_file()
    }

    fn izvezi_datoteku(&self, predlozeno_ime: &str) -> Option<PathBuf> {
        rfd::FileDialog::new()
            .add_filter("KML", &["kml"])
            .add_filter("KMZ", &["kmz"])
            .set_file_name(predlozeno_ime)
            .save_file()
    }
}

/// Lažni birač za automatske testove koji ne otvara stvarni prozor sustava.
#[derive(Clone, Default)]
pub struct LazniBirac {
    sljedeca_za_otvaranje: Arc<Mutex<Option<PathBuf>>>,
    sljedeca_za_spremanje: Arc<Mutex<Option<PathBuf>>>,
    sljedeca_za_uvoz: Arc<Mutex<Option<PathBuf>>>,
    sljedeca_za_izvoz: Arc<Mutex<Option<PathBuf>>>,
}

impl LazniBirac {
    #[must_use]
    pub fn novi() -> Self {
        Self::default()
    }

    pub fn postavi_otvaranje(&self, staza: impl Into<PathBuf>) {
        if let Ok(mut g) = self.sljedeca_za_otvaranje.lock() {
            *g = Some(staza.into());
        }
    }

    pub fn postavi_uvoz(&self, staza: impl Into<PathBuf>) {
        if let Ok(mut g) = self.sljedeca_za_uvoz.lock() {
            *g = Some(staza.into());
        }
    }

    pub fn postavi_izvoz(&self, staza: impl Into<PathBuf>) {
        if let Ok(mut g) = self.sljedeca_za_izvoz.lock() {
            *g = Some(staza.into());
        }
    }

    pub fn postavi_spremanje(&self, staza: impl Into<PathBuf>) {
        if let Ok(mut g) = self.sljedeca_za_spremanje.lock() {
            *g = Some(staza.into());
        }
    }
}

impl BiracDatoteka for LazniBirac {
    fn otvori_datoteku(&self) -> Option<PathBuf> {
        self.sljedeca_za_otvaranje
            .lock()
            .ok()
            .and_then(|mut g| g.take())
    }

    fn spremi_datoteku(&self, _predlozeno_ime: Option<&str>) -> Option<PathBuf> {
        self.sljedeca_za_spremanje
            .lock()
            .ok()
            .and_then(|mut g| g.take())
    }

    fn uvezi_datoteku(&self) -> Option<PathBuf> {
        self.sljedeca_za_uvoz.lock().ok().and_then(|mut g| g.take())
    }

    fn izvezi_datoteku(&self, _predlozeno_ime: &str) -> Option<PathBuf> {
        self.sljedeca_za_izvoz
            .lock()
            .ok()
            .and_then(|mut g| g.take())
    }
}

/// Podaci o trenutno otvorenoj datoteci projekta.
#[derive(Default)]
pub struct PodaciDatoteke {
    pub staza: Option<PathBuf>,
    pub zadnje_spremanje: Option<chrono::NaiveTime>,
    pub gpkg: Option<geopaket::Datoteka>,
}

impl PodaciDatoteke {
    #[must_use]
    pub fn nova() -> Self {
        Self::default()
    }

    pub fn ocisti(&mut self) {
        self.staza = None;
        self.zadnje_spremanje = None;
        self.gpkg = None;
    }
}

#[must_use]
pub fn trenutno_vrijeme() -> chrono::NaiveTime {
    chrono::Local::now().time()
}

/// Formatira tekst naziva datoteke i stanja spremanja za gornju traku.
#[must_use]
pub fn tekst_statusa_datoteke(
    stanje: &StanjePrikaza,
    datoteka: &PodaciDatoteke,
    uredjivac: &Uredjivac,
    katalog: &KatalogPoruka,
) -> Option<String> {
    if stanje.prikazi_primjer && datoteka.staza.is_none() {
        return Some(katalog.tekst("primjer-datoteka-status"));
    }

    if let Some(staza) = &datoteka.staza {
        let naziv = staza
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("projekt.gpkg");
        let izmijenjen = uredjivac.je_li_izmijenjen();
        return match (datoteka.zadnje_spremanje, izmijenjen) {
            (Some(v), false) => {
                let vrijeme_tekst = vrijeme(v.hour(), v.minute());
                Some(katalog.status_datoteke_spremljeno(naziv, &vrijeme_tekst))
            }
            (Some(v), true) => {
                let vrijeme_tekst = vrijeme(v.hour(), v.minute());
                Some(katalog.status_datoteke_spremljeno_izmijenjeno(naziv, &vrijeme_tekst))
            }
            (None, true) => Some(katalog.status_datoteke_izmijenjeno(naziv)),
            (None, false) => Some(naziv.to_string()),
        };
    }

    if uredjivac.je_li_izmijenjen() {
        Some(katalog.tekst("datoteka-novi-nespremljeno"))
    } else {
        None
    }
}

/// Stvara novi prazan projekt.
pub fn radnja_novi(
    uredjivac: &mut Uredjivac,
    datoteka: &mut PodaciDatoteke,
    stanje: &mut StanjePrikaza,
) {
    *uredjivac = Uredjivac::novi();
    stanje.odabrano = None;
    stanje.preimenovanje = None;
    stanje.prikazi_primjer = false;
    datoteka.ocisti();
}

/// Otvara postojeći projekt iz zadane staze. Vraća zabilježena upozorenja ili grešku.
pub fn radnja_otvori(
    uredjivac: &mut Uredjivac,
    datoteka: &mut PodaciDatoteke,
    stanje: &mut StanjePrikaza,
    katalog: &KatalogPoruka,
    staza: PathBuf,
) -> Result<Vec<String>, String> {
    let mut d =
        geopaket::Datoteka::otvori(&staza).map_err(|e| katalog.greska_otvaranja(&e.to_string()))?;
    let projekt = d
        .ucitaj()
        .map_err(|e| katalog.greska_otvaranja(&e.to_string()))?;
    let upozorenja = std::mem::take(&mut d.upozorenja);

    *uredjivac = Uredjivac::iz_projekta(projekt);
    uredjivac.oznaci_spremljeno();
    stanje.odabrano = None;
    stanje.preimenovanje = None;
    stanje.prikazi_primjer = false;
    datoteka.staza = Some(staza);
    datoteka.gpkg = Some(d);
    datoteka.zadnje_spremanje = Some(trenutno_vrijeme());

    Ok(upozorenja)
}

/// Zašto spremanje nije uspjelo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GreskaSpremanja {
    /// Datoteku je u međuvremenu promijenio drugi program, npr. QGIS. Prozor nudi Spremi kao
    /// (Odluke (9. 11. 2026.)).
    Sukob,
    /// Ostale greške, s tekstom za korisnika.
    Poruka(String),
}

impl GreskaSpremanja {
    fn iz(
        greska: &(dyn std::error::Error + Send + Sync + 'static),
        katalog: &KatalogPoruka,
    ) -> Self {
        if greska.is::<geopaket::DatotekaPromijenjena>() {
            Self::Sukob
        } else {
            Self::Poruka(katalog.greska_spremanja(greska))
        }
    }
}

/// Sprema projekt u postojeću datoteku. Vraća `Ok(true)` ako je spremljeno,
/// `Ok(false)` ako nema stazu pa treba "Spremi kao", ili grešku.
pub fn radnja_spremi(
    uredjivac: &mut Uredjivac,
    datoteka: &mut PodaciDatoteke,
    katalog: &KatalogPoruka,
) -> Result<bool, GreskaSpremanja> {
    let Some(staza) = &datoteka.staza else {
        return Ok(false);
    };

    if datoteka.gpkg.is_none() {
        let gpkg =
            geopaket::Datoteka::otvori(staza).map_err(|e| GreskaSpremanja::iz(&*e, katalog))?;
        datoteka.gpkg = Some(gpkg);
    }

    let Some(gpkg) = datoteka.gpkg.as_mut() else {
        return Ok(false);
    };

    gpkg.spremi(uredjivac.projekt())
        .map_err(|e| GreskaSpremanja::iz(&*e, katalog))?;
    uredjivac.oznaci_spremljeno();
    datoteka.zadnje_spremanje = Some(trenutno_vrijeme());
    Ok(true)
}

/// Sprema projekt u novu datoteku na zadanoj stazi.
pub fn radnja_spremi_kao(
    uredjivac: &mut Uredjivac,
    datoteka: &mut PodaciDatoteke,
    katalog: &KatalogPoruka,
    nova_staza: PathBuf,
) -> Result<(), String> {
    if let Some(roditelj) = nova_staza.parent() {
        let _ = std::fs::create_dir_all(roditelj);
    }
    if nova_staza.exists() {
        let _ = std::fs::remove_file(&nova_staza);
    }

    let nova_d = spremi_gpkg_na_stazu(datoteka.gpkg.as_mut(), &nova_staza, uredjivac, katalog)?;
    datoteka.gpkg = Some(nova_d);
    datoteka.staza = Some(nova_staza);
    datoteka.zadnje_spremanje = Some(trenutno_vrijeme());
    uredjivac.oznaci_spremljeno();
    Ok(())
}

fn spremi_gpkg_na_stazu(
    postojeci: Option<&mut geopaket::Datoteka>,
    nova_staza: &Path,
    uredjivac: &Uredjivac,
    katalog: &KatalogPoruka,
) -> Result<geopaket::Datoteka, String> {
    match postojeci {
        Some(gpkg) => spremi_kopiju_gpkg(gpkg, nova_staza, uredjivac, katalog),
        None => stvori_novi_gpkg(nova_staza, uredjivac, katalog),
    }
}

fn spremi_kopiju_gpkg(
    gpkg: &mut geopaket::Datoteka,
    nova_staza: &Path,
    uredjivac: &Uredjivac,
    katalog: &KatalogPoruka,
) -> Result<geopaket::Datoteka, String> {
    gpkg.spremi_kao(nova_staza, uredjivac.projekt())
        .map_err(|e| katalog.greska_spremanja(&*e))?;
    let mut nova_d =
        geopaket::Datoteka::otvori(nova_staza).map_err(|e| katalog.greska_spremanja(&*e))?;
    let _ = nova_d.ucitaj().map_err(|e| katalog.greska_spremanja(&*e))?;
    Ok(nova_d)
}

fn stvori_novi_gpkg(
    nova_staza: &Path,
    uredjivac: &Uredjivac,
    katalog: &KatalogPoruka,
) -> Result<geopaket::Datoteka, String> {
    let mut nova_d =
        geopaket::Datoteka::otvori(nova_staza).map_err(|e| katalog.greska_spremanja(&*e))?;
    nova_d
        .spremi(uredjivac.projekt())
        .map_err(|e| katalog.greska_spremanja(&*e))?;
    Ok(nova_d)
}

#[cfg(test)]
mod testovi {
    use super::*;
    use slojevi::{NaredbaStabla, VrstaGeometrije};

    #[test]
    fn lazni_birac_vraca_postavljene_staze() {
        let birac = LazniBirac::novi();
        assert_eq!(birac.otvori_datoteku(), None);
        assert_eq!(birac.spremi_datoteku(None), None);

        birac.postavi_otvaranje("a.gpkg");
        birac.postavi_spremanje("b.gpkg");

        assert_eq!(birac.otvori_datoteku(), Some(PathBuf::from("a.gpkg")));
        assert_eq!(birac.otvori_datoteku(), None);
        assert_eq!(birac.spremi_datoteku(None), Some(PathBuf::from("b.gpkg")));
        assert_eq!(birac.spremi_datoteku(None), None);
    }

    #[test]
    fn novi_otvori_spremi_i_spremi_kao_ciklusi() {
        let mapa = PathBuf::from(".quality/test_datoteka");
        let _ = std::fs::create_dir_all(&mapa);
        let staza_1 = mapa.join("prvi.gpkg");
        let staza_2 = mapa.join("drugi.gpkg");
        let _ = std::fs::remove_file(&staza_1);
        let _ = std::fs::remove_file(&staza_2);

        let katalog = KatalogPoruka::hrvatski();
        let mut uredjivac = Uredjivac::novi();
        let mut datoteka = PodaciDatoteke::nova();
        let mut stanje = StanjePrikaza::novo();

        // Dodaj sloj da imamo izmjenu
        uredjivac
            .izvrsi(NaredbaStabla::DodajSloj {
                naziv: "Granice".into(),
                vrsta: VrstaGeometrije::Linija,
                u: None,
            })
            .unwrap();
        assert!(uredjivac.je_li_izmijenjen());

        // Spremi bez staze vraća Ok(false)
        assert!(!radnja_spremi(&mut uredjivac, &mut datoteka, &katalog).unwrap());

        // Spremi kao na staza_1
        radnja_spremi_kao(&mut uredjivac, &mut datoteka, &katalog, staza_1.clone()).unwrap();
        assert!(!uredjivac.je_li_izmijenjen());
        assert!(staza_1.exists());
        assert_eq!(datoteka.staza.as_ref(), Some(&staza_1));

        // Spremi kao na staza_2
        radnja_spremi_kao(&mut uredjivac, &mut datoteka, &katalog, staza_2.clone()).unwrap();
        assert!(!uredjivac.je_li_izmijenjen());
        assert!(staza_2.exists());
        assert_eq!(datoteka.staza.as_ref(), Some(&staza_2));

        // Novi projekt čisti stanje
        radnja_novi(&mut uredjivac, &mut datoteka, &mut stanje);
        assert!(!uredjivac.je_li_izmijenjen());
        assert_eq!(datoteka.staza, None);
        assert_eq!(uredjivac.projekt().slojevi().count(), 0);

        // Otvaranje projekta sa staza_1
        let upozorenja = radnja_otvori(
            &mut uredjivac,
            &mut datoteka,
            &mut stanje,
            &katalog,
            staza_1,
        )
        .unwrap();
        assert!(upozorenja.is_empty());
        assert_eq!(uredjivac.projekt().slojevi().count(), 1);
        assert!(!uredjivac.je_li_izmijenjen());

        // Čišćenje
        let _ = std::fs::remove_dir_all(&mapa);
    }

    /// Odluke (9. 11. 2026.): nakon sukoba s QGIS-om crtež se sprema kao nova datoteka i
    /// rad se u njoj nastavlja.
    #[test]
    fn nakon_sukoba_spremi_kao_sprema_crtez_i_rad_se_nastavlja() {
        let mapa = PathBuf::from(".quality/test_sukob_spremi_kao");
        let _ = std::fs::remove_dir_all(&mapa);
        std::fs::create_dir_all(&mapa).unwrap();
        let izvorna = mapa.join("izvorna.gpkg");
        let nova = mapa.join("nova.gpkg");
        let katalog = KatalogPoruka::hrvatski();
        let mut uredjivac = Uredjivac::novi();
        let mut datoteka = PodaciDatoteke::nova();
        let sloj = match uredjivac
            .izvrsi(NaredbaStabla::DodajSloj {
                naziv: "Točke".into(),
                vrsta: VrstaGeometrije::Tocka,
                u: None,
            })
            .unwrap()
        {
            Some(slojevi::Novo::Sloj(s)) => s,
            _ => panic!("nema sloja"),
        };
        radnja_spremi_kao(&mut uredjivac, &mut datoteka, &katalog, izvorna.clone()).unwrap();

        uredjivac
            .izvrsi(slojevi::NaredbaObjekta::Dodaj {
                sloj,
                naziv: "Crtež".into(),
                geometrija: slojevi::Geometrija::Tocka(slojevi::Vrh::novi(18.7, 45.55)),
            })
            .unwrap();
        rusqlite::Connection::open(&izvorna)
            .unwrap()
            .execute_batch("CREATE TABLE iz_qgisa(v TEXT)")
            .unwrap();

        let greska = radnja_spremi(&mut uredjivac, &mut datoteka, &katalog).unwrap_err();
        assert_eq!(greska, GreskaSpremanja::Sukob);
        assert!(uredjivac.je_li_izmijenjen());

        radnja_spremi_kao(&mut uredjivac, &mut datoteka, &katalog, nova.clone()).unwrap();
        assert!(!uredjivac.je_li_izmijenjen());
        assert_eq!(datoteka.staza.as_ref(), Some(&nova));
        let procitaj = |staza: &Path| {
            let mut d = geopaket::Datoteka::otvori(staza).unwrap();
            let p = d.ucitaj().unwrap();
            p.sloj(sloj).unwrap().objekti.len()
        };
        assert_eq!(procitaj(&nova), 1);
        assert_eq!(procitaj(&izvorna), 0);

        uredjivac
            .izvrsi(slojevi::NaredbaObjekta::Dodaj {
                sloj,
                naziv: "Nastavak".into(),
                geometrija: slojevi::Geometrija::Tocka(slojevi::Vrh::novi(18.71, 45.55)),
            })
            .unwrap();
        assert!(radnja_spremi(&mut uredjivac, &mut datoteka, &katalog).unwrap());
        assert_eq!(procitaj(&nova), 2);
        let _ = std::fs::remove_dir_all(&mapa);
    }

    #[test]
    fn status_datoteke_provjerava_primjer_i_stazu() {
        let katalog = KatalogPoruka::hrvatski();
        let uredjivac = Uredjivac::novi();
        let mut datoteka = PodaciDatoteke::nova();
        let mut stanje = StanjePrikaza::novo();

        // 1. Prazan projekt bez primjera -> None
        stanje.prikazi_primjer = false;
        assert_eq!(
            tekst_statusa_datoteke(&stanje, &datoteka, &uredjivac, &katalog),
            None
        );

        // 2. Primjer bez staze -> primjer status
        stanje.prikazi_primjer = true;
        assert_eq!(
            tekst_statusa_datoteke(&stanje, &datoteka, &uredjivac, &katalog),
            Some(katalog.tekst("primjer-datoteka-status"))
        );

        // 3. Primjer s otvorenom pravom stazom -> prikazuje stazu, a ne status primjera
        datoteka.staza = Some(PathBuf::from("prava.gpkg"));
        let status = tekst_statusa_datoteke(&stanje, &datoteka, &uredjivac, &katalog);
        assert_ne!(status, Some(katalog.tekst("primjer-datoteka-status")));
        assert!(status.unwrap().contains("prava.gpkg"));
    }
}
