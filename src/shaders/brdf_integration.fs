#version 330

in vec3 frag_pos;
in vec2 frag_texture_coord;

out vec4 color;

const float PI = 3.14159265359;
const uint SAMPLE_COUNT = 1024u;

// Useable on new hardware. NOT usable on WebGL nor OpenGL ES 2.0
float radical_inverse_van_der_corpus(uint bits) {
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return float(bits) * 2.3283064365386963e-10;  // / 0x100000000
}

// Same as above but works on older hardware
// Slower because loop MUST run each time
float van_der_corpus_old(uint n, uint base) {
    float inv_base = 1.0 / float(base);
    float denominator = 1.0;
    float result = 0.0;

    for (uint i = 0u; i < 32u; ++i) {
        if (n > 0u) {
            denominator = mod(float(n), 2.0);
            result += denominator * inv_base;
            inv_base = inv_base / 2.0;
            n = uint(float(n) / 2.0);
        }
    }

    return result;
}

vec2 hammersley(uint i, uint N) {
    return vec2(float(i) / float(N), radical_inverse_van_der_corpus(i));
}

vec2 hammersley_old(uint i, uint N) {
    return vec2(float(i) / float(N), van_der_corpus_old(i, 2u));
}

vec3 importance_sample_GGX(vec2 x_i, vec3 N, float roughness) {
    float a = roughness * roughness;
    float phi = 2.0 * PI * x_i.x;

    float numerator = 1.0 - x_i.y;
    float denominator = 1.0 + (a * a - 1.0) * x_i.y;
    float cos_theta = sqrt(numerator / denominator);
    float sin_theta = sqrt(1.0 - cos_theta * cos_theta);

    vec3 H = vec3(cos(phi) * sin_theta, sin(phi) * sin_theta, cos_theta);

    vec3 up = vec3(1.0, 0.0, 0.0);
    if (abs(N.z) < 0.999) {
        up = vec3(0.0, 0.0, 1.0);
    }
    vec3 tangent = normalize(cross(up, N));
    vec3 bitangent = normalize(cross(N, tangent));

    vec3 sample_direction = tangent * H.x + bitangent * H.y + N * H.z;
    return normalize(sample_direction);
}

float geometry_schlick_GGX(float NdotV, float roughness) {
    float a = roughness;
    float k = (a * a) / 2.0;

    float numerator = NdotV;
    float denominator = NdotV * (1.0 - k) + k;

    return numerator / denominator;
}

float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = geometry_schlick_GGX(NdotV, roughness);
    float ggx1 = geometry_schlick_GGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec2 integrate_BRDF(float NdotV, float roughness) {
    vec3 V = vec3(sqrt(1.0 - NdotV * NdotV), 0.0, NdotV);

    float A = 0.0;
    float B = 0.0;

    vec3 N = vec3(0.0, 0.0, 1.0);

    const uint SAMPLE_COUNT = 1024u;
    for (uint i = 0u; i < SAMPLE_COUNT; ++i) {
        vec2 x_i = hammersley(i, SAMPLE_COUNT);
        vec3 H = importance_sample_GGX(x_i, N, roughness);
        vec3 L = normalize(2.0 * dot(V, H) * H - V);

        float NdotL = max(L.z, 0.0);
        float NdotH = max(H.z, 0.0);
        float VdotH = max(dot(V, H), 0.0);

        if (NdotL > 0.0) {
            float G = geometry_smith(N, V, L, roughness);
            float G_Vis = (G * VdotH) / (NdotH * NdotV);
            float F_c = pow(1.0 - VdotH, 5.0);

            A += (1.0 - F_c) * G_Vis;
            B += F_c * G_Vis;
        }
    }
    A /= float(SAMPLE_COUNT);
    B /= float(SAMPLE_COUNT);
    return vec2(A, B);
}

void main() {
    vec2 integrated_BRDF =
        integrate_BRDF(frag_texture_coord.x, frag_texture_coord.y);
    //    color = vec4(1.0, 0.0, 0.0, 1.0);
    color = vec4(integrated_BRDF, 0.0, 1.0);
}
