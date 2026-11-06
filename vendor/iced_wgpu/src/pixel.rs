//! Passe d'écran du launcher : la fenêtre entière est dessinée dans une texture,
//! puis recopiée par un shader qui ajoute
//! - le mode pixel : texture N fois plus petite, agrandie sans lissage ;
//! - les transitions de page (fondu, glissement, zoom, dissolution, balayage, rideau).
//! Ajout local au crate `iced_wgpu` (voir `vendor/README.md` à la racine du projet).

use std::sync::atomic::{AtomicU32, Ordering};

static FACTOR: AtomicU32 = AtomicU32::new(1);

/// Règle le facteur de pixelisation (1 = désactivé, 2 = pixels de 2 × 2, etc.).
pub fn set_factor(factor: u32) {
    FACTOR.store(factor.clamp(1, 8), Ordering::Relaxed);
}

/// Facteur actuel.
pub fn factor() -> u32 {
    FACTOR.load(Ordering::Relaxed)
}

/// Effet de transition en cours : 0 aucun, 1 fondu, 2 glissement, 3 zoom,
/// 4 dissolution, 5 balayage, 6 rideau.
static EFFECT: AtomicU32 = AtomicU32::new(0);
/// Avancement de 0 à 1 (bits d'un f32).
static PROGRESS: AtomicU32 = AtomicU32::new(0);
/// Couleur de fond utilisée par l'effet (0xRRGGBB).
static BACKGROUND: AtomicU32 = AtomicU32::new(0x0C0E14);

/// Règle l'effet de transition et son avancement (`effect = 0` : pas d'effet).
pub fn set_effect(effect: u32, progress: f32, background: u32) {
    EFFECT.store(effect.min(6), Ordering::Relaxed);
    PROGRESS.store(progress.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    BACKGROUND.store(background, Ordering::Relaxed);
}

/// Une passe est-elle nécessaire pour cette image ?
pub fn active() -> bool {
    factor() > 1 || EFFECT.load(Ordering::Relaxed) != 0
}

/// Colonnes et lignes de la grille de blocs (PROMPT §11 : 16 × 6).
const COLS: u32 = 16;
const ROWS: u32 = 6;

const SHADER: &str = r#"
struct Params {
    // facteur, effet, colonnes, lignes
    ints: vec4<u32>,
    // avancement, fond r, g, b
    floats: vec4<f32>,
};
@group(0) @binding(0) var low: texture_2d<f32>;
@group(0) @binding(1) var<uniform> params: Params;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

// Même hachage que `transition::order` côté interface (aléatoire fixe).
fn hash(n: u32) -> f32 {
    var x = n ^ 0x9E3779B9u;
    x = x ^ (x >> 16u);
    x = x * 0x7FEB352Du;
    x = x ^ (x >> 15u);
    return f32(x % 1000u) / 1000.0;
}

fn sample(p: vec2<f32>, full: vec2<f32>, bg: vec4<f32>) -> vec4<f32> {
    if (p.x < 0.0 || p.y < 0.0 || p.x >= full.x || p.y >= full.y) {
        return bg;
    }
    let size = vec2<i32>(textureDimensions(low));
    let texel = min(vec2<i32>(floor(p)) / i32(params.ints.x), size - vec2<i32>(1));
    return textureLoad(low, texel, 0);
}

fn ease(t: f32) -> f32 {
    return t * t * (3.0 - 2.0 * t);
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let factor = f32(params.ints.x);
    let full = vec2<f32>(textureDimensions(low)) * factor;
    let effect = params.ints.y;
    let p = params.floats.x;
    let bg = vec4<f32>(params.floats.yzw, 1.0);
    // Première moitié : l'ancienne page s'en va ; seconde moitié : la nouvelle arrive.
    let out_t = ease(min(p * 2.0, 1.0));
    let in_t = ease(clamp(p * 2.0 - 1.0, 0.0, 1.0));
    let fade = 1.0 - abs(2.0 * p - 1.0);
    switch effect {
        case 1u: {
            return mix(sample(pos.xy, full, bg), bg, fade);
        }
        case 2u: {
            var shift = 0.0;
            if (p < 0.5) { shift = out_t * 0.25; } else { shift = (in_t - 1.0) * 0.25; }
            let c = sample(pos.xy + vec2<f32>(shift * full.x, 0.0), full, bg);
            return mix(c, bg, fade);
        }
        case 3u: {
            var scale = 1.0;
            if (p < 0.5) { scale = 1.0 - 0.06 * out_t; } else { scale = 1.06 - 0.06 * in_t; }
            let center = full * 0.5;
            let c = sample(center + (pos.xy - center) / scale, full, bg);
            return mix(c, bg, fade);
        }
        case 4u, 5u, 6u: {
            let cols = params.ints.z;
            let rows = params.ints.w;
            let cell = vec2<u32>(floor(pos.xy / (full / vec2<f32>(f32(cols), f32(rows)))));
            let col = min(cell.x, cols - 1u);
            let row = min(cell.y, rows - 1u);
            var order = hash(row * cols + col);
            if (effect == 5u) { order = f32(col) / f32(cols); }
            if (effect == 6u) { order = f32(row) / f32(rows); }
            var covered = false;
            if (p < 0.5) { covered = order < p * 2.0; } else { covered = order >= (p - 0.5) * 2.0; }
            if (covered) { return bg; }
            return sample(pos.xy, full, bg);
        }
        default: {
            return sample(pos.xy, full, bg);
        }
    }
}
"#;

pub(crate) struct Pass {
    format: wgpu::TextureFormat,
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    params: wgpu::Buffer,
    target: Option<(wgpu::Texture, wgpu::TextureView, wgpu::BindGroup, u32, u32)>,
}

impl Pass {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("launcher.pixel.shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("launcher.pixel.layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("launcher.pixel.pipeline_layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("launcher.pixel.pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("launcher.pixel.params"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self { format, pipeline, layout, params, target: None }
    }

    pub(crate) fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Texture basse définition de la bonne taille (recréée si la fenêtre change).
    pub(crate) fn target(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        factor: u32,
    ) -> &wgpu::TextureView {
        let stale = !matches!(&self.target, Some((_, _, _, w, h)) if *w == width && *h == height);
        if stale {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("launcher.pixel.low"),
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("launcher.pixel.bind"),
                layout: &self.layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                    wgpu::BindGroupEntry { binding: 1, resource: self.params.as_entire_binding() },
                ],
            });
            self.target = Some((texture, view, bind, width, height));
        }
        let mut bytes = [0u8; 32];
        let bg = BACKGROUND.load(Ordering::Relaxed);
        let srgb = self.format.is_srgb();
        let channel = |shift: u32| {
            let c = ((bg >> shift) & 0xFF) as f32 / 255.0;
            // Cible sRGB : le shader travaille en linéaire.
            if srgb { if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) } } else { c }
        };
        let ints = [factor, EFFECT.load(Ordering::Relaxed), COLS, ROWS];
        let floats = [f32::from_bits(PROGRESS.load(Ordering::Relaxed)), channel(16), channel(8), channel(0)];
        for (i, v) in ints.iter().enumerate() {
            bytes[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        for (i, v) in floats.iter().enumerate() {
            bytes[16 + i * 4..20 + i * 4].copy_from_slice(&v.to_le_bytes());
        }
        queue.write_buffer(&self.params, 0, &bytes);
        match &self.target {
            Some((_, view, ..)) => view,
            None => unreachable!("la texture vient d'être créée"),
        }
    }

    /// Agrandit la texture basse définition sur l'image finale.
    pub(crate) fn upscale(&self, encoder: &mut wgpu::CommandEncoder, frame: &wgpu::TextureView) {
        let Some((_, _, bind, ..)) = &self.target else { return };
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("launcher.pixel.upscale"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: frame,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind, &[]);
        pass.draw(0..3, 0..1);
    }
}
