use super::*;

fn kml(tijelo: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><kml xmlns="http://www.opengis.net/kml/2.2" xmlns:gx="http://www.google.com/kml/ext/2.2"><Document>{tijelo}</Document></kml>"#
    )
}

fn citaj(tijelo: &str) -> DokumentKml {
    procitaj_tekst(&kml(tijelo)).expect("čitanje")
}

fn objekti(d: &DokumentKml) -> Vec<&ObjektKml> {
    let mut izlaz = Vec::new();
    let mut stog = vec![&d.korijen];
    while let Some(m) = stog.pop() {
        izlaz.extend(&m.objekti);
        stog.extend(&m.mape);
    }
    izlaz
}

const TOCKA: &str = "<Point><coordinates>18.6,45.5</coordinates></Point>";

#[test]
fn opis_cuva_html_i_entitete() {
    let d = citaj(&format!(
        "<name>A &amp; B</name><Placemark><name>T &#269;&#x0107; &nbsp;</name>\
         <description>Prije <b>masno <i>kurziv</i></b> &lt;x&gt;</description>{TOCKA}</Placemark>"
    ));
    assert_eq!(d.korijen.naziv, "A & B");
    let o = objekti(&d)[0];
    assert_eq!(o.naziv, "T čć &nbsp;", "nepoznat entitet ostaje kako piše");
    assert_eq!(o.opis, "Prije <b>masno <i>kurziv</i></b> <x>");
}

#[test]
fn prazne_oznake_i_preskocene_oznake() {
    let d = citaj(&format!(
        "<Folder/><Placemark><name/><Snippet>s<name>krivo</name></Snippet>\
         <LookAt><name>krivo</name></LookAt>{TOCKA}</Placemark>\
         <NetworkLink><name>veza</name></NetworkLink><GroundOverlay/>\
         <Placemark><Point><coordinates/></Point></Placemark>"
    ));
    assert_eq!(d.korijen.mape.len(), 1, "prazna mapa ostaje mapa");
    let o = objekti(&d);
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].naziv, "");
    assert_eq!(d.gubici.preskoceno.get("NetworkLink"), Some(&1));
    assert_eq!(d.gubici.preskoceno.get("GroundOverlay"), Some(&1));
    assert_eq!(d.gubici.prazne_geometrije, 1);
    assert_eq!(d.gubici.bez_geometrije, 1);
}

#[test]
fn linija_trag_i_samostalni_prsten() {
    let d = citaj(
        "<Placemark><MultiGeometry>\
         <LineString><coordinates>18.6,45.5 18.7,45.6</coordinates></LineString>\
         <LineString><coordinates>18.6,45.5</coordinates></LineString>\
         <LinearRing><coordinates>18.6,45.5 18.7,45.6 18.8,45.5 18.6,45.5</coordinates></LinearRing>\
         <gx:Track><gx:coord>18.6 45.5 10</gx:coord><gx:coord>18.7 45.6 11</gx:coord></gx:Track>\
         <gx:Track><gx:coord>18.6 x</gx:coord><gx:coord>18.7 45.6</gx:coord></gx:Track>\
         <gx:Track><gx:coord>18.6 45.5</gx:coord></gx:Track>\
         </MultiGeometry></Placemark><gx:coord>18 45</gx:coord>",
    );
    let o = objekti(&d);
    let linije = o[0]
        .geometrije
        .iter()
        .filter(|g| matches!(g, GeometrijaKml::Linija(_)))
        .count();
    assert_eq!(linije, 3, "linija, prsten kao crta i ispravan trag");
    assert_eq!(
        d.gubici.prazne_geometrije, 2,
        "linija i trag s jednom točkom"
    );
    assert_eq!(d.gubici.neispravne_koordinate, 1, "pokvaren trag");
}

#[test]
fn poligon_s_rupama_bez_vanjskog_i_pokvaren() {
    let prsten = |k: &str| format!("<LinearRing><coordinates>{k}</coordinates></LinearRing>");
    let v = prsten("18.6,45.5 18.7,45.5 18.7,45.6 18.6,45.5");
    let d = citaj(&format!(
        "<Placemark><Polygon><outerBoundaryIs>{v}</outerBoundaryIs>\
         <innerBoundaryIs>{v}{}</innerBoundaryIs></Polygon></Placemark>\
         <Placemark><Polygon><innerBoundaryIs>{v}</innerBoundaryIs></Polygon></Placemark>\
         <Placemark><Polygon><outerBoundaryIs>{}</outerBoundaryIs></Polygon></Placemark>",
        prsten("18.6,45.5 18.7,45.5"),
        prsten("18.6,x 18.7,45.5 18.7,45.6"),
    ));
    let o = objekti(&d);
    assert_eq!(o.len(), 1);
    let GeometrijaKml::Poligon(p) = &o[0].geometrije[0] else {
        panic!("poligon");
    };
    assert_eq!(p.len(), 2, "vanjski rub i jedna ispravna rupa");
    assert_eq!(p[0].len(), 3, "bez ponovljene zadnje točke");
    assert_eq!(d.gubici.prazne_geometrije, 2, "kratka rupa i lik bez ruba");
    assert_eq!(d.gubici.neispravne_koordinate, 1);
    assert_eq!(d.gubici.bez_geometrije, 2);
}

#[test]
fn stilovi_po_dokumentu_i_vlastiti_stil() {
    let d = citaj(&format!(
        r##"<Style id="a&amp;b"><LineStyle><color>ff0000ff</color><width>4</width></LineStyle>
        <PolyStyle><color>800000ff</color><fill>0</fill><outline>0</outline></PolyStyle>
        <IconStyle><color>ff00ff00</color><scale>2</scale></IconStyle></Style>
        <StyleMap id="m"><Pair><key>highlight</key><styleUrl>#nema</styleUrl></Pair>
        <Pair><key>normal</key><styleUrl>datoteka.kml#a&amp;b</styleUrl></Pair></StyleMap>
        <StyleMap id="petlja"><Pair><key>normal</key><styleUrl>#petlja</styleUrl></Pair></StyleMap>
        <StyleMap id="bez"/>
        <Placemark><name>1</name><styleUrl>#m</styleUrl>{TOCKA}</Placemark>
        <Placemark><name>2</name><styleUrl>#petlja</styleUrl>{TOCKA}</Placemark>
        <Placemark><name>3</name><styleUrl>#nepoznat</styleUrl>{TOCKA}</Placemark>
        <Placemark><name>4</name><styleUrl>#m</styleUrl>
          <Style><LineStyle><width>9</width><color>krivo</color></LineStyle></Style>{TOCKA}</Placemark>
        <Placemark><name>5</name><StyleMap><Pair><key>normal</key><styleUrl>#m</styleUrl></Pair></StyleMap>{TOCKA}</Placemark>
        <Document><Style id="a&amp;b"><LineStyle><width>1</width></LineStyle></Style>
          <Placemark><name>6</name><styleUrl>#a&amp;b</styleUrl>{TOCKA}</Placemark>
          <Placemark><name>7</name><styleUrl>#m</styleUrl>{TOCKA}</Placemark>
        </Document>"##
    ));
    let po_nazivu = |n: &str| {
        objekti(&d)
            .into_iter()
            .find(|o| o.naziv == n)
            .expect("objekt")
            .stil
    };
    let prvi = po_nazivu("1");
    assert_eq!(prvi.boja_crte.map(|b| b.r), Some(255));
    assert_eq!(prvi.sirina_px, Some(4.0));
    assert_eq!(prvi.boja_ispune.map(|b| b.a), Some(0x80));
    assert!(prvi.bez_ispune && prvi.bez_ruba);
    assert_eq!(prvi.boja_ikone.map(|b| b.g), Some(255));
    assert_eq!(prvi.mjerilo_ikone, Some(2.0));
    assert_eq!(po_nazivu("2"), StilKml::default(), "petlja nema stila");
    assert_eq!(po_nazivu("3"), StilKml::default());
    let cetvrti = po_nazivu("4");
    assert_eq!(cetvrti.sirina_px, Some(9.0), "vlastiti stil ima prednost");
    assert_eq!(
        cetvrti.boja_crte.map(|b| b.r),
        Some(255),
        "a ostalo od zajedničkog"
    );
    assert_eq!(po_nazivu("5").sirina_px, Some(4.0), "StyleMap u objektu");
    assert_eq!(
        po_nazivu("6").sirina_px,
        Some(1.0),
        "stil iz istog dokumenta"
    );
    assert_eq!(
        po_nazivu("7").sirina_px,
        Some(4.0),
        "stil iz vanjskog dokumenta"
    );
}

#[test]
fn mape_bez_korijenskog_dokumenta_i_nezatvorene() {
    let d = procitaj_tekst(&format!(
        "<kml><Folder><name>F</name><Placemark>{TOCKA}</Placemark><Folder><name>G</name>"
    ))
    .expect("nezatvoreno na kraju nije greška");
    assert_eq!(d.korijen.mape[0].naziv, "F");
    assert_eq!(d.korijen.mape[0].mape[0].naziv, "G");
    let d = procitaj_tekst("<kml/>").expect("prazno");
    assert_eq!(d, DokumentKml::default());
}

#[test]
fn greske_datoteke() {
    let duboko = "<a>".repeat(NAJVECA_DUBINA + 1);
    assert_eq!(procitaj_tekst(&duboko), Err(GreskaKml::PreDuboko));
    let duboko = format!("{}<b/>", "<a>".repeat(NAJVECA_DUBINA));
    assert_eq!(procitaj_tekst(&duboko), Err(GreskaKml::PreDuboko));
    assert!(matches!(
        procitaj_tekst("<kml><Document></Folder>"),
        Err(GreskaKml::NeispravanXml { .. })
    ));
    assert!(matches!(
        procitaj_tekst("<kml><NetworkLink><a></b></NetworkLink>"),
        Err(GreskaKml::NeispravanXml { polozaj }) if polozaj > 0
    ));
    for g in [
        GreskaKml::Citanje,
        GreskaKml::NeispravanXml { polozaj: 7 },
        GreskaKml::PreDuboko,
        GreskaKml::NeispravnaArhiva,
        GreskaKml::NemaKmlUArhivi,
        GreskaKml::NedopustenaPutanja,
        GreskaKml::PrevelikaArhiva,
    ] {
        assert!(!g.to_string().is_empty());
    }
    assert_eq!(
        GreskaKml::NeispravanXml { polozaj: 7 }.to_string(),
        "neispravan XML na bajtu 7"
    );
    assert_eq!(GreskaKml::Citanje.uz(5), GreskaKml::Citanje);
}

#[test]
fn naziv_izvan_mape_i_objekta_se_ne_pamti() {
    let d = citaj(&format!(
        "<Placemark><Point><name>krivo</name><coordinates>18.6,45.5</coordinates></Point></Placemark>\
         <Style><LineStyle><name>x</name></LineStyle></Style><color>ff000000</color>{TOCKA}\
         <Style id='z'><color>ff000000</color></Style>\
         <Placemark><MultiGeometry><coordinates>18,45</coordinates></MultiGeometry></Placemark>"
    ));
    assert_eq!(objekti(&d)[0].naziv, "");
    assert_eq!(
        d.gubici.prazne_geometrije, 0,
        "točka izvan objekta se ne broji"
    );
}

#[test]
fn svaka_greska_ima_opis_a_pukla_datoteka_je_greska_citanja() {
    let greske = [
        GreskaKml::Citanje,
        GreskaKml::NeispravanXml { polozaj: 7 },
        GreskaKml::PreDuboko,
        GreskaKml::NeispravnaArhiva,
        GreskaKml::NemaKmlUArhivi,
        GreskaKml::NedopustenaPutanja,
        GreskaKml::PrevelikaArhiva,
        GreskaKml::NeispravanJson,
        GreskaKml::NepoznatSustav,
        GreskaKml::BezSustava {
            istok: 1,
            sjever: 2,
        },
    ];
    let mut opisi: Vec<String> = greske.iter().map(ToString::to_string).collect();
    assert!(opisi.iter().all(|o| !o.is_empty()));
    opisi.dedup();
    assert_eq!(opisi.len(), greske.len());
    assert_eq!(GreskaKml::Citanje.opis_arhive(), None);

    struct Pukla;
    impl std::io::Read for Pukla {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("pukla"))
        }
    }
    assert_eq!(procitaj_json(Pukla, None).unwrap_err(), GreskaKml::Citanje);
    assert!(je_json(b"\xEF\xBB\xBF {"));
    assert!(!je_json(b"<kml"));
}
