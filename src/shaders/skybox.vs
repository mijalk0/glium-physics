#version 330
in vec3 position;

out vec3 frag_pos;

uniform mat4 view_matrix;
uniform mat4 projection_matrix;

void main() {
    frag_pos = position;
    vec4 position = projection_matrix * view_matrix * vec4(position, 1.0);
    gl_Position = position.xyww;
}
