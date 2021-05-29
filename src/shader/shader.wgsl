struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] norpos: vec3<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

[[block]]
struct Uniforms {
    mvp: mat4x4<f32>;
    transpose_inverse_modelview: mat3x3<f32>;
    light_direction: vec3<f32>;
    has_diffuse_texture: u32;
    has_normal_texture: u32;
    base_diffuse_color: vec4<f32>;
    shaded: u32;
};
[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec4<f32>,
    [[location(1)]] normal: vec3<f32>,
    [[location(2)]] tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.normal = uniforms.transpose_inverse_modelview * normal;
    out.position = uniforms.mvp * position;
    out.norpos = out.position.xyz / out.position.w;
    out.tex_coord = tex_coord;
    return out;
}

fn cotangent_frame(normal: vec3<f32>, pos: vec3<f32>, uv: vec2<f32>) -> mat3x3<f32> {
    let dp1 = dpdx(pos);
    let dp2 = dpdy(pos);
    let duv1 = dpdx(uv);
    let duv2 = dpdy(uv);

    let dp2perp = cross(dp2, normal);
    let dp1perp = cross(normal, dp1);
    let T = dp2perp * duv1.x + dp1perp * duv2.x;
    let B = dp2perp * duv1.y + dp1perp * duv2.y;

    let invmax = inverseSqrt(max(dot(T, T), dot(B, B)));
    return mat3x3<f32>(T * invmax, B * invmax, normal);
}

[[group(0), binding(1)]]
var diffuse_texture: texture_2d<u32>;
[[group(0), binding(2)]]
var normal_texture: texture_2d<u32>;



[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    if (uniforms.shaded == u32(0)) {
        var diffuse_color: vec3<f32>;
        if (uniforms.has_diffuse_texture > u32(0)) {
            let i_diffuse_color = textureLoad(diffuse_texture, vec2<i32>(in.tex_coord * 256.0), 0).rgb;
            diffuse_color = vec3<f32>(i_diffuse_color) / 256.0;
        } else {
            diffuse_color = uniforms.base_diffuse_color.rgb;
        }
        return vec4<f32>(diffuse_color, 1.0);
    }
    
    var real_normal: vec3<f32>;
    if (uniforms.has_normal_texture > u32(0)) {
        let i_real_normal = textureLoad(normal_texture, vec2<i32>(in.tex_coord * 256.0), 0).rgb;
        real_normal = vec3<f32>(i_real_normal) / 256.0;
    } else {
        real_normal = in.normal;
    }

    let diffuse = max(dot(normalize(real_normal), normalize(uniforms.light_direction)), 0.0);

    let camera_dir = normalize(-in.norpos);
    let half_direction = normalize(normalize(uniforms.light_direction) + camera_dir);
    let tbn = cotangent_frame(in.normal, in.norpos, in.tex_coord);
    let specular = pow(max(dot(half_direction, normalize(tbn * -(real_normal * 2.0 - 1.0))), 0.0), 16.0);

    var diffuse_color: vec3<f32>;
    if (uniforms.has_diffuse_texture > u32(0)) {
        let i_diffuse_color = textureLoad(diffuse_texture, vec2<i32>(in.tex_coord * 256.0), 0).rgb;
        diffuse_color = vec3<f32>(i_diffuse_color) / 256.0;
    } else {
        diffuse_color = uniforms.base_diffuse_color.rgb;
    }
    let ambient_color = diffuse_color * 0.1;

    let specular_color = vec3<f32>(1.0, 1.0, 1.0);
    return vec4<f32>(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
}