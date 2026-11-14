use super::*;
use std::path::PathBuf;

fn podatak(ime: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/podaci/shp")
        .join(ime)
}

fn citaj(ime: &str, s: Option<Sustav>) -> Result<DokumentKml, GreskaKml> {
    crate::procitaj_u_sustavu(&podatak(ime), s)
}

#[test]
fn kanali_u_htrs96_s_kodnom_stranicom_1250_i_krivim_cpg() {
    let d = citaj("kanali.shp", None).unwrap();
    assert_eq!(d.korijen.naziv, "kanali");
    let o = &d.korijen.objekti;
    assert_eq!(o.len(), 2);
    assert_eq!(o[0].naziv, "Kanal Čađavica-Šž");
    assert_eq!(o[0].opis, "Izmišljeni kanal");
    let GeometrijaKml::Linija(v) = &o[0].geometrije[0] else {
        panic!("linija");
    };
    assert!(
        (v[0].duzina - 18.6).abs() < 1e-6 && (v[0].sirina - 45.5).abs() < 1e-6,
        "{v:?}"
    );
    assert_eq!(o[1].geometrije.len(), 2, "dva dijela linije");
    assert_eq!(o[1].opis, "");
    assert_eq!(
        d.gubici.izostavljena_svojstva.iter().collect::<Vec<_>>(),
        ["DULJINA"]
    );
}

#[test]
fn likovi_s_rupom_visinom_i_vise_dijelova() {
    let d = citaj("obuhvati.shp", None).unwrap();
    let o = &d.korijen.objekti;
    let GeometrijaKml::Poligon(p) = &o[0].geometrije[0] else {
        panic!("lik");
    };
    assert_eq!(p.len(), 2, "vanjski rub i rupa");
    assert_eq!(p[0].len(), 4);
    assert!(p[0].iter().any(|v| v.visina == Some(91.0)));
    assert_eq!(o[1].geometrije.len(), 2, "dvije retencije");
}

#[test]
fn bez_prj_trazi_sustav_a_sa_zadanim_cita() {
    let Err(GreskaKml::BezSustava { istok, sjever }) = citaj("crpke.shp", None) else {
        panic!("treba pitati za sustav");
    };
    assert_eq!(prijedlog_sustava(istok, sjever), Sustav::HdksZona6);
    let d = citaj("crpke.shp", Some(Sustav::HdksZona6)).unwrap();
    let o = &d.korijen.objekti;
    assert_eq!(o.len(), 2);
    assert_eq!(o[0].naziv, "Crpka 1");
    let GeometrijaKml::Tocka(v) = o[0].geometrije[0] else {
        panic!("točka");
    };
    assert!(
        (v.duzina - 18.61).abs() < 1e-4 && (v.sirina - 45.51).abs() < 1e-4,
        "{v:?}"
    );
    assert_eq!(d.gubici.bez_geometrije, 1);
    // Krivi sustav daje koordinate izvan Zemlje, a one se broje, ne crtaju.
    let krivo = citaj("crpke.shp", Some(Sustav::Wgs84)).unwrap();
    assert!(krivo.korijen.objekti.is_empty());
    assert_eq!(krivo.gubici.neispravne_koordinate, 2);
}

#[test]
fn skupina_tocaka_postaje_vise_tocaka() {
    let d = citaj("skupine.shp", None).unwrap();
    assert_eq!(d.korijen.objekti[0].naziv, "Skupina");
    assert_eq!(d.korijen.objekti[0].geometrije.len(), 3);
}

#[test]
fn sustavi_iz_prj_i_prijedlozi() {
    let wkt = |e: &str| std::fs::read_to_string(podatak(&format!("{e}.prj"))).unwrap();
    assert_eq!(sustav_iz_prj(&wkt("kanali")), Some(Sustav::Htrs96Tm));
    assert_eq!(sustav_iz_prj(&wkt("skupine")), Some(Sustav::Wgs84));
    let hdks = |z: u8| format!("PROJCS[\"MGI_1901_Balkans_zone_{z}\",GEOGCS[\"GCS_MGI_1901\"]]");
    assert_eq!(sustav_iz_prj(&hdks(5)), Some(Sustav::HdksZona5));
    assert_eq!(sustav_iz_prj(&hdks(6)), Some(Sustav::HdksZona6));
    let mercator = "PROJCS[\"WGS_1984_Web_Mercator_Auxiliary_Sphere\",GEOGCS[\"GCS_WGS_1984\"]]";
    assert_eq!(sustav_iz_prj(mercator), Some(Sustav::WebMercator));
    assert_eq!(sustav_iz_prj("PROJCS[\"ETRS89_UTM_33N\"]"), None);
    assert_eq!(prijedlog_sustava(18, 45), Sustav::Wgs84);
    assert_eq!(prijedlog_sustava(5_500_000, 5_000_000), Sustav::HdksZona5);
    assert_eq!(prijedlog_sustava(650_000, 5_040_000), Sustav::Htrs96Tm);
    assert_eq!(prijedlog_sustava(18, 4_000), Sustav::Htrs96Tm);
}

#[test]
fn pokvareni_zapisi_i_rubni_slucajevi() {
    // Zapis kraći od najave ili nepoznate vrste ne ruši čitanje.
    let mut b = std::fs::read(podatak("skupine.shp")).unwrap();
    b.truncate(120);
    assert_eq!(zapisi(&b), vec![None]);
    let nepoznat = [31_i32.to_le_bytes().as_slice(), &[0; 20]].concat();
    assert!(zapis(&nepoznat).is_none());
    assert!(zapis(&[]).is_none());
    // Prsten bez vanjskog ruba postaje svoj lik.
    let rupa = vec![
        Vrh::novi(0.0, 0.0),
        Vrh::novi(1.0, 0.0),
        Vrh::novi(1.0, 1.0),
    ];
    assert_eq!(likovi(vec![rupa.clone()]).len(), 1);
    let mut vanjski = rupa.clone();
    vanjski.reverse();
    let daleko = vec![
        Vrh::novi(5.0, 5.0),
        Vrh::novi(6.0, 5.0),
        Vrh::novi(6.0, 6.0),
    ];
    assert_eq!(
        likovi(vec![vanjski, daleko]).len(),
        2,
        "rupa izvan svih rubova"
    );
    assert!(i32_le(&(-1_i32).to_le_bytes(), 0).is_none());
}

#[test]
fn skraceni_zapisi_svake_vrste_nisu_objekt() {
    for vrsta in [1_i32, 3, 5, 8, 13, 25] {
        let c = vrsta.to_le_bytes();
        assert!(zapis(&c).is_none(), "{vrsta}");
    }
    // Linija s najavljenim dijelom kojeg nema.
    let mut c = 3_i32.to_le_bytes().to_vec();
    c.extend([0; 32]);
    c.extend(2_i32.to_le_bytes());
    c.extend(1_i32.to_le_bytes());
    assert!(zapis(&c).is_none());
    assert!(f64_le(&[0; 7], 0).is_none());
    assert!(i32_be(&[0; 3], 0).is_none());
}

#[test]
fn dbf_kratak_ili_skracen() {
    assert!(dbf::procitaj(b"x").is_none());
    let mut b = std::fs::read(podatak("kanali.dbf")).unwrap();
    b.truncate(b.len() - 20);
    let t = dbf::procitaj(&b).unwrap();
    assert_eq!(t.redci.len(), 2);
    assert!(t.redci[1].is_empty(), "drugi redak je odrezan");
    assert!(t.vrijednost(5, &["NAZIV"]).is_none());
    assert_eq!(dbf::dekodiraj(b"A\x8a"), "AŠ");
}
