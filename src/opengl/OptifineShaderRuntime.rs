use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::c_void,
    path::PathBuf,
    time::Instant,
};

use anyhow::{anyhow, Context};
use gl::types::{GLchar, GLenum, GLint, GLsizei, GLsizeiptr, GLuint};

use crate::net::minecraft::client::renderer::ShaderFrameState::ShaderFrameState;
use crate::net::optifine::shader::ClippingHelperShadow::ClippingHelperShadow;
use crate::net::optifine::shader::config::ShaderPackOptions::ShaderPackOptions;
use crate::net::optifine::shader::config::ShaderPackSource::{
    loadShaderSourceWithOptions, ShaderMacroEnvironment,
};
use crate::net::optifine::shader::IShaderPack::IShaderPack;
use crate::net::optifine::shader::Shaders::{
    packNameDefault, packNameNone, PropertyDefaultTrueFalse, Shaders,
};
use crate::renderer::DesktopRenderer::RendererExtent;

const COLOR_BUFFER_COUNT: usize = 8;
const SHADOW_DEPTH_BUFFER_COUNT: usize = 2;
const SHADOW_COLOR_BUFFER_COUNT: usize = 2;
const GBUFFER_PROGRAM_COUNT: usize = 20;
const GBUFFER_NAMES: [&str; GBUFFER_PROGRAM_COUNT] = [
    "gbuffers_basic",
    "gbuffers_textured",
    "gbuffers_textured_lit",
    "gbuffers_skybasic",
    "gbuffers_skytextured",
    "gbuffers_clouds",
    "gbuffers_terrain",
    "gbuffers_terrain_solid",
    "gbuffers_terrain_cutout_mip",
    "gbuffers_terrain_cutout",
    "gbuffers_damagedblock",
    "gbuffers_water",
    "gbuffers_block",
    "gbuffers_beaconbeam",
    "gbuffers_item",
    "gbuffers_entities",
    "gbuffers_armor_glint",
    "gbuffers_spidereyes",
    "gbuffers_hand",
    "gbuffers_weather",
];
// Exact `Shaders.programBackups[1..=20]` mapping. Values retain the original
// OptiFine program numbers; zero means the compatibility fixed-function path.
const GBUFFER_BACKUPS: [usize; GBUFFER_PROGRAM_COUNT] = [
    0, 1, 2, 1, 2, 2, 3, 7, 7, 7, 7, 7, 7, 2, 3, 3, 2, 2, 3, 3,
];
const SHADOW_NAMES: [&str; 3] = ["shadow", "shadow_solid", "shadow_cutout"];
const COMPOSITE_NAMES: [&str; 8] = [
    "composite", "composite1", "composite2", "composite3",
    "composite4", "composite5", "composite6", "composite7",
];
const COLOR_TEXTURE_UNITS: [u32; COLOR_BUFFER_COUNT] = [0, 1, 2, 3, 7, 8, 9, 10];
const GBUFFER_ATTRIBUTES: [(GLuint, &str); 8] = [
    (0, "mc112_position"),
    (1, "mc112_texcoord0"),
    (2, "mc112_color"),
    (3, "mc112_lightmap"),
    (4, "mc112_normal"),
    (5, "mc_Entity"),
    (6, "mc_midTexCoord"),
    (7, "at_tangent"),
];
const FULLSCREEN_ATTRIBUTES: [(GLuint, &str); 2] = [
    (0, "mc_position"),
    (1, "mc_texcoord"),
];

const BLIT_VERTEX_SHADER: &str = r#"#version 330 compatibility
layout(location = 0) in vec2 in_position;
layout(location = 1) in vec2 in_uv;
out vec2 texcoord;
void main() {
    gl_Position = vec4(in_position, 0.0, 1.0);
    texcoord = in_uv;
}
"#;

const BLIT_FRAGMENT_SHADER: &str = r#"#version 330 compatibility
uniform sampler2D colortex0;
in vec2 texcoord;
out vec4 fragment_color;
void main() { fragment_color = texture(colortex0, texcoord); }
"#;

// `Shaders.setupProgram` accepts a program when either source stage exists and
// lets the OpenGL compatibility pipeline provide the missing fixed-function
// stage. The Rust backend uses explicit equivalents so fragment-only composite
// programs retain the 0..1 texture coordinates produced by `drawComposite`.
const FIXED_COMPOSITE_VERTEX_SHADER: &str = r#"#version 120
void main() {
    gl_Position = ftransform();
    gl_TexCoord[0] = gl_MultiTexCoord0;
    gl_FrontColor = gl_Color;
}
"#;

const FIXED_COMPOSITE_FRAGMENT_SHADER: &str = r#"#version 120
uniform sampler2D colortex0;
void main() { gl_FragColor = texture2D(colortex0, gl_TexCoord[0].st); }
"#;

// Explicit equivalents for the fixed-function half of a scene program. Vanilla
// OptiFine accepts vertex-only or fragment-only programs; this backend cannot
// rely on legacy matrix/client-array state because the same meshes are shared
// with Vulkan, so the missing stage is supplied with equivalent attributes.
const FIXED_GBUFFER_VERTEX_SHADER: &str = r#"#version 120
void main() {
    gl_Position = ftransform();
    gl_FrontColor = gl_Color;
    gl_TexCoord[0] = gl_MultiTexCoord0;
    gl_TexCoord[1] = gl_TextureMatrix[1] * gl_MultiTexCoord1;
    gl_FogFragCoord = abs((gl_ModelViewMatrix * gl_Vertex).z);
}
"#;

const FIXED_GBUFFER_FRAGMENT_SHADER: &str = r#"#version 120
uniform sampler2D texture;
void main() {
    gl_FragData[0] = texture2D(texture, gl_TexCoord[0].st) * gl_Color;
}
"#;

// Explicit compatibility equivalent of the fixed-function fragment stage used
// by `ShadersRender.renderShadowMap` when program 30 (`shadow`) has ID zero.
// Vanilla alpha testing still rejects cutout texels in that path. The shared
// Rust renderer has no fixed-function alpha-test state, so the threshold is
// carried by the existing draw-state uniform and applied here.
const FIXED_SHADOW_FRAGMENT_SHADER: &str = r#"#version 120
uniform sampler2D texture;
uniform float mc112_alpha_cutoff;
void main() {
    vec4 color = texture2D(texture, gl_TexCoord[0].st) * gl_Color;
    if (mc112_alpha_cutoff >= 0.0 && color.a <= mc112_alpha_cutoff) discard;
    gl_FragColor = color;
}
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackProgramStage {
    Gbuffer,
    Composite,
    Final,
    Shadow,
}

#[derive(Debug, Clone, Copy, Default)]
struct ProgramResourceUsage {
    colorBuffers: usize,
    depthBuffers: usize,
    shadowDepthBuffers: usize,
    shadowColorBuffers: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum GbufferProgram {
    Basic = 0,
    Textured = 1,
    TexturedLit = 2,
    SkyBasic = 3,
    SkyTextured = 4,
    Clouds = 5,
    Terrain = 6,
    TerrainSolid = 7,
    TerrainCutoutMip = 8,
    TerrainCutout = 9,
    DamagedBlock = 10,
    Water = 11,
    Block = 12,
    BeaconBeam = 13,
    Item = 14,
    Entities = 15,
    ArmorGlint = 16,
    SpiderEyes = 17,
    Hand = 18,
    Weather = 19,
}

#[derive(Debug, Clone, Copy)]
pub struct GbufferDrawState {
    pub program: GbufferProgram,
    pub atlasSize: [i32; 2],
    /// Combined Vulkan-clip transform for the actual draw range. Sky and hand
    /// passes do not share the ordinary world transform.
    pub viewProjection: [f32; 16],
    pub fogColor: [f32; 4],
    pub fogParameters: [f32; 4],
    pub lightmapParameters: [f32; 4],
    pub entityId: i32,
    pub blockEntityId: i32,
    pub entityColor: [f32; 4],
}

impl GbufferDrawState {
    pub const fn new(
        program: GbufferProgram,
        atlasSize: [i32; 2],
        viewProjection: [f32; 16],
        fogColor: [f32; 4],
        fogParameters: [f32; 4],
        lightmapParameters: [f32; 4],
    ) -> Self {
        Self {
            program,
            atlasSize,
            viewProjection,
            fogColor,
            fogParameters,
            lightmapParameters,
            entityId: -1,
            blockEntityId: -1,
            entityColor: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

struct PackProgram {
    id: GLuint,
    drawBuffers: Vec<usize>,
    drawBuffersExplicit: bool,
    requiredColorBuffers: usize,
    requiredDepthBuffers: usize,
    requiredShadowDepthBuffers: usize,
    requiredShadowColorBuffers: usize,
    colorFormats: [Option<GLint>; COLOR_BUFFER_COUNT],
    usesGdepthUniform: bool,
    noiseTextureResolution: Option<i32>,
    compositeMipmapMask: u32,
    clearDisabledMask: u32,
    shadowHardwareFilteringMask: u32,
    shadowMapHalfPlane: Option<f32>,
    shadowMapFov: Option<f32>,
    shadowIntervalSize: Option<f32>,
    shadowMapResolution: Option<i32>,
    shadowMipmapMask: u32,
    shadowNearestMask: u32,
    shadowColorMipmapMask: u32,
    shadowColorNearestMask: u32,
    sunPathRotation: Option<f32>,
    uniforms: HashMap<&'static str, GLint>,
}

impl PackProgram {
    fn uniform(&mut self, name: &'static str) -> GLint {
        if let Some(location) = self.uniforms.get(name).copied() {
            return location;
        }
        let cName = CString::new(name).expect("static shader uniform name");
        let location = unsafe { gl::GetUniformLocation(self.id, cName.as_ptr()) };
        self.uniforms.insert(name, location);
        location
    }

    fn destroy(&mut self) {
        if self.id != 0 {
            unsafe { gl::DeleteProgram(self.id); }
            self.id = 0;
        }
    }
}

struct ShaderTargets {
    framebuffer: GLuint,
    colorTextures: [[GLuint; 2]; COLOR_BUFFER_COUNT],
    /// OptiFine `dfbDepthTextures`: depthtex0 is attached to the deferred FBO;
    /// depthtex1/2 retain the pre-translucent and pre-weather snapshots.
    depthTextures: [GLuint; 3],
    colorFormats: [GLint; COLOR_BUFFER_COUNT],
    extent: RendererExtent,
    colorToggle: [usize; COLOR_BUFFER_COUNT],
    usedColorBuffers: usize,
    usedDepthBuffers: usize,
    clearBuffers: [bool; COLOR_BUFFER_COUNT],
}

impl ShaderTargets {
    fn new() -> anyhow::Result<Self> {
        let mut framebuffer = 0;
        let mut colorTextures = [[0; 2]; COLOR_BUFFER_COUNT];
        let mut depthTextures = [0_u32; 3];
        unsafe {
            gl::GenFramebuffers(1, &mut framebuffer);
            for pair in &mut colorTextures {
                gl::GenTextures(2, pair.as_mut_ptr());
            }
            gl::GenTextures(depthTextures.len() as GLsizei, depthTextures.as_mut_ptr());
        }
        anyhow::ensure!(
            framebuffer != 0 && depthTextures.iter().all(|texture| *texture != 0),
            "OptiFine framebuffer allocation failed",
        );
        anyhow::ensure!(colorTextures.iter().flatten().all(|texture| *texture != 0), "OptiFine color texture allocation failed");
        Ok(Self {
            framebuffer,
            colorTextures,
            depthTextures,
            colorFormats: [gl::RGBA8 as GLint; COLOR_BUFFER_COUNT],
            extent: RendererExtent::default(),
            colorToggle: [0; COLOR_BUFFER_COUNT],
            usedColorBuffers: 4,
            usedDepthBuffers: 1,
            clearBuffers: [true; COLOR_BUFFER_COUNT],
        })
    }

    fn setColorFormats(&mut self, colorFormats: [GLint; COLOR_BUFFER_COUNT]) {
        if self.colorFormats != colorFormats {
            self.colorFormats = colorFormats;
            // Force glTexImage2D recreation on the next prepareScene call.
            self.extent = RendererExtent::default();
        }
    }

    fn setUsage(&mut self, usedColorBuffers: usize, usedDepthBuffers: usize) {
        let usedColorBuffers = usedColorBuffers.clamp(1, COLOR_BUFFER_COUNT);
        let usedDepthBuffers = usedDepthBuffers.clamp(1, self.depthTextures.len());
        if self.usedColorBuffers != usedColorBuffers || self.usedDepthBuffers != usedDepthBuffers {
            self.usedColorBuffers = usedColorBuffers;
            self.usedDepthBuffers = usedDepthBuffers;
            self.extent = RendererExtent::default();
        }
    }

    fn setClearDisabledMask(&mut self, disabledMask: u32) {
        for index in 0..COLOR_BUFFER_COUNT {
            self.clearBuffers[index] = disabledMask & (1_u32 << index) == 0;
        }
    }

    fn invalidate(&mut self) {
        self.extent = RendererExtent::default();
    }

    fn ensureSize(&mut self, extent: RendererExtent) -> anyhow::Result<()> {
        if self.extent == extent || extent.width == 0 || extent.height == 0 {
            return Ok(());
        }
        self.extent = extent;
        self.colorToggle = [0; COLOR_BUFFER_COUNT];
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            for (index, pair) in self.colorTextures.iter().enumerate() {
                let internalFormat = self.colorFormats[index];
                let (externalFormat, externalType) = textureUploadFormat(internalFormat);
                let (textureWidth, textureHeight) = if index < self.usedColorBuffers {
                    (extent.width as GLsizei, extent.height as GLsizei)
                } else {
                    // Keep every sampler name backed by a complete texture while
                    // avoiding two full-resolution images for an unused attachment.
                    (1, 1)
                };
                for texture in pair {
                    gl::BindTexture(gl::TEXTURE_2D, *texture);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
                    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        internalFormat,
                        textureWidth,
                        textureHeight,
                        0,
                        externalFormat,
                        externalType,
                        std::ptr::null(),
                    );
                }
            }
            for (index, texture) in self.depthTextures.into_iter().enumerate() {
                let (textureWidth, textureHeight) = if index < self.usedDepthBuffers {
                    (extent.width as GLsizei, extent.height as GLsizei)
                } else {
                    (1, 1)
                };
                gl::BindTexture(gl::TEXTURE_2D, texture);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::DEPTH_COMPONENT24 as GLint,
                    textureWidth,
                    textureHeight,
                    0,
                    gl::DEPTH_COMPONENT,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
            }
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                self.depthTextures[0],
                0,
            );
            for index in 0..COLOR_BUFFER_COUNT {
                let texture = if index < self.usedColorBuffers {
                    self.colorTextures[index][0]
                } else {
                    0
                };
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + index as GLenum,
                    gl::TEXTURE_2D,
                    texture,
                    0,
                );
            }
            let mut drawBuffers = [gl::NONE; COLOR_BUFFER_COUNT];
            for (index, drawBuffer) in drawBuffers
                .iter_mut()
                .take(self.usedColorBuffers)
                .enumerate()
            {
                *drawBuffer = gl::COLOR_ATTACHMENT0 + index as GLenum;
            }
            gl::DrawBuffers(
                self.usedColorBuffers as GLsizei,
                drawBuffers.as_ptr(),
            );
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            anyhow::ensure!(status == gl::FRAMEBUFFER_COMPLETE, "OptiFine framebuffer is incomplete: 0x{status:04X}");
        }
        Ok(())
    }

    fn beginScene(
        &mut self,
        extent: RendererExtent,
        clearColor: [f32; 4],
    ) -> anyhow::Result<()> {
        self.ensureSize(extent)?;
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                self.depthTextures[0],
                0,
            );
            // Shaders.beginRender resets every colorTexturesToggle entry to A
            // and reattaches dfbColorTexturesA[index] after the shadow pass.
            // Keeping the previous composite attachment here makes the next
            // G-buffer frame render into an arbitrary ping-pong side and breaks
            // temporal packs. `colortexNClear = false` still preserves side A;
            // it only suppresses the clear, not this per-frame attachment reset.
            self.colorToggle = [0; COLOR_BUFFER_COUNT];
            gl::ActiveTexture(gl::TEXTURE0);
            // `Shaders.beginRender` restores both A/B deferred textures to
            // linear filtering before the scene pass. Composite mipmap
            // generation temporarily changes the active read side's min filter;
            // carrying that state into the next frame changes ordinary texture
            // sampling and is not equivalent to OptiFine 1.12.2.
            for index in 0..self.usedColorBuffers {
                for texture in self.colorTextures[index] {
                    gl::BindTexture(gl::TEXTURE_2D, texture);
                    gl::TexParameteri(
                        gl::TEXTURE_2D,
                        gl::TEXTURE_MIN_FILTER,
                        gl::LINEAR as GLint,
                    );
                    gl::TexParameteri(
                        gl::TEXTURE_2D,
                        gl::TEXTURE_MAG_FILTER,
                        gl::LINEAR as GLint,
                    );
                }
                gl::BindTexture(gl::TEXTURE_2D, 0);
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + index as GLenum,
                    gl::TEXTURE_2D,
                    self.colorTextures[index][0],
                    0,
                );
                if !self.clearBuffers[index] {
                    continue;
                }
                gl::DrawBuffer(gl::COLOR_ATTACHMENT0 + index as GLenum);
                gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
                if index == 0 {
                    gl::ClearColor(clearColor[0], clearColor[1], clearColor[2], clearColor[3]);
                } else if index == 1 {
                    gl::ClearColor(1.0, 1.0, 1.0, 1.0);
                } else {
                    gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                }
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
            gl::DepthMask(gl::TRUE);
            gl::ClearDepth(1.0);
            gl::Clear(gl::DEPTH_BUFFER_BIT);
        }
        Ok(())
    }

    fn restoreSceneFramebuffer(&self, extent: RendererExtent) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Viewport(0, 0, extent.width as GLsizei, extent.height as GLsizei);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                self.depthTextures[0],
                0,
            );
            for index in 0..COLOR_BUFFER_COUNT {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + index as GLenum,
                    gl::TEXTURE_2D,
                    if index < self.usedColorBuffers {
                        self.colorTextures[index][0]
                    } else {
                        0
                    },
                    0,
                );
            }
            let mut drawBuffers = [gl::NONE; COLOR_BUFFER_COUNT];
            for (index, drawBuffer) in drawBuffers
                .iter_mut()
                .take(self.usedColorBuffers)
                .enumerate()
            {
                *drawBuffer = gl::COLOR_ATTACHMENT0 + index as GLenum;
            }
            gl::DrawBuffers(
                self.usedColorBuffers as GLsizei,
                drawBuffers.as_ptr(),
            );
        }
    }

    fn bindGbufferDepthTextures(&self) {
        unsafe {
            // Shaders.setupFrameBuffer leaves the deferred depth textures on
            // their fixed units while the world G-buffer programs execute.
            // Program 1..20 names depthtex1 on unit 12; composite/final uses
            // the conventional depthtex1=11 and depthtex2=12 mapping.
            for (index, unit) in [6_u32, 11, 12].into_iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + unit);
                gl::BindTexture(
                    gl::TEXTURE_2D,
                    if index < self.usedDepthBuffers { self.depthTextures[index] } else { 0 },
                );
            }
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }

    fn bindReadTextures(&self) {
        unsafe {
            for index in 0..COLOR_BUFFER_COUNT {
                gl::ActiveTexture(gl::TEXTURE0 + COLOR_TEXTURE_UNITS[index]);
                gl::BindTexture(
                    gl::TEXTURE_2D,
                    if index < self.usedColorBuffers {
                        self.colorTextures[index][self.colorToggle[index]]
                    } else {
                        0
                    },
                );
            }
            for (index, unit) in [6_u32, 11, 12].into_iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + unit);
                gl::BindTexture(
                    gl::TEXTURE_2D,
                    if index < self.usedDepthBuffers { self.depthTextures[index] } else { 0 },
                );
            }
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }


    fn copyDepthSnapshot(&self, snapshot: usize, textureUnit: u32) {
        if snapshot == 0
            || snapshot >= self.usedDepthBuffers
            || snapshot >= self.depthTextures.len()
            || self.extent.width == 0
            || self.extent.height == 0
        {
            return;
        }
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + textureUnit);
            gl::BindTexture(gl::TEXTURE_2D, self.depthTextures[snapshot]);
            gl::CopyTexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                0,
                0,
                self.extent.width as GLsizei,
                self.extent.height as GLsizei,
            );
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }

    fn generateCompositeMipmaps(&self, mask: u32) {
        if mask == 0 {
            return;
        }
        unsafe {
            for index in 0..self.usedColorBuffers {
                if mask & (1_u32 << index) == 0 {
                    continue;
                }
                gl::ActiveTexture(gl::TEXTURE0 + COLOR_TEXTURE_UNITS[index]);
                gl::BindTexture(
                    gl::TEXTURE_2D,
                    self.colorTextures[index][self.colorToggle[index]],
                );
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR_MIPMAP_LINEAR as GLint,
                );
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }

    fn beginCompositePass(&mut self, drawBuffers: &[usize]) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, gl::TEXTURE_2D, 0, 0);
            // OptiFine binds the B side of every color attachment before
            // composite begins while sampling the A side. Detaching every read
            // image is required: sampling a texture that remains attached to the
            // active FBO is an OpenGL feedback loop even when that attachment is
            // omitted from the current draw-buffer list.
            for index in 0..COLOR_BUFFER_COUNT {
                let texture = if index < self.usedColorBuffers {
                    let writeSide = 1 - self.colorToggle[index];
                    self.colorTextures[index][writeSide]
                } else {
                    0
                };
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + index as GLenum,
                    gl::TEXTURE_2D,
                    texture,
                    0,
                );
            }
            let mut attachments = [gl::NONE; COLOR_BUFFER_COUNT];
            let mut attachmentCount = 0_usize;
            for &index in drawBuffers {
                if index >= self.usedColorBuffers { continue; }
                if attachmentCount == COLOR_BUFFER_COUNT { break; }
                attachments[attachmentCount] = gl::COLOR_ATTACHMENT0 + index as GLenum;
                attachmentCount += 1;
            }
            if attachmentCount == 0 {
                attachments[0] = gl::COLOR_ATTACHMENT0;
                attachmentCount = 1;
            }
            gl::DrawBuffers(attachmentCount as GLsizei, attachments.as_ptr());
        }
    }

    fn finishCompositePass(&mut self, drawBuffers: &[usize]) {
        for &index in drawBuffers {
            if index < self.usedColorBuffers {
                self.colorToggle[index] = 1 - self.colorToggle[index];
            }
        }
    }

    fn destroy(&mut self) {
        unsafe {
            for pair in &self.colorTextures {
                gl::DeleteTextures(2, pair.as_ptr());
            }
            gl::DeleteTextures(self.depthTextures.len() as GLsizei, self.depthTextures.as_ptr());
            gl::DeleteFramebuffers(1, &self.framebuffer);
        }
        self.framebuffer = 0;
        self.depthTextures = [0; 3];
        self.colorTextures = [[0; 2]; COLOR_BUFFER_COUNT];
    }
}


struct ShadowTargets {
    framebuffer: GLuint,
    depthTextures: [GLuint; SHADOW_DEPTH_BUFFER_COUNT],
    colorTextures: [GLuint; SHADOW_COLOR_BUFFER_COUNT],
    resolution: i32,
    usedDepthBuffers: usize,
    usedColorBuffers: usize,
    hardwareFilteringMask: u32,
    depthMipmapMask: u32,
    depthNearestMask: u32,
    colorMipmapMask: u32,
    colorNearestMask: u32,
}

impl ShadowTargets {
    fn new() -> anyhow::Result<Self> {
        let mut framebuffer = 0;
        let mut depthTextures = [0; SHADOW_DEPTH_BUFFER_COUNT];
        let mut colorTextures = [0; SHADOW_COLOR_BUFFER_COUNT];
        unsafe {
            gl::GenFramebuffers(1, &mut framebuffer);
            gl::GenTextures(depthTextures.len() as GLsizei, depthTextures.as_mut_ptr());
            gl::GenTextures(colorTextures.len() as GLsizei, colorTextures.as_mut_ptr());
        }
        anyhow::ensure!(
            framebuffer != 0
                && depthTextures.iter().all(|texture| *texture != 0)
                && colorTextures.iter().all(|texture| *texture != 0),
            "OptiFine shadow framebuffer allocation failed",
        );
        Ok(Self {
            framebuffer,
            depthTextures,
            colorTextures,
            resolution: 0,
            usedDepthBuffers: 0,
            usedColorBuffers: 0,
            hardwareFilteringMask: 0,
            depthMipmapMask: 0,
            depthNearestMask: 0,
            colorMipmapMask: 0,
            colorNearestMask: 0,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn configure(
        &mut self,
        resolution: i32,
        usedDepthBuffers: usize,
        usedColorBuffers: usize,
        hardwareFilteringMask: u32,
        depthMipmapMask: u32,
        depthNearestMask: u32,
        colorMipmapMask: u32,
        colorNearestMask: u32,
    ) -> anyhow::Result<()> {
        let usedDepthBuffers = usedDepthBuffers.min(SHADOW_DEPTH_BUFFER_COUNT);
        let usedColorBuffers = usedColorBuffers.min(SHADOW_COLOR_BUFFER_COUNT);
        let resolution = if usedDepthBuffers == 0 { 0 } else { resolution.max(1) };
        let changed = self.resolution != resolution
            || self.usedDepthBuffers != usedDepthBuffers
            || self.usedColorBuffers != usedColorBuffers
            || self.hardwareFilteringMask != hardwareFilteringMask
            || self.depthMipmapMask != depthMipmapMask
            || self.depthNearestMask != depthNearestMask
            || self.colorMipmapMask != colorMipmapMask
            || self.colorNearestMask != colorNearestMask;
        self.resolution = resolution;
        self.usedDepthBuffers = usedDepthBuffers;
        self.usedColorBuffers = usedColorBuffers;
        self.hardwareFilteringMask = hardwareFilteringMask & 0b11;
        self.depthMipmapMask = depthMipmapMask & 0b11;
        self.depthNearestMask = depthNearestMask & 0b11;
        self.colorMipmapMask = colorMipmapMask & 0b11;
        self.colorNearestMask = colorNearestMask & 0b11;
        if !changed || usedDepthBuffers == 0 {
            return Ok(());
        }

        let mut maximum = 0;
        unsafe { gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut maximum); }
        anyhow::ensure!(
            maximum > 0 && resolution <= maximum,
            "shadowMapResolution {resolution} exceeds OpenGL GL_MAX_TEXTURE_SIZE {maximum}",
        );

        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            for (index, texture) in self.depthTextures.iter().copied().enumerate() {
                gl::BindTexture(gl::TEXTURE_2D, texture);
                let nearest = self.depthNearestMask & (1_u32 << index) != 0;
                let filter = if nearest { gl::NEAREST } else { gl::LINEAR };
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as GLint);
                // OptiFine 1.12.2 uses legacy GL_CLAMP (10496) for shadow maps.
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, 0x2900);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, 0x2900);
                let compare = self.hardwareFilteringMask & (1_u32 << index) != 0;
                gl::TexParameteri(
                    gl::TEXTURE_2D,
                    gl::TEXTURE_COMPARE_MODE,
                    if compare { gl::COMPARE_REF_TO_TEXTURE as GLint } else { gl::NONE as GLint },
                );
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_COMPARE_FUNC, gl::LEQUAL as GLint);
                let size = if index < usedDepthBuffers { resolution } else { 1 };
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::DEPTH_COMPONENT as GLint,
                    size,
                    size,
                    0,
                    gl::DEPTH_COMPONENT,
                    gl::FLOAT,
                    std::ptr::null(),
                );
            }
            for (index, texture) in self.colorTextures.iter().copied().enumerate() {
                gl::BindTexture(gl::TEXTURE_2D, texture);
                let nearest = self.colorNearestMask & (1_u32 << index) != 0;
                let filter = if nearest { gl::NEAREST } else { gl::LINEAR };
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, filter as GLint);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, 0x2900);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, 0x2900);
                let size = if index < usedColorBuffers { resolution } else { 1 };
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as GLint,
                    size,
                    size,
                    0,
                    gl::BGRA,
                    gl::UNSIGNED_INT_8_8_8_8_REV,
                    std::ptr::null(),
                );
            }
            gl::BindTexture(gl::TEXTURE_2D, 0);
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                self.depthTextures[0],
                0,
            );
            for index in 0..SHADOW_COLOR_BUFFER_COUNT {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0 + index as GLenum,
                    gl::TEXTURE_2D,
                    if index < usedColorBuffers { self.colorTextures[index] } else { 0 },
                    0,
                );
            }
            if usedColorBuffers == 0 {
                gl::DrawBuffer(gl::NONE);
                gl::ReadBuffer(gl::NONE);
            } else {
                let drawBuffers = (0..usedColorBuffers)
                    .map(|index| gl::COLOR_ATTACHMENT0 + index as GLenum)
                    .collect::<Vec<_>>();
                gl::DrawBuffers(drawBuffers.len() as GLsizei, drawBuffers.as_ptr());
                gl::ReadBuffer(gl::NONE);
            }
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            anyhow::ensure!(
                status == gl::FRAMEBUFFER_COMPLETE,
                "OptiFine shadow framebuffer is incomplete: 0x{status:04X}",
            );
        }
        Ok(())
    }

    fn isActive(&self) -> bool {
        self.usedDepthBuffers > 0 && self.resolution > 0
    }

    fn begin(&self) {
        if !self.isActive() {
            return;
        }
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::Viewport(0, 0, self.resolution, self.resolution);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                self.depthTextures[0],
                0,
            );
            if self.usedColorBuffers == 0 {
                gl::DrawBuffer(gl::NONE);
                gl::ReadBuffer(gl::NONE);
            } else {
                let mut drawBuffers = [gl::NONE; SHADOW_COLOR_BUFFER_COUNT];
                for (index, drawBuffer) in drawBuffers
                    .iter_mut()
                    .take(self.usedColorBuffers)
                    .enumerate()
                {
                    *drawBuffer = gl::COLOR_ATTACHMENT0 + index as GLenum;
                }
                gl::DrawBuffers(
                    self.usedColorBuffers as GLsizei,
                    drawBuffers.as_ptr(),
                );
                gl::ReadBuffer(gl::NONE);
            }
            gl::ClearColor(1.0, 1.0, 1.0, 1.0);
            gl::ClearDepth(1.0);
            let mask = if self.usedColorBuffers > 0 {
                gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT
            } else {
                gl::DEPTH_BUFFER_BIT
            };
            gl::DepthMask(gl::TRUE);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::Clear(mask);
        }
    }

    fn copyOpaqueDepth(&self) {
        if self.usedDepthBuffers < 2 || !self.isActive() {
            return;
        }
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + 5);
            gl::BindTexture(gl::TEXTURE_2D, self.depthTextures[1]);
            gl::CopyTexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                0,
                0,
                self.resolution,
                self.resolution,
            );
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }

    fn finish(&self) {
        if !self.isActive() {
            return;
        }
        unsafe {
            for index in 0..self.usedDepthBuffers {
                if self.depthMipmapMask & (1_u32 << index) == 0 {
                    continue;
                }
                gl::ActiveTexture(gl::TEXTURE0 + 4 + index as u32);
                gl::BindTexture(gl::TEXTURE_2D, self.depthTextures[index]);
                gl::GenerateMipmap(gl::TEXTURE_2D);
                let filter = if self.depthNearestMask & (1_u32 << index) != 0 {
                    gl::NEAREST_MIPMAP_NEAREST
                } else {
                    gl::LINEAR_MIPMAP_LINEAR
                };
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as GLint);
            }
            for index in 0..self.usedColorBuffers {
                if self.colorMipmapMask & (1_u32 << index) == 0 {
                    continue;
                }
                gl::ActiveTexture(gl::TEXTURE0 + 13 + index as u32);
                gl::BindTexture(gl::TEXTURE_2D, self.colorTextures[index]);
                gl::GenerateMipmap(gl::TEXTURE_2D);
                let filter = if self.colorNearestMask & (1_u32 << index) != 0 {
                    gl::NEAREST_MIPMAP_NEAREST
                } else {
                    gl::LINEAR_MIPMAP_LINEAR
                };
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, filter as GLint);
            }
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }

    fn bindTextures(&self) {
        unsafe {
            for index in 0..SHADOW_DEPTH_BUFFER_COUNT {
                gl::ActiveTexture(gl::TEXTURE0 + 4 + index as u32);
                gl::BindTexture(
                    gl::TEXTURE_2D,
                    if index < self.usedDepthBuffers { self.depthTextures[index] } else { 0 },
                );
            }
            for index in 0..SHADOW_COLOR_BUFFER_COUNT {
                gl::ActiveTexture(gl::TEXTURE0 + 13 + index as u32);
                gl::BindTexture(
                    gl::TEXTURE_2D,
                    if index < self.usedColorBuffers { self.colorTextures[index] } else { 0 },
                );
            }
            gl::ActiveTexture(gl::TEXTURE0);
        }
    }

    fn destroy(&mut self) {
        unsafe {
            gl::DeleteTextures(self.depthTextures.len() as GLsizei, self.depthTextures.as_ptr());
            gl::DeleteTextures(self.colorTextures.len() as GLsizei, self.colorTextures.as_ptr());
            gl::DeleteFramebuffers(1, &self.framebuffer);
        }
        self.framebuffer = 0;
        self.depthTextures = [0; SHADOW_DEPTH_BUFFER_COUNT];
        self.colorTextures = [0; SHADOW_COLOR_BUFFER_COUNT];
        self.usedDepthBuffers = 0;
        self.usedColorBuffers = 0;
        self.resolution = 0;
    }
}

pub struct OptifineShaderRuntime {
    gameDir: PathBuf,
    selectedName: String,
    loadedDimension: Option<i32>,
    gbufferPrograms: [Option<PackProgram>; GBUFFER_PROGRAM_COUNT],
    programs: Vec<PackProgram>,
    finalProgram: Option<PackProgram>,
    shadowPrograms: [Option<PackProgram>; 3],
    fixedShadowProgram: PackProgram,
    packOptions: Option<ShaderPackOptions>,
    targets: ShaderTargets,
    shadowTargets: ShadowTargets,
    fullscreenVao: GLuint,
    fullscreenBuffer: GLuint,
    blitProgram: GLuint,
    noiseTexture: GLuint,
    noiseTextureResolution: i32,
    frameCounter: i32,
    lastFrame: Instant,
    currentFrameTime: f32,
    frameTimeCounter: f32,
    previousProjection: [f32; 16],
    previousModelView: [f32; 16],
    previousCameraPosition: [f32; 3],
    shadowProjection: [f32; 16],
    shadowProjectionInverse: [f32; 16],
    shadowModelView: [f32; 16],
    shadowModelViewInverse: [f32; 16],
    shadowLightPositionVector: [f32; 4],
    shadowMapHalfPlane: f32,
    shadowMapFov: f32,
    shadowMapIsOrtho: bool,
    shadowIntervalSize: f32,
    sunPathRotation: f32,
    shadowHardwareFilteringMask: u32,
    shadowPassInterval: i32,
    shadowPassCounter: i32,
    renderShadowTranslucent: bool,
    renderResMul: f32,
    shadowResMul: f32,
    handDepthMul: f32,
    renderExtent: RendererExtent,
    active: bool,
}

impl OptifineShaderRuntime {
    pub fn new(gameDir: impl Into<PathBuf>) -> anyhow::Result<Self> {
        Self::withGameDir(gameDir)
    }

    pub fn withGameDir(gameDir: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let gameDir = gameDir.into();
        let config = Shaders::loadConfig(&gameDir);
        let selectedName = config.currentshadername.clone();
        let (fullscreenVao, fullscreenBuffer) = createFullscreenMesh()?;
        let blitProgram = compileProgram(
            "internal_blit",
            BLIT_VERTEX_SHADER,
            BLIT_FRAGMENT_SHADER,
            &[(0, "in_position"), (1, "in_uv")],
        )?;
        let fixedShadowProgram = createFixedShadowProgram()?;
        Ok(Self {
            gameDir,
            selectedName,
            loadedDimension: None,
            gbufferPrograms: std::array::from_fn(|_| None),
            programs: Vec::new(),
            finalProgram: None,
            shadowPrograms: std::array::from_fn(|_| None),
            fixedShadowProgram,
            packOptions: None,
            targets: ShaderTargets::new()?,
            shadowTargets: ShadowTargets::new()?,
            fullscreenVao,
            fullscreenBuffer,
            blitProgram,
            noiseTexture: 0,
            noiseTextureResolution: 0,
            frameCounter: 0,
            lastFrame: Instant::now(),
            currentFrameTime: 0.0,
            frameTimeCounter: 0.0,
            previousProjection: identity4(),
            previousModelView: identity4(),
            previousCameraPosition: [0.0; 3],
            shadowProjection: identity4(),
            shadowProjectionInverse: identity4(),
            shadowModelView: identity4(),
            shadowModelViewInverse: identity4(),
            shadowLightPositionVector: [0.0, 1.0, 0.0, 0.0],
            shadowMapHalfPlane: 160.0,
            shadowMapFov: 90.0,
            shadowMapIsOrtho: true,
            shadowIntervalSize: 2.0,
            sunPathRotation: 0.0,
            shadowHardwareFilteringMask: 0,
            shadowPassInterval: 0,
            shadowPassCounter: 0,
            renderShadowTranslucent: true,
            renderResMul: config.configRenderResMul,
            shadowResMul: config.configShadowResMul,
            handDepthMul: config.configHandDepthMul,
            renderExtent: RendererExtent::default(),
            active: false,
        })
    }

    pub fn selectedName(&self) -> &str { &self.selectedName }
    pub fn isActive(&self) -> bool { self.active }
    pub fn hasGbufferPrograms(&self) -> bool {
        self.gbufferPrograms.iter().any(Option::is_some)
    }

    /// `SVertexBuilder` attributes are enabled whenever a pack supplies either
    /// a G-buffer program or a custom shadow program. The fixed shadow fallback
    /// consumes the vanilla vertex layout and therefore does not force an
    /// unnecessary expanded upload.
    pub fn requiresExtendedVertexAttributes(&self) -> bool {
        self.gbufferPrograms.iter().any(Option::is_some)
            || self.shadowPrograms.iter().any(Option::is_some)
    }

    pub fn reloadSelection(&mut self) {
        let config = Shaders::loadConfig(&self.gameDir);
        self.selectedName = config.currentshadername.clone();
        self.renderResMul = config.configRenderResMul;
        self.shadowResMul = config.configShadowResMul;
        self.handDepthMul = config.configHandDepthMul;
        self.renderExtent = RendererExtent::default();
        self.loadedDimension = None;
        self.clearPrograms();
        self.lastFrame = Instant::now();
        self.currentFrameTime = 0.0;
        self.frameTimeCounter = 0.0;
        self.previousProjection = identity4();
        self.previousModelView = identity4();
        self.previousCameraPosition = [0.0; 3];
        self.shadowProjection = identity4();
        self.shadowProjectionInverse = identity4();
        self.shadowModelView = identity4();
        self.shadowModelViewInverse = identity4();
        self.shadowLightPositionVector = [0.0, 1.0, 0.0, 0.0];
        self.shadowMapHalfPlane = 160.0;
        self.shadowMapFov = 90.0;
        self.shadowMapIsOrtho = true;
        self.shadowIntervalSize = 2.0;
        self.sunPathRotation = 0.0;
        self.shadowHardwareFilteringMask = 0;
        self.shadowPassInterval = 0;
        self.shadowPassCounter = 0;
        self.renderShadowTranslucent = true;
        self.targets.invalidate();
    }

    pub fn disableAfterRuntimeError(&mut self, error: &anyhow::Error) {
        log::error!(
            "Disabling OptiFine shader pack {:?} after runtime framebuffer error: {error:#}",
            self.selectedName,
        );
        self.clearPrograms();
        // Retain the loaded dimension so the same broken selection is not
        // recompiled every frame. A user selection/reload resets it explicitly.
        self.active = false;
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
    }

    pub fn prepareScene(&mut self, frame: &ShaderFrameState, extent: RendererExtent) -> anyhow::Result<bool> {
        self.ensureLoaded(frame.dimension)?;
        if !self.active {
            unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
            return Ok(false);
        }
        let now = Instant::now();
        self.currentFrameTime = now.duration_since(self.lastFrame).as_secs_f32().clamp(0.0, 1.0);
        self.lastFrame = now;
        self.frameTimeCounter = (self.frameTimeCounter + self.currentFrameTime) % 3600.0;
        self.updateShadowUniformMatrices(frame);
        self.renderExtent = scaledExtent(extent, self.renderResMul);
        self.targets.beginScene(self.renderExtent, frame.clearColor)?;
        Ok(true)
    }

    /// Binds the exact OptiFine 1.12.2 G-buffer program (or its declared
    /// backup) for one Minecraft render stage. `false` means the caller must
    /// use its vanilla compatibility program and attachment zero.
    pub fn bindGbufferProgram(
        &mut self,
        draw: GbufferDrawState,
        frame: &ShaderFrameState,
        _extent: RendererExtent,
    ) -> bool {
        if !self.active {
            return false;
        }
        let Some(index) = self.resolveGbufferProgram(draw.program as usize) else {
            return false;
        };
        let noiseTexture = self.noiseTexture;
        let Some(program) = self.gbufferPrograms[index].as_mut() else {
            return false;
        };
        unsafe {
            // `ShadersRender.beginBlockDamage` has one special fallback rule:
            // when gbuffers_damagedblock resolves to the terrain program it
            // writes only gcolor and disables depth writes. Retain that exact
            // distinction instead of applying the terrain program's full MRT.
            let damagedBlockTerrainFallback = draw.program == GbufferProgram::DamagedBlock
                && index == GbufferProgram::Terrain as usize;
            if damagedBlockTerrainFallback {
                gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
                gl::DepthMask(gl::FALSE);
            } else {
                // G-buffer attachment routing is a fixed-size OptiFine state
                // (at most COLOR_BUFFER_COUNT entries). Building a temporary
                // Vec here allocated on every program bind, which is directly
                // on the shader-pack render hot path. Preserve the exact
                // declared attachment order in stack storage instead.
                let mut attachments = [gl::NONE; COLOR_BUFFER_COUNT];
                let mut attachmentCount = 0_usize;
                for index in program.drawBuffers.iter().copied() {
                    if index >= COLOR_BUFFER_COUNT {
                        continue;
                    }
                    if attachmentCount == COLOR_BUFFER_COUNT {
                        break;
                    }
                    attachments[attachmentCount] = gl::COLOR_ATTACHMENT0 + index as GLenum;
                    attachmentCount += 1;
                }
                if attachmentCount == 0 {
                    gl::DrawBuffer(gl::COLOR_ATTACHMENT0);
                } else {
                    gl::DrawBuffers(attachmentCount as GLsizei, attachments.as_ptr());
                }
            }
        }
        self.targets.bindGbufferDepthTextures();
        self.shadowTargets.bindTextures();
        bindNoiseTexture(noiseTexture);
        useGbufferProgram(
            program,
            draw,
            frame,
            self.renderExtent,
            self.frameCounter,
            self.currentFrameTime,
            self.frameTimeCounter,
            &self.previousProjection,
            &self.previousModelView,
            self.previousCameraPosition,
            &self.shadowProjection,
            &self.shadowProjectionInverse,
            &self.shadowModelView,
            &self.shadowModelViewInverse,
        );
        true
    }

    fn resolveGbufferProgram(&self, requested: usize) -> Option<usize> {
        resolveGbufferProgramIndex(requested, |index| {
            self.gbufferPrograms.get(index).and_then(Option::as_ref).is_some()
        })
    }

    /// `ShadersRender.preWater` / `beginTranslucent`: preserve the opaque-world
    /// depth image in depthtex1 before translucent geometry changes depthtex0.
    pub fn captureDepthBeforeTranslucent(&self) {
        if self.active {
            self.targets.copyDepthSnapshot(1, 11);
        }
    }

    /// `Shaders.beginWeather`: preserve the completed world/translucent depth
    /// image in depthtex2 before weather and first-person rendering. The shared
    /// frame currently has no separate weather mesh, so this boundary is placed
    /// immediately before hand rendering, matching the existing source order.
    pub fn captureDepthBeforeWeather(&self) {
        if self.active {
            self.targets.copyDepthSnapshot(2, 12);
        }
    }

    pub fn finishScene(&mut self, frame: &ShaderFrameState, extent: RendererExtent) -> anyhow::Result<()> {
        if !self.active {
            unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
            return Ok(());
        }
        let frameTime = self.currentFrameTime;
        let noiseTexture = self.noiseTexture;
        let renderExtent = if self.renderExtent.width == 0 || self.renderExtent.height == 0 { extent } else { self.renderExtent };
        unsafe {
            gl::Viewport(0, 0, renderExtent.width as GLsizei, renderExtent.height as GLsizei);
            gl::Disable(gl::DEPTH_TEST);
            gl::DepthMask(gl::FALSE);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::BindVertexArray(self.fullscreenVao);
        }

        for program in &mut self.programs {
            self.targets.bindReadTextures();
            self.targets.generateCompositeMipmaps(program.compositeMipmapMask);
            self.shadowTargets.bindTextures();
            bindNoiseTexture(noiseTexture);
            self.targets.beginCompositePass(&program.drawBuffers);
            usePackProgram(
                program,
                frame,
                renderExtent,
                self.frameCounter,
                frameTime,
                self.frameTimeCounter,
                &self.previousProjection,
                &self.previousModelView,
                self.previousCameraPosition,
                &self.shadowProjection,
                &self.shadowProjectionInverse,
                &self.shadowModelView,
                &self.shadowModelViewInverse,
            );
            unsafe { gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4); }
            self.targets.finishCompositePass(&program.drawBuffers);
        }

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
            gl::DrawBuffer(gl::BACK);
            gl::Viewport(0, 0, extent.width as GLsizei, extent.height as GLsizei);
            // Shaders.renderCompositeFinal clears Minecraft's main framebuffer
            // before the final pass. Restore a writable depth buffer first;
            // Batch 91 established why clearing under a false depth mask is not
            // equivalent OpenGL state.
            gl::DepthMask(gl::TRUE);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::ClearColor(
                frame.clearColor[0],
                frame.clearColor[1],
                frame.clearColor[2],
                frame.clearColor[3],
            );
            gl::ClearDepth(1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Disable(gl::DEPTH_TEST);
            gl::DepthMask(gl::FALSE);
        }
        self.targets.bindReadTextures();
        if let Some(program) = self.finalProgram.as_ref() {
            self.targets.generateCompositeMipmaps(program.compositeMipmapMask);
        }
        self.shadowTargets.bindTextures();
        bindNoiseTexture(noiseTexture);
        if let Some(program) = self.finalProgram.as_mut() {
            usePackProgram(
                program,
                frame,
                renderExtent,
                self.frameCounter,
                frameTime,
                self.frameTimeCounter,
                &self.previousProjection,
                &self.previousModelView,
                self.previousCameraPosition,
                &self.shadowProjection,
                &self.shadowProjectionInverse,
                &self.shadowModelView,
                &self.shadowModelViewInverse,
            );
        } else {
            unsafe {
                gl::UseProgram(self.blitProgram);
                let name = CString::new("colortex0").expect("static sampler name");
                let location = gl::GetUniformLocation(self.blitProgram, name.as_ptr());
                if location >= 0 { gl::Uniform1i(location, 0); }
            }
        }
        unsafe {
            gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
            gl::BindVertexArray(0);
            gl::UseProgram(0);
        }
        self.previousProjection = openGlProjection(frame.projectionMatrix);
        self.previousModelView = frame.modelViewMatrix;
        self.previousCameraPosition = frame.cameraPosition;
        self.frameCounter = self.frameCounter.wrapping_add(1) & 0x000F_FFFF;
        Ok(())
    }

    /// Begins the OptiFine 1.12.2 `ShadersRender.renderShadowMap` traversal.
    /// Program 30 may legitimately have ID zero; in that case the original
    /// client uses the compatibility fixed pipeline, represented here by the
    /// explicit `fixedShadowProgram` bridge.
    pub fn beginShadowPass(&mut self) -> bool {
        if !self.active
            || !self.shadowTargets.isActive()
            || self.shadowPassInterval <= 0
        {
            return false;
        }
        self.shadowPassCounter -= 1;
        if self.shadowPassCounter > 0 {
            return false;
        }
        self.shadowPassCounter = self.shadowPassInterval;
        self.shadowTargets.begin();
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
            gl::DepthMask(gl::TRUE);
            gl::ColorMask(gl::TRUE, gl::TRUE, gl::TRUE, gl::TRUE);
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
        }
        true
    }

    pub fn bindShadowProgram(
        &mut self,
        draw: GbufferDrawState,
        frame: &ShaderFrameState,
        _extent: RendererExtent,
    ) -> bool {
        if !self.shadowTargets.isActive() {
            return false;
        }
        let noiseTexture = self.noiseTexture;
        unsafe {
            if self.shadowTargets.usedColorBuffers == 0 {
                gl::DrawBuffer(gl::NONE);
            } else {
                let mut attachments = [gl::NONE; SHADOW_COLOR_BUFFER_COUNT];
                for (index, attachment) in attachments
                    .iter_mut()
                    .take(self.shadowTargets.usedColorBuffers)
                    .enumerate()
                {
                    *attachment = gl::COLOR_ATTACHMENT0 + index as GLenum;
                }
                gl::DrawBuffers(
                    self.shadowTargets.usedColorBuffers as GLsizei,
                    attachments.as_ptr(),
                );
            }
        }
        bindNoiseTexture(noiseTexture);
        let (shadowPrograms, fixedShadowProgram) =
            (&mut self.shadowPrograms, &mut self.fixedShadowProgram);
        let program = shadowPrograms[0].as_mut().unwrap_or(fixedShadowProgram);
        useShadowProgram(
            program,
            draw,
            frame,
            self.renderExtent,
            self.frameCounter,
            self.currentFrameTime,
            self.frameTimeCounter,
            &self.previousProjection,
            &self.previousModelView,
            self.previousCameraPosition,
            &self.shadowProjection,
            &self.shadowProjectionInverse,
            &self.shadowModelView,
            &self.shadowModelViewInverse,
        );
        true
    }

    pub fn captureOpaqueShadowDepth(&self) {
        self.shadowTargets.copyOpaqueDepth();
    }

    pub fn shouldRenderShadowTranslucent(&self) -> bool {
        self.renderShadowTranslucent
    }

    pub fn shadowLightPositionVector(&self) -> [f32; 4] {
        self.shadowLightPositionVector
    }

    /// Builds the same camera-frustum silhouette extrusion used by OptiFine
    /// 1.12.2 `ClippingHelperShadow#getInstance`. Minecraft's fixed-function
    /// model-view at this point contains camera rotation but RenderChunk
    /// translations are supplied separately, so remove the absolute camera
    /// translation from the shared world-space view matrix before extracting
    /// planes.
    pub fn shadowCullingHelper(&self, frame: &ShaderFrameState) -> ClippingHelperShadow {
        let projection = openGlProjection(frame.projectionMatrix);
        let modelView = multiply4(
            frame.modelViewMatrix,
            translation4(
                frame.cameraPosition[0],
                frame.cameraPosition[1],
                frame.cameraPosition[2],
            ),
        );
        ClippingHelperShadow::fromMatrices(
            projection,
            modelView,
            self.shadowLightPositionVector,
        )
    }

    pub fn finishShadowPass(&mut self, _displayExtent: RendererExtent) {
        self.shadowTargets.finish();
        // `ShadersRender.renderShadowMap` restores the deferred framebuffer at
        // its render-resolution dimensions, not the physical window size.
        // Using the display extent here breaks renderResMul != 1.0 and makes
        // subsequent G-buffer draws address only part of the DFB.
        self.targets.restoreSceneFramebuffer(self.renderExtent);
        self.shadowTargets.bindTextures();
        unsafe {
            gl::DepthMask(gl::TRUE);
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LEQUAL);
            gl::Enable(gl::CULL_FACE);
            gl::Disable(gl::BLEND);
            gl::UseProgram(0);
        }
    }

    fn updateShadowUniformMatrices(&mut self, frame: &ShaderFrameState) {
        let projection = if self.shadowMapIsOrtho {
            orthographic4(
                -self.shadowMapHalfPlane,
                self.shadowMapHalfPlane,
                -self.shadowMapHalfPlane,
                self.shadowMapHalfPlane,
                0.05,
                256.0,
            )
        } else {
            perspective4(self.shadowMapFov, 1.0, 0.05, 256.0)
        };

        // Exact matrix order from OptiFine 1.12.2 `Shaders.setCameraShadow`.
        // Chunk/entity vertices are translated by -camera during the actual
        // shadow draw; this stored base matrix remains the uniform value that
        // OptiFine exposes as `shadowModelView`.
        let celestialDegrees = frame.celestialAngle * -360.0;
        let sunAngle = if frame.celestialAngle < 0.75 {
            frame.celestialAngle + 0.25
        } else {
            frame.celestialAngle - 0.75
        };
        let lightDegrees = if sunAngle <= 0.5 {
            celestialDegrees
        } else {
            celestialDegrees + 180.0
        };
        let mut modelView = multiply4(
            translation4(0.0, 0.0, -100.0),
            rotationX4(90.0_f32.to_radians()),
        );
        modelView = multiply4(modelView, rotationZ4(lightDegrees.to_radians()));
        modelView = multiply4(modelView, rotationX4(self.sunPathRotation.to_radians()));
        if self.shadowMapIsOrtho && self.shadowIntervalSize.abs() > 1.0e-6 {
            let interval = self.shadowIntervalSize;
            let half = interval * 0.5;
            modelView = multiply4(
                modelView,
                translation4(
                    frame.cameraPosition[0] % interval - half,
                    frame.cameraPosition[1] % interval - half,
                    frame.cameraPosition[2] % interval - half,
                ),
            );
        }

        // Port the source's `shadowLightPositionVector` calculation verbatim;
        // ClippingHelperShadow consumes this world-space direction.
        let angle = sunAngle * std::f32::consts::TAU;
        let cosine = angle.cos();
        let sine = angle.sin();
        let path = self.sunPathRotation * std::f32::consts::TAU;
        let mut light = [cosine, sine * path.cos(), sine * path.sin(), 0.0];
        if sunAngle > 0.5 {
            light[0] = -light[0];
            light[1] = -light[1];
            light[2] = -light[2];
        }

        self.shadowProjection = projection;
        self.shadowProjectionInverse = inverse4(projection).unwrap_or_else(identity4);
        self.shadowModelView = modelView;
        self.shadowModelViewInverse = inverse4(modelView).unwrap_or_else(identity4);
        self.shadowLightPositionVector = light;
    }

    fn setNoiseTexture(&mut self, resolution: Option<i32>) -> anyhow::Result<()> {
        let resolution = resolution.unwrap_or(0);
        if resolution == self.noiseTextureResolution
            && (resolution == 0 || self.noiseTexture != 0)
        {
            return Ok(());
        }
        self.destroyNoiseTexture();
        if resolution <= 0 {
            return Ok(());
        }
        let mut maximum = 0;
        unsafe { gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut maximum); }
        anyhow::ensure!(
            maximum > 0 && resolution <= maximum,
            "noiseTextureResolution {resolution} exceeds OpenGL GL_MAX_TEXTURE_SIZE {maximum}",
        );
        let image = generateHfNoiseImage(resolution, resolution)?;
        let mut texture = 0;
        unsafe {
            gl::GenTextures(1, &mut texture);
            gl::ActiveTexture(gl::TEXTURE0 + 15);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as GLint,
                resolution,
                resolution,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                image.as_ptr().cast::<c_void>(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as GLint);
            gl::ActiveTexture(gl::TEXTURE0);
        }
        anyhow::ensure!(texture != 0, "OptiFine HFNoiseTexture allocation failed");
        self.noiseTexture = texture;
        self.noiseTextureResolution = resolution;
        log::info!("OptiFine HFNoiseTexture enabled: {resolution}x{resolution}, texture unit 15");
        Ok(())
    }

    fn destroyNoiseTexture(&mut self) {
        if self.noiseTexture != 0 {
            unsafe { gl::DeleteTextures(1, &self.noiseTexture); }
            self.noiseTexture = 0;
        }
        self.noiseTextureResolution = 0;
    }

    fn ensureLoaded(&mut self, dimension: i32) -> anyhow::Result<()> {
        if self.loadedDimension == Some(dimension) {
            return Ok(());
        }
        let loadStarted = Instant::now();
        self.clearPrograms();
        self.targets.setColorFormats([gl::RGBA8 as GLint; COLOR_BUFFER_COUNT]);
        self.loadedDimension = Some(dimension);
        if self.selectedName.is_empty()
            || self.selectedName == packNameNone
            || self.selectedName == packNameDefault
        {
            self.active = false;
            return Ok(());
        }

        let shaders = Shaders::loadConfig(&self.gameDir);
        self.renderResMul = shaders.configRenderResMul;
        self.shadowResMul = shaders.configShadowResMul;
        self.handDepthMul = shaders.configHandDepthMul;
        let mut pack = shaders.loadShaderPack(None);
        if pack.getName() == packNameNone {
            log::warn!("Selected shader pack {:?} is unavailable; shaders remain off", self.selectedName);
            self.active = false;
            return Ok(());
        }
        // OptiFine 1.12.2 `Shaders.loadShaderPackDimensions` probes every
        // world-128..world128 directory before parsing the global option model.
        let optionStarted = Instant::now();
        let shaderPackDimensions = detectShaderPackDimensions(&mut *pack);
        let packOptions = ShaderPackOptions::loadCachedForLanguage(
            &self.gameDir,
            &mut *pack,
            &shaderPackDimensions,
            "en_US",
        )
        .with_context(|| format!("failed loading shader options for {}", pack.getName()))?;
        log::info!(
            "OptiFine shader option model ready for {} in {:.3}s",
            pack.getName(),
            optionStarted.elapsed().as_secs_f64(),
        );
        let environment = currentMacroEnvironment(&shaders, &packOptions);
        // ShaderPackParser.parseDimensions selects worldN only when that
        // directory exists. It does not merge root and dimension programs.
        let dimensionDirectory = format!("/shaders/world{dimension}");
        let programDimension = pack.hasDirectory(&dimensionDirectory).then_some(dimension);
        let mut failures = Vec::new();
        let mut colorFormats = [gl::RGBA8 as GLint; COLOR_BUFFER_COUNT];
        let mut noiseTextureResolution = None;
        for (index, name) in GBUFFER_NAMES.iter().copied().enumerate() {
            if packOptions.isProgramDisabled(name) {
                log::info!("OptiFine profile disabled program {name}");
                continue;
            }
            match loadPackProgram(
                &mut *pack,
                name,
                programDimension,
                PackProgramStage::Gbuffer,
                &environment,
                &packOptions,
            ) {
                Ok(Some(program)) => {
                    mergeColorFormats(&mut colorFormats, &program, name);
                    mergeNoiseResolution(&mut noiseTextureResolution, &program);
                    self.gbufferPrograms[index] = Some(program);
                }
                Ok(None) => {}
                Err(error) => failures.push(format!("{name}: {error:#}")),
            }
        }
        for (index, name) in SHADOW_NAMES.iter().copied().enumerate() {
            if packOptions.isProgramDisabled(name) {
                log::info!("OptiFine profile disabled program {name}");
                continue;
            }
            match loadPackProgram(
                &mut *pack,
                name,
                programDimension,
                PackProgramStage::Shadow,
                &environment,
                &packOptions,
            ) {
                Ok(Some(program)) => {
                    mergeNoiseResolution(&mut noiseTextureResolution, &program);
                    self.shadowPrograms[index] = Some(program);
                }
                Ok(None) => {}
                Err(error) => failures.push(format!("{name}: {error:#}")),
            }
        }
        for name in COMPOSITE_NAMES {
            if packOptions.isProgramDisabled(name) {
                log::info!("OptiFine profile disabled program {name}");
                continue;
            }
            match loadPackProgram(
                &mut *pack,
                name,
                programDimension,
                PackProgramStage::Composite,
                &environment,
                &packOptions,
            ) {
                Ok(Some(program)) => {
                    mergeColorFormats(&mut colorFormats, &program, name);
                    mergeNoiseResolution(&mut noiseTextureResolution, &program);
                    self.programs.push(program);
                }
                Ok(None) => {}
                Err(error) => failures.push(format!("{name}: {error:#}")),
            }
        }
        if packOptions.isProgramDisabled("final") {
            log::info!("OptiFine profile disabled program final");
        } else {
        match loadPackProgram(
            &mut *pack,
            "final",
            programDimension,
            PackProgramStage::Final,
            &environment,
            &packOptions,
        ) {
            Ok(Some(program)) => {
                mergeColorFormats(&mut colorFormats, &program, "final");
                mergeNoiseResolution(&mut noiseTextureResolution, &program);
                self.finalProgram = Some(program);
            }
            Ok(None) => {}
            Err(error) => failures.push(format!("final: {error:#}")),
        }
        }

        let mut usedColorBuffers = 4_usize;
        let mut usedDepthBuffers = 1_usize;
        let mut usedShadowDepthBuffers = 0_usize;
        let mut usedShadowColorBuffers = 0_usize;
        let mut clearDisabledMask = 0_u32;
        let mut shadowHardwareFilteringMask = 0_u32;
        let mut shadowMapHalfPlane = None;
        let mut shadowMapFov = None;
        let mut shadowIntervalSize = None;
        let mut shadowMapResolution = None;
        let mut shadowMipmapMask = 0_u32;
        let mut shadowNearestMask = 0_u32;
        let mut shadowColorMipmapMask = 0_u32;
        let mut shadowColorNearestMask = 0_u32;
        let mut sunPathRotation = None;
        let mut collectUsage = |program: &PackProgram| {
            clearDisabledMask |= program.clearDisabledMask;
            shadowHardwareFilteringMask |= program.shadowHardwareFilteringMask;
            if program.shadowMapHalfPlane.is_some() {
                shadowMapHalfPlane = program.shadowMapHalfPlane;
                shadowMapFov = None;
            }
            if program.shadowMapFov.is_some() {
                shadowMapFov = program.shadowMapFov;
                shadowMapHalfPlane = None;
            }
            if program.shadowIntervalSize.is_some() { shadowIntervalSize = program.shadowIntervalSize; }
            if program.shadowMapResolution.is_some() {
                shadowMapResolution = program.shadowMapResolution;
            }
            shadowMipmapMask |= program.shadowMipmapMask;
            shadowNearestMask |= program.shadowNearestMask;
            shadowColorMipmapMask |= program.shadowColorMipmapMask;
            shadowColorNearestMask |= program.shadowColorNearestMask;
            if program.sunPathRotation.is_some() { sunPathRotation = program.sunPathRotation; }
            usedColorBuffers = usedColorBuffers.max(program.requiredColorBuffers);
            usedDepthBuffers = usedDepthBuffers.max(program.requiredDepthBuffers);
            usedShadowDepthBuffers =
                usedShadowDepthBuffers.max(program.requiredShadowDepthBuffers);
            usedShadowColorBuffers =
                usedShadowColorBuffers.max(program.requiredShadowColorBuffers);
            if let Some(highest) = program.drawBuffers.iter().max().copied() {
                usedColorBuffers = usedColorBuffers.max(highest + 1);
            }
        };
        for program in self.gbufferPrograms.iter().filter_map(Option::as_ref) {
            collectUsage(program);
        }
        for program in &self.programs {
            collectUsage(program);
        }
        if let Some(program) = self.finalProgram.as_ref() {
            collectUsage(program);
        }
        // End the mutable captures before reading the shadow-only program
        // configuration. Shadow draw buffers belong to the SFB and must not
        // expand the deferred framebuffer's colortex count.
        drop(collectUsage);
        for program in self.shadowPrograms.iter().filter_map(Option::as_ref) {
            shadowHardwareFilteringMask |= program.shadowHardwareFilteringMask;
            if program.shadowMapHalfPlane.is_some() {
                shadowMapHalfPlane = program.shadowMapHalfPlane;
                shadowMapFov = None;
            }
            if program.shadowMapFov.is_some() {
                shadowMapFov = program.shadowMapFov;
                shadowMapHalfPlane = None;
            }
            if program.shadowIntervalSize.is_some() {
                shadowIntervalSize = program.shadowIntervalSize;
            }
            if program.shadowMapResolution.is_some() {
                shadowMapResolution = program.shadowMapResolution;
            }
            shadowMipmapMask |= program.shadowMipmapMask;
            shadowNearestMask |= program.shadowNearestMask;
            shadowColorMipmapMask |= program.shadowColorMipmapMask;
            shadowColorNearestMask |= program.shadowColorNearestMask;
            if program.sunPathRotation.is_some() {
                sunPathRotation = program.sunPathRotation;
            }
            usedShadowDepthBuffers =
                usedShadowDepthBuffers.max(program.requiredShadowDepthBuffers);
            usedShadowColorBuffers =
                usedShadowColorBuffers.max(program.requiredShadowColorBuffers);
        }
        usedColorBuffers = usedColorBuffers.clamp(1, COLOR_BUFFER_COUNT);
        usedDepthBuffers = usedDepthBuffers.clamp(1, 3);
        for program in self.gbufferPrograms.iter_mut().filter_map(Option::as_mut) {
            if !program.drawBuffersExplicit {
                program.drawBuffers = (0..usedColorBuffers).collect();
            }
        }
        for program in &mut self.programs {
            if !program.drawBuffersExplicit {
                program.drawBuffers = (0..usedColorBuffers).collect();
            }
        }
        pack.close();
        let renderShadowTranslucent = packOptions.shadowTranslucent;
        self.packOptions = Some(packOptions);
        self.targets.setUsage(usedColorBuffers, usedDepthBuffers);
        self.targets.setClearDisabledMask(clearDisabledMask);
        self.targets.setColorFormats(colorFormats);
        self.shadowHardwareFilteringMask = shadowHardwareFilteringMask & 0b11;
        self.shadowTargets.configure(
            ((shadowMapResolution.unwrap_or(1024) as f32 * self.shadowResMul).round() as i32).max(1),
            usedShadowDepthBuffers,
            usedShadowColorBuffers,
            shadowHardwareFilteringMask,
            shadowMipmapMask,
            shadowNearestMask,
            shadowColorMipmapMask,
            shadowColorNearestMask,
        )?;
        self.shadowPassInterval = if usedShadowDepthBuffers > 0 { 1 } else { 0 };
        self.shadowPassCounter = 0;
        self.renderShadowTranslucent = renderShadowTranslucent;
        self.shadowMapHalfPlane = shadowMapHalfPlane.unwrap_or(160.0).max(1.0);
        self.shadowMapFov = shadowMapFov.unwrap_or(90.0).clamp(1.0, 179.0);
        self.shadowMapIsOrtho = shadowMapFov.is_none();
        self.shadowIntervalSize = shadowIntervalSize.unwrap_or(2.0).max(0.0);
        self.sunPathRotation = sunPathRotation.unwrap_or(0.0);
        self.setNoiseTexture(noiseTextureResolution)?;
        log::info!(
            "OptiFine deferred color formats: {}",
            colorFormats
                .iter()
                .map(|format| textureFormatName(*format).unwrap_or("UNKNOWN"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        log::info!(
            "OptiFine buffer usage: color={}, depth={}, shadow_depth={}, shadow_color={}",
            usedColorBuffers,
            usedDepthBuffers,
            usedShadowDepthBuffers,
            usedShadowColorBuffers,
        );
        if clearDisabledMask != 0 {
            log::info!("OptiFine persistent color buffers (clear=false): 0x{clearDisabledMask:02X}");
        }
        let gbufferCount = self.gbufferPrograms.iter().filter(|program| program.is_some()).count();
        self.active = gbufferCount > 0 || !self.programs.is_empty() || self.finalProgram.is_some();
        if self.active {
            log::info!(
                "Loaded OptiFine shader execution stage {:?}: gbuffers={}, shadow={}, composite_passes={}, final={}, total_load={:.3}s",
                self.selectedName,
                gbufferCount,
                self.shadowTargets.isActive(),
                self.programs.len(),
                self.finalProgram.is_some(),
                loadStarted.elapsed().as_secs_f64(),
            );
        } else {
            log::warn!(
                "Shader pack {:?} has no usable gbuffers/composite/final program for dimension {}; vanilla OpenGL remains active",
                self.selectedName,
                dimension,
            );
        }
        for failure in failures {
            log::error!("OptiFine shader program failed: {failure}");
        }
        Ok(())
    }

    fn clearPrograms(&mut self) {
        for program in &mut self.gbufferPrograms {
            if let Some(mut program) = program.take() { program.destroy(); }
        }
        for program in &mut self.programs { program.destroy(); }
        self.programs.clear();
        if let Some(mut program) = self.finalProgram.take() { program.destroy(); }
        for program in &mut self.shadowPrograms {
            if let Some(mut program) = program.take() { program.destroy(); }
        }
        self.destroyNoiseTexture();
        self.packOptions = None;
        self.active = false;
    }

    pub fn destroy(&mut self) {
        self.clearPrograms();
        self.targets.destroy();
        self.shadowTargets.destroy();
        self.fixedShadowProgram.destroy();
        unsafe {
            if self.blitProgram != 0 { gl::DeleteProgram(self.blitProgram); }
            if self.fullscreenBuffer != 0 { gl::DeleteBuffers(1, &self.fullscreenBuffer); }
            if self.fullscreenVao != 0 { gl::DeleteVertexArrays(1, &self.fullscreenVao); }
        }
        self.blitProgram = 0;
        self.fullscreenBuffer = 0;
        self.fullscreenVao = 0;
    }
}

fn createFixedShadowProgram() -> anyhow::Result<PackProgram> {
    let vertex = adaptGbufferVertexShader(FIXED_GBUFFER_VERTEX_SHADER);
    let fragment = adaptGbufferFragmentShader(FIXED_SHADOW_FRAGMENT_SHADER);
    let id = compileProgram(
        "internal_fixed_shadow",
        &vertex,
        &fragment,
        &GBUFFER_ATTRIBUTES,
    )?;
    Ok(PackProgram {
        id,
        drawBuffers: Vec::new(),
        drawBuffersExplicit: false,
        requiredColorBuffers: 0,
        requiredDepthBuffers: 0,
        requiredShadowDepthBuffers: 0,
        requiredShadowColorBuffers: 0,
        colorFormats: [None; COLOR_BUFFER_COUNT],
        usesGdepthUniform: false,
        noiseTextureResolution: None,
        compositeMipmapMask: 0,
        clearDisabledMask: 0,
        shadowHardwareFilteringMask: 0,
        shadowMapHalfPlane: None,
        shadowMapFov: None,
        shadowIntervalSize: None,
        shadowMapResolution: None,
        shadowMipmapMask: 0,
        shadowNearestMask: 0,
        shadowColorMipmapMask: 0,
        shadowColorNearestMask: 0,
        sunPathRotation: None,
        uniforms: HashMap::new(),
    })
}

fn loadPackProgram(
    pack: &mut dyn IShaderPack,
    name: &str,
    dimension: Option<i32>,
    stage: PackProgramStage,
    environment: &ShaderMacroEnvironment,
    options: &ShaderPackOptions,
) -> anyhow::Result<Option<PackProgram>> {
    let base = match dimension {
        Some(dimension) => format!("/shaders/world{dimension}/{name}"),
        None => format!("/shaders/{name}"),
    };
    let Some((rawVertex, rawFragment)) = loadProgramPair(pack, &base, stage, environment, options)? else {
        return Ok(None);
    };
    let usage = detectProgramResourceUsage(&rawVertex, &rawFragment);
    let drawBufferSetting = if stage == PackProgramStage::Final {
        None
    } else {
        parseDrawBuffers(&rawFragment)
    };
    let colorFormats = parseColorFormats(&rawFragment);
    let usesGdepthUniform = hasUniform(&rawFragment, "gdepth");
    let noiseTextureResolution = parseConstInt(&rawFragment, "noiseTextureResolution");
    let compositeStage = matches!(stage, PackProgramStage::Composite | PackProgramStage::Final);
    let compositeMipmapMask = if compositeStage {
        parseCompositeMipmapMask(&rawFragment)
    } else {
        0
    };
    let clearDisabledMask = if compositeStage {
        parseClearDisabledMask(&rawFragment)
    } else {
        0
    };
    let shadowHardwareFilteringMask =
        parseShadowHardwareFilteringMask(&rawVertex) | parseShadowHardwareFilteringMask(&rawFragment);
    let shadowMapHalfPlane = parseConstFloat(&rawFragment, "shadowDistance")
        .or_else(|| parseLegacyCommentFloat(&rawFragment, "SHADOWHPL"))
        .or_else(|| parseConstFloat(&rawVertex, "shadowDistance"))
        .or_else(|| parseLegacyCommentFloat(&rawVertex, "SHADOWHPL"));
    let shadowMapFov = parseConstFloat(&rawFragment, "shadowMapFov")
        .or_else(|| parseLegacyCommentFloat(&rawFragment, "SHADOWFOV"))
        .or_else(|| parseConstFloat(&rawVertex, "shadowMapFov"))
        .or_else(|| parseLegacyCommentFloat(&rawVertex, "SHADOWFOV"));
    let shadowIntervalSize = parseConstFloat(&rawFragment, "shadowIntervalSize")
        .or_else(|| parseConstFloat(&rawVertex, "shadowIntervalSize"));
    let shadowMapResolution = parseConstInt(&rawFragment, "shadowMapResolution")
        .or_else(|| parseLegacyCommentInt(&rawFragment, "SHADOWRES"))
        .or_else(|| parseConstInt(&rawVertex, "shadowMapResolution"))
        .or_else(|| parseLegacyCommentInt(&rawVertex, "SHADOWRES"));
    let shadowMipmapMask =
        parseShadowMipmapMask(&rawVertex) | parseShadowMipmapMask(&rawFragment);
    let shadowNearestMask =
        parseShadowNearestMask(&rawVertex) | parseShadowNearestMask(&rawFragment);
    let shadowColorMipmapMask =
        parseShadowColorMipmapMask(&rawVertex) | parseShadowColorMipmapMask(&rawFragment);
    let shadowColorNearestMask =
        parseShadowColorNearestMask(&rawVertex) | parseShadowColorNearestMask(&rawFragment);
    let sunPathRotation = parseConstFloat(&rawFragment, "sunPathRotation")
        .or_else(|| parseConstFloat(&rawVertex, "sunPathRotation"));
    let (vertex, fragment, attributes): (String, String, &[(GLuint, &str)]) = match stage {
        PackProgramStage::Gbuffer | PackProgramStage::Shadow => (
            adaptGbufferVertexShader(&rawVertex),
            adaptGbufferFragmentShader(&rawFragment),
            &GBUFFER_ATTRIBUTES,
        ),
        PackProgramStage::Composite | PackProgramStage::Final => (
            adaptFullscreenVertexShader(&rawVertex),
            adaptLegacyFragmentShader(&rawFragment),
            &FULLSCREEN_ATTRIBUTES,
        ),
    };
    let candidates = adaptProgramCandidates(vertex, fragment);
    let mut id = None;
    let mut failures = Vec::new();
    let mut lastSources = None;
    for (strategy, vertex, fragment) in candidates {
        match compileProgram(name, &vertex, &fragment, attributes) {
            Ok(program) => {
                if strategy != "extensions" {
                    log::info!("Loaded OptiFine GLSL program {base} using {strategy} adaptation");
                }
                id = Some(program);
                break;
            }
            Err(error) => {
                failures.push(format!("{strategy}: {error:#}"));
                lastSources = Some((vertex, fragment));
            }
        }
    }
    let Some(id) = id else {
        if shouldDumpAdaptedShaderSources() {
            if let Some((vertex, fragment)) = lastSources {
                dumpAdaptedShaderSources(pack.getName(), name, &vertex, &fragment);
            }
        }
        return Err(anyhow!(failures.join("\nretry ")));
    };
    log::info!("Loaded OptiFine GLSL program {base}");
    Ok(Some(PackProgram {
        id,
        drawBuffers: drawBufferSetting.clone().unwrap_or_default(),
        drawBuffersExplicit: drawBufferSetting.is_some(),
        requiredColorBuffers: usage.colorBuffers,
        requiredDepthBuffers: usage.depthBuffers,
        requiredShadowDepthBuffers: usage.shadowDepthBuffers,
        requiredShadowColorBuffers: usage.shadowColorBuffers,
        colorFormats,
        usesGdepthUniform,
        noiseTextureResolution,
        compositeMipmapMask,
        clearDisabledMask,
        shadowHardwareFilteringMask,
        shadowMapHalfPlane,
        shadowMapFov,
        shadowIntervalSize,
        shadowMapResolution,
        shadowMipmapMask,
        shadowNearestMask,
        shadowColorMipmapMask,
        shadowColorNearestMask,
        sunPathRotation,
        uniforms: HashMap::new(),
    }))
}

fn loadProgramPair(
    pack: &mut dyn IShaderPack,
    base: &str,
    stage: PackProgramStage,
    environment: &ShaderMacroEnvironment,
    options: &ShaderPackOptions,
) -> anyhow::Result<Option<(String, String)>> {
    let vertexPath = format!("{base}.vsh");
    let fragmentPath = format!("{base}.fsh");
    let vertex = loadShaderSourceWithOptions(pack, &vertexPath, environment, Some(options))
        .with_context(|| format!("failed reading {vertexPath}"))?;
    let fragment = loadShaderSourceWithOptions(pack, &fragmentPath, environment, Some(options))
        .with_context(|| format!("failed reading {fragmentPath}"))?;
    let (fixedVertex, fixedFragment) = match stage {
        PackProgramStage::Gbuffer => {
            (FIXED_GBUFFER_VERTEX_SHADER, FIXED_GBUFFER_FRAGMENT_SHADER)
        }
        PackProgramStage::Shadow => {
            (FIXED_GBUFFER_VERTEX_SHADER, FIXED_SHADOW_FRAGMENT_SHADER)
        }
        PackProgramStage::Composite | PackProgramStage::Final => {
            (FIXED_COMPOSITE_VERTEX_SHADER, FIXED_COMPOSITE_FRAGMENT_SHADER)
        }
    };
    match (vertex, fragment) {
        (Some(vertex), Some(fragment)) => Ok(Some((vertex, fragment))),
        (Some(vertex), None) => Ok(Some((vertex, fixedFragment.to_owned()))),
        (None, Some(fragment)) => Ok(Some((fixedVertex.to_owned(), fragment))),
        (None, None) => Ok(None),
    }
}

fn adaptGbufferVertexShader(source: &str) -> String {
    let source = replaceGlslIdentifiers(
        source,
        &[
            ("gl_ModelViewProjectionMatrix", "mc112_modelview_projection"),
            ("gl_ModelViewProjectionMatrixInverse", "mc112_modelview_projection_inverse"),
            ("gl_ProjectionMatrix", "mc112_projection"),
            ("gl_ProjectionMatrixInverse", "mc112_projection_inverse"),
            ("gl_ModelViewMatrix", "mc112_modelview"),
            ("gl_ModelViewMatrixInverse", "mc112_modelview_inverse"),
            ("gl_NormalMatrix", "mc112_normal_matrix"),
            ("gl_TextureMatrix", "mc112_texture_matrix"),
            ("gl_Fog", "mc112_fog"),
            ("ftransform", "mc112_ftransform"),
            ("gl_Vertex", "mc112_vertex"),
            ("gl_Color", "mc112_color"),
            ("gl_Normal", "mc112_normal"),
            ("gl_MultiTexCoord0", "mc112_texcoord0"),
            ("gl_MultiTexCoord1", "mc112_texcoord1"),
            ("gl_MultiTexCoord2", "mc112_texcoord2"),
            ("gl_MultiTexCoord3", "mc112_texcoord3"),
        ],
    );
    let declaration = r#"
attribute vec3 mc112_position;
attribute vec4 mc112_texcoord0;
attribute vec4 mc112_color;
attribute vec2 mc112_lightmap;
attribute vec3 mc112_normal;
uniform mat4 mc112_projection;
uniform mat4 mc112_projection_inverse;
uniform mat4 mc112_modelview;
uniform mat4 mc112_modelview_inverse;
uniform mat4 mc112_modelview_projection;
uniform mat4 mc112_modelview_projection_inverse;
uniform mat3 mc112_normal_matrix;
uniform mat4 mc112_texture_matrix[8];
struct Mc112FogParameters {
    vec4 color;
    float density;
    float start;
    float end;
    float scale;
};
uniform Mc112FogParameters mc112_fog;
#define mc112_vertex vec4(mc112_position, 1.0)
#define mc112_texcoord1 vec4(mc112_lightmap * 16.0, 0.0, 1.0)
#define mc112_texcoord2 vec4(mc112_lightmap * 16.0, 0.0, 1.0)
#define mc112_texcoord3 vec4(mc112_lightmap * 16.0, 0.0, 1.0)
vec4 mc112_ftransform() { return mc112_modelview_projection * mc112_vertex; }
"#;
    insertAfterDirectives(&source, declaration)
}

fn adaptGbufferFragmentShader(source: &str) -> String {
    let source = replaceGlslIdentifiers(
        source,
        &[
            ("gl_ModelViewProjectionMatrix", "mc112_modelview_projection"),
            ("gl_ModelViewProjectionMatrixInverse", "mc112_modelview_projection_inverse"),
            ("gl_ProjectionMatrix", "mc112_projection"),
            ("gl_ProjectionMatrixInverse", "mc112_projection_inverse"),
            ("gl_ModelViewMatrix", "mc112_modelview"),
            ("gl_ModelViewMatrixInverse", "mc112_modelview_inverse"),
            ("gl_NormalMatrix", "mc112_normal_matrix"),
            ("gl_TextureMatrix", "mc112_texture_matrix"),
            ("gl_Fog", "mc112_fog"),
        ],
    );
    let declaration = r#"
uniform mat4 mc112_projection;
uniform mat4 mc112_projection_inverse;
uniform mat4 mc112_modelview;
uniform mat4 mc112_modelview_inverse;
uniform mat4 mc112_modelview_projection;
uniform mat4 mc112_modelview_projection_inverse;
uniform mat3 mc112_normal_matrix;
uniform mat4 mc112_texture_matrix[8];
struct Mc112FogParameters {
    vec4 color;
    float density;
    float start;
    float end;
    float scale;
};
uniform Mc112FogParameters mc112_fog;
"#;
    adaptLegacyFragmentShader(&insertAfterDirectives(&source, declaration))
}

fn promoteShaderVersion(source: &str, requested: i32) -> String {
    let Some(current) = source.lines().find_map(parseVersionDirective) else {
        return source.to_owned();
    };
    if current >= requested {
        return source.to_owned();
    }
    let mut output = String::with_capacity(source.len() + 8);
    let mut replaced = false;
    for line in source.split_inclusive('\n') {
        if !replaced && parseVersionDirective(line).is_some() {
            let suffix = if requested >= 150 || line.trim_end().ends_with(" compatibility") {
                " compatibility"
            } else {
                ""
            };
            output.push_str(&format!("#version {requested}{suffix}\n"));
            replaced = true;
        } else {
            output.push_str(line);
        }
    }
    output
}

#[derive(Clone, Copy)]
enum GlslProgramStage {
    Vertex,
    Fragment,
}

/// Produces a conservative extension-based candidate first, then a core-GLSL
/// compatibility candidate only when it differs. This avoids changing
/// `__VERSION__` branches in legacy OptiFine packs unless the driver rejects
/// the original language level.
fn adaptProgramCandidates(vertex: String, fragment: String) -> Vec<(&'static str, String, String)> {
    let legacy = adaptProgramPairExtensions(vertex.clone(), fragment.clone());
    let core = adaptProgramPair(vertex, fragment);
    let mut candidates = vec![("extensions", legacy.0, legacy.1)];
    if candidates[0].1 != core.0 || candidates[0].2 != core.1 {
        candidates.push(("core-compatibility", core.0, core.1));
    }
    candidates
}

fn adaptProgramPairExtensions(mut vertex: String, mut fragment: String) -> (String, String) {
    let vertexVersion = vertex.lines().find_map(parseVersionDirective).unwrap_or(120);
    let fragmentVersion = fragment.lines().find_map(parseVersionDirective).unwrap_or(120);
    let sharedVersion = vertexVersion.max(fragmentVersion);
    if vertexVersion < sharedVersion { vertex = promoteShaderVersion(&vertex, sharedVersion); }
    if fragmentVersion < sharedVersion { fragment = promoteShaderVersion(&fragment, sharedVersion); }
    if sharedVersion >= 130 {
        vertex = rewriteLegacyInterfaceQualifiers(&vertex, GlslProgramStage::Vertex);
        fragment = rewriteLegacyInterfaceQualifiers(&fragment, GlslProgramStage::Fragment);
    }
    if requiresGpuShader4(&vertex) || requiresGpuShader4(&fragment) {
        vertex = injectExtensionDirective(&vertex, "GL_EXT_gpu_shader4");
        fragment = injectExtensionDirective(&fragment, "GL_EXT_gpu_shader4");
    }
    if requiresShaderTextureLod(&vertex) || requiresShaderTextureLod(&fragment) {
        vertex = injectExtensionDirective(&vertex, "GL_ARB_shader_texture_lod");
        fragment = injectExtensionDirective(&fragment, "GL_ARB_shader_texture_lod");
    }
    (vertex, fragment)
}

/// Keep the source language selected by the pack whenever OpenGL extensions
/// are sufficient. OptiFine 1.12.2 does not blindly rewrite every shader to a
/// newer GLSL dialect. Promotion is reserved for functions which have no 1.20
/// extension spelling (for example generic `textureLod` and `modf`). When a
/// pair is promoted, legacy interface qualifiers are converted together so a
/// vertex/fragment pair cannot end up with `flat varying` under GLSL 1.30.
fn adaptProgramPair(vertex: String, fragment: String) -> (String, String) {
    let vertexVersion = vertex.lines().find_map(parseVersionDirective).unwrap_or(120);
    let fragmentVersion = fragment.lines().find_map(parseVersionDirective).unwrap_or(120);
    let required = requiredCoreGlslVersion(&vertex)
        .max(requiredCoreGlslVersion(&fragment))
        .max(vertexVersion)
        .max(fragmentVersion);

    let mut vertex = if vertexVersion < required {
        promoteShaderVersion(&vertex, required)
    } else {
        vertex
    };
    let mut fragment = if fragmentVersion < required {
        promoteShaderVersion(&fragment, required)
    } else {
        fragment
    };

    if required >= 130 {
        vertex = rewriteLegacyInterfaceQualifiers(&vertex, GlslProgramStage::Vertex);
        fragment = rewriteLegacyInterfaceQualifiers(&fragment, GlslProgramStage::Fragment);
    }

    if requiresGpuShader4(&vertex) || requiresGpuShader4(&fragment) {
        vertex = injectExtensionDirective(&vertex, "GL_EXT_gpu_shader4");
        fragment = injectExtensionDirective(&fragment, "GL_EXT_gpu_shader4");
    }
    if requiresShaderTextureLod(&vertex) || requiresShaderTextureLod(&fragment) {
        vertex = injectExtensionDirective(&vertex, "GL_ARB_shader_texture_lod");
        fragment = injectExtensionDirective(&fragment, "GL_ARB_shader_texture_lod");
    }

    // GLSL 1.30 introduces the generic `texture(...)` built-in. Legacy
    // Minecraft programs also conventionally declare a sampler named
    // `texture`. NVIDIA resolves the declaration as an object and then rejects
    // calls to the built-in as "cannot call a non-function" after promotion.
    // Rename only the sampler identifier (declarations and non-call uses), and
    // only in the promoted compatibility candidate. The untouched extension
    // candidate remains byte-for-byte faithful to the pack apart from the
    // required fixed-function bridge.
    if hasSamplerUniformNamed(&vertex, "texture") && hasGlslFunctionCall(&vertex, "texture") {
        vertex = renameGlslIdentifierExceptCalls(&vertex, "texture", "mc112_texture_sampler");
    }
    if hasSamplerUniformNamed(&fragment, "texture") && hasGlslFunctionCall(&fragment, "texture") {
        fragment = renameGlslIdentifierExceptCalls(&fragment, "texture", "mc112_texture_sampler");
    }

    (vertex, fragment)
}

fn requiredCoreGlslVersion(source: &str) -> i32 {
    let code = stripGlslComments(source);
    let identifiers130 = [
        "textureLod", "textureLodOffset", "textureProjLod", "textureProjLodOffset",
        "textureGrad", "textureGradOffset", "textureProjGrad", "textureProjGradOffset",
        "modf", "isnan", "isinf", "trunc", "round", "roundEven",
    ];
    if identifiers130.iter().any(|name| containsGlslIdentifier(&code, name)) {
        return 130;
    }
    // The boolean-selector overload of mix is core in 1.30. A precise type
    // parser would be excessive here; requiring both mix and a bvec token is a
    // narrow, deterministic indication matching the NVIDIA diagnostics from
    // real packs.
    if containsGlslIdentifier(&code, "mix")
        && ["bvec2", "bvec3", "bvec4"].iter().any(|name| containsGlslIdentifier(&code, name))
    {
        // GLSL 1.30 and GL_EXT_gpu_shader4 provide the boolean-selector
        // overloads used by legacy OptiFine packs. Do not jump to a 4.x source
        // language: Minecraft 1.12.2 shader packs are expected to remain valid
        // on the compatibility context requested by the original client.
        return 130;
    }
    // Explicit modern interface declarations cannot be compiled as GLSL 1.20.
    if containsGlslIdentifier(&code, "layout")
        || ["inverse", "determinant", "transpose", "outerProduct"]
            .iter()
            .any(|name| containsGlslIdentifier(&code, name))
    {
        return 140;
    }
    120
}

fn requiresGpuShader4(source: &str) -> bool {
    let code = stripGlslComments(source);
    let identifiers = [
        "uint", "uvec2", "uvec3", "uvec4", "usampler1D", "usampler2D",
        "usampler3D", "usamplerCube", "isampler1D", "isampler2D", "isampler3D",
        "isamplerCube", "texelFetch1D", "texelFetch2D", "texelFetch3D",
        "texelFetchBuffer", "textureSize1D", "textureSize2D", "textureSize3D",
    ];
    if identifiers.iter().any(|name| containsGlslIdentifier(&code, name)) {
        return true;
    }
    if containsGlslIdentifier(&code, "flat") && containsGlslIdentifier(&code, "varying") {
        return true;
    }
    code.contains("<<")
        || code.contains(">>")
        || code.contains('%')
        || containsSingleBitwiseOperator(&code, '&')
        || containsSingleBitwiseOperator(&code, '|')
        || containsSingleBitwiseOperator(&code, '^')
        || code.contains('~')
}

fn containsSingleBitwiseOperator(source: &str, operator: char) -> bool {
    let bytes = source.as_bytes();
    let requested = operator as u8;
    for index in 0..bytes.len() {
        if bytes[index] != requested { continue; }
        let previous = index.checked_sub(1).and_then(|i| bytes.get(i)).copied();
        let next = bytes.get(index + 1).copied();
        if previous != Some(requested) && next != Some(requested) {
            return true;
        }
    }
    false
}

fn requiresShaderTextureLod(source: &str) -> bool {
    let code = stripGlslComments(source);
    [
        "texture1DLod", "texture2DLod", "texture3DLod", "textureCubeLod",
        "texture1DProjLod", "texture2DProjLod", "texture1DGradARB",
        "texture2DGradARB", "textureCubeGradARB",
    ]
    .iter()
    .any(|name| containsGlslIdentifier(&code, name))
}

fn injectExtensionDirective(source: &str, extension: &str) -> String {
    if source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("#extension") && trimmed.contains(extension)
    }) {
        return source.to_owned();
    }
    let directive = format!("#extension {extension} : enable\n");
    let mut output = String::with_capacity(source.len() + directive.len());
    let mut inserted = false;
    for line in source.split_inclusive('\n') {
        output.push_str(line);
        if !inserted && parseVersionDirective(line).is_some() {
            output.push_str(&directive);
            inserted = true;
        }
    }
    if !inserted {
        output.insert_str(0, &directive);
    }
    output
}

fn rewriteLegacyInterfaceQualifiers(source: &str, stage: GlslProgramStage) -> String {
    let mut output = String::with_capacity(source.len());
    let mut inBlockComment = false;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            output.push_str(line);
            continue;
        }
        let mut rewritten = String::with_capacity(line.len());
        let bytes = line.as_bytes();
        let mut index = 0usize;
        while index < bytes.len() {
            if inBlockComment {
                if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                    rewritten.push_str("*/");
                    index += 2;
                    inBlockComment = false;
                } else {
                    rewritten.push(bytes[index] as char);
                    index += 1;
                }
                continue;
            }
            if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
                rewritten.push_str("/*");
                index += 2;
                inBlockComment = true;
                continue;
            }
            if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'/' {
                rewritten.push_str(&line[index..]);
                index = bytes.len();
                continue;
            }
            if bytes[index] == b'_' || bytes[index].is_ascii_alphabetic() {
                let start = index;
                index += 1;
                while index < bytes.len() && (bytes[index] == b'_' || bytes[index].is_ascii_alphanumeric()) {
                    index += 1;
                }
                let token = &line[start..index];
                match token {
                    "attribute" if matches!(stage, GlslProgramStage::Vertex) => rewritten.push_str("in"),
                    "varying" if matches!(stage, GlslProgramStage::Vertex) => rewritten.push_str("out"),
                    "varying" if matches!(stage, GlslProgramStage::Fragment) => rewritten.push_str("in"),
                    _ => rewritten.push_str(token),
                }
            } else {
                rewritten.push(bytes[index] as char);
                index += 1;
            }
        }
        output.push_str(&rewritten);
    }
    output
}

fn adaptLegacyFragmentShader(source: &str) -> String {
    // Pair-level adaptation is performed by `adaptProgramPair`, where the
    // vertex and fragment language versions and interface qualifiers can be
    // changed atomically. Keep this hook for the existing call sites.
    source.to_owned()
}

fn parseVersionDirective(line: &str) -> Option<i32> {
    let rest = line.trim_start().strip_prefix("#version")?.trim_start();
    rest.split_whitespace().next()?.parse().ok()
}

fn containsGlslIdentifier(source: &str, requested: &str) -> bool {
    let code = stripGlslComments(source);
    code.split(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
        .any(|identifier| identifier == requested)
}

fn hasSamplerUniformNamed(source: &str, requested: &str) -> bool {
    let code = stripGlslComments(source);
    code.lines().any(|line| {
        let mut tokens = line
            .split(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
            .filter(|token| !token.is_empty());
        tokens.next() == Some("uniform")
            && tokens.next().is_some_and(|ty| ty.contains("sampler"))
            && tokens.next() == Some(requested)
    })
}

fn hasGlslFunctionCall(source: &str, requested: &str) -> bool {
    let code = stripGlslComments(source);
    let bytes = code.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'_' || bytes[index].is_ascii_alphabetic() {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index] == b'_' || bytes[index].is_ascii_alphanumeric())
            {
                index += 1;
            }
            if &code[start..index] == requested {
                let mut next = index;
                while next < bytes.len() && bytes[next].is_ascii_whitespace() { next += 1; }
                if bytes.get(next) == Some(&b'(') { return true; }
            }
        } else {
            index += 1;
        }
    }
    false
}

fn renameGlslIdentifierExceptCalls(source: &str, requested: &str, replacement: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len() + 32);
    let mut index = 0usize;
    let mut inBlockComment = false;
    while index < bytes.len() {
        if inBlockComment {
            if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                output.push_str("*/");
                index += 2;
                inBlockComment = false;
            } else {
                output.push(bytes[index] as char);
                index += 1;
            }
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            output.push_str("/*");
            index += 2;
            inBlockComment = true;
            continue;
        }
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'/' {
            let start = index;
            while index < bytes.len() && bytes[index] != b'\n' { index += 1; }
            output.push_str(&source[start..index]);
            continue;
        }
        if bytes[index] == b'_' || bytes[index].is_ascii_alphabetic() {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index] == b'_' || bytes[index].is_ascii_alphanumeric())
            {
                index += 1;
            }
            let identifier = &source[start..index];
            if identifier == requested {
                let mut next = index;
                while next < bytes.len() && bytes[next].is_ascii_whitespace() { next += 1; }
                if bytes.get(next) == Some(&b'(') {
                    output.push_str(identifier);
                } else {
                    output.push_str(replacement);
                }
            } else {
                output.push_str(identifier);
            }
        } else if bytes[index].is_ascii() {
            output.push(bytes[index] as char);
            index += 1;
        } else {
            let character = source[index..].chars().next().expect("valid UTF-8 shader source");
            output.push(character);
            index += character.len_utf8();
        }
    }
    output
}

fn adaptFullscreenVertexShader(source: &str) -> String {
    // OptiFine draws a 0..1 quad under glOrtho(0, 1, 0, 1, 0, 1).
    // The internal VAO stores clip-space -1..1 positions. Composite programs
    // frequently multiply UVs by gl_TextureMatrix[n]. The shared renderer does
    // not maintain legacy texture-matrix state, so replace the eight standard
    // fixed indices with explicit identity matrices. This prevents the tiled
    // half-screen/quadrant output seen when a stale compatibility matrix is read.
    let mut source = source.to_owned();
    for index in 0..8 {
        source = source.replace(
            &format!("gl_TextureMatrix[{index}]"),
            &format!("mc112_texture_matrix_{index}"),
        );
    }
    // Replace only complete GLSL identifiers: substring replacement corrupts
    // names such as gl_ProjectionMatrixInverse and gl_VertexID.
    let source = replaceGlslIdentifiers(
        &source,
        &[
            ("gl_ModelViewProjectionMatrix", "mc112_projection"),
            ("gl_ProjectionMatrix", "mc112_projection"),
            ("gl_ModelViewMatrix", "mc112_modelview"),
            ("ftransform", "mc112_ftransform"),
            ("gl_Vertex", "mc112_vertex"),
            ("gl_MultiTexCoord0", "mc112_texcoord0"),
            ("gl_MultiTexCoord1", "mc112_texcoord1"),
            ("gl_MultiTexCoord2", "mc112_texcoord2"),
        ],
    );
    let declaration = r#"
attribute vec2 mc_position;
attribute vec2 mc_texcoord;
const mat4 mc112_projection = mat4(
     2.0,  0.0,  0.0, 0.0,
     0.0,  2.0,  0.0, 0.0,
     0.0,  0.0, -2.0, 0.0,
    -1.0, -1.0, -1.0, 1.0
);
const mat4 mc112_modelview = mat4(1.0);
const mat4 mc112_texture_matrix_0 = mat4(1.0);
const mat4 mc112_texture_matrix_1 = mat4(1.0);
const mat4 mc112_texture_matrix_2 = mat4(1.0);
const mat4 mc112_texture_matrix_3 = mat4(1.0);
const mat4 mc112_texture_matrix_4 = mat4(1.0);
const mat4 mc112_texture_matrix_5 = mat4(1.0);
const mat4 mc112_texture_matrix_6 = mat4(1.0);
const mat4 mc112_texture_matrix_7 = mat4(1.0);
#define mc112_vertex vec4(mc_position * 0.5 + vec2(0.5), 0.0, 1.0)
#define mc112_texcoord0 vec4(mc_texcoord, 0.0, 1.0)
#define mc112_texcoord1 vec4(mc_texcoord, 0.0, 1.0)
#define mc112_texcoord2 vec4(mc_texcoord, 0.0, 1.0)
vec4 mc112_ftransform() { return vec4(mc_position, 0.0, 1.0); }
"#;
    // Keep #extension directives ahead of ordinary declarations. Some 1.12.2
    // packs place them immediately after #version; inserting attributes before
    // those directives is rejected by conforming GLSL compilers.
    let mut insertion = 0usize;
    let mut inBlockComment = false;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let preprocessorOrComment = if inBlockComment {
            if trimmed.contains("*/") { inBlockComment = false; }
            true
        } else if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") { inBlockComment = true; }
            true
        } else {
            trimmed.is_empty()
                || trimmed.starts_with("//")
                || ["#version", "#extension", "#define", "#undef", "#line", "#pragma"]
                    .iter()
                    .any(|directive| trimmed.starts_with(directive))
        };
        if !preprocessorOrComment { break; }
        insertion += line.len();
    }
    let mut output = String::with_capacity(source.len() + declaration.len());
    output.push_str(&source[..insertion]);
    output.push_str(declaration);
    output.push_str(&source[insertion..]);
    output
}

fn insertAfterDirectives(source: &str, declaration: &str) -> String {
    // Keep #extension directives ahead of ordinary declarations. Some 1.12.2
    // packs place them immediately after #version; inserting attributes before
    // those directives is rejected by conforming GLSL compilers.
    let mut insertion = 0usize;
    let mut inBlockComment = false;
    for line in source.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let preprocessorOrComment = if inBlockComment {
            if trimmed.contains("*/") { inBlockComment = false; }
            true
        } else if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") { inBlockComment = true; }
            true
        } else {
            trimmed.is_empty()
                || trimmed.starts_with("//")
                || ["#version", "#extension", "#define", "#undef", "#line", "#pragma"]
                    .iter()
                    .any(|directive| trimmed.starts_with(directive))
        };
        if !preprocessorOrComment { break; }
        insertion += line.len();
    }
    let mut output = String::with_capacity(source.len() + declaration.len());
    output.push_str(&source[..insertion]);
    output.push_str(declaration);
    output.push_str(&source[insertion..]);
    output
}

fn replaceGlslIdentifiers(source: &str, replacements: &[(&str, &str)]) -> String {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len() + 64);
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte == b'_' || byte.is_ascii_alphabetic() {
            let start = index;
            index += 1;
            while index < bytes.len()
                && (bytes[index] == b'_' || bytes[index].is_ascii_alphanumeric())
            {
                index += 1;
            }
            let identifier = &source[start..index];
            if let Some((_, replacement)) = replacements
                .iter()
                .find(|(candidate, _)| *candidate == identifier)
            {
                output.push_str(replacement);
            } else {
                output.push_str(identifier);
            }
        } else if byte.is_ascii() {
            output.push(byte as char);
            index += 1;
        } else {
            let character = source[index..].chars().next().expect("valid UTF-8 shader source");
            output.push(character);
            index += character.len_utf8();
        }
    }
    output
}


const TEXTURE_FORMAT_NAMES: [&str; 37] = [
    "R8", "RG8", "RGB8", "RGBA8", "R8_SNORM", "RG8_SNORM", "RGB8_SNORM", "RGBA8_SNORM",
    "R16", "RG16", "RGB16", "RGBA16", "R16_SNORM", "RG16_SNORM", "RGB16_SNORM", "RGBA16_SNORM",
    "R16F", "RG16F", "RGB16F", "RGBA16F", "R32F", "RG32F", "RGB32F", "RGBA32F",
    "R32I", "RG32I", "RGB32I", "RGBA32I", "R32UI", "RG32UI", "RGB32UI", "RGBA32UI",
    "R3_G3_B2", "RGB5_A1", "RGB10_A2", "R11F_G11F_B10F", "RGB9_E5",
];

// Exact OptiFine 1.12.2 `Shaders.formatIds` table. Numeric constants are
// retained because several legacy formats are not named by every generated
// OpenGL binding version.
const TEXTURE_FORMAT_IDS: [GLint; 37] = [
    33321, 33323, 32849, 32856, 36756, 36757, 36758, 36759,
    33322, 33324, 32852, 32859, 36760, 36761, 36762, 36763,
    33325, 33327, 34843, 34842, 33326, 33328, 34837, 34836,
    33333, 33339, 36227, 36226, 33334, 33340, 36209, 36208,
    10768, 32855, 32857, 35898, 35901,
];

fn textureFormatFromString(value: &str) -> Option<GLint> {
    let value = value.trim();
    TEXTURE_FORMAT_NAMES
        .iter()
        .position(|name| *name == value)
        .map(|index| TEXTURE_FORMAT_IDS[index])
}

fn textureFormatName(value: GLint) -> Option<&'static str> {
    if value == gl::RGBA as GLint || value == gl::RGBA8 as GLint {
        return Some("RGBA8");
    }
    TEXTURE_FORMAT_IDS
        .iter()
        .position(|format| *format == value)
        .map(|index| TEXTURE_FORMAT_NAMES[index])
}

fn textureFormatComponents(value: GLint) -> usize {
    match value {
        33321 | 36756 | 33322 | 36760 | 33325 | 33326 | 33333 | 33334 => 1,
        33323 | 36757 | 33324 | 36761 | 33327 | 33328 | 33339 | 33340 => 2,
        32849 | 36758 | 32852 | 36762 | 34843 | 34837 | 36227 | 36209
            | 10768 | 35898 | 35901 => 3,
        _ => 4,
    }
}

fn isIntegerTextureFormat(value: GLint) -> bool {
    matches!(value, 33333 | 33339 | 36227 | 36226 | 33334 | 33340 | 36209 | 36208)
}

fn textureUploadFormat(internalFormat: GLint) -> (GLenum, GLenum) {
    let components = textureFormatComponents(internalFormat);
    if isIntegerTextureFormat(internalFormat) {
        let format = match components {
            1 => gl::RED_INTEGER,
            2 => gl::RG_INTEGER,
            3 => gl::RGB_INTEGER,
            _ => gl::RGBA_INTEGER,
        };
        let signed = matches!(internalFormat, 33333 | 33339 | 36227 | 36226);
        return (format, if signed { gl::INT } else { gl::UNSIGNED_INT });
    }
    let format = match components {
        1 => gl::RED,
        2 => gl::RG,
        3 => gl::RGB,
        _ => gl::RGBA,
    };
    (format, gl::FLOAT)
}

fn bufferIndexFromString(name: &str) -> Option<usize> {
    match name.trim() {
        "gcolor" | "colortex0" => Some(0),
        "gdepth" | "colortex1" => Some(1),
        "gnormal" | "colortex2" => Some(2),
        "composite" | "colortex3" => Some(3),
        "gaux1" | "colortex4" => Some(4),
        "gaux2" | "colortex5" => Some(5),
        "gaux3" | "colortex6" => Some(6),
        "gaux4" | "colortex7" => Some(7),
        _ => None,
    }
}

fn stripGlslComments(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut output = String::with_capacity(source.len());
    let mut index = 0usize;
    let mut blockComment = false;
    while index < bytes.len() {
        if blockComment {
            if index + 1 < bytes.len() && bytes[index] == b'*' && bytes[index + 1] == b'/' {
                blockComment = false;
                output.push_str("  ");
                index += 2;
            } else {
                output.push(if bytes[index] == b'\n' { '\n' } else { ' ' });
                index += 1;
            }
        } else if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            blockComment = true;
            output.push_str("  ");
            index += 2;
        } else if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'/' {
            while index < bytes.len() && bytes[index] != b'\n' {
                output.push(' ');
                index += 1;
            }
        } else if bytes[index].is_ascii() {
            output.push(bytes[index] as char);
            index += 1;
        } else {
            let character = source[index..].chars().next().expect("valid UTF-8 shader source");
            output.push(character);
            index += character.len_utf8();
        }
    }
    output
}

fn parseColorFormats(source: &str) -> [Option<GLint>; COLOR_BUFFER_COUNT] {
    let mut result = [None; COLOR_BUFFER_COUNT];
    let code = stripGlslComments(source);
    for line in code.lines() {
        let Some((declaration, value)) = line.split_once('=') else { continue; };
        let mut tokens = declaration.split_whitespace();
        if tokens.next() != Some("const") || tokens.next() != Some("int") {
            continue;
        }
        let Some(name) = tokens.next() else { continue; };
        if tokens.next().is_some() {
            continue;
        }
        let Some(bufferName) = name.strip_suffix("Format") else { continue; };
        let value = value.trim().trim_end_matches(';').split_whitespace().next().unwrap_or("");
        let Some(index) = bufferIndexFromString(bufferName) else { continue; };
        if let Some(format) = textureFormatFromString(value) {
            result[index] = Some(format);
        }
    }

    // Legacy pre-1.7 shader-pack comments retained by OptiFine 1.12.2.
    let compact = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    if compact.contains("GAUX4FORMAT:RGBA32F") {
        result[7] = textureFormatFromString("RGBA32F");
    } else if compact.contains("GAUX4FORMAT:RGB32F") {
        result[7] = textureFormatFromString("RGB32F");
    } else if compact.contains("GAUX4FORMAT:RGB16") {
        result[7] = textureFormatFromString("RGB16");
    }
    result
}

fn hasUniform(source: &str, name: &str) -> bool {
    let code = stripGlslComments(source);
    code.lines().any(|line| {
        let line = line.trim_start();
        line.starts_with("uniform ")
            && line
                .split(|character: char| !(character == '_' || character.is_ascii_alphanumeric()))
                .any(|token| token == name)
    })
}

fn bindNoiseTexture(texture: GLuint) {
    if texture == 0 {
        return;
    }
    unsafe {
        gl::ActiveTexture(gl::TEXTURE0 + 15);
        gl::BindTexture(gl::TEXTURE_2D, texture);
        gl::ActiveTexture(gl::TEXTURE0);
    }
}

fn parseClearDisabledMask(source: &str) -> u32 {
    let code = stripGlslComments(source);
    let mut mask = 0_u32;
    for line in code.lines() {
        let Some((declaration, value)) = line.split_once('=') else {
            continue;
        };
        let mut tokens = declaration.split_whitespace();
        if tokens.next() != Some("const") || tokens.next() != Some("bool") {
            continue;
        }
        let Some(name) = tokens.next() else {
            continue;
        };
        if tokens.next().is_some() || !name.ends_with("Clear") {
            continue;
        }
        let disabled = value
            .trim()
            .trim_end_matches(';')
            .split_whitespace()
            .next()
            == Some("false");
        if !disabled {
            continue;
        }
        let buffer = name.trim_end_matches("Clear");
        if let Some(index) = bufferIndexFromString(buffer) {
            mask |= 1_u32 << index;
        }
    }
    mask
}

fn parseCompositeMipmapMask(source: &str) -> u32 {
    let code = stripGlslComments(source);
    let mut mask = 0_u32;
    for line in code.lines() {
        let Some((declaration, value)) = line.split_once('=') else {
            continue;
        };
        let mut tokens = declaration.split_whitespace();
        if tokens.next() != Some("const") || tokens.next() != Some("bool") {
            continue;
        }
        let Some(name) = tokens.next() else {
            continue;
        };
        if tokens.next().is_some() || !name.ends_with("MipmapEnabled") {
            continue;
        }
        let enabled = value
            .trim()
            .trim_end_matches(';')
            .split_whitespace()
            .next()
            == Some("true");
        if !enabled {
            continue;
        }
        let buffer = name.trim_end_matches("MipmapEnabled");
        if let Some(index) = bufferIndexFromString(buffer) {
            mask |= 1_u32 << index;
        }
    }
    mask
}

fn parseConstInt(source: &str, requestedName: &str) -> Option<i32> {
    let code = stripGlslComments(source);
    for line in code.lines() {
        let Some((declaration, value)) = line.split_once('=') else { continue; };
        let mut tokens = declaration.split_whitespace();
        if tokens.next() != Some("const") || tokens.next() != Some("int") {
            continue;
        }
        if tokens.next() != Some(requestedName) || tokens.next().is_some() {
            continue;
        }
        let value = value.trim().trim_end_matches(';').split_whitespace().next()?;
        return value.parse::<i32>().ok();
    }
    None
}

fn mergeNoiseResolution(resolution: &mut Option<i32>, program: &PackProgram) {
    if let Some(requested) = program.noiseTextureResolution {
        *resolution = Some(requested);
    }
}

fn hfNoiseRandom(mut seed: i32) -> i32 {
    seed ^= seed.wrapping_shl(13);
    seed ^= seed >> 17;
    seed ^= seed.wrapping_shl(5);
    seed
}

fn hfNoiseSample(x: i32, y: i32, z: i32) -> i8 {
    let seed = hfNoiseRandom(x)
        .wrapping_add(hfNoiseRandom(y.wrapping_mul(19)))
        .wrapping_mul(hfNoiseRandom(z.wrapping_mul(23)))
        .wrapping_sub(z);
    (hfNoiseRandom(seed) % 128) as i8
}

fn generateHfNoiseImage(width: i32, height: i32) -> anyhow::Result<Vec<u8>> {
    anyhow::ensure!(width > 0 && height > 0, "noise texture dimensions must be positive");
    let length = (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(3))
        .context("noise texture dimensions overflow")?;
    let mut image = Vec::with_capacity(length);
    for y in 0..height {
        for x in 0..width {
            for channel in 1..4 {
                image.push(hfNoiseSample(x, y, channel) as u8);
            }
        }
    }
    Ok(image)
}

fn mergeColorFormats(formats: &mut [GLint; COLOR_BUFFER_COUNT], program: &PackProgram, name: &str) {
    // Shaders.java promotes gdepth to RGBA32F only while buffer 1 still has its
    // default format. Explicit `*Format` declarations always win thereafter.
    if program.usesGdepthUniform && formats[1] == gl::RGBA8 as GLint {
        formats[1] = textureFormatFromString("RGBA32F").expect("OptiFine format table");
    }
    for (index, requested) in program.colorFormats.iter().copied().enumerate() {
        let Some(requested) = requested else { continue; };
        if isIntegerTextureFormat(requested) {
            // Integer deferred targets require integer clear/write semantics in
            // every vanilla fallback program. Do not create a formally complete
            // FBO that the current compatibility fallback would write illegally.
            log::warn!(
                "OptiFine program {name} requests unsupported integer format {} for colortex{}; retaining {}",
                textureFormatName(requested).unwrap_or("UNKNOWN"),
                index,
                textureFormatName(formats[index]).unwrap_or("UNKNOWN"),
            );
            continue;
        }
        formats[index] = requested;
    }
}

fn detectProgramResourceUsage(vertex: &str, fragment: &str) -> ProgramResourceUsage {
    let mut usage = ProgramResourceUsage::default();
    for source in [vertex, fragment] {
        for (index, names) in [
            (0, &["gcolor", "colortex0"][..]),
            (1, &["gdepth", "colortex1"][..]),
            (2, &["gnormal", "colortex2"][..]),
            (3, &["composite", "colortex3"][..]),
            (4, &["gaux1", "colortex4"][..]),
            (5, &["gaux2", "colortex5"][..]),
            (6, &["gaux3", "colortex6"][..]),
            (7, &["gaux4", "colortex7"][..]),
        ] {
            if names.iter().any(|name| hasUniform(source, name)) {
                usage.colorBuffers = usage.colorBuffers.max(index + 1);
            }
        }
        for (index, names) in [
            (0, &["gdepthtex", "depthtex0"][..]),
            (1, &["depthtex1"][..]),
            (2, &["depthtex2"][..]),
        ] {
            if names.iter().any(|name| hasUniform(source, name)) {
                usage.depthBuffers = usage.depthBuffers.max(index + 1);
            }
        }
        for (index, names) in [
            (0, &["shadow", "watershadow", "shadowtex0"][..]),
            (1, &["shadowtex1"][..]),
        ] {
            if names.iter().any(|name| hasUniform(source, name)) {
                usage.shadowDepthBuffers = usage.shadowDepthBuffers.max(index + 1);
            }
        }
        for (index, names) in [
            (0, &["shadowcolor", "shadowcolor0"][..]),
            (1, &["shadowcolor1"][..]),
        ] {
            if names.iter().any(|name| hasUniform(source, name)) {
                usage.shadowColorBuffers = usage.shadowColorBuffers.max(index + 1);
            }
        }
    }
    usage
}

fn parseDrawBuffers(source: &str) -> Option<Vec<usize>> {
    if let Some(start) = source.find("DRAWBUFFERS:") {
        let mut result = Vec::new();
        for byte in source[start + "DRAWBUFFERS:".len()..].bytes() {
            if byte.is_ascii_digit() {
                let index = (byte - b'0') as usize;
                if index < COLOR_BUFFER_COUNT && !result.contains(&index) {
                    result.push(index);
                }
            } else if !matches!(byte, b' ' | b'\t') {
                break;
            }
        }
        if !result.is_empty() {
            return Some(result);
        }
    }
    if let Some(start) = source.find("RENDERTARGETS:") {
        let tail = &source[start + "RENDERTARGETS:".len()..];
        let line = tail.lines().next().unwrap_or("");
        let result = line
            .split(',')
            .filter_map(|value| {
                let digits = value
                    .trim()
                    .chars()
                    .take_while(|character| character.is_ascii_digit())
                    .collect::<String>();
                digits.parse::<usize>().ok()
            })
            .filter(|index| *index < COLOR_BUFFER_COUNT)
            .fold(Vec::new(), |mut result, index| {
                if !result.contains(&index) {
                    result.push(index);
                }
                result
            });
        if !result.is_empty() {
            return Some(result);
        }
    }
    None
}

fn parseConstBool(source: &str, requestedName: &str) -> Option<bool> {
    let code = stripGlslComments(source);
    for line in code.lines() {
        let Some((declaration, value)) = line.split_once('=') else {
            continue;
        };
        let mut declarationTokens = declaration.split_whitespace();
        if declarationTokens.next() != Some("const") || declarationTokens.next() != Some("bool") {
            continue;
        }
        if declarationTokens.next() != Some(requestedName) || declarationTokens.next().is_some() {
            continue;
        }

        let Some((literal, _trailing)) = value.trim().split_once(';') else {
            continue;
        };
        let mut literalTokens = literal.split_whitespace();
        let Some(literal) = literalTokens.next() else {
            continue;
        };
        if literalTokens.next().is_some() {
            continue;
        }
        if literal.eq_ignore_ascii_case("true") {
            return Some(true);
        }
        if literal.eq_ignore_ascii_case("false") {
            return Some(false);
        }
        return None;
    }
    None
}

fn parseConstFloat(source: &str, requestedName: &str) -> Option<f32> {
    let code = stripGlslComments(source);
    for line in code.lines() {
        let Some((declaration, value)) = line.split_once('=') else {
            continue;
        };
        let mut tokens = declaration.split_whitespace();
        if tokens.next() != Some("const") || tokens.next() != Some("float") {
            continue;
        }
        if tokens.next() != Some(requestedName) || tokens.next().is_some() {
            continue;
        }
        let value = value
            .trim()
            .trim_end_matches(';')
            .split_whitespace()
            .next()?
            .trim_end_matches('f')
            .trim_end_matches('F');
        return value.parse::<f32>().ok();
    }
    None
}

fn parseLegacyCommentFloat(source: &str, key: &str) -> Option<f32> {
    let key = key.to_ascii_uppercase();
    for line in source.lines() {
        let upper = line.to_ascii_uppercase();
        let Some(position) = upper.find(&key) else {
            continue;
        };
        let tail = &line[position + key.len()..];
        let tail = tail.trim_start_matches(|character: char| {
            character.is_whitespace() || matches!(character, ':' | '=')
        });
        let token = tail
            .split(|character: char| character.is_whitespace() || matches!(character, ';' | '*' | '/'))
            .next()?;
        if let Ok(value) = token.parse::<f32>() {
            return Some(value);
        }
    }
    None
}

fn parseLegacyCommentInt(source: &str, key: &str) -> Option<i32> {
    parseLegacyCommentFloat(source, key).map(|value| value as i32)
}

fn parseShadowMipmapMask(source: &str) -> u32 {
    let mut mask = 0_u32;
    if parseConstBool(source, "generateShadowMipmap") == Some(true) {
        mask |= 0b11;
    }
    if ["shadowtex0Mipmap", "shadowtexMipmap"]
        .into_iter()
        .any(|name| parseConstBool(source, name) == Some(true))
    {
        mask |= 0b01;
    }
    if parseConstBool(source, "shadowtex1Mipmap") == Some(true) {
        mask |= 0b10;
    }
    mask
}

fn parseShadowNearestMask(source: &str) -> u32 {
    let mut mask = 0_u32;
    if ["shadowtex0Nearest", "shadowtexNearest", "shadow0MinMagNearest"]
        .into_iter()
        .any(|name| parseConstBool(source, name) == Some(true))
    {
        mask |= 0b01;
    }
    if ["shadowtex1Nearest", "shadow1MinMagNearest"]
        .into_iter()
        .any(|name| parseConstBool(source, name) == Some(true))
    {
        mask |= 0b10;
    }
    mask
}

fn parseShadowColorMipmapMask(source: &str) -> u32 {
    let mut mask = 0_u32;
    if parseConstBool(source, "generateShadowColorMipmap") == Some(true) {
        mask |= 0b11;
    }
    if ["shadowcolor0Mipmap", "shadowColor0Mipmap"]
        .into_iter()
        .any(|name| parseConstBool(source, name) == Some(true))
    {
        mask |= 0b01;
    }
    if ["shadowcolor1Mipmap", "shadowColor1Mipmap"]
        .into_iter()
        .any(|name| parseConstBool(source, name) == Some(true))
    {
        mask |= 0b10;
    }
    mask
}

fn parseShadowColorNearestMask(source: &str) -> u32 {
    let mut mask = 0_u32;
    if ["shadowcolor0Nearest", "shadowColor0Nearest", "shadowColor0MinMagNearest"]
        .into_iter()
        .any(|name| parseConstBool(source, name) == Some(true))
    {
        mask |= 0b01;
    }
    if ["shadowcolor1Nearest", "shadowColor1Nearest", "shadowColor1MinMagNearest"]
        .into_iter()
        .any(|name| parseConstBool(source, name) == Some(true))
    {
        mask |= 0b10;
    }
    mask
}

fn parseShadowHardwareFilteringMask(source: &str) -> u32 {
    let mut mask = 0_u32;
    if parseConstBool(source, "shadowHardwareFiltering") == Some(true) {
        mask |= 0b11;
    }
    if parseConstBool(source, "shadowHardwareFiltering0") == Some(true) {
        mask |= 0b01;
    }
    if parseConstBool(source, "shadowHardwareFiltering1") == Some(true) {
        mask |= 0b10;
    }
    mask
}

fn detectShaderPackDimensions(pack: &mut dyn IShaderPack) -> Vec<i32> {
    (-128..=128)
        .filter(|dimension| pack.hasDirectory(&format!("/shaders/world{dimension}")))
        .collect()
}

fn scaledExtent(extent: RendererExtent, multiplier: f32) -> RendererExtent {
    let multiplier = if multiplier.is_finite() { multiplier.max(0.01) } else { 1.0 };
    RendererExtent {
        width: ((extent.width as f32 * multiplier).round() as u32).max(1),
        height: ((extent.height as f32 * multiplier).round() as u32).max(1),
    }
}

fn currentMacroEnvironment(
    config: &Shaders,
    packOptions: &ShaderPackOptions,
) -> ShaderMacroEnvironment {
    let vendor = glString(gl::VENDOR).unwrap_or_default().to_ascii_lowercase();
    let renderer = glString(gl::RENDERER).unwrap_or_default().to_ascii_lowercase();
    let version = glString(gl::VERSION).unwrap_or_default();
    let glsl = glString(gl::SHADING_LANGUAGE_VERSION).unwrap_or_default();
    let vendorMacro = if vendor.starts_with("nvidia") {
        "MC_GL_VENDOR_NVIDIA"
    } else if vendor.starts_with("intel") {
        "MC_GL_VENDOR_INTEL"
    } else if vendor.starts_with("ati") || vendor.starts_with("amd") {
        "MC_GL_VENDOR_ATI"
    } else if vendor.starts_with("x.org") {
        "MC_GL_VENDOR_XORG"
    } else {
        "MC_GL_VENDOR_OTHER"
    };
    let rendererMacro = if renderer.contains("geforce") || renderer.starts_with("nvidia") {
        "MC_GL_RENDERER_GEFORCE"
    } else if renderer.contains("quadro") || renderer.starts_with("nvs") {
        "MC_GL_RENDERER_QUADRO"
    } else if renderer.contains("radeon") || renderer.starts_with("amd") || renderer.starts_with("ati") {
        "MC_GL_RENDERER_RADEON"
    } else if renderer.starts_with("intel") {
        "MC_GL_RENDERER_INTEL"
    } else if renderer.starts_with("gallium") {
        "MC_GL_RENDERER_GALLIUM"
    } else if renderer.starts_with("mesa") {
        "MC_GL_RENDERER_MESA"
    } else {
        "MC_GL_RENDERER_OTHER"
    };
    let effectiveProperty = |configured: PropertyDefaultTrueFalse, pack: Option<bool>| {
        match configured {
            PropertyDefaultTrueFalse::True => true,
            PropertyDefaultTrueFalse::False => false,
            PropertyDefaultTrueFalse::Default => pack.unwrap_or(true),
        }
    };
    ShaderMacroEnvironment {
        glVersion: parseVersionNumber(&version).unwrap_or(330),
        glslVersion: parseVersionNumber(&glsl).unwrap_or(330),
        vendorMacro,
        rendererMacro,
        fxaaLevel: config.configAntialiasingLevel,
        // OptiFine exposes these macros from the corresponding global shader
        // options. Missing per-sprite maps use ShadersTex's exact default
        // normal/specular values; suppressing the macros changes source branch
        // structure and caused the logged `relative_position` failures.
        normalMap: config.configNormalMap,
        specularMap: config.configSpecularMap,
        renderQuality: config.configRenderResMul,
        shadowQuality: config.configShadowResMul,
        handDepth: config.configHandDepthMul,
        oldHandLight: effectiveProperty(config.configOldHandLight, packOptions.oldHandLight),
        oldLighting: effectiveProperty(config.configOldLighting, packOptions.oldLighting),
        extensionMacros: currentExtensionMacros(),
        ..ShaderMacroEnvironment::default()
    }
}

fn currentExtensionMacros() -> Vec<String> {
    let mut count = 0;
    unsafe { gl::GetIntegerv(gl::NUM_EXTENSIONS, &mut count); }
    let mut output = Vec::with_capacity(count.max(0) as usize);
    for index in 0..count.max(0) as u32 {
        let pointer = unsafe { gl::GetStringi(gl::EXTENSIONS, index) };
        if pointer.is_null() {
            continue;
        }
        let extension = unsafe { CStr::from_ptr(pointer.cast()) }.to_string_lossy();
        output.push(format!("MC_{extension}"));
    }
    output
}

fn parseVersionNumber(value: &str) -> Option<i32> {
    let mut parts = value.split(|ch: char| !ch.is_ascii_digit()).filter(|part| !part.is_empty());
    let major = parts.next()?.parse::<i32>().ok()?;
    let minor = parts.next()?.parse::<i32>().ok()?;
    Some(major * 100 + if minor >= 10 { minor } else { minor * 10 })
}

fn createFullscreenMesh() -> anyhow::Result<(GLuint, GLuint)> {
    let vertices: [f32; 16] = [
        -1.0, -1.0, 0.0, 0.0,
         1.0, -1.0, 1.0, 0.0,
        -1.0,  1.0, 0.0, 1.0,
         1.0,  1.0, 1.0, 1.0,
    ];
    let mut vao = 0;
    let mut buffer = 0;
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut buffer);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            std::mem::size_of_val(&vertices) as GLsizeiptr,
            vertices.as_ptr().cast::<c_void>(),
            gl::STATIC_DRAW,
        );
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 16, std::ptr::null());
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, 16, 8usize as *const c_void);
        gl::BindVertexArray(0);
    }
    anyhow::ensure!(vao != 0 && buffer != 0, "fullscreen shader mesh allocation failed");
    Ok((vao, buffer))
}

fn shouldDumpAdaptedShaderSources() -> bool {
    std::env::var("MC112_DUMP_FAILED_SHADERS")
        .ok()
        .is_some_and(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

fn dumpAdaptedShaderSources(packName: &str, programName: &str, vertex: &str, fragment: &str) {
    let sanitize = |value: &str| {
        value
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                    character
                } else {
                    '_'
                }
            })
            .collect::<String>()
    };
    let directory = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("shaderpacks")
        .join("debug-rust")
        .join(sanitize(packName));
    if std::fs::create_dir_all(&directory).is_err() {
        return;
    }
    let base = sanitize(programName);
    let vertexPath = directory.join(format!("{base}.adapted.vsh"));
    let fragmentPath = directory.join(format!("{base}.adapted.fsh"));
    let vertexWritten = std::fs::write(&vertexPath, vertex).is_ok();
    let fragmentWritten = std::fs::write(&fragmentPath, fragment).is_ok();
    if vertexWritten || fragmentWritten {
        log::warn!(
            "Saved failed adapted OptiFine shader sources under {}",
            directory.display(),
        );
    }
}

fn compileProgram(name: &str, vertexSource: &str, fragmentSource: &str, attributes: &[(GLuint, &str)]) -> anyhow::Result<GLuint> {
    let vertex = compileShader(name, gl::VERTEX_SHADER, vertexSource)?;
    let fragment = match compileShader(name, gl::FRAGMENT_SHADER, fragmentSource) {
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
        for &(location, attribute) in attributes {
            let attribute = CString::new(attribute).expect("static shader attribute name");
            gl::BindAttribLocation(program, location, attribute.as_ptr());
        }
        gl::LinkProgram(program);
        gl::DeleteShader(vertex);
        gl::DeleteShader(fragment);
    }
    let mut status = 0;
    unsafe { gl::GetProgramiv(program, gl::LINK_STATUS, &mut status); }
    if status == gl::TRUE as GLint { return Ok(program); }
    let log = programLog(program);
    unsafe { gl::DeleteProgram(program); }
    Err(anyhow!("{name} program link failed: {log}"))
}

fn compileShader(name: &str, kind: GLenum, source: &str) -> anyhow::Result<GLuint> {
    let shader = unsafe { gl::CreateShader(kind) };
    let source = CString::new(source).context("shader source contains NUL")?;
    unsafe {
        gl::ShaderSource(shader, 1, &source.as_ptr(), std::ptr::null());
        gl::CompileShader(shader);
    }
    let mut status = 0;
    unsafe { gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status); }
    if status == gl::TRUE as GLint { return Ok(shader); }
    let log = shaderLog(shader);
    unsafe { gl::DeleteShader(shader); }
    let stage = if kind == gl::VERTEX_SHADER { "vertex" } else { "fragment" };
    Err(anyhow!("{name} {stage} shader compilation failed: {log}"))
}

fn resolveGbufferProgramIndex(
    requested: usize,
    mut available: impl FnMut(usize) -> bool,
) -> Option<usize> {
    let mut current = requested;
    for _ in 0..GBUFFER_PROGRAM_COUNT {
        if current >= GBUFFER_PROGRAM_COUNT {
            return None;
        }
        if available(current) {
            return Some(current);
        }
        let backup = GBUFFER_BACKUPS[current];
        if backup == 0 {
            return None;
        }
        let next = backup - 1;
        if next == current {
            return None;
        }
        current = next;
    }
    None
}

fn useGbufferProgram(
    program: &mut PackProgram,
    draw: GbufferDrawState,
    frame: &ShaderFrameState,
    extent: RendererExtent,
    frameCounter: i32,
    frameTime: f32,
    frameTimeCounter: f32,
    previousProjection: &[f32; 16],
    previousModelView: &[f32; 16],
    previousCameraPosition: [f32; 3],
    shadowProjection: &[f32; 16],
    shadowProjectionInverse: &[f32; 16],
    shadowModelView: &[f32; 16],
    shadowModelViewInverse: &[f32; 16],
) {
    usePackProgram(
        program,
        frame,
        extent,
        frameCounter,
        frameTime,
        frameTimeCounter,
        previousProjection,
        previousModelView,
        previousCameraPosition,
        shadowProjection,
        shadowProjectionInverse,
        shadowModelView,
        shadowModelViewInverse,
    );
    for name in ["texture", "mc112_texture_sampler", "tex"] {
        uniform1i(program, name, 0);
    }
    uniform1i(program, "lightmap", 1);
    uniform1i(program, "normals", 2);
    uniform1i(program, "specular", 3);
    uniform1i(program, "shadow", 4);
    uniform1i(program, "watershadow", 4);
    uniform1i(program, "shadowtex0", 4);
    uniform1i(program, "shadowtex1", 5);
    uniform1i(program, "depthtex0", 6);
    // OptiFine sets gaux1..4 in G-buffer programs only when
    // customTexturesGbuffers is present. Those resources are not aliases for
    // deferred colortex4..7 and must not be bound to stale frame attachments.
    uniform1i(program, "depthtex1", 12);
    uniform1i(program, "shadowcolor", 13);
    uniform1i(program, "shadowcolor0", 13);
    uniform1i(program, "shadowcolor1", 14);
    uniform1i(program, "noisetex", 15);
    uniform1i(program, "entityId", draw.entityId);
    uniform1i(program, "blockEntityId", draw.blockEntityId);
    uniform4f(program, "entityColor", draw.entityColor);
    uniform2i(program, "atlasSize", draw.atlasSize);
    uniform2i(program, "terrainTextureSize", draw.atlasSize);
    uniform1i(program, "terrainIconSize", 16);
    uniform1i(program, "fogMode", gl::LINEAR as i32);
    uniform1f(program, "eyeAltitude", frame.cameraPosition[1]);

    let projection = openGlProjection(frame.projectionMatrix);
    let modelViewProjection = openGlProjection(draw.viewProjection);
    let modelView = inverse4(projection)
        .map(|inverseProjection| multiply4(inverseProjection, modelViewProjection))
        .unwrap_or(frame.modelViewMatrix);
    uniformMatrix4(program, "mc112_projection", &projection);
    uniformMatrix4(program, "mc112_modelview", &modelView);
    uniformMatrix4(program, "mc112_modelview_projection", &modelViewProjection);
    if let Some(inverse) = inverse4(projection) {
        uniformMatrix4(program, "mc112_projection_inverse", &inverse);
    }
    if let Some(inverse) = inverse4(modelView) {
        uniformMatrix4(program, "mc112_modelview_inverse", &inverse);
    }
    if let Some(inverse) = inverse4(modelViewProjection) {
        uniformMatrix4(program, "mc112_modelview_projection_inverse", &inverse);
    }
    uniform4f(program, "mc112_fog.color", draw.fogColor);
    uniform1f(program, "mc112_fog.density", 0.0);
    uniform1f(program, "mc112_fog.start", draw.fogParameters[0]);
    uniform1f(program, "mc112_fog.end", draw.fogParameters[1]);
    let fogSpan = draw.fogParameters[1] - draw.fogParameters[0];
    uniform1f(
        program,
        "mc112_fog.scale",
        if fogSpan.abs() > 1.0e-6 { fogSpan.recip() } else { 0.0 },
    );
    uniform3f(
        program,
        "fogColor",
        [draw.fogColor[0], draw.fogColor[1], draw.fogColor[2]],
    );
    uniform1f(program, "mc112_fog_start", draw.fogParameters[0]);
    uniform1f(program, "mc112_fog_end", draw.fogParameters[1]);
    uniform4f(program, "mc112_lightmap_parameters", draw.lightmapParameters);
    if let Some(normalMatrix) = inverseTranspose3(modelView) {
        uniformMatrix3(program, "mc112_normal_matrix", &normalMatrix);
    }
    let identity = identity4();
    let lightmap = [
        1.0 / 256.0, 0.0, 0.0, 0.0,
        0.0, 1.0 / 256.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        8.0 / 256.0, 8.0 / 256.0, 0.0, 1.0,
    ];
    for name in [
        "mc112_texture_matrix[0]",
        "mc112_texture_matrix[1]",
        "mc112_texture_matrix[2]",
        "mc112_texture_matrix[3]",
        "mc112_texture_matrix[4]",
        "mc112_texture_matrix[5]",
        "mc112_texture_matrix[6]",
        "mc112_texture_matrix[7]",
    ] {
        uniformMatrix4(program, name, &identity);
    }
    uniformMatrix4(program, "mc112_texture_matrix[1]", &lightmap);
}


fn useShadowProgram(
    program: &mut PackProgram,
    draw: GbufferDrawState,
    frame: &ShaderFrameState,
    extent: RendererExtent,
    frameCounter: i32,
    frameTime: f32,
    frameTimeCounter: f32,
    previousProjection: &[f32; 16],
    previousModelView: &[f32; 16],
    previousCameraPosition: [f32; 3],
    shadowProjection: &[f32; 16],
    shadowProjectionInverse: &[f32; 16],
    shadowModelView: &[f32; 16],
    shadowModelViewInverse: &[f32; 16],
) {
    usePackProgram(
        program,
        frame,
        extent,
        frameCounter,
        frameTime,
        frameTimeCounter,
        previousProjection,
        previousModelView,
        previousCameraPosition,
        shadowProjection,
        shadowProjectionInverse,
        shadowModelView,
        shadowModelViewInverse,
    );
    for name in ["texture", "mc112_texture_sampler", "tex"] {
        uniform1i(program, name, 0);
    }
    uniform1i(program, "lightmap", 1);
    uniform1i(program, "normals", 2);
    uniform1i(program, "specular", 3);
    uniform1i(program, "shadow", 4);
    uniform1i(program, "watershadow", 4);
    uniform1i(program, "shadowtex0", 4);
    uniform1i(program, "shadowtex1", 5);
    uniform1i(program, "depthtex0", 6);
    uniform1i(program, "depthtex1", 12);
    uniform1i(program, "shadowcolor", 13);
    uniform1i(program, "shadowcolor0", 13);
    uniform1i(program, "shadowcolor1", 14);
    uniform1i(program, "noisetex", 15);
    uniform1i(program, "entityId", draw.entityId);
    uniform1i(program, "blockEntityId", draw.blockEntityId);
    uniform4f(program, "entityColor", draw.entityColor);
    uniform2i(program, "atlasSize", draw.atlasSize);
    uniform2i(program, "terrainTextureSize", draw.atlasSize);
    uniform1i(program, "terrainIconSize", 16);
    uniform1i(program, "fogMode", gl::LINEAR as i32);
    uniform1f(program, "eyeAltitude", frame.cameraPosition[1]);
    uniform1f(program, "mc112_alpha_cutoff", draw.fogParameters[3]);

    // Vanilla RenderChunk/VBO draws translate every mesh by -camera after
    // Shaders.setCameraShadow. Shared Rust meshes store absolute world
    // coordinates, so apply the same translation explicitly here.
    let drawModelView = multiply4(
        *shadowModelView,
        translation4(
            -frame.cameraPosition[0],
            -frame.cameraPosition[1],
            -frame.cameraPosition[2],
        ),
    );
    let drawProjection = *shadowProjection;
    let drawModelViewProjection = multiply4(drawProjection, drawModelView);
    uniformMatrix4(program, "mc112_projection", &drawProjection);
    uniformMatrix4(program, "mc112_modelview", &drawModelView);
    uniformMatrix4(program, "mc112_modelview_projection", &drawModelViewProjection);
    uniformMatrix4(program, "mc112_projection_inverse", shadowProjectionInverse);
    if let Some(inverse) = inverse4(drawModelView) {
        uniformMatrix4(program, "mc112_modelview_inverse", &inverse);
    }
    if let Some(inverse) = inverse4(drawModelViewProjection) {
        uniformMatrix4(program, "mc112_modelview_projection_inverse", &inverse);
    }
    uniform4f(program, "mc112_fog.color", draw.fogColor);
    uniform1f(program, "mc112_fog.density", 0.0);
    uniform1f(program, "mc112_fog.start", draw.fogParameters[0]);
    uniform1f(program, "mc112_fog.end", draw.fogParameters[1]);
    let fogSpan = draw.fogParameters[1] - draw.fogParameters[0];
    uniform1f(
        program,
        "mc112_fog.scale",
        if fogSpan.abs() > 1.0e-6 { fogSpan.recip() } else { 0.0 },
    );
    uniform3f(
        program,
        "fogColor",
        [draw.fogColor[0], draw.fogColor[1], draw.fogColor[2]],
    );
    uniform1f(program, "mc112_fog_start", draw.fogParameters[0]);
    uniform1f(program, "mc112_fog_end", draw.fogParameters[1]);
    uniform4f(program, "mc112_lightmap_parameters", draw.lightmapParameters);
    if let Some(normalMatrix) = inverseTranspose3(drawModelView) {
        uniformMatrix3(program, "mc112_normal_matrix", &normalMatrix);
    }
    let identity = identity4();
    let lightmap = [
        1.0 / 256.0, 0.0, 0.0, 0.0,
        0.0, 1.0 / 256.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        8.0 / 256.0, 8.0 / 256.0, 0.0, 1.0,
    ];
    for name in [
        "mc112_texture_matrix[0]",
        "mc112_texture_matrix[1]",
        "mc112_texture_matrix[2]",
        "mc112_texture_matrix[3]",
        "mc112_texture_matrix[4]",
        "mc112_texture_matrix[5]",
        "mc112_texture_matrix[6]",
        "mc112_texture_matrix[7]",
    ] {
        uniformMatrix4(program, name, &identity);
    }
    uniformMatrix4(program, "mc112_texture_matrix[1]", &lightmap);
}

fn usePackProgram(
    program: &mut PackProgram,
    frame: &ShaderFrameState,
    extent: RendererExtent,
    frameCounter: i32,
    frameTime: f32,
    frameTimeCounter: f32,
    previousProjection: &[f32; 16],
    previousModelView: &[f32; 16],
    previousCameraPosition: [f32; 3],
    shadowProjection: &[f32; 16],
    shadowProjectionInverse: &[f32; 16],
    shadowModelView: &[f32; 16],
    shadowModelViewInverse: &[f32; 16],
) {
    unsafe { gl::UseProgram(program.id); }
    for (index, names) in [
        (0, ["gcolor", "colortex0"]),
        (1, ["gdepth", "colortex1"]),
        (2, ["gnormal", "colortex2"]),
        (3, ["composite", "colortex3"]),
        (4, ["gaux1", "colortex4"]),
        (5, ["gaux2", "colortex5"]),
        (6, ["gaux3", "colortex6"]),
        (7, ["gaux4", "colortex7"]),
    ] {
        for name in names {
            uniform1i(program, name, COLOR_TEXTURE_UNITS[index] as i32);
        }
    }
    for name in ["gdepthtex", "depthtex0"] { uniform1i(program, name, 6); }
    uniform1i(program, "depthtex1", 11);
    uniform1i(program, "depthtex2", 12);
    uniform1i(program, "shadow", 4);
    uniform1i(program, "watershadow", 4);
    uniform1i(program, "shadowtex0", 4);
    uniform1i(program, "shadowtex1", 5);
    uniform1i(program, "shadowcolor", 13);
    uniform1i(program, "shadowcolor0", 13);
    uniform1i(program, "shadowcolor1", 14);
    uniform1i(program, "noisetex", 15);
    uniform1f(program, "viewWidth", extent.width as f32);
    uniform1f(program, "viewHeight", extent.height as f32);
    uniform1f(program, "aspectRatio", extent.width.max(1) as f32 / extent.height.max(1) as f32);
    uniform1f(program, "near", 0.05);
    uniform1f(program, "far", frame.farPlane);
    uniform1i(program, "worldTime", frame.worldTime.rem_euclid(24000) as i32);
    uniform1i(program, "worldDay", frame.worldTime.div_euclid(24000) as i32);
    uniform1i(program, "moonPhase", ((frame.worldTime / 24000) & 7) as i32);
    uniform1i(program, "frameCounter", frameCounter);
    uniform1f(program, "frameTime", frameTime);
    uniform1f(program, "frameTimeCounter", frameTimeCounter);
    let sunAngle = if frame.celestialAngle < 0.75 {
        frame.celestialAngle + 0.25
    } else {
        frame.celestialAngle - 0.75
    };
    uniform1f(program, "sunAngle", sunAngle);
    let shadowAngle = if sunAngle <= 0.5 { sunAngle } else { sunAngle - 0.5 };
    uniform1f(program, "shadowAngle", shadowAngle);
    uniform1f(program, "rainStrength", 0.0);
    uniform1f(program, "wetness", 0.0);
    uniform1f(program, "eyeAltitude", frame.cameraPosition[1]);
    uniform2i(program, "eyeBrightness", frame.eyeBrightness);
    uniform2i(program, "eyeBrightnessSmooth", frame.eyeBrightness);
    uniform1i(program, "isEyeInWater", 0);
    uniform1f(program, "nightVision", 0.0);
    uniform1f(program, "blindness", 0.0);
    uniform1f(program, "screenBrightness", frame.screenBrightness);
    uniform2i(program, "terrainTextureSize", frame.atlasSize);
    uniform2i(program, "atlasSize", frame.atlasSize);
    uniform1i(program, "hideGUI", 0);
    uniform1f(program, "centerDepthSmooth", 1.0);
    uniform1i(program, "heldItemId", -1);
    uniform1i(program, "heldBlockLightValue", 0);
    uniform1i(program, "heldItemId2", -1);
    uniform1i(program, "heldBlockLightValue2", 0);
    uniform3f(program, "fogColor", frame.fogColor);
    uniform3f(program, "skyColor", frame.skyColor);
    uniform3f(program, "cameraPosition", frame.cameraPosition);
    uniform3f(program, "previousCameraPosition", previousCameraPosition);
    let projection = openGlProjection(frame.projectionMatrix);
    uniformMatrix4(program, "gbufferProjection", &projection);
    uniformMatrix4(program, "gbufferPreviousProjection", previousProjection);
    uniformMatrix4(program, "gbufferModelView", &frame.modelViewMatrix);
    uniformMatrix4(program, "gbufferPreviousModelView", previousModelView);
    if let Some(inverse) = inverse4(projection) {
        uniformMatrix4(program, "gbufferProjectionInverse", &inverse);
    }
    if let Some(inverse) = inverse4(frame.modelViewMatrix) {
        uniformMatrix4(program, "gbufferModelViewInverse", &inverse);
    }

    let celestial = frame.celestialAngle * std::f32::consts::TAU;
    let sunWorld = [celestial.cos() * 100.0, celestial.sin() * 100.0, 0.0, 0.0];
    let moonWorld = [-sunWorld[0], -sunWorld[1], -sunWorld[2], 0.0];
    let upWorld = [0.0, 100.0, 0.0, 0.0];
    let sunPosition = transform4(frame.modelViewMatrix, sunWorld);
    let moonPosition = transform4(frame.modelViewMatrix, moonWorld);
    let upPosition = transform4(frame.modelViewMatrix, upWorld);
    let shadowLightPosition = if shadowAngle == sunAngle {
        sunPosition
    } else {
        moonPosition
    };
    uniform3f(program, "sunPosition", [sunPosition[0], sunPosition[1], sunPosition[2]]);
    uniform3f(program, "moonPosition", [moonPosition[0], moonPosition[1], moonPosition[2]]);
    uniform3f(
        program,
        "shadowLightPosition",
        [
            shadowLightPosition[0],
            shadowLightPosition[1],
            shadowLightPosition[2],
        ],
    );
    uniform3f(program, "upPosition", [upPosition[0], upPosition[1], upPosition[2]]);

    // The real shadow traversal is not active yet, but OptiFine still exposes
    // the actual shadow-camera matrices to all programs. Pair neutral depth
    // textures with the source-equivalent camera transform rather than identity.
    uniformMatrix4(program, "shadowProjection", shadowProjection);
    uniformMatrix4(program, "shadowProjectionInverse", shadowProjectionInverse);
    uniformMatrix4(program, "shadowModelView", shadowModelView);
    uniformMatrix4(program, "shadowModelViewInverse", shadowModelViewInverse);
}

fn uniform1i(program: &mut PackProgram, name: &'static str, value: i32) {
    let location = program.uniform(name);
    if location >= 0 { unsafe { gl::Uniform1i(location, value); } }
}

fn uniform1f(program: &mut PackProgram, name: &'static str, value: f32) {
    let location = program.uniform(name);
    if location >= 0 { unsafe { gl::Uniform1f(location, value); } }
}

fn uniform2i(program: &mut PackProgram, name: &'static str, value: [i32; 2]) {
    let location = program.uniform(name);
    if location >= 0 { unsafe { gl::Uniform2i(location, value[0], value[1]); } }
}

fn uniform4f(program: &mut PackProgram, name: &'static str, value: [f32; 4]) {
    let location = program.uniform(name);
    if location >= 0 {
        unsafe { gl::Uniform4f(location, value[0], value[1], value[2], value[3]); }
    }
}

fn uniform3f(program: &mut PackProgram, name: &'static str, value: [f32; 3]) {
    let location = program.uniform(name);
    if location >= 0 { unsafe { gl::Uniform3f(location, value[0], value[1], value[2]); } }
}

fn uniformMatrix3(program: &mut PackProgram, name: &'static str, value: &[f32; 9]) {
    let location = program.uniform(name);
    if location >= 0 { unsafe { gl::UniformMatrix3fv(location, 1, gl::FALSE, value.as_ptr()); } }
}

fn uniformMatrix4(program: &mut PackProgram, name: &'static str, value: &[f32; 16]) {
    let location = program.uniform(name);
    if location >= 0 { unsafe { gl::UniformMatrix4fv(location, 1, gl::FALSE, value.as_ptr()); } }
}

fn translation4(x: f32, y: f32, z: f32) -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        x, y, z, 1.0,
    ]
}

fn rotationX4(angle: f32) -> [f32; 16] {
    let (sine, cosine) = angle.sin_cos();
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, cosine, sine, 0.0,
        0.0, -sine, cosine, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn rotationZ4(angle: f32) -> [f32; 16] {
    let (sine, cosine) = angle.sin_cos();
    [
        cosine, sine, 0.0, 0.0,
        -sine, cosine, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
}

fn orthographic4(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> [f32; 16] {
    let width = right - left;
    let height = top - bottom;
    let depth = far - near;
    [
        2.0 / width, 0.0, 0.0, 0.0,
        0.0, 2.0 / height, 0.0, 0.0,
        0.0, 0.0, -2.0 / depth, 0.0,
        -(right + left) / width,
        -(top + bottom) / height,
        -(far + near) / depth,
        1.0,
    ]
}

fn perspective4(fieldOfViewDegrees: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let reciprocalTangent = (fieldOfViewDegrees.to_radians() * 0.5).tan().recip();
    let depth = near - far;
    [
        reciprocalTangent / aspect.max(1.0e-6), 0.0, 0.0, 0.0,
        0.0, reciprocalTangent, 0.0, 0.0,
        0.0, 0.0, (far + near) / depth, -1.0,
        0.0, 0.0, (2.0 * far * near) / depth, 0.0,
    ]
}

fn openGlProjection(vulkanProjection: [f32; 16]) -> [f32; 16] {
    // Column-major C * P where C flips Vulkan Y and maps depth 0..w to -w..w.
    let conversion = [
        1.0, 0.0, 0.0, 0.0,
        0.0, -1.0, 0.0, 0.0,
        0.0, 0.0, 2.0, 0.0,
        0.0, 0.0, -1.0, 1.0,
    ];
    multiply4(conversion, vulkanProjection)
}

fn transform4(matrix: [f32; 16], vector: [f32; 4]) -> [f32; 4] {
    let mut output = [0.0_f32; 4];
    for row in 0..4 {
        output[row] = (0..4)
            .map(|column| matrix[column * 4 + row] * vector[column])
            .sum();
    }
    output
}

fn multiply4(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    let mut output = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            output[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    output
}

fn inverse4(matrix: [f32; 16]) -> Option<[f32; 16]> {
    let mut augmented = [[0.0_f32; 8]; 4];
    for row in 0..4 {
        for column in 0..4 { augmented[row][column] = matrix[column * 4 + row]; }
        augmented[row][4 + row] = 1.0;
    }
    for pivot in 0..4 {
        let swap = (pivot..4).max_by(|left, right| {
            augmented[*left][pivot].abs().total_cmp(&augmented[*right][pivot].abs())
        })?;
        if augmented[swap][pivot].abs() <= 1.0e-8 { return None; }
        if swap != pivot { augmented.swap(swap, pivot); }
        let divisor = augmented[pivot][pivot];
        for column in 0..8 { augmented[pivot][column] /= divisor; }
        for row in 0..4 {
            if row == pivot { continue; }
            let factor = augmented[row][pivot];
            for column in 0..8 { augmented[row][column] -= factor * augmented[pivot][column]; }
        }
    }
    let mut output = [0.0; 16];
    for row in 0..4 {
        for column in 0..4 { output[column * 4 + row] = augmented[row][4 + column]; }
    }
    Some(output)
}

fn inverseTranspose3(matrix: [f32; 16]) -> Option<[f32; 9]> {
    let inverse = inverse4(matrix)?;
    Some([
        inverse[0], inverse[4], inverse[8],
        inverse[1], inverse[5], inverse[9],
        inverse[2], inverse[6], inverse[10],
    ])
}

fn identity4() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ]
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

fn glString(name: GLenum) -> Option<String> {
    let pointer = unsafe { gl::GetString(name) };
    if pointer.is_null() { return None; }
    Some(unsafe { std::ffi::CStr::from_ptr(pointer.cast()) }.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_optifine_drawbuffers_and_render_targets() {
        assert_eq!(parseDrawBuffers("/* DRAWBUFFERS:024 */"), Some(vec![0, 2, 4]));
        assert_eq!(
            parseDrawBuffers("/* RENDERTARGETS: 1, 3, 7 */"),
            Some(vec![1, 3, 7])
        );
        assert_eq!(
            parseDrawBuffers("/* RENDERTARGETS: 0, 2, 2, 5 */"),
            Some(vec![0, 2, 5])
        );
        assert_eq!(parseDrawBuffers("void main(){}"), None);
    }

    #[test]
    fn detects_optifine_resource_counts_before_default_draw_buffers_are_assigned() {
        let usage = detectProgramResourceUsage(
            "#version 120\nuniform sampler2D shadowtex1;\n",
            "#version 120\nuniform sampler2D colortex6; uniform sampler2D depthtex2; uniform sampler2D shadowcolor1;\n",
        );
        assert_eq!(usage.colorBuffers, 7);
        assert_eq!(usage.depthBuffers, 3);
        assert_eq!(usage.shadowDepthBuffers, 2);
        assert_eq!(usage.shadowColorBuffers, 2);
    }

    #[test]
    fn fullscreen_adapter_preserves_extension_directive_order() {
        let source = "#version 120\n#extension GL_ARB_shader_texture_lod : enable\nvoid main(){ gl_Position = ftransform(); }\n";
        let adapted = adaptFullscreenVertexShader(source);
        let extension = adapted.find("#extension").unwrap();
        let declaration = adapted.find("attribute vec2 mc_position").unwrap();
        let body = adapted.find("void main").unwrap();
        assert!(extension < declaration && declaration < body);
        assert!(adapted.contains("mc112_ftransform"));
        assert!(adapted.contains("vec4(mc_position, 0.0, 1.0)"));
    }

    #[test]
    fn fullscreen_adapter_replaces_only_complete_glsl_identifiers() {
        let source = "#version 120\nuniform mat4 gl_ProjectionMatrixInverse;\nvoid main(){ int gl_VertexID = 0; gl_Position = gl_ProjectionMatrix * gl_Vertex; }\n";
        let adapted = adaptFullscreenVertexShader(source);
        assert!(adapted.contains("gl_ProjectionMatrixInverse"));
        assert!(adapted.contains("gl_VertexID"));
        assert!(adapted.contains("mc112_projection * mc112_vertex"));
    }

    #[test]
    fn promoted_generic_texture_calls_do_not_collide_with_legacy_sampler_name() {
        let vertex = "#version 120\nvoid main(){ gl_Position=ftransform(); }\n".to_owned();
        let fragment = "#version 120\nuniform sampler2D texture;\nvoid main(){ float whole; modf(1.5, whole); gl_FragColor=texture(texture, vec2(0.0)); }\n".to_owned();
        let (_, fragment) = adaptProgramPair(vertex, fragment);
        assert!(fragment.starts_with("#version 130\n"));
        assert!(fragment.contains("uniform sampler2D mc112_texture_sampler;"));
        assert!(fragment.contains("texture(mc112_texture_sampler, vec2(0.0))"));
    }

    #[test]
    fn inverse_promotes_the_pair_to_glsl_140() {
        let vertex = "#version 120\nvoid main(){ gl_Position=ftransform(); }\n".to_owned();
        let fragment = "#version 120\nvoid main(){ mat4 m=inverse(mat4(1.0)); gl_FragColor=vec4(m[0][0]); }\n".to_owned();
        let (vertex, fragment) = adaptProgramPair(vertex, fragment);
        assert!(vertex.starts_with("#version 140\n"));
        assert!(fragment.starts_with("#version 140\n"));
    }

    #[test]
    fn program_pair_adapter_promotes_lod_and_rewrites_flat_interfaces_together() {
        let vertex = "#version 120\nflat varying vec2 uv;\nvoid main(){ uv=vec2(0.0); gl_Position=ftransform(); }\n".to_owned();
        let fragment = "#version 120\nflat varying vec2 uv; uniform sampler2D tex;\nvoid main(){ gl_FragColor=textureLod(tex, uv, 0.0); }\n".to_owned();
        let (vertex, fragment) = adaptProgramPair(vertex, fragment);
        assert!(vertex.starts_with("#version 130\n"));
        assert!(fragment.starts_with("#version 130\n"));
        assert!(vertex.contains("flat out vec2 uv"));
        assert!(fragment.contains("flat in vec2 uv"));
        assert!(!vertex.contains("flat varying"));
        assert!(!fragment.contains("flat varying"));
    }

    #[test]
    fn program_pair_adapter_injects_gpu_shader4_for_legacy_integer_operations() {
        let vertex = "#version 120\nvoid main(){ uint x=1u; x=x<<1; gl_Position=ftransform(); }\n".to_owned();
        let fragment = "#version 120\nuniform sampler2D tex;\nvoid main(){ gl_FragColor=texelFetch2D(tex, ivec2(0), 0); }\n".to_owned();
        let (vertex, fragment) = adaptProgramPair(vertex, fragment);
        assert!(vertex.contains("#extension GL_EXT_gpu_shader4 : enable"));
        assert!(fragment.contains("#extension GL_EXT_gpu_shader4 : enable"));
    }

    #[test]
    fn program_candidates_try_extensions_before_changing_version_branches() {
        let vertex = "#version 120\nflat varying vec2 uv; void main(){ uint x=1u; uv=vec2(float(x)); gl_Position=ftransform(); }\n".to_owned();
        let fragment = "#version 120\nflat varying vec2 uv; void main(){ gl_FragColor=vec4(uv,0.0,1.0); }\n".to_owned();
        let candidates = adaptProgramCandidates(vertex, fragment);
        assert_eq!(candidates[0].0, "extensions");
        assert!(candidates[0].1.starts_with("#version 120\n"));
        assert!(candidates[0].1.contains("#extension GL_EXT_gpu_shader4 : enable"));
        assert!(candidates[0].1.contains("flat varying vec2 uv"));
    }

    #[test]
    fn program_pair_adapter_promotes_boolean_mix_with_the_program_pair() {
        let vertex = "#version 120\nvoid main(){ bvec3 pick=bvec3(true); vec3 v=mix(vec3(0.0), vec3(1.0), pick); gl_Position=ftransform()+vec4(v,0.0); }\n".to_owned();
        let fragment = "#version 120\nvoid main(){ gl_FragColor=vec4(1.0); }\n".to_owned();
        let (vertex, fragment) = adaptProgramPair(vertex, fragment);
        assert!(vertex.starts_with("#version 130\n"));
        assert!(fragment.starts_with("#version 130\n"));
        assert!(vertex.contains("#extension GL_EXT_gpu_shader4 : enable"));
    }

    #[test]
    fn bloop_style_legacy_features_receive_one_coherent_pair_adaptation() {
        let vertex = "#version 120\nflat varying vec3 color;\nvoid main(){ uint bits=1u<<2; float whole; modf(1.5, whole); bvec3 pick=bvec3(true); color=mix(vec3(0.0), vec3(1.0), pick)+vec3(float(bits)); gl_Position=ftransform(); }\n".to_owned();
        let fragment = "#version 120\nflat varying vec3 color; uniform sampler2D tex;\nvoid main(){ gl_FragColor=texelFetch2D(tex, ivec2(0), 0)+vec4(color,1.0); }\n".to_owned();
        let candidates = adaptProgramCandidates(vertex, fragment);
        assert_eq!(candidates[0].0, "extensions");
        assert_eq!(candidates[1].0, "core-compatibility");
        assert!(candidates[1].1.starts_with("#version 130\n"));
        assert!(candidates[1].2.starts_with("#version 130\n"));
        assert!(candidates[1].1.contains("flat out vec3 color"));
        assert!(candidates[1].2.contains("flat in vec3 color"));
        assert!(candidates[1].1.contains("#extension GL_EXT_gpu_shader4 : enable"));
        assert!(candidates[1].2.contains("#extension GL_EXT_gpu_shader4 : enable"));
    }

    #[test]
    fn version_parser_matches_optifine_integer_shape() {
        assert_eq!(parseVersionNumber("4.6.0 NVIDIA 576.88"), Some(460));
        assert_eq!(parseVersionNumber("3.30 NVIDIA"), Some(330));
    }

    #[test]
    fn gbuffer_backup_chain_matches_optifine_program_numbers() {
        // water (12) -> terrain (7) -> textured_lit (3) -> textured (2)
        // -> basic (1) -> fixed-function fallback.
        let water = GbufferProgram::Water as usize;
        assert_eq!(resolveGbufferProgramIndex(water, |index| index == GbufferProgram::Terrain as usize), Some(GbufferProgram::Terrain as usize));
        assert_eq!(resolveGbufferProgramIndex(water, |index| index == GbufferProgram::Textured as usize), Some(GbufferProgram::Textured as usize));
        assert_eq!(resolveGbufferProgramIndex(water, |_| false), None);
        assert_eq!(resolveGbufferProgramIndex(GBUFFER_PROGRAM_COUNT, |_| true), None);
    }

    #[test]
    fn gbuffer_adapter_preserves_pack_attributes_and_replaces_fixed_inputs() {
        let source = "#version 120\nattribute vec4 mc_Entity;\nvoid main(){ gl_Position=ftransform(); vec4 uv=gl_TextureMatrix[1]*gl_MultiTexCoord1; vec3 n=gl_NormalMatrix*gl_Normal; vec4 p=gl_ModelViewProjectionMatrixInverse*gl_Vertex; vec4 fog=gl_Fog.color; }\n";
        let adapted = adaptGbufferVertexShader(source);
        assert!(adapted.contains("attribute vec4 mc_Entity"));
        assert!(adapted.contains("mc112_ftransform()"));
        assert!(adapted.contains("mc112_texture_matrix[1]*mc112_texcoord1"));
        assert!(adapted.contains("mc112_normal_matrix*mc112_normal"));
        assert!(adapted.contains("mc112_modelview_projection_inverse*mc112_vertex"));
        assert!(adapted.contains("mc112_fog.color"));
        assert!(adapted.contains("uniform mat4 mc112_texture_matrix[8]"));
        assert!(!adapted.contains("gl_MultiTexCoord1"));
    }

    #[test]
    fn gbuffer_fragment_adapter_replaces_fixed_uniform_state() {
        let source = "#version 120\nvoid main(){ vec4 fog=gl_Fog.color; vec4 p=gl_ProjectionMatrixInverse*vec4(1.0); gl_FragData[0]=fog+p; }\n";
        let adapted = adaptGbufferFragmentShader(source);
        assert!(adapted.contains("mc112_fog.color"));
        assert!(adapted.contains("mc112_projection_inverse*vec4(1.0)"));
        assert!(adapted.contains("uniform Mc112FogParameters mc112_fog"));
        assert!(!adapted.contains("gl_ProjectionMatrixInverse"));
    }

    #[test]
    fn color_format_parser_matches_optifine_aliases_and_legacy_comments() {
        let formats = parseColorFormats(
            "const int colortex0Format = RGBA16F;\nconst int gaux2Format=RGB32F;\n/* GAUX4FORMAT:RGB16 */\n",
        );
        assert_eq!(formats[0], textureFormatFromString("RGBA16F"));
        assert_eq!(formats[5], textureFormatFromString("RGB32F"));
        assert_eq!(formats[7], textureFormatFromString("RGB16"));
        assert_eq!(formats[1], None);
    }

    #[test]
    fn color_format_parser_ignores_commented_declarations_and_detects_gdepth_uniform() {
        let source = "// const int colortex2Format = RGBA32F;\n/* const int colortex3Format = RGB16F; */\nuniform sampler2D gdepth;\n";
        let formats = parseColorFormats(source);
        assert!(formats.iter().all(Option::is_none));
        assert!(hasUniform(source, "gdepth"));
        assert!(!hasUniform(source, "depthtex1"));
    }

    #[test]
    fn texture_format_table_matches_optifine_1122_ids() {
        assert_eq!(textureFormatFromString("RGBA8"), Some(32856));
        assert_eq!(textureFormatFromString("RGBA16F"), Some(34842));
        assert_eq!(textureFormatFromString("RGBA32F"), Some(34836));
        assert_eq!(textureFormatFromString("RGB9_E5"), Some(35901));
        assert_eq!(textureFormatFromString("rgba16f"), None);
    }

    #[test]
    fn parses_optifine_persistent_clear_flags() {
        let source = "const bool colortex4Clear = false;\nconst bool gaux2Clear = false;\nconst bool colortex1Clear = true;\n";
        assert_eq!(parseClearDisabledMask(source), (1 << 4) | (1 << 5));
    }

    #[test]
    fn parses_optifine_composite_mipmap_requests() {
        let source = "const bool colortex0MipmapEnabled = true;\nconst bool gaux2MipmapEnabled = true;\nconst bool colortex7MipmapEnabled = false;\n";
        assert_eq!(parseCompositeMipmapMask(source), (1 << 0) | (1 << 5));
    }

    #[test]
    fn parses_optifine_const_bools_without_matching_comments_or_similar_names() {
        let source = "const bool shadowHardwareFiltering = TRUE;\nconst bool shadowHardwareFiltering0 = false;\n// const bool shadowHardwareFiltering1 = true;\n";
        assert_eq!(parseConstBool(source, "shadowHardwareFiltering"), Some(true));
        assert_eq!(parseConstBool(source, "shadowHardwareFiltering0"), Some(false));
        assert_eq!(parseConstBool(source, "shadowHardwareFiltering1"), None);
        assert_eq!(parseConstBool(source, "shadowHardware"), None);
        assert_eq!(parseConstBool("const bool noSemicolon = true", "noSemicolon"), None);
    }

    #[test]
    fn parses_and_generates_optifine_hf_noise_deterministically() {
        assert_eq!(parseConstInt("const int noiseTextureResolution = 64;", "noiseTextureResolution"), Some(64));
        assert_eq!(parseConstInt("// const int noiseTextureResolution = 128;", "noiseTextureResolution"), None);
        let image = generateHfNoiseImage(2, 2).unwrap();
        assert_eq!(image.len(), 12);
        assert_eq!(image, generateHfNoiseImage(2, 2).unwrap());
        assert_ne!(&image[0..3], &image[3..6]);
    }

}
