#version 330

in vec3 v_position;
in vec3 v_normal;
in vec2 v_uv;
out vec4 color;

uniform vec3 u_light_direction;
uniform sampler2D diffuse_texture;
uniform sampler2D normal_texture;
uniform bool has_diffuse_texture;
uniform bool has_normal_texture;
uniform vec4 base_diffuse_color;
uniform bool shaded;

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
    if (!shaded) {
        vec3 diffuse_color;
        if (has_diffuse_texture) {
            diffuse_color = texture(diffuse_texture, v_uv).rgb;
        } else {
            diffuse_color = base_diffuse_color.rgb;
        }
        color = vec4(diffuse_color, 1.0);
        return;
    }

    vec3 real_normal;
    if (has_normal_texture) {
        real_normal = texture(normal_texture, v_uv).rgb;
    } else {
        real_normal = v_normal;
    }

    float diffuse = max(dot(normalize(real_normal), normalize(u_light_direction)), 0.0);

    vec3 camera_dir = normalize(-v_position);
    vec3 half_direction = normalize(normalize(u_light_direction) + camera_dir);
    mat3 tbn = cotangent_frame(v_normal, v_position, v_uv);
    float specular = pow(max(dot(half_direction, normalize(tbn * -(real_normal * 2.0 - 1.0))), 0.0), 16.0);

    vec3 diffuse_color;
    if (has_diffuse_texture) {
        diffuse_color = texture(diffuse_texture, v_uv).rgb;
    } else {
        diffuse_color = base_diffuse_color.rgb;
    }
    vec3 ambient_color = diffuse_color * 0.1;

    color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
}