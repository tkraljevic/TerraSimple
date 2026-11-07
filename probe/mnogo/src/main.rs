//! Proba: vuče li grafička kartica stvarnih 3 milijuna vrhova, bez prozora.
//!
//! `proba_mnogo izmisljeno|<kml> sve|vidljivo|naivno [mapa za slike]`
//! `proba_mnogo provjera`
//!
//! Crta u sliku 1920 x 1080 i čeka da kartica završi svaki okvir. Za stvarnu datoteku ispisuje
//! samo brojeve i nikad ne sprema sliku.

use proba_mnogo::crtac::{Crtac, Kamera, Okvir};
use proba_mnogo::{podaci, priprema};
use std::path::Path;
use std::time::Instant;

const SIRINA: u32 = 1920;
const VISINA: u32 = 1080;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const UZORAKA: u32 = 4;
const POLA: [f32; 2] = [SIRINA as f32 / 2.0, VISINA as f32 / 2.0];

fn vrh_memorije_mb() -> f64 {
    let mut r: libc::rusage = unsafe { std::mem::zeroed() };
    unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut r) };
    r.ru_maxrss as f64 / 1e6
}

struct Platno {
    uredaj: wgpu::Device,
    red: wgpu::Queue,
    granice: wgpu::Limits,
    uzorak: wgpu::TextureView,
    cilj: wgpu::Texture,
    cilj_pogled: wgpu::TextureView,
}

fn napravi_platno() -> Platno {
    let instanca = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instanca.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        ..Default::default()
    }))
    .expect("nema grafičke kartice");
    let i = adapter.get_info();
    println!("kartica: {} ({:?}), {:?}", i.name, i.device_type, i.backend);
    let granice = adapter.limits();
    let (uredaj, red) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_limits: wgpu::Limits { max_buffer_size: granice.max_buffer_size, ..wgpu::Limits::default() },
        ..Default::default()
    }))
    .expect("uređaj");
    let velicina = wgpu::Extent3d { width: SIRINA, height: VISINA, depth_or_array_layers: 1 };
    let tekstura = |n, u| {
        uredaj.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: velicina,
            mip_level_count: 1,
            sample_count: n,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: u,
            view_formats: &[],
        })
    };
    let uzorak = tekstura(UZORAKA, wgpu::TextureUsages::RENDER_ATTACHMENT).create_view(&Default::default());
    let cilj = tekstura(1, wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
    let cilj_pogled = cilj.create_view(&Default::default());
    Platno { uredaj, red, granice, uzorak, cilj, cilj_pogled }
}

fn nacrtaj_razinu(p: &Platno, c: &Crtac, k: Kamera, razina: usize) -> Okvir {
    let plan = c.pripremi(&p.red, k, razina);
    let mut enk = p.uredaj.create_command_encoder(&Default::default());
    let okvir;
    {
        let mut prolaz = enk.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &p.uzorak,
                depth_slice: None,
                resolve_target: Some(&p.cilj_pogled),
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.95, g: 0.95, b: 0.93, a: 1.0 }), store: wgpu::StoreOp::Discard },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        okvir = c.crtaj(&mut prolaz, &plan);
    }
    let predano = p.red.submit([enk.finish()]);
    // Čeka karticu, pa vrijeme uključuje i njezin stvarni rad, a ne samo naš kod.
    p.uredaj.poll(wgpu::PollType::Wait { submission_index: Some(predano), timeout: None }).expect("čekanje kartice");
    okvir
}

fn nacrtaj(p: &Platno, c: &Crtac, k: Kamera) -> Okvir {
    nacrtaj_razinu(p, c, k, c.razina_za(k.metara_po_px))
}

fn procitaj(p: &Platno) -> image::RgbaImage {
    let red_b = SIRINA * 4;
    let razmak = red_b.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let izlaz = p.uredaj.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(razmak * VISINA),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut enk = p.uredaj.create_command_encoder(&Default::default());
    enk.copy_texture_to_buffer(
        p.cilj.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &izlaz,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(razmak), rows_per_image: Some(VISINA) },
        },
        wgpu::Extent3d { width: SIRINA, height: VISINA, depth_or_array_layers: 1 },
    );
    p.red.submit([enk.finish()]);
    izlaz.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    p.uredaj.poll(wgpu::PollType::wait_indefinitely()).expect("čekanje");
    let podaci = izlaz.slice(..).get_mapped_range().expect("čitanje slike");
    let mut slika = image::RgbaImage::new(SIRINA, VISINA);
    for y in 0..VISINA {
        let o = (y * razmak) as usize;
        for x in 0..SIRINA {
            let i = o + x as usize * 4;
            slika.put_pixel(x, y, image::Rgba([podaci[i], podaci[i + 1], podaci[i + 2], 255]));
        }
    }
    slika
}

/// Okvir svih piksela koji su blizu zadane boje: lijevo, gore, desno, dolje.
fn okvir_boje(s: &image::RgbaImage, b: [u8; 3]) -> Option<[u32; 4]> {
    let mut o: Option<[u32; 4]> = None;
    for (x, y, p) in s.enumerate_pixels() {
        if (0..3).all(|i| p[i].abs_diff(b[i]) <= 12) {
            o = Some(match o {
                None => [x, y, x, y],
                Some(v) => [v[0].min(x), v[1].min(y), v[2].max(x), v[3].max(y)],
            });
        }
    }
    o
}

/// Crta poznate oblike na poznatom mjestu i mjeri ih u pikselima. Slika se ne gleda, nego mjeri.
fn provjera() -> i32 {
    let s = [2_000_000.0, 5_700_000.0];
    let z = |ispuna, crta, sirina, d, lik, tocka| podaci::Znacajka { ispuna, crta, sirina, dijelovi: d, lik, tocka, vidljivo: true };
    let sve = vec![
        // Lik 400 x 200 m, zapadno i sjeverno od sredine. Mjerilo je 1 m/px.
        z(Some([255, 0, 0, 255]), [255, 0, 0, 255], 0.5, vec![vec![[s[0] - 500.0, s[1] + 100.0], [s[0] - 100.0, s[1] + 100.0], [s[0] - 100.0, s[1] + 300.0], [s[0] - 500.0, s[1] + 300.0], [s[0] - 500.0, s[1] + 100.0]]], true, false),
        // Crta 300 m istočno-zapadno, južno od sredine, debljine 6 px.
        z(None, [0, 0, 255, 255], 6.0, vec![vec![[s[0] + 100.0, s[1] - 200.0], [s[0] + 400.0, s[1] - 200.0]]], false, false),
        // Točka 10 px, sjeveroistočno.
        z(None, [0, 200, 0, 255], 10.0, vec![vec![[s[0] + 300.0, s[1] + 200.0]]], false, true),
    ];
    let pr = priprema::pripremi(&sve, false);
    let p = napravi_platno();
    let c = Crtac::novi(&p.uredaj, &p.granice, pr.razine, FORMAT, UZORAKA);
    nacrtaj(&p, &c, Kamera { sredina: s, metara_po_px: 1.0, pola: POLA });
    let sl = procitaj(&p);
    let (sx, sy) = (960i64, 540i64);
    let ocekivano = [
        ("lik", [255u8, 0, 0], [sx - 500, sy - 300, sx - 100, sy - 100]),
        ("crta", [0, 0, 255], [sx + 100, sy + 200 - 3, sx + 400, sy + 200 + 3]),
        ("točka", [0, 200, 0], [sx + 300 - 5, sy - 200 - 5, sx + 300 + 5, sy - 200 + 5]),
    ];
    let mut palo = 0;
    for (ime, b, o) in ocekivano {
        match okvir_boje(&sl, b) {
            None => {
                println!("  PALO    {ime}: nema ga na slici");
                palo += 1;
            }
            Some(v) => {
                let d = (0..4).map(|i| (i64::from(v[i]) - o[i]).abs()).max().unwrap_or(99);
                let dobro = d <= 2;
                println!("  {}  {ime}: izmjereno {v:?}, očekivano {o:?}, razlika {d} px", if dobro { "u redu" } else { "PALO  " });
                palo += i32::from(!dobro);
            }
        }
    }
    palo
}

struct Slucajni(u64);

impl Slucajni {
    fn f(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % 1_000_000) as f64 / 1_000_000.0
    }
}

fn mjeri_mjerilo(p: &Platno, c: &Crtac, srediste: &[[f64; 2]], metara_po_px: f64, cijela: Kamera, slucajni: &mut Slucajni) -> String {
    let mut vremena = Vec::new();
    let mut najvise = Okvir::default();
    let mut bilo = srediste[(slucajni.f() * (srediste.len() - 1) as f64) as usize];
    for i in 0..60 {
        let k = if metara_po_px >= cijela.metara_po_px {
            cijela
        } else {
            // Svakih 15 okvira skok na novo mjesto, a inače povlačenje za 20 piksela u okviru.
            if i % 15 == 0 {
                bilo = srediste[(slucajni.f() * (srediste.len() - 1) as f64) as usize];
            }
            bilo[0] += 20.0 * metara_po_px;
            Kamera { sredina: bilo, metara_po_px, pola: POLA }
        };
        let t = Instant::now();
        let o = nacrtaj(p, c, k);
        vremena.push(t.elapsed().as_secs_f64() * 1000.0);
        if o.odsjecaka + o.trokuta >= najvise.odsjecaka + najvise.trokuta {
            najvise = o;
        }
    }
    vremena.sort_by(f64::total_cmp);
    format!(
        "{:>9.1} m/px | razina {} | najviše na ekranu: komada {:>5}, trokuta {:>8}, odsječaka {:>8} | medijan {:>6.2} ms | najgore {:>6.2} ms",
        metara_po_px.min(cijela.metara_po_px),
        najvise.razina,
        najvise.komada,
        najvise.trokuta,
        najvise.odsjecaka,
        vremena[vremena.len() / 2],
        vremena[vremena.len() - 1],
    )
}

fn naivno(sve: &[podaci::Znacajka]) {
    // Ovako radi `Plugin` u probi karte: svaki okvir sve ide kroz egui.
    use epaint::{tessellator::Tessellator, Color32, Mesh, Pos2, Shape, Stroke, TessellationOptions};
    let sredina = sve.iter().find_map(|z| z.dijelovi.first().and_then(|d| d.first())).copied().unwrap_or([0.0, 0.0]);
    let t = Instant::now();
    let mut ten = Tessellator::new(1.0, TessellationOptions::default(), [1, 1], vec![]);
    let mut mreza = Mesh::default();
    let mut vrhova = 0usize;
    for z in sve {
        for d in &z.dijelovi {
            let tocke: Vec<Pos2> = d.iter().map(|p| Pos2::new(((p[0] - sredina[0]) / 500.0) as f32, ((p[1] - sredina[1]) / 500.0) as f32)).collect();
            vrhova += tocke.len();
            ten.tessellate_shape(Shape::line(tocke, Stroke::new(1.0, Color32::RED)), &mut mreza);
        }
    }
    println!(
        "naivno kroz egui, samo crte, jedan okvir bez slanja na karticu: {:.0} ms, vrhova {vrhova}, trokuta {}, memorija {:.0} MB",
        t.elapsed().as_secs_f64() * 1000.0,
        mreza.indices.len() / 3,
        (mreza.vertices.len() * std::mem::size_of_val(&mreza.vertices[0]) + mreza.indices.len() * 4) as f64 / 1e6
    );
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.get(1).is_some_and(|x| x == "provjera") {
        let palo = provjera();
        println!("provjera položaja: {}", if palo == 0 { "PROŠLO" } else { "PALO" });
        std::process::exit(palo);
    }
    if a.len() < 3 {
        eprintln!("upotreba: proba_mnogo provjera | izmisljeno|<kml> sve|vidljivo|naivno [mapa za slike]");
        std::process::exit(2);
    }
    let stvarno = a[1] != "izmisljeno";
    let samo_vidljivo = a[2] == "vidljivo";
    let t = Instant::now();
    let sve = if stvarno {
        match proba_kml::citac::procitaj_datoteku(Path::new(&a[1])) {
            Ok(st) => podaci::iz_kml(&st.korijen),
            // Poruka čitača ne citira sadržaj, ali ionako ispisujemo samo da nije uspjelo.
            Err(_) => {
                eprintln!("datoteka se ne može pročitati");
                std::process::exit(3);
            }
        }
    } else {
        podaci::izmisljeno(85_683, 1_482, 1_738, 34, 80)
    };
    let citanje = t.elapsed().as_secs_f64();
    let vrhova: usize = sve.iter().map(|z| z.dijelovi.iter().map(Vec::len).sum::<usize>()).sum();
    let vidljivih = sve.iter().filter(|z| z.vidljivo).count();
    println!("izvor: {}, način: {}", if stvarno { "stvarna datoteka" } else { "izmišljeno" }, a[2]);
    println!("značajki {}, vidljivih {vidljivih}, vrhova {vrhova}, čitanje {citanje:.2} s", sve.len());
    if a[2] == "naivno" {
        // Zasebno, da njegova memorija ne uđe u mjerenje crtanja na kartici.
        naivno(&sve);
        println!("najviše memorije procesa: {:.0} MB", vrh_memorije_mb());
        return;
    }

    let t = Instant::now();
    let pr = priprema::pripremi(&sve, samo_vidljivo);
    println!("priprema (jedna jezgra): {:.2} s, likova koje lyon nije mogao ispuniti: {}", t.elapsed().as_secs_f64(), pr.pogresaka_ispune);
    for r in &pr.razine {
        let (v, tr, o) = r.vrhova_odsjecaka();
        println!("  razina greške {:>5.0} m: komada {:>6}, vrhova ispune {v:>8}, trokuta {tr:>8}, odsječaka {o:>8}", r.greska_m, r.komadi.len());
    }
    // Sredina karte: 2. i 98. postotak, da nekoliko točaka na drugom kontinentu ne pomakne sve.
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    let mut srediste = Vec::new();
    for z in sve.iter().filter(|z| !samo_vidljivo || z.vidljivo) {
        if let Some(t) = z.dijelovi.first().and_then(|d| d.first()) {
            xs.push(t[0]);
            ys.push(t[1]);
            srediste.push(*t);
        }
    }
    xs.sort_by(f64::total_cmp);
    ys.sort_by(f64::total_cmp);
    let udio = |v: &[f64], u: f64| v[((v.len() - 1) as f64 * u) as usize];
    let ok = [udio(&xs, 0.02), udio(&ys, 0.02), udio(&xs, 0.98), udio(&ys, 0.98)];
    drop(sve);
    let t = Instant::now();
    let p = napravi_platno();
    let c = Crtac::novi(&p.uredaj, &p.granice, pr.razine, FORMAT, UZORAKA);
    println!("slanje na karticu: {:.2} s, {:.0} MB na kartici; memorija procesa do sada {:.0} MB", t.elapsed().as_secs_f64(), c.bajtova as f64 / 1e6, vrh_memorije_mb());
    let cijela = Kamera {
        sredina: [(ok[0] + ok[2]) / 2.0, (ok[1] + ok[3]) / 2.0],
        metara_po_px: ((ok[2] - ok[0]) / f64::from(SIRINA)).max((ok[3] - ok[1]) / f64::from(VISINA)) * 1.05,
        pola: POLA,
    };
    let mut slucajni = Slucajni(0x1234_5678_9abc);
    // Prvi okvir plaća pripremu cjevovoda, pa se ne broji.
    nacrtaj(&p, &c, cijela);
    for m in [0.5, 2.0, 8.0, 32.0, 128.0, f64::MAX] {
        println!("{}", mjeri_mjerilo(&p, &c, &srediste, m, cijela, &mut slucajni));
    }
    // Najgori slučaj: cijela karta bez ikakvog pojednostavljenja, svih 3 milijuna vrhova.
    let mut vremena = Vec::new();
    let mut zadnji = Okvir::default();
    for _ in 0..30 {
        let t = Instant::now();
        zadnji = nacrtaj_razinu(&p, &c, cijela, 0);
        vremena.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    vremena.sort_by(f64::total_cmp);
    println!(
        "najgore, bez pojednostavljenja: komada {} | trokuta {} | odsječaka {} | medijan {:.2} ms | najgore {:.2} ms",
        zadnji.komada, zadnji.trokuta, zadnji.odsjecaka, vremena[vremena.len() / 2], vremena[vremena.len() - 1]
    );
    println!("najviše memorije procesa: {:.0} MB", vrh_memorije_mb());
    if let (false, Some(mapa)) = (stvarno, a.get(3)) {
        nacrtaj(&p, &c, cijela);
        procitaj(&p).save(format!("{mapa}/cijela_{}.png", a[2])).expect("spremanje");
        nacrtaj(&p, &c, Kamera { sredina: srediste[srediste.len() / 3], metara_po_px: 8.0, pola: POLA });
        procitaj(&p).save(format!("{mapa}/blizu_{}.png", a[2])).expect("spremanje");
    }
}
