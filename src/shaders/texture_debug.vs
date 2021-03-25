#version 330

in vec3 position;
in vec2 texture_coord;

out vec3 frag_pos;
out vec2 frag_texture_coord;

uniform mat4 projection_matrix;
uniform mat4 view_matrix;

void main() {
    frag_pos = vec3(position.xy, 1.0);
    frag_texture_coord = texture_coord;
    gl_Position = vec4(position.xy, 1.0, 1.0);
//    gl_Position = projection_matrix * view_matrix * vec4(position.xy, 1.0, 1.0);
}
