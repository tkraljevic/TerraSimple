//! Crtanje pripremljenih slojeva na grafičkoj kartici.
//!
//! Vrhovi ostaju na kartici, a pomicanje i približavanje mijenjaju samo nekoliko brojeva. Kroz `egui`
//! bi ista količina samo za pripremu crta trajala 65 ms u svakom okviru (probe/mnogo).

use crate::kamera::{Kamera, Pogled};
use crate::priprema::{Odsjecak, Razina, VrhIspune};
use eframe::egui;
use eframe::egui_wgpu::{self, CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use slojevi::IdSloja;
use std::collections::BTreeMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;

const SJENCAR: &str = r#"
struct Pogled { pomak: vec2<f32>, mjerilo: f32, _p: f32, pola: vec2<f32>, _q: vec2<f32> };
@group(0) @binding(0) var<uniform> p: Pogled;
struct Izlaz {
    @builtin(position) pos: vec4<f32>,
    @location(0) boja: vec4<f32>,
    @location(1) lok: vec2<f32>,
    @location(2) krug: f32,
    // Udaljenost od početka crte u točkama zaslona, uzorak i debljina crte.
    @location(3) duz: f32,
    @location(4) uzorak: f32,
    @location(5) s: f32,
};
fn t_px(t: vec2<f32>) -> vec2<f32> { return t * p.mjerilo + p.pomak; }
@vertex fn v_ispuna(@location(0) t: vec2<f32>, @location(1) b: vec4<f32>, @location(2) uz: f32) -> Izlaz {
    var i: Izlaz; i.pos = vec4(t_px(t) / p.pola, 0.0, 1.0); i.boja = b; i.lok = vec2(0.0); i.krug = 0.0;
    i.duz = 0.0; i.uzorak = uz; i.s = 0.0; return i;
}
@vertex fn v_crta(@builtin(vertex_index) k: u32, @location(0) a: vec2<f32>, @location(1) c: vec2<f32>,
                  @location(2) b: vec4<f32>, @location(3) s: f32, @location(4) put: f32,
                  @location(5) uz: f32) -> Izlaz {
    var kut = array<vec2<f32>, 6>(vec2(0.,-1.), vec2(1.,-1.), vec2(1.,1.), vec2(0.,-1.), vec2(1.,1.), vec2(0.,1.));
    let u = kut[k];
    let pa = t_px(a); let pc = t_px(c);
    var d = pc - pa;
    let dl = length(d);
    var i: Izlaz;
    i.boja = b;
    i.uzorak = uz;
    i.s = s;
    i.duz = put * p.mjerilo + u.x * dl;
    if (dl < 0.0001) {
        // Točka je krug širine s, a lik manji od piksela postaje isto takav krug.
        let l = vec2(u.x * 2.0 - 1.0, u.y);
        i.pos = vec4((pa + l * s * 0.5) / p.pola, 0.0, 1.0);
        i.lok = l;
        i.krug = 1.0;
    } else {
        d = d / dl;
        let o = mix(pa, pc, u.x) + vec2(-d.y, d.x) * u.y * s * 0.5;
        i.pos = vec4(o / p.pola, 0.0, 1.0);
        i.lok = vec2(0.0);
        i.krug = 0.0;
    }
    return i;
}
// Isprekidana crta: 4 debljine crte i 2 razmaka. Točkasta: 1 i 1,5. Crta-točka: 4, 2, 1, 2.
fn u_razmaku(duz: f32, uzorak: f32, s: f32) -> bool {
    let j = max(s, 1.5);
    if (uzorak < 0.5) { return false; }
    if (uzorak < 1.5) { return duz % (6.0 * j) > 4.0 * j; }
    if (uzorak < 2.5) { return duz % (2.5 * j) > j; }
    let m = duz % (9.0 * j);
    return (m > 4.0 * j && m < 6.0 * j) || m > 7.0 * j;
}
@fragment fn f(i: Izlaz) -> @location(0) vec4<f32> {
    if (i.krug > 0.5 && dot(i.lok, i.lok) > 1.0) { discard; }
    // Šrafura su kose crte svakih 10 piksela zaslona, debele 2 piksela.
    if (i.uzorak > 9.5) {
        if ((i.pos.x + i.pos.y) % 10.0 > 2.0) { discard; }
    } else if (u_razmaku(i.duz, i.uzorak, i.s)) { discard; }
    // `egui` crta s bojom već pomnoženom prozirnošću, a vrhovi nose običnu boju.
    return vec4(i.boja.rgb * i.boja.a, i.boja.a);
}
"#;

const VELICINA_POGLEDA: u64 = 32;

const fn atribut(
    format: wgpu::VertexFormat,
    offset: u64,
    shader_location: u32,
) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format,
        offset,
        shader_location,
    }
}

/// Mora se slagati s `VrhIspune`.
const ATRIBUTI_ISPUNE: [wgpu::VertexAttribute; 3] = [
    atribut(wgpu::VertexFormat::Float32x2, 0, 0),
    atribut(wgpu::VertexFormat::Unorm8x4, 8, 1),
    atribut(wgpu::VertexFormat::Float32, 12, 2),
];

/// Mora se slagati s `Odsjecak`.
const ATRIBUTI_CRTE: [wgpu::VertexAttribute; 6] = [
    atribut(wgpu::VertexFormat::Float32x2, 0, 0),
    atribut(wgpu::VertexFormat::Float32x2, 8, 1),
    atribut(wgpu::VertexFormat::Unorm8x4, 16, 2),
    atribut(wgpu::VertexFormat::Float32, 20, 3),
    atribut(wgpu::VertexFormat::Float32, 24, 4),
    atribut(wgpu::VertexFormat::Float32, 28, 5),
];

struct GpuKomad {
    ishodiste: [f64; 2],
    okvir: [f64; 4],
    pocetak_vrha: i32,
    indeksi: std::ops::Range<u32>,
    odsjecci: std::ops::Range<u32>,
}

struct GpuRazina {
    greska_m: f64,
    komadi: Vec<GpuKomad>,
}

struct GpuSloj {
    izvor: Arc<Vec<Razina>>,
    vrhovi: wgpu::Buffer,
    indeksi: wgpu::Buffer,
    odsjecci: wgpu::Buffer,
    razine: Vec<GpuRazina>,
}

/// Komad koji se crta u ovom okviru, redom crtanja.
struct Stavka {
    sloj: IdSloja,
    razina: usize,
    komad: usize,
}

pub(crate) struct Crtac {
    ispuna: wgpu::RenderPipeline,
    crta: wgpu::RenderPipeline,
    raspored: wgpu::BindGroupLayout,
    pogledi: wgpu::Buffer,
    veza: wgpu::BindGroup,
    razmak: u64,
    kapacitet: u64,
    slojevi: BTreeMap<IdSloja, GpuSloj>,
    plan: Vec<Stavka>,
}

fn spremnik(
    uredaj: &wgpu::Device,
    naziv: &str,
    sadrzaj: &[u8],
    u: wgpu::BufferUsages,
) -> wgpu::Buffer {
    // Prazan spremnik kartica ne prima, pa uvijek ima barem nešto.
    let sadrzaj: &[u8] = if sadrzaj.is_empty() {
        &[0u8; 64]
    } else {
        sadrzaj
    };
    uredaj.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(naziv),
        contents: sadrzaj,
        usage: u,
    })
}

fn poslozi_komade(razine: &[Razina]) -> (Vec<VrhIspune>, Vec<u32>, Vec<Odsjecak>, Vec<GpuRazina>) {
    let (mut v, mut i, mut o) = (Vec::new(), Vec::new(), Vec::new());
    let mut gpu = Vec::with_capacity(razine.len());
    for r in razine {
        let mut komadi = Vec::with_capacity(r.komadi.len());
        for k in &r.komadi {
            let (pv, pi, po) = (v.len() as i32, i.len() as u32, o.len() as u32);
            v.extend_from_slice(&k.vrhovi);
            i.extend_from_slice(&k.indeksi);
            o.extend_from_slice(&k.odsjecci);
            komadi.push(GpuKomad {
                ishodiste: k.ishodiste,
                okvir: k.okvir,
                pocetak_vrha: pv,
                indeksi: pi..i.len() as u32,
                odsjecci: po..o.len() as u32,
            });
        }
        gpu.push(GpuRazina {
            greska_m: r.greska_m,
            komadi,
        });
    }
    (v, i, o, gpu)
}

fn novi_sloj(uredaj: &wgpu::Device, izvor: Arc<Vec<Razina>>) -> GpuSloj {
    let (v, i, o, razine) = poslozi_komade(&izvor);
    GpuSloj {
        vrhovi: spremnik(
            uredaj,
            "vrhovi",
            bytemuck::cast_slice(&v),
            wgpu::BufferUsages::VERTEX,
        ),
        indeksi: spremnik(
            uredaj,
            "indeksi",
            bytemuck::cast_slice(&i),
            wgpu::BufferUsages::INDEX,
        ),
        odsjecci: spremnik(
            uredaj,
            "odsjecci",
            bytemuck::cast_slice(&o),
            wgpu::BufferUsages::VERTEX,
        ),
        razine,
        izvor,
    }
}

fn novi_pogledi(uredaj: &wgpu::Device, velicina: u64) -> wgpu::Buffer {
    uredaj.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pogledi"),
        size: velicina,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn nova_veza(
    uredaj: &wgpu::Device,
    raspored: &wgpu::BindGroupLayout,
    pogledi: &wgpu::Buffer,
) -> wgpu::BindGroup {
    uredaj.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: raspored,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: pogledi,
                offset: 0,
                size: wgpu::BufferSize::new(VELICINA_POGLEDA),
            }),
        }],
    })
}

fn novi_raspored(uredaj: &wgpu::Device) -> wgpu::BindGroupLayout {
    uredaj.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: wgpu::BufferSize::new(VELICINA_POGLEDA),
            },
            count: None,
        }],
    })
}

struct Cjevovodi<'a> {
    uredaj: &'a wgpu::Device,
    raspored: wgpu::PipelineLayout,
    sjencar: wgpu::ShaderModule,
    format: wgpu::TextureFormat,
    uzoraka: u32,
}

impl Cjevovodi<'_> {
    fn novi(&self, vrh: &str, raspored_vrha: wgpu::VertexBufferLayout<'_>) -> wgpu::RenderPipeline {
        let boja = [Some(wgpu::ColorTargetState {
            format: self.format,
            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        self.uredaj
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&self.raspored),
                vertex: wgpu::VertexState {
                    module: &self.sjencar,
                    entry_point: Some(vrh),
                    compilation_options: Default::default(),
                    buffers: &[Some(raspored_vrha)],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: self.uzoraka,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.sjencar,
                    entry_point: Some("f"),
                    compilation_options: Default::default(),
                    targets: &boja,
                }),
                multiview_mask: None,
                cache: None,
            })
    }
}

impl Crtac {
    pub(crate) fn novi(
        uredaj: &wgpu::Device,
        granice: &wgpu::Limits,
        format: wgpu::TextureFormat,
        uzoraka: u32,
    ) -> Self {
        let raspored = novi_raspored(uredaj);
        let c = Cjevovodi {
            uredaj,
            raspored: uredaj.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&raspored)],
                immediate_size: 0,
            }),
            sjencar: uredaj.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::ShaderSource::Wgsl(SJENCAR.into()),
            }),
            format,
            uzoraka,
        };
        let ispuna = c.novi(
            "v_ispuna",
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<VrhIspune>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &ATRIBUTI_ISPUNE,
            },
        );
        let crta = c.novi(
            "v_crta",
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Odsjecak>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &ATRIBUTI_CRTE,
            },
        );
        let razmak = u64::from(granice.min_uniform_buffer_offset_alignment).max(64);
        let pogledi = novi_pogledi(uredaj, razmak);
        let veza = nova_veza(uredaj, &raspored, &pogledi);
        Self {
            ispuna,
            crta,
            raspored,
            pogledi,
            veza,
            razmak,
            kapacitet: 1,
            slojevi: BTreeMap::new(),
            plan: Vec::new(),
        }
    }

    /// Šalje kartici samo slojeve koji su se promijenili, a slojeve kojih više nema briše.
    fn uskladi(&mut self, uredaj: &wgpu::Device, slojevi: &[(IdSloja, Arc<Vec<Razina>>)]) {
        self.slojevi
            .retain(|id, _| slojevi.iter().any(|(s, _)| s == id));
        for (id, izvor) in slojevi {
            let isti = self
                .slojevi
                .get(id)
                .is_some_and(|s| Arc::ptr_eq(&s.izvor, izvor));
            if !isti {
                self.slojevi.insert(*id, novi_sloj(uredaj, izvor.clone()));
            }
        }
    }

    /// Bira komade na ekranu i šalje njihove pomake kartici. Ide prije prolaza crtanja.
    fn pripremi(
        &mut self,
        uredaj: &wgpu::Device,
        red: &wgpu::Queue,
        k: Kamera,
        redoslijed: &[IdSloja],
    ) {
        self.plan = izaberi(&self.slojevi, k, redoslijed);
        let potrebno = self.plan.len() as u64;
        if potrebno > self.kapacitet {
            self.kapacitet = potrebno.next_power_of_two();
            self.pogledi = novi_pogledi(uredaj, self.kapacitet * self.razmak);
            self.veza = nova_veza(uredaj, &self.raspored, &self.pogledi);
        }
        let razmak = self.razmak as usize;
        let mut bajtovi = vec![0u8; self.plan.len() * razmak];
        for (s, mjesto) in self.plan.iter().zip(bajtovi.chunks_mut(razmak)) {
            let komad = &self.slojevi[&s.sloj].razine[s.razina].komadi[s.komad];
            let pogled: Pogled = k.pogled(komad.ishodiste);
            mjesto[..VELICINA_POGLEDA as usize].copy_from_slice(bytemuck::bytes_of(&pogled));
        }
        if !bajtovi.is_empty() {
            red.write_buffer(&self.pogledi, 0, &bajtovi);
        }
    }

    fn crtaj(&self, prolaz: &mut wgpu::RenderPass<'_>) {
        for (i, s) in self.plan.iter().enumerate() {
            let sloj = &self.slojevi[&s.sloj];
            let komad = &sloj.razine[s.razina].komadi[s.komad];
            prolaz.set_bind_group(0, &self.veza, &[(i as u64 * self.razmak) as u32]);
            if !komad.indeksi.is_empty() {
                prolaz.set_pipeline(&self.ispuna);
                prolaz.set_vertex_buffer(0, sloj.vrhovi.slice(..));
                prolaz.set_index_buffer(sloj.indeksi.slice(..), wgpu::IndexFormat::Uint32);
                prolaz.draw_indexed(komad.indeksi.clone(), komad.pocetak_vrha, 0..1);
            }
            if !komad.odsjecci.is_empty() {
                prolaz.set_pipeline(&self.crta);
                prolaz.set_vertex_buffer(0, sloj.odsjecci.slice(..));
                prolaz.draw(0..6, komad.odsjecci.clone());
            }
        }
    }
}

fn izaberi(slojevi: &BTreeMap<IdSloja, GpuSloj>, k: Kamera, redoslijed: &[IdSloja]) -> Vec<Stavka> {
    let mut plan = Vec::new();
    for sloj in redoslijed {
        let Some(s) = slojevi.get(sloj) else {
            continue;
        };
        let greske: Vec<f64> = s.razine.iter().map(|r| r.greska_m).collect();
        let razina = k.razina(&greske);
        for (komad, c) in s.razine[razina].komadi.iter().enumerate() {
            if k.vidi(c.okvir) {
                plan.push(Stavka {
                    sloj: *sloj,
                    razina,
                    komad,
                });
            }
        }
    }
    plan
}

/// Što karta u jednom okviru traži od kartice.
pub(crate) struct Poziv {
    pub kamera: Kamera,
    /// Redom crtanja, odozdo prema gore.
    pub slojevi: Arc<Vec<(IdSloja, Arc<Vec<Razina>>)>>,
}

impl CallbackTrait for Poziv {
    fn prepare(
        &self,
        uredaj: &wgpu::Device,
        red: &wgpu::Queue,
        _ekran: &ScreenDescriptor,
        _enkoder: &mut wgpu::CommandEncoder,
        resursi: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        if let Some(c) = resursi.get_mut::<Crtac>() {
            let redoslijed: Vec<IdSloja> = self.slojevi.iter().map(|(id, _)| *id).collect();
            c.uskladi(uredaj, &self.slojevi);
            c.pripremi(uredaj, red, self.kamera, &redoslijed);
        }
        Vec::new()
    }

    fn paint(
        &self,
        _info: egui::PaintCallbackInfo,
        prolaz: &mut wgpu::RenderPass<'static>,
        resursi: &CallbackResources,
    ) {
        if let Some(c) = resursi.get::<Crtac>() {
            c.crtaj(prolaz);
        }
    }
}

pub(crate) fn poziv(okvir: egui::Rect, poziv: Poziv) -> egui::PaintCallback {
    egui_wgpu::Callback::new_paint_callback(okvir, poziv)
}

/// Priprema karticu. `uzoraka` mora biti isti broj kao u postavkama prozora.
pub fn ugradi(stanje: &RenderState, uzoraka: u32) {
    let granice = stanje.device.limits();
    let crtac = Crtac::novi(&stanje.device, &granice, stanje.target_format, uzoraka);
    stanje.renderer.write().callback_resources.insert(crtac);
    walkers::install_renderer(Some(stanje));
}

/// Crtanje bez prozora, za provjeru na pravoj kartici.
pub mod provjera {
    use super::*;
    use crate::crtanje::slojevi_odozgo;
    use crate::mercator::{metara_po_stupnju, u_mercator};
    use crate::priprema::pripremi_sloj;
    use slojevi::Uredjivac;

    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

    /// Slika u RGBA, redak po redak.
    pub struct Slika {
        pub sirina: u32,
        pub bajtovi: Vec<u8>,
    }

    impl Slika {
        pub fn piksel(&self, x: u32, y: u32) -> [u8; 4] {
            let i = ((y * self.sirina + x) * 4) as usize;
            [
                self.bajtovi[i],
                self.bajtovi[i + 1],
                self.bajtovi[i + 2],
                self.bajtovi[i + 3],
            ]
        }
    }

    async fn uredaj() -> Result<(wgpu::Device, wgpu::Queue, wgpu::Limits), String> {
        let instanca = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instanca
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|g| g.to_string())?;
        let (u, r) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|g| g.to_string())?;
        // Granice otvorene kartice, a ne adaptera: adapter smije dopustiti više nego kartica sa zadanim granicama.
        let granice = u.limits();
        Ok((u, r, granice))
    }

    /// Crta sve slojeve na prozirnu podlogu.
    pub async fn nacrtaj_bez_prozora(
        u: &Uredjivac,
        sredina: [f64; 2],
        zum: f64,
        velicina: [u32; 2],
    ) -> Result<Slika, String> {
        let (uredaj, red, granice) = uredaj().await?;
        let mut crtac = Crtac::novi(&uredaj, &granice, FORMAT, 1);
        let projekt = u.projekt();
        let slojevi: Vec<(IdSloja, Arc<Vec<Razina>>)> = slojevi_odozgo(projekt)
            .into_iter()
            .rev()
            .filter_map(|id| Some((id, Arc::new(pripremi_sloj(projekt.sloj(id)?, None)))))
            .collect();
        let redoslijed: Vec<IdSloja> = slojevi.iter().map(|(id, _)| *id).collect();
        let kamera = Kamera {
            sredina: u_mercator(sredina[0], sredina[1]),
            metara_po_px: 360.0 * metara_po_stupnju() / (256.0 * 2.0_f64.powf(zum)),
            pola: [velicina[0] as f32 / 2.0, velicina[1] as f32 / 2.0],
        };
        crtac.uskladi(&uredaj, &slojevi);
        crtac.pripremi(&uredaj, &red, kamera, &redoslijed);
        let bajtovi = iscrtaj(&uredaj, &red, &crtac, velicina)?;
        Ok(Slika {
            sirina: velicina[0],
            bajtovi,
        })
    }

    fn iscrtaj(
        uredaj: &wgpu::Device,
        red: &wgpu::Queue,
        crtac: &Crtac,
        v: [u32; 2],
    ) -> Result<Vec<u8>, String> {
        let opseg = wgpu::Extent3d {
            width: v[0],
            height: v[1],
            depth_or_array_layers: 1,
        };
        let tekstura = uredaj.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: opseg,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let redak = v[0] * 4;
        let poravnato = redak.div_ceil(256) * 256;
        let izlaz = uredaj.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(poravnato) * u64::from(v[1]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let pogled = tekstura.create_view(&Default::default());
        let mut e = uredaj.create_command_encoder(&Default::default());
        {
            let mut prolaz = e.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &pogled,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            crtac.crtaj(&mut prolaz);
        }
        e.copy_texture_to_buffer(
            tekstura.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &izlaz,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(poravnato),
                    rows_per_image: Some(v[1]),
                },
            },
            opseg,
        );
        red.submit([e.finish()]);
        let (tx, rx) = std::sync::mpsc::channel();
        izlaz.slice(..).map_async(wgpu::MapMode::Read, move |r| {
            let _ishod_javlja_primatelj = tx.send(r);
        });
        uredaj
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|g| g.to_string())?;
        rx.recv()
            .map_err(|g| g.to_string())?
            .map_err(|g| g.to_string())?;
        let podaci = izlaz
            .slice(..)
            .get_mapped_range()
            .map_err(|g| g.to_string())?;
        Ok(podaci
            .chunks(poravnato as usize)
            .flat_map(|r| r[..redak as usize].iter().copied())
            .collect())
    }
}
