use std::borrow::Cow;
use std::sync::Arc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger");
}

// Fullscreen triangle vertex shader (same as vibe-renderer's full_screen_vertex.wgsl)
const VERTEX_SHADER: &str = r#"
const VERTICES: array<vec2f, 3> = array(
    vec2f(-3., -1.),
    vec2f(1., -1.),
    vec2f(1., 3.)
);

@vertex
fn main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4f {
    return vec4f(VERTICES[idx], 0., 1.);
}
"#;

// Web-specific fragment preamble (uniform-only for WebGL2 compat; freqs packed as vec4f)
const FRAGMENT_PREAMBLE: &str = r#"
@group(0) @binding(0) var<uniform> iResolution: vec2f;
@group(0) @binding(1) var<uniform> _freqs_packed: array<vec4f, 64>;
@group(0) @binding(2) var<uniform> iTime: f32;
@group(0) @binding(3) var<uniform> iMouse: vec2f;
@group(0) @binding(4) var<uniform> iBPM: f32;

struct ColorPalette {
    color1: vec4f,
    color2: vec4f,
    color3: vec4f,
    color4: vec4f,
}

@group(0) @binding(5) var<uniform> iColors: ColorPalette;
@group(0) @binding(6) var<uniform> iMouseClick: vec4f;

const FREQ_COUNT: u32 = 256u;

fn get_freq(idx: u32) -> f32 {
    return _freqs_packed[idx / 4u][idx % 4u];
}
"#;

// Simple fallback shader — minimal, uses only iTime to verify pipeline works
const FALLBACK_SHADER: &str = r#"
@fragment
fn main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv = pos.xy / iResolution;
    return vec4f(uv.x, 0.2 + 0.2 * sin(iTime), uv.y, 1.0);
}
"#;

const DEFAULT_FREQ_COUNT: usize = 256;

fn now_secs() -> f64 {
    web_sys::window()
        .and_then(|w| w.performance())
        .map(|p| p.now() / 1000.0)
        .unwrap_or(0.0)
}

#[wasm_bindgen]
pub struct VibeApp {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    format: wgpu::TextureFormat,

    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    vertex_module: wgpu::ShaderModule,

    iresolution: wgpu::Buffer,
    freqs: wgpu::Buffer,
    itime: wgpu::Buffer,
    imouse: wgpu::Buffer,
    ibpm: wgpu::Buffer,
    icolors: wgpu::Buffer,
    imouseclick: wgpu::Buffer,

    sensitivity: f32,
    start_time: f64,
}

impl VibeApp {
    fn make_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let uniform_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("vibe-web bind group layout"),
            entries: &[
                uniform_entry(0), // iResolution
                uniform_entry(1), // freqs (packed as array<vec4f, 64>)
                uniform_entry(2), // iTime
                uniform_entry(3), // iMouse
                uniform_entry(4), // iBPM
                uniform_entry(5), // iColors
                uniform_entry(6), // iMouseClick
            ],
        })
    }

    fn make_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        iresolution: &wgpu::Buffer,
        freqs: &wgpu::Buffer,
        itime: &wgpu::Buffer,
        imouse: &wgpu::Buffer,
        ibpm: &wgpu::Buffer,
        icolors: &wgpu::Buffer,
        imouseclick: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("vibe-web bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: iresolution.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: freqs.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: itime.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: imouse.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: ibpm.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: icolors.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 6, resource: imouseclick.as_entire_binding() },
            ],
        })
    }

    fn make_pipeline(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        vertex_module: &wgpu::ShaderModule,
        fragment_code: &str,
        format: wgpu::TextureFormat,
    ) -> wgpu::RenderPipeline {
        let full_code = format!("{}\n{}", FRAGMENT_PREAMBLE, fragment_code);
        let fragment_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("User fragment shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(full_code)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("vibe-web pipeline layout"),
            bind_group_layouts: &[layout],
            ..Default::default()
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("vibe-web render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: vertex_module,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &fragment_module,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            multiview_mask: None,
            cache: None,
        })
    }

    fn rebuild_bind_group(&mut self) {
        self.bind_group = Self::make_bind_group(
            &self.device,
            &self.bind_group_layout,
            &self.iresolution,
            &self.freqs,
            &self.itime,
            &self.imouse,
            &self.ibpm,
            &self.icolors,
            &self.imouseclick,
        );
    }

    fn elapsed_secs(&self) -> f32 {
        (now_secs() - self.start_time) as f32
    }
}

#[wasm_bindgen]
impl VibeApp {
    #[wasm_bindgen(constructor)]
    pub async fn new(canvas_id: &str) -> Result<VibeApp, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;
        let canvas: HtmlCanvasElement = document
            .get_element_by_id(canvas_id)
            .ok_or("Canvas element not found")?
            .dyn_into()
            .map_err(|_| "Element is not a canvas")?;

        let width = canvas.width().max(1);
        let height = canvas.height().max(1);

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU | wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas))
            .map_err(|e| JsValue::from_str(&format!("{e}")))?;

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
        {
            Ok(a) => {
                log::info!("Got adapter: {:?}", a.get_info());
                a
            }
            Err(_) => {
                log::warn!("No high-perf adapter, trying software fallback...");
                instance
                    .request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::LowPower,
                        compatible_surface: Some(&surface),
                        force_fallback_adapter: true,
                    })
                    .await
                    .map_err(|e| {
                        JsValue::from_str(&format!(
                            "No GPU adapter (tried HW + SW fallback): {e}"
                        ))
                    })?
            }
        };

        let (device, queue): (wgpu::Device, wgpu::Queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|e| JsValue::from_str(&format!("{e}")))?;

        device.on_uncaptured_error(Arc::new(|error: wgpu::Error| {
            log::error!("WebGPU uncaptured error: {error}");
        }));

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .first()
            .copied()
            .unwrap_or(wgpu::TextureFormat::Bgra8Unorm);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        // Create uniform buffers
        let make_uniform = |label: &str, size: usize| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: size as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };

        let iresolution = make_uniform("iResolution", 8);
        let itime = make_uniform("iTime", 4);
        let imouse = make_uniform("iMouse", 8);
        let ibpm = make_uniform("iBPM", 4);
        let icolors = make_uniform("iColors", 64);
        let imouseclick = make_uniform("iMouseClick", 16);

        let freqs = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("freqs"),
            size: (DEFAULT_FREQ_COUNT * std::mem::size_of::<f32>()) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Set initial values
        queue.write_buffer(
            &iresolution,
            0,
            bytemuck::cast_slice(&[width as f32, height as f32]),
        );
        // Match ~/.config/vibe/colors.toml defaults
        let default_colors: [[f32; 4]; 4] = [
            [0.9, 0.3, 0.9, 1.0],
            [0.1, 0.2, 0.0, 1.0],
            [0.1, 0.9, 0.2, 1.0],
            [0.2, 0.2, 0.9, 1.0],
        ];
        queue.write_buffer(&icolors, 0, bytemuck::cast_slice(&default_colors));

        let bind_group_layout = Self::make_bind_group_layout(&device);
        let bind_group = Self::make_bind_group(
            &device,
            &bind_group_layout,
            &iresolution,
            &freqs,
            &itime,
            &imouse,
            &ibpm,
            &icolors,
            &imouseclick,
        );

        let vertex_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Fullscreen vertex shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(VERTEX_SHADER)),
        });

        let pipeline = Self::make_pipeline(
            &device,
            &bind_group_layout,
            &vertex_module,
            FALLBACK_SHADER,
            format,
        );

        Ok(VibeApp {
            device,
            queue,
            surface,
            surface_config,
            format,
            pipeline,
            bind_group,
            bind_group_layout,
            vertex_module,
            iresolution,
            freqs,
            itime,
            imouse,
            ibpm,
            icolors,
            imouseclick,
            sensitivity: 3.0,
            start_time: now_secs(),
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let w = width.max(1);
        let h = height.max(1);
        self.surface_config.width = w;
        self.surface_config.height = h;
        self.surface.configure(&self.device, &self.surface_config);
        self.queue.write_buffer(
            &self.iresolution,
            0,
            bytemuck::cast_slice(&[w as f32, h as f32]),
        );
    }

    pub fn set_sensitivity(&mut self, val: f32) {
        self.sensitivity = val;
    }

    pub fn set_shader(&mut self, code: &str) {
        self.pipeline = Self::make_pipeline(
            &self.device,
            &self.bind_group_layout,
            &self.vertex_module,
            code,
            self.format,
        );
    }

    pub fn set_frequencies(&mut self, data: &[f32]) {
        if data.is_empty() {
            return;
        }
        let mut buf = [0.0f32; DEFAULT_FREQ_COUNT];
        let len = data.len().min(DEFAULT_FREQ_COUNT);
        for i in 0..len {
            buf[i] = data[i] * self.sensitivity;
        }
        self.queue
            .write_buffer(&self.freqs, 0, bytemuck::cast_slice(&buf));
    }

    pub fn set_mouse(&self, x: f32, y: f32) {
        self.queue
            .write_buffer(&self.imouse, 0, bytemuck::cast_slice(&[x, y]));
    }

    pub fn on_click(&self, x: f32, y: f32) {
        let time = self.elapsed_secs();
        self.queue.write_buffer(
            &self.imouseclick,
            0,
            bytemuck::cast_slice(&[x, y, time, 0.0f32]),
        );
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let time = self.elapsed_secs();
        self.queue
            .write_buffer(&self.itime, 0, bytemuck::bytes_of(&time));

        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| JsValue::from_str(&format!("{e}")))?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
