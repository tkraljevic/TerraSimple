//! Ispisuje pročitano stablo KML-a s nazivima i stilovima, za provjeru čitača na izmišljenim primjerima.

fn obidji(m: &formati::MapaKml, d: usize) {
    for o in &m.objekti {
        println!(
            "{:w$}{} {:?} {:?}",
            "",
            o.naziv,
            o.adresa_stila,
            o.stil,
            w = d * 2
        );
    }
    for p in &m.mape {
        println!("{:w$}[{}]", "", p.naziv, w = d * 2);
        obidji(p, d + 1);
    }
}

fn main() {
    let staza = std::env::args().nth(1).expect("staza");
    let d = formati::procitaj(std::path::Path::new(&staza)).expect("čitanje");
    obidji(&d.korijen, 0);
    println!("{:?}", d.gubici);
}
