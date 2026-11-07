//! Stil sloja i stil jednog objekta. Debljine su u milimetrima, jer ih tako čuva i QGIS.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Boja {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OblikZnaka {
    Krug,
    Kvadrat,
    Trokut,
    Zvijezda,
}

/// Stil sloja, koji ima sve vrijednosti.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stil {
    pub boja_crte: Boja,
    pub debljina_mm: f64,
    pub boja_ispune: Boja,
    pub oblik_znaka: OblikZnaka,
    pub velicina_znaka_mm: f64,
}

impl Default for Stil {
    /// Narančasta boja crteža iz nacrta, a debljina 0,8 mm je oko 3 piksela na 96 dpi.
    fn default() -> Self {
        Self {
            boja_crte: Boja {
                r: 0xE6,
                g: 0x51,
                b: 0x00,
                a: 0xFF,
            },
            debljina_mm: 0.8,
            boja_ispune: Boja {
                r: 0xE6,
                g: 0x51,
                b: 0x00,
                a: 0x40,
            },
            oblik_znaka: OblikZnaka::Krug,
            velicina_znaka_mm: 3.0,
        }
    }
}

/// Stil jednog objekta. Prazna vrijednost znači da vrijedi stil sloja.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StilObjekta {
    pub boja_crte: Option<Boja>,
    pub debljina_mm: Option<f64>,
    pub boja_ispune: Option<Boja>,
    pub oblik_znaka: Option<OblikZnaka>,
    pub velicina_znaka_mm: Option<f64>,
}

impl StilObjekta {
    /// Stil kojim se objekt stvarno crta.
    pub fn na_stilu_sloja(&self, sloj: &Stil) -> Stil {
        Stil {
            boja_crte: self.boja_crte.unwrap_or(sloj.boja_crte),
            debljina_mm: self.debljina_mm.unwrap_or(sloj.debljina_mm),
            boja_ispune: self.boja_ispune.unwrap_or(sloj.boja_ispune),
            oblik_znaka: self.oblik_znaka.unwrap_or(sloj.oblik_znaka),
            velicina_znaka_mm: self.velicina_znaka_mm.unwrap_or(sloj.velicina_znaka_mm),
        }
    }
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn stil_objekta_mijenja_samo_ono_sto_je_zadano() {
        let sloj = Stil::default();
        assert_eq!(StilObjekta::default().na_stilu_sloja(&sloj), sloj);
        let plavo = Boja {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        };
        let svi = StilObjekta {
            boja_crte: Some(plavo),
            debljina_mm: Some(2.0),
            boja_ispune: Some(plavo),
            oblik_znaka: Some(OblikZnaka::Zvijezda),
            velicina_znaka_mm: Some(5.0),
        };
        let r = svi.na_stilu_sloja(&sloj);
        assert_eq!(r.boja_crte, plavo);
        assert_eq!(r.debljina_mm, 2.0);
        assert_eq!(r.boja_ispune, plavo);
        assert_eq!(r.oblik_znaka, OblikZnaka::Zvijezda);
        assert_eq!(r.velicina_znaka_mm, 5.0);
    }
}
