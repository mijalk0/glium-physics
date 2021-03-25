#version 330
in vec3 frag_pos;

out vec4 color;

uniform samplerCube cubemap;

void main() {
//    vec3 final_color = pow(texture(cubemap, frag_position).rgb, vec3(2.2));
    vec3 final_color = textureLod(cubemap, frag_pos, 0.0).rgb;
//    vec3 final_color = texture(cubemap, frag_pos).rgb;
    final_color = final_color / (final_color + vec3(1.0));
    final_color = pow(final_color, vec3(1.0 / 2.2));
    color = vec4(final_color, 1.0);
}
