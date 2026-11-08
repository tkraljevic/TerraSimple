use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let a: Vec<String> = env::args().skip(1).collect();
    let mapa = env::current_dir().unwrap();
    if (a[0] != "metadata" && mapa.join("pad-alata").exists()) || (a[0] == "test" && mapa.join("pad-testa").exists()) {
        eprintln!("Namjeran pad probnog alata.");
        return ExitCode::FAILURE;
    }
    if a[0] == "mutants" && a.len() > 2 {
        assert_eq!(env::var("CARGO_TARGET_DIR").unwrap(), "target");
        if !a.windows(2).any(|p|p == ["--gitignore", "true"]) || a.iter().any(|p|p == "--copy-target") {
            eprintln!("Mutacije moraju preskočiti zanemarene izlaze i stare gradnje.");
            return ExitCode::FAILURE;
        }
        let i = a.iter().position(|s| s == "--output").unwrap();
        let izlaz = std::path::Path::new(&a[i+1]).join("mutants.out");
        fs::create_dir_all(&izlaz).unwrap();
        let stanje = fs::read_to_string(mapa.join("ishod")).unwrap_or("CaughtMutant".into());
        fs::write(izlaz.join("outcomes.json"), format!(r#"{{"end_time":"kraj","total_mutants":1,"outcomes":[{{"scenario":"Baseline","summary":"Success"}},{{"scenario":{{"Mutant":{{"name":"src/lib.rs:1: replace a with true","file":"src/lib.rs","function":null}}}},"summary":"{stanje}"}}]}}"#)).unwrap();
        return ExitCode::from(fs::read_to_string(mapa.join("kod")).unwrap_or("0".into()).parse::<u8>().unwrap());
    }
    let ime = if mapa.join("kritican").exists() { "koordinate" } else { "primjer" };
    let mapa = mapa.to_string_lossy().replace('\\', "\\\\");
    if a[0] == "metadata" {
        println!(r#"{{"workspace_members":["p"],"packages":[{{"id":"p","name":"{ime}","manifest_path":"{mapa}/Cargo.toml","targets":[{{"kind":["lib"],"src_path":"{mapa}/src/lib.rs"}}]}}]}}"#);
    }
    if a[0] == "llvm-cov" && a.len() > 2 {
        let i = a.iter().position(|s| s == "--output-path").unwrap();
        fs::write(&a[i+1], format!(r#"{{"data":[{{"functions":[{{"filenames":["{mapa}/src/lib.rs"],"regions":[[1,12,1,18,1,0,0,0]]}}]}}]}}"#)).unwrap();
    }
    ExitCode::SUCCESS
}
