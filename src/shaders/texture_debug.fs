#version 330

in vec3 frag_pos;
in vec2 frag_texture_coord;

out vec4 color;

uniform sampler2D test_texture;

void main() {
   // color = vec4(1.0, 0.0, 0.0, 1.0);
    color = vec4(texture(test_texture, frag_texture_coord).rgb, 1.0);
}
