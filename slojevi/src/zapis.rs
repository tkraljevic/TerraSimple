//! Podaci izvora i usporedba za spremanje, odvojeni od korisničkih naredbi.
use crate::{Greska, IdMape, IdObjekta, IdSloja, Mapa, Projekt, Sloj, Stavka, Uredjivac};
use koordinate::Sustav;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum Vrijednost {
    Prazno,
    Cijeli(i64),
    Broj(f64),
    Tekst(String),
    Bajtovi(Vec<u8>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct ZapisObjekta {
    pub geometrija: Option<Arc<[u8]>>,
    pub fid: i64,
    pub nepoznati: BTreeMap<String, Vrijednost>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct IzvorSloja {
    pub srs_id: i32,
    pub sql: String,
    pub stupci: Vec<String>,
    pub indeksi: Vec<String>,
    pub okidaci: Vec<String>,
    pub stil: crate::Stil,
    pub sustav: Sustav,
    pub tablica: String,
    pub fid_stupac: String,
    pub geometrija_stupac: String,
    pub objekti: BTreeMap<IdObjekta, ZapisObjekta>,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Zapisi {
    pub najveci_id: u64,
    pub slojevi: BTreeMap<IdSloja, IzvorSloja>,
}
#[derive(Debug, Default, PartialEq)]
pub struct IzmjeneSpremanja {
    pub upisi: BTreeSet<(IdSloja, IdObjekta)>,
    pub obrisi: BTreeSet<(IdSloja, IdObjekta)>,
}
impl Projekt {
    pub fn zapisi(&self) -> &Zapisi {
        &self.zapisi
    }
    /// Uvoz provjerava stablo i geometrije prije predaje uređivaču.
    pub fn iz_zapisa(
        korijen: Vec<Stavka>,
        mape: BTreeMap<IdMape, Mapa>,
        slojevi: BTreeMap<IdSloja, Sloj>,
        zapisi: Zapisi,
    ) -> Result<Self, Greska> {
        let p = Self {
            korijen,
            mape,
            slojevi,
            zapisi,
        };
        p.provjeri_stablo()?;
        for (_, sloj) in p.slojevi() {
            provjeri_objekte(sloj)?;
        }
        Ok(p)
    }
    fn provjeri_stablo(&self) -> Result<(), Greska> {
        let mut red: Vec<_> = self.korijen.iter().map(|s| (*s, None)).collect();
        let mut vidjeno = BTreeSet::new();
        while let Some((stavka, roditelj)) = red.pop() {
            let kljuc = kljuc_stavke(stavka);
            if !vidjeno.insert(kljuc) {
                return Err(Greska::MapaUSebe);
            }
            red.extend(self.provjeri_stavku(stavka, roditelj)?);
        }
        if vidjeno.len() != self.mape.len() + self.slojevi.len() {
            return Err(Greska::NepoznataMapa);
        }
        Ok(())
    }
    fn provjeri_stavku(
        &self,
        s: Stavka,
        roditelj: Option<IdMape>,
    ) -> Result<Vec<(Stavka, Option<IdMape>)>, Greska> {
        match s {
            Stavka::Mapa(id) => {
                let m = self
                    .mapa(id)
                    .filter(|m| m.roditelj == roditelj)
                    .ok_or(Greska::NepoznataMapa)?;
                Ok(m.djeca.iter().map(|s| (*s, Some(id))).collect())
            }
            Stavka::Sloj(id) => {
                self.sloj(id)
                    .filter(|s| s.roditelj == roditelj)
                    .ok_or(Greska::NepoznatSloj)?;
                Ok(Vec::new())
            }
        }
    }
    /// Usporedba s posljednjim uspješnim spremanjem uključuje i poništene izmjene.
    pub fn izmjene_od(&self, prije: &Self) -> IzmjeneSpremanja {
        let mut r = IzmjeneSpremanja::default();
        for (s, sloj) in self.slojevi() {
            for (id, o) in &sloj.objekti {
                if prije.objekt(s, *id) != Some(o) {
                    r.upisi.insert((s, *id));
                }
            }
        }
        for (s, sloj) in prije.slojevi() {
            for id in sloj.objekti.keys() {
                if self.objekt(s, *id).is_none() {
                    r.obrisi.insert((s, *id));
                }
            }
        }
        r
    }
}
fn kljuc_stavke(s: Stavka) -> (u8, u64) {
    match s {
        Stavka::Mapa(id) => (0, id.0),
        Stavka::Sloj(id) => (1, id.0),
    }
}
fn provjeri_objekte(s: &Sloj) -> Result<(), Greska> {
    for (id, o) in &s.objekti {
        if id != &o.id {
            return Err(Greska::NepoznatObjekt);
        }
        if o.geometrija.vrsta() != s.vrsta {
            return Err(Greska::KrivaVrsta {
                ocekivana: s.vrsta,
                dobivena: o.geometrija.vrsta(),
            });
        }
        o.geometrija.provjeri()?;
    }
    Ok(())
}
impl Uredjivac {
    pub fn iz_projekta(projekt: Projekt) -> Self {
        Self::preuzmi_projekt(projekt)
    }
}
