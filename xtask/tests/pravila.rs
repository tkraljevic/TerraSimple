use xtask::pravila::{Funkcija, Mjerenje, Postavke, rizik, usporedi};

fn mjerenje(cc: u32, pokrivenost: f64) -> Mjerenje {
    Mjerenje::novo(
        "primjer",
        vec![Funkcija::nova("src/lib.rs::racun", cc, pokrivenost, false)],
    )
}

#[test]
fn formula_rizika() {
    assert_eq!(rizik(4, 1.0), 4.0);
    assert_eq!(rizik(4, 0.0), 20.0);
    assert_eq!(rizik(4, 0.5), 6.0);
}

#[test]
fn nova_funkcija_mora_biti_unutar_granice() {
    let p = Postavke::default();
    assert!(
        usporedi(&mjerenje(11, 1.0), None, &p)
            .iter()
            .any(|n| n.kljuc.starts_with("slozenost:"))
    );
    assert!(usporedi(&mjerenje(2, 1.0), None, &p).is_empty());
}

#[test]
fn stari_dug_ne_dopusta_rast_slozenosti() {
    let p = Postavke::default();
    assert!(usporedi(&mjerenje(20, 1.0), Some(&mjerenje(20, 1.0)), &p).is_empty());
    assert!(!usporedi(&mjerenje(21, 1.0), Some(&mjerenje(20, 1.0)), &p).is_empty());
}

#[test]
fn pad_pokrivenosti_i_rast_rizika() {
    let p = Postavke::default();
    assert!(!usporedi(&mjerenje(8, 0.5), Some(&mjerenje(8, 1.0)), &p).is_empty());
    let mut novo = mjerenje(2, 1.0);
    let staro = novo.clone();
    novo.pokrivenost -= 0.001;
    assert!(usporedi(&novo, Some(&staro), &p).is_empty());
    novo.pokrivenost -= 0.00001;
    assert!(!usporedi(&novo, Some(&staro), &p).is_empty());
}

#[test]
fn kriticni_kod_ima_stroge_granice_i_kad_je_star() {
    let mut m = mjerenje(8, 0.94);
    m.funkcije[0].kriticna = true;
    let nalazi = usporedi(&m, Some(&m), &Postavke::default());
    assert!(nalazi.iter().any(|n| n.kljuc.starts_with("slozenost:")));
    assert!(nalazi.iter().any(|n| n.kljuc.starts_with("pokrivenost:")));
}
