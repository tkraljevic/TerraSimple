use slojevi::{IdMape, IdObjekta, IdSloja, VrstaGeometrije};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alat {
    Odaberi,
    Tocka,
    Linija,
    Poligon,
    Mjeri,
}

impl Alat {
    pub const SVI: [Self; 5] = [
        Self::Odaberi,
        Self::Tocka,
        Self::Linija,
        Self::Poligon,
        Self::Mjeri,
    ];

    #[must_use]
    pub const fn ikona(self) -> &'static str {
        match self {
            Self::Odaberi => "odaberi",
            Self::Tocka => "tocka",
            Self::Linija => "linija",
            Self::Poligon => "poligon",
            Self::Mjeri => "ravnalo",
        }
    }

    /// Vrsta objekta koji alat crta. Odaberi i Mjeri ne crtaju objekte.
    #[must_use]
    pub const fn vrsta(self) -> Option<VrstaGeometrije> {
        match self {
            Self::Tocka => Some(VrstaGeometrije::Tocka),
            Self::Linija => Some(VrstaGeometrije::Linija),
            Self::Poligon => Some(VrstaGeometrije::Poligon),
            Self::Odaberi | Self::Mjeri => None,
        }
    }

    #[must_use]
    pub const fn kljuc_poruke(self) -> &'static str {
        match self {
            Self::Odaberi => "alat-odaberi",
            Self::Tocka => "alat-tocka",
            Self::Linija => "alat-linija",
            Self::Poligon => "alat-poligon",
            Self::Mjeri => "alat-mjeri",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Odabir {
    Mapa(IdMape),
    Sloj(IdSloja),
    Objekt(IdSloja, IdObjekta),
}

/// Prozorčić Novi sloj po zaslonu F: naziv, vrsta i mapa.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoviSloj {
    pub naziv: String,
    pub vrsta: VrstaGeometrije,
    pub mapa: Option<IdMape>,
    /// Korisnik je pokušao napraviti sloj bez naziva.
    pub prazan_naziv: bool,
}

/// Naziv upravo nacrtanog objekta, dok ga korisnik upisuje.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NazivNovog {
    pub sloj: IdSloja,
    pub objekt: IdObjekta,
    pub naziv: String,
    /// Prvi put se cijeli predloženi naziv označi, pa ga upisivanje zamijeni.
    pub oznaci: bool,
}

impl NazivNovog {
    #[must_use]
    pub fn novi(sloj: IdSloja, objekt: IdObjekta, naziv: &str) -> Self {
        Self {
            sloj,
            objekt,
            naziv: naziv.to_owned(),
            oznaci: true,
        }
    }
}

/// Ploča Stil, otvorena za sloj ili za jedan njegov objekt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UredjivanjeStila {
    pub sloj: IdSloja,
    /// Objekt iz čijeg je retka ploča otvorena. Tada se bira cijeli sloj ili samo on.
    pub objekt: Option<IdObjekta>,
    pub samo_objekt: bool,
    /// Izmjena je počela dok je tipka miša još dolje, pa joj se sljedeće pridružuju.
    pub potez: bool,
    /// Otvorena je kartica Natpis, koja uvijek vrijedi za cijeli sloj.
    pub natpis: bool,
}

impl UredjivanjeStila {
    /// Iz retka sloja mijenja se sloj, a iz retka objekta zadano samo taj objekt. Mapa nema stil.
    #[must_use]
    pub const fn za(odabir: Odabir) -> Option<Self> {
        let (sloj, objekt) = match odabir {
            Odabir::Mapa(_) => return None,
            Odabir::Sloj(s) => (s, None),
            Odabir::Objekt(s, o) => (s, Some(o)),
        };
        Some(Self {
            sloj,
            objekt,
            samo_objekt: objekt.is_some(),
            potez: false,
            natpis: false,
        })
    }

    /// Objekt čiji se stil mijenja, ako se ne mijenja cijeli sloj.
    #[must_use]
    pub fn ciljni_objekt(&self) -> Option<IdObjekta> {
        self.objekt.filter(|_| self.samo_objekt && !self.natpis)
    }
}

pub struct StanjePrikaza {
    zatvorene_mape: BTreeSet<IdMape>,
    otvoreni_slojevi: BTreeSet<IdSloja>,
    skrivene_mape: BTreeSet<IdMape>,
    skriveni_slojevi: BTreeSet<IdSloja>,
    skriveni_objekti: BTreeSet<(IdSloja, IdObjekta)>,
    pub skrij_podnaslov_mape: BTreeSet<IdMape>,
    pub ogranicenje_objekata: BTreeMap<IdSloja, usize>,
    pub oznake_slojeva: BTreeMap<IdSloja, String>,
    pub ikone_slojeva: BTreeMap<IdSloja, (String, String)>,
    pub odabrano: Option<Odabir>,
    /// Objekti odabrani na karti, da ih ploča istakne i kad ih je više.
    pub odabrani_na_karti: Vec<(IdSloja, IdObjekta)>,
    /// Prozorčić s nazivom upravo nacrtanog objekta.
    pub naziv_novog: Option<NazivNovog>,
    pub preimenovanje: Option<(Odabir, String)>,
    /// Cijeli naziv u polju preimenovanja je označen, pa ga prvi upisani znak zamjenjuje.
    pub naziv_oznacen: bool,
    /// Izbornik retka u ploči traži brisanje odabranog; prozor ga izvodi, s upozorenjem ako treba.
    pub zatrazeno_brisanje: bool,
    /// Gumb „Uvezi sloj” traži uvoz; prozor otvara birač datoteka.
    pub zatrazen_uvoz: bool,
    /// Mapa, sloj ili objekt koji korisnik upravo povlači mišem u ploči Slojevi.
    pub povuceno: Option<Odabir>,
    pub novi_sloj: Option<NoviSloj>,
    /// Dok je postavljeno, umjesto ploče Slojevi crta se ploča Stil.
    pub stil: Option<UredjivanjeStila>,
    pub alat: Alat,
    pub podloga: usize,
    pub ploca_lijevo: bool,
    pub ploca_otvorena: bool,
    pub tema_tamna: bool,
    pub povlaci_plocu: bool,
    pub redni_novog_sloja: usize,
    pub redni_nove_mape: usize,
    pub prikazi_primjer: bool,
}

impl Default for StanjePrikaza {
    fn default() -> Self {
        Self::novo()
    }
}

impl StanjePrikaza {
    #[must_use]
    pub fn novo() -> Self {
        Self {
            zatvorene_mape: BTreeSet::new(),
            otvoreni_slojevi: BTreeSet::new(),
            skrivene_mape: BTreeSet::new(),
            skriveni_slojevi: BTreeSet::new(),
            skriveni_objekti: BTreeSet::new(),
            skrij_podnaslov_mape: BTreeSet::new(),
            ogranicenje_objekata: BTreeMap::new(),
            oznake_slojeva: BTreeMap::new(),
            ikone_slojeva: BTreeMap::new(),
            odabrano: None,
            odabrani_na_karti: Vec::new(),
            naziv_novog: None,
            preimenovanje: None,
            naziv_oznacen: false,
            zatrazeno_brisanje: false,
            zatrazen_uvoz: false,
            povuceno: None,
            novi_sloj: None,
            stil: None,
            alat: Alat::Odaberi,
            podloga: 0,
            ploca_lijevo: false,
            ploca_otvorena: true,
            tema_tamna: false,
            povlaci_plocu: false,
            redni_novog_sloja: 1,
            redni_nove_mape: 1,
            prikazi_primjer: false,
        }
    }

    #[must_use]
    pub fn je_mapa_otvorena(&self, id: IdMape) -> bool {
        !self.zatvorene_mape.contains(&id)
    }

    pub fn postavi_otvorenost_mape(&mut self, id: IdMape, otvoreno: bool) {
        if otvoreno {
            self.zatvorene_mape.remove(&id);
        } else {
            self.zatvorene_mape.insert(id);
        }
    }

    pub fn prebaci_otvorenost_mape(&mut self, id: IdMape) {
        let novo = !self.je_mapa_otvorena(id);
        self.postavi_otvorenost_mape(id, novo);
    }

    #[must_use]
    pub fn je_sloj_otvoren(&self, id: IdSloja) -> bool {
        self.otvoreni_slojevi.contains(&id)
    }

    pub fn postavi_otvorenost_sloja(&mut self, id: IdSloja, otvoreno: bool) {
        if otvoreno {
            self.otvoreni_slojevi.insert(id);
        } else {
            self.otvoreni_slojevi.remove(&id);
        }
    }

    pub fn prebaci_otvorenost_sloja(&mut self, id: IdSloja) {
        let novo = !self.je_sloj_otvoren(id);
        self.postavi_otvorenost_sloja(id, novo);
    }

    #[must_use]
    pub fn je_mapa_vidljiva(&self, id: IdMape) -> bool {
        !self.skrivene_mape.contains(&id)
    }

    pub fn postavi_vidljivost_mape(&mut self, id: IdMape, vidljivo: bool) {
        if vidljivo {
            self.skrivene_mape.remove(&id);
        } else {
            self.skrivene_mape.insert(id);
        }
    }

    pub fn prebaci_vidljivost_mape(&mut self, id: IdMape) {
        let novo = !self.je_mapa_vidljiva(id);
        self.postavi_vidljivost_mape(id, novo);
    }

    #[must_use]
    pub fn je_sloj_vidljiv(&self, id: IdSloja) -> bool {
        !self.skriveni_slojevi.contains(&id)
    }

    pub fn postavi_vidljivost_sloja(&mut self, id: IdSloja, vidljivo: bool) {
        if vidljivo {
            self.skriveni_slojevi.remove(&id);
        } else {
            self.skriveni_slojevi.insert(id);
        }
    }

    pub fn prebaci_vidljivost_sloja(&mut self, id: IdSloja) {
        let novo = !self.je_sloj_vidljiv(id);
        self.postavi_vidljivost_sloja(id, novo);
    }

    #[must_use]
    pub fn je_objekt_vidljiv(&self, sloj: IdSloja, objekt: IdObjekta) -> bool {
        !self.skriveni_objekti.contains(&(sloj, objekt))
    }

    pub fn postavi_vidljivost_objekta(&mut self, sloj: IdSloja, objekt: IdObjekta, vidljivo: bool) {
        if vidljivo {
            self.skriveni_objekti.remove(&(sloj, objekt));
        } else {
            self.skriveni_objekti.insert((sloj, objekt));
        }
    }

    pub fn prebaci_vidljivost_objekta(&mut self, sloj: IdSloja, objekt: IdObjekta) {
        let novo = !self.je_objekt_vidljiv(sloj, objekt);
        self.postavi_vidljivost_objekta(sloj, objekt, novo);
    }

    pub fn zapocni_preimenovanje(&mut self, ciljani: Odabir, trenutni_naziv: &str) {
        self.odabrano = Some(ciljani);
        self.preimenovanje = Some((ciljani, trenutni_naziv.to_owned()));
        self.naziv_oznacen = true;
    }

    /// Upisani znakovi zamjenjuju označeni naziv, a inače se dodaju na kraj.
    pub fn upisi_u_naziv(&mut self, uneseno: &str) {
        let oznacen = std::mem::take(&mut self.naziv_oznacen);
        if let Some((_, tekst)) = &mut self.preimenovanje {
            if oznacen {
                tekst.clear();
            }
            tekst.extend(uneseno.chars().filter(|znak| !znak.is_control()));
        }
    }

    /// Backspace briše označeni naziv cijeli, a inače zadnji znak.
    pub fn obrisi_znak_naziva(&mut self) {
        let oznacen = std::mem::take(&mut self.naziv_oznacen);
        if let Some((_, tekst)) = &mut self.preimenovanje {
            if oznacen {
                tekst.clear();
            } else {
                let _ = tekst.pop();
            }
        }
    }

    pub fn otkazi_preimenovanje(&mut self) {
        self.preimenovanje = None;
    }
}

#[cfg(test)]
mod testovi {
    use super::*;
    use slojevi::{NaredbaStabla, Novo, Uredjivac, VrstaGeometrije};

    #[test]
    fn vidljivost_i_otvorenost_ne_ulaze_u_povijest_uredjivaca() {
        let mut uredjivac = Uredjivac::novi();
        let mut stanje = StanjePrikaza::default();

        let Some(Novo::Mapa(id_mape)) = uredjivac
            .izvrsi(NaredbaStabla::DodajMapu {
                u: None,
                naziv: "Teren".into(),
            })
            .unwrap()
        else {
            panic!("očekivana mapa");
        };
        let Some(Novo::Sloj(id_sloja)) = uredjivac
            .izvrsi(NaredbaStabla::DodajSloj {
                u: Some(id_mape),
                naziv: "Parcele".into(),
                vrsta: VrstaGeometrije::Poligon,
            })
            .unwrap()
        else {
            panic!("očekivan sloj");
        };

        assert!(stanje.je_mapa_otvorena(id_mape));
        assert!(stanje.je_sloj_vidljiv(id_sloja));

        stanje.prebaci_otvorenost_mape(id_mape);
        stanje.prebaci_vidljivost_sloja(id_sloja);
        assert!(!stanje.je_mapa_otvorena(id_mape));
        assert!(!stanje.je_sloj_vidljiv(id_sloja));

        assert!(uredjivac.ponisti().unwrap());
        assert!(uredjivac.projekt().mapa(id_mape).unwrap().djeca.is_empty());
        assert!(!stanje.je_mapa_otvorena(id_mape));
        assert!(!stanje.je_sloj_vidljiv(id_sloja));
    }

    #[test]
    fn alati_i_ostale_zastavice_stanja_prikaza() {
        let mut stanje = StanjePrikaza::novo();
        for alat in Alat::SVI {
            assert!(!alat.ikona().is_empty());
            assert!(!alat.kljuc_poruke().is_empty());
        }
        let mapa = IdMape(1);
        let sloj = IdSloja(2);
        let objekt = IdObjekta(3);

        stanje.prebaci_vidljivost_mape(mapa);
        assert!(!stanje.je_mapa_vidljiva(mapa));
        stanje.prebaci_vidljivost_mape(mapa);
        assert!(stanje.je_mapa_vidljiva(mapa));

        stanje.prebaci_otvorenost_sloja(sloj);
        assert!(stanje.je_sloj_otvoren(sloj));
        stanje.prebaci_otvorenost_sloja(sloj);
        assert!(!stanje.je_sloj_otvoren(sloj));

        stanje.prebaci_vidljivost_objekta(sloj, objekt);
        assert!(!stanje.je_objekt_vidljiv(sloj, objekt));
        stanje.prebaci_vidljivost_objekta(sloj, objekt);
        assert!(stanje.je_objekt_vidljiv(sloj, objekt));

        stanje.zapocni_preimenovanje(Odabir::Sloj(sloj), "Stari");
        assert_eq!(
            stanje.preimenovanje,
            Some((Odabir::Sloj(sloj), "Stari".into()))
        );
        stanje.otkazi_preimenovanje();
        assert!(stanje.preimenovanje.is_none());
    }

    #[test]
    fn oznaceni_naziv_se_zamjenjuje_a_inace_dopisuje() {
        let mut stanje = StanjePrikaza::novo();
        // Bez preimenovanja upis ne radi ništa.
        stanje.upisi_u_naziv("x");
        stanje.obrisi_znak_naziva();
        assert!(stanje.preimenovanje.is_none());

        let cilj = Odabir::Mapa(IdMape(1));
        stanje.zapocni_preimenovanje(cilj, "Mapa 1");
        assert!(stanje.naziv_oznacen);
        stanje.upisi_u_naziv("Kanali\n");
        assert_eq!(stanje.preimenovanje, Some((cilj, "Kanali".into())));
        assert!(!stanje.naziv_oznacen);
        stanje.upisi_u_naziv(" 2");
        stanje.obrisi_znak_naziva();
        assert_eq!(stanje.preimenovanje, Some((cilj, "Kanali ".into())));

        stanje.zapocni_preimenovanje(cilj, "Mapa 1");
        stanje.obrisi_znak_naziva();
        assert_eq!(stanje.preimenovanje, Some((cilj, String::new())));
        assert!(!stanje.naziv_oznacen);
    }

    #[test]
    fn svaki_alat_ima_svoju_ikonu_u_skladistu() {
        let skladiste = crate::ikone::SkladisteIkona::novo();
        let mut vidjene = BTreeSet::new();
        for alat in Alat::SVI {
            let ikona = alat.ikona();
            assert!(
                skladiste.manifest().izvozi.iter().any(|i| i.ikona == ikona),
                "{alat:?} nema ikonu {ikona}"
            );
            assert!(
                vidjene.insert(ikona),
                "{alat:?} dijeli ikonu s drugim alatom"
            );
        }
    }
}
