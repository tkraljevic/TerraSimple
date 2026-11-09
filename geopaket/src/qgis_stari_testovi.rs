use super::*;
use crate::qgis_projekt::{NAZIV_PROJEKTA, spremi, zapakiraj_qgz};
use rusqlite::types::Value;
use slojevi::{Boja, IdSloja, NaredbaStabla, Projekt, Stavka, Stil, Uredjivac, VrstaGeometrije};

const QGS: &str = "<qgis><projectlayers>\
<maplayer type=\"vector\"><extent><id>nije oznaka</id></extent>\
<id>TerraSimple_a&amp;b_1</id><datasource>./stari.gpkg|layername=a&amp;b</datasource>\
<layername>Stari naziv</layername><renderer-v2 type=\"categorizedSymbol\"/></maplayer>\
<maplayer type=\"raster\"><id>osm</id><datasource>xyz</datasource><layername>OSM</layername></maplayer>\
<maplayer type=\"vector\"><id>bez_naziva</id><datasource>x</datasource><layername/></maplayer>\
<maplayer><datasource>x</datasource><layername>bez oznake</layername></maplayer>\
<maplayer><id>bez_staze</id><layername>n</layername></maplayer>\
<maplayer><id>krivo&nepoznato;</id><datasource>x</datasource><layername>n</layername></maplayer>\
</projectlayers></qgis>";

fn heks(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn baza_s_projektom(sadrzaj: Value) -> Connection {
    let c = Connection::open_in_memory().unwrap();
    c.execute_batch(
        "CREATE TABLE qgis_projects(name TEXT PRIMARY KEY, metadata BLOB, content BLOB)",
    )
    .unwrap();
    c.execute(
        "INSERT INTO qgis_projects VALUES(?1, '', ?2)",
        rusqlite::params![NAZIV_PROJEKTA, sadrzaj],
    )
    .unwrap();
    c
}

fn zip_s(qgs: &str) -> Vec<u8> {
    zapakiraj_qgz(qgs.as_bytes()).unwrap()
}

/// Zip s više datoteka, kakav QGIS spremi kad uz projekt ima i pomoćnu `.qgd` datoteku. Svaki
/// ulaz ima i dodatno polje i opis, koje čitanje mora preskočiti.
fn zip_s_ulazima(ulazi: &[(&str, &[u8])]) -> Vec<u8> {
    let mut z = Vec::new();
    let mut imenik = Vec::new();
    for (ime, sadrzaj) in ulazi {
        let jedan = zapakiraj_qgz(sadrzaj).unwrap();
        let kraj = kraj_imenika(&jedan).unwrap();
        let ulaz_od = le(&kraj[16..20]);
        let sazeto = &jedan[30 + 15..ulaz_od];
        let pomak = u32::try_from(z.len()).unwrap().to_le_bytes();
        let duljina = u16::try_from(ime.len()).unwrap().to_le_bytes();
        let mut lokalni = jedan[..30].to_vec();
        lokalni[26..28].copy_from_slice(&duljina);
        lokalni[28..30].copy_from_slice(&[3, 0]);
        z.extend_from_slice(&lokalni);
        z.extend_from_slice(ime.as_bytes());
        z.extend_from_slice(b"lok");
        z.extend_from_slice(sazeto);
        let mut u = jedan[ulaz_od..ulaz_od + 46].to_vec();
        u[28..30].copy_from_slice(&duljina);
        u[30..32].copy_from_slice(&[3, 0]);
        u[32..34].copy_from_slice(&[4, 0]);
        u[42..46].copy_from_slice(&pomak);
        imenik.extend_from_slice(&u);
        imenik.extend_from_slice(ime.as_bytes());
        imenik.extend_from_slice(b"dodopis");
    }
    let imenik_od = u32::try_from(z.len()).unwrap().to_le_bytes();
    let broj = u16::try_from(ulazi.len()).unwrap().to_le_bytes();
    z.extend_from_slice(&imenik);
    z.extend_from_slice(b"PK\x05\x06\0\0\0\0");
    z.extend_from_slice(&broj);
    z.extend_from_slice(&broj);
    z.extend_from_slice(&u32::try_from(imenik.len()).unwrap().to_le_bytes());
    z.extend_from_slice(&imenik_od);
    z.extend_from_slice(&[0, 0]);
    z
}

#[test]
fn cita_slojeve_iz_projekta_u_paketu() {
    let s = slojevi(
        &baza_s_projektom(Value::Text(heks(&zip_s(QGS)))),
        NAZIV_PROJEKTA,
    );
    assert_eq!(
        s.keys().collect::<Vec<_>>(),
        ["TerraSimple_a&b_1", "osm"],
        "čuva se samo sloj s oznakom, stazom i nazivom; <id> unutar <extent> nije oznaka sloja"
    );
}

#[test]
fn projekt_se_trazi_i_iza_druge_datoteke_u_zipu() {
    let zip = zip_s_ulazima(&[("projekt.qgd", b"pomocno"), ("projekt.qgs", QGS.as_bytes())]);
    assert_eq!(qgs_iz_zipa(&zip).as_deref(), Some(QGS));
    let bez_projekta = zip_s_ulazima(&[("a.qgd", b"x"), ("b.txt", b"y")]);
    assert_eq!(qgs_iz_zipa(&bez_projekta), None);
}

#[test]
fn prilagodi_mijenja_samo_naziv_i_stazu() {
    let s = slojevi_iz_qgs(QGS);
    let sloj = &s["TerraSimple_a&b_1"];
    assert_eq!(
        sloj.prilagodi("./nova.gpkg|layername=a&amp;b", "Novi &amp; naziv"),
        "<maplayer type=\"vector\"><extent><id>nije oznaka</id></extent>\
<id>TerraSimple_a&amp;b_1</id><datasource>./nova.gpkg|layername=a&amp;b</datasource>\
<layername>Novi &amp; naziv</layername><renderer-v2 type=\"categorizedSymbol\"/></maplayer>"
    );
    assert_eq!(
        s["osm"].prilagodi("I", "N"),
        "<maplayer type=\"raster\"><id>osm</id><datasource>I</datasource><layername>N</layername></maplayer>"
    );
}

#[test]
fn ugnijezdeni_maplayer_i_drugi_projekt_se_ne_citaju() {
    let qgs = "<qgis><maplayer><id>vanjski</id><datasource>a</datasource><layername>b</layername>\
<x><maplayer><id>unutarnji</id></maplayer></x></maplayer>\
<maplayer><id>vanjski</id><datasource>drugi</datasource><layername>b</layername></maplayer></qgis>";
    let s = slojevi_iz_qgs(qgs);
    assert_eq!(s.keys().collect::<Vec<_>>(), ["vanjski"]);
    assert!(s["vanjski"].xml.contains("<datasource>a<"), "vrijedi prvi");
    let c = baza_s_projektom(Value::Text(heks(&zip_s(QGS))));
    assert!(slojevi(&c, "DrugiProjekt").is_empty());
}

/// Svako pokvareno mjesto u zipu daje „nema starog projekta”, bez pada.
#[test]
fn neispravan_zip_ne_daje_projekt() {
    let zip = zip_s(QGS);
    let kraj = zip.len() - 22;
    let imenik = le(&zip[kraj + 16..kraj + 20]);
    let pokvari = |i: usize, v: u8| {
        let mut z = zip.clone();
        z[i] = v;
        z
    };
    let mut kriv_sadrzaj = zip.clone();
    kriv_sadrzaj[30 + 15] = 0xff;
    kriv_sadrzaj[30 + 16] = 0xff;
    for z in [
        zip[..21].to_vec(),
        pokvari(kraj + 16, 0xff),
        pokvari(imenik, b'X'),
        pokvari(imenik + 28, 200),
        pokvari(0, b'X'),
        pokvari(imenik + 10, 0),
        pokvari(imenik + 23, 0x7f),
        kriv_sadrzaj,
        zapakiraj_qgz(&[0xff, 0xfe]).unwrap(),
    ] {
        assert_eq!(qgs_iz_zipa(&z), None);
    }
    assert_eq!(qgs_iz_zipa(&zip).as_deref(), Some(QGS));
}

#[test]
fn neispravan_sadrzaj_u_tablici_ne_daje_slojeve() {
    for sadrzaj in [
        Value::Text("abc".into()),
        Value::Null,
        Value::Blob(zip_s(QGS)),
        Value::Text(heks(&zip_s("<qgis><maplayer><id>x</id"))),
    ] {
        assert!(slojevi(&baza_s_projektom(sadrzaj), NAZIV_PROJEKTA).is_empty());
    }
    assert!(slojevi(&Connection::open_in_memory().unwrap(), NAZIV_PROJEKTA).is_empty());
}

#[test]
fn heks_i_brojevi_u_zipu() {
    assert_eq!(iz_heksa(b"00ff7A"), Some(vec![0, 255, 122]));
    assert_eq!(iz_heksa(b"0"), None);
    assert_eq!(iz_heksa(b"g0"), None);
    assert_eq!(iz_heksa(b"0g"), None);
    assert_eq!(le(&[0x34, 0x12, 0x78, 0x56]), 0x5678_1234);
    assert_eq!(kraj_imenika(&[0; 21]), None);
    let mut z = vec![0; 30];
    z[3..7].copy_from_slice(KRAJ);
    z[8..12].copy_from_slice(KRAJ);
    z[20] = 9;
    assert_eq!(
        kraj_imenika(&z).map(|k| k[12]),
        Some(9),
        "traži se zadnji kraj imenika"
    );
}

#[test]
fn velika_datoteka_se_reze() {
    assert_eq!(NAJVISE_BAJTOVA, 67_108_864, "64 MiB");
    let mut xml = String::from("<qgis>");
    xml.push_str(&" ".repeat(usize::try_from(NAJVISE_BAJTOVA).unwrap()));
    xml.push_str("</qgis>");
    let rezano = qgs_iz_zipa(&zip_s(&xml)).unwrap();
    assert_eq!(rezano.len(), usize::try_from(NAJVISE_BAJTOVA).unwrap());
}

fn sloj(u: &mut Uredjivac, naziv: &str, vrsta: VrstaGeometrije) -> IdSloja {
    u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: naziv.into(),
        vrsta,
    })
    .unwrap();
    u.projekt().slojevi().map(|(id, _)| id).max().unwrap()
}

/// Ono što QGIS napravi kad korisnik namjesti natpise i spremi projekt.
fn qgis_dodaje_natpise(c: &Connection) {
    let qgs = qgs_iz_zipa(&zip_projekta(c, NAZIV_PROJEKTA).unwrap())
        .unwrap()
        .replace("</maplayer>", "<labeling type=\"simple\"/></maplayer>");
    c.execute(
        "UPDATE qgis_projects SET content=?1 WHERE name=?2",
        rusqlite::params![heks(&zip_s(&qgs)), NAZIV_PROJEKTA],
    )
    .unwrap();
}

/// Odluke (9. 11. 2026.): stil iz QGIS-a ostaje dok ga korisnik ne promijeni u TerraSimpleu.
#[test]
fn stil_iz_qgisa_ostaje_dok_ga_terrasimple_ne_promijeni() {
    let c = Connection::open_in_memory().unwrap();
    let mut u = Uredjivac::novi();
    let tocke = sloj(&mut u, "Točke", VrstaGeometrije::Tocka);
    let linije = sloj(&mut u, "Linije", VrstaGeometrije::Linija);
    spremi(&c, &Projekt::default(), u.projekt()).unwrap();
    let ucitan = u.projekt().clone();
    qgis_dodaje_natpise(&c);

    u.izvrsi(NaredbaStabla::Preimenuj {
        stavka: Stavka::Sloj(tocke),
        naziv: "Točke & bunari".into(),
    })
    .unwrap();
    u.izvrsi(NaredbaStabla::PostaviStil {
        sloj: linije,
        stil: Stil {
            boja_crte: Boja {
                r: 200,
                g: 0,
                b: 0,
                a: 255,
            },
            ..Stil::default()
        },
    })
    .unwrap();
    let novi = sloj(&mut u, "Novi", VrstaGeometrije::Poligon);
    spremi(&c, &ucitan, u.projekt()).unwrap();

    let s = slojevi(&c, NAZIV_PROJEKTA);
    let xml = |id: IdSloja| s[&format!("TerraSimple_sloj_{0}_{0}", id.0)].xml.clone();
    let t = xml(tocke);
    assert!(t.contains("<labeling type=\"simple\"/>"), "{t}");
    assert!(
        t.contains("<layername>Točke &amp; bunari</layername>"),
        "{t}"
    );
    assert!(
        t.contains(&format!(
            "<datasource>./projekt.gpkg|layername=sloj_{}</datasource>",
            tocke.0
        )),
        "{t}"
    );
    assert!(
        !xml(linije).contains("<labeling"),
        "promijenjen stil zamjenjuje QGIS-ov"
    );
    assert!(xml(linije).contains("200,0,0,255"), "{}", xml(linije));
    assert!(!xml(novi).contains("<labeling"));
}
