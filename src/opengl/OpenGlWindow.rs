use std::collections::{BTreeMap, BTreeSet, HashMap};
use rustc_hash::{FxHashMap, FxHashSet};
use std::ffi::{CStr, CString};
use std::num::NonZeroU32;
use std::os::raw::c_void;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context};
use gl::types::{GLchar, GLenum, GLint, GLsizei, GLsizeiptr, GLuint};
use glutin::config::{Config, ConfigTemplateBuilder, GlConfig};
use glutin::context::{
    ContextApi, ContextAttributesBuilder, GlProfile, PossiblyCurrentContext, Version,
};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::prelude::*;
use glutin::surface::{Surface, SwapInterval, WindowSurface};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::net::minecraft::client::renderer::EntityRenderer::EntityRenderer;
use crate::net::minecraft::client::renderer::chunk::RenderChunk::RenderChunkKey;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use self::BlendMode::{
    Additive, Alpha, BlockDamage, Disabled, Glint, InvertCrosshair, SourceAlphaAdditive,
    TntFlash,
};
use crate::renderer::DesktopRenderer::RendererExtent;
use crate::opengl::OptifineShaderRuntime::{
    GbufferDrawState, GbufferProgram, OptifineShaderRuntime,
};
use crate::vulkan::CpuFrame::CpuFrame;
use crate::vulkan::GuiCompiler::{CompiledGuiStep, GuiBatch, VulkanGuiVertex};
use crate::vulkan::GuiDrawList::GuiTopology;
use crate::vulkan::GuiRenderFrame::GuiRenderFrame;
use crate::vulkan::PanoramaRenderer::{PanoramaCompositeVertex, PanoramaPassPlan};
use crate::vulkan::TextureSource::TextureSource;
use crate::vulkan::VulkanWorldRenderer::{
    ChunkLayerRange, EntityOverlayPipelineKind, FirstPersonPipelineKind, HudPipelineKind,
    WorldEntityDrawRange, WorldEntityMeshKind, WorldEntityPipelineKind, WorldPushConstants,
    WorldRenderFrame, WorldVertex,
};

const WORLD_VERTEX_SHADER: &str = r#"#version 330 compatibility
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_color;
layout(location = 3) in vec2 in_lightmap;

uniform mat4 view_projection;
uniform vec4 camera_position;
uniform vec4 fog_color;
uniform vec4 fog_parameters;
uniform vec4 lightmap_parameters;

out vec2 vertex_uv;
out vec4 vertex_color;
out float vertex_fog_distance;
out vec2 vertex_lightmap;

void main() {
    vec4 clip = view_projection * vec4(in_position, 1.0);
    // WorldRenderFrame matrices use Vulkan's Y direction and 0..1 depth.
    // Convert only at this backend boundary; all MCP camera math remains shared.
    clip.y = -clip.y;
    clip.z = clip.z * 2.0 - clip.w;
    gl_Position = clip;
    vertex_uv = in_uv;
    vertex_color = in_color;
    vertex_fog_distance = distance(in_position, camera_position.xyz);
    vertex_lightmap = in_lightmap;
}
"#;

const WORLD_FRAGMENT_SHADER: &str = r#"#version 330 compatibility
uniform sampler2D block_atlas;
uniform sampler2D lightmap_texture;
uniform vec4 camera_position;
uniform vec4 fog_color;
uniform vec4 fog_parameters;
uniform vec4 lightmap_parameters;

in vec2 vertex_uv;
in vec4 vertex_color;
in float vertex_fog_distance;
in vec2 vertex_lightmap;
out vec4 fragment_color;

vec3 sample_vanilla_lightmap(vec2 levels) {
    // MCP EntityRenderer updates a 16 x 16 DynamicTexture and samples it
    // linearly. Integer light levels therefore address texel centres while
    // fractional values retain the exact bilinear interpolation semantics.
    vec2 lightmap_uv = (clamp(levels, vec2(0.0), vec2(15.0)) + vec2(0.5)) / 16.0;
    return texture(lightmap_texture, lightmap_uv).rgb;
}

void main() {
    if (fog_parameters.w <= -2.0) {
        float fog_start = fog_parameters.x;
        float fog_end = max(fog_parameters.y, fog_start + 0.001);
        float fog = clamp((vertex_fog_distance - fog_start) / (fog_end - fog_start), 0.0, 1.0);
        fragment_color = vec4(mix(vertex_color.rgb, fog_color.rgb, fog), vertex_color.a);
        return;
    }
    vec2 sampled_uv = vertex_uv;
    vec4 sampled_vertex_color = vertex_color;
    if (sampled_vertex_color.a < 0.0) {
        float code = -sampled_vertex_color.a;
        bool layer_one = code > 2.0;
        sampled_vertex_color.a = layer_one ? code - 2.0 : code;
        sampled_uv.y += layer_one ? fog_color.w : camera_position.w;
    }
    vec4 texture_color = texture(block_atlas, sampled_uv);
    float alpha = texture_color.a * sampled_vertex_color.a;
    float alpha_cutoff = fog_parameters.w;
    if (alpha_cutoff >= 0.0 && alpha <= alpha_cutoff) discard;
    if (lightmap_parameters.w > 97.5 && lightmap_parameters.w < 98.5) {
        float fog_start = fog_parameters.x;
        float fog_end = max(fog_parameters.y, fog_start + 0.001);
        float fog = clamp((vertex_fog_distance - fog_start) / (fog_end - fog_start), 0.0, 1.0);
        fragment_color = vec4(mix(texture_color.rgb * sampled_vertex_color.rgb, fog_color.rgb, fog), alpha);
        return;
    }
    if (lightmap_parameters.w > 10.0) {
        fragment_color = vec4(texture_color.rgb * sampled_vertex_color.rgb, alpha);
        return;
    }
    float fog_start = fog_parameters.x;
    float fog_end = max(fog_parameters.y, fog_start + 0.001);
    float fog = clamp((vertex_fog_distance - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    bool hurt_overlay = vertex_lightmap.x > 15.5;
    vec2 light_levels = vertex_lightmap;
    if (hurt_overlay) light_levels.x -= 16.0;
    vec3 lightmap = sample_vanilla_lightmap(light_levels);
    vec3 base_color = texture_color.rgb * sampled_vertex_color.rgb;
    if (hurt_overlay) base_color = mix(base_color, vec3(1.0, 0.0, 0.0), 0.3);
    fragment_color = vec4(mix(base_color * lightmap, fog_color.rgb, fog), alpha);
}
"#;

const GUI_VERTEX_SHADER: &str = r#"#version 330 compatibility
layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_uv;
out vec2 vertex_uv;
void main() { gl_Position = vec4(in_position, 0.0, 1.0); vertex_uv = in_uv; }
"#;
const GUI_FRAGMENT_SHADER: &str = r#"#version 330 compatibility
uniform sampler2D gui_texture;
in vec2 vertex_uv;
out vec4 fragment_color;
void main() { fragment_color = texture(gui_texture, vertex_uv); }
"#;

const NATIVE_GUI_VERTEX_SHADER: &str = r#"#version 330 compatibility
layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_uv;
layout(location = 2) in vec4 in_color;
uniform vec2 gui_size;
out vec2 vertex_uv;
out vec4 vertex_color;
void main() {
    vec2 safe_size = max(gui_size, vec2(1.0));
    vec2 ndc = vec2(in_position.x / safe_size.x * 2.0 - 1.0,
                    1.0 - in_position.y / safe_size.y * 2.0);
    gl_Position = vec4(ndc, 0.0, 1.0);
    vertex_uv = in_uv;
    vertex_color = in_color;
}
"#;

const NATIVE_GUI_FRAGMENT_SHADER: &str = r#"#version 330 compatibility
uniform sampler2D gui_texture;
uniform int use_texture;
in vec2 vertex_uv;
in vec4 vertex_color;
out vec4 fragment_color;
void main() {
    vec4 sampled = use_texture != 0 ? texture(gui_texture, vertex_uv) : vec4(1.0);
    vec4 color = sampled * vertex_color;
    // Gui.drawGradientRect disables alpha testing while textured GUI quads
    // use the ordinary 0.1 threshold. Applying one unconditional discard to
    // both paths truncated the main-menu gradient and diverged from 1.12.2.
    if (use_texture != 0 && color.a <= 0.1) discard;
    fragment_color = color;
}
"#;

const PANORAMA_VERTEX_SHADER: &str = r#"#version 330 compatibility
layout(location = 0) in vec2 in_position;
out vec2 vertex_uv;
void main() {
    gl_Position = vec4(in_position, 0.0, 1.0);
    vertex_uv = in_position * 0.5 + 0.5;
}
"#;

const PANORAMA_FRAGMENT_SHADER: &str = r#"#version 330 compatibility
uniform sampler2D panorama_face_0;
uniform sampler2D panorama_face_1;
uniform sampler2D panorama_face_2;
uniform sampler2D panorama_face_3;
uniform sampler2D panorama_face_4;
uniform sampler2D panorama_face_5;
uniform float panorama_timer;
uniform int sample_count;
in vec2 vertex_uv;
out vec4 fragment_color;

vec4 sample_face(int face, vec2 uv) {
    if (face == 0) return texture(panorama_face_0, uv);
    if (face == 1) return texture(panorama_face_1, uv);
    if (face == 2) return texture(panorama_face_2, uv);
    if (face == 3) return texture(panorama_face_3, uv);
    if (face == 4) return texture(panorama_face_4, uv);
    return texture(panorama_face_5, uv);
}

vec3 rotate_inverse(vec3 value, float sin_pitch, float cos_pitch, float sin_yaw, float cos_yaw) {
    float x1 = value.x;
    float y1 = cos_pitch * value.y + sin_pitch * value.z;
    float z1 = -sin_pitch * value.y + cos_pitch * value.z;
    return vec3(cos_yaw * x1 - sin_yaw * z1,
                y1,
                sin_yaw * x1 + cos_yaw * z1);
}

vec3 intersect_cube(vec3 origin, vec3 direction, out int face, out vec2 uv) {
    float distance_value = 1e20;
    int axis = 2;
    for (int candidate = 0; candidate < 3; ++candidate) {
        float component = direction[candidate];
        if (abs(component) < 1e-8) continue;
        float boundary = component >= 0.0 ? 1.0 : -1.0;
        float value = (boundary - origin[candidate]) / component;
        if (value > 0.0 && value < distance_value) {
            distance_value = value;
            axis = candidate;
        }
    }
    vec3 point = origin + direction * distance_value;
    vec2 local;
    if (axis == 0 && point.x >= 0.0) { face = 1; local = vec2(-point.z, point.y); }
    else if (axis == 0) { face = 3; local = vec2(point.z, point.y); }
    else if (axis == 1 && point.y >= 0.0) { face = 5; local = vec2(point.x, -point.z); }
    else if (axis == 1) { face = 4; local = vec2(point.x, point.z); }
    else if (point.z >= 0.0) { face = 0; local = vec2(point.x, point.y); }
    else { face = 2; local = vec2(-point.x, point.y); }
    uv = (local + vec2(1.0)) * 0.5;
    return point;
}

void main() {
    float tangent = tan(radians(60.0));
    // OpenGL framebuffer row zero is the lower row, while NativeImage and
    // the existing CPU reference treat v=0 as the upper row. Flip only the
    // cube-ray lookup here; blur/composite texture coordinates remain in the
    // project's top-left resource convention.
    vec2 panorama_uv = vec2(vertex_uv.x, 1.0 - vertex_uv.y);
    vec2 normalized = panorama_uv * 2.0 - 1.0;
    vec3 base_ray = vec3(-normalized.y * tangent, -normalized.x * tangent, 1.0);
    float pitch = radians(sin(panorama_timer / 400.0) * 25.0 + 20.0);
    float yaw = radians(-panorama_timer * 0.1);
    float sin_pitch = sin(pitch);
    float cos_pitch = cos(pitch);
    float sin_yaw = sin(yaw);
    float cos_yaw = cos(yaw);
    vec3 accumulated = vec3(0.0);
    int count = clamp(sample_count, 0, 64);
    for (int k = 0; k < 64; ++k) {
        if (k >= count) break;
        float translate_x = ((float(k % 8) / 8.0) - 0.5) / 64.0;
        float translate_y = ((float(k / 8) / 8.0) - 0.5) / 64.0;
        vec3 translated = rotate_inverse(vec3(-translate_x, -translate_y, 0.0),
                                         sin_pitch, cos_pitch, sin_yaw, cos_yaw);
        vec3 direction = rotate_inverse(base_ray, sin_pitch, cos_pitch, sin_yaw, cos_yaw);
        int face;
        vec2 uv;
        intersect_cube(translated, direction, face, uv);
        float alpha = float(255 / (k + 1)) / 255.0;
        vec3 sampled = sample_face(face, uv).rgb;
        accumulated = sampled * alpha + accumulated * (1.0 - alpha);
    }
    fragment_color = vec4(accumulated, 1.0);
}
"#;

const PANORAMA_BLUR_FRAGMENT_SHADER: &str = r#"#version 330 compatibility
uniform sampler2D panorama_source;
uniform int layer_count;
in vec2 vertex_uv;
out vec4 fragment_color;
void main() {
    vec4 color = texture(panorama_source, vertex_uv);
    int count = clamp(layer_count, 0, 16);
    for (int layer = 0; layer < 16; ++layer) {
        if (layer >= count) break;
        float alpha = 1.0 / float(layer + 1);
        float offset = (float(layer) - 1.0) / 256.0;
        vec4 sampled = texture(panorama_source, vec2(vertex_uv.x + offset, vertex_uv.y));
        color.rgb = sampled.rgb * alpha + color.rgb * (1.0 - alpha);
    }
    fragment_color = vec4(color.rgb, 1.0);
}
"#;


#[derive(Clone, Copy)]
enum BlendMode {
    Disabled,
    Alpha,
    InvertCrosshair,
    Glint,
    BlockDamage,
    TntFlash,
    Additive,
    SourceAlphaAdditive,
}

struct ProgramUniforms {
    viewProjection: GLint,
    cameraPosition: GLint,
    fogColor: GLint,
    fogParameters: GLint,
    lightmapParameters: GLint,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct GlShaderVertex {
    position: [f32; 3],
    uv: [f32; 2],
    color: [f32; 4],
    lightmap: [f32; 2],
    normal: [f32; 3],
    shaderEntity: [i16; 3],
    shaderPadding: i16,
    midTexCoord: [f32; 2],
    /// Exact `SVertexBuilder` storage contract: four signed 16-bit values,
    /// exposed with `normalized = false` so GLSL receives the -32767..32767
    /// range used by OptiFine 1.12.2 shader packs.
    tangent: [i16; 4],
}

impl GlShaderVertex {
    const STRIDE: GLsizei = std::mem::size_of::<Self>() as GLsizei;
}

#[derive(Clone, Copy, Default)]
struct ShaderVertexAccum {
    normal: [f32; 3],
    tangent: [f32; 3],
    bitangent: [f32; 3],
    midTexCoord: [f32; 2],
    contributions: u32,
}

#[derive(Default)]
struct GlShaderBuildScratch {
    accumulators: Vec<ShaderVertexAccum>,
    vertices: Vec<GlShaderVertex>,
}

struct GlMesh {
    vao: GLuint,
    vertexBuffer: GLuint,
    indexBuffer: GLuint,
    indexCount: u32,
    vertexCapacity: usize,
    indexCapacity: usize,
    contentHash: Option<u64>,
    /// Monotonic dynamic CPU mesh generation. Static render-region uploads use
    /// the content hash; per-frame streams use this cheaper generation test.
    contentGeneration: Option<u64>,
    shaderAttributes: bool,
}

fn gl_mesh_content_hash(
    vertices: &[WorldVertex],
    indices: &[u32],
    topology: GLenum,
    shaderAttributes: bool,
) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    let vertexBytes = unsafe {
        std::slice::from_raw_parts(
            vertices.as_ptr().cast::<u8>(),
            std::mem::size_of_val(vertices),
        )
    };
    let indexBytes = unsafe {
        std::slice::from_raw_parts(
            indices.as_ptr().cast::<u8>(),
            std::mem::size_of_val(indices),
        )
    };
    for byte in (topology as u64)
        .to_le_bytes()
        .into_iter()
        .chain([u8::from(shaderAttributes)])
        .chain((vertexBytes.len() as u64).to_le_bytes())
        .chain(vertexBytes.iter().copied())
        .chain((indexBytes.len() as u64).to_le_bytes())
        .chain(indexBytes.iter().copied())
    {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

unsafe fn upload_gl_buffer(
    target: GLenum,
    data: *const c_void,
    requiredBytes: usize,
    usage: GLenum,
    capacity: &mut usize,
) {
    if requiredBytes > *capacity {
        *capacity = requiredBytes.next_power_of_two().max(256);
        gl::BufferData(
            target,
            *capacity as GLsizeiptr,
            std::ptr::null(),
            usage,
        );
    } else if usage == gl::DYNAMIC_DRAW {
        // Orphan dynamic storage before updating it. This follows the
        // reference-renderer-style frame-streaming contract and avoids waiting for a
        // previous high-FPS draw which still reads the same GL buffer.
        gl::BufferData(
            target,
            *capacity as GLsizeiptr,
            std::ptr::null(),
            usage,
        );
    }
    gl::BufferSubData(target, 0, requiredBytes as GLsizeiptr, data);
}

impl GlMesh {
    fn new() -> anyhow::Result<Self> {
        let mut vao = 0;
        let mut vertexBuffer = 0;
        let mut indexBuffer = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vertexBuffer);
            gl::GenBuffers(1, &mut indexBuffer);
        }
        anyhow::ensure!(vao != 0 && vertexBuffer != 0 && indexBuffer != 0, "OpenGL buffer allocation failed");
        let mesh = Self {
            vao,
            vertexBuffer,
            indexBuffer,
            indexCount: 0,
            vertexCapacity: 0,
            indexCapacity: 0,
            contentHash: None,
            contentGeneration: None,
            shaderAttributes: false,
        };
        mesh.configureVertexFormat(false);
        Ok(mesh)
    }

    fn configureVertexFormat(&self, shaderAttributes: bool) {
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertexBuffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.indexBuffer);
            let stride = if shaderAttributes {
                GlShaderVertex::STRIDE
            } else {
                WorldVertex::STRIDE as GLsizei
            };
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, 12usize as *const c_void);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, 20usize as *const c_void);
            gl::EnableVertexAttribArray(3);
            gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, stride, 36usize as *const c_void);
            if shaderAttributes {
                gl::EnableVertexAttribArray(4);
                gl::VertexAttribPointer(4, 3, gl::FLOAT, gl::FALSE, stride, 44usize as *const c_void);
                // SVertexBuilder binds mc_Entity as three unnormalised shorts:
                // mapped block id, metadata and EnumBlockRenderType ordinal.
                gl::EnableVertexAttribArray(5);
                gl::VertexAttribPointer(5, 3, gl::SHORT, gl::FALSE, stride, 56usize as *const c_void);
                gl::EnableVertexAttribArray(6);
                gl::VertexAttribPointer(6, 2, gl::FLOAT, gl::FALSE, stride, 64usize as *const c_void);
                gl::EnableVertexAttribArray(7);
                gl::VertexAttribPointer(7, 4, gl::SHORT, gl::FALSE, stride, 72usize as *const c_void);
            } else {
                for attribute in 4..=7 {
                    gl::DisableVertexAttribArray(attribute);
                }
            }
            gl::BindVertexArray(0);
        }
    }

    fn upload(
        &mut self,
        vertices: &[WorldVertex],
        indices: &[u32],
        usage: GLenum,
        topology: GLenum,
        shaderAttributes: bool,
        contentGeneration: Option<u64>,
        cacheUnchanged: bool,
        shaderScratch: &mut GlShaderBuildScratch,
    ) {
        if vertices.is_empty() || indices.is_empty() {
            // Keep allocated storage for reuse, but never submit stale data from
            // the preceding frame when the current MCP layer is empty.
            self.indexCount = 0;
            self.contentHash = None;
            self.contentGeneration = None;
            return;
        }
        self.indexCount = indices.len() as u32;
        let contentHash = contentGeneration.is_none().then(|| {
            cacheUnchanged.then(|| gl_mesh_content_hash(vertices, indices, topology, shaderAttributes))
        }).flatten();
        if self.shaderAttributes == shaderAttributes
            && (contentGeneration.is_some_and(|generation| {
                self.contentGeneration == Some(generation)
            }) || (contentHash.is_some() && self.contentHash == contentHash))
        {
            return;
        }
        if self.shaderAttributes != shaderAttributes {
            self.configureVertexFormat(shaderAttributes);
            self.shaderAttributes = shaderAttributes;
            self.contentHash = None;
            self.contentGeneration = None;
        }

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertexBuffer);
            if shaderAttributes {
                buildShaderVerticesInto(vertices, indices, topology, shaderScratch);
                upload_gl_buffer(
                    gl::ARRAY_BUFFER,
                    shaderScratch.vertices.as_ptr().cast(),
                    std::mem::size_of_val(shaderScratch.vertices.as_slice()),
                    usage,
                    &mut self.vertexCapacity,
                );
            } else {
                upload_gl_buffer(
                    gl::ARRAY_BUFFER,
                    vertices.as_ptr().cast(),
                    std::mem::size_of_val(vertices),
                    usage,
                    &mut self.vertexCapacity,
                );
            }
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.indexBuffer);
            upload_gl_buffer(
                gl::ELEMENT_ARRAY_BUFFER,
                indices.as_ptr().cast(),
                std::mem::size_of_val(indices),
                usage,
                &mut self.indexCapacity,
            );
            gl::BindVertexArray(0);
        }
        self.contentHash = contentHash;
        self.contentGeneration = contentGeneration;
    }

    fn uploadExpandedShaderVertices(
        &mut self,
        vertices: &[GlShaderVertex],
        indices: &[u32],
        usage: GLenum,
        contentGeneration: u64,
    ) {
        if vertices.is_empty() || indices.is_empty() {
            self.indexCount = 0;
            self.contentHash = None;
            self.contentGeneration = None;
            return;
        }
        self.indexCount = indices.len() as u32;
        if self.shaderAttributes && self.contentGeneration == Some(contentGeneration) {
            return;
        }
        if !self.shaderAttributes {
            self.configureVertexFormat(true);
            self.shaderAttributes = true;
            self.contentHash = None;
            self.contentGeneration = None;
        }
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertexBuffer);
            upload_gl_buffer(
                gl::ARRAY_BUFFER,
                vertices.as_ptr().cast(),
                std::mem::size_of_val(vertices),
                usage,
                &mut self.vertexCapacity,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.indexBuffer);
            upload_gl_buffer(
                gl::ELEMENT_ARRAY_BUFFER,
                indices.as_ptr().cast(),
                std::mem::size_of_val(indices),
                usage,
                &mut self.indexCapacity,
            );
            gl::BindVertexArray(0);
        }
        self.contentHash = None;
        self.contentGeneration = Some(contentGeneration);
    }

    /// Update one resident `RenderChunk` vertex span without repacking its
    /// complete 4x4x4 OpenGL RenderRegion. This is valid only while the chunk
    /// keeps the same vertex count, so every neighbouring chunk retains the
    /// exact vertex base assigned by `rebuildRegion`.
    fn updateWorldVertexRange(&mut self, firstVertex: u32, vertices: &[WorldVertex]) {
        if vertices.is_empty() {
            return;
        }
        debug_assert!(!self.shaderAttributes);
        let byteOffset = firstVertex as usize * std::mem::size_of::<WorldVertex>();
        let byteLength = std::mem::size_of_val(vertices);
        debug_assert!(byteOffset.saturating_add(byteLength) <= self.vertexCapacity);
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertexBuffer);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                byteOffset as isize,
                byteLength as GLsizeiptr,
                vertices.as_ptr().cast(),
            );
            gl::BindVertexArray(0);
        }
        self.contentHash = None;
        self.contentGeneration = None;
    }

    /// OptiFine/SVertexBuilder form of `updateWorldVertexRange`. The resident
    /// VBO uses `GlShaderVertex` while shader attributes are enabled, but the
    /// region slot/base contract is identical.
    fn updateShaderVertexRange(&mut self, firstVertex: u32, vertices: &[GlShaderVertex]) {
        if vertices.is_empty() {
            return;
        }
        debug_assert!(self.shaderAttributes);
        let byteOffset = firstVertex as usize * std::mem::size_of::<GlShaderVertex>();
        let byteLength = std::mem::size_of_val(vertices);
        debug_assert!(byteOffset.saturating_add(byteLength) <= self.vertexCapacity);
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertexBuffer);
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                byteOffset as isize,
                byteLength as GLsizeiptr,
                vertices.as_ptr().cast(),
            );
            gl::BindVertexArray(0);
        }
        self.contentHash = None;
        self.contentGeneration = None;
    }

    /// `RenderChunk#resortTransparency` changes only six-index quad groups.
    /// Keep the resident vertex buffer and update the exact region index span.
    fn updateIndexRange(&mut self, firstIndex: u32, indices: &[u32]) {
        if indices.is_empty() {
            return;
        }
        let byteOffset = firstIndex as usize * std::mem::size_of::<u32>();
        let byteLength = std::mem::size_of_val(indices);
        debug_assert!(byteOffset.saturating_add(byteLength) <= self.indexCapacity);
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.indexBuffer);
            gl::BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                byteOffset as isize,
                byteLength as GLsizeiptr,
                indices.as_ptr().cast(),
            );
            gl::BindVertexArray(0);
        }
        self.contentHash = None;
        self.contentGeneration = None;
    }

    fn draw(&self, topology: GLenum, firstIndex: u32, indexCount: u32) {
        if indexCount == 0 { return; }
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::DrawElements(
                topology,
                indexCount as GLsizei,
                gl::UNSIGNED_INT,
                (firstIndex as usize * std::mem::size_of::<u32>()) as *const c_void,
            );
        }
    }

    /// Submit multiple index ranges from one region buffer with a single
    /// OpenGL driver call. `RenderGlobal#renderBlockLayer` still determines
    /// the exact visible chunk set; this only mirrors OptiFine's render-region
    /// batching at the backend boundary.
    fn drawRanges(&self, topology: GLenum, ranges: &[ChunkLayerRange]) -> usize {
        // A region contains at most 4 * 4 * 4 sections. Keep the OpenGL
        // MultiDraw argument vectors on the stack so the hot render-layer path
        // performs no per-region heap allocation.
        const MAX_REGION_RANGES: usize = GlRenderRegionKey::MAX_CHUNKS;
        debug_assert!(ranges.len() <= MAX_REGION_RANGES);
        let mut counts = [0 as GLsizei; MAX_REGION_RANGES];
        let mut offsets = [std::ptr::null::<c_void>(); MAX_REGION_RANGES];
        let mut drawCount = 0usize;
        for range in ranges.iter().copied() {
            if range.indexCount == 0 { continue; }
            if drawCount == MAX_REGION_RANGES { break; }
            counts[drawCount] = range.indexCount as GLsizei;
            offsets[drawCount] =
                (range.firstIndex as usize * std::mem::size_of::<u32>()) as *const c_void;
            drawCount += 1;
        }
        if drawCount == 0 { return 0; }
        unsafe {
            gl::BindVertexArray(self.vao);
            gl::MultiDrawElements(
                topology,
                counts.as_ptr(),
                gl::UNSIGNED_INT,
                offsets.as_ptr(),
                drawCount as GLsizei,
            );
        }
        drawCount
    }

    /// Submit one source-order run of entity ranges from the same mesh/pipeline.
    /// Runs are formed only across consecutive `RenderManager` / TESR entries,
    /// so this reduces OpenGL driver calls without regrouping or reordering any
    /// Minecraft draw boundary.
    fn drawEntityRangeRun(&self, topology: GLenum, ranges: &[WorldEntityDrawRange]) -> (u64, u64) {
        const MAX_MULTI_DRAW_RANGES: usize = 256;
        if ranges.is_empty() || self.indexCount == 0 {
            return (0, 0);
        }
        let mut submitCalls = 0usize;
        let mut logicalRanges = 0usize;
        let mut cursor = 0usize;
        unsafe { gl::BindVertexArray(self.vao); }
        while cursor < ranges.len() {
            let end = (cursor + MAX_MULTI_DRAW_RANGES).min(ranges.len());
            let mut counts = [0 as GLsizei; MAX_MULTI_DRAW_RANGES];
            let mut offsets = [std::ptr::null::<c_void>(); MAX_MULTI_DRAW_RANGES];
            let mut drawCount = 0usize;
            for range in &ranges[cursor..end] {
                if range.indexCount == 0
                    || range.firstIndex.saturating_add(range.indexCount) > self.indexCount
                {
                    continue;
                }
                counts[drawCount] = range.indexCount as GLsizei;
                offsets[drawCount] =
                    (range.firstIndex as usize * std::mem::size_of::<u32>()) as *const c_void;
                drawCount += 1;
            }
            if drawCount > 0 {
                unsafe {
                    gl::MultiDrawElements(
                        topology,
                        counts.as_ptr(),
                        gl::UNSIGNED_INT,
                        offsets.as_ptr(),
                        drawCount as GLsizei,
                    );
                }
                submitCalls += 1;
                logicalRanges += drawCount;
            }
            cursor = end;
        }
        (submitCalls as u64, logicalRanges as u64)
    }

    fn destroy(&mut self) {
        unsafe {
            if self.indexBuffer != 0 { gl::DeleteBuffers(1, &self.indexBuffer); }
            if self.vertexBuffer != 0 { gl::DeleteBuffers(1, &self.vertexBuffer); }
            if self.vao != 0 { gl::DeleteVertexArrays(1, &self.vao); }
        }
        self.vao = 0;
        self.vertexBuffer = 0;
        self.indexBuffer = 0;
    }
}

fn buildShaderVertices(
    vertices: &[WorldVertex],
    indices: &[u32],
    topology: GLenum,
) -> Vec<GlShaderVertex> {
    let mut scratch = GlShaderBuildScratch::default();
    buildShaderVerticesInto(vertices, indices, topology, &mut scratch);
    scratch.vertices
}

fn buildShaderVerticesInto(
    vertices: &[WorldVertex],
    indices: &[u32],
    topology: GLenum,
    scratch: &mut GlShaderBuildScratch,
) {
    // OptiFine SVertexBuilder expansion is allocation-sensitive for animated
    // entity/particle/hand streams. Keep both the accumulator and expanded
    // vertex capacities renderer-owned and overwrite them in place.
    scratch.accumulators.clear();
    scratch
        .accumulators
        .resize(vertices.len(), ShaderVertexAccum::default());

    if topology == gl::TRIANGLES {
        let mut groups = indices.chunks_exact(6);
        for group in &mut groups {
            let mut unique = [0_usize; 4];
            let mut uniqueCount = 0usize;
            for &rawIndex in group {
                let index = rawIndex as usize;
                if !unique[..uniqueCount].contains(&index) && uniqueCount < unique.len() {
                    unique[uniqueCount] = index;
                    uniqueCount += 1;
                }
            }
            if uniqueCount == 4 {
                // Minecraft's BufferBuilder emits quads and the shared backend
                // triangulates them. SVertexBuilder.calcNormal works on the
                // original four vertices, not on two averaged triangle faces.
                let mut sorted = unique;
                sorted.sort_unstable();
                let face = if sorted.windows(2).all(|pair| pair[1] == pair[0] + 1) {
                    sorted
                } else {
                    unique
                };
                accumulateShaderFace(vertices, &face, &mut scratch.accumulators);
            } else {
                for triangle in group.chunks_exact(3) {
                    let face = [triangle[0] as usize, triangle[1] as usize, triangle[2] as usize];
                    accumulateShaderFace(vertices, &face, &mut scratch.accumulators);
                }
            }
        }
        for triangle in groups.remainder().chunks_exact(3) {
            let face = [triangle[0] as usize, triangle[1] as usize, triangle[2] as usize];
            accumulateShaderFace(vertices, &face, &mut scratch.accumulators);
        }
    }

    scratch.vertices.clear();
    if scratch.vertices.capacity() < vertices.len() {
        scratch.vertices.reserve(vertices.len() - scratch.vertices.capacity());
    }
    for (vertex, accumulator) in vertices
        .iter()
        .zip(scratch.accumulators.iter().copied())
    {
        let normal = normalize3(accumulator.normal).unwrap_or([0.0, 1.0, 0.0]);
        let projectedTangent = sub3(
            accumulator.tangent,
            mul3(normal, dot3(normal, accumulator.tangent)),
        );
        let tangent = normalize3(projectedTangent).unwrap_or_else(|| {
            normalize3(cross3([0.0, 0.0, 1.0], normal))
                .or_else(|| normalize3(cross3([0.0, 1.0, 0.0], normal)))
                .unwrap_or([1.0, 0.0, 0.0])
        });
        let handedness = if dot3(cross3(normal, tangent), accumulator.bitangent) < 0.0 {
            -1.0
        } else {
            1.0
        };
        let midTexCoord = if accumulator.contributions > 0 {
            let divisor = accumulator.contributions as f32;
            [
                accumulator.midTexCoord[0] / divisor,
                accumulator.midTexCoord[1] / divisor,
            ]
        } else {
            vertex.uv
        };
        scratch.vertices.push(GlShaderVertex {
            position: vertex.position,
            uv: vertex.uv,
            color: vertex.color,
            lightmap: vertex.lightmap,
            normal,
            shaderEntity: vertex.shaderEntity,
            shaderPadding: vertex.shaderPadding,
            midTexCoord,
            tangent: [
                packShaderShort(tangent[0]),
                packShaderShort(tangent[1]),
                packShaderShort(tangent[2]),
                packShaderShort(handedness),
            ],
        });
    }
}

fn accumulateShaderFace(
    vertices: &[WorldVertex],
    face: &[usize],
    accumulators: &mut [ShaderVertexAccum],
) {
    if face.len() < 3 || face.iter().any(|index| *index >= vertices.len()) {
        return;
    }
    let i0 = face[0];
    let i1 = face[1];
    let i2 = face[2];
    let p0 = vertices[i0].position;
    let p1 = vertices[i1].position;
    let p2 = vertices[i2].position;
    let uv0 = vertices[i0].uv;
    let uv1 = vertices[i1].uv;
    let uv2 = vertices[i2].uv;

    // SVertexBuilder.calcNormal uses diagonal vectors v2-v0 and v3-v1 for a
    // quad. Retain the triangle cross product only for genuinely triangular
    // geometry generated outside BufferBuilder's GL_QUADS path.
    let normal = if face.len() >= 4 {
        cross3(sub3(vertices[face[2]].position, p0), sub3(vertices[face[3]].position, p1))
    } else {
        cross3(sub3(p1, p0), sub3(p2, p0))
    };
    let edge1 = sub3(p1, p0);
    let edge2 = sub3(p2, p0);
    let duv1 = [uv1[0] - uv0[0], uv1[1] - uv0[1]];
    let duv2 = [uv2[0] - uv0[0], uv2[1] - uv0[1]];
    let determinant = duv1[0] * duv2[1] - duv2[0] * duv1[1];
    let (tangent, bitangent) = if determinant.abs() > 1.0e-12 {
        let inverse = determinant.recip();
        (
            [
                (duv2[1] * edge1[0] - duv1[1] * edge2[0]) * inverse,
                (duv2[1] * edge1[1] - duv1[1] * edge2[1]) * inverse,
                (duv2[1] * edge1[2] - duv1[1] * edge2[2]) * inverse,
            ],
            [
                (duv1[0] * edge2[0] - duv2[0] * edge1[0]) * inverse,
                (duv1[0] * edge2[1] - duv2[0] * edge1[1]) * inverse,
                (duv1[0] * edge2[2] - duv2[0] * edge1[2]) * inverse,
            ],
        )
    } else {
        (edge1, edge2)
    };
    let center = face.iter().fold([0.0_f32; 2], |mut result, index| {
        result[0] += vertices[*index].uv[0];
        result[1] += vertices[*index].uv[1];
        result
    });
    let divisor = face.len() as f32;
    let center = [center[0] / divisor, center[1] / divisor];

    for &index in face {
        let accumulator = &mut accumulators[index];
        accumulator.normal = add3(accumulator.normal, normal);
        accumulator.tangent = add3(accumulator.tangent, tangent);
        accumulator.bitangent = add3(accumulator.bitangent, bitangent);
        accumulator.midTexCoord[0] += center[0];
        accumulator.midTexCoord[1] += center[1];
        accumulator.contributions += 1;
    }
}

fn add3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

fn sub3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

fn mul3(value: [f32; 3], scalar: f32) -> [f32; 3] {
    [value[0] * scalar, value[1] * scalar, value[2] * scalar]
}

fn dot3(left: [f32; 3], right: [f32; 3]) -> f32 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}

fn cross3(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn normalize3(value: [f32; 3]) -> Option<[f32; 3]> {
    let lengthSquared = dot3(value, value);
    if lengthSquared <= 1.0e-20 {
        return None;
    }
    Some(mul3(value, lengthSquared.sqrt().recip()))
}

fn packShaderShort(value: f32) -> i16 {
    // `SVertexBuilder.calcNormal` casts after multiplying by 32767. Retain the
    // same finite range while preventing a malformed mesh from overflowing the
    // Rust conversion path.
    (value.clamp(-1.0, 1.0) * 32767.0) as i16
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct GlRenderRegionKey {
    x: i32,
    y: i32,
    z: i32,
}

impl GlRenderRegionKey {
    // Backend-only batching cells. This implementation repacks a region when
    // one member changes, unlike OptiFine's range allocator. A 4x4x4 section
    // cell bounds that repack cost while still collapsing hundreds of vanilla
    // per-RenderChunk submissions into a few dozen region calls. Visibility
    // and translucent ordering remain those produced by RenderGlobal.
    const HORIZONTAL_CHUNKS: i32 = 4;
    const VERTICAL_CHUNKS: i32 = 4;
    const MAX_CHUNKS: usize =
        (Self::HORIZONTAL_CHUNKS * Self::VERTICAL_CHUNKS * Self::HORIZONTAL_CHUNKS) as usize;

    fn fromChunk(key: RenderChunkKey) -> Self {
        Self {
            x: key.x.div_euclid(Self::HORIZONTAL_CHUNKS),
            y: key.y.div_euclid(Self::VERTICAL_CHUNKS),
            z: key.z.div_euclid(Self::HORIZONTAL_CHUNKS),
        }
    }
}

struct GlChunk {
    layerRanges: [ChunkLayerRange; 4],
    meshRevision: u64,
    vertices: Arc<Vec<WorldVertex>>,
    indices: Arc<Vec<u32>>,
    /// OptiFine SVertexBuilder expansion belongs to this immutable chunk
    /// topology. Cache it across RenderRegion repacks so one neighbouring
    /// section update does not recalculate normals/tangents for all 64 slots.
    shaderVertices: Option<Arc<Vec<GlShaderVertex>>>,
    region: GlRenderRegionKey,
}

/// One order-preserving translucent submission run. Vanilla's translucent
/// layer must remain globally far-to-near, so unlike opaque terrain we may not
/// regroup arbitrary chunks by RenderRegion. Consecutive chunks which already
/// belong to the same region can however be submitted with one
/// `glMultiDrawElements` call without changing their draw order.
#[derive(Debug)]
struct GlOrderedRegionRun {
    region: GlRenderRegionKey,
    ranges: Vec<ChunkLayerRange>,
}

struct GlRegion {
    mesh: GlMesh,
    /// Monotonic upload generation. `rebuildRegion` is entered only after the
    /// region was marked dirty, so hashing the complete repacked vertex/index
    /// payload a second time is redundant CPU/memory-bandwidth work.
    uploadGeneration: u64,
    chunkKeys: BTreeSet<RenderChunkKey>,
    chunkLayerRanges: FxHashMap<RenderChunkKey, [ChunkLayerRange; 4]>,
    chunkVertexBases: FxHashMap<RenderChunkKey, u32>,
    chunkIndexBases: FxHashMap<RenderChunkKey, u32>,
    stagingVertices: Vec<WorldVertex>,
    stagingShaderVertices: Vec<GlShaderVertex>,
    stagingIndices: Vec<u32>,
}

struct OpenGlGuiPipeline {
    program: GLuint,
    vao: GLuint,
    vertexBuffer: GLuint,
    texture: GLuint,
    textureSize: RendererExtent,
}

impl OpenGlGuiPipeline {
    fn new() -> anyhow::Result<Self> {
        let program = compileProgram(GUI_VERTEX_SHADER, GUI_FRAGMENT_SHADER)?;
        let vertices: [f32; 16] = [
            -1.0, -1.0, 0.0, 1.0,
             1.0, -1.0, 1.0, 1.0,
            -1.0,  1.0, 0.0, 0.0,
             1.0,  1.0, 1.0, 0.0,
        ];
        let mut vao = 0;
        let mut vertexBuffer = 0;
        let mut texture = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vertexBuffer);
            gl::GenTextures(1, &mut texture);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertexBuffer);
            gl::BufferData(gl::ARRAY_BUFFER, std::mem::size_of_val(&vertices) as GLsizeiptr, vertices.as_ptr().cast(), gl::STATIC_DRAW);
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 16, std::ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 16, 8usize as *const c_void);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
            gl::UseProgram(program);
            gl::Uniform1i(uniformLocation(program, "gui_texture"), 0);
            gl::BindVertexArray(0);
        }
        Ok(Self { program, vao, vertexBuffer, texture, textureSize: RendererExtent::default() })
    }

    fn draw(&mut self, frame: &CpuFrame, extent: RendererExtent) -> anyhow::Result<()> {
        anyhow::ensure!(frame.width() == extent.width && frame.height() == extent.height, "CPU GUI frame does not match OpenGL drawable size");
        unsafe {
            gl::Viewport(0, 0, extent.width as GLsizei, extent.height as GLsizei);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::UseProgram(self.program);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            if self.textureSize != extent {
                gl::TexImage2D(
                    gl::TEXTURE_2D, 0, gl::RGBA8 as GLint,
                    extent.width as GLsizei, extent.height as GLsizei, 0,
                    gl::RGBA, gl::UNSIGNED_BYTE, frame.rgba().as_ptr().cast(),
                );
                self.textureSize = extent;
            } else {
                gl::TexSubImage2D(
                    gl::TEXTURE_2D, 0, 0, 0,
                    extent.width as GLsizei, extent.height as GLsizei,
                    gl::RGBA, gl::UNSIGNED_BYTE, frame.rgba().as_ptr().cast(),
                );
            }
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            gl::BindVertexArray(0);
        }
        Ok(())
    }

    fn destroy(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteBuffers(1, &self.vertexBuffer);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteProgram(self.program);
        }
    }
}


struct GlGuiTexture {
    id: GLuint,
    source: Arc<TextureSource>,
}

struct OpenGlNativeGuiPipeline {
    program: GLuint,
    guiSizeUniform: GLint,
    useTextureUniform: GLint,
    vao: GLuint,
    vertexBuffer: GLuint,
    indexBuffer: GLuint,
    fullscreenVao: GLuint,
    fullscreenBuffer: GLuint,
    panoramaProgram: GLuint,
    panoramaTimerUniform: GLint,
    panoramaSampleCountUniform: GLint,
    blurProgram: GLuint,
    blurLayerCountUniform: GLint,
    panoramaFramebuffer: GLuint,
    panoramaTextures: [GLuint; 2],
    textures: HashMap<ResourceLocation, GlGuiTexture>,
    profileStarted: Instant,
    profileFrames: u64,
    profileDraws: u64,
    profileSubmitNanos: u128,
}

impl OpenGlNativeGuiPipeline {
    fn new() -> anyhow::Result<Self> {
        let program = compileProgram(NATIVE_GUI_VERTEX_SHADER, NATIVE_GUI_FRAGMENT_SHADER)?;
        let panoramaProgram = compileProgram(PANORAMA_VERTEX_SHADER, PANORAMA_FRAGMENT_SHADER)?;
        let blurProgram = compileProgram(PANORAMA_VERTEX_SHADER, PANORAMA_BLUR_FRAGMENT_SHADER)?;
        let mut vao = 0;
        let mut vertexBuffer = 0;
        let mut indexBuffer = 0;
        let mut fullscreenVao = 0;
        let mut fullscreenBuffer = 0;
        let mut panoramaFramebuffer = 0;
        let mut panoramaTextures = [0_u32; 2];
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vertexBuffer);
            gl::GenBuffers(1, &mut indexBuffer);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vertexBuffer);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indexBuffer);
            let stride = std::mem::size_of::<VulkanGuiVertex>() as GLsizei;
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
            gl::EnableVertexAttribArray(1);
            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, stride, 12usize as *const c_void);
            gl::EnableVertexAttribArray(2);
            gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, 20usize as *const c_void);

            let fullscreen: [f32; 8] = [-1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0];
            gl::GenVertexArrays(1, &mut fullscreenVao);
            gl::GenBuffers(1, &mut fullscreenBuffer);
            gl::BindVertexArray(fullscreenVao);
            gl::BindBuffer(gl::ARRAY_BUFFER, fullscreenBuffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(&fullscreen) as GLsizeiptr,
                fullscreen.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 8, std::ptr::null());

            gl::GenFramebuffers(1, &mut panoramaFramebuffer);
            gl::GenTextures(2, panoramaTextures.as_mut_ptr());
            for texture in panoramaTextures {
                gl::BindTexture(gl::TEXTURE_2D, texture);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA8 as GLint,
                    256,
                    256,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    std::ptr::null(),
                );
            }
            gl::BindFramebuffer(gl::FRAMEBUFFER, panoramaFramebuffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                panoramaTextures[0],
                0,
            );
            anyhow::ensure!(
                gl::CheckFramebufferStatus(gl::FRAMEBUFFER) == gl::FRAMEBUFFER_COMPLETE,
                "OpenGL panorama framebuffer is incomplete"
            );
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::UseProgram(program);
            gl::Uniform1i(uniformLocation(program, "gui_texture"), 0);
            gl::UseProgram(panoramaProgram);
            for unit in 0..6 {
                let name = format!("panorama_face_{unit}");
                gl::Uniform1i(uniformLocation(panoramaProgram, &name), unit);
            }
            gl::UseProgram(blurProgram);
            gl::Uniform1i(uniformLocation(blurProgram, "panorama_source"), 0);
            gl::UseProgram(0);
            gl::BindVertexArray(0);
        }
        anyhow::ensure!(
            vao != 0
                && vertexBuffer != 0
                && indexBuffer != 0
                && fullscreenVao != 0
                && fullscreenBuffer != 0
                && panoramaFramebuffer != 0
                && panoramaTextures.iter().all(|texture| *texture != 0),
            "OpenGL native GUI resource allocation failed"
        );
        Ok(Self {
            program,
            guiSizeUniform: uniformLocation(program, "gui_size"),
            useTextureUniform: uniformLocation(program, "use_texture"),
            vao,
            vertexBuffer,
            indexBuffer,
            fullscreenVao,
            fullscreenBuffer,
            panoramaProgram,
            panoramaTimerUniform: uniformLocation(panoramaProgram, "panorama_timer"),
            panoramaSampleCountUniform: uniformLocation(panoramaProgram, "sample_count"),
            blurProgram,
            blurLayerCountUniform: uniformLocation(blurProgram, "layer_count"),
            panoramaFramebuffer,
            panoramaTextures,
            textures: HashMap::new(),
            profileStarted: Instant::now(),
            profileFrames: 0,
            profileDraws: 0,
            profileSubmitNanos: 0,
        })
    }

    fn texture(&mut self, location: &ResourceLocation, source: &Arc<TextureSource>) -> GLuint {
        if let Some(cached) = self.textures.get(location) {
            if Arc::ptr_eq(&cached.source, source) {
                return cached.id;
            }
        }
        if let Some(old) = self.textures.remove(location) {
            unsafe { gl::DeleteTextures(1, &old.id); }
        }
        let mut id = 0;
        unsafe {
            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            let filter = if source.sampling.blur { gl::LINEAR } else { gl::NEAREST } as GLint;
            let wrap = if source.sampling.clamp { gl::CLAMP_TO_EDGE } else { gl::REPEAT } as GLint;
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as GLint,
                source.image.width() as GLsizei,
                source.image.height() as GLsizei,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                source.image.rgba().as_ptr().cast(),
            );
        }
        self.textures.insert(location.clone(), GlGuiTexture { id, source: Arc::clone(source) });
        id
    }

    fn draw(&mut self, frame: &GuiRenderFrame, extent: RendererExtent) -> anyhow::Result<()> {
        let submitStarted = Instant::now();
        anyhow::ensure!(
            frame.outputWidth == extent.width && frame.outputHeight == extent.height,
            "native GUI frame does not match OpenGL drawable size"
        );
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::Viewport(0, 0, extent.width as GLsizei, extent.height as GLsizei);
            gl::Disable(gl::SCISSOR_TEST);
            gl::Disable(gl::DEPTH_TEST);
            gl::DepthMask(gl::FALSE);
            gl::Disable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
            gl::BlendFuncSeparate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ZERO,
            );
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        let mut drawCount = 0_u64;
        for step in &frame.compiled.steps {
            match step {
                CompiledGuiStep::Panorama(plan) => {
                    let panoramaTexture = self.renderPanorama(plan, frame)?;
                    unsafe {
                        gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                        gl::Viewport(0, 0, extent.width as GLsizei, extent.height as GLsizei);
                        // renderPanorama intentionally disables blending for its
                        // off-screen accumulation. GuiMainMenu immediately
                        // resumes the normal Gui blend state for the panorama
                        // composite, gradients, title and controls.
                        gl::Enable(gl::BLEND);
                        gl::BlendFuncSeparate(
                            gl::SRC_ALPHA,
                            gl::ONE_MINUS_SRC_ALPHA,
                            gl::ONE,
                            gl::ZERO,
                        );
                        gl::ActiveTexture(gl::TEXTURE0);
                    }
                    self.drawPanoramaComposite(plan, panoramaTexture, frame.guiWidth, frame.guiHeight);
                    // One panorama draw, one draw for each source blur invocation,
                    // and one final GUI composite.
                    drawCount = drawCount
                        .saturating_add(2)
                        .saturating_add(plan.blur_invocations.len() as u64);
                }
                CompiledGuiStep::Draw(batch) => {
                    let texture = batch.texture.as_ref().and_then(|location| {
                        frame.textures.get(location).map(|source| self.texture(location, source))
                    });
                    self.drawBatch(batch, texture, frame.guiWidth, frame.guiHeight);
                    drawCount += 1;
                }
            }
        }
        unsafe {
            gl::DepthMask(gl::TRUE);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::BindVertexArray(0);
            gl::UseProgram(0);
            gl::ActiveTexture(gl::TEXTURE0);
        }
        self.profileFrames = self.profileFrames.saturating_add(1);
        self.profileDraws = self.profileDraws.saturating_add(drawCount);
        self.profileSubmitNanos = self
            .profileSubmitNanos
            .saturating_add(submitStarted.elapsed().as_nanos());
        let elapsed = self.profileStarted.elapsed();
        if elapsed >= Duration::from_secs(5) {
            log::info!(
                "OpenGL native GUI workload: {:.1} fps, submit={:.3} ms, batches/frame={:.1}, cached_textures={}",
                self.profileFrames as f64 / elapsed.as_secs_f64().max(0.001),
                self.profileSubmitNanos as f64 / self.profileFrames.max(1) as f64 / 1_000_000.0,
                self.profileDraws as f64 / self.profileFrames.max(1) as f64,
                self.textures.len(),
            );
            self.profileStarted = Instant::now();
            self.profileFrames = 0;
            self.profileDraws = 0;
            self.profileSubmitNanos = 0;
        }
        Ok(())
    }

    fn renderPanorama(&mut self, plan: &PanoramaPassPlan, frame: &GuiRenderFrame) -> anyhow::Result<GLuint> {
        let firstSample = plan.samples.first().context("panorama plan has no samples")?;
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.panoramaFramebuffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                self.panoramaTextures[0],
                0,
            );
            gl::Viewport(0, 0, plan.target_width as GLsizei, plan.target_height as GLsizei);
            gl::Disable(gl::DEPTH_TEST);
            gl::DepthMask(gl::FALSE);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::UseProgram(self.panoramaProgram);
            gl::Uniform1f(self.panoramaTimerUniform, inferPanoramaTimer(plan));
            gl::Uniform1i(self.panoramaSampleCountUniform, plan.samples.len().min(64) as GLint);
            for (unit, face) in firstSample.faces.iter().enumerate() {
                let source = frame.textures.get(&face.texture)
                    .with_context(|| format!("missing panorama texture {}", face.texture))?;
                let texture = self.texture(&face.texture, source);
                gl::ActiveTexture(gl::TEXTURE0 + unit as GLenum);
                gl::BindTexture(gl::TEXTURE_2D, texture);
            }
            gl::BindVertexArray(self.fullscreenVao);
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }

        let mut current = 0usize;
        for invocation in &plan.blur_invocations {
            let target = 1 - current;
            unsafe {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    self.panoramaTextures[target],
                    0,
                );
                gl::UseProgram(self.blurProgram);
                gl::Uniform1i(self.blurLayerCountUniform, invocation.layers.len().min(16) as GLint);
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, self.panoramaTextures[current]);
                gl::BindVertexArray(self.fullscreenVao);
                gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            }
            current = target;
        }
        Ok(self.panoramaTextures[current])
    }

    fn drawPanoramaComposite(
        &mut self,
        plan: &PanoramaPassPlan,
        texture: GLuint,
        guiWidth: i32,
        guiHeight: i32,
    ) {
        let vertices = plan.composite.map(|vertex| panoramaCompositeGuiVertex(vertex));
        let batch = GuiBatch {
            texture: None,
            topology: GuiTopology::Quads,
            vertices: vertices.to_vec(),
            indices: vec![0, 1, 2, 0, 2, 3],
        };
        self.drawBatch(&batch, Some(texture), guiWidth, guiHeight);
    }

    fn drawBatch(&mut self, batch: &GuiBatch, texture: Option<GLuint>, guiWidth: i32, guiHeight: i32) {
        if batch.indices.is_empty() || batch.vertices.is_empty() { return; }
        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform2f(self.guiSizeUniform, guiWidth.max(1) as f32, guiHeight.max(1) as f32);
            gl::Uniform1i(self.useTextureUniform, if texture.is_some() { 1 } else { 0 });
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, texture.unwrap_or(0));
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertexBuffer);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(batch.vertices.as_slice()) as GLsizeiptr,
                batch.vertices.as_ptr().cast(),
                gl::STREAM_DRAW,
            );
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.indexBuffer);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                std::mem::size_of_val(batch.indices.as_slice()) as GLsizeiptr,
                batch.indices.as_ptr().cast(),
                gl::STREAM_DRAW,
            );
            gl::DrawElements(
                gl::TRIANGLES,
                batch.indices.len() as GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }

    fn destroy(&mut self) {
        unsafe {
            for texture in self.textures.values() {
                gl::DeleteTextures(1, &texture.id);
            }
            gl::DeleteTextures(2, self.panoramaTextures.as_ptr());
            gl::DeleteFramebuffers(1, &self.panoramaFramebuffer);
            gl::DeleteBuffers(1, &self.fullscreenBuffer);
            gl::DeleteVertexArrays(1, &self.fullscreenVao);
            gl::DeleteBuffers(1, &self.indexBuffer);
            gl::DeleteBuffers(1, &self.vertexBuffer);
            gl::DeleteVertexArrays(1, &self.vao);
            gl::DeleteProgram(self.blurProgram);
            gl::DeleteProgram(self.panoramaProgram);
            gl::DeleteProgram(self.program);
        }
        self.textures.clear();
    }
}

fn inferPanoramaTimer(plan: &PanoramaPassPlan) -> f32 {
    let Some(sample) = plan.samples.first() else { return 0.0; };
    // PanoramaPassPlan already stores the exact animated pitch/yaw. Recovering
    // from yaw avoids duplicating GuiMainMenu state in the backend.
    -sample.yaw_degrees * 10.0
}

fn panoramaCompositeGuiVertex(vertex: PanoramaCompositeVertex) -> VulkanGuiVertex {
    VulkanGuiVertex {
        position: [vertex.x, vertex.y, 0.0],
        uv: [vertex.u, vertex.v],
        color_rgba: [1.0, 1.0, 1.0, 1.0],
    }
}

struct OpenGlWorldPipeline {
    program: GLuint,
    uniforms: ProgramUniforms,
    atlasTexture: GLuint,
    lightmapTexture: GLuint,
    normalTexture: GLuint,
    specularTexture: GLuint,
    atlasRevision: u64,
    lightmapParameters: [f32; 4],
    shaderAttributes: bool,
    chunks: FxHashMap<RenderChunkKey, GlChunk>,
    regions: FxHashMap<GlRenderRegionKey, GlRegion>,
    /// Reused dirty-region set/order storage. Chunk streaming used to allocate
    /// a fresh HashSet and Vec every frame that touched terrain; keeping these
    /// renderer-owned mirrors the persistent scratch discipline used by the
    /// Vulkan upload path without changing RenderChunk invalidation semantics.
    dirtyRegionsScratch: FxHashSet<GlRenderRegionKey>,
    dirtyRegionOrderScratch: Vec<GlRenderRegionKey>,
    /// Shadow camera opaque/cutout plan. One frustum traversal populates all
    /// three Minecraft opaque/cutout layers instead of rescanning the same
    /// shadow-visible RenderChunks once per layer.
    shadowBatchPlan: BTreeMap<GlRenderRegionKey, [Vec<ChunkLayerRange>; 3]>,
    /// Reused OptiFine shadow-camera visibility/order storage. Shader packs
    /// can render a second full world traversal every frame; retaining this
    /// Vec avoids allocating a render-distance-sized key list for that pass.
    shadowVisibleChunksScratch: Vec<RenderChunkKey>,
    /// Cached ordinary-camera RenderRegion ranges for SOLID, CUTOUT_MIPPED and
    /// CUTOUT. One visibility scan populates all three layers.
    terrainBatchPlan: BTreeMap<GlRenderRegionKey, [Vec<ChunkLayerRange>; 3]>,
    /// Exact far-to-near TRANSLUCENT order from RenderGlobal, compressed only
    /// across consecutive chunks which already share one RenderRegion.
    terrainTranslucentPlan: Vec<GlOrderedRegionRun>,
    /// Frontend-computed exact RenderGlobal visible-order signature. Together
    /// with the RenderRegion topology revision this makes steady-state terrain
    /// plan reuse a constant-time decision.
    cachedTerrainVisibleSignature: Option<u64>,
    terrainTopologyRevision: u64,
    cachedTerrainTopologyRevision: u64,
    performanceTerrainPlanRebuilds: u64,
    performanceTerrainPlanReuses: u64,
    shaderBuildScratch: GlShaderBuildScratch,
    entityMesh: GlMesh,
    blockEntityMesh: GlMesh,
    staticEntityMesh: GlMesh,
    entityDepthMesh: GlMesh,
    overlayMesh: GlMesh,
    particleMesh: GlMesh,
    transparentParticleMesh: GlMesh,
    damageMesh: GlMesh,
    selectionMesh: GlMesh,
    firstPersonMesh: GlMesh,
    hudMesh: GlMesh,
    performanceStarted: Instant,
    performanceFrames: u64,
    performanceDraws: u64,
    performanceRanges: u64,
    performanceRegionRebuilds: u64,
    performanceRegionRebuildNanos: u128,
    performanceResidentSpanUpdates: u64,
    performanceResidentSpanBytes: u64,
    performanceDynamicUploadNanos: u128,
    loggedFirstDraw: bool,
}

impl OpenGlWorldPipeline {
    fn new() -> anyhow::Result<Self> {
        let program = compileProgram(WORLD_VERTEX_SHADER, WORLD_FRAGMENT_SHADER)?;
        let uniforms = ProgramUniforms {
            viewProjection: uniformLocation(program, "view_projection"),
            cameraPosition: uniformLocation(program, "camera_position"),
            fogColor: uniformLocation(program, "fog_color"),
            fogParameters: uniformLocation(program, "fog_parameters"),
            lightmapParameters: uniformLocation(program, "lightmap_parameters"),
        };
        let mut textures = [0_u32; 4];
        unsafe {
            gl::GenTextures(textures.len() as GLsizei, textures.as_mut_ptr());
        }
        anyhow::ensure!(textures.iter().all(|texture| *texture != 0), "OpenGL world texture allocation failed");
        let [atlasTexture, lightmapTexture, normalTexture, specularTexture] = textures;
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, atlasTexture);
            configureWorldTexture(gl::NEAREST as GLint, gl::CLAMP_TO_EDGE as GLint);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, lightmapTexture);
            configureWorldTexture(gl::LINEAR as GLint, gl::CLAMP_TO_EDGE as GLint);
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGBA8 as GLint, 16, 16, 0,
                gl::RGBA, gl::UNSIGNED_BYTE, std::ptr::null(),
            );

            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, normalTexture);
            configureWorldTexture(gl::NEAREST as GLint, gl::REPEAT as GLint);
            let normal = [127_u8, 127, 255, 255];
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGBA8 as GLint, 1, 1, 0,
                gl::RGBA, gl::UNSIGNED_BYTE, normal.as_ptr().cast(),
            );

            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, specularTexture);
            configureWorldTexture(gl::NEAREST as GLint, gl::REPEAT as GLint);
            let specular = [0_u8, 0, 0, 0];
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::RGBA8 as GLint, 1, 1, 0,
                gl::RGBA, gl::UNSIGNED_BYTE, specular.as_ptr().cast(),
            );

            gl::ActiveTexture(gl::TEXTURE0);
            gl::UseProgram(program);
            gl::Uniform1i(uniformLocation(program, "block_atlas"), 0);
            gl::Uniform1i(uniformLocation(program, "lightmap_texture"), 1);
        }
        Ok(Self {
            program,
            uniforms,
            atlasTexture,
            lightmapTexture,
            normalTexture,
            specularTexture,
            atlasRevision: u64::MAX,
            lightmapParameters: [f32::NAN; 4],
            shaderAttributes: false,
            chunks: FxHashMap::default(),
            regions: FxHashMap::default(),
            dirtyRegionsScratch: FxHashSet::default(),
            dirtyRegionOrderScratch: Vec::new(),
            shadowBatchPlan: BTreeMap::new(),
            shadowVisibleChunksScratch: Vec::new(),
            terrainBatchPlan: BTreeMap::new(),
            terrainTranslucentPlan: Vec::new(),
            cachedTerrainVisibleSignature: None,
            terrainTopologyRevision: 1,
            cachedTerrainTopologyRevision: 0,
            performanceTerrainPlanRebuilds: 0,
            performanceTerrainPlanReuses: 0,
            shaderBuildScratch: GlShaderBuildScratch::default(),
            entityMesh: GlMesh::new()?,
            blockEntityMesh: GlMesh::new()?,
            staticEntityMesh: GlMesh::new()?,
            entityDepthMesh: GlMesh::new()?,
            overlayMesh: GlMesh::new()?,
            particleMesh: GlMesh::new()?,
            transparentParticleMesh: GlMesh::new()?,
            damageMesh: GlMesh::new()?,
            selectionMesh: GlMesh::new()?,
            firstPersonMesh: GlMesh::new()?,
            hudMesh: GlMesh::new()?,
            performanceStarted: Instant::now(),
            performanceFrames: 0,
            performanceDraws: 0,
            performanceRanges: 0,
            performanceRegionRebuilds: 0,
            performanceRegionRebuildNanos: 0,
            performanceResidentSpanUpdates: 0,
            performanceResidentSpanBytes: 0,
            performanceDynamicUploadNanos: 0,
            loggedFirstDraw: false,
        })
    }

    fn newRegion() -> anyhow::Result<GlRegion> {
        Ok(GlRegion {
            mesh: GlMesh::new()?,
            uploadGeneration: 0,
            chunkKeys: BTreeSet::new(),
            chunkLayerRanges: FxHashMap::default(),
            chunkVertexBases: FxHashMap::default(),
            chunkIndexBases: FxHashMap::default(),
            stagingVertices: Vec::new(),
            stagingShaderVertices: Vec::new(),
            stagingIndices: Vec::new(),
        })
    }

    fn rebuildRegion(&mut self, regionKey: GlRenderRegionKey) -> anyhow::Result<()> {
        let Some(mut region) = self.regions.remove(&regionKey) else {
            return Ok(());
        };
        // Keep explicit region membership instead of scanning every resident
        // RenderChunk for every upload. The previous O(resident_chunks ×
        // dirty_regions) scan was visible as periodic stalls while terrain
        // streamed in, despite only a handful of sections changing.
        region.chunkKeys.retain(|key| {
            self.chunks
                .get(key)
                .is_some_and(|chunk| chunk.region == regionKey)
        });
        if region.chunkKeys.is_empty() {
            region.mesh.destroy();
            return Ok(());
        }

        let vertexCount = region
            .chunkKeys
            .iter()
            .filter_map(|key| self.chunks.get(key))
            .map(|chunk| chunk.vertices.len())
            .sum::<usize>();
        let indexCount = region
            .chunkKeys
            .iter()
            .filter_map(|key| self.chunks.get(key))
            .map(|chunk| chunk.indices.len())
            .sum::<usize>();
        anyhow::ensure!(vertexCount <= u32::MAX as usize, "OpenGL render region exceeds u32 vertex addressing");
        anyhow::ensure!(indexCount <= u32::MAX as usize, "OpenGL render region exceeds u32 index addressing");

        region.stagingVertices.clear();
        region.stagingShaderVertices.clear();
        region.stagingIndices.clear();
        if self.shaderAttributes {
            if region.stagingShaderVertices.capacity() < vertexCount {
                region.stagingShaderVertices.reserve(
                    vertexCount - region.stagingShaderVertices.capacity(),
                );
            }
        } else if region.stagingVertices.capacity() < vertexCount {
            region.stagingVertices.reserve(vertexCount - region.stagingVertices.capacity());
        }
        if region.stagingIndices.capacity() < indexCount {
            region.stagingIndices.reserve(indexCount - region.stagingIndices.capacity());
        }
        region.chunkLayerRanges.clear();
        region.chunkLayerRanges.reserve(region.chunkKeys.len());
        region.chunkVertexBases.clear();
        region.chunkVertexBases.reserve(region.chunkKeys.len());
        region.chunkIndexBases.clear();
        region.chunkIndexBases.reserve(region.chunkKeys.len());

        for key in region.chunkKeys.iter().copied() {
            let chunk = self
                .chunks
                .get(&key)
                .expect("render-region membership references a resident chunk");
            let vertexBase = if self.shaderAttributes {
                region.stagingShaderVertices.len() as u32
            } else {
                region.stagingVertices.len() as u32
            };
            let indexBase = region.stagingIndices.len() as u32;
            region.chunkVertexBases.insert(key, vertexBase);
            region.chunkIndexBases.insert(key, indexBase);
            if self.shaderAttributes {
                let expanded = chunk.shaderVertices.as_ref().context(
                    "OpenGL shader vertex cache missing during RenderRegion rebuild",
                )?;
                anyhow::ensure!(
                    expanded.len() == chunk.vertices.len(),
                    "OpenGL shader vertex cache length diverged for {:?}",
                    key,
                );
                region.stagingShaderVertices.extend_from_slice(expanded.as_slice());
            } else {
                region.stagingVertices.extend_from_slice(chunk.vertices.as_slice());
            }
            for &index in chunk.indices.iter() {
                anyhow::ensure!(
                    index < chunk.vertices.len() as u32,
                    "OpenGL render-region chunk index {} exceeds vertex count {} for {:?}",
                    index,
                    chunk.vertices.len(),
                    key,
                );
                region.stagingIndices.push(index + vertexBase);
            }
            let mut adjusted = [ChunkLayerRange::default(); 4];
            for (layer, range) in chunk.layerRanges.iter().copied().enumerate() {
                let firstIndex = range
                    .firstIndex
                    .checked_add(indexBase)
                    .context("OpenGL render-region layer offset overflow")?;
                adjusted[layer] = ChunkLayerRange {
                    firstIndex,
                    indexCount: range.indexCount,
                };
            }
            region.chunkLayerRanges.insert(key, adjusted);
        }

        region.uploadGeneration = region.uploadGeneration.wrapping_add(1);
        if self.shaderAttributes {
            region.mesh.uploadExpandedShaderVertices(
                region.stagingShaderVertices.as_slice(),
                region.stagingIndices.as_slice(),
                gl::STATIC_DRAW,
                region.uploadGeneration,
            );
        } else {
            region.mesh.upload(
                region.stagingVertices.as_slice(),
                region.stagingIndices.as_slice(),
                gl::STATIC_DRAW,
                gl::TRIANGLES,
                false,
                Some(region.uploadGeneration),
                false,
                &mut self.shaderBuildScratch,
            );
        }
        self.regions.insert(regionKey, region);
        Ok(())
    }

    fn prepareShadowOpaquePlan(&mut self, keys: &[RenderChunkKey]) {
        let regions = &self.regions;
        self.shadowBatchPlan
            .retain(|regionKey, _| regions.contains_key(regionKey));
        for layers in self.shadowBatchPlan.values_mut() {
            for ranges in layers {
                ranges.clear();
            }
        }
        for key in keys.iter().copied() {
            let Some(chunk) = self.chunks.get(&key) else { continue; };
            let Some(region) = self.regions.get(&chunk.region) else { continue; };
            let Some(ranges) = region.chunkLayerRanges.get(&key) else { continue; };
            let layerPlan = self
                .shadowBatchPlan
                .entry(chunk.region)
                .or_insert_with(|| std::array::from_fn(|_| Vec::new()));
            for layer in 0..3 {
                let range = ranges[layer];
                if range.indexCount > 0 {
                    layerPlan[layer].push(range);
                }
            }
        }
    }

    fn drawShadowOpaqueLayer(&self, layer: usize) -> (u64, u64) {
        debug_assert!(layer < 3);
        let mut submitCalls = 0_u64;
        let mut logicalRanges = 0_u64;
        for (regionKey, layers) in &self.shadowBatchPlan {
            let ranges = &layers[layer];
            if ranges.is_empty() { continue; }
            let Some(region) = self.regions.get(regionKey) else { continue; };
            let drawn = region.mesh.drawRanges(gl::TRIANGLES, ranges.as_slice());
            if drawn > 0 {
                submitCalls = submitCalls.saturating_add(1);
                logicalRanges = logicalRanges.saturating_add(drawn as u64);
            }
        }
        (submitCalls, logicalRanges)
    }

    fn drawChunkLayerOrdered<I>(&self, keys: I, layer: usize) -> (u64, u64)
    where
        I: IntoIterator<Item = RenderChunkKey>,
    {
        let mut submitCalls = 0_u64;
        let mut logicalRanges = 0_u64;
        let mut currentRegion = None::<GlRenderRegionKey>;
        let mut ranges = [ChunkLayerRange::default(); GlRenderRegionKey::MAX_CHUNKS];
        let mut rangeCount = 0_usize;

        let flush = |regionKey: Option<GlRenderRegionKey>,
                     ranges: &[ChunkLayerRange],
                     submitCalls: &mut u64,
                     logicalRanges: &mut u64| {
            let Some(regionKey) = regionKey else { return; };
            let Some(region) = self.regions.get(&regionKey) else { return; };
            let drawn = region.mesh.drawRanges(gl::TRIANGLES, ranges);
            if drawn > 0 {
                *submitCalls = (*submitCalls).saturating_add(1);
                *logicalRanges = (*logicalRanges).saturating_add(drawn as u64);
            }
        };

        for key in keys {
            let Some(chunk) = self.chunks.get(&key) else { continue; };
            let Some(region) = self.regions.get(&chunk.region) else { continue; };
            let Some(chunkRanges) = region.chunkLayerRanges.get(&key) else { continue; };
            let range = chunkRanges[layer];
            if range.indexCount == 0 { continue; }

            if currentRegion != Some(chunk.region) || rangeCount == GlRenderRegionKey::MAX_CHUNKS {
                flush(
                    currentRegion,
                    &ranges[..rangeCount],
                    &mut submitCalls,
                    &mut logicalRanges,
                );
                currentRegion = Some(chunk.region);
                rangeCount = 0;
            }
            ranges[rangeCount] = range;
            rangeCount += 1;
        }
        flush(
            currentRegion,
            &ranges[..rangeCount],
            &mut submitCalls,
            &mut logicalRanges,
        );
        (submitCalls, logicalRanges)
    }

    fn ensureTerrainDrawPlan(&mut self, frame: &WorldRenderFrame) {
        if self.cachedTerrainVisibleSignature == Some(frame.visibleChunkOrderSignature)
            && self.cachedTerrainTopologyRevision == self.terrainTopologyRevision
        {
            self.performanceTerrainPlanReuses = self.performanceTerrainPlanReuses.saturating_add(1);
            return;
        }
        self.performanceTerrainPlanRebuilds = self.performanceTerrainPlanRebuilds.saturating_add(1);

        let regions = &self.regions;
        self.terrainBatchPlan
            .retain(|regionKey, _| regions.contains_key(regionKey));
        for layers in self.terrainBatchPlan.values_mut() {
            for ranges in layers {
                ranges.clear();
            }
        }
        self.terrainTranslucentPlan.clear();

        // One exact RenderGlobal visible-chunk scan feeds all four layers. The
        // three opaque/cutout layers keep RenderRegion/MultiDraw batching; the
        // translucent list is reversed afterwards to preserve vanilla order.
        for visible in &frame.visibleChunks {
            let key = visible.key;
            let Some(chunk) = self.chunks.get(&key) else { continue; };
            let Some(region) = self.regions.get(&chunk.region) else { continue; };
            let Some(ranges) = region.chunkLayerRanges.get(&key) else { continue; };
            let layerPlan = self
                .terrainBatchPlan
                .entry(chunk.region)
                .or_insert_with(|| std::array::from_fn(|_| Vec::new()));
            for layer in 0..3 {
                let range = ranges[layer];
                if range.indexCount > 0 {
                    layerPlan[layer].push(range);
                }
            }
            let translucent = ranges[3];
            if translucent.indexCount > 0 {
                if let Some(run) = self.terrainTranslucentPlan.last_mut()
                    .filter(|run| run.region == chunk.region)
                {
                    run.ranges.push(translucent);
                } else {
                    self.terrainTranslucentPlan.push(GlOrderedRegionRun {
                        region: chunk.region,
                        ranges: vec![translucent],
                    });
                }
            }
        }
        // The scan above follows RenderGlobal's prepared front-to-back list.
        // Vanilla draws TRANSLUCENT in the exact reverse order. Reversing both
        // the run list and each run's internal ranges is exactly equivalent to
        // reversing the original flat list; no region is moved across another.
        self.terrainTranslucentPlan.reverse();
        for run in &mut self.terrainTranslucentPlan {
            run.ranges.reverse();
        }
        self.cachedTerrainVisibleSignature = Some(frame.visibleChunkOrderSignature);
        self.cachedTerrainTopologyRevision = self.terrainTopologyRevision;
    }

    fn drawCachedTerrainLayer(&self, layer: usize) -> (u64, u64) {
        debug_assert!(layer < 3);
        let mut submitCalls = 0_u64;
        let mut logicalRanges = 0_u64;
        for (regionKey, layers) in &self.terrainBatchPlan {
            let ranges = &layers[layer];
            if ranges.is_empty() { continue; }
            let Some(region) = self.regions.get(regionKey) else { continue; };
            let drawn = region.mesh.drawRanges(gl::TRIANGLES, ranges.as_slice());
            if drawn > 0 {
                submitCalls = submitCalls.saturating_add(1);
                logicalRanges = logicalRanges.saturating_add(drawn as u64);
            }
        }
        (submitCalls, logicalRanges)
    }

    fn drawCachedTranslucentLayer(&self) -> (u64, u64) {
        let mut submitCalls = 0_u64;
        let mut logicalRanges = 0_u64;
        for run in &self.terrainTranslucentPlan {
            let Some(region) = self.regions.get(&run.region) else { continue; };
            let drawn = region.mesh.drawRanges(gl::TRIANGLES, run.ranges.as_slice());
            if drawn > 0 {
                submitCalls = submitCalls.saturating_add(1);
                logicalRanges = logicalRanges.saturating_add(drawn as u64);
            }
        }
        (submitCalls, logicalRanges)
    }

    fn updateFrameResources(
        &mut self,
        frame: &WorldRenderFrame,
        shaderAttributes: bool,
    ) -> anyhow::Result<()> {
        if self.atlasRevision != frame.atlasRevision {
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, self.atlasTexture);
                gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
                gl::TexImage2D(
                    gl::TEXTURE_2D, 0, gl::RGBA8 as GLint,
                    frame.atlas.width as GLsizei, frame.atlas.height as GLsizei, 0,
                    gl::RGBA, gl::UNSIGNED_BYTE, frame.atlas.rgba.as_ptr().cast(),
                );
            }
            self.atlasRevision = frame.atlasRevision;
        }
        self.updateLightmap(frame.pushConstants.lightmapParameters);

        self.dirtyRegionsScratch.clear();
        if self.shaderAttributes != shaderAttributes {
            self.dirtyRegionsScratch.extend(self.regions.keys().copied());
            self.shaderAttributes = shaderAttributes;
            if shaderAttributes {
                for chunk in self.chunks.values_mut() {
                    if chunk.shaderVertices.is_none() {
                        chunk.shaderVertices = Some(Arc::new(buildShaderVertices(
                            chunk.vertices.as_slice(),
                            chunk.indices.as_slice(),
                            gl::TRIANGLES,
                        )));
                    }
                }
            } else {
                for chunk in self.chunks.values_mut() {
                    chunk.shaderVertices = None;
                }
            }
            log::info!(
                "OpenGL world vertex format switched: optifine_attributes={}, resident_chunks={}, render_regions={}",
                shaderAttributes,
                self.chunks.len(),
                self.regions.len(),
            );
        }

        for key in &frame.removedChunks {
            if let Some(old) = self.chunks.remove(key) {
                if let Some(region) = self.regions.get_mut(&old.region) {
                    region.chunkKeys.remove(key);
                }
                self.dirtyRegionsScratch.insert(old.region);
            }
        }
        for upload in &frame.chunkUploads {
            let replace = self
                .chunks
                .get(&upload.key)
                .map_or(true, |chunk| chunk.meshRevision != upload.meshRevision);
            if !replace { continue; }
            let regionKey = GlRenderRegionKey::fromChunk(upload.key);
            if !self.regions.contains_key(&regionKey) {
                self.regions.insert(regionKey, Self::newRegion()?);
            }

            let canUpdateIndicesOnly = upload.verticesUnchanged
                && !self.dirtyRegionsScratch.contains(&regionKey)
                && self.chunks.get(&upload.key).is_some_and(|chunk| {
                    chunk.region == regionKey
                        && chunk.layerRanges == upload.layerRanges
                        && chunk.indices.len() == upload.indices.len()
                        && Arc::ptr_eq(&chunk.vertices, &upload.vertices)
                });
            if canUpdateIndicesOnly {
                let (vertexBase, indexBase) = self
                    .regions
                    .get(&regionKey)
                    .and_then(|region| {
                        Some((
                            *region.chunkVertexBases.get(&upload.key)?,
                            *region.chunkIndexBases.get(&upload.key)?,
                        ))
                    })
                    .context("OpenGL render-region offsets missing for translucent resort")?;
                let start = indexBase as usize;
                let end = start + upload.indices.len();
                let region = self
                    .regions
                    .get_mut(&regionKey)
                    .context("OpenGL render region disappeared during translucent resort")?;
                anyhow::ensure!(
                    end <= region.stagingIndices.len(),
                    "OpenGL translucent index range exceeds resident region storage"
                );
                // RenderChunk#resortTransparency changes only the translucent
                // quad order. Rewrite the exact resident RenderRegion span in
                // place instead of allocating an adjusted-index Vec and then
                // copying it back into the same staging storage. Visibility,
                // stable far-to-near order and OptiFine pass boundaries are
                // unchanged; this removes one heap allocation/copy from the
                // OpenGL transparency hot path.
                for (destination, source) in region.stagingIndices[start..end]
                    .iter_mut()
                    .zip(upload.indices.iter())
                {
                    *destination = *source + vertexBase;
                }
                let (mesh, stagingIndices) = (&mut region.mesh, &region.stagingIndices);
                mesh.updateIndexRange(indexBase, &stagingIndices[start..end]);
                if let Some(chunk) = self.chunks.get_mut(&upload.key) {
                    chunk.meshRevision = upload.meshRevision;
                    chunk.indices = Arc::clone(&upload.indices);
                }
                continue;
            }

            // OptiFine's VboRegion keeps long-lived region buffers and updates
            // changed ranges instead of rebuilding unrelated RenderChunks. Our
            // region packing has the same stable-offset opportunity whenever a
            // replacement mesh keeps both its vertex and index counts. In that
            // case all neighbour bases remain valid, so update only this chunk's
            // VBO/EBO spans with BufferSubData. If either count changes we fall
            // back to `rebuildRegion`, which preserves the existing conservative
            // allocation/topology path. This is a backend-only optimisation: MCP
            // RenderChunk contents, layer boundaries and draw ordering are not
            // changed.
            let canUpdateChunkInPlace = !self.dirtyRegionsScratch.contains(&regionKey)
                && self.chunks.get(&upload.key).is_some_and(|chunk| {
                    chunk.region == regionKey
                        && chunk.vertices.len() == upload.vertices.len()
                        && chunk.indices.len() == upload.indices.len()
                })
                && self.regions.get(&regionKey).is_some_and(|region| {
                    region.chunkVertexBases.contains_key(&upload.key)
                        && region.chunkIndexBases.contains_key(&upload.key)
                        && region.chunkLayerRanges.contains_key(&upload.key)
                });
            if canUpdateChunkInPlace {
                let (vertexBase, indexBase) = {
                    let region = self.regions.get(&regionKey)
                        .context("OpenGL render region disappeared during resident chunk update")?;
                    (
                        *region.chunkVertexBases.get(&upload.key)
                            .context("OpenGL resident chunk vertex base missing")?,
                        *region.chunkIndexBases.get(&upload.key)
                            .context("OpenGL resident chunk index base missing")?,
                    )
                };
                let vertexStart = vertexBase as usize;
                let vertexEnd = vertexStart + upload.vertices.len();
                let indexStart = indexBase as usize;
                let indexEnd = indexStart + upload.indices.len();

                let shaderVertices = self.shaderAttributes.then(|| {
                    Arc::new(buildShaderVertices(
                        upload.vertices.as_slice(),
                        upload.indices.as_slice(),
                        gl::TRIANGLES,
                    ))
                });

                let region = self.regions.get_mut(&regionKey)
                    .context("OpenGL render region disappeared during resident span update")?;
                anyhow::ensure!(
                    indexEnd <= region.stagingIndices.len(),
                    "OpenGL resident chunk index span exceeds render-region storage"
                );
                if self.shaderAttributes {
                    let expanded = shaderVertices.as_ref()
                        .context("OpenGL shader vertex expansion missing for resident span update")?;
                    anyhow::ensure!(
                        vertexEnd <= region.stagingShaderVertices.len()
                            && expanded.len() == upload.vertices.len(),
                        "OpenGL resident shader vertex span exceeds render-region storage"
                    );
                    let (mesh, stagingShaderVertices) =
                        (&mut region.mesh, &mut region.stagingShaderVertices);
                    stagingShaderVertices[vertexStart..vertexEnd]
                        .copy_from_slice(expanded.as_slice());
                    mesh.updateShaderVertexRange(vertexBase, expanded.as_slice());
                } else {
                    anyhow::ensure!(
                        vertexEnd <= region.stagingVertices.len(),
                        "OpenGL resident vertex span exceeds render-region storage"
                    );
                    let (mesh, stagingVertices) =
                        (&mut region.mesh, &mut region.stagingVertices);
                    stagingVertices[vertexStart..vertexEnd]
                        .copy_from_slice(upload.vertices.as_slice());
                    mesh.updateWorldVertexRange(vertexBase, upload.vertices.as_slice());
                }

                for (destination, source) in region.stagingIndices[indexStart..indexEnd]
                    .iter_mut()
                    .zip(upload.indices.iter().copied())
                {
                    anyhow::ensure!(
                        source < upload.vertices.len() as u32,
                        "OpenGL resident chunk index {} exceeds replacement vertex count {} for {:?}",
                        source, upload.vertices.len(), upload.key,
                    );
                    *destination = source + vertexBase;
                }
                let (mesh, stagingIndices) = (&mut region.mesh, &region.stagingIndices);
                mesh.updateIndexRange(indexBase, &stagingIndices[indexStart..indexEnd]);

                let mut adjusted = [ChunkLayerRange::default(); 4];
                for (layer, range) in upload.layerRanges.iter().copied().enumerate() {
                    adjusted[layer] = ChunkLayerRange {
                        firstIndex: range.firstIndex.checked_add(indexBase)
                            .context("OpenGL resident chunk layer offset overflow")?,
                        indexCount: range.indexCount,
                    };
                }
                region.chunkLayerRanges.insert(upload.key, adjusted);

                if let Some(chunk) = self.chunks.get_mut(&upload.key) {
                    chunk.meshRevision = upload.meshRevision;
                    chunk.layerRanges = upload.layerRanges;
                    chunk.vertices = Arc::clone(&upload.vertices);
                    chunk.indices = Arc::clone(&upload.indices);
                    chunk.shaderVertices = shaderVertices;
                }
                self.performanceResidentSpanUpdates =
                    self.performanceResidentSpanUpdates.saturating_add(1);
                let vertexBytes = if self.shaderAttributes {
                    upload.vertices.len().saturating_mul(std::mem::size_of::<GlShaderVertex>())
                } else {
                    upload.vertices.len().saturating_mul(std::mem::size_of::<WorldVertex>())
                };
                let indexBytes = upload.indices.len().saturating_mul(std::mem::size_of::<u32>());
                self.performanceResidentSpanBytes = self.performanceResidentSpanBytes
                    .saturating_add(vertexBytes.saturating_add(indexBytes) as u64);
                continue;
            }

            let shaderVertices = self.shaderAttributes.then(|| {
                Arc::new(buildShaderVertices(
                    upload.vertices.as_slice(),
                    upload.indices.as_slice(),
                    gl::TRIANGLES,
                ))
            });
            if let Some(old) = self.chunks.insert(upload.key, GlChunk {
                layerRanges: upload.layerRanges,
                meshRevision: upload.meshRevision,
                vertices: Arc::clone(&upload.vertices),
                indices: Arc::clone(&upload.indices),
                shaderVertices,
                region: regionKey,
            }) {
                if old.region != regionKey {
                    if let Some(region) = self.regions.get_mut(&old.region) {
                        region.chunkKeys.remove(&upload.key);
                    }
                    self.dirtyRegionsScratch.insert(old.region);
                }
            }
            self.regions
                .get_mut(&regionKey)
                .expect("OpenGL render region was created before membership update")
                .chunkKeys
                .insert(upload.key);
            self.dirtyRegionsScratch.insert(regionKey);
        }
        self.dirtyRegionOrderScratch.clear();
        self.dirtyRegionOrderScratch
            .extend(self.dirtyRegionsScratch.drain());
        self.dirtyRegionOrderScratch.sort_unstable();
        let regionRebuildCount = self.dirtyRegionOrderScratch.len() as u64;
        let regionRebuildStarted = Instant::now();
        for index in 0..self.dirtyRegionOrderScratch.len() {
            let region = self.dirtyRegionOrderScratch[index];
            self.rebuildRegion(region)?;
            self.terrainTopologyRevision = self.terrainTopologyRevision.wrapping_add(1);
        }
        self.performanceRegionRebuilds = self
            .performanceRegionRebuilds
            .saturating_add(regionRebuildCount);
        self.performanceRegionRebuildNanos = self
            .performanceRegionRebuildNanos
            .saturating_add(regionRebuildStarted.elapsed().as_nanos());

        let dynamicUploadStarted = Instant::now();
        self.entityMesh.upload(
            frame.entityVertices.as_slice(), frame.entityIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, shaderAttributes, Some(frame.entityMeshGeneration), false,
            &mut self.shaderBuildScratch,
        );
        self.blockEntityMesh.upload(
            frame.blockEntityVertices.as_slice(), frame.blockEntityIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, shaderAttributes,
            Some(frame.blockEntityMeshGeneration), false,
            &mut self.shaderBuildScratch,
        );
        self.staticEntityMesh.upload(
            frame.staticEntityVertices.as_slice(), frame.staticEntityIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, shaderAttributes,
            Some(frame.staticEntityMeshGeneration), false,
            &mut self.shaderBuildScratch,
        );
        self.entityDepthMesh.upload(
            frame.entityDepthVertices.as_slice(), frame.entityDepthIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, shaderAttributes, Some(frame.entityDepthMeshGeneration), false,
            &mut self.shaderBuildScratch,
        );
        self.overlayMesh.upload(
            frame.entityOverlayVertices.as_slice(), frame.entityOverlayIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, shaderAttributes, Some(frame.entityOverlayMeshGeneration), false,
            &mut self.shaderBuildScratch,
        );
        self.particleMesh.upload(
            frame.particleVertices.as_slice(), frame.particleIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, shaderAttributes, Some(frame.particleMeshGeneration), false,
            &mut self.shaderBuildScratch,
        );
        self.transparentParticleMesh.upload(
            frame.transparentParticleVertices.as_slice(), frame.transparentParticleIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, shaderAttributes, Some(frame.transparentParticleMeshGeneration), false,
            &mut self.shaderBuildScratch,
        );
        self.damageMesh.upload(
            frame.damageVertices.as_slice(), frame.damageIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, shaderAttributes, Some(frame.damageMeshGeneration), true,
            &mut self.shaderBuildScratch,
        );
        self.selectionMesh.upload(
            frame.selectionVertices.as_slice(), frame.selectionIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::LINE_STRIP, shaderAttributes, Some(frame.selectionMeshGeneration), true,
            &mut self.shaderBuildScratch,
        );
        self.firstPersonMesh.upload(
            frame.firstPersonVertices.as_slice(), frame.firstPersonIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, shaderAttributes, Some(frame.firstPersonMeshGeneration), false,
            &mut self.shaderBuildScratch,
        );
        // GuiIngame is outside Shaders.endRender and never consumes the
        // extended SVertexBuilder attributes.
        self.hudMesh.upload(
            frame.hudVertices.as_slice(), frame.hudIndices.as_slice(),
            gl::DYNAMIC_DRAW, gl::TRIANGLES, false, Some(frame.hudMeshGeneration), true,
            &mut self.shaderBuildScratch,
        );
        self.performanceDynamicUploadNanos = self
            .performanceDynamicUploadNanos
            .saturating_add(dynamicUploadStarted.elapsed().as_nanos());
        Ok(())
    }

    fn updateLightmap(&mut self, parameters: [f32; 4]) {
        if self.lightmapParameters == parameters {
            return;
        }
        let lightmap = EntityRenderer::buildLightmapRgbaFromArray(parameters);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.lightmapTexture);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 4);
            gl::TexSubImage2D(
                gl::TEXTURE_2D, 0, 0, 0, 16, 16,
                gl::RGBA, gl::UNSIGNED_BYTE, lightmap.as_ptr().cast(),
            );
            gl::ActiveTexture(gl::TEXTURE0);
        }
        self.lightmapParameters = parameters;
    }

    fn bindWorldTextures(&self) {
        unsafe {
            for (unit, texture) in [
                (0_u32, self.atlasTexture),
                (1, self.lightmapTexture),
                (2, self.normalTexture),
                (3, self.specularTexture),
            ] {
                gl::ActiveTexture(gl::TEXTURE0 + unit);
                gl::BindTexture(gl::TEXTURE_2D, texture);
            }
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }

    fn bindPassProgram(
        &self,
        shaderRuntime: &mut Option<&mut OptifineShaderRuntime>,
        frame: &WorldRenderFrame,
        extent: RendererExtent,
        program: GbufferProgram,
        constants: &WorldPushConstants,
        entityId: i32,
        blockEntityId: i32,
        entityColor: [f32; 4],
    ) {
        // Vanilla drawScene binds all four world textures once. OptiFine may
        // mutate active texture units/program samplers between passes, so only
        // the shader-pack path conservatively reasserts the bindings here.
        if shaderRuntime.is_some() {
            self.bindWorldTextures();
        }
        if let Some(runtime) = shaderRuntime.as_deref_mut() {
            let mut draw = GbufferDrawState::new(
                program,
                [frame.atlas.width as i32, frame.atlas.height as i32],
                constants.viewProjection,
                constants.fogColor,
                constants.fogParameters,
                constants.lightmapParameters,
            );
            draw.entityId = entityId;
            draw.blockEntityId = blockEntityId;
            draw.entityColor = entityColor;
            if runtime.bindGbufferProgram(draw, &frame.shaderState, extent) {
                return;
            }
            unsafe { gl::DrawBuffer(gl::COLOR_ATTACHMENT0); }
        } else {
            unsafe { gl::DrawBuffer(gl::BACK); }
        }
        self.uploadConstants(constants);
    }

    /// Executes the dedicated OptiFine 1.12.2
    /// `ShadersRender.renderShadowMap` world traversal. This is deliberately
    /// separate from the ordinary G-buffer pass: the shadow framebuffer is
    /// rendered from the sun/moon camera and only terrain, entities, entity
    /// multipass depth, and the optional translucent layer participate.
    fn entityStreamMesh(&self, kind: WorldEntityMeshKind) -> &GlMesh {
        match kind {
            WorldEntityMeshKind::Dynamic => &self.entityMesh,
            WorldEntityMeshKind::BlockEntities => &self.blockEntityMesh,
            WorldEntityMeshKind::StaticEntities => &self.staticEntityMesh,
        }
    }

    fn drawShadowScene(
        &mut self,
        frame: &WorldRenderFrame,
        extent: RendererExtent,
        shaderRuntime: &mut OptifineShaderRuntime,
    ) -> anyhow::Result<()> {
        self.bindWorldTextures();

        let clipping = shaderRuntime.shadowCullingHelper(&frame.shaderState);
        let camera = frame.shaderState.cameraPosition;
        let mut shadowChunks = std::mem::take(&mut self.shadowVisibleChunksScratch);
        shadowChunks.clear();
        shadowChunks.extend(self.chunks.keys().copied().filter(|key| {
            let min = key.minBlock();
            let max = key.maxBlock();
            clipping.isBoxInFrustum(
                min[0] as f64,
                min[1] as f64,
                min[2] as f64,
                max[0] as f64,
                max[1] as f64,
                max[2] as f64,
                camera,
            )
        }));
        shadowChunks.sort_unstable();
        self.prepareShadowOpaquePlan(shadowChunks.as_slice());

        let bindShadow = |runtime: &mut OptifineShaderRuntime,
                          constants: &WorldPushConstants,
                          entityId: i32,
                          blockEntityId: i32,
                          entityColor: [f32; 4]| {
            let mut draw = GbufferDrawState::new(
                GbufferProgram::Terrain,
                [frame.atlas.width as i32, frame.atlas.height as i32],
                constants.viewProjection,
                constants.fogColor,
                constants.fogParameters,
                constants.lightmapParameters,
            );
            draw.entityId = entityId;
            draw.blockEntityId = blockEntityId;
            draw.entityColor = entityColor;
            runtime.bindShadowProgram(draw, &frame.shaderState, extent)
        };

        // SOLID: alpha test disabled, depth/write enabled, culling disabled.
        setDrawState(Disabled, true, true, gl::LEQUAL, false, true, None);
        let mut constants = frame.pushConstants;
        constants.fogParameters[3] = -1.0;
        if bindShadow(shaderRuntime, &constants, -1, -1, [0.0; 4]) {
            self.drawShadowOpaqueLayer(0);
        }

        // CUTOUT_MIPPED and CUTOUT: alpha threshold 0.1. The source switches
        // texture blur/mipmap state between these layers; the shared atlas is
        // already nearest-filtered, so no synthetic alternate filtering is
        // introduced here.
        constants.fogParameters[3] = 0.1;
        for layer in [1usize, 2usize] {
            if !bindShadow(shaderRuntime, &constants, -1, -1, [0.0; 4]) {
                break;
            }
            self.drawShadowOpaqueLayer(layer);
        }

        // RenderGlobal#renderEntities followed by renderMultipass. OptiFine
        // forces every beginEntities/beginBlockEntities request back to
        // program 30 while `isShadowPass` is true.
        if !frame.entityDrawRanges.is_empty()
            && bindShadow(shaderRuntime, &constants, -1, -1, [0.0; 4])
        {
            setDrawState(Alpha, true, true, gl::LEQUAL, false, true, None);
            for range in frame.entityDrawRanges.iter() {
                let mesh = self.entityStreamMesh(range.mesh);
                if range.indexCount > 0
                    && range.firstIndex.saturating_add(range.indexCount) <= mesh.indexCount
                {
                    mesh.draw(gl::TRIANGLES, range.firstIndex, range.indexCount);
                }
            }
        }
        if self.entityDepthMesh.indexCount > 0
            && bindShadow(shaderRuntime, &constants, -1, -1, [0.0; 4])
        {
            setDrawState(Alpha, true, true, gl::LEQUAL, false, false, None);
            self.entityDepthMesh
                .draw(gl::TRIANGLES, 0, self.entityDepthMesh.indexCount);
            unsafe { gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE); }
        }

        shaderRuntime.captureOpaqueShadowDepth();

        if shaderRuntime.shouldRenderShadowTranslucent() {
            // RenderGlobal sorts translucent chunks far-to-near for pass 2.
            shadowChunks.sort_unstable_by(|left, right| {
                let leftMin = left.minBlock();
                let rightMin = right.minBlock();
                let distance = |min: [i32; 3]| {
                    let center = [
                        min[0] as f32 + 8.0,
                        min[1] as f32 + 8.0,
                        min[2] as f32 + 8.0,
                    ];
                    let dx = center[0] - camera[0];
                    let dy = center[1] - camera[1];
                    let dz = center[2] - camera[2];
                    dx * dx + dy * dy + dz * dz
                };
                distance(rightMin).total_cmp(&distance(leftMin))
            });
            setDrawState(Alpha, true, true, gl::LEQUAL, false, true, None);
            if bindShadow(shaderRuntime, &constants, -1, -1, [0.0; 4]) {
                self.drawChunkLayerOrdered(shadowChunks.iter().copied(), 3);
            }
        }

        unsafe {
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::BindVertexArray(0);
            gl::UseProgram(0);
            gl::Flush();
        }
        shadowChunks.clear();
        self.shadowVisibleChunksScratch = shadowChunks;
        Ok(())
    }

    /// MCP `EntityRenderer#renderCloudsCheck` plus
    /// `RenderGlobal#renderCloudsFancy`'s depth-only first pass.
    fn drawCloudPass(
        &self,
        frame: &WorldRenderFrame,
        extent: RendererExtent,
        shaderRuntime: &mut Option<&mut OptifineShaderRuntime>,
    ) -> u64 {
        if frame.cloudIndexCount == 0 || self.overlayMesh.indexCount == 0 {
            return 0;
        }
        let firstIndex = frame.skyAlphaIndexCount + frame.skyCelestialIndexCount;
        if firstIndex.saturating_add(frame.cloudIndexCount) > self.overlayMesh.indexCount {
            return 0;
        }
        self.bindPassProgram(
            shaderRuntime,
            frame,
            extent,
            GbufferProgram::Clouds,
            &frame.cloudPushConstants,
            -3,
            -1,
            [0.0; 4],
        );
        let mut draws = 0;
        if frame.cloudFancy {
            setDrawState(Alpha, true, true, gl::LEQUAL, false, false, None);
            self.overlayMesh
                .draw(gl::TRIANGLES, firstIndex, frame.cloudIndexCount);
            draws += 1;
        }
        setDrawState(Alpha, true, true, gl::LEQUAL, false, true, None);
        self.overlayMesh
            .draw(gl::TRIANGLES, firstIndex, frame.cloudIndexCount);
        unsafe { gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE); }
        draws + 1
    }

    fn drawScene(
        &mut self,
        frame: &WorldRenderFrame,
        extent: RendererExtent,
        mut shaderRuntime: Option<&mut OptifineShaderRuntime>,
    ) -> anyhow::Result<()> {
        let shaderActive = shaderRuntime.is_some();
        let mut drawCount = 0_u64;
        let mut batchedRangeExpansion = 0_u64;
        self.ensureTerrainDrawPlan(frame);
        unsafe {
            gl::Viewport(0, 0, extent.width as GLsizei, extent.height as GLsizei);
            gl::DepthMask(gl::TRUE);
            if !shaderActive {
                gl::ClearColor(
                    frame.clearColor[0],
                    frame.clearColor[1],
                    frame.clearColor[2],
                    frame.clearColor[3],
                );
                gl::ClearDepth(1.0);
                // The previous frame ends in the HUD pass with depth writes
                // disabled. Restore a writable mask before vanilla clears.
                gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            // Active OptiFine rendering is cleared by ShaderTargets::beginScene
            // so `colortexNClear = false` history buffers remain intact.
        }
        self.bindWorldTextures();

        // RenderGlobal sky: Shaders.beginSky uses skybasic while texturing is
        // disabled and skytextured for the celestial textures.
        if self.overlayMesh.indexCount > 0 {
            setDrawState(Alpha, false, false, gl::ALWAYS, false, true, None);
            if frame.skyAlphaIndexCount > 0 {
                self.bindPassProgram(
                    &mut shaderRuntime,
                    frame,
                    extent,
                    GbufferProgram::SkyBasic,
                    &frame.skyPushConstants,
                    -2,
                    -1,
                    [0.0; 4],
                );
                self.overlayMesh.draw(gl::TRIANGLES, 0, frame.skyAlphaIndexCount);
                drawCount += 1;
            }
            if frame.skyCelestialIndexCount > 0 {
                setDrawState(SourceAlphaAdditive, false, false, gl::ALWAYS, false, true, None);
                self.bindPassProgram(
                    &mut shaderRuntime,
                    frame,
                    extent,
                    GbufferProgram::SkyTextured,
                    &frame.skyPushConstants,
                    -2,
                    -1,
                    [0.0; 4],
                );
                self.overlayMesh.draw(
                    gl::TRIANGLES,
                    frame.skyAlphaIndexCount,
                    frame.skyCelestialIndexCount,
                );
                drawCount += 1;
            }
        }

        // EntityRenderer draws the cloud layer before terrain while the eye is
        // below the provider cloud height.
        if !frame.cloudsAboveCamera {
            drawCount += self.drawCloudPass(frame, extent, &mut shaderRuntime);
        }

        // ShadersRender.beginTerrainSolid/CutoutMipped/Cutout all select
        // program 7 (`gbuffers_terrain`) in OptiFine 1.12.2. The three layer
        // names remain loadable for pack compatibility, but are not invented
        // as extra hook points here.
        setDrawState(Disabled, true, true, gl::LEQUAL, true, true, None);
        for (layer, alphaCutoff) in [(0usize, -1.0_f32), (1, 0.1), (2, 0.1)] {
            let mut constants = frame.pushConstants;
            constants.fogParameters[3] = alphaCutoff;
            self.bindPassProgram(
                &mut shaderRuntime,
                frame,
                extent,
                GbufferProgram::Terrain,
                &constants,
                -1,
                -1,
                [0.0; 4],
            );
            let (submitCalls, logicalRanges) = self.drawCachedTerrainLayer(layer);
            drawCount += submitCalls;
            batchedRangeExpansion += logicalRanges.saturating_sub(submitCalls);
        }

        if !frame.entityDrawRanges.is_empty() {
            let mut constants = frame.pushConstants;
            // Preserve exact source order, but collapse only consecutive ranges
            // which already share the same pipeline and resident mesh. This is
            // equivalent to issuing the same DrawElements calls in sequence,
            // while allowing the OpenGL driver to consume them as one MultiDraw.
            let ranges = frame.entityDrawRanges.as_slice();
            let mut runStart = 0usize;
            while runStart < ranges.len() {
                let first = ranges[runStart];
                let mut runEnd = runStart + 1;
                while runEnd < ranges.len()
                    && ranges[runEnd].pipeline == first.pipeline
                    && ranges[runEnd].mesh == first.mesh
                {
                    runEnd += 1;
                }
                let (program, depthTest, depthWrite, depthFunction, textureSentinel) =
                    match first.pipeline {
                        WorldEntityPipelineKind::Entities => {
                            (GbufferProgram::Entities, true, true, gl::LEQUAL, 0.1)
                        }
                        WorldEntityPipelineKind::BlockEntities => {
                            (GbufferProgram::Block, true, true, gl::LEQUAL, 0.1)
                        }
                        WorldEntityPipelineKind::NameplateBackgroundSeeThrough => {
                            (GbufferProgram::Entities, false, false, gl::ALWAYS, -2.0)
                        }
                        WorldEntityPipelineKind::NameplateTextSeeThrough => {
                            (GbufferProgram::Entities, false, false, gl::ALWAYS, 0.1)
                        }
                        WorldEntityPipelineKind::NameplateBackgroundDepthNoWrite => {
                            (GbufferProgram::Entities, true, false, gl::LEQUAL, -2.0)
                        }
                        WorldEntityPipelineKind::NameplateTextDepthWrite => {
                            (GbufferProgram::Entities, true, true, gl::LEQUAL, 0.1)
                        }
                    };
                constants.fogParameters[3] = textureSentinel;
                setDrawState(
                    Alpha,
                    depthTest,
                    depthWrite,
                    depthFunction,
                    false,
                    true,
                    None,
                );
                self.bindPassProgram(
                    &mut shaderRuntime,
                    frame,
                    extent,
                    program,
                    &constants,
                    -1,
                    -1,
                    [0.0; 4],
                );
                let mesh = self.entityStreamMesh(first.mesh);
                let (submitCalls, logicalRanges) =
                    mesh.drawEntityRangeRun(gl::TRIANGLES, &ranges[runStart..runEnd]);
                drawCount += submitCalls;
                batchedRangeExpansion += logicalRanges.saturating_sub(submitCalls);
                runStart = runEnd;
            }
        }

        if self.overlayMesh.indexCount > 0 {
            for range in frame.entityOverlayDrawRanges.iter() {
                let (
                    blend,
                    depthWrite,
                    depthFunction,
                    cull,
                    textureSentinel,
                    unlit,
                    noFog,
                    blackFog,
                    program,
                ) = match range.pipeline {
                    EntityOverlayPipelineKind::ArmorGlint => (
                        Glint,
                        false,
                        gl::EQUAL,
                        true,
                        0.1,
                        true,
                        false,
                        true,
                        GbufferProgram::ArmorGlint,
                    ),
                    EntityOverlayPipelineKind::TntFlash => (
                        TntFlash,
                        true,
                        gl::LEQUAL,
                        true,
                        -2.0,
                        false,
                        false,
                        false,
                        GbufferProgram::Entities,
                    ),
                    EntityOverlayPipelineKind::EndPortalAlpha => (
                        Alpha,
                        true,
                        gl::LEQUAL,
                        false,
                        0.1,
                        true,
                        false,
                        false,
                        GbufferProgram::Block,
                    ),
                    EntityOverlayPipelineKind::EndPortalAdditive => (
                        Additive,
                        false,
                        gl::LEQUAL,
                        false,
                        0.1,
                        true,
                        false,
                        true,
                        GbufferProgram::Block,
                    ),
                    EntityOverlayPipelineKind::BeaconCore => (
                        Disabled,
                        true,
                        gl::LEQUAL,
                        false,
                        0.1,
                        true,
                        true,
                        false,
                        GbufferProgram::BeaconBeam,
                    ),
                    EntityOverlayPipelineKind::BeaconGlow => (
                        Alpha,
                        false,
                        gl::LEQUAL,
                        false,
                        0.1,
                        true,
                        true,
                        false,
                        GbufferProgram::BeaconBeam,
                    ),
                };
                let mut constants = frame.pushConstants;
                constants.fogParameters[3] = textureSentinel;
                if noFog {
                    constants.lightmapParameters[3] = 99.0;
                } else if unlit {
                    constants.lightmapParameters[3] = 98.0;
                }
                if blackFog {
                    constants.fogColor = [0.0, 0.0, 0.0, 1.0];
                }
                self.bindPassProgram(
                    &mut shaderRuntime,
                    frame,
                    extent,
                    program,
                    &constants,
                    -1,
                    -1,
                    [0.0; 4],
                );
                setDrawState(blend, true, depthWrite, depthFunction, cull, true, None);
                self.overlayMesh.draw(gl::TRIANGLES, range.firstIndex, range.indexCount);
                if range.indexCount > 0 { drawCount += 1; }
            }
        }

        if self.entityDepthMesh.indexCount > 0 {
            setDrawState(Alpha, true, true, gl::LEQUAL, false, false, None);
            let mut constants = frame.pushConstants;
            constants.fogParameters[3] = 0.1;
            self.bindPassProgram(
                &mut shaderRuntime,
                frame,
                extent,
                GbufferProgram::Entities,
                &constants,
                -1,
                -1,
                [0.0; 4],
            );
            self.entityDepthMesh.draw(gl::TRIANGLES, 0, self.entityDepthMesh.indexCount);
            drawCount += 1;
        }

        if self.selectionMesh.indexCount > 0 {
            setDrawState(Alpha, true, false, gl::LEQUAL, false, true, None);
            unsafe { gl::LineWidth(2.0); }
            let mut constants = frame.pushConstants;
            constants.fogParameters[3] = -2.0;
            self.bindPassProgram(
                &mut shaderRuntime,
                frame,
                extent,
                GbufferProgram::Basic,
                &constants,
                -1,
                -1,
                [0.0; 4],
            );
            self.selectionMesh.draw(gl::LINE_STRIP, 0, self.selectionMesh.indexCount);
            unsafe { gl::LineWidth(1.0); }
            drawCount += 1;
        }

        if self.damageMesh.indexCount > 0 {
            setDrawState(BlockDamage, true, true, gl::LEQUAL, true, true, Some((-3.0, -3.0)));
            let mut constants = frame.pushConstants;
            constants.fogParameters[3] = 0.1;
            constants.lightmapParameters[3] = 98.0;
            self.bindPassProgram(
                &mut shaderRuntime,
                frame,
                extent,
                GbufferProgram::DamagedBlock,
                &constants,
                -1,
                -1,
                [0.0; 4],
            );
            self.damageMesh.draw(gl::TRIANGLES, 0, self.damageMesh.indexCount);
            drawCount += 1;
        }

        if self.transparentParticleMesh.indexCount > 0 {
            setDrawState(Alpha, true, false, gl::LEQUAL, true, true, None);
            let mut constants = frame.pushConstants;
            constants.fogParameters[3] = 0.003921569;
            self.bindPassProgram(
                &mut shaderRuntime,
                frame,
                extent,
                GbufferProgram::Textured,
                &constants,
                -1,
                -1,
                [0.0; 4],
            );
            self.transparentParticleMesh.draw(
                gl::TRIANGLES,
                0,
                self.transparentParticleMesh.indexCount,
            );
            drawCount += 1;
        }
        if self.particleMesh.indexCount > 0 {
            setDrawState(Alpha, true, true, gl::LEQUAL, true, true, None);
            let mut constants = frame.pushConstants;
            constants.fogParameters[3] = 0.003921569;
            self.bindPassProgram(
                &mut shaderRuntime,
                frame,
                extent,
                GbufferProgram::TexturedLit,
                &constants,
                -1,
                -1,
                [0.0; 4],
            );
            self.particleMesh.draw(gl::TRIANGLES, 0, self.particleMesh.indexCount);
            drawCount += 1;
        }

        // `ShadersRender.preWater` / beginTranslucent copies opaque depth into
        // depthtex1 before translucent terrain can update depthtex0.
        if let Some(runtime) = shaderRuntime.as_deref_mut() {
            runtime.captureDepthBeforeTranslucent();
        }
        // ShadersRender.beginTranslucent selects gbuffers_water and explicitly
        // restores depth writes. Preserve the previous non-shader path while
        // applying that OptiFine state only when the shader runtime is active.
        setDrawState(
            Alpha,
            true,
            shaderRuntime.is_some(),
            gl::LEQUAL,
            true,
            true,
            None,
        );
        let mut translucentConstants = frame.pushConstants;
        translucentConstants.fogParameters[3] = 0.1;
        self.bindPassProgram(
            &mut shaderRuntime,
            frame,
            extent,
            GbufferProgram::Water,
            &translucentConstants,
            -1,
            -1,
            [0.0; 4],
        );
        let (translucentCalls, _) = self.drawCachedTranslucentLayer();
        drawCount += translucentCalls;

        // At or above the cloud layer MCP delays clouds until translucent
        // terrain has completed and before weather/hand rendering.
        if frame.cloudsAboveCamera {
            drawCount += self.drawCloudPass(frame, extent, &mut shaderRuntime);
        }

        // Shaders.beginWeather copies the completed scene depth to depthtex2.
        // There is no separate weather mesh in the shared frame yet, so this
        // source-order boundary is immediately before hand rendering.
        if let Some(runtime) = shaderRuntime.as_deref_mut() {
            runtime.captureDepthBeforeWeather();
        }

        if self.firstPersonMesh.indexCount > 0 {
            unsafe {
                // EntityRenderer#renderWorldPass clears world depth before
                // ItemRenderer#renderItemInFirstPerson. The translucent terrain
                // pass can leave depth writes disabled in the vanilla path.
                gl::DepthMask(gl::TRUE);
                gl::Clear(gl::DEPTH_BUFFER_BIT);
            }
            for range in frame.firstPersonDrawRanges.iter() {
                let program = match range.pipeline {
                    FirstPersonPipelineKind::Alpha => {
                        setDrawState(Alpha, true, true, gl::LEQUAL, true, true, None);
                        GbufferProgram::Hand
                    }
                    FirstPersonPipelineKind::Fire => {
                        setDrawState(Alpha, false, false, gl::ALWAYS, true, true, None);
                        GbufferProgram::Hand
                    }
                    FirstPersonPipelineKind::Glint => {
                        setDrawState(Glint, true, false, gl::EQUAL, true, true, None);
                        GbufferProgram::ArmorGlint
                    }
                };
                self.bindPassProgram(
                    &mut shaderRuntime,
                    frame,
                    extent,
                    program,
                    &frame.firstPersonPushConstants,
                    -1,
                    -1,
                    [0.0; 4],
                );
                self.firstPersonMesh.draw(gl::TRIANGLES, range.firstIndex, range.indexCount);
                if range.indexCount > 0 { drawCount += 1; }
            }
        }

        unsafe {
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
        if drawCount > 0 && !self.loggedFirstDraw {
            log::info!("first OpenGL world draw submitted: {drawCount} layer draws");
            self.loggedFirstDraw = true;
        }
        self.performanceFrames = self.performanceFrames.saturating_add(1);
        self.performanceDraws = self.performanceDraws.saturating_add(drawCount);
        self.performanceRanges = self
            .performanceRanges
            .saturating_add(drawCount.saturating_add(batchedRangeExpansion));
        let elapsed = self.performanceStarted.elapsed();
        if elapsed >= Duration::from_secs(5) {
            let seconds = elapsed.as_secs_f64().max(0.001);
            log::info!(
                "OpenGL world workload: {:.1} fps, visible_chunks={}, submit_calls/frame={:.1}, logical_ranges/frame={:.1}, region_rebuilds/frame={:.2}, region_rebuild={:.3} ms, resident_span_updates/frame={:.2}, resident_span_kib/frame={:.1}, dynamic_upload={:.3} ms, terrain_plan_reuse={:.1}% ({}/{}), resident_chunks={}, render_regions={}",
                self.performanceFrames as f64 / seconds,
                frame.visibleChunks.len(),
                self.performanceDraws as f64 / self.performanceFrames.max(1) as f64,
                self.performanceRanges as f64 / self.performanceFrames.max(1) as f64,
                self.performanceRegionRebuilds as f64 / self.performanceFrames.max(1) as f64,
                self.performanceRegionRebuildNanos as f64 / self.performanceFrames.max(1) as f64 / 1_000_000.0,
                self.performanceResidentSpanUpdates as f64 / self.performanceFrames.max(1) as f64,
                self.performanceResidentSpanBytes as f64 / self.performanceFrames.max(1) as f64 / 1024.0,
                self.performanceDynamicUploadNanos as f64 / self.performanceFrames.max(1) as f64 / 1_000_000.0,
                self.performanceTerrainPlanReuses as f64 * 100.0
                    / (self.performanceTerrainPlanReuses + self.performanceTerrainPlanRebuilds).max(1) as f64,
                self.performanceTerrainPlanReuses,
                self.performanceTerrainPlanReuses + self.performanceTerrainPlanRebuilds,
                self.chunks.len(),
                self.regions.len(),
            );
            self.performanceStarted = Instant::now();
            self.performanceFrames = 0;
            self.performanceDraws = 0;
            self.performanceRanges = 0;
            self.performanceRegionRebuilds = 0;
            self.performanceRegionRebuildNanos = 0;
            self.performanceResidentSpanUpdates = 0;
            self.performanceResidentSpanBytes = 0;
            self.performanceDynamicUploadNanos = 0;
            self.performanceTerrainPlanRebuilds = 0;
            self.performanceTerrainPlanReuses = 0;
        }
        Ok(())
    }

    fn drawHud(&mut self, frame: &WorldRenderFrame, extent: RendererExtent) {
        if self.hudMesh.indexCount == 0 { return; }
        unsafe {
            gl::Viewport(0, 0, extent.width as GLsizei, extent.height as GLsizei);
            gl::UseProgram(self.program);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.atlasTexture);
        }
        self.uploadConstants(&frame.hudPushConstants);
        for range in frame.hudDrawRanges.iter() {
            let blend = match range.pipeline {
                HudPipelineKind::Alpha => Alpha,
                HudPipelineKind::Crosshair => InvertCrosshair,
                HudPipelineKind::Glint => Glint,
            };
            setDrawState(blend, false, false, gl::LEQUAL, false, true, None);
            self.hudMesh.draw(gl::TRIANGLES, range.firstIndex, range.indexCount);
        }
        unsafe {
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
    }

    fn uploadConstants(&self, constants: &WorldPushConstants) {
        unsafe {
            gl::UseProgram(self.program);
            gl::UniformMatrix4fv(self.uniforms.viewProjection, 1, gl::FALSE, constants.viewProjection.as_ptr());
            gl::Uniform4fv(self.uniforms.cameraPosition, 1, constants.cameraPosition.as_ptr());
            gl::Uniform4fv(self.uniforms.fogColor, 1, constants.fogColor.as_ptr());
            gl::Uniform4fv(self.uniforms.fogParameters, 1, constants.fogParameters.as_ptr());
            gl::Uniform4fv(self.uniforms.lightmapParameters, 1, constants.lightmapParameters.as_ptr());
        }
    }

    fn destroy(&mut self) {
        for region in self.regions.values_mut() { region.mesh.destroy(); }
        self.regions.clear();
        self.chunks.clear();
        self.entityMesh.destroy();
        self.blockEntityMesh.destroy();
        self.staticEntityMesh.destroy();
        self.entityDepthMesh.destroy();
        self.overlayMesh.destroy();
        self.particleMesh.destroy();
        self.transparentParticleMesh.destroy();
        self.damageMesh.destroy();
        self.selectionMesh.destroy();
        self.firstPersonMesh.destroy();
        self.hudMesh.destroy();
        unsafe {
            let textures = [
                self.atlasTexture,
                self.lightmapTexture,
                self.normalTexture,
                self.specularTexture,
            ];
            gl::DeleteTextures(textures.len() as GLsizei, textures.as_ptr());
            gl::DeleteProgram(self.program);
        }
    }
}

/// Window-bound OpenGL 3.3 compatibility context. The compatibility profile is
/// intentional: OptiFine 1.12.2 shader packs are GLSL/OpenGL programs and will
/// be attached here in later batches, while the same MCP frame builder remains
/// shared with Vulkan.
pub struct OpenGlWindow {
    context: PossiblyCurrentContext,
    surface: Surface<WindowSurface>,
    extent: RendererExtent,
    deviceName: String,
    enableVsync: bool,
    guiPipeline: OpenGlGuiPipeline,
    nativeGuiPipeline: OpenGlNativeGuiPipeline,
    worldPipeline: OpenGlWorldPipeline,
    shaderRuntime: OptifineShaderRuntime,
    worldPerformanceStarted: Instant,
    worldPerformanceFrames: u64,
    worldShaderPrepareNanos: u128,
    worldResourceUpdateNanos: u128,
    worldShadowNanos: u128,
    worldSceneNanos: u128,
    worldCompositeNanos: u128,
    worldHudNanos: u128,
    worldSwapNanos: u128,
}

impl OpenGlWindow {
    pub fn create(
        eventLoop: &ActiveEventLoop,
        attributes: WindowAttributes,
        gameSettings: &GameSettings,
        gameDir: &Path,
    ) -> anyhow::Result<(Window, Self)> {
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_depth_size(24)
            .with_stencil_size(8);
        let displayBuilder = DisplayBuilder::new().with_window_attributes(Some(attributes));
        let (window, config) = displayBuilder
            .build(eventLoop, template, chooseConfig)
            .map_err(|error| anyhow!(error.to_string()))
            .context("failed creating OpenGL display/window configuration")?;
        let window = window.context("OpenGL display builder returned no window")?;
        let rawWindow = window
            .window_handle()
            .context("failed obtaining OpenGL native window handle")?
            .as_raw();
        let contextAttributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .with_profile(GlProfile::Compatibility)
            .build(Some(rawWindow));
        let display = config.display();
        let notCurrent = unsafe { display.create_context(&config, &contextAttributes) }
            .context("failed creating OpenGL 3.3 compatibility context required by OptiFine shaders")?;
        let surfaceAttributes = window
            .build_surface_attributes(Default::default())
            .context("failed building OpenGL window-surface attributes")?;
        let surface = unsafe { display.create_window_surface(&config, &surfaceAttributes) }
            .context("failed creating OpenGL window surface")?;
        let context = notCurrent
            .make_current(&surface)
            .context("failed making OpenGL context current")?;
        gl::load_with(|symbol| {
            let symbol = CString::new(symbol).expect("OpenGL symbol CString");
            display.get_proc_address(symbol.as_c_str()).cast()
        });
        let version = glString(gl::VERSION).unwrap_or_else(|| "unknown OpenGL version".to_owned());
        let renderer = glString(gl::RENDERER).unwrap_or_else(|| "unknown OpenGL renderer".to_owned());
        let shading = glString(gl::SHADING_LANGUAGE_VERSION).unwrap_or_else(|| "unknown GLSL version".to_owned());
        log::info!("OpenGL output device: {renderer}; version={version}; GLSL={shading}");
        let size = window.inner_size();
        let extent = RendererExtent { width: size.width, height: size.height };
        let output = Self {
            context,
            surface,
            extent,
            deviceName: format!("{renderer} (OpenGL {version})"),
            enableVsync: gameSettings.enableVsync,
            guiPipeline: OpenGlGuiPipeline::new()?,
            nativeGuiPipeline: OpenGlNativeGuiPipeline::new()?,
            worldPipeline: OpenGlWorldPipeline::new()?,
            shaderRuntime: OptifineShaderRuntime::new(gameDir)?,
            worldPerformanceStarted: Instant::now(),
            worldPerformanceFrames: 0,
            worldShaderPrepareNanos: 0,
            worldResourceUpdateNanos: 0,
            worldShadowNanos: 0,
            worldSceneNanos: 0,
            worldCompositeNanos: 0,
            worldHudNanos: 0,
            worldSwapNanos: 0,
        };
        output.applySwapInterval()?;
        Ok((window, output))
    }

    pub const fn extent(&self) -> RendererExtent { self.extent }
    pub fn deviceName(&self) -> &str { &self.deviceName }

    pub fn drawFrame(&mut self, _window: &Window, frame: &CpuFrame) -> anyhow::Result<()> {
        if self.extent.width == 0 || self.extent.height == 0 { return Ok(()); }
        self.guiPipeline.draw(frame, self.extent)?;
        self.surface.swap_buffers(&self.context).context("failed swapping OpenGL GUI buffers")
    }

    pub fn drawNativeGuiFrame(&mut self, _window: &Window, frame: &GuiRenderFrame) -> anyhow::Result<()> {
        if self.extent.width == 0 || self.extent.height == 0 { return Ok(()); }
        self.nativeGuiPipeline.draw(frame, self.extent)?;
        self.surface.swap_buffers(&self.context).context("failed swapping native OpenGL GUI buffers")
    }

    pub fn drawWorldFrame(&mut self, _window: &Window, frame: &WorldRenderFrame) -> anyhow::Result<()> {
        if self.extent.width == 0 || self.extent.height == 0 { return Ok(()); }
        let shaderPrepareStarted = Instant::now();
        let shaderActive = match self.shaderRuntime.prepareScene(&frame.shaderState, self.extent) {
            Ok(active) => active,
            Err(error) => {
                self.shaderRuntime.disableAfterRuntimeError(&error);
                false
            }
        };
        self.worldShaderPrepareNanos = self
            .worldShaderPrepareNanos
            .saturating_add(shaderPrepareStarted.elapsed().as_nanos());
        let shaderAttributes = shaderActive
            && self.shaderRuntime.requiresExtendedVertexAttributes();
        let resourceUpdateStarted = Instant::now();
        self.worldPipeline.updateFrameResources(frame, shaderAttributes)?;
        self.worldResourceUpdateNanos = self
            .worldResourceUpdateNanos
            .saturating_add(resourceUpdateStarted.elapsed().as_nanos());
        if shaderActive {
            let shadowStarted = Instant::now();
            if self.shaderRuntime.beginShadowPass() {
                let shadowResult = self.worldPipeline.drawShadowScene(
                    frame,
                    self.extent,
                    &mut self.shaderRuntime,
                );
                // Always restore the deferred scene framebuffer, even when a
                // shadow traversal draw fails, so the runtime does not leave
                // the next frame attached to the shadow FBO.
                self.shaderRuntime.finishShadowPass(self.extent);
                shadowResult?;
            }
            self.worldShadowNanos = self
                .worldShadowNanos
                .saturating_add(shadowStarted.elapsed().as_nanos());
            let sceneStarted = Instant::now();
            self.worldPipeline.drawScene(
                frame,
                self.extent,
                Some(&mut self.shaderRuntime),
            )?;
            self.worldSceneNanos = self
                .worldSceneNanos
                .saturating_add(sceneStarted.elapsed().as_nanos());
            let compositeStarted = Instant::now();
            self.shaderRuntime.finishScene(&frame.shaderState, self.extent)?;
            self.worldCompositeNanos = self
                .worldCompositeNanos
                .saturating_add(compositeStarted.elapsed().as_nanos());
        } else {
            let sceneStarted = Instant::now();
            self.worldPipeline.drawScene(frame, self.extent, None)?;
            unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
            self.worldSceneNanos = self
                .worldSceneNanos
                .saturating_add(sceneStarted.elapsed().as_nanos());
        }
        // In 1.12.2 GuiIngame is rendered after Shaders.endRender(), so HUD
        // pixels are not fed through composite/final programs.
        let hudStarted = Instant::now();
        self.worldPipeline.drawHud(frame, self.extent);
        self.worldHudNanos = self
            .worldHudNanos
            .saturating_add(hudStarted.elapsed().as_nanos());
        let swapStarted = Instant::now();
        self.surface
            .swap_buffers(&self.context)
            .context("failed swapping OpenGL world buffers")?;
        self.worldSwapNanos = self
            .worldSwapNanos
            .saturating_add(swapStarted.elapsed().as_nanos());

        self.worldPerformanceFrames = self.worldPerformanceFrames.saturating_add(1);
        let elapsed = self.worldPerformanceStarted.elapsed();
        if elapsed >= Duration::from_secs(2) {
            let frames = self.worldPerformanceFrames.max(1) as f64;
            log::info!(
                "OpenGL frame pacing: {:.1} fps, shader_prepare={:.3} ms, resources={:.3} ms, shadow={:.3} ms, scene={:.3} ms, composite={:.3} ms, hud={:.3} ms, swap={:.3} ms",
                self.worldPerformanceFrames as f64 / elapsed.as_secs_f64().max(0.001),
                self.worldShaderPrepareNanos as f64 / frames / 1_000_000.0,
                self.worldResourceUpdateNanos as f64 / frames / 1_000_000.0,
                self.worldShadowNanos as f64 / frames / 1_000_000.0,
                self.worldSceneNanos as f64 / frames / 1_000_000.0,
                self.worldCompositeNanos as f64 / frames / 1_000_000.0,
                self.worldHudNanos as f64 / frames / 1_000_000.0,
                self.worldSwapNanos as f64 / frames / 1_000_000.0,
            );
            self.worldPerformanceStarted = Instant::now();
            self.worldPerformanceFrames = 0;
            self.worldShaderPrepareNanos = 0;
            self.worldResourceUpdateNanos = 0;
            self.worldShadowNanos = 0;
            self.worldSceneNanos = 0;
            self.worldCompositeNanos = 0;
            self.worldHudNanos = 0;
            self.worldSwapNanos = 0;
        }
        Ok(())
    }

    pub fn reloadShaderPack(&mut self) {
        self.shaderRuntime.reloadSelection();
    }

    pub fn resize(&mut self, window: &Window) -> anyhow::Result<()> {
        let size = window.inner_size();
        self.extent = RendererExtent { width: size.width, height: size.height };
        let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else {
            return Ok(());
        };
        self.surface.resize(&self.context, width, height);
        Ok(())
    }

    pub fn setVsync(&mut self, enableVsync: bool) -> anyhow::Result<()> {
        if self.enableVsync == enableVsync { return Ok(()); }
        self.enableVsync = enableVsync;
        self.applySwapInterval()
    }

    fn applySwapInterval(&self) -> anyhow::Result<()> {
        let interval = if self.enableVsync {
            SwapInterval::Wait(NonZeroU32::new(1).expect("1 is non-zero"))
        } else {
            SwapInterval::DontWait
        };
        self.surface
            .set_swap_interval(&self.context, interval)
            .context("failed changing OpenGL swap interval")
    }
}

impl Drop for OpenGlWindow {
    fn drop(&mut self) {
        self.shaderRuntime.destroy();
        self.worldPipeline.destroy();
        self.nativeGuiPipeline.destroy();
        self.guiPipeline.destroy();
    }
}

fn configureWorldTexture(filter: GLint, wrap: GLint) {
    unsafe {
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap);
    }
}

fn chooseConfig(configs: Box<dyn Iterator<Item = Config> + '_>) -> Config {
    // Minecraft 1.12.2 does not force framebuffer multisampling. Selecting the
    // configuration with the largest sample count would silently enable costly
    // driver MSAA even when OptiFine AA is off, changing both performance and
    // edge coverage. Prefer zero samples, or the smallest available count.
    configs
        .min_by_key(|config| config.num_samples())
        .expect("glutin supplied at least one OpenGL configuration")
}

fn compileProgram(vertexSource: &str, fragmentSource: &str) -> anyhow::Result<GLuint> {
    let vertex = compileShader(gl::VERTEX_SHADER, vertexSource)?;
    let fragment = match compileShader(gl::FRAGMENT_SHADER, fragmentSource) {
        Ok(shader) => shader,
        Err(error) => {
            unsafe { gl::DeleteShader(vertex); }
            return Err(error);
        }
    };
    let program = unsafe { gl::CreateProgram() };
    unsafe {
        gl::AttachShader(program, vertex);
        gl::AttachShader(program, fragment);
        gl::LinkProgram(program);
        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);
    }
    let mut status = 0;
    unsafe { gl::GetProgramiv(program, gl::LINK_STATUS, &mut status); }
    if status == gl::TRUE as GLint { return Ok(program); }
    let log = programLog(program);
    unsafe { gl::DeleteProgram(program); }
    Err(anyhow!("OpenGL program link failed: {log}"))
}

fn compileShader(kind: GLenum, source: &str) -> anyhow::Result<GLuint> {
    let shader = unsafe { gl::CreateShader(kind) };
    let source = CString::new(source).context("OpenGL shader source contains NUL")?;
    unsafe {
        gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
    }
    let mut status = 0;
    unsafe { gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status); }
    if status == gl::TRUE as GLint { return Ok(shader); }
    let log = shaderLog(shader);
    unsafe { gl::DeleteShader(shader); }
    Err(anyhow!("OpenGL shader compilation failed: {log}"))
}

fn shaderLog(shader: GLuint) -> String {
    let mut length = 0;
    unsafe { gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut length); }
    let mut buffer = vec![0_u8; length.max(1) as usize];
    unsafe { gl::GetShaderInfoLog(shader, length, std::ptr::null_mut(), buffer.as_mut_ptr().cast::<GLchar>()); }
    String::from_utf8_lossy(&buffer).trim_end_matches('\0').to_owned()
}

fn programLog(program: GLuint) -> String {
    let mut length = 0;
    unsafe { gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut length); }
    let mut buffer = vec![0_u8; length.max(1) as usize];
    unsafe { gl::GetProgramInfoLog(program, length, std::ptr::null_mut(), buffer.as_mut_ptr().cast::<GLchar>()); }
    String::from_utf8_lossy(&buffer).trim_end_matches('\0').to_owned()
}

fn uniformLocation(program: GLuint, name: &str) -> GLint {
    let name = CString::new(name).expect("static uniform CString");
    unsafe { gl::GetUniformLocation(program, name.as_ptr()) }
}

fn glString(name: GLenum) -> Option<String> {
    let pointer = unsafe { gl::GetString(name) };
    if pointer.is_null() { return None; }
    Some(unsafe { CStr::from_ptr(pointer.cast()) }.to_string_lossy().into_owned())
}

fn setDrawState(
    blend: BlendMode,
    depthTest: bool,
    depthWrite: bool,
    depthFunction: GLenum,
    cullBackFaces: bool,
    colorWrite: bool,
    polygonOffset: Option<(f32, f32)>,
) {
    unsafe {
        if depthTest { gl::Enable(gl::DEPTH_TEST); } else { gl::Disable(gl::DEPTH_TEST); }
        gl::DepthMask(if depthWrite { gl::TRUE } else { gl::FALSE });
        gl::DepthFunc(depthFunction);
        if cullBackFaces {
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
        } else {
            gl::Disable(gl::CULL_FACE);
        }
        gl::ColorMask(
            if colorWrite { gl::TRUE } else { gl::FALSE },
            if colorWrite { gl::TRUE } else { gl::FALSE },
            if colorWrite { gl::TRUE } else { gl::FALSE },
            if colorWrite { gl::TRUE } else { gl::FALSE },
        );
        if let Some((factor, units)) = polygonOffset {
            gl::Enable(gl::POLYGON_OFFSET_FILL);
            gl::PolygonOffset(factor, units);
        } else {
            gl::Disable(gl::POLYGON_OFFSET_FILL);
        }
        match blend {
            Disabled => gl::Disable(gl::BLEND),
            Alpha => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ONE, gl::ZERO);
            }
            InvertCrosshair => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(gl::ONE_MINUS_DST_COLOR, gl::ONE_MINUS_SRC_COLOR, gl::ONE, gl::ZERO);
            }
            Glint => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(gl::SRC_COLOR, gl::ONE, gl::ONE, gl::ZERO);
            }
            BlockDamage => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(gl::DST_COLOR, gl::SRC_COLOR, gl::ONE, gl::ZERO);
            }
            TntFlash => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::DST_ALPHA, gl::SRC_ALPHA, gl::DST_ALPHA);
            }
            Additive => {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::ONE, gl::ONE);
            }
            SourceAlphaAdditive => {
                gl::Enable(gl::BLEND);
                gl::BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE, gl::ONE, gl::ZERO);
            }
        }
    }
}

#[cfg(test)]
mod shader_vertex_tests {
    use super::*;

    fn vertex(position: [f32; 3], uv: [f32; 2]) -> WorldVertex {
        WorldVertex {
            position,
            uv,
            color: [1.0; 4],
            lightmap: [15.0, 15.0],
            shaderEntity: [-1, -1, -1],
            shaderPadding: 0,
        }
    }

    #[test]
    fn shader_vertex_layout_has_initialized_optifine_padding() {
        assert_eq!(WorldVertex::STRIDE, 52);
        assert_eq!(GlShaderVertex::STRIDE, 80);
    }

    #[test]
    fn shader_quad_attributes_match_svertexbuilder_geometry() {
        let vertices = [
            vertex([0.0, 0.0, 0.0], [0.0, 0.0]),
            vertex([1.0, 0.0, 0.0], [1.0, 0.0]),
            vertex([1.0, 1.0, 0.0], [1.0, 1.0]),
            vertex([0.0, 1.0, 0.0], [0.0, 1.0]),
        ];
        let output = buildShaderVertices(&vertices, &[0, 1, 2, 2, 3, 0], gl::TRIANGLES);
        assert_eq!(output.len(), 4);
        for expanded in output {
            assert!((expanded.normal[0]).abs() < 1.0e-6);
            assert!((expanded.normal[1]).abs() < 1.0e-6);
            assert!((expanded.normal[2] - 1.0).abs() < 1.0e-6);
            assert!((expanded.midTexCoord[0] - 0.5).abs() < 1.0e-6);
            assert!((expanded.midTexCoord[1] - 0.5).abs() < 1.0e-6);
            assert_eq!(expanded.tangent, [32767, 0, 0, 32767]);
        }
    }
}
