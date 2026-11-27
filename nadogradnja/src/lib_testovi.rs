use super::*;
use crate::pomoc_testova::{Mapa, tar_gz};
use base64::Engine;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};

type Odgovori = HashMap<String, (u16, Vec<u8>)>;

/// Mali HTTP poslužitelj za testove: staza → (status, tijelo). Tijela smiju ovisiti o adresi
/// poslužitelja, pa ih slaže `odgovori` kad je adresa poznata.
fn posluzitelj(odgovori: impl FnOnce(&str) -> Odgovori) -> String {
    let slusac = TcpListener::bind("127.0.0.1:0").expect("vrata");
    let adresa = format!("http://{}", slusac.local_addr().expect("adresa"));
    let odgovori = odgovori(&adresa);
    std::thread::spawn(move || {
        for veza in slusac.incoming().flatten() {
            let mut citac = BufReader::new(&veza);
            let mut redak = String::new();
            let _ = citac.read_line(&mut redak); // loš zahtjev dobije 404
            let staza = redak.split_whitespace().nth(1).unwrap_or("").to_string();
            let mut zaglavlje = String::new();
            while citac.read_line(&mut zaglavlje).is_ok_and(|n| n > 2) {
                zaglavlje.clear();
            }
            let (status, tijelo) = odgovori.get(&staza).cloned().unwrap_or((404, Vec::new()));
            let mut veza = veza;
            let _ = write!(
                veza,
                "HTTP/1.1 {status} X\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                tijelo.len()
            ); // klijent sam javlja prekinutu vezu
            let _ = veza.write_all(&tijelo); // isto
        }
    });
    adresa
}

fn javni(tajni: &[u8]) -> Vec<u8> {
    base64::engine::general_purpose::STANDARD
        .decode(javni_kljuc_iz_tajnog(tajni).expect("javni"))
        .expect("base64")
}

fn izvor(adresa: &str, tajni: &[u8]) -> Izvor {
    Izvor {
        popis: format!("{adresa}/releases"),
        prefiks: format!("{adresa}/download/"),
        javni_kljuc: javni(tajni),
    }
}

fn ime() -> String {
    Sustav::Linux.ime_paketa(Inacica(0, 0, 9))
}

fn paket() -> Vec<u8> {
    tar_gz(&[
        ("TerraSimple-0.0.9-linux-x86_64/TerraSimple", b"novi", 0o755),
        (
            "TerraSimple-0.0.9-linux-x86_64/Dozvole/LICENSE",
            b"EUPL",
            0o644,
        ),
    ])
}

fn popis(adresa: &str, velicina: usize) -> Vec<u8> {
    let datoteka = |ime: String, velicina: usize| {
        serde_json::json!({
            "name": ime,
            "size": velicina,
            "browser_download_url": format!("{adresa}/download/v0.0.9/{ime}"),
        })
    };
    serde_json::json!([{
        "tag_name": "v0.0.9",
        "html_url": format!("{adresa}/v0.0.9"),
        "assets": [datoteka(ime(), velicina), datoteka(format!("{}.sig", ime()), 88)],
    }])
    .to_string()
    .into_bytes()
}

/// Izdanje 0.0.9 s paketom i potpisom kojim potpisuje `tajni_potpisa`.
fn izdanje(tajni_potpisa: &[u8]) -> impl FnOnce(&str) -> Odgovori {
    let paket = paket();
    let potpis = potpisi(tajni_potpisa, &ime(), &paket).expect("potpis");
    move |adresa| {
        HashMap::from([
            ("/releases".into(), (200, popis(adresa, paket.len()))),
            (format!("/download/v0.0.9/{}", ime()), (200, paket)),
            (
                format!("/download/v0.0.9/{}.sig", ime()),
                (200, potpis.into_bytes()),
            ),
        ])
    }
}

fn instaliran(m: &Mapa) -> (PathBuf, Mjesto) {
    let exe = m.0.join("TerraSimple-0.0.8/TerraSimple");
    std::fs::create_dir_all(exe.with_file_name("Dozvole")).expect("mapa");
    std::fs::write(&exe, b"stari").expect("program");
    let mjesto = Mjesto::za_program(&exe, Sustav::Linux).expect("mjesto");
    (exe, mjesto)
}

fn ponuda(izvor: &Izvor) -> Ponuda {
    match provjeri(izvor, Inacica(0, 0, 8), Sustav::Linux).expect("provjera") {
        Ishod::Dostupna(p) => p,
        ishod => panic!("treba ponuda, a stiglo je {ishod:?}"),
    }
}

#[test]
fn provjera_i_ugradnja_s_poslužitelja() {
    let tajni = novi_tajni_kljuc().expect("ključ");
    let izvor = izvor(&posluzitelj(izdanje(&tajni)), &tajni);
    let ponuda = ponuda(&izvor);
    assert_eq!(ponuda.inacica, Inacica(0, 0, 9));
    assert_eq!(
        provjeri(&izvor, Inacica(0, 0, 9), Sustav::Linux).expect("provjera"),
        Ishod::Najnovija(Inacica(0, 0, 9))
    );
    let m = Mapa::nova("posluzitelj");
    let (exe, mjesto) = instaliran(&m);
    preuzmi_i_ugradi(&izvor, &ponuda, &mjesto).expect("ugrađeno");
    assert_eq!(std::fs::read(&exe).expect("program"), b"novi");
}

#[test]
fn tudi_potpis_ne_dira_program() {
    let tajni = novi_tajni_kljuc().expect("ključ");
    let tudji = novi_tajni_kljuc().expect("ključ");
    let izvor = izvor(&posluzitelj(izdanje(&tudji)), &tajni);
    let m = Mapa::nova("tudi-potpis");
    let (exe, mjesto) = instaliran(&m);
    assert!(matches!(
        preuzmi_i_ugradi(&izvor, &ponuda(&izvor), &mjesto),
        Err(Greska::KrivPotpis)
    ));
    assert_eq!(std::fs::read(&exe).expect("program"), b"stari");
}

#[test]
fn greske_veze_i_odgovora() {
    let tajni = novi_tajni_kljuc().expect("ključ");
    let adresa = posluzitelj(|_| {
        HashMap::from([
            ("/los-json/releases".into(), (200, b"{nije popis".to_vec())),
            ("/prevelik/releases".into(), (200, vec![b' '; 5 << 20])),
        ])
    });
    let s = |put: &str| Izvor {
        popis: format!("{adresa}{put}/releases"),
        ..izvor(&adresa, &tajni)
    };
    let provjera = |i: &Izvor| provjeri(i, Inacica(0, 0, 8), Sustav::Linux);
    assert!(matches!(provjera(&s("/nema")), Err(Greska::Odgovor(404))));
    assert!(matches!(
        provjera(&s("/los-json")),
        Err(Greska::NeocekivanOdgovor(_))
    ));
    assert!(matches!(
        provjera(&s("/prevelik")),
        Err(Greska::Prevelika { .. })
    ));
    let bez_veze = Izvor {
        popis: "http://127.0.0.1:1/releases".into(),
        ..izvor(&adresa, &tajni)
    };
    assert!(matches!(provjera(&bez_veze), Err(Greska::Veza(_))));
}

#[test]
fn preuzimanje_odbija_tudu_adresu_i_preveliku_datoteku() {
    let tajni = novi_tajni_kljuc().expect("ključ");
    let adresa = posluzitelj(|_| HashMap::from([("/download/velik".into(), (200, vec![0; 2048]))]));
    let izvor = izvor(&adresa, &tajni);
    let datoteka = |adresa: String, velicina| Datoteka {
        ime: "p".into(),
        velicina,
        adresa,
    };
    assert!(matches!(
        izvor.preuzmi(&datoteka("https://primjer.invalid/p".into(), 1), 10),
        Err(Greska::TudaAdresa(_))
    ));
    let velik = format!("{adresa}/download/velik");
    // GitHub kaže veličinu unaprijed; i kad je kriva, čita se najviše dopušteno.
    for velicina in [4096, 1] {
        assert!(matches!(
            izvor.preuzmi(&datoteka(velik.clone(), velicina), 1024),
            Err(Greska::Prevelika { ime, .. }) if ime == "p"
        ));
    }
}

#[test]
fn izvor_github_ima_ispravan_kljuc() {
    let g = Izvor::github();
    assert_eq!(g.javni_kljuc.len(), 32);
    assert!(
        g.popis
            .starts_with("https://api.github.com/repos/tkraljevic/TerraSimple/")
    );
    assert!(Path::new(&g.prefiks).starts_with("https://github.com/tkraljevic/TerraSimple"));
}

#[test]
fn greske_imaju_kratak_opis() {
    assert_eq!(Greska::Odgovor(503).to_string(), "Odgovor(503)");
    assert_eq!(Greska::KrivPotpis.to_string(), "KrivPotpis");
}

#[test]
fn inacica_se_cita_i_ispisuje() {
    assert_eq!(Inacica::procitaj("v0.0.9"), Some(Inacica(0, 0, 9)));
    assert_eq!(Inacica::procitaj("1.2.30"), Some(Inacica(1, 2, 30)));
    for los in ["0.0", "0.0.9.1", "0.0.9-rc1", "", "v", "a.b.c"] {
        assert_eq!(Inacica::procitaj(los), None, "{los}");
    }
    assert!(Inacica(0, 0, 10) > Inacica(0, 0, 9));
    assert!(Inacica(0, 1, 0) > Inacica(0, 0, 99));
    assert_eq!(Inacica(0, 0, 9).to_string(), "0.0.9");
}

#[test]
fn izvor_za_probu_zadrzava_nas_kljuc() {
    let proba = Izvor::za_program(Some("http://127.0.0.1:8000"));
    assert_eq!(proba.popis, "http://127.0.0.1:8000/releases");
    assert_eq!(proba.prefiks, "http://127.0.0.1:8000/download/");
    assert_eq!(proba.javni_kljuc, Izvor::github().javni_kljuc);
    assert_eq!(Izvor::za_program(None).popis, Izvor::github().popis);
}
