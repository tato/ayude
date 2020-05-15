
use glium::{implement_vertex, Display, VertexBuffer, Program, texture::RawImage2d, Texture2d, Surface, index::{PrimitiveType, NoIndices}, program, uniform};
use crate::GameState;
use std::f32::consts::PI;
use cgmath::{Rad, Matrix4};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    uv: [f32; 2],
}
implement_vertex!(Vertex, position, normal, uv);

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
}

pub fn initialize_render_state(display: &Display) -> RenderState {
    let shape = VertexBuffer::new(display, &[
        Vertex { position: [-1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 1.0] },
        Vertex { position: [ 1.0,  1.0, 0.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 1.0] },
        Vertex { position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], uv: [0.0, 0.0] },
        Vertex { position: [ 1.0, -1.0, 0.0], normal: [0.0, 0.0, -1.0], uv: [1.0, 0.0] },
    ]).unwrap();

    let vertex_shader_src = r#"
        #version 150

        in vec3 position;
        in vec3 normal;
        in vec2 uv;

        out vec3 v_position;
        out vec3 v_normal;
        out vec2 v_uv;

        uniform mat4 perspective;
        uniform mat4 view;
        uniform mat4 model;

        void main() {
            mat4 modelview = view * model;

            v_normal = transpose(inverse(mat3(modelview))) * normal;

            gl_Position = perspective * modelview * vec4(position, 1.0);
            v_position = gl_Position.xyz / gl_Position.w;

            v_uv = uv;
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        in vec3 v_position;
        in vec3 v_normal;
        in vec2 v_uv;
        out vec4 color;

        uniform vec3 u_light_direction;
        uniform sampler2D diffuse_texture;
        uniform sampler2D normal_texture;

        const vec3 specular_color = vec3(1.0, 1.0, 1.0);

        mat3 cotangent_frame(vec3 normal, vec3 pos, vec2 uv) {
            vec3 dp1 = dFdx(pos);
            vec3 dp2 = dFdy(pos);
            vec2 duv1 = dFdx(uv);
            vec2 duv2 = dFdy(uv);
        
            vec3 dp2perp = cross(dp2, normal);
            vec3 dp1perp = cross(normal, dp1);
            vec3 T = dp2perp * duv1.x + dp1perp * duv2.x;
            vec3 B = dp2perp * duv1.y + dp1perp * duv2.y;
        
            float invmax = inversesqrt(max(dot(T, T), dot(B, B)));
            return mat3(T * invmax, B * invmax, normal);
        }

        void main() {
            vec3 normal_map = texture(normal_texture, v_uv).rgb;
            mat3 tbn = cotangent_frame(v_normal, v_position, v_uv);
            vec3 real_normal = normalize(tbn * -(normal_map * 2.0 - 1.0));

            float diffuse = max(dot(normalize(real_normal), normalize(u_light_direction)), 0.0);

            vec3 camera_dir = normalize(-v_position);
            vec3 half_direction = normalize(normalize(u_light_direction) + camera_dir);
            float specular = pow(max(dot(half_direction, normalize(real_normal)), 0.0), 16.0);

            vec3 diffuse_color = texture(diffuse_texture, v_uv).rgb;
            vec3 ambient_color = diffuse_color * 0.1;

            color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
        }
    "#;

    let program = Program::from_source(display, vertex_shader_src, fragment_shader_src, None).unwrap();

    
    let diffuse_texture = {
        let image = image::load_from_memory(include_bytes!("bonfire.png")).unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        let raw_image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        Texture2d::new(display, raw_image).unwrap()
    };

    let normal_texture = {
        let image = image::load_from_memory(include_bytes!("normal.png")).unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        let raw_image = RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions);
        Texture2d::new(display, raw_image).unwrap()
    };

    RenderState{ shape, program, diffuse_texture, normal_texture }
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

    let scale = Matrix4::from_scale(100.0);
    let rotation = Matrix4::from_angle_z(Rad(PI/2.0));
    let translation = Matrix4::from_translation([0.0, 0.0, 0.0].into());
    let model: [[f32; 4]; 4] = (scale * rotation * translation).into();
    
    let uniforms = uniform! {
        perspective: perspective,
        view: view,
        model: model,
        diffuse_texture: &state.diffuse_texture,
        normal_texture: &state.normal_texture,
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

    target.draw(&state.shape, &NoIndices(PrimitiveType::TriangleStrip), &state.program, &uniforms, &params).unwrap();
    target.finish().unwrap();
}