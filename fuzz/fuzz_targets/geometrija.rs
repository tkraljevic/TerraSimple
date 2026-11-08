#![no_main]
use koordinate::Sustav;
libfuzzer_sys::fuzz_target!(|b: &[u8]| {
    for sustav in [
        Sustav::Wgs84,
        Sustav::WebMercator,
        Sustav::Htrs96Tm,
        Sustav::HdksZona5,
        Sustav::HdksZona6,
    ] {
        // Nevaljan ulaz smije vratiti grešku, ali ne smije srušiti proces.
        drop(geopaket::fuzz::geometrija(b, sustav));
    }
});
