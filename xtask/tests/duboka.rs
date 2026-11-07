use xtask::{duboka::*, pravila::Postavke};

#[test]
fn prezivjeli_i_nepoznati_rezultati_ruse_provjeru() {
    let p = Postavke::default();
    let mut m = Ishod {
        total_mutants: 1,
        end_time: Some("kraj".into()),
        outcomes: vec![
            Pokus {
                scenario: Scenarij::Baseline,
                summary: "Success".into(),
            },
            Pokus {
                scenario: Scenarij::Mutant(Mutacija {
                    name: "a.rs:2: replace a with true".into(),
                    file: "a.rs".into(),
                    function: Some(Funkcija {
                        function_name: "a".into(),
                    }),
                }),
                summary: "CaughtMutant".into(),
            },
        ],
    };
    assert!(ocijeni(&m, &p).is_ok());
    m.outcomes[1].summary = "MissedMutant".into();
    assert!(ocijeni(&m, &p).unwrap_err().contains("a.rs"));
    let iznimke = serde_json::from_str(r#"{"iznimke":[{"kljuc":"mutacija:a.rs:2: replace a with true","razlog":"Pregledano.","odgovorna_osoba":"Održavatelj","rok":"2099-01-01"}]}"#).unwrap();
    assert!(ocijeni(&m, &iznimke).is_ok());
    for stanje in ["Timeout", "Failure", "nepoznato"] {
        m.outcomes[1].summary = stanje.into();
        assert!(ocijeni(&m, &p).is_err());
    }
    m.outcomes[1].summary = "Unviable".into();
    assert!(ocijeni(&m, &p).is_ok());
    m.end_time = None;
    assert!(ocijeni(&m, &p).is_err());
    m.end_time = Some("kraj".into());
    m.total_mutants = 2;
    assert!(ocijeni(&m, &p).is_err());
    m.total_mutants = 1;
    m.outcomes[1].scenario = Scenarij::Baseline;
    m.outcomes[1].summary = "Success".into();
    assert!(ocijeni(&m, &p).is_err());
    m.outcomes[1].summary = "Failure".into();
    assert!(ocijeni(&m, &p).is_err());
}

#[test]
fn brojac_oznacava_sve_retke_preklopljenih_blokova() {
    let a = (0..12)
        .map(|i| format!("redak {i}"))
        .collect::<Vec<_>>()
        .join("\n");
    let (svi, isti) = prebroji(&[a.clone(), a.clone()]);
    assert_eq!((svi, isti), (24, 24));
    assert_eq!(prebroji(&[a]), (12, 0));
    assert_eq!(prebroji(&["// komentar\n\njedan".into()]), (1, 0));
    let a = (0..10)
        .map(|i| format!("r{i}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(prebroji(&[format!("{a}\nx"), format!("{a}\ny")]), (22, 20));
}
