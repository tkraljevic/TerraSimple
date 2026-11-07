use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let a: Vec<String> = env::args().skip(1).collect();
    let mapa = env::current_dir().unwrap();
    if mapa.join("pad-alata").exists() || (a[0] == "test" && mapa.join("pad-testa").exists()) {
        eprintln!("Namjeran pad probnog alata.");
        return ExitCode::FAILURE;
    }
    let mapa = mapa.to_string_lossy().replace('\\', "\\\\");
    if a[0] == "metadata" {
        println!(r#"{{"workspace_members":["p"],"packages":[{{"id":"p","name":"primjer","manifest_path":"{mapa}/Cargo.toml","targets":[{{"kind":["lib"],"src_path":"{mapa}/src/lib.rs"}}]}}]}}"#);
    }
    if a[0] == "llvm-cov" && a.len() > 2 {
        let i = a.iter().position(|s| s == "--output-path").unwrap();
        fs::write(&a[i+1], format!(r#"{{"data":[{{"functions":[{{"filenames":["{mapa}/src/lib.rs"],"regions":[[1,12,1,18,1,0,0,0]]}}]}}]}}"#)).unwrap();
    }
    ExitCode::SUCCESS
}
