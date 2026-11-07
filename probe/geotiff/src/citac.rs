//! Čitač koji iz TIFF-a uzima samo ono što ekran treba.
//!
//! Memoriju ograničava spremnik od `SPREMNIK_BAJTOVA`. Nesažete trake čita izravno iz
//! datoteke, redak po redak, pa i jedna traka od 2,4 GB ne puni memoriju.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use tiff::decoder::{ChunkType, Decoder, DecodingResult, Limits};
use tiff::tags::Tag;
use tiff::ColorType;

pub const EKRAN: (usize, usize) = (1280, 800);
const SPREMNIK_BAJTOVA: usize = 128 * 1024 * 1024;
const NAJVECI_SAZETI_KOMAD: u64 = 256 * 1024 * 1024;

type Rez<T> = Result<T, String>;
type Citac = Decoder<BufReader<File>>;

fn g<E: std::fmt::Display>(opis: &'static str) -> impl Fn(E) -> String {
    move |e| format!("{opis}: {e}")
}

pub fn ime(put: &str) -> String {
    Path::new(put).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

fn otvori_bez_spremanja(put: &Path) -> Rez<File> {
    let f = File::open(put).map_err(g("otvaranje"))?;
    // Traži od sustava da ne zadržava pročitano, koliko se bez ovlasti može.
    unsafe { libc::fcntl(f.as_raw_fd(), libc::F_NOCACHE, 1) };
    Ok(f)
}

fn citac(put: &Path) -> Rez<Citac> {
    let f = otvori_bez_spremanja(put)?;
    Ok(Decoder::new(BufReader::new(f)).map_err(g("zaglavlje"))?.with_limits(Limits::unlimited()))
}

pub struct Polozaj {
    /// Isti oblik kao u GDAL-a: zapad, korak x, nagib, sjever, nagib, korak y.
    pub preobrazba: [f64; 6],
    pub epsg: Option<u32>,
    pub izvor: &'static str,
}

/// Položaj iz GeoTIFF oznaka. `PixelIsPoint` znači da je točka u sredini piksela.
fn polozaj_iz_oznaka(d: &mut Citac) -> Option<([f64; 6], bool, Option<u32>)> {
    let mut tocka = false;
    let mut epsg = None;
    if let Ok(Some(v)) = d.find_tag_unsigned_vec::<u32>(Tag::GeoKeyDirectoryTag) {
        for k in v.get(4..).unwrap_or(&[]).chunks_exact(4) {
            match k[0] {
                1025 => tocka = k[3] == 2,
                2048 | 3072 if k[3] != 32767 && k[1] == 0 => epsg = Some(k[3]),
                _ => {}
            }
        }
    }
    let realno = |d: &mut Citac, oznaka: Tag| -> Option<Vec<f64>> {
        d.find_tag(oznaka).ok().flatten().and_then(|v| v.into_f64_vec().ok())
    };
    if let Some(m) = realno(d, Tag::Unknown(34264)).filter(|m| m.len() >= 8) {
        return Some(([m[3], m[0], m[1], m[7], m[4], m[5]], tocka, epsg));
    }
    let korak = realno(d, Tag::ModelPixelScaleTag)?;
    let veza = realno(d, Tag::ModelTiepointTag)?;
    if korak.len() < 2 || veza.len() < 6 {
        return None;
    }
    let (sx, sy) = (korak[0], korak[1]);
    Some(([veza[3] - veza[0] * sx, sx, 0.0, veza[4] + veza[1] * sy, 0.0, -sy], tocka, epsg))
}

/// Položaj iz `.tfw` uz sliku. U njoj su sredina gornjeg lijevog piksela i koraci.
fn polozaj_iz_tfw(put: &Path) -> Option<[f64; 6]> {
    let tekst = ["tfw", "TFW", "tifw", "tiffw"].iter().find_map(|e| std::fs::read_to_string(put.with_extension(e)).ok())?;
    let b: Vec<f64> = tekst.split_whitespace().filter_map(|v| v.parse().ok()).collect();
    if b.len() != 6 {
        return None;
    }
    Some([b[4] - b[0] / 2.0 - b[2] / 2.0, b[0], b[2], b[5] - b[1] / 2.0 - b[3] / 2.0, b[1], b[3]])
}

struct Razina {
    /// Staza i redni broj slike u datoteci.
    put: PathBuf,
    redni: usize,
    s: u32,
    v: u32,
    pod_s: u32,
    pod_v: u32,
    ploce: bool,
    kompresija: u32,
    uzoraka: usize,
    paleta: Option<Rc<Vec<[u8; 3]>>>,
    jpeg_ycbcr: bool,
    /// Ofseti traka, samo za nesažete trake koje se čitaju izravno.
    ofseti: Option<Vec<u64>>,
    /// Svaka boja u svojem komadu, kako GDAL zna zapisati vanjske piramide.
    planarno: bool,
    /// Ofseti i duljine za paletu koju raspakiramo sami.
    sazeta_paleta: Option<(Vec<u64>, Vec<u64>)>,
    citac: Option<Citac>,
    izravno: Option<File>,
}

struct Komad {
    podaci: Vec<u8>,
    sirina: usize,
    uzoraka: usize,
}

pub struct Slika {
    razine: Vec<Razina>,
    polozaj: Option<Polozaj>,
    spremnik: HashMap<(usize, u32), (Rc<Komad>, u64)>,
    bajtova_u_spremniku: usize,
    otkucaj: u64,
    procitano: u64,
}

fn procitaj_razine(put: &Path, razine: &mut Vec<Razina>, prvi: Option<&mut Citac>) -> Rez<()> {
    let mut vlastiti;
    let d: &mut Citac = match prvi {
        Some(d) => d,
        None => {
            vlastiti = citac(put)?;
            &mut vlastiti
        }
    };
    let mut redni = 0;
    loop {
        d.seek_to_image(redni).map_err(g("slika u datoteci"))?;
        let vrsta = d.find_tag_unsigned::<u32>(Tag::NewSubfileType).ok().flatten().unwrap_or(0);
        let maska = vrsta & 4 != 0;
        if !maska {
            razine.push(opis_razine(d, put, redni)?);
        }
        if !d.more_images() {
            return Ok(());
        }
        redni += 1;
    }
}

fn opis_razine(d: &mut Citac, put: &Path, redni: usize) -> Rez<Razina> {
    let (s, v) = d.dimensions().map_err(g("veličina"))?;
    let (pod_s, pod_v) = d.chunk_dimensions();
    // Biblioteka za paletu ne daje vrstu boje i ne čita komade, pa to radimo sami.
    let fotometrija = d.find_tag_unsigned::<u32>(Tag::PhotometricInterpretation).ok().flatten().unwrap_or(1);
    let (uzoraka, je_paleta, jpeg_ycbcr) = if fotometrija == 3 {
        (1, true, false)
    } else {
        match d.colortype().map_err(g("vrsta boje"))? {
            ColorType::RGB(8) => (3, false, false),
            ColorType::RGBA(8) => (4, false, false),
            ColorType::Gray(8) => (1, false, false),
            ColorType::YCbCr(8) => (3, false, true),
            c => return Err(format!("nepodržana vrsta boje {c:?}")),
        }
    };
    let ravnina = d.find_tag_unsigned::<u32>(Tag::PlanarConfiguration).ok().flatten().unwrap_or(1);
    let planarno = ravnina != 1 && uzoraka > 1;
    let kompresija = d.find_tag_unsigned::<u32>(Tag::Compression).ok().flatten().unwrap_or(1);
    let ploce = matches!(d.get_chunk_type(), ChunkType::Tile);
    let sazeta_paleta = if je_paleta && (kompresija != 1 || ploce) {
        if !matches!(kompresija, 1 | 5 | 32773) {
            return Err(format!("paleta s kompresijom {kompresija} nije podržana"));
        }
        let (o, b) = if ploce { (Tag::TileOffsets, Tag::TileByteCounts) } else { (Tag::StripOffsets, Tag::StripByteCounts) };
        let o = d.find_tag_unsigned_vec::<u64>(o).ok().flatten().ok_or("nema ofseta komada")?;
        let b = d.find_tag_unsigned_vec::<u64>(b).ok().flatten().ok_or("nema duljina komada")?;
        Some((o, b))
    } else {
        None
    };
    let paleta = if je_paleta {
        let t = d.get_tag_u16_vec(Tag::ColorMap).map_err(g("paleta"))?;
        let n = t.len() / 3;
        let k = |x: u16| (x >> 8) as u8;
        Some(Rc::new((0..n).map(|i| [k(t[i]), k(t[n + i]), k(t[2 * n + i])]).collect::<Vec<_>>()))
    } else {
        None
    };
    let ofseti = if kompresija == 1 && !ploce && !planarno {
        Some(d.find_tag_unsigned_vec::<u64>(Tag::StripOffsets).ok().flatten().ok_or("nema ofseta traka")?)
    } else {
        None
    };
    Ok(Razina { put: put.to_path_buf(), redni, s, v, pod_s, pod_v, ploce, kompresija, uzoraka, paleta, jpeg_ycbcr, planarno, sazeta_paleta, ofseti, citac: None, izravno: None })
}

/// PackBits: bajt n, pa n + 1 doslovnih bajtova, ili jedan bajt ponovljen 1 - n puta.
fn raspakiraj_packbits(ulaz: &[u8], ocekivano: usize) -> Vec<u8> {
    let mut izlaz = Vec::with_capacity(ocekivano);
    let mut i = 0;
    while i < ulaz.len() && izlaz.len() < ocekivano {
        let n = ulaz[i] as i8;
        i += 1;
        if n >= 0 {
            let kraj = (i + n as usize + 1).min(ulaz.len());
            izlaz.extend_from_slice(&ulaz[i..kraj]);
            i = kraj;
        } else if n != -128 {
            if let Some(b) = ulaz.get(i) {
                izlaz.resize(izlaz.len() + 1 + n.unsigned_abs() as usize, *b);
            }
            i += 1;
        }
    }
    izlaz.resize(ocekivano, 0);
    izlaz
}

fn raspakiraj_lzw(ulaz: &[u8], ocekivano: usize) -> Rez<Vec<u8>> {
    let mut izlaz = Vec::with_capacity(ocekivano);
    let rezultat = weezl::decode::Decoder::with_tiff_size_switch(weezl::BitOrder::Msb, 8)
        .into_stream(&mut izlaz)
        .decode_all(ulaz);
    rezultat.status.map_err(g("LZW"))?;
    izlaz.resize(ocekivano, 0);
    Ok(izlaz)
}

/// JPEG u TIFF-u biblioteka vraća kao YCbCr, a ekran treba RGB.
fn ycbcr_u_rgb(p: &[u8]) -> [u8; 3] {
    let (y, cb, cr) = (f32::from(p[0]), f32::from(p[1]) - 128.0, f32::from(p[2]) - 128.0);
    let k = |v: f32| v.round().clamp(0.0, 255.0) as u8;
    [k(y + 1.402 * cr), k(y - 0.344_136 * cb - 0.714_136 * cr), k(y + 1.772 * cb)]
}

/// Jedan komad palete: sažeti bajtovi iz datoteke, pa raspakiranje. Tu ne pomaže biblioteka.
fn procitaj_paletu(r: &mut Razina, indeks: u32) -> Rez<(Vec<u8>, usize)> {
    let (ofseti, duljine) = r.sazeta_paleta.as_ref().ok_or("nema ofseta palete")?;
    let i = indeks as usize;
    let (ofset, duljina) = (*ofseti.get(i).ok_or("nema komada")?, *duljine.get(i).ok_or("nema duljine")? as usize);
    let (sirina, visina) = if r.ploce {
        (r.pod_s as usize, r.pod_v as usize)
    } else {
        (r.s as usize, (r.v - (indeks * r.pod_v).min(r.v)).min(r.pod_v) as usize)
    };
    let ocekivano = sirina * visina;
    if r.izravno.is_none() {
        r.izravno = Some(otvori_bez_spremanja(&r.put)?);
    }
    let f = r.izravno.as_mut().expect("datoteka");
    f.seek(SeekFrom::Start(ofset)).map_err(g("pomak"))?;
    let mut ulaz = vec![0; duljina];
    f.read_exact(&mut ulaz).map_err(g("čitanje komada"))?;
    let izlaz = match r.kompresija {
        5 => raspakiraj_lzw(&ulaz, ocekivano)?,
        32773 => raspakiraj_packbits(&ulaz, ocekivano),
        _ => {
            ulaz.resize(ocekivano, 0);
            ulaz
        }
    };
    Ok((izlaz, sirina))
}

impl Razina {
    fn pikselni_redak(&self) -> u64 {
        u64::from(self.s) * self.uzoraka as u64
    }

    fn podijeli_komad(&self, x: u32, y: u32) -> u32 {
        if self.ploce {
            (y / self.pod_v) * self.s.div_ceil(self.pod_s) + x / self.pod_s
        } else {
            y / self.pod_v
        }
    }

    fn pocetak_komada(&self, x: u32, y: u32) -> (u32, u32) {
        if self.ploce {
            (x / self.pod_s * self.pod_s, y / self.pod_v * self.pod_v)
        } else {
            (0, y / self.pod_v * self.pod_v)
        }
    }
}

impl Slika {
    pub fn otvori(put: &Path) -> Rez<Self> {
        let mut razine = Vec::new();
        let mut d = citac(put)?;
        let oznake = polozaj_iz_oznaka(&mut d);
        procitaj_razine(put, &mut razine, Some(&mut d))?;
        let mut vanjske = put.as_os_str().to_owned();
        vanjske.push(".ovr");
        let vanjske = PathBuf::from(vanjske);
        if vanjske.exists() {
            procitaj_razine(&vanjske, &mut razine, None)?;
        }
        razine.sort_by_key(|r| std::cmp::Reverse(r.s));
        let prva = razine.first().ok_or("u datoteci nema slike")?;
        let (ps, pv) = (prva.s, prva.v);
        // Polovica piksela za PixelIsPoint, kako radi i GDAL.
        let polozaj = match oznake {
            Some((mut t, tocka, epsg)) => {
                if tocka {
                    t[0] -= t[1] / 2.0;
                    t[3] -= t[5] / 2.0;
                }
                Some(Polozaj { preobrazba: t, epsg, izvor: "oznake u slici" })
            }
            None => polozaj_iz_tfw(put).map(|t| Polozaj { preobrazba: t, epsg: None, izvor: "datoteka .tfw" }),
        };
        let _ = (ps, pv);
        Ok(Self { razine, polozaj, spremnik: HashMap::new(), bajtova_u_spremniku: 0, otkucaj: 0, procitano: 0 })
    }

    pub fn velicina(&self) -> (u32, u32) {
        (self.razine[0].s, self.razine[0].v)
    }

    pub fn velicina_razine(&self, i: usize) -> Option<(u32, u32)> {
        self.razine.get(i).map(|r| (r.s, r.v))
    }

    pub fn broj_razina(&self) -> usize {
        self.razine.len()
    }

    pub fn polozaj(&self) -> Option<&Polozaj> {
        self.polozaj.as_ref()
    }

    pub fn jpeg(&self) -> bool {
        self.razine.iter().any(|r| r.kompresija == 7)
    }

    pub fn opis(&self) -> String {
        let r = &self.razine[0];
        let polozaj = self.polozaj.as_ref().map_or("bez položaja".to_string(), |p| {
            format!("položaj: {}, sustav {}", p.izvor, p.epsg.map_or("NEPOZNAT".to_string(), |e| format!("EPSG:{e}")))
        });
        format!(
            "{} {}x{}, kompresija {}, uzoraka {}{}{}; {polozaj}; pročitano {:.0} MB",
            if r.ploce { "pločice" } else { "trake" },
            r.pod_s,
            r.pod_v,
            r.kompresija,
            r.uzoraka,
            if r.paleta.is_some() { ", paleta" } else { "" },
            if r.jpeg_ycbcr { ", YCbCr" } else { "" },
            self.procitano as f64 / 1e6
        )
    }

    fn komad(&mut self, razina: usize, indeks: u32) -> Rez<Rc<Komad>> {
        self.otkucaj += 1;
        let otkucaj = self.otkucaj;
        if let Some((k, zadnji)) = self.spremnik.get_mut(&(razina, indeks)) {
            *zadnji = otkucaj;
            return Ok(Rc::clone(k));
        }
        let r = &mut self.razine[razina];
        let (s, v) = (u64::from(r.pod_s), u64::from(r.pod_v));
        if s * v * r.uzoraka as u64 > NAJVECI_SAZETI_KOMAD {
            return Err(format!("sažeti komad veći od {} MB nije podržan", NAJVECI_SAZETI_KOMAD / 1024 / 1024));
        }
        let (podaci, sirina, uzoraka) = if r.sazeta_paleta.is_some() {
            let sirina = if r.ploce { r.pod_s } else { r.s };
            let (podaci, _) = procitaj_paletu(r, indeks)?;
            (podaci, sirina, 1)
        } else {
            if r.citac.is_none() {
                let mut d = citac(&r.put)?;
                d.seek_to_image(r.redni).map_err(g("razina"))?;
                r.citac = Some(d);
            }
            let d = r.citac.as_mut().expect("čitač");
            let (sirina, visina) = d.chunk_data_dimensions(indeks);
            let podaci = match d.read_chunk(indeks).map_err(g("čitanje komada"))? {
                DecodingResult::U8(p) => p,
                _ => return Err("očekivano je 8 bita po uzorku".into()),
            };
            let uzoraka = podaci.len() / (sirina as usize * visina as usize).max(1);
            (podaci, sirina, uzoraka)
        };
        self.procitano += podaci.len() as u64;
        let velicina = podaci.len();
        let komad = Rc::new(Komad { podaci, sirina: sirina as usize, uzoraka });
        self.spremnik.insert((razina, indeks), (Rc::clone(&komad), otkucaj));
        self.bajtova_u_spremniku += velicina;
        while self.bajtova_u_spremniku > SPREMNIK_BAJTOVA && self.spremnik.len() > 1 {
            let najstariji = *self.spremnik.iter().min_by_key(|(_, (_, t))| *t).map(|(k, _)| k).expect("spremnik");
            if let Some((k, _)) = self.spremnik.remove(&najstariji) {
                self.bajtova_u_spremniku -= k.podaci.len();
            }
        }
        Ok(komad)
    }

    /// Jedan redak nesažete trake, izravno iz datoteke.
    fn redak_izravno(&mut self, razina: usize, y: u32, izlaz: &mut Vec<u8>) -> Rez<()> {
        let r = &mut self.razine[razina];
        let duljina = r.pikselni_redak();
        let traka = (y / r.pod_v) as usize;
        let u_traci = u64::from(y % r.pod_v);
        let ofset = r.ofseti.as_ref().and_then(|o| o.get(traka).copied()).ok_or("nema ofseta za redak")?;
        if r.izravno.is_none() {
            r.izravno = Some(otvori_bez_spremanja(&r.put)?);
        }
        let f = r.izravno.as_mut().expect("datoteka");
        f.seek(SeekFrom::Start(ofset + u_traci * duljina)).map_err(g("pomak"))?;
        izlaz.resize(duljina as usize, 0);
        f.read_exact(izlaz).map_err(g("čitanje retka"))?;
        self.procitano += duljina;
        Ok(())
    }

    /// Piše točke (x, y) iz razine kao RGB, s bijelom podlogom ispod prozirnog ako treba.
    fn uzorkuj(&mut self, razina: usize, xs: &[u32], ys: &[u32], podloga: bool) -> Rez<Vec<u8>> {
        let mut izlaz = Vec::with_capacity(xs.len() * ys.len() * 3);
        let paleta = self.razine[razina].paleta.clone();
        let izravno = self.razine[razina].ofseti.is_some();
        let uzoraka = self.razine[razina].uzoraka;
        let ycbcr = self.razine[razina].jpeg_ycbcr;
        let planarno = self.razine[razina].planarno;
        let u_ravnini = {
            let r = &self.razine[razina];
            if r.ploce { r.s.div_ceil(r.pod_s) * r.v.div_ceil(r.pod_v) } else { r.v.div_ceil(r.pod_v) }
        };
        let mut redak = Vec::new();
        for &y in ys {
            if izravno {
                self.redak_izravno(razina, y, &mut redak)?;
            }
            for &x in xs {
                if izravno {
                    let p = &redak[x as usize * uzoraka..];
                    upisi_piksel(&mut izlaz, &p[..uzoraka], uzoraka, paleta.as_deref(), podloga, ycbcr);
                    continue;
                }
                let r = &self.razine[razina];
                let i = r.podijeli_komad(x, y);
                let (x0, y0) = r.pocetak_komada(x, y);
                let k = self.komad(razina, i)?;
                let polozaj = ((y - y0) as usize * k.sirina + (x - x0) as usize) * k.uzoraka;
                if planarno {
                    let mut p = [k.podaci[polozaj], 0, 0, 255];
                    for ravnina in 1..uzoraka {
                        p[ravnina] = self.komad(razina, i + ravnina as u32 * u_ravnini)?.podaci[polozaj];
                    }
                    upisi_piksel(&mut izlaz, &p[..uzoraka], uzoraka, None, podloga, ycbcr);
                } else {
                    let n = k.uzoraka;
                    upisi_piksel(&mut izlaz, &k.podaci[polozaj..polozaj + n], n, paleta.as_deref(), podloga, ycbcr);
                }
            }
        }
        Ok(izlaz)
    }

    /// Isječak bez ikakvog preuzorkovanja, za usporedbu s GDAL-om.
    pub fn isjecak(&mut self, razina: usize, x: u32, y: u32, s: u32, v: u32) -> Rez<Vec<u8>> {
        let xs: Vec<u32> = (x..x + s).collect();
        let ys: Vec<u32> = (y..y + v).collect();
        self.uzorkuj(razina, &xs, &ys, false)
    }

    /// Što ekran pokazuje za dio slike: uzima najbližu razinu koja je dovoljno oštra.
    pub fn pogled(&mut self, x: f64, y: f64, s: f64, v: f64) -> Rez<Vec<u8>> {
        let (ps, pv) = self.velicina();
        let mjerilo = s / EKRAN.0 as f64;
        let mut razina = 0;
        for (i, r) in self.razine.iter().enumerate() {
            if f64::from(ps) / f64::from(r.s) <= mjerilo.max(1.0) {
                razina = i;
            }
        }
        let r = &self.razine[razina];
        let (fs, fv) = (f64::from(r.s) / f64::from(ps), f64::from(r.v) / f64::from(pv));
        let (rs, rv) = (r.s, r.v);
        let xs: Vec<u32> = (0..EKRAN.0).map(|i| (((x + s * (i as f64 + 0.5) / EKRAN.0 as f64) * fs) as u32).min(rs - 1)).collect();
        let ys: Vec<u32> = (0..EKRAN.1).map(|i| (((y + v * (i as f64 + 0.5) / EKRAN.1 as f64) * fv) as u32).min(rv - 1)).collect();
        self.uzorkuj(razina, &xs, &ys, true)
    }

    /// Pročita svaki komad glavne slike, kao što bi morala izrada piramida.
    pub fn procitaj_sve(&mut self) -> Rez<u64> {
        let v = self.razine[0].v;
        let mut ukupno = 0;
        if self.razine[0].ofseti.is_some() {
            let mut redak = Vec::new();
            for y in 0..v {
                self.redak_izravno(0, y, &mut redak)?;
                ukupno += redak.len() as u64;
            }
            return Ok(ukupno);
        }
        let r = &self.razine[0];
        let komada = if r.ploce { r.s.div_ceil(r.pod_s) * r.v.div_ceil(r.pod_v) } else { r.v.div_ceil(r.pod_v) };
        for i in 0..komada {
            ukupno += self.komad(0, i)?.podaci.len() as u64;
        }
        Ok(ukupno)
    }
}

fn upisi_piksel(izlaz: &mut Vec<u8>, p: &[u8], n: usize, paleta: Option<&Vec<[u8; 3]>>, podloga: bool, ycbcr: bool) {
    if ycbcr {
        izlaz.extend_from_slice(&ycbcr_u_rgb(p));
        return;
    }
    match (paleta, n) {
        (Some(t), _) => izlaz.extend_from_slice(t.get(p[0] as usize).unwrap_or(&[0, 0, 0])),
        (None, 1) => izlaz.extend_from_slice(&[p[0], p[0], p[0]]),
        (None, 4) if podloga => {
            let a = u32::from(p[3]);
            for c in &p[..3] {
                izlaz.push(((u32::from(*c) * a + 255 * (255 - a)) / 255) as u8);
            }
        }
        (None, _) => izlaz.extend_from_slice(&p[..3]),
    }
}

#[cfg(test)]
mod testovi {
    use super::*;

    #[test]
    fn prozirno_postaje_bijelo() {
        let mut i = Vec::new();
        upisi_piksel(&mut i, &[10, 20, 30, 0], 4, None, true, false);
        assert_eq!(i, [255, 255, 255]);
    }

    #[test]
    fn neprozirno_ostaje_isto() {
        let mut i = Vec::new();
        upisi_piksel(&mut i, &[10, 20, 30, 255], 4, None, true, false);
        assert_eq!(i, [10, 20, 30]);
    }

    #[test]
    fn indeks_izvan_palete_ne_ruši() {
        let mut i = Vec::new();
        let paleta = vec![[1, 2, 3]];
        upisi_piksel(&mut i, &[200], 1, Some(&paleta), false, false);
        assert_eq!(i, [0, 0, 0]);
    }

    #[test]
    fn ycbcr_siva_ostaje_siva_a_crvena_crvena() {
        assert_eq!(ycbcr_u_rgb(&[100, 128, 128]), [100, 100, 100]);
        assert_eq!(ycbcr_u_rgb(&[76, 85, 255]), [254, 0, 0]);
    }

    #[test]
    fn packbits_primjer_iz_specifikacije_tiffa() {
        let ulaz = [0xFE, 0xAA, 0x02, 0x80, 0x00, 0x2A, 0xFD, 0xAA, 0x03, 0x80, 0x00, 0x2A, 0x22, 0xF7, 0xAA];
        let ocekivano = [0xAA, 0xAA, 0xAA, 0x80, 0x00, 0x2A, 0xAA, 0xAA, 0xAA, 0xAA, 0x80, 0x00, 0x2A, 0x22, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA];
        assert_eq!(raspakiraj_packbits(&ulaz, ocekivano.len()), ocekivano);
    }

    #[test]
    fn packbits_ne_rusi_na_odrezanom_ulazu() {
        assert_eq!(raspakiraj_packbits(&[0x05, 1, 2], 4), [1, 2, 0, 0]);
        assert_eq!(raspakiraj_packbits(&[0xFE], 2), [0, 0]);
    }

    #[test]
    fn tfw_je_sredina_piksela() {
        let mapa = std::env::temp_dir().join("proba_geotiff_tfw");
        std::fs::create_dir_all(&mapa).unwrap();
        let put = mapa.join("slika.tif");
        std::fs::write(mapa.join("slika.tfw"), "8\n0\n0\n-8\n6500008\n5095173\n").unwrap();
        let t = polozaj_iz_tfw(&put).unwrap();
        assert_eq!((t[0], t[1], t[3], t[5]), (6_500_004.0, 8.0, 5_095_177.0, -8.0));
        std::fs::remove_dir_all(&mapa).unwrap();
    }
}
