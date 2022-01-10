use std::{borrow::Cow, rc::Rc};

use glam::{Mat4, Vec3};

use once_cell::sync::OnceCell;
use wgpu::util::DeviceExt;

use bytemuck::{Pod, Zeroable};

use crate::transform::GLOBAL_UP;

#[derive(Debug, Clone)]
pub struct Material {
    pub normal: Option<Texture>,
    pub diffuse: Option<Texture>,
    pub base_diffuse_color: [f32; 4],
    pub shaded: bool,
}

pub struct GraphicsContext {
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
    device: wgpu::Device, // todo! not pub
    queue: wgpu::Queue,   // todo! not pub
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    textures_bind_group_layout: wgpu::BindGroupLayout,
    default_texture: OnceCell<Texture>,
    quad_mesh: OnceCell<Mesh>,
    depth_view: wgpu::TextureView, // todo! not pub
}

impl GraphicsContext {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to find an appropriate graphics adapter.");

        let adapter_info = adapter.get_info();
        println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to acquire GPU device.");

        let surface_format = surface
            .get_preferred_format(&adapter)
            .expect("Surface is not compatible with graphics adapter.");

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &surface_config);

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let textures_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &uniform_bind_group_layout,
                &textures_bind_group_layout,
                &textures_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader/shader.wgsl"))),
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 4 * 4,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 7,
                    shader_location: 2,
                },
            ],
        }];

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[surface_format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Self::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let depth_texture = Self::create_depth_texture(&&surface_config, &device);

        Self {
            surface,
            surface_config,
            device,
            queue,
            pipeline: render_pipeline,
            uniform_bind_group_layout,
            textures_bind_group_layout,
            default_texture: OnceCell::new(),
            quad_mesh: OnceCell::new(),
            depth_view: depth_texture,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface_config.width = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);

        self.depth_view = Self::create_depth_texture(&self.surface_config, &self.device);
    }

    fn create_depth_texture(
        sc_desc: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
    ) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn create_mesh(&self, vertices: &[Vertex], indices: &[u16]) -> Mesh {
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        Mesh {
            inner: (vertex_buffer, index_buffer).into(),
            index_count: indices.len(),
        }
    }

    pub fn create_uniform_buffer(&self) -> UniformBuffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: (std::mem::size_of::<Uniforms>() + 4) as _,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        UniformBuffer { buffer, bind_group }
    }

    pub fn create_texture(&self, desc: &TextureDescription) -> Texture {
        let texture_extent = wgpu::Extent3d {
            width: desc.width,
            height: desc.height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: desc.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::default(),
            },
            desc.texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(std::num::NonZeroU32::new(desc.width * 4).unwrap()),
                rows_per_image: None,
            },
            texture_extent,
        );

        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: desc.wrap_s,
            address_mode_v: desc.wrap_t,
            mag_filter: desc.mag_filter,
            min_filter: desc.min_filter,
            ..Default::default()
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.textures_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Texture {
            bind_group: bind_group.into(),
            width: desc.width,
            height: desc.height,
        }
    }

    pub fn get_current_frame<'gfx>(&'gfx mut self) -> Frame<'gfx> {
        let current_texture = match self.surface.get_current_texture() {
            Ok(texture) => texture,
            Err(_) => {
                self.surface.configure(&self.device, &self.surface_config);
                self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture!")
            }
        };

        let encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let current_texture_view = current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        Frame {
            graphics: self,
            _current_texture: current_texture,
            current_texture_view,
            encoder,
        }
    }

    fn get_quad_mesh(&self) -> &Mesh {
        self.quad_mesh.get_or_init(|| {
            macro_rules! v {
                ($pos:expr, $norm:expr, $uv:expr) => {
                    Vertex {
                        position: $pos,
                        normal: $norm,
                        tex_coord: $uv,
                    }
                };
            }
            let vertices = [
                v!([-1.0, -1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0]),
                v!([1.0, -1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 1.0]),
                v!([-1.0, 1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 0.0]),
                v!([1.0, 1.0, 0.0, 1.0], [1.0, 0.0, 0.0], [1.0, 0.0]),
            ];
            let indices = [0, 1, 2, 3, 2, 1];
            let mesh = self.create_mesh(&vertices, &indices);
            mesh
        })
    }

    fn get_default_texture(&self) -> &Texture {
        self.default_texture.get_or_init(|| {
            let pixels = [
                255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255u8,
            ];
            self.create_texture(&TextureDescription::new(
                &pixels,
                2,
                2,
                wgpu::TextureFormat::Rgba8Unorm,
            ))
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 4],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct Mesh {
    /// vertex_buffer, index_buffer
    inner: Rc<(wgpu::Buffer, wgpu::Buffer)>,
    pub index_count: usize,
}

impl Mesh {
    pub fn vertex(&self) -> &wgpu::Buffer {
        let (vertex, _) = self.inner.as_ref();
        vertex
    }
    pub fn index(&self) -> &wgpu::Buffer {
        let (_, index) = self.inner.as_ref();
        index
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    bind_group: Rc<wgpu::BindGroup>,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct TextureDescription<'a> {
    texels: &'a [u8],
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    wrap_s: wgpu::AddressMode,
    wrap_t: wgpu::AddressMode,
    min_filter: wgpu::FilterMode,
    mag_filter: wgpu::FilterMode,
}

impl<'a> TextureDescription<'a> {
    pub fn new(texels: &'a [u8], width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        Self {
            texels,
            width,
            height,
            format,
            wrap_s: wgpu::AddressMode::ClampToEdge,
            wrap_t: wgpu::AddressMode::ClampToEdge,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
        }
    }
    pub fn wrap_s(mut self, mode: wgpu::AddressMode) -> Self {
        self.wrap_s = mode;
        self
    }
    pub fn wrap_t(mut self, mode: wgpu::AddressMode) -> Self {
        self.wrap_t = mode;
        self
    }
    pub fn min_filter(mut self, mode: wgpu::FilterMode) -> Self {
        self.min_filter = mode;
        self
    }
    pub fn mag_filter(mut self, mode: wgpu::FilterMode) -> Self {
        self.mag_filter = mode;
        self
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
struct Uniforms {
    mvp: [f32; 16],
    transpose_inverse_modelview: [f32; 16],
    light_direction: [f32; 4],
    base_diffuse_color: [f32; 4],
    has_diffuse_texture: u32,
    has_normal_texture: u32,
    shaded: u32,
}

#[derive(Debug)]
pub struct UniformBuffer {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

pub struct Frame<'gfx> {
    graphics: &'gfx GraphicsContext,
    _current_texture: wgpu::SurfaceTexture,
    current_texture_view: wgpu::TextureView,
    encoder: wgpu::CommandEncoder,
}

impl<'gfx> Frame<'gfx> {
    pub fn begin_render_pass<'frame>(&'frame mut self) -> Pass<'gfx, 'frame> {
        let pass = self.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &self.current_texture_view,
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
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.graphics.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: false,
                }),
                stencil_ops: None,
            }),
        });

        Pass {
            graphics: self.graphics,
            pass,
        }
    }

    pub fn submit(self) {
        self.graphics.queue.submit(Some(self.encoder.finish()));
    }
}

pub struct Pass<'gfx, 'frame> {
    graphics: &'gfx GraphicsContext,
    pass: wgpu::RenderPass<'frame>,
}

impl<'gfx: 'frame, 'frame> Pass<'gfx, 'frame> {
    pub fn render_mesh(
        &mut self,
        mesh: &'frame Mesh,
        uniform_buffer: &'frame UniformBuffer,
        material: &'frame Material,
        perspective: Mat4,
        view: Mat4,
        model: Mat4,
    ) {
        let diffuse = material.diffuse.as_ref();
        let normal = material.normal.as_ref();

        let uniforms = Uniforms {
            mvp: (perspective * view * model).to_cols_array(),
            transpose_inverse_modelview: (view * model).inverse().transpose().to_cols_array(),
            light_direction: [-1.0, 0.4, 0.9f32, 0.0],
            base_diffuse_color: material.base_diffuse_color,
            has_diffuse_texture: if diffuse.is_some() { 1 } else { 0 },
            has_normal_texture: if normal.is_some() { 1 } else { 0 },
            shaded: if material.shaded { 1 } else { 0 },
        };
        self.graphics.queue.write_buffer(
            &uniform_buffer.buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        let diffuse = diffuse.unwrap_or_else(|| self.graphics.get_default_texture());
        let normal = normal.unwrap_or_else(|| self.graphics.get_default_texture());

        self.pass.set_pipeline(&self.graphics.pipeline);
        self.pass.set_bind_group(0, &uniform_buffer.bind_group, &[]);
        self.pass.set_bind_group(1, diffuse.bind_group(), &[]);
        self.pass.set_bind_group(2, normal.bind_group(), &[]);
        self.pass
            .set_index_buffer(mesh.index().slice(..), wgpu::IndexFormat::Uint16);
        self.pass.set_vertex_buffer(0, mesh.vertex().slice(..));
        self.pass.draw_indexed(0..mesh.index_count as u32, 0, 0..1);
    }

    pub fn render_billboard(
        &mut self,
        uniform_buffer: &'frame UniformBuffer,
        material: &'frame Material,
        perspective: Mat4,
        view: Mat4,
        position: Vec3,
        camera_position: Vec3,
    ) {
        let mesh = self.graphics.get_quad_mesh();
        let texture = material.diffuse.as_ref().unwrap();

        let w = texture.width as f32;
        let h = texture.height as f32;
        let s = 1.2;
        let scale = Vec3::new(w / w.max(h) * s, h / w.max(h) * s, 1.0);
        let rotation = {
            let fwd = camera_position - position;
            let fwd = -fwd.normalize().cross(GLOBAL_UP.into()).normalize();
            let yaw = f32::atan2(fwd.z, fwd.x);
            let pitch = f32::asin(fwd.y);
            Mat4::from_euler(glam::EulerRot::YXZ, -yaw, pitch, 0.0)
        };
        let model = Mat4::from_translation(position) * rotation * Mat4::from_scale(scale);

        self.render_mesh(&mesh, uniform_buffer, material, perspective, view, model);
    }
}
