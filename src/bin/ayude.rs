use ayude::{
    camera::Camera,
    graphics::{self, Material, Mesh},
    import_gltf,
    transform::{Transform, GLOBAL_UP},
    Scene,
};
use glam::{Mat4, Vec2, Vec3};
use rusttype::{Font, Scale};
use wgpu::util::DeviceExt;
use std::{
    borrow::Cow,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct World {
    camera: Camera,

    movement: [f32; 2], // stores WASD input

    the_scene: Scene,
    the_sphere: Scene,

    ricardo: graphics::Texture,

    rendering_skin: bool,

    renderer: GraphicsContext,
}

impl World {
    fn new(renderer: GraphicsContext) -> Self {
        static VERTEX_SOURCE: &str = include_str!("../resources/vertex.glsl");
        static FRAGMENT_SOURCE: &str = include_str!("../resources/fragment.glsl");
        let shader = graphics::Shader::from_sources(VERTEX_SOURCE, FRAGMENT_SOURCE).unwrap();

        let gltf_file_name = "samples/knight/knight.gltf";
        // let gltf_file_name = "samples/principito_y_el_aviador/scene.gltf";
        let the_entity = import_gltf::import_default(gltf_file_name).unwrap();

        let the_sphere = import_gltf::import_default("samples/sphere.gltf").unwrap();

        // let ricardo = {
        //     let file = std::fs::read("samples/ricardo.jpg").unwrap();
        //     let image = image::load_from_memory(&file).unwrap();
        //     let image = image.into_rgba();
        //     graphics::Texture::builder(
        //         image.as_bytes(),
        //         image.width() as u16,
        //         image.height() as u16,
        //         graphics::texture::TextureFormat::RGBA,
        //     )
        //     .build()
        // };

        let camera = Camera::new(Vec3::from([0.0, 0.0, 37.0]), std::f32::consts::PI, 0.0);

        let test_font_texture = {
            let data = std::fs::read("data/Cousine.ttf").expect("font file should exist");
            let font = Font::try_from_vec(data).expect("font should load");

            let height: f32 = 12.4;
            let pixel_height = height.ceil() as usize;

            let scale = Scale {
                x: height * 2.0,
                y: height,
            };

            let v_metrics = font.v_metrics(scale);
            let offset = rusttype::point(0.0, v_metrics.ascent);

            let glyphs: Vec<_> = font.layout("RIGHT NOW.", scale, offset).collect();

            let width = glyphs
                .iter()
                .rev()
                .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
                .next()
                .unwrap_or(0.0)
                .ceil() as usize;

            let mut pixel_data = vec![0u8; width * pixel_height * 4];
            for g in glyphs {
                if let Some(bb) = g.pixel_bounding_box() {
                    g.draw(|x, y, v| {
                        let gray = (v * 255.5) as u8;
                        let x = x as i32 + bb.min.x;
                        let y = y as i32 + bb.min.y;
                        if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                            let i = (y as usize * width + x as usize) * 4;
                            pixel_data[i] = gray;
                            pixel_data[i + 1] = gray;
                            pixel_data[i + 2] = gray;
                            pixel_data[i + 3] = 255;
                        }
                    });
                }
            }

            graphics::Texture::builder(
                &pixel_data,
                width as u16,
                pixel_height as u16,
                graphics::texture::TextureFormat::RGBA,
            )
            .build()
        };

        let world = World {
            camera,

            movement: [0.0, 0.0],

            the_scene: the_entity,
            the_sphere,

            ricardo: test_font_texture,

            rendering_skin: false,

            renderer,
        };

        world
    }

    fn update(&mut self, delta: Duration) {
        let mov = Vec2::from(self.movement) * delta.as_secs_f32();
        self.camera.drive(mov);
    }

    fn render(&mut self, window_dimensions: (i32, i32)) {
        let frame = graphics::Frame::start([0.1, 0.1, 0.1], window_dimensions);

        let perspective = glam::Mat4::perspective_rh_gl(
            std::f32::consts::PI / 3.0,
            window_dimensions.0 as f32 / window_dimensions.1 as f32,
            0.1,
            1024.0,
        );

        let view = self.camera.view();

        {
            if !self.rendering_skin {
                self.renderer
                    .render_scene(&self.the_scene, &frame, &perspective, &view);
                let translation = Vec3::new(-1.0, -1.0, 0.0);
                self.renderer.render_billboard(
                    &self.ricardo,
                    &frame,
                    translation,
                    &perspective,
                    &self.camera,
                );
            } else {
                let scene = &self.the_scene;
                for node in &scene.nodes {
                    let skin = match node.skin.as_ref() {
                        Some(skin) => skin,
                        None => continue,
                    };

                    let skeleton_transform = match skin.skeleton {
                        Some(skeleton) => Transform::from(
                            scene.nodes[usize::from(skeleton)].transform.mat4().clone(),
                        ),
                        None => scene.transform.clone(),
                    };

                    for &joint in &skin.joints {
                        let joint = &scene.nodes[usize::from(joint)];

                        let mut transform = joint.transform.mat4().clone();
                        let mut current = joint;
                        'transform: loop {
                            match current.parent {
                                Some(index) => current = &scene.nodes[usize::from(index)],
                                None => break 'transform,
                            }
                            transform = transform.mul_mat4(current.transform.mat4());
                        }

                        self.the_sphere.transform = Transform::from(
                            transform.mul_mat4(skeleton_transform.mat4())
                                * Mat4::from_scale(Vec3::new(0.25, 0.25, 0.25)),
                        );

                        self.renderer
                            .render_scene(&self.the_sphere, &frame, &perspective, &view);
                    }
                }
            };
        }
    }
}

pub struct GraphicsContext {
    surface: wgpu::Surface,
    device: wgpu::Device,
    swap_chain: wgpu::SwapChain,
    swap_chain_descriptor: wgpu::SwapChainDescriptor,
}

impl GraphicsContext {
    pub async fn new(window: &Window) -> Self {
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

        todo!("create the vertex and index buffers");
        0;

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let texture_view =  todo!("create the texture");
        1;

        let mx_ref: &[f32; 16] = &[
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        ];
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
            label: None,
        });

        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shader/shader.wgsl"))),
            flags: wgpu::ShaderFlags::all(),
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.swap_chain_descriptor.width = width;
        self.swap_chain_descriptor.height = height;
        self.swap_chain = self
            .device
            .create_swap_chain(&self.surface, &self.swap_chain_descriptor);
    }

    fn render_scene(
        &mut self,
        scene: &Scene,
        frame: &graphics::Frame,
        perspective: &Mat4,
        view: &Mat4,
    ) {
        todo!()
        // let base_transform = &scene.transform;
        // for node in &scene.nodes {
        //     if node.meshes.is_empty() {
        //         continue;
        //     }

        //     let transform = {
        //         let mut current = node;
        //         let mut transform = node.transform.mat4().clone();
        //         'transform: loop {
        //             current = match current.parent {
        //                 Some(index) => &scene.nodes[usize::from(index)],
        //                 None => break 'transform,
        //             };

        //             transform = transform.mul_mat4(current.transform.mat4());
        //         }
        //         Transform::from(transform)
        //     };

        //     for mesh in &node.meshes {
        //         let material = &mesh.material;
        //         let diffuse = material.diffuse.as_ref();
        //         let normal = material.normal.as_ref();

        //         let base_transform = base_transform.mat4().clone();
        //         let mesh_transform = transform.mat4().clone();
        //         let model = (mesh_transform * base_transform).to_cols_array_2d();

        //         self.shader
        //             .uniform("perspective", perspective.to_cols_array_2d());
        //         self.shader.uniform("view", view.to_cols_array_2d());
        //         self.shader.uniform("model", model);
        //         self.shader.uniform(
        //             "diffuse_texture",
        //             diffuse.cloned().unwrap_or(graphics::Texture::empty()),
        //         );
        //         self.shader.uniform(
        //             "normal_texture",
        //             normal.cloned().unwrap_or(graphics::Texture::empty()),
        //         );
        //         self.shader
        //             .uniform("has_diffuse_texture", diffuse.is_some());
        //         self.shader.uniform("has_normal_texture", normal.is_some());
        //         self.shader
        //             .uniform("base_diffuse_color", material.base_diffuse_color);
        //         self.shader
        //             .uniform("u_light_direction", [-1.0, 0.4, 0.9f32]);
        //         self.shader.uniform("shaded", true);

        //         frame.render(mesh, &self.shader);
        //     }
        // }
    }

    fn render_billboard(
        &mut self,
        texture: &graphics::Texture,
        frame: &graphics::Frame,
        position: Vec3,
        perspective: &Mat4,
        camera: &Camera,
    ) {
        todo!()
        // let positions = [
        //     [-1.0, -1.0, 0.0],
        //     [1.0, -1.0, 0.0],
        //     [-1.0, 1.0, 0.0],
        //     [1.0, 1.0, 0.0],
        // ];
        // let normals = [
        //     [1.0, 0.0, 0.0],
        //     [1.0, 0.0, 0.0],
        //     [1.0, 0.0, 0.0],
        //     [1.0, 0.0, 0.0],
        // ];
        // let uvs = [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]];
        // let indices = [0, 1, 2, 3, 2, 1];
        // let material = Material {
        //     base_diffuse_color: [1.0, 1.0, 1.0, 1.0],
        //     diffuse: None,
        //     normal: None,
        // };
        // let mesh = Mesh::new(&positions, &normals, &uvs, &indices, &material);

        // let w = texture.width() as f32;
        // let h = texture.height() as f32;
        // let scale = Vec3::new(w / w.max(h) * 10.0, h / w.max(h) * 10.0, 1.0);
        // let rotation = {
        //     let fwd = camera.transform().position() - position;
        //     let fwd = -fwd.normalize().cross(GLOBAL_UP.into()).normalize();
        //     let yaw = f32::atan2(fwd.z, fwd.x);
        //     let pitch = f32::asin(fwd.y);
        //     Mat4::from_euler(glam::EulerRot::YXZ, -yaw, pitch, 0.0)
        // };
        // let model = Mat4::from_translation(position) * rotation * Mat4::from_scale(scale);

        // self.shader
        //     .uniform("perspective", perspective.to_cols_array_2d());
        // self.shader
        //     .uniform("view", camera.view().to_cols_array_2d());
        // self.shader.uniform("model", model.to_cols_array_2d());
        // self.shader.uniform("diffuse_texture", texture.clone());
        // self.shader.uniform("has_diffuse_texture", true);
        // self.shader.uniform("has_normal_texture", false);
        // self.shader.uniform("shaded", false);

        // frame.render(&mesh, &self.shader);
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("a.yude")
        .with_inner_size(LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)
        .expect("Failed to open window.");

    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);

    let window: Arc<Window> = window.into();

    set_panic_hook(window.clone());

    pollster::block_on(async_main(event_loop, window));
}

async fn async_main(event_loop: EventLoop<()>, window: Arc<Window>) {
    let renderer = GraphicsContext::new(&window).await;

    let mut game = World::new(renderer);

    let mut previous_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    game.renderer.resize(size.width, size.height);
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => return,
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    game.camera
                        .rotate(Vec2::new(delta.0 as f32, delta.1 as f32) / 150.0);
                }
                DeviceEvent::Key(input) => match input.virtual_keycode {
                    Some(VirtualKeyCode::W) => {
                        if input.state == ElementState::Pressed {
                            game.movement[1] = 1.0;
                        } else if input.state == ElementState::Released {
                            game.movement[1] = 0.0f32.min(game.movement[1]);
                        }
                    }
                    Some(VirtualKeyCode::A) => {
                        if input.state == ElementState::Pressed {
                            game.movement[0] = -1.0;
                        } else if input.state == ElementState::Released {
                            game.movement[0] = 0.0f32.max(game.movement[0]);
                        }
                    }
                    Some(VirtualKeyCode::S) => {
                        if input.state == ElementState::Pressed {
                            game.movement[1] = -1.0;
                        } else if input.state == ElementState::Released {
                            game.movement[1] = 0.0f32.max(game.movement[1]);
                        }
                    }
                    Some(VirtualKeyCode::D) => {
                        if input.state == ElementState::Pressed {
                            game.movement[0] = 1.0;
                        } else if input.state == ElementState::Released {
                            game.movement[0] = 0.0f32.min(game.movement[0]);
                        }
                    }
                    Some(VirtualKeyCode::Tab) if input.state == ElementState::Pressed => {
                        game.rendering_skin = !game.rendering_skin;
                    }
                    _ => return,
                },
                _ => return,
            },
            Event::MainEventsCleared => {
                let delta = previous_frame_time.elapsed();
                previous_frame_time = Instant::now();
                game.update(delta);
                window.request_redraw();
            }
            Event::RedrawRequested(..) => {
                game.render(get_window_dimensions(&window));
                todo!();
                // window.swap_buffers().unwrap();
            }
            _ => return,
        }
    });
}

fn get_window_dimensions(window: &Window) -> (i32, i32) {
    let inner_size = window.inner_size();
    (inner_size.width as i32, inner_size.height as i32)
}

fn set_panic_hook(window: Arc<Window>) {
    std::panic::set_hook(Box::new(move |panic_info| {
        window.set_cursor_grab(false).unwrap();
        window.set_cursor_visible(true);

        let mut lines = vec![];
        if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            lines.push(message.to_string());
        }
        if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            lines.push(message.to_string());
        }
        if let Some(location) = panic_info.location() {
            let loc = format!(
                "[{},{}] {}",
                location.line(),
                location.column(),
                location.file()
            );
            lines.push(loc);
        }

        msgbox::create("Error", &lines.join("\n"), msgbox::IconType::Error).unwrap_or_else(|_| {
            println!("{}", lines.join("\n"));
        })
    }));
}
