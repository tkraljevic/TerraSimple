use std::path::PathBuf;

/// Zašto nadogradnja nije uspjela. Tekst za korisnika slaže sučelje iz kataloga poruka.
#[derive(Debug)]
pub enum Greska {
    /// Veza s poslužiteljem nije uspjela ili je prekinuta.
    Veza(String),
    /// Poslužitelj je odgovorio, ali ne s 200.
    Odgovor(u16),
    /// Odgovor nije popis izdanja kakav očekujemo.
    NeocekivanOdgovor(String),
    /// Datoteka je veća od dopuštenog, pa preuzimanje nije nastavljeno.
    Prevelika { ime: String, najvise: u64 },
    /// Adresa paketa nije na stranici izdanja TerraSimplea.
    TudaAdresa(String),
    /// Potpis ne odgovara paketu ili nije potpis.
    KrivPotpis,
    /// Paket se ne može raspakirati ili u njemu nema programa.
    OstecenPaket(String),
    /// Program nije pokrenut iz paketa izdanja (npr. izgrađen iz izvornog koda).
    NijeIzPaketa(PathBuf),
    /// Zamjena programa na disku nije uspjela; prethodno stanje je vraćeno.
    Zamjena { mjesto: PathBuf, detalji: String },
}

/// Kratak tehnički opis za ispis u naredbenom retku; korisnik vidi rečenicu iz kataloga.
impl std::fmt::Display for Greska {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Greska {}

pub(crate) fn neocekivano(e: impl std::fmt::Display) -> Greska {
    Greska::NeocekivanOdgovor(e.to_string())
}

pub(crate) fn ostecen(e: impl std::fmt::Display) -> Greska {
    Greska::OstecenPaket(e.to_string())
}
