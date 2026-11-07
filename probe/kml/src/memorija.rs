//! Mjerenje najvećeg utroška radne memorije procesa (`ru_maxrss`) preko sustavskog poziva.
//!
//! Razlog za `unsafe` (7. 11. 2026.): čitanje `libc::getrusage` u probi radi mjerenja memorije,
//! jednako kao u `probe/geotiff`. Ovaj modul ne ulazi u program.

pub fn vrh_memorije_mb() -> f64 {
    let mut r: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut r) };
    r.ru_maxrss as f64 / 1e6
}
