#version 330
in vec3 frag_pos;

out vec4 color;

uniform samplerCube skybox;

const float PI = 3.14159265359;
const vec3 UP = vec3(0.0, 1.0, 0.0);

void main() {
    vec3 irradiance = vec3(0.0);

    vec3 direction = normalize(frag_pos);
    vec3 right = normalize(cross(UP, direction));
    vec3 up = normalize(cross(direction, right));

    // Accuracy of irradiance map
    float sample_delta = 0.025;
    int sample_count = 0;
    for (float phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
        for (float theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
            // Tangent space
            vec3 tangent_direction =
                vec3(sin(theta) * cos(phi), sin(theta) * sin(phi), cos(theta));

            // World space
            vec3 sample_direction = tangent_direction.x * right +
                                    tangent_direction.y * up +
                                    tangent_direction.z * direction;

            // Sample and increase radiance of current solid angle
            irradiance +=
                texture(skybox, sample_direction).rgb * cos(theta) * sin(theta);
            sample_count++;
        }
    }
    // Clamp to ensure no divide by zero error
    irradiance =
        PI * irradiance * (1.0 / max(float(sample_count), 0.000001));
    color = vec4(irradiance, 1.0);
}
