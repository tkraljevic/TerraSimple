//! KMZ je ZIP s jednom glavnom KML datotekom, obično `doc.kml`, i slikama. Arhiva je nepouzdana,
//! pa se odbijaju putanje izvan nje i sadržaj koji se raspakira u neočekivano veliku datoteku.

use crate::citac::{GreskaKml, procitaj_iz};
use crate::model::DokumentKml;
use std::fs::File;
use std::io::{self, BufReader, Read};

/// `myplaces.kml` ima 147 MB, pa granica ostavlja dovoljno mjesta i za veće zbirke.
const NAJVECA_VELICINA: u64 = 1024 * 1024 * 1024;
/// Tekst se dobro sažima, ali ne bolje od oko 1:50. Veći omjer znači podmetnutu arhivu.
const NAJVECI_OMJER: u64 = 200;
/// Omjer se ne gleda dok nije raspakiran barem 1 MB, jer male datoteke imaju velik omjer.
const OMJER_OD: u64 = 1024 * 1024;

pub(crate) fn procitaj(datoteka: File) -> Result<DokumentKml, GreskaKml> {
    let mut arhiva = zip::ZipArchive::new(datoteka).map_err(|_| GreskaKml::NeispravnaArhiva)?;
    let indeks = glavni_kml(arhiva.file_names_raw())?;
    let mut stavka = arhiva
        .by_index(indeks)
        .map_err(|_| GreskaKml::NeispravnaArhiva)?;
    let sazeto = stavka.compressed_size();
    let mut granica = Granica {
        unutarnji: &mut stavka,
        sazeto,
        procitano: 0,
        prekoraceno: false,
    };
    let ishod = procitaj_iz(BufReader::with_capacity(64 * 1024, &mut granica));
    if granica.prekoraceno {
        return Err(GreskaKml::PrevelikaArhiva);
    }
    ishod
}

/// Redni broj `doc.kml` ako postoji, inače prve KML datoteke. Svaka putanja mora ostati u arhivi.
fn glavni_kml<'a>(imena: impl Iterator<Item = &'a [u8]>) -> Result<usize, GreskaKml> {
    let mut nadjeni: Option<(usize, bool)> = None;
    for (i, ime) in imena.enumerate() {
        let ime = String::from_utf8_lossy(ime).to_ascii_lowercase();
        if !sigurna_putanja(&ime) {
            return Err(GreskaKml::NedopustenaPutanja);
        }
        let glavni = ime == "doc.kml" || ime.ends_with("/doc.kml");
        if ime.ends_with(".kml") && nadjeni.is_none_or(|(_, g)| glavni && !g) {
            nadjeni = Some((i, glavni));
        }
    }
    nadjeni.map(|(i, _)| i).ok_or(GreskaKml::NemaKmlUArhivi)
}

fn sigurna_putanja(ime: &str) -> bool {
    !ime.is_empty()
        && !ime.starts_with('/')
        && !ime.contains('\\')
        && !ime.contains(':')
        && ime.split('/').all(|dio| dio != "..")
}

/// Prekida čitanje čim raspakirano prijeđe granicu, bez obzira na veličinu zapisanu u arhivi.
struct Granica<'a, R: Read> {
    unutarnji: &'a mut R,
    sazeto: u64,
    procitano: u64,
    prekoraceno: bool,
}

impl<R: Read> Read for Granica<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.unutarnji.read(buf)?;
        self.procitano += n as u64;
        let preveliko = self.procitano > NAJVECA_VELICINA;
        let omjer =
            self.procitano > OMJER_OD && self.procitano / self.sazeto.max(1) > NAJVECI_OMJER;
        self.prekoraceno = preveliko || omjer;
        if self.prekoraceno {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "prevelika arhiva",
            ));
        }
        Ok(n)
    }
}

#[cfg(test)]
mod testovi {
    use super::*;
    use std::io::Write;

    #[test]
    fn putanje_izvan_arhive_su_nedopustene() {
        assert!(sigurna_putanja("doc.kml"));
        assert!(sigurna_putanja("files/slika.png"));
        assert!(!sigurna_putanja(""));
        assert!(!sigurna_putanja("/etc/doc.kml"));
        assert!(!sigurna_putanja("..\\doc.kml"));
        assert!(!sigurna_putanja("c:doc.kml"));
        assert!(!sigurna_putanja("files/../../doc.kml"));
    }

    #[test]
    fn doc_kml_ima_prednost_pred_prvim() {
        let imena: [&[u8]; 4] = [b"slika.png", b"a.kml", b"files/doc.KML", b"b.kml"];
        assert_eq!(glavni_kml(imena.into_iter()), Ok(2));
        let imena: [&[u8]; 2] = [b"a.kml", b"b.kml"];
        assert_eq!(glavni_kml(imena.into_iter()), Ok(0));
        let imena: [&[u8]; 1] = [b"slika.png"];
        assert_eq!(
            glavni_kml(imena.into_iter()),
            Err(GreskaKml::NemaKmlUArhivi)
        );
        let imena: [&[u8]; 2] = [b"a.kml", b"../b.kml"];
        assert_eq!(
            glavni_kml(imena.into_iter()),
            Err(GreskaKml::NedopustenaPutanja)
        );
    }

    #[test]
    fn granica_prekida_preveliko_raspakiravanje() {
        let mut nule = io::repeat(0).take(3 * OMJER_OD);
        let mut g = Granica {
            unutarnji: &mut nule,
            sazeto: 1000,
            procitano: 0,
            prekoraceno: false,
        };
        let mut izlaz = Vec::new();
        assert!(g.read_to_end(&mut izlaz).is_err());
        assert!(g.prekoraceno);

        let mut malo = io::repeat(1).take(10);
        let mut g = Granica {
            unutarnji: &mut malo,
            sazeto: 1,
            procitano: 0,
            prekoraceno: false,
        };
        let mut izlaz = Vec::new();
        assert_eq!(g.read_to_end(&mut izlaz).ok(), Some(10));
        assert!(!g.prekoraceno);
    }

    fn arhiva(stavke: &[(&str, &[u8])]) -> Vec<u8> {
        let mut z = zip::ZipWriter::new(io::Cursor::new(Vec::new()));
        let opcije = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (ime, sadrzaj) in stavke {
            z.start_file(*ime, opcije).expect("stavka");
            z.write_all(sadrzaj).expect("upis");
        }
        z.finish().expect("kraj").into_inner()
    }

    fn u_datoteku(ime: &str, bajtovi: &[u8]) -> File {
        let staza = std::env::temp_dir().join(format!("terrasimple-kmz-{ime}"));
        std::fs::write(&staza, bajtovi).expect("upis");
        File::open(staza).expect("otvaranje")
    }

    #[test]
    fn arhiva_bez_kml_pokvarena_i_ispravna() {
        let bez = arhiva(&[("slika.png", b"x")]);
        assert_eq!(
            procitaj(u_datoteku("bez.kmz", &bez)),
            Err(GreskaKml::NemaKmlUArhivi)
        );
        assert_eq!(
            procitaj(u_datoteku("smece.kmz", b"PK\x03\x04smece")),
            Err(GreskaKml::NeispravnaArhiva)
        );
        let dobra = arhiva(&[
            ("slika.png", b"x"),
            ("doc.kml", b"<kml><Document><name>D</name></Document></kml>"),
        ]);
        let d = procitaj(u_datoteku("dobra.kmz", &dobra)).expect("čitanje");
        assert_eq!(d.korijen.naziv, "D");

        // Lokalno zaglavlje KML-a je pokvareno, a popis na kraju arhive je ispravan.
        let mut pokvarena = dobra.clone();
        let pocetak = pokvarena
            .windows(4)
            .enumerate()
            .filter(|(_, w)| *w == b"PK\x03\x04")
            .map(|(i, _)| i)
            .nth(1)
            .expect("drugo zaglavlje");
        pokvarena[pocetak + 2] = 0;
        assert_eq!(
            procitaj(u_datoteku("pokvarena.kmz", &pokvarena)),
            Err(GreskaKml::NeispravnaArhiva)
        );
    }
}
