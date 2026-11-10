//! Greška pri pokretanju mora se vidjeti. Na Windowsu program nema konzolu, pa bi poruka o padu
//! inače nestala, a korisnik bi vidio samo da se ništa ne događa (proba 0.0.1, 10. 11. 2026.).

use std::path::{Path, PathBuf};

/// Datoteka u koju se zapiše opis greške, u privremenoj mapi korisnika.
pub fn datoteka_greske() -> PathBuf {
    std::env::temp_dir().join("TerraSimple-greska.txt")
}

/// Tekst prozorčića: što se dogodilo i gdje je opis.
pub fn tekst_prozorcica(opis: &str, datoteka: &Path) -> String {
    format!(
        "TerraSimple se nije mogao pokrenuti.\n\n{opis}\n\nOpis greške spremljen je u {}. Pošalji ga uz prijavu greške.",
        datoteka.display()
    )
}

/// Zapiše opis u datoteku i na izlaz za greške. Vraća tekst za prozorčić.
pub fn zapisi(opis: &str, datoteka: &Path) -> String {
    let _ = std::fs::write(datoteka, opis);
    eprintln!("{opis}");
    tekst_prozorcica(opis, datoteka)
}

/// Opis pada s mjestom u kodu i tragom poziva.
pub fn opis_pada(poruka: &str) -> String {
    format!(
        "Program je pao: {poruka}\n\n{}",
        std::backtrace::Backtrace::force_capture()
    )
}

/// Prozorčić s greškom. Kod provjere pokretanja se ne prikazuje, jer ga nitko ne bi zatvorio.
pub fn prikazi(tekst: &str, provjera: bool) {
    if provjera {
        return;
    }
    let _ = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("TerraSimple")
        .set_description(tekst)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}

/// Pad bilo gdje u programu zapiše se i prikaže, uz uobičajeni ispis.
pub fn ugradi(provjera: bool) {
    ugradi_u(provjera, datoteka_greske());
}

fn ugradi_u(provjera: bool, datoteka: PathBuf) {
    let zadano = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        zadano(info);
        prikazi(&zapisi(&opis_pada(&info.to_string()), &datoteka), provjera);
    }));
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn opis_ide_u_datoteku_i_u_tekst_prozorcica() {
        let mapa = std::env::temp_dir().join(format!("ts-javljanje-{}", std::process::id()));
        std::fs::create_dir_all(&mapa).unwrap();
        let datoteka = mapa.join("greska.txt");
        let tekst = zapisi("Grafička kartica nije dostupna.", &datoteka);
        assert_eq!(
            std::fs::read_to_string(&datoteka).unwrap(),
            "Grafička kartica nije dostupna."
        );
        assert!(tekst.starts_with("TerraSimple se nije mogao pokrenuti.\n\nGrafička kartica"));
        assert!(tekst.contains(&datoteka.display().to_string()), "{tekst}");
        let _ = std::fs::remove_dir_all(&mapa);
    }

    #[test]
    fn pad_se_zapise_u_datoteku() {
        let datoteka = std::env::temp_dir().join(format!("ts-pad-{}.txt", std::process::id()));
        ugradi_u(true, datoteka.clone());
        let ishod = std::panic::catch_unwind(|| panic!("namjerni pad u testu"));
        let _ = std::panic::take_hook();
        assert!(ishod.is_err());
        let opis = std::fs::read_to_string(&datoteka).unwrap();
        assert!(opis.starts_with("Program je pao: panicked at "), "{opis}");
        assert!(opis.contains("namjerni pad u testu"), "{opis}");
        let _ = std::fs::remove_file(&datoteka);
    }

    #[test]
    fn opis_pada_ima_poruku_i_trag() {
        let o = opis_pada("index out of bounds");
        assert!(
            o.starts_with("Program je pao: index out of bounds\n\n"),
            "{o}"
        );
        assert!(o.len() > "Program je pao: index out of bounds\n\n".len());
        assert!(datoteka_greske().ends_with("TerraSimple-greska.txt"));
        // Kod provjere pokretanja prozorčić se ne otvara, pa se ovo vrati odmah.
        prikazi("x", true);
    }
}
