//! Najmanje izmjene projekta. Svaka vraća promjenu koja je poništava, pa poništavanje ne kopira projekt
//! i brisanje mape s milijun vrhova samo premjesti podatke u povijest.

use crate::greska::Greska;
use crate::model::{IdMape, IdObjekta, IdSloja, Mapa, Objekt, Projekt, Sloj, Stavka};
use crate::naredba::IzmjenaObjekta;
use crate::stil::Stil;

/// Stavka izvađena iz stabla, s cijelim sadržajem.
#[derive(Clone, Debug)]
pub(crate) struct Izvadak {
    pub stavka: Stavka,
    pub roditelj: Option<IdMape>,
    pub polozaj: usize,
    pub mape: Vec<(IdMape, Mapa)>,
    pub slojevi: Vec<(IdSloja, Sloj)>,
}

#[derive(Clone, Debug)]
pub(crate) enum Promjena {
    Izvadi(Stavka),
    Umetni(Izvadak),
    Preimenuj {
        stavka: Stavka,
        naziv: String,
    },
    PremjestiStavku {
        stavka: Stavka,
        u: Option<IdMape>,
        polozaj: usize,
    },
    StilSloja {
        sloj: IdSloja,
        stil: Stil,
    },
    IzvadiObjekt {
        sloj: IdSloja,
        id: IdObjekta,
    },
    UmetniObjekt {
        sloj: IdSloja,
        objekt: Box<Objekt>,
    },
    IzmijeniObjekt {
        sloj: IdSloja,
        id: IdObjekta,
        izmjena: IzmjenaObjekta,
    },
    PremjestiObjekt {
        iz: IdSloja,
        u: IdSloja,
        id: IdObjekta,
    },
}

impl Promjena {
    /// Slojevi čiji se objekti ovom promjenom mijenjaju.
    fn slojevi_objekata(&self) -> [Option<IdSloja>; 2] {
        match self {
            Self::IzvadiObjekt { sloj, .. }
            | Self::UmetniObjekt { sloj, .. }
            | Self::IzmijeniObjekt { sloj, .. } => [Some(*sloj), None],
            Self::PremjestiObjekt { iz, u, .. } => [Some(*iz), Some(*u)],
            _ => [None, None],
        }
    }

    /// Slojevi kojima se ovom promjenom mijenja izgled na karti.
    pub(crate) fn promijenjeni_slojevi(&self) -> [Option<IdSloja>; 2] {
        match self {
            Self::StilSloja { sloj, .. } => [Some(*sloj), None],
            _ => self.slojevi_objekata(),
        }
    }

    /// Sloj se između izmjene i poništavanja može zaključati. Slojeva kojih još nema nije briga,
    /// jer nastaju ranijom promjenom iste skupine.
    pub(crate) fn dopustena(&self, p: &Projekt) -> Result<(), Greska> {
        let zakljucan = |id: &IdSloja| {
            p.slojevi
                .get(id)
                .is_some_and(|s| !s.stanje.smije_mijenjati())
        };
        if self.slojevi_objekata().iter().flatten().any(zakljucan) {
            return Err(Greska::SlojZakljucan);
        }
        Ok(())
    }

    /// Izvršava promjenu i vraća onu koja je poništava.
    pub(crate) fn primijeni(self, p: &mut Projekt) -> Result<Self, Greska> {
        self.dopustena(p)?;
        match self {
            Self::Izvadi(stavka) => izvadi(p, stavka).map(Self::Umetni),
            Self::Umetni(izvadak) => umetni(p, izvadak),
            Self::Preimenuj { stavka, naziv } => preimenuj(p, stavka, naziv),
            Self::PremjestiStavku { stavka, u, polozaj } => premjesti(p, stavka, u, polozaj),
            Self::StilSloja { sloj, stil } => postavi_stil(p, sloj, stil),
            objekt => objekt.primijeni_na_objekt(p),
        }
    }

    fn primijeni_na_objekt(self, p: &mut Projekt) -> Result<Self, Greska> {
        match self {
            Self::IzvadiObjekt { sloj, id } => izvadi_objekt(p, sloj, id),
            Self::UmetniObjekt { sloj, objekt } => umetni_objekt(p, sloj, *objekt),
            Self::IzmijeniObjekt { sloj, id, izmjena } => izmijeni_objekt(p, sloj, id, izmjena),
            Self::PremjestiObjekt { iz, u, id } => premjesti_objekt(p, iz, u, id),
            ostalo => Ok(ostalo),
        }
    }
}

fn mapa_mut(p: &mut Projekt, id: IdMape) -> Result<&mut Mapa, Greska> {
    p.mape.get_mut(&id).ok_or(Greska::NepoznataMapa)
}

pub(crate) fn sloj_mut(p: &mut Projekt, id: IdSloja) -> Result<&mut Sloj, Greska> {
    p.slojevi.get_mut(&id).ok_or(Greska::NepoznatSloj)
}

fn djeca_mut(p: &mut Projekt, roditelj: Option<IdMape>) -> Result<&mut Vec<Stavka>, Greska> {
    match roditelj {
        None => Ok(&mut p.korijen),
        Some(id) => Ok(&mut mapa_mut(p, id)?.djeca),
    }
}

fn roditelj_od(p: &Projekt, stavka: Stavka) -> Result<Option<IdMape>, Greska> {
    match stavka {
        Stavka::Mapa(id) => p
            .mape
            .get(&id)
            .map(|m| m.roditelj)
            .ok_or(Greska::NepoznataMapa),
        Stavka::Sloj(id) => p
            .slojevi
            .get(&id)
            .map(|s| s.roditelj)
            .ok_or(Greska::NepoznatSloj),
    }
}

fn postavi_roditelja(p: &mut Projekt, stavka: Stavka, u: Option<IdMape>) -> Result<(), Greska> {
    match stavka {
        Stavka::Mapa(id) => mapa_mut(p, id)?.roditelj = u,
        Stavka::Sloj(id) => sloj_mut(p, id)?.roditelj = u,
    }
    Ok(())
}

/// Vadi stavku iz roditeljeva popisa i vraća gdje je bila.
fn iskopcaj(p: &mut Projekt, stavka: Stavka) -> Result<(Option<IdMape>, usize), Greska> {
    let roditelj = roditelj_od(p, stavka)?;
    let djeca = djeca_mut(p, roditelj)?;
    let polozaj = djeca.iter().position(|s| *s == stavka);
    if let Some(i) = polozaj {
        djeca.remove(i);
    }
    Ok((roditelj, polozaj.unwrap_or(djeca.len())))
}

fn ukopcaj(
    p: &mut Projekt,
    stavka: Stavka,
    u: Option<IdMape>,
    polozaj: usize,
) -> Result<(), Greska> {
    let djeca = djeca_mut(p, u)?;
    djeca.insert(polozaj.min(djeca.len()), stavka);
    postavi_roditelja(p, stavka, u)
}

/// Bez rekurzije, jer dubinu stabla određuje datoteka koju je netko donio.
fn izvadi(p: &mut Projekt, stavka: Stavka) -> Result<Izvadak, Greska> {
    let (roditelj, polozaj) = iskopcaj(p, stavka)?;
    let mut izvadak = Izvadak {
        stavka,
        roditelj,
        polozaj,
        mape: Vec::new(),
        slojevi: Vec::new(),
    };
    let mut za_obilazak = vec![stavka];
    while let Some(s) = za_obilazak.pop() {
        match s {
            Stavka::Mapa(id) => {
                let mapa = p.mape.remove(&id).ok_or(Greska::NepoznataMapa)?;
                za_obilazak.extend(&mapa.djeca);
                izvadak.mape.push((id, mapa));
            }
            Stavka::Sloj(id) => {
                let sloj = p.slojevi.remove(&id).ok_or(Greska::NepoznatSloj)?;
                izvadak.slojevi.push((id, sloj));
            }
        }
    }
    Ok(izvadak)
}

fn umetni(p: &mut Projekt, izvadak: Izvadak) -> Result<Promjena, Greska> {
    // Prvo provjera, da neuspjela naredba ne ostavi mapu koja nije u stablu.
    djeca_mut(p, izvadak.roditelj)?;
    p.mape.extend(izvadak.mape);
    p.slojevi.extend(izvadak.slojevi);
    ukopcaj(p, izvadak.stavka, izvadak.roditelj, izvadak.polozaj)?;
    Ok(Promjena::Izvadi(izvadak.stavka))
}

fn preimenuj(p: &mut Projekt, stavka: Stavka, naziv: String) -> Result<Promjena, Greska> {
    let staro = match stavka {
        Stavka::Mapa(id) => std::mem::replace(&mut mapa_mut(p, id)?.naziv, naziv),
        Stavka::Sloj(id) => std::mem::replace(&mut sloj_mut(p, id)?.naziv, naziv),
    };
    Ok(Promjena::Preimenuj {
        stavka,
        naziv: staro,
    })
}

/// Je li `u` sama `mapa` ili neka njezina podmapa.
fn unutar_sebe(p: &Projekt, mapa: IdMape, u: Option<IdMape>) -> bool {
    let mut sada = u;
    while let Some(id) = sada {
        if id == mapa {
            return true;
        }
        sada = p.mape.get(&id).and_then(|m| m.roditelj);
    }
    false
}

fn premjesti(
    p: &mut Projekt,
    stavka: Stavka,
    u: Option<IdMape>,
    polozaj: usize,
) -> Result<Promjena, Greska> {
    if let Stavka::Mapa(id) = stavka
        && unutar_sebe(p, id, u)
    {
        return Err(Greska::MapaUSebe);
    }
    djeca_mut(p, u)?;
    let (stari, stari_polozaj) = iskopcaj(p, stavka)?;
    ukopcaj(p, stavka, u, polozaj)?;
    Ok(Promjena::PremjestiStavku {
        stavka,
        u: stari,
        polozaj: stari_polozaj,
    })
}

fn postavi_stil(p: &mut Projekt, sloj: IdSloja, stil: Stil) -> Result<Promjena, Greska> {
    let stari = std::mem::replace(&mut sloj_mut(p, sloj)?.stil, stil);
    Ok(Promjena::StilSloja { sloj, stil: stari })
}

fn izvadi_objekt(p: &mut Projekt, sloj: IdSloja, id: IdObjekta) -> Result<Promjena, Greska> {
    let objekt = sloj_mut(p, sloj)?
        .objekti
        .remove(&id)
        .ok_or(Greska::NepoznatObjekt)?;
    Ok(Promjena::UmetniObjekt {
        sloj,
        objekt: Box::new(objekt),
    })
}

fn umetni_objekt(p: &mut Projekt, sloj: IdSloja, objekt: Objekt) -> Result<Promjena, Greska> {
    let id = objekt.id;
    sloj_mut(p, sloj)?.objekti.insert(id, objekt);
    Ok(Promjena::IzvadiObjekt { sloj, id })
}

fn izmijeni_objekt(
    p: &mut Projekt,
    sloj: IdSloja,
    id: IdObjekta,
    izmjena: IzmjenaObjekta,
) -> Result<Promjena, Greska> {
    let o = sloj_mut(p, sloj)?
        .objekti
        .get_mut(&id)
        .ok_or(Greska::NepoznatObjekt)?;
    let staro = match izmjena {
        IzmjenaObjekta::Naziv(n) => IzmjenaObjekta::Naziv(std::mem::replace(&mut o.naziv, n)),
        IzmjenaObjekta::Opis(n) => IzmjenaObjekta::Opis(std::mem::replace(&mut o.opis, n)),
        IzmjenaObjekta::Geometrija(g) => {
            IzmjenaObjekta::Geometrija(std::mem::replace(&mut o.geometrija, g))
        }
        IzmjenaObjekta::Stil(s) => IzmjenaObjekta::Stil(std::mem::replace(&mut o.stil, s)),
    };
    Ok(Promjena::IzmijeniObjekt {
        sloj,
        id,
        izmjena: staro,
    })
}

fn premjesti_objekt(
    p: &mut Projekt,
    iz: IdSloja,
    u: IdSloja,
    id: IdObjekta,
) -> Result<Promjena, Greska> {
    let vrsta_cilja = sloj_mut(p, u)?.vrsta;
    let izvor = sloj_mut(p, iz)?;
    if izvor.vrsta != vrsta_cilja {
        return Err(Greska::KrivaVrsta {
            ocekivana: vrsta_cilja,
            dobivena: izvor.vrsta,
        });
    }
    let objekt = izvor.objekti.remove(&id).ok_or(Greska::NepoznatObjekt)?;
    sloj_mut(p, u)?.objekti.insert(id, objekt);
    Ok(Promjena::PremjestiObjekt { iz: u, u: iz, id })
}
