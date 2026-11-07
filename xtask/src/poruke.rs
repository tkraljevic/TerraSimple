use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};

pub fn broj(vrijednost: f64, decimale: usize) -> String {
    format!("{vrijednost:.decimale$}").replace('.', ",")
}

pub fn tekst(kljuc: &str, vrijednosti: &[(&str, String)]) -> String {
    let jezik = unic_langid::LanguageIdentifier::default();
    let mut katalog = FluentBundle::new(vec![jezik]);
    katalog.set_use_isolating(false);
    let resurs = match FluentResource::try_new(include_str!("../prijevodi/hr.ftl").into()) {
        Ok(r) => r,
        Err(_) => return kljuc.into(),
    };
    if katalog.add_resource(resurs).is_err() {
        return kljuc.into();
    }
    oblikuj(&katalog, kljuc, vrijednosti).unwrap_or_else(|| kljuc.into())
}

fn oblikuj(
    katalog: &FluentBundle<FluentResource>,
    kljuc: &str,
    vrijednosti: &[(&str, String)],
) -> Option<String> {
    let poruka = katalog.get_message(kljuc)?.value()?;
    let mut argumenti = FluentArgs::new();
    for (ime, vrijednost) in vrijednosti {
        argumenti.set(*ime, vrijednost.as_str());
    }
    let mut greske = Vec::new();
    let tekst = katalog.format_pattern(poruka, Some(&argumenti), &mut greske);
    if greske.is_empty() {
        Some(tekst.into_owned())
    } else {
        None
    }
}
