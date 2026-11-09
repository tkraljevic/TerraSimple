#![forbid(unsafe_code)]
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod pokretanje;

fn main() -> Result<(), String> {
    pokretanje::pokreni(std::env::args().skip(1).collect())
}
