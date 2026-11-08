#![no_main]
use slojevi::{Stil, VrstaGeometrije};
libfuzzer_sys::fuzz_target!(|b: &[u8]| {
    if let Ok(xml) = std::str::from_utf8(b) {
        for vrsta in [
            VrstaGeometrije::Tocka,
            VrstaGeometrije::Linija,
            VrstaGeometrije::Poligon,
        ] {
            // Odbijanje nevaljanog XML-a očekivani je ishod.
            drop(geopaket::fuzz::stil(xml, vrsta, Stil::default()));
        }
    }
});
