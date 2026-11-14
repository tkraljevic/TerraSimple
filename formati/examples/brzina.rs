//! Mjeri čitanje i uvoz velike KML datoteke. Ispisuje samo brojke, jer prava datoteka može biti
//! privatna: `cargo run --release -p formati --example brzina -- staza.kml`.

use std::time::Instant;

fn main() {
    let staza = std::env::args().nth(1).expect("staza");
    let pocetak = Instant::now();
    let d = formati::procitaj(std::path::Path::new(&staza)).expect("čitanje");
    let citanje = pocetak.elapsed();
    let mut u = slojevi::Uredjivac::novi();
    let nazivi = formati::NaziviSlojeva {
        tocke: "Točke".into(),
        linije: "Linije".into(),
        poligoni: "Poligoni".into(),
        bez_naziva: "Mapa bez naziva".into(),
    };
    let pocetak = Instant::now();
    let s = formati::uvezi(&mut u, &d, "datoteka", &nazivi).expect("uvoz");
    let uvoz = pocetak.elapsed();
    println!("čitanje {citanje:?}, uvoz {uvoz:?}");
    println!(
        "mapa {}, slojeva {}, objekata {}, odbijeno {}",
        s.mapa_ukupno, s.slojeva, s.objekata, s.odbijeno
    );
    println!("{:?}", s.gubici);
    // Drugi argument: GeoPackage u koji se uvezeno sprema, za mjerenje spremanja.
    if let Some(izlaz) = std::env::args().nth(2) {
        let pocetak = Instant::now();
        let mut datoteka = geopaket::Datoteka::otvori(&izlaz).expect("GeoPackage");
        datoteka.spremi(u.projekt()).expect("spremanje");
        println!("spremanje {:?}", pocetak.elapsed());
        let pocetak = Instant::now();
        let projekt = geopaket::Datoteka::otvori(&izlaz)
            .and_then(|mut d| d.ucitaj())
            .expect("učitavanje");
        println!(
            "učitavanje {:?}, slojeva {}",
            pocetak.elapsed(),
            projekt.slojevi().count()
        );
    }
}
