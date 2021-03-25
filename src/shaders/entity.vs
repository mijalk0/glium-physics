#version 330
in vec3 position;
in vec2 texture_coord;
in vec3 normal;
in vec3 tangent;

out vec3 frag_position;
out vec2 frag_texture_coord;
out vec3 frag_normal;
out mat3 frag_TBN;

//uniform mat4 mvp_matrix;
uniform mat4 model_matrix;
uniform mat4 view_matrix;
uniform mat4 projection_matrix;

void main() {
    frag_position = (model_matrix * vec4(position, 1.0)).xyz;
    frag_texture_coord = texture_coord;

    mat3 normal_matrix = inverse(transpose(mat3(model_matrix)));
    frag_normal = normalize(normal_matrix * normal);

    vec3 vertex_normal = normalize(normal_matrix * normal);
    vec3 vertex_tangent = normalize(normal_matrix * tangent);
    vertex_tangent = normalize(vertex_tangent - dot(vertex_tangent, vertex_normal) * vertex_normal);
    vec3 vertex_bitangent = cross(vertex_normal, vertex_tangent);
    frag_TBN = mat3(vertex_tangent, vertex_bitangent, vertex_normal);

    gl_Position = projection_matrix * view_matrix * model_matrix * vec4(position, 1.0);
    //gl_Position = mvp_matrix * vec4(position, 1.0);
}
