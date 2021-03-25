#version 330

in vec3 position;
in vec2 texture_coord;

out vec3 frag_pos;
out vec2 frag_texture_coord;

void main() {
    frag_pos = position;
    frag_texture_coord = texture_coord;
    gl_Position = vec4(position, 1.0);
}
