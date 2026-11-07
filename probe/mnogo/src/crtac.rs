//! Crtanje pripremljene karte na grafičkoj kartici.
//!
//! Isti kod crta i bez prozora, za mjerenje, i u prozoru s kartom. Pomicanje i približavanje
//! mijenjaju samo jednu matricu, a vrhovi na kartici ostaju isti.

use crate::priprema::{Odsjecak, Razina, VrhIspune};
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const SJENCAR: &str = r#"
struct Pogled { pomak: vec2<f32>, mjerilo: f32, _p: f32, pola: vec2<f32>, _q: vec2<f32> };
@group(0) @binding(0) var<uniform> p: Pogled;
struct Izlaz { @builtin(position) pos: vec4<f32>, @location(0) boja: vec4<f32> };
fn t_px(t: vec2<f32>) -> vec2<f32> { return t * p.mjerilo + p.pomak; }
@vertex fn v_ispuna(@location(0) t: vec2<f32>, @location(1) b: vec4<f32>) -> Izlaz {
    var i: Izlaz; i.pos = vec4(t_px(t) / p.pola, 0.0, 1.0); i.boja = b; return i;
}
@vertex fn v_crta(@builtin(vertex_index) k: u32, @location(0) a: vec2<f32>, @location(1) c: vec2<f32>,
                  @location(2) b: vec4<f32>, @location(3) s: f32) -> Izlaz {
    var kut = array<vec2<f32>, 6>(vec2(0.,-1.), vec2(1.,-1.), vec2(1.,1.), vec2(0.,-1.), vec2(1.,1.), vec2(0.,1.));
    let u = kut[k];
    let pa = t_px(a); let pc = t_px(c);
    var d = pc - pa;
    let dl = length(d);
    var o: vec2<f32>;
    if (dl < 0.0001) {
        // Točka je kvadrat širine s, a lik manji od piksela postaje isto takav kvadrat.
        o = pa + vec2(u.x * 2.0 - 1.0, u.y) * s * 0.5;
    } else {
        d = d / dl;
        o = mix(pa, pc, u.x) + vec2(-d.y, d.x) * u.y * s * 0.5;
    }
    var i: Izlaz; i.pos = vec4(o / p.pola, 0.0, 1.0); i.boja = b; return i;
}
@fragment fn f(i: Izlaz) -> @location(0) vec4<f32> { return i.boja; }
"#;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Pogled {
    pomak: [f32; 2],
    mjerilo: f32,
    _p: f32,
    pola: [f32; 2],
    _q: [f32; 2],
}

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

pub struct Crtac {
    ispuna: wgpu::RenderPipeline,
    crta: wgpu::RenderPipeline,
    vrhovi: wgpu::Buffer,
    indeksi: wgpu::Buffer,
    odsjecci: wgpu::Buffer,
    pogledi: wgpu::Buffer,
    veza: wgpu::BindGroup,
    razmak: u32,
    razine: Vec<GpuRazina>,
    pub bajtova: u64,
}

/// `pola` je pola ekrana u istim jedinicama kao širina crte: pikseli ili točke.
#[derive(Clone, Copy, Debug)]
pub struct Kamera {
    pub sredina: [f64; 2],
    pub metara_po_px: f64,
    pub pola: [f32; 2],
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Okvir {
    pub komada: u32,
    pub trokuta: u64,
    pub odsjecaka: u64,
    pub razina: usize,
}

#[derive(Default)]
pub struct Plan {
    razina: usize,
    komadi: Vec<usize>,
}

impl Crtac {
    pub fn novi(uredaj: &wgpu::Device, granice: &wgpu::Limits, razine: Vec<Razina>, format: wgpu::TextureFormat, uzoraka: u32) -> Self {
        let mut sve_vrhovi: Vec<VrhIspune> = Vec::new();
        let mut svi_indeksi: Vec<u32> = Vec::new();
        let mut svi_odsjecci: Vec<Odsjecak> = Vec::new();
        let mut gpu_razine = Vec::new();
        let mut najvise = 1;
        for r in razine {
            let mut komadi = Vec::new();
            for k in r.komadi {
                let (iv, ii, io) = (sve_vrhovi.len() as i32, svi_indeksi.len() as u32, svi_odsjecci.len() as u32);
                sve_vrhovi.extend_from_slice(&k.vrhovi);
                svi_indeksi.extend_from_slice(&k.indeksi);
                svi_odsjecci.extend_from_slice(&k.odsjecci);
                komadi.push(GpuKomad {
                    ishodiste: k.ishodiste,
                    okvir: k.okvir,
                    pocetak_vrha: iv,
                    indeksi: ii..svi_indeksi.len() as u32,
                    odsjecci: io..svi_odsjecci.len() as u32,
                });
            }
            najvise = najvise.max(komadi.len());
            gpu_razine.push(GpuRazina { greska_m: r.greska_m, komadi });
        }
        let bajtova = (std::mem::size_of_val(sve_vrhovi.as_slice())
            + std::mem::size_of_val(svi_indeksi.as_slice())
            + std::mem::size_of_val(svi_odsjecci.as_slice())) as u64;
        // Prazan spremnik kartica ne prima, pa uvijek ima barem jedan element.
        let spremnik = |naziv, sadrzaj: &[u8], u| {
            let sadrzaj: &[u8] = if sadrzaj.is_empty() { &[0u8; 64] } else { sadrzaj };
            uredaj.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(naziv), contents: sadrzaj, usage: u })
        };
        let vrhovi = spremnik("vrhovi", bytemuck::cast_slice(&sve_vrhovi), wgpu::BufferUsages::VERTEX);
        let indeksi = spremnik("indeksi", bytemuck::cast_slice(&svi_indeksi), wgpu::BufferUsages::INDEX);
        let odsjecci = spremnik("odsjecci", bytemuck::cast_slice(&svi_odsjecci), wgpu::BufferUsages::VERTEX);

        let razmak = granice.min_uniform_buffer_offset_alignment.max(64);
        let pogledi = uredaj.create_buffer(&wgpu::BufferDescriptor {
            label: Some("pogledi"),
            size: u64::from(razmak) * (najvise as u64 + 1),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let raspored = uredaj.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(32),
                },
                count: None,
            }],
        });
        let veza = uredaj.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &raspored,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &pogledi, offset: 0, size: wgpu::BufferSize::new(32) }),
            }],
        });
        let sjencar = uredaj.create_shader_module(wgpu::ShaderModuleDescriptor { label: None, source: wgpu::ShaderSource::Wgsl(SJENCAR.into()) });
        let raspored_cjevovoda = uredaj.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: None, bind_group_layouts: &[Some(&raspored)], immediate_size: 0 });
        let boja = [Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::ALPHA_BLENDING), write_mask: wgpu::ColorWrites::ALL })];
        let cjevovod = |vrh: &str, raspored_vrha: wgpu::VertexBufferLayout| {
            uredaj.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(&raspored_cjevovoda),
                vertex: wgpu::VertexState { module: &sjencar, entry_point: Some(vrh), compilation_options: Default::default(), buffers: &[Some(raspored_vrha)] },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState { count: uzoraka, mask: !0, alpha_to_coverage_enabled: false },
                fragment: Some(wgpu::FragmentState { module: &sjencar, entry_point: Some("f"), compilation_options: Default::default(), targets: &boja }),
                multiview_mask: None,
                cache: None,
            })
        };
        let ispuna = cjevovod(
            "v_ispuna",
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<VrhIspune>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Unorm8x4],
            },
        );
        let crta = cjevovod(
            "v_crta",
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Odsjecak>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4, 3 => Float32],
            },
        );
        Self { ispuna, crta, vrhovi, indeksi, odsjecci, pogledi, veza, razmak, razine: gpu_razine, bajtova }
    }

    /// Najgrublja razina čija je greška ispod jednog piksela.
    pub fn razina_za(&self, metara_po_px: f64) -> usize {
        self.razine.iter().rposition(|r| r.greska_m <= metara_po_px).unwrap_or(0)
    }

    /// Bira komade na ekranu i šalje njihove pomake kartici. Ide prije prolaza crtanja.
    pub fn pripremi(&self, red: &wgpu::Queue, k: Kamera, razina: usize) -> Plan {
        let pola_m = [f64::from(k.pola[0]) * k.metara_po_px, f64::from(k.pola[1]) * k.metara_po_px];
        let komadi: Vec<usize> = self.razine[razina]
            .komadi
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.okvir[2] >= k.sredina[0] - pola_m[0]
                    && c.okvir[0] <= k.sredina[0] + pola_m[0]
                    && c.okvir[3] >= k.sredina[1] - pola_m[1]
                    && c.okvir[1] <= k.sredina[1] + pola_m[1]
            })
            .map(|(i, _)| i)
            .collect();
        let mut bajtovi = vec![0u8; komadi.len() * self.razmak as usize];
        for (i, indeks) in komadi.iter().enumerate() {
            let c = &self.razine[razina].komadi[*indeks];
            let p = Pogled {
                pomak: [((c.ishodiste[0] - k.sredina[0]) / k.metara_po_px) as f32, ((c.ishodiste[1] - k.sredina[1]) / k.metara_po_px) as f32],
                mjerilo: (1.0 / k.metara_po_px) as f32,
                _p: 0.0,
                pola: k.pola,
                _q: [0.0; 2],
            };
            let o = i * self.razmak as usize;
            bajtovi[o..o + 32].copy_from_slice(bytemuck::bytes_of(&p));
        }
        if !bajtovi.is_empty() {
            red.write_buffer(&self.pogledi, 0, &bajtovi);
        }
        Plan { razina, komadi }
    }

    pub fn crtaj(&self, prolaz: &mut wgpu::RenderPass<'_>, plan: &Plan) -> Okvir {
        let mut okvir = Okvir { razina: plan.razina, komada: plan.komadi.len() as u32, ..Okvir::default() };
        prolaz.set_pipeline(&self.ispuna);
        prolaz.set_vertex_buffer(0, self.vrhovi.slice(..));
        prolaz.set_index_buffer(self.indeksi.slice(..), wgpu::IndexFormat::Uint32);
        for (i, indeks) in plan.komadi.iter().enumerate() {
            let c = &self.razine[plan.razina].komadi[*indeks];
            if !c.indeksi.is_empty() {
                prolaz.set_bind_group(0, &self.veza, &[i as u32 * self.razmak]);
                prolaz.draw_indexed(c.indeksi.clone(), c.pocetak_vrha, 0..1);
                okvir.trokuta += u64::from(c.indeksi.len() as u32 / 3);
            }
        }
        prolaz.set_pipeline(&self.crta);
        prolaz.set_vertex_buffer(0, self.odsjecci.slice(..));
        for (i, indeks) in plan.komadi.iter().enumerate() {
            let c = &self.razine[plan.razina].komadi[*indeks];
            if !c.odsjecci.is_empty() {
                prolaz.set_bind_group(0, &self.veza, &[i as u32 * self.razmak]);
                prolaz.draw(0..6, c.odsjecci.clone());
                okvir.odsjecaka += u64::from(c.odsjecci.len() as u32);
            }
        }
        okvir
    }
}
