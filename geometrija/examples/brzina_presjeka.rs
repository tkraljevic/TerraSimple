//! Brzina provjere presjeka na izmišljenim likovima: krug bez presjeka i „parcela” s nazubljenim
//! rubom, oboje cijeli pregledani jer presjeka nema.
use geometrija::{Tocka, presjek_lika};
use std::time::Instant;

fn krug(n: usize) -> Vec<Tocka> {
    (0..n)
        .map(|i| {
            let k = i as f64 / n as f64 * std::f64::consts::TAU;
            [18.0 + k.cos(), 45.0 + k.sin()]
        })
        .collect()
}

/// Rub s malim zubima, kao ručno crtana međa: x raste, a y skače gore-dolje.
fn nazubljen(n: usize) -> Vec<Tocka> {
    let mut v: Vec<Tocka> = (0..n - 2)
        .map(|i| {
            [
                18.0 + i as f64 * 1e-6,
                45.0 + if i % 2 == 0 { 0.0 } else { 1e-6 },
            ]
        })
        .collect();
    v.push([18.0 + n as f64 * 1e-6, 44.0]);
    v.push([18.0, 44.0]);
    v
}

fn main() {
    for n in [10_000, 100_000, 1_000_000] {
        for (ime, lik) in [("krug", krug(n)), ("nazubljen", nazubljen(n))] {
            let pocetak = Instant::now();
            let presjek = presjek_lika(&[&lik]);
            println!(
                "{n:>9} vrhova, {ime:<9}: {:>8.1} ms, presjek {presjek:?}",
                pocetak.elapsed().as_secs_f64() * 1e3
            );
        }
    }
}
