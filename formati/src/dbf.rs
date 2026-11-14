//! Tablica atributa shapefilea (`.dbf`). Tekst je u UTF-8 ili u kodnoj stranici 1250. Datoteke iz
//! Hrvatskih voda u `.cpg` znaju pisati UTF-8 iako su u 1250, pa se gleda sadržaj, a ne `.cpg`:
//! tekst koji nije ispravan UTF-8 čita se kao 1250.

/// Znakovi kodne stranice 1250 od 0x80 do 0xFF.
const CP1250: [char; 128] = [
    '\u{20AC}', '\u{FFFD}', '\u{201A}', '\u{FFFD}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{FFFD}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{015A}', '\u{0164}', '\u{017D}', '\u{0179}',
    '\u{FFFD}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{FFFD}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{015B}', '\u{0165}', '\u{017E}', '\u{017A}',
    '\u{00A0}', '\u{02C7}', '\u{02D8}', '\u{0141}', '\u{00A4}', '\u{0104}', '\u{00A6}', '\u{00A7}',
    '\u{00A8}', '\u{00A9}', '\u{015E}', '\u{00AB}', '\u{00AC}', '\u{00AD}', '\u{00AE}', '\u{017B}',
    '\u{00B0}', '\u{00B1}', '\u{02DB}', '\u{0142}', '\u{00B4}', '\u{00B5}', '\u{00B6}', '\u{00B7}',
    '\u{00B8}', '\u{0105}', '\u{015F}', '\u{00BB}', '\u{013D}', '\u{02DD}', '\u{013E}', '\u{017C}',
    '\u{0154}', '\u{00C1}', '\u{00C2}', '\u{0102}', '\u{00C4}', '\u{0139}', '\u{0106}', '\u{00C7}',
    '\u{010C}', '\u{00C9}', '\u{0118}', '\u{00CB}', '\u{011A}', '\u{00CD}', '\u{00CE}', '\u{010E}',
    '\u{0110}', '\u{0143}', '\u{0147}', '\u{00D3}', '\u{00D4}', '\u{0150}', '\u{00D6}', '\u{00D7}',
    '\u{0158}', '\u{016E}', '\u{00DA}', '\u{0170}', '\u{00DC}', '\u{00DD}', '\u{0162}', '\u{00DF}',
    '\u{0155}', '\u{00E1}', '\u{00E2}', '\u{0103}', '\u{00E4}', '\u{013A}', '\u{0107}', '\u{00E7}',
    '\u{010D}', '\u{00E9}', '\u{0119}', '\u{00EB}', '\u{011B}', '\u{00ED}', '\u{00EE}', '\u{010F}',
    '\u{0111}', '\u{0144}', '\u{0148}', '\u{00F3}', '\u{00F4}', '\u{0151}', '\u{00F6}', '\u{00F7}',
    '\u{0159}', '\u{016F}', '\u{00FA}', '\u{0171}', '\u{00FC}', '\u{00FD}', '\u{0163}', '\u{02D9}',
];

fn znak_1250(c: u8) -> char {
    CP1250
        .get(usize::from(c).wrapping_sub(0x80))
        .copied()
        .unwrap_or(char::from(c))
}

/// UTF-8 ako je ispravan, inače kodna stranica 1250.
pub(crate) fn dekodiraj(b: &[u8]) -> String {
    std::str::from_utf8(b).map_or_else(|_| b.iter().map(|c| znak_1250(*c)).collect(), str::to_owned)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Tablica {
    pub nazivi: Vec<String>,
    /// Vrijednosti po retcima, redom kao objekti u `.shp`.
    pub redci: Vec<Vec<String>>,
}

impl Tablica {
    /// Vrijednost prvog stupca čiji je naziv na popisu, bez obzira na velika i mala slova.
    pub fn vrijednost(&self, redak: usize, nazivi: &[&str]) -> Option<&str> {
        let r = self.redci.get(redak)?;
        nazivi.iter().find_map(|n| {
            let i = self.nazivi.iter().position(|s| s.eq_ignore_ascii_case(n))?;
            r.get(i).map(String::as_str).filter(|v| !v.is_empty())
        })
    }
}

fn broj(b: &[u8], od: usize, duljina: usize) -> Option<usize> {
    b.get(od..od + duljina).map(|s| {
        let mut v = [0u8; 4];
        v[..duljina].copy_from_slice(s);
        u32::from_le_bytes(v) as usize
    })
}

/// Opisi stupaca: naziv i širina. Popis završava bajtom 0x0D.
fn stupci(b: &[u8], zaglavlje: usize) -> Vec<(String, usize)> {
    (32..zaglavlje.saturating_sub(1))
        .step_by(32)
        .map_while(|i| b.get(i..i + 32).filter(|o| o[0] != 0x0D))
        .map(|o| {
            let ime = o[..11].split(|c| *c == 0).next().unwrap_or_default();
            (dekodiraj(ime).trim().to_owned(), usize::from(o[16]))
        })
        .collect()
}

fn redak(zapis: &[u8], sirine: &[usize]) -> Vec<String> {
    let mut od = 1;
    sirine
        .iter()
        .map(|s| {
            let polje = zapis.get(od..od + s).unwrap_or_default();
            od += s;
            dekodiraj(polje).trim().to_owned()
        })
        .collect()
}

/// Čita tablicu. Ništa ako zaglavlje nije ispravno.
pub(crate) fn procitaj(b: &[u8]) -> Option<Tablica> {
    let ((broj_redaka, zaglavlje), duljina) =
        broj(b, 4, 4).zip(broj(b, 8, 2)).zip(broj(b, 10, 2))?;
    let (nazivi, sirine): (Vec<String>, Vec<usize>) = stupci(b, zaglavlje).into_iter().unzip();
    let redci = (0..broj_redaka)
        .map(|r| zaglavlje + r * duljina)
        .map(|od| {
            b.get(od..od + duljina)
                .map(|z| redak(z, &sirine))
                .unwrap_or_default()
        })
        .collect();
    Some(Tablica { nazivi, redci })
}
