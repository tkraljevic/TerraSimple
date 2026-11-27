/// Inačica programa u obliku `glavna.sporedna.popravak`, npr. 0.0.8.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Inacica(pub u32, pub u32, pub u32);

impl Inacica {
    /// Čita `0.0.8` ili oznaku `v0.0.8`. Sve drugo, i nastavci poput `-rc1`, vraća `None`.
    #[must_use]
    pub fn procitaj(tekst: &str) -> Option<Self> {
        let tekst = tekst.strip_prefix('v').unwrap_or(tekst);
        let brojevi: Vec<u32> = tekst
            .split('.')
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        match brojevi.as_slice() {
            [a, b, c] => Some(Self(*a, *b, *c)),
            _ => None,
        }
    }
}

impl std::fmt::Display for Inacica {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}
