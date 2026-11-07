use xtask::slozenost::analiziraj;

#[test]
fn broji_grane_a_ne_komentare_i_tekst() {
    let kod = r#"pub fn racun(x: i32) -> Option<i32> {
        // if while match ?
        let _tekst = "if && ?";
        if x > 0 && x < 10 { Some(x)?; }
        for _ in 0..2 { }
        match x { 1 | 2 => Some(1), _ => None }
    }"#;
    let f = analiziraj(kod).unwrap();
    assert_eq!(f[0].cc, 7);
}

#[test]
fn testovi_ne_ulaze_u_mjerenje() {
    let kod = "fn a() {} #[cfg(test)] mod testovi { #[test] fn b() {} }";
    assert_eq!(analiziraj(kod).unwrap().len(), 1);
}

#[test]
fn identitet_metode_ukljucuje_tip_i_trait() {
    let kod = "impl A { fn a() {} } impl B { fn a() {} } impl T for A { fn a() {} }";
    let f = analiziraj(kod).unwrap();
    assert_ne!(f[0].ime, f[1].ime);
    assert_ne!(f[0].ime, f[2].ime);
}

#[test]
fn novi_rust_i_neispravna_sintaksa() {
    assert!(analiziraj("async fn a() { let f = async || 3; f().await; }").is_ok());
    assert!(analiziraj("fn {").is_err());
}

#[test]
fn makroi_s_izrazima_ne_skrivaju_grananje() {
    let kod = r#"fn a(x: u8) {
        let _ = vec![if x > 1 { 1 } else { 0 }; 2];
        let _ = matches!(x, 1 | 2 if x < 4);
        println!("{}", if x > 0 { x } else { 0 });
    }"#;
    assert_eq!(analiziraj(kod).unwrap()[0].cc, 6);
    assert!(analiziraj("fn a() { neki! { x => y } }").is_err());
    assert!(analiziraj("fn a() { neki!(x); }").is_err());
    assert!(analiziraj("neki! { fn a() {} }").is_err());
    assert!(analiziraj("#![cfg(test)] fn a() {}").unwrap().is_empty());
    assert!(analiziraj("fn a() { let _ = quote::quote!(fn x() {}); }").is_ok());
    let f = analiziraj(
        r#"fn a(x: u8) { json!({ "x": [if x > 0 { 1 } else { 0 }, {"y": x}], "z": null }); }"#,
    )
    .unwrap();
    assert_eq!(f[0].cc, 2);
}

#[test]
fn broji_cuvare_uzorke_petlje_i_ugnijezdene_funkcije() {
    let kod = r#"trait T { fn a() { } }
    mod m { fn b(x: Option<u8>) {
        let Some(x) = x else { return; };
        while x > 1 || x == 0 { break; }
        loop { break; }
        match x { 1 | 2 if x < 3 => (), _ => () }
        let _ = || { if x > 0 { 1 } else { 0 } };
        fn mala() { if true {} }
    } }"#;
    let f = analiziraj(kod).unwrap();
    assert_eq!(f.len(), 3);
    assert_eq!(f[1].cc, 9);
    assert_eq!(f[2].cc, 2);
    assert_eq!(f[0].ime, "T::a");
    assert_eq!(f[2].ime, "m::b::mala");
}
