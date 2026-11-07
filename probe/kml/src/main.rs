//! Proba čitanja KML i KMZ datoteka (`probe/kml`).
//!
//! Naredbe:
//! - `proba_kml izradi-primjere <mapa>` — stvara sve izmišljene primjere u zadanoj mapi.
//! - `proba_kml provjeri-primjere <mapa>` — pokreće provjere nad izmišljenim primjerima i provjeru padova.
//! - `proba_kml usporedi <datoteka> <referenca.csv> <oznaka>` — uspoređuje naš čitač s Python i GDAL referencom.
//! - `proba_kml mjeri <datoteka> <oznaka>` — mjeri vrijeme čitanja i vršnu memoriju našeg čitača u zasebnom procesu.
//! - `proba_kml mjeri-kml-crate <datoteka> <oznaka>` — mjeri vrijeme i vršnu memoriju biblioteke `kml` 0.15 u zasebnom procesu.

mod memorija;

use std::collections::HashMap;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use proba_kml::citac::procitaj_datoteku;
use proba_kml::primjeri::{
    izradi_sve_primjere, provjeri_da_kvarovi_padaju, provjeri_sve_primjere,
};
use proba_kml::slojevi::podijeli_na_slojeve;
use proba_kml::usporedba_kml::procitaj_s_kml_crateom;

fn velicina_mb(put: &Path) -> f64 {
    std::fs::metadata(put)
        .map(|m| m.len() as f64 / 1e6)
        .unwrap_or(0.0)
}

fn formatiraj_mapu_gubitaka(mapa: & std::collections::BTreeMap<String, usize>) -> String {
    if mapa.is_empty() {
        return "nema (0)".to_string();
    }
    mapa.iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn naredba_usporedi(put: &Path, ref_put: &Path, oznaka: &str) -> Result<u32, String> {
    let tekst = std::fs::read_to_string(ref_put).map_err(|_| "ne mogu pročitati referencu")?;
    if tekst.starts_with("neotvoriva;") {
        return match procitaj_datoteku(put) {
            Err(g) => {
                println!("  u redu  {oznaka}: neispravna datoteka uredno odbijena ({g})");
                Ok(0)
            }
            Ok(_) => {
                println!(
                    "  PALO    {oznaka}: neispravna datoteka je prihvaćena, a morala je pasti"
                );
                Ok(1)
            }
        };
    }

    let mut ref_vrijednosti: HashMap<String, usize> = HashMap::new();
    for redak in tekst.lines() {
        let mut dijelovi = redak.splitn(2, ';');
        if let (Some(k), Some(v)) = (dijelovi.next(), dijelovi.next()) {
            if let Ok(broj) = v.trim().parse::<usize>() {
                ref_vrijednosti.insert(k.trim().to_string(), broj);
            }
        }
    }

    let stablo = procitaj_datoteku(put).map_err(|g| format!("{oznaka}: {g}"))?;
    let s = &stablo.statistika;
    let slojevi = podijeli_na_slojeve(&stablo);

    let mut palo = 0u32;
    let mut provjeri_broj = |kljuc: &str, nas_broj: usize| {
        if let Some(&ocekivano) = ref_vrijednosti.get(kljuc) {
            if nas_broj != ocekivano {
                println!(
                    "  PALO    {oznaka} [{kljuc}]: naš čitač={nas_broj}, referenca={ocekivano}"
                );
                palo += 1;
            }
        } else {
            println!("  PALO    {oznaka}: nedostaje ključ {kljuc} u referenci");
            palo += 1;
        }
    };

    provjeri_broj("dokumenti", s.dokumenti);
    provjeri_broj("mape", s.mape);
    provjeri_broj("najveca_dubina_mapa", s.najveca_dubina_mapa);
    provjeri_broj("najveca_dubina_spremnika", s.najveca_dubina_spremnika);
    provjeri_broj("objekti", s.objekti);
    provjeri_broj("objekti_s_nazivom", s.objekti_s_nazivom);
    provjeri_broj("objekti_s_opisom", s.objekti_s_opisom);
    provjeri_broj("mape_s_opisom", s.mape_s_opisom);
    provjeri_broj("tocke", s.tocke);
    provjeri_broj("linije", s.linije);
    provjeri_broj("poligoni", s.poligoni);
    provjeri_broj("rupe_poligona", s.rupe_poligona);
    provjeri_broj("tragovi", s.tragovi);
    provjeri_broj("ukupno_vrhova", s.ukupno_vrhova);
    provjeri_broj("vrhovi_s_nenultom_visinom", s.vrhovi_s_nenultom_visinom);
    provjeri_broj("stilovi", s.stilovi);
    provjeri_broj("mape_stilova", s.mape_stilova);
    provjeri_broj("mrezne_veze", s.mrezne_veze);
    provjeri_broj("slikovni_prekrivaci", s.slikovni_prekrivaci);
    provjeri_broj("pogledi_kamere", s.pogledi_kamere);
    provjeri_broj("vremenski_zapisi", s.vremenski_zapisi);
    provjeri_broj("prosireni_podaci", s.prosireni_podaci);
    provjeri_broj("slike_u_kmz", s.slike_u_kmz);

    if s.nerijeseni_stilovi_objekata > 0 {
        println!(
            "  PALO    {oznaka}: neriješeni stilovi objekata={}",
            s.nerijeseni_stilovi_objekata
        );
        palo += 1;
    }

    // Usporedba s GDAL/ogrinfo (LIBKML)
    let ogr_otvara = ref_vrijednosti.get("ogr_otvara").copied().unwrap_or(0);
    let ogr_slojevi = ref_vrijednosti.get("ogr_slojevi").copied().unwrap_or(0);
    let ogr_objekti = ref_vrijednosti.get("ogr_objekti").copied().unwrap_or(0);
    let ogr_rupe = ref_vrijednosti.get("ogr_rupe").copied().unwrap_or(0);
    let unutarnje_granice = ref_vrijednosti
        .get("unutarnje_granice_tag")
        .copied()
        .unwrap_or(0);

    if ogr_otvara == 1 {
        let ocekivano_ogr_obj = s.objekti + s.slikovni_prekrivaci;
        if ocekivano_ogr_obj != ogr_objekti {
            println!(
                "  PALO    {oznaka} [ogr_objekti]: naš čitač (Placemark+GroundOverlay)={ocekivano_ogr_obj}, ogrinfo={ogr_objekti}"
            );
            palo += 1;
        }
        if ogr_rupe != unutarnje_granice {
            println!(
                "  PALO    {oznaka} [ogr_rupe]: neočekivan broj rupa u ogrinfo={ogr_rupe}, innerBoundaryIs={unutarnje_granice}"
            );
            palo += 1;
        }
    }

    let status = if palo == 0 { "u redu" } else { "PALO  " };
    println!(
        "  {status}  {oznaka:<24} {:>6.2} MB | dok {:>2}, mape {:>3} (dubina {:>2}/{:>2}) | obj {:>6} (naziv {:>5}, opis {:>5}) | T {:>4} L {:>4} P {:>5} (rupe {:>4}, ogr {:>4}) trag {:>2} | vrhova {:>7} (3D!=0: {:>6}) | stil {:>3}+{:>3} (riješeno {:>5})",
        velicina_mb(put),
        s.dokumenti,
        s.mape,
        s.najveca_dubina_mapa,
        s.najveca_dubina_spremnika,
        s.objekti,
        s.objekti_s_nazivom,
        s.objekti_s_opisom,
        s.tocke,
        s.linije,
        s.poligoni,
        s.rupe_poligona,
        ogr_rupe,
        s.tragovi,
        s.ukupno_vrhova,
        s.vrhovi_s_nenultom_visinom,
        s.stilovi,
        s.mape_stilova,
        s.rijeseni_stilovi_objekata,
    );
    println!(
        "          slojevi: spremnika {:>3} (nepraznih {:>3}, miješanih {:>2}, miješani MG {:>4}, ogrinfo slojeva {:>3}) -> ukupno {:>3} slojeva (T:{:>3}, L:{:>3}, P:{:>3})",
        slojevi.ukupno_spremnika,
        slojevi.nepraznih_spremnika,
        slojevi.mijesanih_spremnika,
        slojevi.mijesanih_visegeometrija,
        ogr_slojevi,
        slojevi.ukupno_slojeva,
        slojevi.slojeva_tocaka,
        slojevi.slojeva_linija,
        slojevi.slojeva_poligona,
    );
    println!(
        "          ne-2D elementi: {} | nepoznate oznake: {}",
        formatiraj_mapu_gubitaka(&stablo.izgubljeno_za_2d),
        formatiraj_mapu_gubitaka(&stablo.nepoznati_elementi),
    );

    if let Ok(kml_stat) = procitaj_s_kml_crateom(put) {
        let gub_vrhova = s.ukupno_vrhova.saturating_sub(kml_stat.ukupno_vrhova);
        let gub_stil = s.stilovi.saturating_sub(kml_stat.stilovi);
        let gub_opis_mapa = s.mape_s_opisom.saturating_sub(kml_stat.mape_s_opisom);
        let gub_mv = s.mrezne_veze.saturating_sub(kml_stat.mrezne_veze);
        let gub_sp = s
            .slikovni_prekrivaci
            .saturating_sub(kml_stat.slikovni_prekrivaci);
        let gub_trag = kml_stat
            .objekti_bez_geometrije
            .saturating_sub(s.objekti_bez_geometrije);
        if gub_vrhova > 0
            || gub_stil > 0
            || gub_opis_mapa > 0
            || gub_mv > 0
            || gub_sp > 0
            || gub_trag > 0
        {
            println!(
                "          usporedba kml 0.15: TIHO GUBI vrhova={gub_vrhova}, stilova={gub_stil}, CDATA_opisa_mapa={gub_opis_mapa}, NetworkLink={gub_mv}, GroundOverlay={gub_sp}, gx:Track_objekata={gub_trag}"
            );
        }
    }

    Ok(palo)
}

fn naredba_mjeri(put: &Path, oznaka: &str) -> Result<(), String> {
    let mb = velicina_mb(put);
    let t0 = Instant::now();
    let stablo = procitaj_datoteku(put).map_err(|g| format!("{oznaka}: {g}"))?;
    let sek = t0.elapsed().as_secs_f64();
    let vrh_mb = memorija::vrh_memorije_mb();
    let s = &stablo.statistika;
    let slojevi = podijeli_na_slojeve(&stablo);
    println!(
        "  naš čitač (quick-xml) | {oznaka:<18} | {:>6.2} MB | vrijeme {:>6.3} s ({:>6.1} ms) | vrh memorije {:>6.1} MB | obj {:>6}, vrhova {:>7}, slojeva {:>3}",
        mb,
        sek,
        sek * 1000.0,
        vrh_mb,
        s.objekti,
        s.ukupno_vrhova,
        slojevi.ukupno_slojeva,
    );
    Ok(())
}

fn naredba_mjeri_kml_crate(put: &Path, oznaka: &str) -> Result<(), String> {
    let mb = velicina_mb(put);
    let t0 = Instant::now();
    let stat = procitaj_s_kml_crateom(put)
        .map_err(|vrsta| format!("{oznaka}: kml 0.15 greška ({vrsta})"))?;
    let sek = t0.elapsed().as_secs_f64();
    let vrh_mb = memorija::vrh_memorije_mb();
    println!(
        "  kml 0.15 iz plana     | {oznaka:<18} | {:>6.2} MB | vrijeme {:>6.3} s ({:>6.1} ms) | vrh memorije {:>6.1} MB | obj {:>6}, vrhova {:>7} (tiho gubi vrhove, stilove, CDATA opise mapa i gx:Track)",
        mb,
        sek,
        sek * 1000.0,
        vrh_mb,
        stat.objekti,
        stat.ukupno_vrhova,
    );
    Ok(())
}

fn pokreni() -> Result<u32, String> {
    let arg: Vec<String> = std::env::args().collect();
    let Some(naredba) = arg.get(1).map(String::as_str) else {
        return Err("upotreba: proba_kml <izradi-primjere|provjeri-primjere|usporedi|mjeri|mjeri-kml-crate> ...".to_string());
    };
    match naredba {
        "izradi-primjere" => {
            let mapa = arg.get(2).ok_or("nedostaje putanja mape za primjere")?;
            izradi_sve_primjere(Path::new(mapa))?;
            println!("  izrađeno 8 izmišljenih primjera u {mapa}");
            Ok(0)
        }
        "provjeri-primjere" => {
            let mapa = arg.get(2).ok_or("nedostaje putanja mape s primjerima")?;
            let broj = provjeri_sve_primjere(Path::new(mapa))?;
            println!("  u redu  svih {broj} izmišljenih primjera prošlo provjeru");
            let padovi = provjeri_da_kvarovi_padaju()?;
            println!(
                "  u redu  svih {} namjerno pokvarenih provjera očekivano je palo:",
                padovi.len()
            );
            for (opis, razlog) in padovi {
                println!("          - {opis} -> uhvaćeno: {razlog}");
            }
            Ok(0)
        }
        "usporedi" => {
            let put = arg.get(2).ok_or("nedostaje putanja datoteke")?;
            let ref_put = arg.get(3).ok_or("nedostaje putanja reference")?;
            let oznaka = arg.get(4).map(String::as_str).unwrap_or("datoteka");
            naredba_usporedi(Path::new(put), Path::new(ref_put), oznaka)
        }
        "mjeri" => {
            let put = arg.get(2).ok_or("nedostaje putanja datoteke")?;
            let oznaka = arg.get(3).map(String::as_str).unwrap_or("datoteka");
            naredba_mjeri(Path::new(put), oznaka)?;
            Ok(0)
        }
        "mjeri-kml-crate" => {
            let put = arg.get(2).ok_or("nedostaje putanja datoteke")?;
            let oznaka = arg.get(3).map(String::as_str).unwrap_or("datoteka");
            naredba_mjeri_kml_crate(Path::new(put), oznaka)?;
            Ok(0)
        }
        _ => Err(format!("nepoznata naredba: {naredba}")),
    }
}

fn main() -> ExitCode {
    match pokreni() {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::from(1),
        Err(g) => {
            eprintln!("greška: {g}");
            ExitCode::from(1)
        }
    }
}
