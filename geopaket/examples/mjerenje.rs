//! Izmišljena količina iz zadatka; mjerenje se pokreće u release načinu.
use geopaket::{Datoteka, Rezultat};
use slojevi::*;
use std::{path::PathBuf, time::Instant};
fn memorija() -> Rezultat<u64> {
    let r = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()?;
    Ok(std::str::from_utf8(&r.stdout)?.trim().parse::<u64>()? * 1024)
}
fn main() -> Rezultat<()> {
    let p = PathBuf::from(std::env::args().nth(1).ok_or("Nema putanje za mjerenje.")?);
    if p.exists() {
        return Err("Mjerenje traži novu datoteku.".into());
    }
    let mut u = Uredjivac::novi();
    let mut objekata = 0;
    for i in 0..208 {
        let Some(Novo::Sloj(s)) = u.izvrsi(NaredbaStabla::DodajSloj {
            u: None,
            naziv: format!("Izmišljeni sloj {i}"),
            vrsta: VrstaGeometrije::Linija,
        })?
        else {
            return Err("Nema sloja.".into());
        };
        let n = if i < 87105 % 208 {
            87105 / 208 + 1
        } else {
            87105 / 208
        };
        u.skupina(|g| {
            for j in 0..n {
                let v = (0..35)
                    .map(|k| Vrh {
                        duzina: 18.0 + i as f64 * 0.001 + k as f64 * 0.00001,
                        sirina: 45.0 + j as f64 * 0.00001,
                        visina: Some(100.0 + k as f64),
                    })
                    .collect();
                g.izvrsi(NaredbaObjekta::Dodaj {
                    sloj: s,
                    naziv: format!("Crta {j}"),
                    geometrija: Geometrija::Linija(v),
                })?;
                objekata += 1;
            }
            Ok(())
        })?;
    }
    let pripremljeno = memorija()?;
    let mut d = Datoteka::otvori(&p)?;
    let t = Instant::now();
    d.spremi(u.projekt())?;
    let spremanje = t.elapsed();
    let upis = d.trajanje_upisa;
    drop(u);
    let nakon_spremanja = memorija()?;
    let t = Instant::now();
    let p = d.ucitaj()?;
    let citanje = t.elapsed();
    let nakon_citanja = memorija()?;
    assert_eq!(p.slojevi().count(), 208);
    assert_eq!(
        p.slojevi().map(|(_, s)| s.objekti.len()).sum::<usize>(),
        87105
    );
    let (s, o) = p
        .slojevi()
        .find_map(|(s, l)| l.objekti.keys().next().map(|o| (s, *o)))
        .ok_or("Nema objekta.")?;
    let mut u = Uredjivac::iz_projekta(p);
    u.izvrsi(NaredbaObjekta::Izmijeni {
        sloj: s,
        objekt: o,
        izmjena: IzmjenaObjekta::Opis("Jedan izmijenjeni objekt".into()),
    })?;
    let t = Instant::now();
    d.spremi(u.projekt())?;
    let jedan = t.elapsed();
    println!(
        "{}",
        serde_json::json!({"objekata":objekata,"slojeva":208,"vrhova":objekata*35,"spremanje_s":spremanje.as_secs_f64(),"transakcija_pocetna_s":upis.as_secs_f64(),"citanje_s":citanje.as_secs_f64(),"jedan_objekt_s":jedan.as_secs_f64(),"jedan_objekt_transakcija_s":d.trajanje_upisa.as_secs_f64(),"memorija_pripremljeno_b":pripremljeno,"memorija_nakon_spremanja_b":nakon_spremanja,"memorija_nakon_citanja_b":nakon_citanja})
    );
    Ok(())
}
