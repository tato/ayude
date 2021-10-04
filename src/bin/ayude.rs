#![feature(clamp)]
use glam::Vec3;
use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};
use ayude::*;
use glutin::{window::WindowBuilder, event_loop::{EventLoop, ControlFlow}, dpi::LogicalSize, ContextBuilder, event::{WindowEvent, Event, DeviceEvent, VirtualKeyCode, ElementState}, Api, GlRequest, Robustness, GlProfile};

pub struct SceneNode {
    pub geometry: Handle<graphics::Geometry>,
    pub transform: [[f32; 4]; 4], // this doesn't go here, it's temporary
    pub diffuse: Option<graphics::Texture>,
    pub normal: Option<graphics::Texture>,
    pub base_diffuse_color: [f32; 4],
}

pub struct Scene {
    pub nodes: Vec<SceneNode>
}

impl Scene {
    fn upload(scene: gltf::UnloadedScene, meshes: &mut Catalog<graphics::Geometry>) -> Result<Self, AyudeError> {
        let mut nodes = Vec::new();
        let textures = scene.images.iter().map(|image| {
            graphics::Texture::from_rgba(&scene.images_byte_buffer[image.offset..image.offset+image.size], image.width as i32, image.height as i32)
        }).collect::<Vec<_>>();
        for unode in scene.nodes {
            let transform = unode.transform;
            let base_diffuse_color = unode.base_diffuse_color;

            let geometry = graphics::Geometry::new(
                &unode.geometry_positions,
                &unode.geometry_normals,
                &unode.geometry_uvs,
                &unode.geometry_indices
            );
            let geometry = meshes.add(geometry);

            let diffuse = unode.diffuse.map(|index| {
                textures[index].clone()
            });
            let normal =  unode.normal.map(|index| {
                textures[index].clone()
            });

            nodes.push(SceneNode{ geometry, transform, diffuse, normal, base_diffuse_color });
        }
        Ok(crate::Scene{ nodes })
    }
}

fn calculate_forward_direction(yaw: f32, pitch: f32) -> Vec3 {
    let result: Vec3 = [
        (-yaw).cos() * pitch.cos(),
        (-yaw).sin() * pitch.cos(),
        pitch.sin(),
    ].into();
    result.normalize()
}

pub struct World {
    camera_position: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,

    movement: [f32; 2], // stores WASD input
    
    shader: graphics::Shader,
    sample_scene: Scene,

    meshes: Catalog<graphics::Geometry>,

    physics: physics::PhysicsState
}

impl World {
    fn new() -> Self {
        static VERTEX_SOURCE: &str = include_str!("../resources/vertex.glsl");
        static FRAGMENT_SOURCE: &str = include_str!("../resources/fragment.glsl");
        let shader = graphics::Shader::from_sources(VERTEX_SOURCE, FRAGMENT_SOURCE).unwrap();

        let mut meshes = Catalog::new();
    
        let sample_scene = {
            let gltf_file_name = "samples/glTF-Sample-Models/2.0/Sponza/glTF/Sponza.gltf";
            // let gltf_file_name = "samples/homework_09_simple_textures/scene.gltf";
            let gltf_file_name = "samples/physicstest.gltf";
            let unloaded = gltf::load_gltf(gltf_file_name).unwrap();
            Scene::upload(unloaded, &mut meshes).unwrap()
        };

        let physics = physics::PhysicsState::new();

        World {
            camera_position: [0.0, 0.0, 0.0].into(),
            camera_yaw: 0.0,
            camera_pitch: 0.0,
    
            movement: [0.0, 0.0],

            shader,
            sample_scene,

            meshes,

            physics
        }
    }

    fn update(&mut self, delta: Duration) {
        self.physics.step(); // TODO! once every 1/60th second. fixed timestep!!

        let forward_direction = calculate_forward_direction(self.camera_yaw, self.camera_pitch);
        let right_direction: Vec3 = forward_direction.cross([0.0, 0.0, 1.0].into()).normalize();
    
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

        let view = glam::Mat4::look_at_rh(self.camera_position, self.camera_position + forward_direction, [0.0, 0.0, 1.0].into());

        for node in &self.sample_scene.nodes {
            // let scale = Matrix4::from_scale(100.0);
            // let rotation = Matrix4::from_angle_z(Rad(PI/2.0));
            // let translation = Matrix4::from_translation([0.0, 0.0, 0.0].into());
            // let model: [[f32; 4]; 4] = (scale * rotation * translation).into();

            let model = node.transform;

            self.shader.uniform("perspective", perspective.to_cols_array_2d());
            self.shader.uniform("view", view.to_cols_array_2d());
            self.shader.uniform("model", model);
            self.shader.uniform("diffuse_texture", node.diffuse.clone().unwrap_or(graphics::Texture::empty()));
            self.shader.uniform("normal_texture", node.normal.clone().unwrap_or(graphics::Texture::empty()));
            self.shader.uniform("has_diffuse_texture", node.diffuse.is_some());
            self.shader.uniform("has_normal_texture", node.normal.is_some());
            self.shader.uniform("base_diffuse_color", node.base_diffuse_color);
            self.shader.uniform("u_light_direction", [-1.0, 0.4, 0.9f32]);

            let geometry = match self.meshes.get(node.geometry) {
                None => continue,
                Some(g) => g,
            };

            frame.render(&geometry, &self.shader);
        }
    }
}

fn main() {
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

    let mut world = World::new();

    let mut previous_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        
        match event {
            Event::WindowEvent{ event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => return,
            },
            Event::DeviceEvent{ event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    world.camera_yaw += delta.0 as f32 * 0.006;
                    if world.camera_yaw >= 2.0 * PI {
                        world.camera_yaw -= 2.0 * PI;
                    }
                    if world.camera_yaw <= 0.0 { world.camera_yaw += 2.0*PI; }
    
                    let freedom_y = 0.8;
                    world.camera_pitch -= delta.1 as f32 * 0.006;
                    world.camera_pitch = world
                        .camera_pitch
                        .clamp(-PI / 2.0 * freedom_y, PI / 2.0 * freedom_y);
                },
                DeviceEvent::Key(input) => match input.virtual_keycode {
                    Some(VirtualKeyCode::W) => {
                        if input.state == ElementState::Pressed {
                            world.movement[1] = 1.0;
                        } else if input.state == ElementState::Released {
                            world.movement[1] = 0.0f32.min(world.movement[1]);
                        }
                    }
                    Some(VirtualKeyCode::A) => {
                        if input.state == ElementState::Pressed {
                            world.movement[0] = -1.0;
                        } else if input.state == ElementState::Released {
                            world.movement[0] = 0.0f32.max(world.movement[0]);
                        }
                    }
                    Some(VirtualKeyCode::S) => {
                        if input.state == ElementState::Pressed {
                            world.movement[1] = -1.0;
                        } else if input.state == ElementState::Released {
                            world.movement[1] = 0.0f32.max(world.movement[1]);
                        }
                    }
                    Some(VirtualKeyCode::D) => {
                        if input.state == ElementState::Pressed {
                            world.movement[0] = 1.0;
                        } else if input.state == ElementState::Released {
                            world.movement[0] = 0.0f32.min(world.movement[0]);
                        }
                    }
                    _ => return,
                },
                _ => return,
            },
            Event::MainEventsCleared => {
                let delta = previous_frame_time.elapsed();
                previous_frame_time = Instant::now();
                world.update(delta);
                window.window().request_redraw();
            },
            Event::RedrawRequested(..) => {
                let inner_size = window.window().inner_size();
                world.render((inner_size.width as i32, inner_size.height as i32));
                window.swap_buffers().unwrap();
            },
            _ => return,
        }
    });
}
