#![feature(clamp)]
use glam::Vec3;
use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};
use glfw::Context;
use ayude::*;

pub struct SceneNode {
    pub geometry: graphics::Geometry,
    pub transform: [[f32; 4]; 4], // this doesn't go here, it's temporary
    pub diffuse: Option<graphics::Texture>,
    pub normal: Option<graphics::Texture>,
    pub base_diffuse_color: [f32; 4],
}

pub struct Scene {
    pub nodes: Vec<SceneNode>
}

impl Scene {
    fn upload(scene: gltf::UnloadedScene) -> Result<Self, AyudeError> {
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

pub struct GameState {
    camera_position: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,

    movement: [f32; 2], // stores WASD input
    
    shader: graphics::Shader,
    sample_scene: Scene,
}

impl GameState {
    fn new() -> Self {
        static VERTEX_SOURCE: &str = include_str!("resources/vertex.glsl");
        static FRAGMENT_SOURCE: &str = include_str!("resources/fragment.glsl");
        let shader = graphics::Shader::from_sources(VERTEX_SOURCE, FRAGMENT_SOURCE).unwrap();
    
        let sample_scene = {
            let unloaded = gltf::load_gltf("samples/glTF-Sample-Models/2.0/Sponza/glTF/Sponza.gltf").unwrap();
            Scene::upload(unloaded).unwrap()
        };

        GameState {
            camera_position: [0.0, 0.0, 0.0].into(),
            camera_yaw: 0.0,
            camera_pitch: 0.0,
    
            movement: [0.0, 0.0],

            shader,
            sample_scene
        }
    }

    fn update_and_render(&mut self, delta: Duration, window_dimensions: (i32, i32)) {
        let mut forward_direction: Vec3 = [
            (-self.camera_yaw).cos() * self.camera_pitch.cos(),
            (-self.camera_yaw).sin() * self.camera_pitch.cos(),
            self.camera_pitch.sin(),
        ].into();
        forward_direction = forward_direction.normalize();
        let right_direction: Vec3 = forward_direction.cross([0.0, 0.0, 1.0].into()).normalize();
    
        let speed = 100.0;
        self.camera_position += forward_direction * self.movement[1] * speed * delta.as_secs_f32();
        self.camera_position += right_direction * self.movement[0] * speed * delta.as_secs_f32();
        
        // state.texture_repository.poll_textures(display);

        let frame = graphics::Frame::start([0.0, 0.0, 1.0], window_dimensions);

        let perspective = glam::Mat4::perspective_rh_gl(
            std::f32::consts::PI / 3.0,
            window_dimensions.0 as f32 / window_dimensions.1 as f32,
            0.1,
            1024.0,
        );

        let view = glam::Mat4::look_at_rh(self.camera_position, self.camera_position + forward_direction, [0.0, 0.0, 1.0].into());

        for mesh in &self.sample_scene.nodes {
            // let scale = Matrix4::from_scale(100.0);
            // let rotation = Matrix4::from_angle_z(Rad(PI/2.0));
            // let translation = Matrix4::from_translation([0.0, 0.0, 0.0].into());
            // let model: [[f32; 4]; 4] = (scale * rotation * translation).into();

            let model = mesh.transform;

            self.shader.uniform("perspective", perspective.to_cols_array_2d());
            self.shader.uniform("view", view.to_cols_array_2d());
            self.shader.uniform("model", model);
            self.shader.uniform("diffuse_texture", mesh.diffuse.clone().unwrap_or(graphics::Texture::empty()));
            self.shader.uniform("normal_texture", mesh.normal.clone().unwrap_or(graphics::Texture::empty()));
            self.shader.uniform("has_diffuse_texture", mesh.diffuse.is_some());
            self.shader.uniform("has_normal_texture", mesh.normal.is_some());
            self.shader.uniform("base_diffuse_color", mesh.base_diffuse_color);
            self.shader.uniform("u_light_direction", [-1.0, 0.4, 0.9f32]);

            frame.render(&mesh.geometry, &self.shader);
        }
    }
}

fn main() {
    let mut glfw = glfw::init(Some(glfw::FAIL_ON_ERRORS.unwrap())).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(false));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    //glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::OpenGlEs));

    let (mut window, events) = glfw.create_window(
        800, 600,
        "a.yude",
        glfw::WindowMode::Windowed
    ).unwrap();

    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.make_current();

    let mut previous_cursor_pos = (0.0, 0.0);

    gl::load_with(|s| window.get_proc_address(s));

    let mut game = GameState::new();

    let mut previous_frame_time = Instant::now();

    'running: while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Close => {
                    break 'running;
                },
                glfw::WindowEvent::CursorPos(x, y) => {
                    let delta = (x - previous_cursor_pos.0, y - previous_cursor_pos.1);
                    previous_cursor_pos = (x, y);

                    game.camera_yaw += delta.0 as f32 * 0.006;
                    if game.camera_yaw >= 2.0 * PI {
                        game.camera_yaw -= 2.0 * PI;
                    }
                    if game.camera_yaw <= 0.0 { game.camera_yaw += 2.0*PI; }

                    let freedom_y = 0.8;
                    game.camera_pitch -= delta.1 as f32 * 0.006;
                    game.camera_pitch = game
                        .camera_pitch
                        .clamp(-PI / 2.0 * freedom_y, PI / 2.0 * freedom_y);
                },
                glfw::WindowEvent::Key(key, _scancode, action, _modifiers) => match key {
                    glfw::Key::W => {
                        if action == glfw::Action::Press {
                            game.movement[1] = 1.0;
                        } else if action == glfw::Action::Release {
                            game.movement[1] = 0.0f32.min(game.movement[1]);
                        }
                    }
                    glfw::Key::A => {
                        if action == glfw::Action::Press {
                            game.movement[0] = -1.0;
                        } else if action == glfw::Action::Release {
                            game.movement[0] = 0.0f32.max(game.movement[0]);
                        }
                    }
                    glfw::Key::S => {
                        if action == glfw::Action::Press {
                            game.movement[1] = -1.0;
                        } else if action == glfw::Action::Release {
                            game.movement[1] = 0.0f32.max(game.movement[1]);
                        }
                    }
                    glfw::Key::D => {
                        if action == glfw::Action::Press {
                            game.movement[0] = 1.0;
                        } else if action == glfw::Action::Release {
                            game.movement[0] = 0.0f32.min(game.movement[0]);
                        }
                    }
                    _ => { },
                },
                _ => return,
            }
        }
        let delta = previous_frame_time.elapsed();
        previous_frame_time = Instant::now();
        game.update_and_render(delta, window.get_framebuffer_size());
        window.swap_buffers();
    }
}
