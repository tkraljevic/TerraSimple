use geopaket::{Datoteka, Rezultat};
use slojevi::*;
fn dodaj(u: &mut Uredjivac, vrsta: VrstaGeometrije, g: Geometrija) -> Rezultat<IdSloja> {
    let Some(Novo::Sloj(s)) = u.izvrsi(NaredbaStabla::DodajSloj {
        u: None,
        naziv: format!("{vrsta:?}"),
        vrsta,
    })?
    else {
        return Err("Nema sloja.".into());
    };
    u.izvrsi(NaredbaObjekta::Dodaj {
        sloj: s,
        naziv: "Izmišljeni objekt".into(),
        geometrija: g,
    })?;
    Ok(s)
}
fn main() -> Rezultat<()> {
    let a: Vec<_> = std::env::args().collect();
    let p = a.get(2).ok_or("Nema putanje.")?;
    match a.get(1).map(String::as_str) {
        Some("stvori") => {
            let mut u = Uredjivac::novi();
            let v = |x, y| Vrh {
                duzina: x,
                sirina: y,
                visina: Some(123.5),
            };
            dodaj(
                &mut u,
                VrstaGeometrije::Tocka,
                Geometrija::Tocka(v(18.69, 45.55)),
            )?;
            dodaj(
                &mut u,
                VrstaGeometrije::Linija,
                Geometrija::Linija(vec![v(18.68, 45.54), v(18.69, 45.55)]),
            )?;
            let s_poligon = dodaj(
                &mut u,
                VrstaGeometrije::Poligon,
                Geometrija::Poligon(vec![
                    vec![v(18.68, 45.54), v(18.7, 45.54), v(18.7, 45.56)],
                    vec![v(18.692, 45.542), v(18.694, 45.542), v(18.694, 45.544)],
                ]),
            )?;
            let Some(Novo::Mapa(m)) = u.izvrsi(NaredbaStabla::DodajMapu {
                u: None,
                naziv: "Područja".into(),
            })?
            else {
                return Err("Nema mape.".into());
            };
            u.izvrsi(NaredbaStabla::Premjesti {
                stavka: Stavka::Sloj(s_poligon),
                u: Some(m),
                polozaj: 0,
            })?;
            Datoteka::otvori(p)?.spremi(u.projekt())?;
        }
        Some("citaj") => {
            let mut d = Datoteka::otvori(p)?;
            let p = d.ucitaj()?;
            let slojevi:Vec<_>=p.slojevi().map(|(id,s)|serde_json::json!({"id":id.0,"naziv":s.naziv,"objekti":s.objekti.values().map(|o|serde_json::json!({"id":o.id.0,"naziv":o.naziv,"opis":o.opis,"geometrija":format!("{:?}",o.geometrija),"stil":format!("{:?}",o.stil)})).collect::<Vec<_>>(),"stil":format!("{:?}",s.stil)})).collect();
            println!(
                "{}",
                serde_json::json!({"slojevi":slojevi,"upozorenja":d.upozorenja})
            );
        }
        Some("spremi") => {
            let mut d = Datoteka::otvori(p)?;
            let p = d.ucitaj()?;
            d.spremi(&p)?;
        }
        Some("promijeni") => {
            let mut d = Datoteka::otvori(p)?;
            let p = d.ucitaj()?;
            let (s, o) = p
                .slojevi()
                .find_map(|(s, l)| l.objekti.keys().next().map(|o| (s, *o)))
                .ok_or("Nema objekta.")?;
            let mut u = Uredjivac::iz_projekta(p);
            u.izvrsi(NaredbaObjekta::Izmijeni {
                sloj: s,
                objekt: o,
                izmjena: IzmjenaObjekta::Opis("Spremljeno nakon QGIS-a".into()),
            })?;
            d.spremi(u.projekt())?;
        }
        Some(n @ ("dodaj-bez-visine" | "dodaj-s-visinom")) => {
            let mut d = Datoteka::otvori(p)?;
            let p = d.ucitaj()?;
            let s = p
                .slojevi()
                .find(|(_, l)| l.vrsta == VrstaGeometrije::Linija)
                .map(|(s, _)| s)
                .ok_or("Nema linijskog sloja.")?;
            let visina = (n == "dodaj-s-visinom").then_some(7.);
            let mut u = Uredjivac::iz_projekta(p);
            u.izvrsi(NaredbaObjekta::Dodaj {
                sloj: s,
                naziv: "Dodana linija".into(),
                geometrija: Geometrija::Linija(vec![
                    Vrh {
                        duzina: 18.,
                        sirina: 45.,
                        visina,
                    },
                    Vrh {
                        duzina: 18.1,
                        sirina: 45.1,
                        visina,
                    },
                ]),
            })?;
            d.spremi(u.projekt())?;
        }
        _ => return Err("Nepoznata naredba.".into()),
    }
    Ok(())
}
