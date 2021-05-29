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
}

pub struct GraphicsContext {
    surface: wgpu::Surface,
    pub device: wgpu::Device, // todo! not pub
    swap_chain: wgpu::SwapChain,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
    pub queue: wgpu::Queue, // todo! not pub
    pipeline: wgpu::RenderPipeline,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    textures_bind_group_layout: wgpu::BindGroupLayout,
    default_texture: OnceCell<Texture>,
    quad_mesh: OnceCell<Mesh>,
}

impl GraphicsContext {
    pub async fn new(window: &winit::window::Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let surface = unsafe { instance.create_surface(window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
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

        let swapchain_format = adapter
            .get_swap_chain_preferred_format(&surface)
            .expect("Surface is not compatible with graphics adapter.");

        let swap_chain_descriptor = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let swap_chain = device.create_swap_chain(&surface, &swap_chain_descriptor);

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
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
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                }],
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
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shader/shader.wgsl"))),
            flags: wgpu::ShaderFlags::all(),
        });

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
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
                targets: &[swapchain_format.into()],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
        });

        Self {
            surface,
            device,
            swap_chain,
            swap_chain_descriptor,
            queue,
            pipeline: render_pipeline,
            uniform_bind_group_layout,
            textures_bind_group_layout,
            default_texture: OnceCell::new(),
            quad_mesh: OnceCell::new(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_descriptor.width = width;
        self.swap_chain_descriptor.height = height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    pub fn render_mesh<'gfx, 'mesh, 'pass>(
        &'gfx self,
        mesh: &'mesh Mesh,
        perspective: Mat4,
        view: Mat4,
        model: Mat4,
        pass: &mut wgpu::RenderPass<'pass>,
    ) where
        'mesh: 'pass,
        'gfx: 'pass,
    {
        let material = &mesh.material;
        let diffuse = material.diffuse.as_ref();
        let normal = material.normal.as_ref();

        let uniforms = Uniforms {
            mvp: (perspective * view * model).to_cols_array(),
            transpose_inverse_modelview: (view * model).inverse().transpose().to_cols_array(),
            light_direction: [-1.0, 0.4, 0.9f32],
            has_diffuse_texture: if diffuse.is_some() { 1 } else { 0 },
            has_normal_texture: if normal.is_some() { 1 } else { 0 },
            base_diffuse_color: material.base_diffuse_color,
            shaded: 1,
        };
        self.queue
            .write_buffer(&mesh.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        let diffuse = diffuse.unwrap_or_else(|| self.get_default_texture());
        let normal = normal.unwrap_or_else(|| self.get_default_texture());

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &mesh.bind_group, &[]);
        pass.set_bind_group(1, &diffuse.bind_group, &[]);
        pass.set_bind_group(2, &normal.bind_group, &[]);
        pass.set_index_buffer(mesh.inner.index.slice(..), wgpu::IndexFormat::Uint16);
        pass.set_vertex_buffer(0, mesh.inner.vertex.slice(..));
        pass.draw_indexed(0..mesh.index_count as u32, 0, 0..1);
    }

    pub fn render_billboard<'gfx, 'pass>(
        &'gfx self,
        texture: &Texture,
        pass: &mut wgpu::RenderPass<'pass>,
        position: Vec3,
        perspective: Mat4,
        camera: &crate::camera::Camera,
    ) where
        'gfx: 'pass,
    {
        let mesh = self.get_quad_mesh();

        let w = texture.width as f32;
        let h = texture.height as f32;
        let scale = Vec3::new(w / w.max(h) * 10.0, h / w.max(h) * 10.0, 1.0);
        let rotation = {
            let fwd = camera.transform().position() - position;
            let fwd = -fwd.normalize().cross(GLOBAL_UP.into()).normalize();
            let yaw = f32::atan2(fwd.z, fwd.x);
            let pitch = f32::asin(fwd.y);
            Mat4::from_euler(glam::EulerRot::YXZ, -yaw, pitch, 0.0)
        };
        let model = Mat4::from_translation(position) * rotation * Mat4::from_scale(scale);

        self.render_mesh(mesh, perspective, camera.view(), model, pass);
    }

    pub fn create_mesh(&self, vertices: &[Vertex], indices: &[u16], material: &Material) -> Mesh {
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsage::INDEX,
            });

        let inner = MeshStorage {
            vertex: vertex_buffer,
            index: index_buffer,
        };
        
        let uniform_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<Uniforms>() as _,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let uniform_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Mesh {
            inner: inner.into(),
            index_count: indices.len(),
            material: material.clone(),
            uniform_buffer: uniform_buffer.into(),
            bind_group: uniform_bind_group.into()
        }
    }

    pub fn create_texture(
        &self,
        texels: &[u8],
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Texture {
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(std::num::NonZeroU32::new(width * 4).unwrap()),
                rows_per_image: None,
            },
            texture_extent,
        );

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.textures_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });

        Texture {
            texture: texture.into(),
            width,
            height,
            bind_group: bind_group.into(),
        }
    }

    pub fn get_current_frame(&mut self) -> wgpu::SwapChainFrame {
        let frame = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(_) => {
                self.swap_chain = self
                    .device
                    .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
                self.swap_chain
                    .get_current_frame()
                    .expect("Failed to acquire next swap chain texture!")
            }
        };
        frame
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
            let material = Material {
                base_diffuse_color: [1.0, 1.0, 1.0, 1.0],
                diffuse: None,
                normal: None,
            };
            let mesh = self.create_mesh(&vertices, &indices, &material);
            mesh
        })
    }

    fn get_default_texture(&self) -> &Texture {
        self.default_texture.get_or_init(|| {
            let pixels = [
                255, 0, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 0, 255, 255u8,
            ];
            self.create_texture(&pixels, 2, 2, wgpu::TextureFormat::Rgba8Uint)
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

#[derive(Debug)]
pub struct MeshStorage {
    pub vertex: wgpu::Buffer,
    pub index: wgpu::Buffer,
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub inner: Rc<MeshStorage>,
    pub index_count: usize,
    pub material: Material,
    bind_group: Rc<wgpu::BindGroup>,
    uniform_buffer: Rc<wgpu::Buffer>,
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub texture: Rc<wgpu::Texture>,
    pub width: u32,
    pub height: u32,
    pub bind_group: Rc<wgpu::BindGroup>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
struct Uniforms {
    mvp: [f32; 16],
    transpose_inverse_modelview: [f32; 16],
    light_direction: [f32; 3],
    has_diffuse_texture: u32,
    has_normal_texture: u32,
    base_diffuse_color: [f32; 4],
    shaded: u32,
}
