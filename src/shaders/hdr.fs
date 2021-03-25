#version 330
in vec3 frag_pos;

out vec4 color;

const vec2 inv_arctan = vec2(0.1591, 0.3183);

uniform sampler2D hdr_texture;

vec2 get_uv(vec3 position) {
    vec2 uv = vec2(atan(position.z, position.x), asin(position.y));
    uv *= inv_arctan;
    uv += 0.5;
    return uv;
}

void main() {
    vec2 uv = get_uv(normalize(frag_pos));
    color = vec4(texture(hdr_texture, uv).rgb, 1.0);
}
