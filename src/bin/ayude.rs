use ayude::*;
use glam::{Mat4, Vec3};
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


// pub struct RenderObject {
//     pub geometry_id: catalog::Id<graphics::Geometry>,
//     pub diffuse: Option<graphics::Texture>,
//     pub normal: Option<graphics::Texture>,
//     pub base_diffuse_color: [f32; 4],
// }
// pub struct StaticEntity {
//     pub transform: [[f32; 4]; 4],
//     pub render_object: RenderObject,
// }

// pub struct World {
//     pub statics: Vec<StaticEntity>,
// }

// impl World {
//     fn upload(scene: gltf::GLTF) -> Result<Self, AyudeError> {
//         let mut nodes = Vec::new();
//         let mut geometries = Catalog::new();

//         let textures = scene.images.iter().map(|image| {
//             graphics::Texture::from_rgba(&scene.images_byte_buffer[image.offset..image.offset+image.size], image.width as i32, image.height as i32)
//         }).collect::<Vec<_>>();
//         for unode in scene.nodes {
//             let transform = unode.transform;
//             let base_diffuse_color = unode.base_diffuse_color;

//             let umetry = &scene.geometries[unode.geometry_index];
//             let geometry = graphics::Geometry::new(
//                 &umetry.positions,
//                 &umetry.normals,
//                 &umetry.uvs,
//                 &umetry.indices
//             );
//             let geometry_id = geometries.add(geometry);

//             let diffuse = unode.diffuse.map(|index| {
//                 textures[index].clone()
//             });
//             let normal =  unode.normal.map(|index| {
//                 textures[index].clone()
//             });

//             let render_object = RenderObject{ geometry_id, diffuse, normal, base_diffuse_color };
//             nodes.push(StaticEntity{ transform, render_object });
//         }
//         Ok(crate::World{ statics: nodes, geometries })
//     }
// }

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
    mesh: Option<catalog::Id<graphics::Mesh>>,
    transform: [[f32; 4]; 4],
}
struct Material {
}

pub struct World {
    camera_position: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,

    movement: [f32; 2], // stores WASD input

    shader: graphics::Shader,

    meshes: Catalog<graphics::Mesh>,
    materials: Catalog<Material>,
    textures: Catalog<graphics::Texture>,
    entities: Catalog<Entity>,

    physics: physics::PhysicsState,
}

impl World {
    fn new() -> Self {
        static VERTEX_SOURCE: &str = include_str!("../resources/vertex.glsl");
        static FRAGMENT_SOURCE: &str = include_str!("../resources/fragment.glsl");
        let shader = graphics::Shader::from_sources(VERTEX_SOURCE, FRAGMENT_SOURCE).unwrap();

        let physics = physics::PhysicsState::new();

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

            physics,
        };

        // let gltf_file_name = "samples/glTF-Sample-Models/2.0/Sponza/glTF/Sponza.gltf";
        // let gltf_file_name = "samples/homework_09_simple_textures/scene.gltf";
        let gltf_file_name = "samples/physicstest.gltf";
        let gltf = gltf::load(gltf_file_name).unwrap();
        world.add_gltf_entities(&gltf);

        world
    }

    fn add_gltf_entities(&mut self, gltf: &gltf::GLTF) {
        let doc = &gltf.document;
        let scene = &doc.scenes[doc.scene.unwrap_or(0)];
        self.entities = scene.nodes.iter()
            .map(|&i| doc.nodes[i])
            .map(|node| Entity {
                children: node.children.iter().map(|&i| i.into()).collect(),
                mesh: node.mesh.map(|i| i.into()),
                transform: ,
            })
            .collect();
        self.meshes = scene.nodes.iter()
            .map(|&i| no puedo pensar estoy malito :()
        self.materials = ;
        self.textures = ;
    }

    fn update(&mut self, delta: Duration) {
        self.physics.step(); // TODO! once every 1/60th second. fixed timestep!!

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

        for entity in &self.world.statics {
            // let scale = Matrix4::from_scale(100.0);
            // let rotation = Matrix4::from_angle_z(Rad(PI/2.0));
            // let translation = Matrix4::from_translation([0.0, 0.0, 0.0].into());
            // let model: [[f32; 4]; 4] = (scale * rotation * translation).into();

            let model = entity.transform;
            let o = &entity.render_object;

            self.shader
                .uniform("perspective", perspective.to_cols_array_2d());
            self.shader.uniform("view", view.to_cols_array_2d());
            self.shader.uniform("model", model);
            self.shader.uniform(
                "diffuse_texture",
                o.diffuse.clone().unwrap_or(graphics::Texture::empty()),
            );
            self.shader.uniform(
                "normal_texture",
                o.normal.clone().unwrap_or(graphics::Texture::empty()),
            );
            self.shader
                .uniform("has_diffuse_texture", o.diffuse.is_some());
            self.shader
                .uniform("has_normal_texture", o.normal.is_some());
            self.shader
                .uniform("base_diffuse_color", o.base_diffuse_color);
            self.shader
                .uniform("u_light_direction", [-1.0, 0.4, 0.9f32]);

            if let Some(geometry) = self.world.geometries.get(o.geometry_id) {
                frame.render(geometry, &self.shader);
            }
        }
    }
}

fn main() {
    // TODO: gui panic -> std::panic::set_hook(Box::new(|_| { }));

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
