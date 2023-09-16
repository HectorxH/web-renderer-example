use crate::{buffers, texture};
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::window::Window;

const N_BUFFERS: usize = 2;

const VERTICES: [&[buffers::Vertex]; N_BUFFERS] = [
    &[
        buffers::Vertex {
            position: [-0.0868241, 0.49240386, 0.0],
            tex_coords: [0.4131759, 0.00759614],
        },
        buffers::Vertex {
            position: [-0.49513406, 0.06958647, 0.0],
            tex_coords: [0.0048659444, 0.43041354],
        },
        buffers::Vertex {
            position: [-0.21918549, -0.44939706, 0.0],
            tex_coords: [0.28081453, 0.949397],
        },
        buffers::Vertex {
            position: [0.35966998, -0.3473291, 0.0],
            tex_coords: [0.85967, 0.84732914],
        },
        buffers::Vertex {
            position: [0.44147372, 0.2347359, 0.0],
            tex_coords: [0.9414737, 0.2652641],
        },
    ],
    &[
        buffers::Vertex {
            position: [-0.9, 0.1, 0.25],
            tex_coords: [1.0, 0.0],
        },
        buffers::Vertex {
            position: [0.1, 0.1, 0.25],
            tex_coords: [1.0, 0.0],
        },
        buffers::Vertex {
            position: [0.1, -0.9, 0.25],
            tex_coords: [1.0, 0.0],
        },
        buffers::Vertex {
            position: [-0.9, -0.9, 0.25],
            tex_coords: [1.0, 0.0],
        },
        buffers::Vertex {
            position: [-0.5, 0.5, 0.75],
            tex_coords: [0.0, 1.0],
        },
        buffers::Vertex {
            position: [0.5, 0.5, 0.75],
            tex_coords: [0.0, 1.0],
        },
        buffers::Vertex {
            position: [0.5, -0.5, 0.75],
            tex_coords: [0.0, 1.0],
        },
        buffers::Vertex {
            position: [-0.5, -0.5, 0.75],
            tex_coords: [0.0, 1.0],
        },
    ],
];

const INDICES: [&[u16]; N_BUFFERS] = [
    &[0, 1, 4, 1, 2, 4, 2, 3, 4],
    &[
        // Back
        0, 1, 2, // T1
        0, 2, 3, // T2
        // Left
        0, 3, 4, // T1
        4, 3, 7, // T2
        // Right
        1, 5, 2, // T1
        2, 5, 6, // T2
        // Top
        0, 4, 5, // T1
        0, 5, 1, // T2
        // Bottom
        3, 2, 6, // T1
        3, 6, 7, // T2
        // Front
        4, 7, 6, // T1
        4, 6, 5, // T2
    ],
];

const N_PIPELINES: usize = 2;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipelines: [wgpu::RenderPipeline; N_PIPELINES],
    current_pipeline: usize,
    vertex_buffers: [wgpu::Buffer; N_BUFFERS],
    index_buffers: [wgpu::Buffer; N_BUFFERS],
    current_buffer: usize,
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
}

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Surface
        // ## Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        // Adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // # Device and Queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        // # Surface Configuration
        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("../../public/assets/happy-tree.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let diffuse_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &diffuse_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        // # Render Pipeline
        let shaders = [
            device.create_shader_module(wgpu::include_wgsl!("shaders/shader1.wgsl")),
            device.create_shader_module(wgpu::include_wgsl!("shaders/shader2.wgsl")),
        ];

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&diffuse_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipelines = core::array::from_fn(|i| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(format!("Render Pipeline {}", i + 1).as_str()),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shaders[i],
                    entry_point: "vs_main",
                    buffers: &[buffers::Vertex::description()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shaders[i],
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    // not 0 => 0xffffffff all bits set to 1
                    // this enables all samples
                    mask: !0,
                    // anti-aliasing related
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            })
        });

        let current_pipeline = 0;

        let vertex_buffers = core::array::from_fn(|i| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("Vertex Buffer {}", i + 1).as_str()),
                contents: bytemuck::cast_slice(VERTICES[i]),
                usage: wgpu::BufferUsages::VERTEX,
            })
        });

        let index_buffers = core::array::from_fn(|i| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("Index Buffer {}", i + 1).as_str()),
                contents: bytemuck::cast_slice(INDICES[i]),
                usage: wgpu::BufferUsages::INDEX,
            })
        });

        let current_buffer = 0;

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipelines,
            current_pipeline,
            vertex_buffers,
            index_buffers,
            current_buffer,
            diffuse_bind_group,
            diffuse_texture,
        }
    }

    pub fn next_pipeline(&mut self) {
        self.current_pipeline = (self.current_pipeline + 1) % N_PIPELINES;
    }

    pub fn next_buffer(&mut self) {
        self.current_buffer = (self.current_buffer + 1) % N_BUFFERS;
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Start render pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.render_pipelines[self.current_pipeline]);
        render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffers[self.current_buffer].slice(..));
        render_pass.set_index_buffer(
            self.index_buffers[self.current_buffer].slice(..),
            wgpu::IndexFormat::Uint16,
        );
        let n_indices = INDICES[self.current_buffer].len() as u32;
        render_pass.draw_indexed(0..n_indices, 0, 0..1);

        drop(render_pass);
        // End render pass, releases `encoder`

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
