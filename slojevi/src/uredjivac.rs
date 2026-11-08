//! Izvršavanje naredbi i povijest za poništavanje.

use crate::greska::Greska;
use crate::model::{
    Geometrija, IdMape, IdObjekta, IdSloja, Mapa, Objekt, Projekt, Sloj, StanjeSloja, Stavka,
    VrstaGeometrije,
};
use crate::naredba::{IzmjenaObjekta, Naredba, NaredbaObjekta, NaredbaStabla, Novo};
use crate::promjena::{Izvadak, Promjena, sloj_mut};
use crate::stil::{Stil, StilObjekta};
use std::collections::BTreeMap;

/// Jedan korak poništavanja. Ima više promjena kad je napravljen skupinom.
type Korak = Vec<Promjena>;

/// Jedan korak poništavanja s oznakom stanja.
#[derive(Debug)]
struct KorakPovijesti {
    promjene: Korak,
    oznaka: u64,
}

/// Drži projekt i povijest, a brojevi se ne ponavljaju ni nakon poništavanja.
#[derive(Debug, Default)]
pub struct Uredjivac {
    projekt: Projekt,
    natrag: Vec<KorakPovijesti>,
    naprijed: Vec<KorakPovijesti>,
    sljedeci_id: u64,
    sljedeca_oznaka: u64,
    spremljena_oznaka: u64,
    inacice: Inacice,
}

/// Što se promijenilo, da karta iznova priprema samo te slojeve. Izvan je projekta, jer poništena
/// izmjena vraća isti projekt, a pripremu ipak treba ponoviti.
#[derive(Debug, Default)]
struct Inacice {
    ukupno: u64,
    slojevi: BTreeMap<IdSloja, u64>,
}

impl Inacice {
    fn zabiljezi(&mut self, promjena: &Promjena) {
        self.ukupno += 1;
        for sloj in promjena.promijenjeni_slojevi().into_iter().flatten() {
            self.slojevi.insert(sloj, self.ukupno);
        }
    }
}

fn provjeri_naziv(naziv: &str) -> Result<(), Greska> {
    if naziv.trim().is_empty() {
        return Err(Greska::PrazanNaziv);
    }
    Ok(())
}

fn provjeri_geometriju(sloj: &Sloj, g: &Geometrija) -> Result<(), Greska> {
    if g.vrsta() != sloj.vrsta {
        return Err(Greska::KrivaVrsta {
            ocekivana: sloj.vrsta,
            dobivena: g.vrsta(),
        });
    }
    g.provjeri()
}

fn nova_stavka(stavka: Stavka, u: Option<IdMape>, mape: Vec<(IdMape, Mapa)>) -> Izvadak {
    Izvadak {
        stavka,
        roditelj: u,
        polozaj: usize::MAX,
        mape,
        slojevi: Vec::new(),
    }
}

/// Primjenjuje promjene redom. Ako jedna ne uspije, vraća projekt kakav je bio.
fn primijeni_redom(p: &mut Projekt, promjene: Korak) -> Result<Korak, Greska> {
    let mut obratno = Vec::with_capacity(promjene.len());
    for promjena in promjene {
        match promjena.primijeni(p) {
            Ok(o) => obratno.push(o),
            Err(greska) => {
                vrati(p, obratno)?;
                return Err(greska);
            }
        }
    }
    obratno.reverse();
    Ok(obratno)
}

fn vrati(p: &mut Projekt, mut obratno: Korak) -> Result<(), Greska> {
    obratno.reverse();
    for o in obratno {
        o.primijeni(p)?;
    }
    Ok(())
}

impl Uredjivac {
    pub fn novi() -> Self {
        Self::default()
    }

    pub fn projekt(&self) -> &Projekt {
        &self.projekt
    }

    /// Raste pri svakoj izmjeni projekta, i pri poništavanju.
    pub fn inacica(&self) -> u64 {
        self.inacice.ukupno
    }

    /// Raste kad se promijene objekti ili stil sloja.
    pub fn inacica_sloja(&self, sloj: IdSloja) -> u64 {
        self.inacice.slojevi.get(&sloj).copied().unwrap_or(0)
    }

    pub fn moze_natrag(&self) -> bool {
        !self.natrag.is_empty()
    }

    pub fn moze_naprijed(&self) -> bool {
        !self.naprijed.is_empty()
    }

    /// Izvršava naredbu kao jedan korak poništavanja.
    pub fn izvrsi(&mut self, naredba: impl Into<Naredba>) -> Result<Option<Novo>, Greska> {
        self.skupina(|s| s.izvrsi(naredba))
    }

    /// Sve naredbe iz `posao` su jedan korak: ili prođu sve, ili projekt ostane kakav je bio.
    pub fn skupina<R>(
        &mut self,
        posao: impl FnOnce(&mut Skupina<'_>) -> Result<R, Greska>,
    ) -> Result<R, Greska> {
        let mut skupina = Skupina {
            uredjivac: self,
            obratno: Vec::new(),
        };
        match posao(&mut skupina) {
            Ok(rezultat) => {
                let Skupina { obratno, .. } = skupina;
                if !obratno.is_empty() {
                    let mut korak = obratno;
                    korak.reverse();
                    self.sljedeca_oznaka += 1;
                    self.natrag.push(KorakPovijesti {
                        promjene: korak,
                        oznaka: self.sljedeca_oznaka,
                    });
                    self.naprijed.clear();
                }
                Ok(rezultat)
            }
            Err(greska) => {
                let Skupina { obratno, .. } = skupina;
                vrati(&mut self.projekt, obratno)?;
                Err(greska)
            }
        }
    }

    /// Oznaka trenutnog stanja u povijesti. 0 označava početno stanje bez izmjena.
    #[must_use]
    pub fn oznaka_stanja(&self) -> u64 {
        self.natrag.last().map_or(0, |k| k.oznaka)
    }

    /// Označava trenutno stanje projekta kao spremljeno.
    pub fn oznaci_spremljeno(&mut self) {
        self.spremljena_oznaka = self.oznaka_stanja();
    }

    /// Vraća je li projekt izmijenjen u odnosu na zadnje spremljeno stanje.
    #[must_use]
    pub fn je_li_izmijenjen(&self) -> bool {
        self.oznaka_stanja() != self.spremljena_oznaka
    }

    /// Vraća je li bilo što poništiti. Ako je sloj u međuvremenu zaključan, povijest ostaje netaknuta.
    pub fn ponisti(&mut self) -> Result<bool, Greska> {
        Self::prebaci(
            &mut self.projekt,
            &mut self.natrag,
            &mut self.naprijed,
            &mut self.inacice,
        )
    }

    pub fn ponovi(&mut self) -> Result<bool, Greska> {
        Self::prebaci(
            &mut self.projekt,
            &mut self.naprijed,
            &mut self.natrag,
            &mut self.inacice,
        )
    }

    fn prebaci(
        p: &mut Projekt,
        iz: &mut Vec<KorakPovijesti>,
        u: &mut Vec<KorakPovijesti>,
        inacice: &mut Inacice,
    ) -> Result<bool, Greska> {
        let Some(korak) = iz.last() else {
            return Ok(false);
        };
        korak
            .promjene
            .iter()
            .try_for_each(|promjena| promjena.dopustena(p))?;
        let Some(korak) = iz.pop() else {
            return Ok(false);
        };
        korak
            .promjene
            .iter()
            .for_each(|promjena| inacice.zabiljezi(promjena));
        let obratno = primijeni_redom(p, korak.promjene)?;
        u.push(KorakPovijesti {
            promjene: obratno,
            oznaka: korak.oznaka,
        });
        Ok(true)
    }

    /// Stanje izvora mijenja izvor, a ne korisnik, pa se ne poništava.
    pub fn postavi_stanje_sloja(
        &mut self,
        sloj: IdSloja,
        stanje: StanjeSloja,
    ) -> Result<(), Greska> {
        sloj_mut(&mut self.projekt, sloj)?.stanje = stanje;
        Ok(())
    }

    fn novi_id(&mut self) -> u64 {
        self.sljedeci_id += 1;
        self.sljedeci_id
    }

    fn pripremi(&mut self, naredba: Naredba) -> Result<(Promjena, Option<Novo>), Greska> {
        match naredba {
            Naredba::Stablo(n) => self.pripremi_stablo(n),
            Naredba::Objekt(n) => self.pripremi_objekt(n),
        }
    }

    fn pripremi_stablo(&mut self, n: NaredbaStabla) -> Result<(Promjena, Option<Novo>), Greska> {
        match n {
            NaredbaStabla::DodajMapu { u, naziv } => self.nova_mapa(u, naziv),
            NaredbaStabla::DodajSloj { u, naziv, vrsta } => self.novi_sloj(u, naziv, vrsta),
            NaredbaStabla::Preimenuj { stavka, naziv } => {
                provjeri_naziv(&naziv)?;
                Ok((Promjena::Preimenuj { stavka, naziv }, None))
            }
            NaredbaStabla::Premjesti { stavka, u, polozaj } => {
                Ok((Promjena::PremjestiStavku { stavka, u, polozaj }, None))
            }
            NaredbaStabla::Obrisi { stavka } => Ok((Promjena::Izvadi(stavka), None)),
            NaredbaStabla::PostaviStil { sloj, stil } => {
                Ok((Promjena::StilSloja { sloj, stil }, None))
            }
        }
    }

    fn nova_mapa(
        &mut self,
        u: Option<IdMape>,
        naziv: String,
    ) -> Result<(Promjena, Option<Novo>), Greska> {
        provjeri_naziv(&naziv)?;
        let id = IdMape(self.novi_id());
        let mapa = Mapa {
            naziv,
            roditelj: u,
            djeca: Vec::new(),
        };
        let izvadak = nova_stavka(Stavka::Mapa(id), u, vec![(id, mapa)]);
        Ok((Promjena::Umetni(izvadak), Some(Novo::Mapa(id))))
    }

    fn novi_sloj(
        &mut self,
        u: Option<IdMape>,
        naziv: String,
        vrsta: VrstaGeometrije,
    ) -> Result<(Promjena, Option<Novo>), Greska> {
        provjeri_naziv(&naziv)?;
        let id = IdSloja(self.novi_id());
        let sloj = Sloj {
            naziv,
            roditelj: u,
            vrsta,
            stanje: StanjeSloja::Uredivo,
            stil: Stil::default(),
            objekti: BTreeMap::new(),
        };
        let mut izvadak = nova_stavka(Stavka::Sloj(id), u, Vec::new());
        izvadak.slojevi.push((id, sloj));
        Ok((Promjena::Umetni(izvadak), Some(Novo::Sloj(id))))
    }

    fn pripremi_objekt(&mut self, n: NaredbaObjekta) -> Result<(Promjena, Option<Novo>), Greska> {
        match n {
            NaredbaObjekta::Dodaj {
                sloj,
                naziv,
                geometrija,
            } => self.novi_objekt(sloj, naziv, geometrija),
            NaredbaObjekta::Izmijeni {
                sloj,
                objekt,
                izmjena,
            } => {
                self.provjeri_izmjenu(sloj, &izmjena)?;
                let id = objekt;
                Ok((Promjena::IzmijeniObjekt { sloj, id, izmjena }, None))
            }
            NaredbaObjekta::Premjesti { iz, u, objekt } => {
                Ok((Promjena::PremjestiObjekt { iz, u, id: objekt }, None))
            }
            NaredbaObjekta::Obrisi { sloj, objekt } => {
                Ok((Promjena::IzvadiObjekt { sloj, id: objekt }, None))
            }
        }
    }

    fn novi_objekt(
        &mut self,
        sloj: IdSloja,
        naziv: String,
        geometrija: Geometrija,
    ) -> Result<(Promjena, Option<Novo>), Greska> {
        let s = self
            .projekt
            .slojevi
            .get(&sloj)
            .ok_or(Greska::NepoznatSloj)?;
        provjeri_geometriju(s, &geometrija)?;
        let id = IdObjekta(self.novi_id());
        let objekt = Objekt {
            id,
            naziv,
            opis: String::new(),
            geometrija,
            stil: StilObjekta::default(),
        };
        let promjena = Promjena::UmetniObjekt {
            sloj,
            objekt: Box::new(objekt),
        };
        Ok((promjena, Some(Novo::Objekt(id))))
    }

    fn provjeri_izmjenu(&self, sloj: IdSloja, izmjena: &IzmjenaObjekta) -> Result<(), Greska> {
        let s = self
            .projekt
            .slojevi
            .get(&sloj)
            .ok_or(Greska::NepoznatSloj)?;
        match izmjena {
            IzmjenaObjekta::Geometrija(g) => provjeri_geometriju(s, g),
            _ => Ok(()),
        }
    }
}

/// Naredbe koje čine jedan korak poništavanja.
#[derive(Debug)]
pub struct Skupina<'a> {
    uredjivac: &'a mut Uredjivac,
    obratno: Korak,
}

impl Skupina<'_> {
    /// Projekt unutar skupine, s dosad izvršenim naredbama.
    pub fn projekt(&self) -> &Projekt {
        &self.uredjivac.projekt
    }

    pub fn izvrsi(&mut self, naredba: impl Into<Naredba>) -> Result<Option<Novo>, Greska> {
        let (promjena, novo) = self.uredjivac.pripremi(naredba.into())?;
        self.uredjivac.inacice.zabiljezi(&promjena);
        self.obratno
            .push(promjena.primijeni(&mut self.uredjivac.projekt)?);
        Ok(novo)
    }
}

impl Uredjivac {
    pub(crate) fn preuzmi_projekt(projekt: Projekt) -> Self {
        let sljedeci_id = projekt
            .slojevi()
            .flat_map(|(id, s)| std::iter::once(id.0).chain(s.objekti.keys().map(|id| id.0)))
            .chain(projekt.mape.keys().map(|id| id.0))
            .chain(std::iter::once(projekt.zapisi.najveci_id))
            .chain(
                projekt
                    .zapisi
                    .slojevi
                    .values()
                    .flat_map(|s| s.objekti.values().filter_map(|o| u64::try_from(o.fid).ok())),
            )
            .max()
            .unwrap_or(0);
        Self {
            projekt,
            sljedeci_id,
            ..Self::default()
        }
    }
}
