#version 330 core
in vec3 frag_position;
in vec2 frag_texture_coord;
in vec3 frag_normal;
in mat3 frag_TBN;

out vec4 color;

const float PI = 3.14159265359;
// MAX_REFLECTION_LOD is mipmap_count in prefiltered_maps - 1
const float MAX_REFLECTION_LOD = 6.0;

uniform vec3 view_position;
uniform light_positions { vec3 light_positions_array[512]; };
uniform light_colours { vec3 light_colours_array[512]; };
uniform sampler2D diffuse_map;
uniform sampler2D occlusion_roughness_metal_map;
uniform sampler2D normal_map;
uniform samplerCube irradiance_map;
uniform samplerCube prefiltered_map;
uniform sampler2D brdf_integration;

struct PBR_data {
    vec3 N;
    vec3 L;
    vec3 V;
    vec3 H;
    vec3 R;
    float NdotL;
    float NdotV;
    float HdotV;
    float HdotN;
    float HdotN2;
    float occlusion;
    float roughness;
    float roughness_remapped;
    float metalness;
    float a;
    float a2;
    float k;
    vec3 albedo;
    vec3 F0;
};

vec3 sample_normalmap() {
    return normalize(
        frag_TBN *
        (texture(normal_map, frag_texture_coord).rgb * 2.0 - vec3(1.0)));
}

// Probably safe to delete
vec3 test_sample_normalmap() {
    vec3 tangentNormal = texture(normal_map, frag_texture_coord).xyz * 2.0 - 1.0;

    vec3 Q1  = dFdx(frag_position);
    vec3 Q2  = dFdy(frag_position);
    vec2 st1 = dFdx(frag_texture_coord);
    vec2 st2 = dFdy(frag_texture_coord);

    vec3 N   = normalize(frag_normal);
    vec3 T  = normalize(Q1*st2.t - Q2*st1.t);
    vec3 B  = -normalize(cross(N, T));
    mat3 TBN = mat3(T, B, N);

    return normalize(TBN * tangentNormal);
}

vec4 legacyshader() {
    // 1.0 is used for ambient colour RGB. Can be any colour.
    float ambient_strength = 0.2;
    vec3 ambient = ambient_strength * light_colours_array[0];

    vec3 frag_normal = texture(normal_map, frag_texture_coord).rgb;
    frag_normal = frag_normal * 2.0 - 1.0;
    frag_normal = normalize(frag_TBN * frag_normal);

    vec3 frag_light_vec = normalize(light_positions_array[0] - frag_position);

    // 1.0 is used for light colour RGB. Can be any colour.
    float diffuse_strength = max(dot(frag_normal, frag_light_vec), 0.0);
    vec3 diffuse = diffuse_strength * vec3(1.0, 1.0, 1.0);

    vec4 frag_colour = (vec4(ambient, 1.0) + vec4(diffuse, 1.0)) *
                       texture(diffuse_map, frag_texture_coord);
    return texture(diffuse_map, frag_texture_coord) *
           texture(occlusion_roughness_metal_map, frag_texture_coord).r;
    // return frag_colour;
}

float distribution_GGX(float HdotN2, float a2) {
    float numerator = a2;
    float denominator = (HdotN2 * (a2 - 1.0) + 1.0);
    denominator = PI * denominator * denominator;

    return numerator / denominator;
}

vec3 fresnel_schlick(float HdotV, vec3 F0) {
    // return F0 + (vec3(1.0) - F0) * pow(1.0 - HdotV, 5.0);
    return F0 +
           (vec3(1.0) - F0) * pow(2, (((-5.55473) * HdotV - 6.98316) * HdotV));
}

// This one is used for IBL
vec3 fresnel_schlick_roughness(float NdotV, vec3 F0, float roughness) {
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow((1.0 - NdotV), 5.0);
}

float geometry_schlick_GGX(float NdotV, float k) {
    float numerator = NdotV;
    float denominator = NdotV * (1.0 - k) + k;
    return numerator / denominator;
}

float geometry_smith(float NdotV, float NdotL, float k) {
    float ggx2 = geometry_schlick_GGX(NdotV, k);
    float ggx1 = geometry_schlick_GGX(NdotL, k);

    return ggx1 * ggx2;
}

void main() {
    vec3 albedo = texture(diffuse_map, frag_texture_coord).rgb;
    vec3 orm_vector =
        texture(occlusion_roughness_metal_map, frag_texture_coord).rgb;

    PBR_data pbr_data;
    pbr_data.N = sample_normalmap();
    pbr_data.V = normalize(view_position - frag_position);
    pbr_data.R = reflect(-pbr_data.V, pbr_data.N);
    pbr_data.occlusion = orm_vector.r;
    pbr_data.roughness = 1 - orm_vector.g;
    // Using value from UE4 instead of learnOpenGL
    pbr_data.roughness_remapped = (pbr_data.roughness + 1) / 2;
    pbr_data.metalness = orm_vector.b;
    pbr_data.a = pbr_data.roughness * pbr_data.roughness;
    pbr_data.a2 = pbr_data.a * pbr_data.a;
    pbr_data.k =
        (pbr_data.roughness_remapped + 1 * pbr_data.roughness_remapped + 1) /
        8.0;
    pbr_data.albedo = albedo;
    pbr_data.F0 = mix(vec3(0.04), pbr_data.albedo, pbr_data.metalness);
    pbr_data.NdotV = clamp(dot(pbr_data.N, pbr_data.V), 0.000001, 1.0);

    vec3 final_color = vec3(0.0);

    //for (int i = 0; i < 512; i++) {
    //    if (light_positions_array[i].x > 10000.0) {
    //        break;
    //    }
    //    vec3 L_unnormalized = light_positions_array[i] - frag_position;
    //    pbr_data.L = normalize(L_unnormalized);
    //    pbr_data.H = normalize(pbr_data.L + pbr_data.V);
    //    pbr_data.NdotL = clamp(dot(pbr_data.N, pbr_data.L), 0.000001, 1.0);
    //    pbr_data.HdotV = clamp(dot(pbr_data.H, pbr_data.V), 0.000001, 1.0);
    //    pbr_data.HdotN = clamp(dot(pbr_data.H, pbr_data.N), 0.000001, 1.0);
    //    pbr_data.HdotN2 = pbr_data.HdotN * pbr_data.HdotN;
    //
    //    float D = distribution_GGX(pbr_data.HdotN2, pbr_data.a2);
    //    vec3 F = fresnel_schlick(pbr_data.HdotV, pbr_data.F0);
    //    float G = geometry_smith(pbr_data.NdotV, pbr_data.NdotL, pbr_data.k);
    //
    //    vec3 numerator = D * F * G;
    //    float denominator = 4.0 * pbr_data.NdotV * pbr_data.NdotL;
    //    vec3 specular = numerator / denominator;
    //
    //    float distance = length(L_unnormalized);
    //    float attentuation = 1.0 / (distance * distance);
    //    vec3 radiance = light_colours_array[i] * attentuation;
    //
    //    vec3 k_s = F;
    //    vec3 k_d = vec3(1.0) - k_s;
    //    k_d *= 1.0 - pbr_data.metalness;
    //
    //    final_color +=
    //        (k_d * pbr_data.albedo / PI + specular) * radiance * pbr_data.NdotL;
    //}

    // color = vec4(final_color, 1.0);

    vec3 IBL_k_s = fresnel_schlick_roughness(pbr_data.NdotV, pbr_data.F0,
                                             pbr_data.roughness);
    vec3 IBL_k_d = vec3(1.0) - IBL_k_s;
    IBL_k_d *= 1.0 - pbr_data.metalness;
    vec3 irradiance = texture(irradiance_map, pbr_data.N).rgb;
    vec3 diffuse = irradiance * pbr_data.albedo;

    vec3 prefiltered_color = textureLod(prefiltered_map, pbr_data.R,
                                        pbr_data.roughness * MAX_REFLECTION_LOD)
                                 .rgb;
    vec2 environment_BRDF =
        texture(brdf_integration, vec2((pbr_data.NdotV), pbr_data.roughness))
            .rg;
    vec3 specular =
        prefiltered_color * (IBL_k_s * environment_BRDF.x + environment_BRDF.y);

    vec3 ambient = (IBL_k_d * diffuse + specular) * pbr_data.occlusion;

    final_color = ambient + final_color;
    final_color = final_color / (final_color + vec3(1.0));
    final_color = pow(final_color, vec3(1.0 / 2.2));
    color = vec4(final_color, 1.0);
    // color = vec4(textureLod(prefiltered_map, pbr_data.R, pbr_data.roughness * MAX_REFLECTION_LOD).rgb, 1.0);
    // color = vec4(texture(brdf_integration, frag_texture_coord).rg, 0.0, 1.0);
    // color = vec4(specular, 1.0);
    // color = texture(normal_map, frag_texture_coord);
    // color = vec4(pbr_data.N, 1.0);
    // color = legacyshader();
    // color = vec4(pbr_data.albedo, 1.0);
    // color = vec4(vec3(pbr_data.roughness), 1.0);
    // color = vec4(vec3(pbr_data.metalness), 1.0);
}
