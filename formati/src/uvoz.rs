//! Pročitani KML postaje mape, slojevi i objekti projekta. Svaka mapa Google Eartha ostaje mapa, a
//! njezini objekti idu u slojeve Točke, Linije i Poligoni (Odluke (10. 11. 2026.)). Objekt s više
//! geometrija, npr. linija i točka natpisa, postaje po jedan objekt u svakom sloju, s istim nazivom.

use crate::model::{DokumentKml, GeometrijaKml, Gubici, MapaKml, ObjektKml, StilKml};
use slojevi::{
    Boja, Geometrija, Greska, IdMape, IdObjekta, IdSloja, IzmjenaObjekta, NaredbaObjekta,
    NaredbaStabla, Novo, Skupina, Stil, StilObjekta, Uredjivac, VrstaGeometrije,
};

/// Nazivi koje bira sučelje, da ovise o jeziku programa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NaziviSlojeva {
    pub tocke: String,
    pub linije: String,
    pub poligoni: String,
    /// Za mapu kojoj KML ne daje naziv.
    pub bez_naziva: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SazetakUvoza {
    /// Nova mapa u koju je uvezeno sve.
    pub mapa: IdMape,
    pub mapa_ukupno: usize,
    pub slojeva: usize,
    pub objekata: usize,
    /// Geometrije koje je projekt odbio, npr. lik koji nije ispravan.
    pub odbijeno: usize,
    pub gubici: Gubici,
}

/// Uvozi sve u novu mapu na vrhu stabla, kao jedan korak poništavanja.
pub fn uvezi(
    uredjivac: &mut Uredjivac,
    dokument: &DokumentKml,
    naziv_datoteke: &str,
    nazivi: &NaziviSlojeva,
) -> Result<SazetakUvoza, Greska> {
    let korijen = &dokument.korijen;
    let naziv = ime_mape(&korijen.naziv, naziv_datoteke, nazivi);
    uredjivac.skupina(|g| {
        let mut sazetak = SazetakUvoza {
            mapa: IdMape(0),
            mapa_ukupno: 0,
            slojeva: 0,
            objekata: 0,
            odbijeno: 0,
            gubici: dokument.gubici.clone(),
        };
        let mut stog: Vec<(Option<IdMape>, &MapaKml, String)> = vec![(None, korijen, naziv)];
        while let Some((roditelj, mapa, naziv)) = stog.pop() {
            let id = nova_mapa(g, roditelj, naziv)?;
            if roditelj.is_none() {
                sazetak.mapa = id;
            }
            sazetak.mapa_ukupno += 1;
            uvezi_slojeve(g, id, mapa, nazivi, &mut sazetak)?;
            for pod in mapa.mape.iter().rev() {
                let ime = ime_mape(&pod.naziv, "", nazivi);
                stog.push((Some(id), pod, ime));
            }
        }
        Ok(sazetak)
    })
}

fn ime_mape(naziv: &str, zamjena: &str, nazivi: &NaziviSlojeva) -> String {
    [naziv.trim(), zamjena.trim()]
        .into_iter()
        .find(|n| !n.is_empty())
        .map_or_else(|| nazivi.bez_naziva.clone(), str::to_owned)
}

/// Mapa s nazivom uvijek nastaje, pa greška ovdje znači grešku u programu.
fn nova_mapa(g: &mut Skupina<'_>, u: Option<IdMape>, naziv: String) -> Result<IdMape, Greska> {
    let novo = g.izvrsi(NaredbaStabla::DodajMapu { u, naziv });
    novo.ok().and_then(id_mape).ok_or(Greska::NepoznataMapa)
}

fn id_mape(novo: Option<Novo>) -> Option<IdMape> {
    match novo {
        Some(Novo::Mapa(id)) => Some(id),
        _ => None,
    }
}

fn id_sloja(novo: Option<Novo>) -> Option<IdSloja> {
    match novo {
        Some(Novo::Sloj(id)) => Some(id),
        _ => None,
    }
}

fn id_objekta(novo: Option<Novo>) -> Option<IdObjekta> {
    match novo {
        Some(Novo::Objekt(id)) => Some(id),
        _ => None,
    }
}

fn uvezi_slojeve(
    g: &mut Skupina<'_>,
    mapa: IdMape,
    izvor: &MapaKml,
    nazivi: &NaziviSlojeva,
    sazetak: &mut SazetakUvoza,
) -> Result<(), Greska> {
    let vrste = [
        (VrstaGeometrije::Tocka, &nazivi.tocke),
        (VrstaGeometrije::Linija, &nazivi.linije),
        (VrstaGeometrije::Poligon, &nazivi.poligoni),
    ];
    for (vrsta, naziv) in vrste {
        let dijelovi = dijelovi_vrste(&izvor.objekti, vrsta);
        if !dijelovi.is_empty() {
            uvezi_sloj(g, mapa, vrsta, naziv, &dijelovi, sazetak)?;
        }
    }
    Ok(())
}

/// Svaka geometrija tražene vrste, uz objekt iz kojeg dolazi.
fn dijelovi_vrste(objekti: &[ObjektKml], vrsta: VrstaGeometrije) -> Vec<(&ObjektKml, Geometrija)> {
    objekti
        .iter()
        .flat_map(|o| o.geometrije.iter().map(move |g| (o, g)))
        .map(|(o, g)| (o, u_geometriju(g)))
        .filter(|(_, g)| g.vrsta() == vrsta)
        .collect()
}

fn u_geometriju(g: &GeometrijaKml) -> Geometrija {
    match g {
        GeometrijaKml::Tocka(v) => Geometrija::Tocka(*v),
        GeometrijaKml::Linija(v) => Geometrija::Linija(v.clone()),
        GeometrijaKml::Poligon(p) => Geometrija::Poligon(p.clone()),
    }
}

fn uvezi_sloj(
    g: &mut Skupina<'_>,
    mapa: IdMape,
    vrsta: VrstaGeometrije,
    naziv: &str,
    dijelovi: &[(&ObjektKml, Geometrija)],
    sazetak: &mut SazetakUvoza,
) -> Result<(), Greska> {
    let novo = g.izvrsi(NaredbaStabla::DodajSloj {
        u: Some(mapa),
        naziv: naziv.to_owned(),
        vrsta,
    });
    let sloj = novo.ok().and_then(id_sloja).ok_or(Greska::NepoznatSloj)?;
    sazetak.slojeva += 1;
    let stilovi: Vec<Stil> = dijelovi
        .iter()
        .map(|(o, _)| stil_za(&o.stil, vrsta))
        .collect();
    let stil_sloja = najcesci(&stilovi);
    // Novi sloj nije zaključan, pa mu se stil uvijek može postaviti.
    let _ = g.izvrsi(NaredbaStabla::PostaviStil {
        sloj,
        stil: stil_sloja,
    });
    for ((objekt, geometrija), stil) in dijelovi.iter().zip(&stilovi) {
        let razlika = razlika(stil, &stil_sloja);
        match dodaj_objekt(g, sloj, objekt, geometrija.clone(), razlika) {
            Ok(()) => sazetak.objekata += 1,
            Err(_) => sazetak.odbijeno += 1,
        }
    }
    Ok(())
}

fn dodaj_objekt(
    g: &mut Skupina<'_>,
    sloj: IdSloja,
    objekt: &ObjektKml,
    geometrija: Geometrija,
    stil: StilObjekta,
) -> Result<(), Greska> {
    let novo = g.izvrsi(NaredbaObjekta::Dodaj {
        sloj,
        naziv: objekt.naziv.clone(),
        geometrija,
    })?;
    let id = id_objekta(novo).ok_or(Greska::NepoznatObjekt)?;
    // Objekt je upravo dodan u sloj koji nije zaključan, pa izmjene prolaze.
    for izmjena in izmjene_objekta(objekt, stil) {
        let _ = g.izvrsi(NaredbaObjekta::Izmijeni {
            sloj,
            objekt: id,
            izmjena,
        });
    }
    Ok(())
}

/// Opis i stil objekta, samo ako ih ima.
fn izmjene_objekta(objekt: &ObjektKml, stil: StilObjekta) -> Vec<IzmjenaObjekta> {
    let opis = (!objekt.opis.is_empty()).then(|| IzmjenaObjekta::Opis(objekt.opis.clone()));
    let stil = (stil != StilObjekta::default()).then_some(IzmjenaObjekta::Stil(stil));
    opis.into_iter().chain(stil).collect()
}

/// KML širinu crte zadaje u pikselima zaslona, a projekt u milimetrima, na 96 točaka po inču.
fn mm_iz_piksela(px: f64) -> f64 {
    (px * 25.4 / 96.0).clamp(0.1, 5.0)
}

/// Google Earth bijelom bojom ikone označava „bez bojanja”, pa bijela točka ostaje zadane boje.
const fn bijela(b: Boja) -> bool {
    b.r == 255 && b.g == 255 && b.b == 255
}

/// Stil kojim bi se objekt crtao. Što KML ne zadaje, ostaje kao u zadanom stilu.
fn stil_za(kml: &StilKml, vrsta: VrstaGeometrije) -> Stil {
    let mut s = Stil::default();
    if vrsta == VrstaGeometrije::Tocka {
        if let Some(b) = kml.boja_ikone.filter(|b| !bijela(*b)) {
            s.boja_crte = b;
        }
        if let Some(m) = kml.mjerilo_ikone {
            s.velicina_znaka_mm = (3.0 * m).clamp(1.0, 10.0);
        }
        return s;
    }
    if let Some(b) = kml.boja_crte {
        s.boja_crte = b;
    }
    if let Some(px) = kml.sirina_px {
        s.debljina_mm = mm_iz_piksela(px);
    }
    if vrsta == VrstaGeometrije::Poligon {
        ispuna(&mut s, kml);
    }
    s
}

fn ispuna(s: &mut Stil, kml: &StilKml) {
    if let Some(b) = kml.boja_ispune {
        s.boja_ispune = b;
    }
    if kml.bez_ispune {
        s.boja_ispune.a = 0;
    }
    if kml.bez_ruba {
        s.boja_crte.a = 0;
    }
}

/// Stil koji ima najviše objekata postaje stil sloja. Kod jednakog broja vrijedi prvi.
fn najcesci(stilovi: &[Stil]) -> Stil {
    let mut brojevi: Vec<(Stil, usize)> = Vec::new();
    for s in stilovi {
        match brojevi.iter_mut().find(|(b, _)| b == s) {
            Some((_, n)) => *n += 1,
            None => brojevi.push((*s, 1)),
        }
    }
    let mut najveci = (Stil::default(), 0);
    for (s, n) in brojevi {
        if n > najveci.1 {
            najveci = (s, n);
        }
    }
    najveci.0
}

/// Objekt pamti samo ono u čemu se razlikuje od sloja.
fn razlika(stil: &Stil, sloj: &Stil) -> StilObjekta {
    let ako = |isto: bool| !isto;
    StilObjekta {
        boja_crte: ako(stil.boja_crte == sloj.boja_crte).then_some(stil.boja_crte),
        debljina_mm: ako(stil.debljina_mm == sloj.debljina_mm).then_some(stil.debljina_mm),
        boja_ispune: ako(stil.boja_ispune == sloj.boja_ispune).then_some(stil.boja_ispune),
        oblik_znaka: None,
        velicina_znaka_mm: ako(stil.velicina_znaka_mm == sloj.velicina_znaka_mm)
            .then_some(stil.velicina_znaka_mm),
    }
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn stil_tocke_linije_i_lika() {
        let crvena = Boja {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        };
        let kml = StilKml {
            boja_crte: Some(crvena),
            sirina_px: Some(96.0 / 25.4),
            boja_ispune: Some(crvena),
            bez_ispune: true,
            bez_ruba: false,
            boja_ikone: Some(crvena),
            mjerilo_ikone: Some(2.0),
        };
        let t = stil_za(&kml, VrstaGeometrije::Tocka);
        assert_eq!(t.boja_crte, crvena);
        assert_eq!(t.velicina_znaka_mm, 6.0);
        assert_eq!(t.debljina_mm, Stil::default().debljina_mm);
        let l = stil_za(&kml, VrstaGeometrije::Linija);
        assert!((l.debljina_mm - 1.0).abs() < 1e-9);
        assert_eq!(l.boja_ispune, Stil::default().boja_ispune);
        let p = stil_za(&kml, VrstaGeometrije::Poligon);
        assert_eq!(p.boja_ispune.a, 0, "fill 0 je bez ispune");
        assert_eq!(p.boja_crte.a, 255);
        let bez_ruba = StilKml {
            bez_ruba: true,
            ..StilKml::default()
        };
        assert_eq!(stil_za(&bez_ruba, VrstaGeometrije::Poligon).boja_crte.a, 0);
        let bijela_ikona = StilKml {
            boja_ikone: Some(Boja {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            }),
            ..StilKml::default()
        };
        assert_eq!(
            stil_za(&bijela_ikona, VrstaGeometrije::Tocka),
            Stil::default()
        );
        assert_eq!(mm_iz_piksela(100.0), 5.0);
        assert_eq!(mm_iz_piksela(0.0), 0.1);
    }

    #[test]
    fn najcesci_stil_i_razlika() {
        let a = Stil::default();
        let b = Stil {
            debljina_mm: 2.0,
            ..Stil::default()
        };
        assert_eq!(najcesci(&[b, a, b]), b);
        assert_eq!(najcesci(&[a, b]), a, "kod jednakog broja vrijedi prvi");
        assert_eq!(najcesci(&[]), Stil::default());
        let r = razlika(&b, &a);
        assert_eq!(r.debljina_mm, Some(2.0));
        assert_eq!(r.boja_crte, None);
        assert_eq!(razlika(&a, &a), StilObjekta::default());
        let c = Stil {
            velicina_znaka_mm: 5.0,
            boja_ispune: b.boja_crte,
            ..Stil::default()
        };
        let r = razlika(&c, &a);
        assert_eq!(r.velicina_znaka_mm, Some(5.0));
        assert!(r.boja_ispune.is_some());
    }

    #[test]
    fn neispravna_geometrija_se_broji_a_ostalo_ulazi() {
        let v = slojevi::Vrh::novi(18.6, 45.5);
        let objekt = |g| ObjektKml {
            naziv: "O".into(),
            opis: "opis".into(),
            geometrije: vec![g],
            ..ObjektKml::default()
        };
        let mut d = DokumentKml::default();
        d.korijen.objekti = vec![
            objekt(GeometrijaKml::Linija(vec![v])),
            objekt(GeometrijaKml::Linija(vec![
                v,
                slojevi::Vrh::novi(18.7, 45.6),
            ])),
        ];
        let nazivi = NaziviSlojeva {
            tocke: "T".into(),
            linije: "L".into(),
            poligoni: "P".into(),
            bez_naziva: "B".into(),
        };
        let mut u = Uredjivac::novi();
        let s = uvezi(&mut u, &d, "datoteka", &nazivi).expect("uvoz");
        assert_eq!((s.objekata, s.odbijeno, s.slojeva), (1, 1, 1));
        let (_, sloj) = u.projekt().slojevi().next().expect("sloj");
        let o = sloj.objekti.values().next().expect("objekt");
        assert_eq!(o.opis, "opis");
        assert_eq!(u.projekt().mapa(s.mapa).expect("mapa").naziv, "datoteka");
    }

    #[test]
    fn novi_id_samo_od_prave_vrste() {
        let m = Some(Novo::Mapa(IdMape(1)));
        let s = Some(Novo::Sloj(IdSloja(2)));
        let o = Some(Novo::Objekt(IdObjekta(3)));
        assert_eq!(id_mape(m), Some(IdMape(1)));
        assert_eq!(id_mape(s), None);
        assert_eq!(id_sloja(s), Some(IdSloja(2)));
        assert_eq!(id_sloja(o), None);
        assert_eq!(id_objekta(o), Some(IdObjekta(3)));
        assert_eq!(id_objekta(None), None);
    }

    #[test]
    fn ime_mape_ima_zamjenu() {
        let n = NaziviSlojeva {
            tocke: "T".into(),
            linije: "L".into(),
            poligoni: "P".into(),
            bez_naziva: "Bez naziva".into(),
        };
        assert_eq!(ime_mape(" Parcele ", "x", &n), "Parcele");
        assert_eq!(ime_mape("", "myplaces", &n), "myplaces");
        assert_eq!(ime_mape(" ", "", &n), "Bez naziva");
    }
}
