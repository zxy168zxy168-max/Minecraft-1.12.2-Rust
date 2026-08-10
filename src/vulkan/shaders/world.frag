#version 450

layout(set = 0, binding = 0) uniform sampler2D block_atlas;
layout(set = 0, binding = 1) uniform sampler2D lightmap_texture;

layout(location = 0) in vec2 vertex_uv;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in float vertex_fog_distance;
layout(location = 3) in vec2 vertex_lightmap;

layout(push_constant) uniform WorldPushConstants {
    mat4 view_projection;
    vec4 camera_position;
    vec4 fog_color;
    vec4 fog_parameters;
    vec4 lightmap_parameters;
} world;

layout(location = 0) out vec4 fragment_color;

vec3 sample_vanilla_lightmap(vec2 levels) {
    // MCP EntityRenderer#updateLightmap writes a 16 x 16 DynamicTexture.
    // Integer light levels address texel centres and the LINEAR sampler
    // provides the same interpolation that vanilla's texture unit performs.
    vec2 lightmap_uv = (clamp(levels, vec2(0.0), vec2(15.0)) + vec2(0.5)) / 16.0;
    return texture(lightmap_texture, lightmap_uv).rgb;
}

void main() {
    // RenderGlobal.drawSelectionBox disables texturing and supplies the
    // BufferBuilder POSITION_COLOR stream directly. The negative sentinel is
    // outside terrain alpha-test values and leaves normal world passes intact.
    if (world.fog_parameters.w <= -2.0) {
        float fog_start = world.fog_parameters.x;
        float fog_end = max(world.fog_parameters.y, fog_start + 0.001);
        float fog = clamp((vertex_fog_distance - fog_start) / (fog_end - fog_start), 0.0, 1.0);
        fragment_color = vec4(mix(vertex_color.rgb, world.fog_color.rgb, fog), vertex_color.a);
        return;
    }

    vec2 sampled_uv = vertex_uv;
    vec4 sampled_vertex_color = vertex_color;
    if (sampled_vertex_color.a < 0.0) {
        // CPU vertices encode TextureMap fire animation without adding a
        // second vertex format: layer 0 uses -(alpha), layer 1 uses
        // -(2 + alpha). camera_position.w/fog_color.w contain the active
        // layer-specific V offsets resolved from AnimationMetadataSection.
        float code = -sampled_vertex_color.a;
        bool layer_one = code > 2.0;
        sampled_vertex_color.a = layer_one ? code - 2.0 : code;
        sampled_uv.y += layer_one ? world.fog_color.w : world.camera_position.w;
    }

    vec4 texture_color = texture(block_atlas, sampled_uv);
    float alpha = texture_color.a * sampled_vertex_color.a;
    float alpha_cutoff = world.fog_parameters.w;
    if (alpha_cutoff >= 0.0 && alpha <= alpha_cutoff) {
        discard;
    }

    // RenderGlobal draws the destroy-stage overlay while the lightmap texture
    // unit is disabled, but world fog remains active. The Vulkan renderer uses
    // dimension sentinel 98 only for that unlit, fogged pass.
    if (world.lightmap_parameters.w > 97.5 && world.lightmap_parameters.w < 98.5) {
        float fog_start = world.fog_parameters.x;
        float fog_end = max(world.fog_parameters.y, fog_start + 0.001);
        float fog = clamp((vertex_fog_distance - fog_start) / (fog_end - fog_start), 0.0, 1.0);
        vec3 unlit = texture_color.rgb * sampled_vertex_color.rgb;
        fragment_color = vec4(mix(unlit, world.fog_color.rgb, fog), alpha);
        return;
    }

    // GUI quads share the world atlas but must bypass lightmap and fog. The
    // sentinel is outside all vanilla dimension IDs and is only supplied by
    // the orthographic HUD pass.
    if (world.lightmap_parameters.w > 10.0) {
        fragment_color = vec4(texture_color.rgb * sampled_vertex_color.rgb, alpha);
        return;
    }

    float fog_start = world.fog_parameters.x;
    float fog_end = max(world.fog_parameters.y, fog_start + 0.001);
    float fog = clamp((vertex_fog_distance - fog_start) / (fog_end - fog_start), 0.0, 1.0);

    // Living-entity model vertices use block-light + 16 as a per-vertex
    // sentinel for MCP `RenderLivingBase#setBrightness`'s hurt/death red
    // combiner. Non-combining layers and held items retain ordinary light.
    // Strip the sentinel before lightmap sampling, then reproduce the fixed
    // function order: interpolate base texture toward constant red by 0.3,
    // and finally modulate by the vanilla lightmap.
    bool hurt_overlay = vertex_lightmap.x > 15.5;
    vec2 light_levels = vertex_lightmap;
    if (hurt_overlay) {
        light_levels.x -= 16.0;
    }
    vec3 lightmap = sample_vanilla_lightmap(light_levels);
    vec3 base_color = texture_color.rgb * sampled_vertex_color.rgb;
    if (hurt_overlay) {
        base_color = mix(base_color, vec3(1.0, 0.0, 0.0), 0.3);
    }
    vec3 lit = base_color * lightmap;
    fragment_color = vec4(mix(lit, world.fog_color.rgb, fog), alpha);
}
