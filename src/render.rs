use crate::{GameState, graphics, gltf };

pub struct Mesh {
    pub geometry: graphics::Geometry,
    pub transform: [[f32; 4]; 4], // this doesn't go here, it's temporary
    pub diffuse: Option<graphics::Texture>,
    pub normal: Option<graphics::Texture>,
    pub base_diffuse_color: [f32; 4],
}

fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s_norm[0], u[0], f[0], 0.0],
        [s_norm[1], u[1], f[1], 0.0],
        [s_norm[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
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

        let perspective = {
            let (width, height) = window_dimensions;
            let aspect_ratio = height as f32 / width as f32;

            let fov: f32 = std::f32::consts::PI / 3.0;
            let zfar = 1024.0;
            let znear = 0.1;

            let f = 1.0 / (fov / 2.0).tan();

            [
                [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
                [         0.0         ,     f ,              0.0              ,   0.0],
                [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
                [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
            ]
        };

        let camera_direction = [
            game.camera_yaw.cos() * game.camera_pitch.cos(),
            game.camera_yaw.sin() * game.camera_pitch.cos(),
            game.camera_pitch.sin(),
        ];
        let view = view_matrix(&game.camera_position.into(), &camera_direction, &[0.0, 0.0, 1.0]);

        for mesh in &self.sample_scene {
            // let scale = Matrix4::from_scale(100.0);
            // let rotation = Matrix4::from_angle_z(Rad(PI/2.0));
            // let translation = Matrix4::from_translation([0.0, 0.0, 0.0].into());
            // let model: [[f32; 4]; 4] = (scale * rotation * translation).into();

            let model = mesh.transform;

            self.shader.uniform("perspective", perspective);
            self.shader.uniform("view", view);
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