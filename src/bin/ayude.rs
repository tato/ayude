use ayude::*;
use glam::{Mat4, Quat, Vec3};
use glutin::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    Api, ContextBuilder, GlProfile, GlRequest, Robustness,
};
use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};

fn calculate_forward_direction(yaw: f32, pitch: f32) -> Vec3 {
    let result: Vec3 = [
        (-yaw).cos() * pitch.cos(),
        (-yaw).sin() * pitch.cos(),
        pitch.sin(),
    ]
    .into();
    result.normalize()
}

struct Entity {
    children: Vec<catalog::Id<Entity>>,
    parent: catalog::Id<Entity>,
    mesh: Option<catalog::Id<graphics::Mesh>>,
    transform: [[f32; 4]; 4],
}

pub struct World {
    camera_position: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,

    movement: [f32; 2], // stores WASD input

    shader: graphics::Shader,

    meshes: Catalog<graphics::Mesh>,
    materials: Catalog<graphics::Material>,
    textures: Catalog<graphics::Texture>,
    entities: Catalog<Entity>,
}

impl World {
    fn new() -> Self {
        static VERTEX_SOURCE: &str = include_str!("../resources/vertex.glsl");
        static FRAGMENT_SOURCE: &str = include_str!("../resources/fragment.glsl");
        let shader = graphics::Shader::from_sources(VERTEX_SOURCE, FRAGMENT_SOURCE).unwrap();

        let mut world = World {
            camera_position: [0.0, 0.0, 0.0].into(),
            camera_yaw: 0.0,
            camera_pitch: 0.0,

            movement: [0.0, 0.0],

            shader,

            meshes: Catalog::new(),
            materials: Catalog::new(),
            textures: Catalog::new(),
            entities: Catalog::new(),
        };

        let gltf_file_name = "samples/physicstest.gltf";
        let gltf = gltf::load(gltf_file_name).unwrap();
        world.add_gltf_entities(&gltf);

        world
    }

    fn add_gltf_entities(&mut self, gltf: &gltf::GLTF) {
        let doc = &gltf.document;
        let scene = &doc.scenes[doc.scene.unwrap_or(0)];
        let buffers = gltf.load_buffers().expect("XD");

        let add_texture = |world: &mut World, index: usize| {
            let image = &doc.images[index];
            if let Some(uri) = &image.uri {
                let loaded = gltf.load_image(uri).expect("XD");
                let width = loaded.width();
                let height = loaded.height();
                let bytes = image::DynamicImage::ImageRgba8(loaded).to_bytes();
                world.textures.add(graphics::Texture::from_rgba(
                    &bytes,
                    width as i32,
                    height as i32,
                ))
            } else {
                unimplemented!("Only relative uri image loading is implemented")
            }
        };

        let add_material = |world: &mut World, index: usize| {
            let material = &doc.materials[index];
            let normal = material
                .normal_texture
                .as_ref()
                .map(|info| add_texture(world, info.index));
            let diffuse = material
                .pbr_metallic_roughness
                .base_color_texture
                .as_ref()
                .map(|info| add_texture(world, info.index));
            let base_diffuse_color = material.pbr_metallic_roughness.base_color_factor;
            world.materials.add(graphics::Material {
                normal,
                diffuse,
                base_diffuse_color,
            })
        };

        let add_mesh = |world: &mut World, index: usize| {
            macro_rules! accessor_get {
                ($index:expr, $component:expr, $_type:expr, $element_size:expr) => {{
                    let accessor = &doc.accessors[$index];
                    debug_assert!(accessor.component_type == $component);
                    debug_assert!(accessor._type == $_type);
                    let view =
                        &doc.buffer_views[accessor.buffer_view.expect("I NEED THIS TO BE HERE")];
                    let buffer = &buffers[view.buffer]
                        [view.byte_offset..(view.byte_offset + view.byte_length)];
                    unsafe {
                        let ptr = std::mem::transmute(buffer.as_ptr());
                        std::slice::from_raw_parts(ptr, buffer.len() / $element_size)
                    }
                }};
            }
            let mesh = &doc.meshes[index];
            let primitives = mesh
                .primitives
                .iter()
                .map(|primitive| {
                    let positions: &[[f32; 3]] =
                        accessor_get!(primitive.attributes["POSITION"], 5126, "VEC3", 12);
                    let normals: &[[f32; 3]] =
                        accessor_get!(primitive.attributes["NORMAL"], 5126, "VEC3", 12);
                    let uvs: &[[f32; 2]] =
                        accessor_get!(primitive.attributes["TEXCOORD_0"], 5126, "VEC2", 8);
                    let indices: &[u16] = accessor_get!(primitive.indices, 5123, "SCALAR", 2);
                    let material = add_material(world, primitive.material);
                    graphics::Primitive::new(positions, normals, uvs, indices, material)
                })
                .collect();
            world.meshes.add(graphics::Mesh { primitives })
        };

        self.entities = scene
            .nodes
            .iter()
            .map(|&i| &doc.nodes[i])
            .map(|node| Entity {
                children: node.children.iter().map(|&i| i.into()).collect(),
                parent: catalog::Id::none(),
                mesh: node.mesh.map(|i| add_mesh(self, i)),
                transform: {
                    let t = if let Some(m) = node.matrix {
                        Mat4::from_cols_array(&m)
                    } else {
                        let t: Vec3 = node.translation.unwrap_or([0.0, 0.0, 0.0]).into();
                        let r: Quat = node.rotation.unwrap_or([0.0, 0.0, 0.0, 1.0]).into();
                        let s: Vec3 = node.scale.unwrap_or([1.0, 1.0, 1.0]).into();
                        Mat4::from_translation(t) * Mat4::from_quat(r) * Mat4::from_scale(s)
                    };
                    t.to_cols_array_2d()
                },
            })
            .collect();
        let entity_ids = self
            .entities
            .iter_ids()
            .cloned()
            .collect::<Vec<_>>();
        for id in entity_ids {
            let children_ids = self
                .entities
                .get(id)
                .map(|it| it.children.clone())
                .unwrap_or_else(|| Vec::new());
            for child in children_ids {
                self.entities.get_mut(child).map(|c| c.parent = id);
            }
        }
    }

    fn update(&mut self, delta: Duration) {
        let forward_direction = calculate_forward_direction(self.camera_yaw, self.camera_pitch);
        let right_direction = forward_direction.cross([0.0, 0.0, 1.0].into()).normalize();

        let speed = 100.0;
        self.camera_position += forward_direction * self.movement[1] * speed * delta.as_secs_f32();
        self.camera_position += right_direction * self.movement[0] * speed * delta.as_secs_f32();
    }

    fn render(&mut self, window_dimensions: (i32, i32)) {
        let forward_direction = calculate_forward_direction(self.camera_yaw, self.camera_pitch);
        let frame = graphics::Frame::start([0.0, 0.0, 1.0], window_dimensions);

        let perspective = glam::Mat4::perspective_rh_gl(
            std::f32::consts::PI / 3.0,
            window_dimensions.0 as f32 / window_dimensions.1 as f32,
            0.1,
            1024.0,
        );

        let view = glam::Mat4::look_at_rh(
            self.camera_position,
            self.camera_position + forward_direction,
            [0.0, 0.0, 1.0].into(),
        );

        for entity in self.entities.iter() {
            let model = entity.transform;
            if let Some(mesh) = self.meshes.get_opt(entity.mesh) {
                for primitive in &mesh.primitives {
                    let material = self.materials.get(primitive.material).expect("XD");
                    let diffuse = self.textures.get_opt(material.diffuse);
                    let normal = self.textures.get_opt(material.normal);

                    self.shader
                        .uniform("perspective", perspective.to_cols_array_2d());
                    self.shader.uniform("view", view.to_cols_array_2d());
                    self.shader.uniform("model", model);
                    self.shader.uniform(
                        "diffuse_texture",
                        diffuse.cloned().unwrap_or(graphics::Texture::empty()),
                    );
                    self.shader.uniform(
                        "normal_texture",
                        normal.cloned().unwrap_or(graphics::Texture::empty()),
                    );
                    self.shader
                        .uniform("has_diffuse_texture", diffuse.is_some());
                    self.shader.uniform("has_normal_texture", normal.is_some());
                    self.shader
                        .uniform("base_diffuse_color", material.base_diffuse_color);
                    self.shader
                        .uniform("u_light_direction", [-1.0, 0.4, 0.9f32]);

                    frame.render(primitive, &self.shader);
                }
            }
        }
    }
}

fn main() {
    std::panic::set_hook(Box::new(|panic_info| {
        // todo!
        // window.window().set_cursor_grab(false).unwrap();
        // window.window().set_cursor_visible(true);
        
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
        msgbox::create("Error", &lines.join("\n"), msgbox::IconType::Error);
    }));

    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("a.yude")
        .with_inner_size(LogicalSize::new(1024.0, 768.0));
    let window = ContextBuilder::new()
        .with_gl(GlRequest::Specific(Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core)
        .with_gl_debug_flag(true)
        .with_gl_robustness(Robustness::RobustLoseContextOnReset)
        .with_vsync(true)
        .build_windowed(window_builder, &event_loop)
        .unwrap();
    let window = unsafe { window.make_current().unwrap() };

    window.window().set_cursor_grab(true).unwrap();
    window.window().set_cursor_visible(false);

    gl::load_with(|s| window.context().get_proc_address(s));

    let mut game = World::new();

    let mut previous_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => return,
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    game.camera_yaw += delta.0 as f32 * 0.006;
                    if game.camera_yaw >= 2.0 * PI {
                        game.camera_yaw -= 2.0 * PI;
                    }
                    if game.camera_yaw <= 0.0 {
                        game.camera_yaw += 2.0 * PI;
                    }

                    let freedom_y = 0.8;
                    game.camera_pitch -= delta.1 as f32 * 0.006;
                    game.camera_pitch = game
                        .camera_pitch
                        .max(-PI / 2.0 * freedom_y)
                        .min(PI / 2.0 * freedom_y);
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
                    _ => return,
                },
                _ => return,
            },
            Event::MainEventsCleared => {
                let delta = previous_frame_time.elapsed();
                previous_frame_time = Instant::now();
                game.update(delta);
                window.window().request_redraw();
            }
            Event::RedrawRequested(..) => {
                let inner_size = window.window().inner_size();
                game.render((inner_size.width as i32, inner_size.height as i32));
                window.swap_buffers().unwrap();
            }
            _ => return,
        }
    });
}
