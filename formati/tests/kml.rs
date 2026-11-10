//! Čitanje i uvoz izmišljenih KML i KMZ datoteka iz probe (`probe/kml/primjeri`).

use formati::{DokumentKml, GeometrijaKml, GreskaKml, MapaKml, NaziviSlojeva, procitaj, uvezi};
use slojevi::{Projekt, Stavka, Uredjivac, VrstaGeometrije};
use std::path::PathBuf;

fn podatak(ime: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/podaci")
        .join(ime)
}

fn nazivi() -> NaziviSlojeva {
    NaziviSlojeva {
        tocke: "Točke".into(),
        linije: "Linije".into(),
        poligoni: "Poligoni".into(),
        bez_naziva: "Mapa bez naziva".into(),
    }
}

fn sve_mape(m: &MapaKml) -> Vec<&MapaKml> {
    let mut izlaz = vec![m];
    for pod in &m.mape {
        izlaz.extend(sve_mape(pod));
    }
    izlaz
}

fn dubina(m: &MapaKml) -> usize {
    1 + m.mape.iter().map(dubina).max().unwrap_or(0)
}

/// Mape i slojevi projekta kao redovi `dubina naziv [vrsta broj]`, redom stabla.
fn stablo(p: &Projekt) -> Vec<String> {
    fn obidji(p: &Projekt, stavke: &[Stavka], d: usize, izlaz: &mut Vec<String>) {
        for s in stavke {
            match *s {
                Stavka::Mapa(id) => {
                    let m = p.mapa(id).expect("mapa");
                    izlaz.push(format!("{d} {}", m.naziv));
                    obidji(p, &m.djeca, d + 1, izlaz);
                }
                Stavka::Sloj(id) => {
                    let s = p.sloj(id).expect("sloj");
                    izlaz.push(format!("{d} {} {:?} {}", s.naziv, s.vrsta, s.objekti.len()));
                }
            }
        }
    }
    let mut izlaz = Vec::new();
    obidji(p, p.korijen(), 0, &mut izlaz);
    izlaz
}

#[test]
fn mijesane_mape_dobivaju_sloj_za_svaku_vrstu() {
    let d = procitaj(&podatak("mijesani_slojevi.kml")).expect("čitanje");
    let mut u = Uredjivac::novi();
    let s = uvezi(&mut u, &d, "mijesani_slojevi", &nazivi()).expect("uvoz");
    let redovi = stablo(u.projekt());
    assert_eq!(redovi[0], "0 Ispit podjele na slojeve");
    assert_eq!(s.slojeva, 6, "{redovi:#?}");
    assert_eq!(s.mapa_ukupno, 5);
    let vrste: Vec<_> = u.projekt().slojevi().map(|(_, s)| s.vrsta).collect();
    assert_eq!(
        vrste
            .iter()
            .filter(|v| **v == VrstaGeometrije::Tocka)
            .count(),
        3
    );
    assert_eq!(
        vrste
            .iter()
            .filter(|v| **v == VrstaGeometrije::Linija)
            .count(),
        2
    );
    assert!(redovi.contains(&"1 Mapa 1: samo točke".to_owned()));
    assert!(redovi.contains(&"2 Točke Tocka 3".to_owned()));

    // Cijeli uvoz je jedan korak poništavanja.
    assert_eq!(u.ponisti(), Ok(true));
    assert!(u.projekt().korijen().is_empty());
    assert_eq!(u.ponisti(), Ok(false));
}

#[test]
fn sve_znacajke_google_eartha() {
    let d = procitaj(&podatak("sve_znacajke.kml")).expect("čitanje");
    let k = &d.korijen;
    assert_eq!(k.naziv, "Ispitno područje Čepin – Đakovo – Županja");
    assert!(k.opis.contains("<b>Glavni dokument</b>"), "CDATA opis mape");
    let mape = sve_mape(k);
    let objekti: Vec<_> = mape.iter().flat_map(|m| &m.objekti).collect();
    assert_eq!(objekti.len(), 6);
    assert_eq!(d.gubici.preskoceno.get("NetworkLink"), Some(&2));
    assert!(d.gubici.preskoceno.contains_key("GroundOverlay"));

    let tocka = objekti
        .iter()
        .find(|o| o.naziv.starts_with("Čvorište"))
        .expect("točka");
    assert_eq!(tocka.naziv, "Čvorište Šećerana – čćđšž");
    assert!(tocka.opis.contains("<b>uredno</b>"));
    let GeometrijaKml::Tocka(v) = tocka.geometrije[0] else {
        panic!("točka");
    };
    assert_eq!(v.visina, Some(95.4), "visina ostaje");
    // StyleMap stoji prije stila, a boja ikone je aabbggrr.
    let boja = tocka.stil.boja_ikone.expect("boja ikone");
    assert_eq!((boja.r, boja.g, boja.b), (0xff, 0x88, 0x00));
    assert_eq!(tocka.stil.mjerilo_ikone, Some(1.2));

    let rupa = objekti
        .iter()
        .flat_map(|o| &o.geometrije)
        .find_map(|g| match g {
            GeometrijaKml::Poligon(p) if p.len() > 2 => Some(p.len()),
            _ => None,
        });
    assert_eq!(
        rupa,
        Some(4),
        "dva prstena u istom innerBoundaryIs su dvije rupe"
    );

    let mut u = Uredjivac::novi();
    let s = uvezi(&mut u, &d, "sve_znacajke", &nazivi()).expect("uvoz");
    assert!(s.objekata >= 6, "linija i točka natpisa su dva objekta");
    assert_eq!(s.odbijeno, 0);
}

#[test]
fn duboke_mape_ostaju_mape() {
    let d = procitaj(&podatak("duboke_mape.kml")).expect("čitanje");
    assert!(dubina(&d.korijen) >= 15);
    let mut u = Uredjivac::novi();
    let s = uvezi(&mut u, &d, "duboke", &nazivi()).expect("uvoz");
    assert_eq!(s.objekata, d.korijen.broj_objekata());
}

#[test]
fn kmz_cita_doc_kml() {
    let d = procitaj(&podatak("arhiva_sa_slikama.kmz")).expect("čitanje");
    assert_eq!(d.korijen.broj_objekata(), 1);
    assert_eq!(d.gubici.preskoceno.get("GroundOverlay"), Some(&1));
}

#[test]
fn nepouzdane_datoteke_se_uredno_odbijaju() {
    assert!(matches!(
        procitaj(&podatak("ostecen.kml")),
        Err(GreskaKml::NeispravanXml { .. })
    ));
    assert_eq!(
        procitaj(&podatak("zip_bomba.kmz")),
        Err(GreskaKml::PrevelikaArhiva)
    );
    assert_eq!(
        procitaj(&podatak("neocekivana_putanja.kmz")),
        Err(GreskaKml::NedopustenaPutanja)
    );
    assert_eq!(procitaj(&podatak("nema.kml")), Err(GreskaKml::Citanje));

    // Prazna točka nije greška cijele datoteke: objekt bez geometrije se preskače i broji.
    let d = procitaj(&podatak("prazna_geometrija.kml")).expect("čitanje");
    assert_eq!(d.korijen.broj_objekata(), 0);
    assert_eq!(d.gubici.prazne_geometrije, 1);
    assert_eq!(d.gubici.bez_geometrije, 1);
    assert_eq!(
        d,
        procitaj(&podatak("prazna_geometrija.kml")).expect("isto")
    );
    let _ = DokumentKml::default();
}
