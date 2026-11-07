//! Što se s projektom može napraviti. Sučelje i MCP šalju iste naredbe.

use crate::model::{Geometrija, IdMape, IdObjekta, IdSloja, Stavka, VrstaGeometrije};
use crate::stil::{Stil, StilObjekta};

#[derive(Clone, Debug, PartialEq)]
pub enum Naredba {
    Stablo(NaredbaStabla),
    Objekt(NaredbaObjekta),
}

/// Naredbe nad mapama i slojevima. `u: None` znači vrh stabla, a položaj iza kraja znači na kraj.
#[derive(Clone, Debug, PartialEq)]
pub enum NaredbaStabla {
    DodajMapu {
        u: Option<IdMape>,
        naziv: String,
    },
    DodajSloj {
        u: Option<IdMape>,
        naziv: String,
        vrsta: VrstaGeometrije,
    },
    Preimenuj {
        stavka: Stavka,
        naziv: String,
    },
    Premjesti {
        stavka: Stavka,
        u: Option<IdMape>,
        polozaj: usize,
    },
    /// Mapa se briše sa svime što sadrži.
    Obrisi {
        stavka: Stavka,
    },
    PostaviStil {
        sloj: IdSloja,
        stil: Stil,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum NaredbaObjekta {
    Dodaj {
        sloj: IdSloja,
        naziv: String,
        geometrija: Geometrija,
    },
    Izmijeni {
        sloj: IdSloja,
        objekt: IdObjekta,
        izmjena: IzmjenaObjekta,
    },
    Premjesti {
        iz: IdSloja,
        u: IdSloja,
        objekt: IdObjekta,
    },
    Obrisi {
        sloj: IdSloja,
        objekt: IdObjekta,
    },
}

/// Jedno svojstvo objekta. Vrhovi se šalju kao cijela geometrija, jednom po povlačenju.
#[derive(Clone, Debug, PartialEq)]
pub enum IzmjenaObjekta {
    Naziv(String),
    Opis(String),
    Geometrija(Geometrija),
    Stil(StilObjekta),
}

/// Što je naredba stvorila, da pozivatelj dalje može raditi s tim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Novo {
    Mapa(IdMape),
    Sloj(IdSloja),
    Objekt(IdObjekta),
}

impl From<NaredbaStabla> for Naredba {
    fn from(n: NaredbaStabla) -> Self {
        Self::Stablo(n)
    }
}

impl From<NaredbaObjekta> for Naredba {
    fn from(n: NaredbaObjekta) -> Self {
        Self::Objekt(n)
    }
}
