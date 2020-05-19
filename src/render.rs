
use glium::{implement_vertex, Display, VertexBuffer, Program, texture::RawImage2d, Texture2d, Surface, uniform, IndexBuffer};
use crate::{GameState};

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}
implement_vertex!(Vertex, position, normal, uv);

pub struct Mesh {
    pub vertices: VertexBuffer<Vertex>,
    pub indices: IndexBuffer<u16>,
    pub transform: [[f32; 4]; 4], // this doesn't go here, it's temporary
    pub diffuse: Option<Texture2d>,
    pub normal: Option<Texture2d>,
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
    shape: VertexBuffer<Vertex>,
    program: Program,
    diffuse_texture: Texture2d,
    normal_texture: Texture2d,

    sample_scene: Vec<Mesh>,
}

pub fn initialize_render_state(display: &Display) -> RenderState {
    let shape = VertexBuffer::new(display, &[
        Vertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0] },
        Vertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0] },
        Vertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0] },
    ]).unwrap();

    static VERTEX_SOURCE: &str = include_str!("vertex.glsl");
    static FRAGMENT_SOURCE: &str = include_str!("fragment.glsl");

    let program = Program::from_source(display, VERTEX_SOURCE, FRAGMENT_SOURCE, None).unwrap();
    
    let diffuse_texture = load_texture_from_image_in_memory(display, include_bytes!("bonfire.png")).unwrap();
    let normal_texture = load_texture_from_image_in_memory(display, include_bytes!("normal.png")).unwrap();

    let sample_scene = crate::gltf::load_gltf(&display, "samples/glTF-Sample-Models/2.0/Sponza/glTF/Sponza.gltf").unwrap();

    RenderState{ shape, program, diffuse_texture, normal_texture, sample_scene }
}

pub fn render(display: &Display, state: &RenderState, game: &GameState) {
    let mut target = display.draw();
    target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

    let perspective = {
        let (width, height) = target.get_dimensions();
        let aspect_ratio = height as f32 / width as f32;
    
        let fov: f32 = 3.141592 / 3.0;
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

    for mesh in &state.sample_scene {
        // let scale = Matrix4::from_scale(100.0);
        // let rotation = Matrix4::from_angle_z(Rad(PI/2.0));
        // let translation = Matrix4::from_translation([0.0, 0.0, 0.0].into());
        // let model: [[f32; 4]; 4] = (scale * rotation * translation).into();

        let model = mesh.transform;

        let diffuse_texture = mesh.diffuse.as_ref().unwrap_or(&state.diffuse_texture);
        let normal_texture = mesh.normal.as_ref().unwrap_or(&state.normal_texture);
        
        let uniforms = uniform! {
            perspective: perspective,
            view: view,
            model: model,
            diffuse_texture: diffuse_texture,
            normal_texture: normal_texture,
            has_diffuse_texture: mesh.diffuse.is_some(),
            has_normal_texture: mesh.normal.is_some(),
            base_diffuse_color: mesh.base_diffuse_color,
            u_light_direction: [-1.0, 0.4, 0.9f32],
        };
    
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                ..Default::default()
            },
            //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            ..Default::default()
        };
    
        // target.draw(&state.shape, &NoIndices(PrimitiveType::TriangleStrip), &state.program, &uniforms, &params).unwrap();
        target.draw(&mesh.vertices, &mesh.indices, &state.program, &uniforms, &params).unwrap();
    }
    
    target.finish().unwrap();
}


pub fn load_texture_from_image_in_memory(display: &Display, input: &[u8]) -> Option<Texture2d> {

    let mut width: i32 = 0;
    let mut height: i32 = 0;
    let mut channels: i32 = 0;

    unsafe {
        let bytes = stb_image::stb_image::bindgen::stbi_load_from_memory(
            input.as_ptr(),
            input.len() as i32,
            &mut width as *mut i32,
            &mut height as *mut i32,
            &mut channels as *mut i32,
            4
        );
        
        if bytes.is_null() {
            let _reason = std::ffi::CStr::from_ptr(stb_image::stb_image::bindgen::stbi_failure_reason());
            None
        } else {
            let bytes_length = (width*height*4) as usize;
            let owned = Vec::from_raw_parts(bytes, bytes_length, bytes_length);
            let raw_image = RawImage2d::from_raw_rgba(owned, (width as u32, height as u32));
            Texture2d::new(display, raw_image).ok()
        }
    }
}