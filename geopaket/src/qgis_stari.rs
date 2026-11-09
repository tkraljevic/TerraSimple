//! QGIS projekt koji je već u datoteci, kako ga je zadnji spremio QGIS ili TerraSimple.
//!
//! Odluke (9. 11. 2026.): TerraSimple zamjenjuje stil sloja u QGIS projektu samo kad ga je
//! korisnik u TerraSimpleu mijenjao. Inače ostaje sve što je korisnik za taj sloj namjestio u
//! QGIS-u: stil, natpisi, prozirnost i ostalo. Zato se iz starog projekta uzima cijeli element
//! `<maplayer>` sloja, a mijenjaju se samo naziv sloja i staza do datoteke.
//!
//! QGIS projekt sprema kao heksadecimalni tekst zip datoteke s jednom `.qgs` datotekom. Sve što
//! se ne da pročitati znači da starog projekta nema, pa se projekt složi iznova, kao i dosad.

use quick_xml::{Reader, events::Event};
use rusqlite::Connection;
use std::collections::BTreeMap;
use std::io::Read;
use std::ops::Range;

/// Najveća raspakirana `.qgs` datoteka koja se čita.
const NAJVISE_BAJTOVA: u64 = 64 * 1024 * 1024;

/// Element `<maplayer>` iz starog projekta i mjesta naziva i staze unutar njega.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SlojQgisa {
    xml: String,
    izvor: Range<usize>,
    naziv: Range<usize>,
}

impl SlojQgisa {
    /// Element s novim nazivom i stazom; oba moraju biti već pripremljena za XML.
    pub(crate) fn prilagodi(&self, izvor: &str, naziv: &str) -> String {
        let mut zamjene = [(self.izvor.clone(), izvor), (self.naziv.clone(), naziv)];
        zamjene.sort_by_key(|(r, _)| r.start);
        let mut izlaz = String::with_capacity(self.xml.len());
        let mut i = 0;
        for (r, tekst) in zamjene {
            izlaz.push_str(&self.xml[i..r.start]);
            izlaz.push_str(tekst);
            i = r.end;
        }
        izlaz.push_str(&self.xml[i..]);
        izlaz
    }
}

/// Slojevi starog projekta po oznaci sloja (`<id>`).
pub(crate) fn slojevi(c: &Connection, naziv_projekta: &str) -> BTreeMap<String, SlojQgisa> {
    zip_projekta(c, naziv_projekta)
        .and_then(|z| qgs_iz_zipa(&z))
        .map(|x| slojevi_iz_qgs(&x))
        .unwrap_or_default()
}

fn zip_projekta(c: &Connection, naziv_projekta: &str) -> Option<Vec<u8>> {
    // QGIS i TerraSimple projekt spremaju kao tekst. Bez tablice ili retka nema ni projekta.
    c.query_row(
        "SELECT content FROM qgis_projects WHERE name=?1",
        [naziv_projekta],
        |r| r.get::<_, String>(0),
    )
    .ok()
    .and_then(|t| iz_heksa(t.as_bytes()))
}

fn cifra(b: u8) -> Option<u8> {
    char::from(b).to_digit(16).map(|d| d as u8)
}

fn iz_heksa(t: &[u8]) -> Option<Vec<u8>> {
    let (parovi, ostatak) = t.as_chunks::<2>();
    if !ostatak.is_empty() {
        return None;
    }
    parovi
        .iter()
        .map(|[a, b]| Some(cifra(*a)? * 16 + cifra(*b)?))
        .collect()
}

/// Broj zapisan u zipu, najniži bajt prvi.
fn le(b: &[u8]) -> usize {
    b.iter().rev().fold(0, |v, x| v * 256 + usize::from(*x))
}

const KRAJ: &[u8] = b"PK\x05\x06";
const IMENIK: &[u8] = b"PK\x01\x02";
const LOKALNI: &[u8] = b"PK\x03\x04";

/// Zapis o kraju središnjeg imenika zip datoteke, 22 bajta.
fn kraj_imenika(z: &[u8]) -> Option<&[u8]> {
    let zadnji = z.len().checked_sub(22)?;
    (0..=zadnji)
        .rev()
        .map(|i| &z[i..i + 22])
        .find(|h| h.starts_with(KRAJ))
}

struct Ulaz<'a> {
    ime: &'a [u8],
    metoda: usize,
    velicina: usize,
    lokalni: usize,
    sljedeci: usize,
}

/// Ulaz središnjeg imenika na položaju `i`: 46 bajtova pa ime datoteke.
fn ulaz(z: &[u8], i: usize) -> Option<Ulaz<'_>> {
    let h = z.get(i..i + 46).filter(|h| h.starts_with(IMENIK))?;
    let ime_od = i + 46;
    let duljina_imena = le(&h[28..30]);
    Some(Ulaz {
        ime: z.get(ime_od..ime_od + duljina_imena)?,
        metoda: le(&h[10..12]),
        velicina: le(&h[20..24]),
        lokalni: le(&h[42..46]),
        sljedeci: ime_od + duljina_imena + le(&h[30..32]) + le(&h[32..34]),
    })
}

fn qgs_iz_zipa(z: &[u8]) -> Option<String> {
    let kraj = kraj_imenika(z)?;
    let mut i = le(&kraj[16..20]);
    for _ in 0..le(&kraj[10..12]) {
        let u = ulaz(z, i)?;
        if u.ime.ends_with(b".qgs") {
            return sadrzaj_ulaza(z, &u);
        }
        i = u.sljedeci;
    }
    None
}

/// Lokalno zaglavlje ima 30 bajtova, pa ime i dodatak, pa sažete podatke.
fn sadrzaj_ulaza(z: &[u8], u: &Ulaz<'_>) -> Option<String> {
    // QGIS projekt uvijek sažima (deflate, metoda 8).
    let h = z
        .get(u.lokalni..u.lokalni + 30)
        .filter(|h| h.starts_with(LOKALNI) && u.metoda == 8)?;
    let pocetak = u.lokalni + 30 + le(&h[26..28]) + le(&h[28..30]);
    let podaci = z.get(pocetak..pocetak + u.velicina)?;
    String::from_utf8(raspakiraj(podaci)?).ok()
}

fn raspakiraj(podaci: &[u8]) -> Option<Vec<u8>> {
    let mut izlaz = Vec::new();
    flate2::read::DeflateDecoder::new(podaci)
        .take(NAJVISE_BAJTOVA)
        .read_to_end(&mut izlaz)
        .ok()
        .map(|_| izlaz)
}

#[derive(Clone, Copy)]
enum Polje {
    Id,
    Izvor,
    Naziv,
}

fn polje(ime: &str) -> Option<Polje> {
    match ime {
        "id" => Some(Polje::Id),
        "datasource" => Some(Polje::Izvor),
        "layername" => Some(Polje::Naziv),
        _ => None,
    }
}

/// Element `<maplayer>` koji se upravo čita. Položaji su u cijelom tekstu projekta.
#[derive(Default)]
struct Otvoren {
    dubina: usize,
    pocetak: usize,
    dijete: Option<(Polje, usize)>,
    id: Option<Range<usize>>,
    izvor: Option<Range<usize>>,
    naziv: Option<Range<usize>>,
}

impl Otvoren {
    fn zatvori_dijete(&mut self, kraj: usize) {
        let Some((p, od)) = self.dijete.take() else {
            return;
        };
        let mjesto = match p {
            Polje::Id => &mut self.id,
            Polje::Izvor => &mut self.izvor,
            Polje::Naziv => &mut self.naziv,
        };
        mjesto.get_or_insert(od..kraj);
    }

    /// Položaji dolaze iz čitača, pa su unutar teksta i na granici znaka.
    fn dovrsi(self, kraj: usize, xml: &str) -> Option<(String, SlojQgisa)> {
        let (id, izvor, naziv) = (self.id?, self.izvor?, self.naziv?);
        let id = quick_xml::escape::unescape(&xml[id]).ok()?.into_owned();
        let p = self.pocetak;
        Some((
            id,
            SlojQgisa {
                xml: xml[p..kraj].to_owned(),
                izvor: izvor.start - p..izvor.end - p,
                naziv: naziv.start - p..naziv.end - p,
            },
        ))
    }
}

#[derive(Default)]
struct Stanje {
    dubina: usize,
    otvoren: Option<Otvoren>,
    gotovi: BTreeMap<String, SlojQgisa>,
}

impl Stanje {
    fn otvori(&mut self, ime: &str, prije: usize, poslije: usize) {
        self.dubina += 1;
        match &mut self.otvoren {
            None if ime == "maplayer" => {
                self.otvoren = Some(Otvoren {
                    dubina: self.dubina,
                    pocetak: prije,
                    ..Otvoren::default()
                });
            }
            Some(o) if self.dubina == o.dubina + 1 => o.dijete = polje(ime).map(|p| (p, poslije)),
            _ => {}
        }
    }

    fn zatvori(&mut self, prije: usize, poslije: usize, xml: &str) {
        let dubina = self.dubina;
        self.dubina = dubina.saturating_sub(1);
        let Some(o) = &mut self.otvoren else {
            return;
        };
        if dubina == o.dubina + 1 {
            o.zatvori_dijete(prije);
        } else if dubina == o.dubina
            && let Some((id, sloj)) = self.otvoren.take().and_then(|o| o.dovrsi(poslije, xml))
        {
            self.gotovi.entry(id).or_insert(sloj);
        }
    }
}

fn pozicija(r: &Reader<&[u8]>) -> usize {
    usize::try_from(r.buffer_position()).unwrap_or(usize::MAX)
}

fn slojevi_iz_qgs(xml: &str) -> BTreeMap<String, SlojQgisa> {
    let mut r = Reader::from_str(xml);
    let mut stanje = Stanje::default();
    loop {
        let prije = pozicija(&r);
        match r.read_event() {
            Ok(Event::Start(e)) => stanje.otvori(e.name().as_ref(), prije, pozicija(&r)),
            Ok(Event::End(_)) => stanje.zatvori(prije, pozicija(&r), xml),
            Ok(Event::Eof) => return stanje.gotovi,
            Ok(_) => {}
            Err(_) => return BTreeMap::new(),
        }
    }
}

#[cfg(test)]
#[path = "qgis_stari_testovi.rs"]
mod testovi;
