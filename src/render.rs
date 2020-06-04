use crate::{GameState, graphics, gltf };

pub struct Mesh {
    pub geometry: graphics::Geometry,
    pub transform: [[f32; 4]; 4], // this doesn't go here, it's temporary
    pub diffuse: Option<graphics::Texture>,
    pub normal: Option<graphics::Texture>,
    pub base_diffuse_color: [f32; 4],
}

pub struct RenderState {
    shader: graphics::Shader,
    sample_scene: Vec<Mesh>,
}

impl RenderState {
    pub fn new() -> RenderState {
        static VERTEX_SOURCE: &str = include_str!("resources/vertex.glsl");
        static FRAGMENT_SOURCE: &str = include_str!("resources/fragment.glsl");
        let shader = graphics::Shader::from_sources(VERTEX_SOURCE, FRAGMENT_SOURCE).unwrap();
    
        let sample_scene = gltf::load_gltf("samples/glTF-Sample-Models/2.0/Sponza/glTF/Sponza.gltf").unwrap();
    
        Self{ shader, sample_scene }
    }

    pub fn render(&mut self, game: &GameState, window_dimensions: (i32, i32)) {
        // state.texture_repository.poll_textures(display);

        let frame = graphics::Frame::start([0.0, 0.0, 1.0], window_dimensions);

        let perspective = glam::Mat4::perspective_rh_gl(
            std::f32::consts::PI / 3.0,
            window_dimensions.0 as f32 / window_dimensions.1 as f32,
            0.1,
            1024.0,
        );

        let camera_direction = [
            game.camera_yaw.cos() * game.camera_pitch.cos(),
            game.camera_yaw.sin() * game.camera_pitch.cos(),
            game.camera_pitch.sin(),
        ].into();
        let view = glam::Mat4::look_at_rh(game.camera_position + camera_direction, game.camera_position, [0.0, 0.0, 1.0].into());

        for mesh in &self.sample_scene {
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