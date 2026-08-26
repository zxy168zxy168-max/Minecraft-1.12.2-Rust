use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Context;
use rayon::prelude::*;

use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::CpuFrame::CpuFrame;
use crate::vulkan::GuiCompiler::{CompiledGuiFrame, CompiledGuiStep, GuiBatch, VulkanGuiVertex};
use crate::vulkan::GuiDrawList::GuiDrawList;
use crate::vulkan::GuiRenderFrame::GuiRenderFrame;
use crate::vulkan::NativeImage::NativeImage;
use crate::vulkan::PanoramaRenderer::PanoramaPassPlan;
use crate::vulkan::TextureSource::TextureSource;

const PANORAMA_SIZE: u32 = 256;
const PANORAMA_PIXELS: usize = (PANORAMA_SIZE as usize) * (PANORAMA_SIZE as usize);

#[derive(Debug, Clone)]
struct PanoramaCache {
    textures: [ResourceLocation; 6],
    timer: f32,
    first_blur_passes: i32,
    second_blur_passes: i32,
    final_blur_pairs: i32,
    image: Arc<CpuFrame>,
}

/// Explicit software/offline fallback for the MCP GUI command stream.
///
/// Normal OpenGL and Vulkan presentation consumes `prepareNativeFrame` and
/// rasterizes the source-ordered Tessellator-equivalent batches on the GPU.
/// This implementation remains isolated for deterministic tests and emergency
/// fallback only; the normal Vulkan main menu no longer creates or uploads a
/// full-window CPU RGBA image.
pub struct SoftwareGuiRenderer {
    resourceManager: ResourceManager,
    textures: HashMap<ResourceLocation, Arc<TextureSource>>,
    panoramaRays: Vec<[f32; 3]>,
    panoramaCache: Option<PanoramaCache>,
    profileStarted: Instant,
    profileFrames: u64,
    profileCompileNanos: u128,
    profilePanoramaNanos: u128,
    profileCompositeNanos: u128,
    profileBatchNanos: u128,
}

impl SoftwareGuiRenderer {
    pub fn new(resourceManager: ResourceManager) -> Self {
        Self {
            resourceManager,
            textures: HashMap::new(),
            panoramaRays: build_panorama_rays(),
            panoramaCache: None,
            profileStarted: Instant::now(),
            profileFrames: 0,
            profileCompileNanos: 0,
            profilePanoramaNanos: 0,
            profileCompositeNanos: 0,
            profileBatchNanos: 0,
        }
    }

    pub fn setResourceManager(&mut self, resourceManager: ResourceManager) {
        self.resourceManager = resourceManager;
        self.clear_texture_cache();
    }

    pub fn clear_texture_cache(&mut self) {
        self.textures.clear();
        self.panoramaCache = None;
    }

    /// Equivalent to `TextureManager#loadTexture` for runtime-only GUI
    /// textures such as resource-pack `pack.png` icons. These images do not
    /// belong to the block TextureMap and therefore must not invalidate chunk
    /// meshes or reopen their source ZIP during drawing.
    pub fn registerDynamicTexture(&mut self, location: ResourceLocation, image: NativeImage) {
        let source = TextureSource {
            requested_location: location.clone(),
            source_pack: "dynamic".to_owned(),
            image,
            sampling: Default::default(),
            animation: None,
            missing: false,
        };
        self.textures.insert(location, Arc::new(source));
    }

    /// Resolves the exact GuiDrawList into a native-GPU frame without CPU
    /// rasterization. OpenGL and Vulkan both consume this source-ordered
    /// BufferBuilder/Tessellator-equivalent frame; the software renderer is
    /// retained only for deterministic tests and explicit emergency fallback.
    pub fn prepareNativeFrame(
        &mut self,
        drawList: &GuiDrawList,
        guiWidth: i32,
        guiHeight: i32,
        outputWidth: u32,
        outputHeight: u32,
    ) -> anyhow::Result<GuiRenderFrame> {
        anyhow::ensure!(
            guiWidth > 0 && guiHeight > 0,
            "GUI dimensions must be positive"
        );
        anyhow::ensure!(
            outputWidth > 0 && outputHeight > 0,
            "output dimensions must be positive"
        );

        let compiled = CompiledGuiFrame::compile(drawList);
        let mut textures = HashMap::new();
        for step in &compiled.steps {
            match step {
                CompiledGuiStep::Draw(batch) => {
                    if let Some(location) = batch.texture.as_ref() {
                        textures
                            .entry(location.clone())
                            .or_insert_with(|| self.texture(location));
                    }
                }
                CompiledGuiStep::Panorama(plan) => {
                    if let Some(sample) = plan.samples.first() {
                        for face in &sample.faces {
                            let location = &face.texture;
                            textures
                                .entry(location.clone())
                                .or_insert_with(|| self.texture(location));
                        }
                    }
                }
            }
        }
        Ok(GuiRenderFrame {
            compiled,
            textures,
            guiWidth,
            guiHeight,
            outputWidth,
            outputHeight,
        })
    }

    pub fn render(
        &mut self,
        drawList: &GuiDrawList,
        guiWidth: i32,
        guiHeight: i32,
        outputWidth: u32,
        outputHeight: u32,
    ) -> anyhow::Result<CpuFrame> {
        anyhow::ensure!(
            guiWidth > 0 && guiHeight > 0,
            "GUI dimensions must be positive"
        );
        anyhow::ensure!(
            outputWidth > 0 && outputHeight > 0,
            "output dimensions must be positive"
        );

        let mut output = CpuFrame::new(outputWidth, outputHeight);
        output.clear([0, 0, 0, 255]);

        let compileStarted = Instant::now();
        let compiled = CompiledGuiFrame::compile(drawList);
        let compileElapsed = compileStarted.elapsed();
        let mut panoramaElapsed = Duration::ZERO;
        let mut compositeElapsed = Duration::ZERO;
        let mut batchElapsed = Duration::ZERO;

        for step in &compiled.steps {
            match step {
                CompiledGuiStep::Panorama(plan) => {
                    let panoramaStarted = Instant::now();
                    let panorama = self.render_panorama(plan)?;
                    panoramaElapsed = panoramaElapsed.saturating_add(panoramaStarted.elapsed());

                    let compositeStarted = Instant::now();
                    composite_panorama(&mut output, panorama.as_ref(), guiWidth, guiHeight);
                    compositeElapsed = compositeElapsed.saturating_add(compositeStarted.elapsed());
                }
                CompiledGuiStep::Draw(batch) => {
                    let batchStarted = Instant::now();
                    let texture = match &batch.texture {
                        Some(location) => Some(self.texture(location)),
                        None => None,
                    };
                    rasterize_batch(&mut output, batch, texture.as_deref(), guiWidth, guiHeight);
                    batchElapsed = batchElapsed.saturating_add(batchStarted.elapsed());
                }
            }
        }
        self.recordProfile(
            compileElapsed,
            panoramaElapsed,
            compositeElapsed,
            batchElapsed,
        );
        Ok(output)
    }

    fn recordProfile(
        &mut self,
        compile: Duration,
        panorama: Duration,
        composite: Duration,
        batches: Duration,
    ) {
        self.profileFrames = self.profileFrames.saturating_add(1);
        self.profileCompileNanos = self.profileCompileNanos.saturating_add(compile.as_nanos());
        self.profilePanoramaNanos = self
            .profilePanoramaNanos
            .saturating_add(panorama.as_nanos());
        self.profileCompositeNanos = self
            .profileCompositeNanos
            .saturating_add(composite.as_nanos());
        self.profileBatchNanos = self.profileBatchNanos.saturating_add(batches.as_nanos());

        let elapsed = self.profileStarted.elapsed();
        if elapsed < Duration::from_secs(5) {
            return;
        }
        let frames = self.profileFrames.max(1) as f64;
        log::info!(
            "Software GUI stages: {:.1} fps, compile={:.3} ms, panorama={:.3} ms, composite={:.3} ms, batches={:.3} ms",
            self.profileFrames as f64 / elapsed.as_secs_f64().max(0.001),
            self.profileCompileNanos as f64 / frames / 1_000_000.0,
            self.profilePanoramaNanos as f64 / frames / 1_000_000.0,
            self.profileCompositeNanos as f64 / frames / 1_000_000.0,
            self.profileBatchNanos as f64 / frames / 1_000_000.0,
        );
        self.profileStarted = Instant::now();
        self.profileFrames = 0;
        self.profileCompileNanos = 0;
        self.profilePanoramaNanos = 0;
        self.profileCompositeNanos = 0;
        self.profileBatchNanos = 0;
    }

    fn texture(&mut self, location: &ResourceLocation) -> Arc<TextureSource> {
        if let Some(texture) = self.textures.get(location) {
            return Arc::clone(texture);
        }
        let source = match TextureSource::load(&self.resourceManager, location) {
            Ok(source) => source,
            Err(error) => {
                log::warn!("failed loading texture {location}; using missingno: {error}");
                TextureSource::missing(location.clone())
            }
        };
        let source = Arc::new(source);
        self.textures.insert(location.clone(), Arc::clone(&source));
        source
    }

    fn render_panorama(&mut self, plan: &PanoramaPassPlan) -> anyhow::Result<Arc<CpuFrame>> {
        let firstSample = plan
            .samples
            .first()
            .context("panorama plan has no samples")?;
        let textures = std::array::from_fn(|index| firstSample.faces[index].texture.clone());
        let first_blur_passes = plan.samples.len() as i32;
        let second_blur_passes = plan
            .blur_invocations
            .first()
            .map_or(0, |invocation| invocation.layers.len() as i32);
        let final_blur_pairs = ((plan.blur_invocations.len().saturating_sub(1)) / 2) as i32;
        let timer = infer_panorama_timer(plan);

        let reusable = self.panoramaCache.as_ref().is_some_and(|cache| {
            cache.textures == textures
                && cache.first_blur_passes == first_blur_passes
                && cache.second_blur_passes == second_blur_passes
                && cache.final_blur_pairs == final_blur_pairs
                // Reuse only within the exact same logical GUI frame. The old
                // one-tick tolerance quantized an uncapped panorama to 20 FPS.
                && cache.timer.to_bits() == timer.to_bits()
        });
        if reusable {
            let cache = self.panoramaCache.as_ref().expect("checked panorama cache");
            return Ok(Arc::clone(&cache.image));
        }

        let faces: [Arc<TextureSource>; 6] =
            std::array::from_fn(|index| self.texture(&textures[index]));
        let image = Arc::new(
            render_panorama_image(plan, &faces, &self.panoramaRays)
                .context("failed rasterizing Minecraft 1.12.2 panorama")?,
        );
        self.panoramaCache = Some(PanoramaCache {
            textures,
            timer,
            first_blur_passes,
            second_blur_passes,
            final_blur_pairs,
            image: Arc::clone(&image),
        });
        Ok(image)
    }
}

fn infer_panorama_timer(plan: &PanoramaPassPlan) -> f32 {
    // All samples share the timer-derived pitch/yaw. The exact timer cannot be
    // reconstructed from pitch alone, but yaw is `-timer * 0.1` in MCP.
    plan.samples
        .first()
        .map_or(0.0, |sample| -sample.yaw_degrees * 10.0)
}

fn build_panorama_rays() -> Vec<[f32; 3]> {
    let tangent = (60.0_f32).to_radians().tan();
    let mut rays = Vec::with_capacity(PANORAMA_PIXELS);
    for y in 0..PANORAMA_SIZE {
        let normalizedY = 1.0 - ((y as f32 + 0.5) / PANORAMA_SIZE as f32) * 2.0;
        for x in 0..PANORAMA_SIZE {
            let normalizedX = ((x as f32 + 0.5) / PANORAMA_SIZE as f32) * 2.0 - 1.0;
            let view = [normalizedX * tangent, normalizedY * tangent, -1.0];
            // Inverse of GuiMainMenu's fixed Rx(180) then Rz(90).
            rays.push([-view[1], -view[0], -view[2]]);
        }
    }
    rays
}

fn render_panorama_image(
    plan: &PanoramaPassPlan,
    faces: &[Arc<TextureSource>; 6],
    rays: &[[f32; 3]],
) -> anyhow::Result<CpuFrame> {
    anyhow::ensure!(rays.len() == PANORAMA_PIXELS, "invalid panorama ray table");
    anyhow::ensure!(!plan.samples.is_empty(), "panorama plan has no samples");

    #[derive(Clone, Copy)]
    struct PreparedSample {
        translated: [f32; 3],
        alpha: f32,
        inverse_alpha: f32,
    }
    let firstSample = plan
        .samples
        .first()
        .expect("panorama samples checked above");
    let pitch = firstSample.pitch_degrees.to_radians();
    let yaw = firstSample.yaw_degrees.to_radians();
    let (sin_pitch, cos_pitch) = pitch.sin_cos();
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    let prepared = plan
        .samples
        .iter()
        .map(|sample| {
            debug_assert_eq!(
                sample.pitch_degrees.to_bits(),
                firstSample.pitch_degrees.to_bits()
            );
            debug_assert_eq!(
                sample.yaw_degrees.to_bits(),
                firstSample.yaw_degrees.to_bits()
            );
            let alpha = sample.faces[0].alpha_u8 as f32 / 255.0;
            PreparedSample {
                translated: rotate_dynamic_inverse(
                    [-sample.translate_x, -sample.translate_y, 0.0],
                    sin_pitch,
                    cos_pitch,
                    sin_yaw,
                    cos_yaw,
                ),
                alpha,
                inverse_alpha: 1.0 - alpha,
            }
        })
        .collect::<Vec<_>>();

    // All 64 jittered samples share the same animated camera rotation. The
    // previous software fallback recalculated that rotation for every
    // pixel/sample pair (over four million matrix operations per menu frame).
    // Precompute one rotated ray per 256x256 target pixel; only the sample
    // origin changes inside the accumulation loop, exactly as in GuiMainMenu.
    let rotatedRays = rays
        .par_iter()
        .map(|&ray| {
            PreparedPanoramaRay::new(rotate_dynamic_inverse(
                ray, sin_pitch, cos_pitch, sin_yaw, cos_yaw,
            ))
        })
        .collect::<Vec<_>>();

    let mut accumulation = vec![0.0_f32; PANORAMA_PIXELS * 3];
    let workerCount = panorama_worker_count(PANORAMA_PIXELS);
    let pixelsPerWorker = PANORAMA_PIXELS.div_ceil(workerCount);
    accumulation
        .par_chunks_mut(pixelsPerWorker * 3)
        .enumerate()
        .for_each(|(workerIndex, chunk)| {
            let firstPixel = workerIndex * pixelsPerWorker;
            for (localPixel, rgb) in chunk.chunks_exact_mut(3).enumerate() {
                let pixelIndex = firstPixel + localPixel;
                let ray = rotatedRays[pixelIndex];
                for sample in &prepared {
                    let (face, u, v) = intersect_cube_prepared(sample.translated, ray);
                    let texel = sample_texture(&faces[face].image, u, v, true, true);
                    rgb[0] = texel[0] * sample.alpha + rgb[0] * sample.inverse_alpha;
                    rgb[1] = texel[1] * sample.alpha + rgb[1] * sample.inverse_alpha;
                    rgb[2] = texel[2] * sample.alpha + rgb[2] * sample.inverse_alpha;
                }
            }
        });

    let mut panorama = CpuFrame::new(PANORAMA_SIZE, PANORAMA_SIZE);
    for pixelIndex in 0..PANORAMA_PIXELS {
        let source = pixelIndex * 3;
        let target = pixelIndex * 4;
        panorama.rgba_mut()[target] = to_u8(accumulation[source]);
        panorama.rgba_mut()[target + 1] = to_u8(accumulation[source + 1]);
        panorama.rgba_mut()[target + 2] = to_u8(accumulation[source + 2]);
        panorama.rgba_mut()[target + 3] = 255;
    }

    let mut blurScratch = CpuFrame::new(PANORAMA_SIZE, PANORAMA_SIZE);
    for invocation in &plan.blur_invocations {
        rotate_and_blur_into(&panorama, &mut blurScratch, invocation.layers.as_slice());
        std::mem::swap(&mut panorama, &mut blurScratch);
    }
    Ok(panorama)
}

fn rotate_dynamic_inverse(
    value: [f32; 3],
    sinPitch: f32,
    cosPitch: f32,
    sinYaw: f32,
    cosYaw: f32,
) -> [f32; 3] {
    // Inverse of Rx(pitch) followed by Ry(yaw), preserving OpenGL's
    // post-multiplication order used by GuiMainMenu.drawPanorama.
    let x1 = value[0];
    let y1 = cosPitch * value[1] + sinPitch * value[2];
    let z1 = -sinPitch * value[1] + cosPitch * value[2];
    [cosYaw * x1 - sinYaw * z1, y1, sinYaw * x1 + cosYaw * z1]
}

#[derive(Clone, Copy)]
struct PreparedPanoramaRay {
    direction: [f32; 3],
    inverse_direction: [f32; 3],
    boundary: [f32; 3],
}

impl PreparedPanoramaRay {
    fn new(direction: [f32; 3]) -> Self {
        Self {
            direction,
            inverse_direction: direction.map(|component| {
                if component.abs() < 1.0e-8 {
                    f32::INFINITY
                } else {
                    component.recip()
                }
            }),
            boundary: direction.map(|component| if component >= 0.0 { 1.0 } else { -1.0 }),
        }
    }
}

fn intersect_cube_prepared(origin: [f32; 3], ray: PreparedPanoramaRay) -> (usize, f32, f32) {
    // Every jitter origin remains inside the unit cube. The exiting face is
    // therefore the smallest positive slab distance. Cache the ray reciprocal
    // and sign boundary once per target pixel: the former fallback performed
    // three floating-point divisions for every one of the 64 samples, i.e.
    // more than twelve million divisions per menu frame.
    let distances = [
        (ray.boundary[0] - origin[0]) * ray.inverse_direction[0],
        (ray.boundary[1] - origin[1]) * ray.inverse_direction[1],
        (ray.boundary[2] - origin[2]) * ray.inverse_direction[2],
    ];
    let mut axis = 2_usize;
    let mut distance = distances[2];
    if distances[0] > 0.0 && distances[0] < distance {
        axis = 0;
        distance = distances[0];
    }
    if distances[1] > 0.0 && distances[1] < distance {
        axis = 1;
        distance = distances[1];
    }
    let point = [
        origin[0] + ray.direction[0] * distance,
        origin[1] + ray.direction[1] * distance,
        origin[2] + ray.direction[2] * distance,
    ];
    let (face, local_x, local_y) = match axis {
        0 if point[0] >= 0.0 => (1, -point[2], point[1]),
        0 => (3, point[2], point[1]),
        1 if point[1] >= 0.0 => (5, point[0], -point[2]),
        1 => (4, point[0], point[2]),
        2 if point[2] >= 0.0 => (0, point[0], point[1]),
        _ => (2, -point[0], point[1]),
    };
    (face, (local_x + 1.0) * 0.5, (local_y + 1.0) * 0.5)
}

fn panorama_worker_count(work_item_count: usize) -> usize {
    rayon::current_num_threads()
        .clamp(1, 8)
        .min(work_item_count.max(1))
}

fn rotate_and_blur_into(
    source: &CpuFrame,
    destination: &mut CpuFrame,
    layers: &[crate::vulkan::PanoramaRenderer::BlurLayer],
) {
    debug_assert_eq!(source.width(), destination.width());
    debug_assert_eq!(source.height(), destination.height());
    destination.rgba_mut().copy_from_slice(source.rgba());
    if layers.is_empty() {
        return;
    }

    // GuiMainMenu.rotateAndBlurSkybox samples every layer from the same
    // pre-invocation texture while blending the layers into the destination in
    // order. Reuse one ping-pong target for all seven vanilla invocations
    // instead of cloning and reallocating a 256x256 image for every pass.
    let width = destination.width() as usize;
    let height = destination.height() as usize;
    let rowBytes = width * 4;
    let workerCount = panorama_worker_count(height);
    let rowsPerWorker = height.div_ceil(workerCount);

    destination
        .rgba_mut()
        .par_chunks_mut(rowsPerWorker * rowBytes)
        .enumerate()
        .for_each(|(chunkIndex, rows)| {
            let firstRow = chunkIndex * rowsPerWorker;
            for (localRow, row) in rows.chunks_exact_mut(rowBytes).enumerate() {
                let y = firstRow + localRow;
                let v = (y as f32 + 0.5) / PANORAMA_SIZE as f32;
                for x in 0..width {
                    let offset = x * 4;
                    for layer in layers {
                        let alpha = layer.alpha.clamp(0.0, 1.0);
                        let inverse = 1.0 - alpha;
                        let u =
                            (x as f32 + 0.5) / PANORAMA_SIZE as f32 + layer.horizontal_uv_offset;
                        let sample = sample_cpu_frame(source, u, v, true, true);
                        row[offset] =
                            to_u8(sample[0] * alpha + row[offset] as f32 / 255.0 * inverse);
                        row[offset + 1] =
                            to_u8(sample[1] * alpha + row[offset + 1] as f32 / 255.0 * inverse);
                        row[offset + 2] =
                            to_u8(sample[2] * alpha + row[offset + 2] as f32 / 255.0 * inverse);
                        row[offset + 3] = 255;
                    }
                }
            }
        });
}

fn composite_panorama(output: &mut CpuFrame, panorama: &CpuFrame, guiWidth: i32, guiHeight: i32) {
    let maxDimension = guiWidth.max(guiHeight) as f32;
    let scale = 120.0 / maxDimension;
    let verticalExtent = guiHeight as f32 * scale / 256.0;
    let horizontalExtent = guiWidth as f32 * scale / 256.0;
    let width = output.width() as usize;
    let height = output.height() as usize;
    let rowBytes = width * 4;
    let workerCount = panorama_worker_count(height);
    let rowsPerWorker = height.div_ceil(workerCount);

    // The full-window stretch was the remaining serial hot path after the
    // 256x256 cube pass had already been parallelized. This keeps the exact
    // GuiMainMenu.drawPanorama UV mapping but prevents a high-resolution menu
    // from presenting a visually quantized panorama on multi-core systems.
    output
        .rgba_mut()
        .par_chunks_mut(rowsPerWorker * rowBytes)
        .enumerate()
        .for_each(|(chunkIndex, rows)| {
            let firstRow = chunkIndex * rowsPerWorker;
            for (localRow, row) in rows.chunks_exact_mut(rowBytes).enumerate() {
                let y = firstRow + localRow;
                let guiY = (y as f32 + 0.5) * guiHeight as f32 / height as f32;
                let normalizedY = guiY / guiHeight as f32;
                let u = 0.5 + verticalExtent - normalizedY * verticalExtent * 2.0;
                for x in 0..width {
                    let guiX = (x as f32 + 0.5) * guiWidth as f32 / width as f32;
                    let normalizedX = guiX / guiWidth as f32;
                    let v = 0.5 + horizontalExtent - normalizedX * horizontalExtent * 2.0;
                    let sample = sample_cpu_frame(panorama, u, v, true, true);
                    let offset = x * 4;
                    row[offset] = to_u8(sample[0]);
                    row[offset + 1] = to_u8(sample[1]);
                    row[offset + 2] = to_u8(sample[2]);
                    row[offset + 3] = 255;
                }
            }
        });
}

fn rasterize_batch(
    output: &mut CpuFrame,
    batch: &GuiBatch,
    texture: Option<&TextureSource>,
    guiWidth: i32,
    guiHeight: i32,
) {
    for triangle in batch.indices.chunks_exact(3) {
        let vertices = [
            scale_vertex(
                batch.vertices[triangle[0] as usize],
                output,
                guiWidth,
                guiHeight,
            ),
            scale_vertex(
                batch.vertices[triangle[1] as usize],
                output,
                guiWidth,
                guiHeight,
            ),
            scale_vertex(
                batch.vertices[triangle[2] as usize],
                output,
                guiWidth,
                guiHeight,
            ),
        ];
        rasterize_triangle(output, vertices, texture);
    }
}

fn scale_vertex(
    mut vertex: VulkanGuiVertex,
    output: &CpuFrame,
    guiWidth: i32,
    guiHeight: i32,
) -> VulkanGuiVertex {
    vertex.position[0] *= output.width() as f32 / guiWidth as f32;
    vertex.position[1] *= output.height() as f32 / guiHeight as f32;
    vertex
}

fn rasterize_triangle(
    output: &mut CpuFrame,
    vertices: [VulkanGuiVertex; 3],
    texture: Option<&TextureSource>,
) {
    let p0 = [vertices[0].position[0], vertices[0].position[1]];
    let p1 = [vertices[1].position[0], vertices[1].position[1]];
    let p2 = [vertices[2].position[0], vertices[2].position[1]];
    let area = edge(p0, p1, p2);
    if area.abs() < 1.0e-8 {
        return;
    }

    let minX = p0[0].min(p1[0]).min(p2[0]).floor().max(0.0) as i32;
    let maxX = p0[0]
        .max(p1[0])
        .max(p2[0])
        .ceil()
        .min(output.width() as f32) as i32;
    let minY = p0[1].min(p1[1]).min(p2[1]).floor().max(0.0) as i32;
    let maxY = p0[1]
        .max(p1[1])
        .max(p2[1])
        .ceil()
        .min(output.height() as f32) as i32;

    for y in minY..maxY {
        for x in minX..maxX {
            let point = [x as f32 + 0.5, y as f32 + 0.5];
            let w0 = edge(p1, p2, point) / area;
            let w1 = edge(p2, p0, point) / area;
            let w2 = edge(p0, p1, point) / area;
            if w0 < -1.0e-5 || w1 < -1.0e-5 || w2 < -1.0e-5 {
                continue;
            }

            let uv = [
                vertices[0].uv[0] * w0 + vertices[1].uv[0] * w1 + vertices[2].uv[0] * w2,
                vertices[0].uv[1] * w0 + vertices[1].uv[1] * w1 + vertices[2].uv[1] * w2,
            ];
            let color = [
                vertices[0].color_rgba[0] * w0
                    + vertices[1].color_rgba[0] * w1
                    + vertices[2].color_rgba[0] * w2,
                vertices[0].color_rgba[1] * w0
                    + vertices[1].color_rgba[1] * w1
                    + vertices[2].color_rgba[1] * w2,
                vertices[0].color_rgba[2] * w0
                    + vertices[1].color_rgba[2] * w1
                    + vertices[2].color_rgba[2] * w2,
                vertices[0].color_rgba[3] * w0
                    + vertices[1].color_rgba[3] * w1
                    + vertices[2].color_rgba[3] * w2,
            ];
            let source = if let Some(texture) = texture {
                let texel = sample_texture(
                    &texture.image,
                    uv[0],
                    uv[1],
                    texture.sampling.blur,
                    texture.sampling.clamp,
                );
                [
                    texel[0] * color[0],
                    texel[1] * color[1],
                    texel[2] * color[2],
                    texel[3] * color[3],
                ]
            } else {
                color
            };
            // Textured Gui quads use Minecraft's ordinary GL_GREATER 0.1
            // alpha test. Gui.drawGradientRect explicitly disables alpha
            // testing, so its low-alpha tail must still blend into the menu.
            if texture.is_some() && source[3] <= 0.1 {
                continue;
            }
            output.blend_pixel(x, y, source);
        }
    }
}

fn edge(a: [f32; 2], b: [f32; 2], point: [f32; 2]) -> f32 {
    (point[0] - a[0]) * (b[1] - a[1]) - (point[1] - a[1]) * (b[0] - a[0])
}

fn sample_texture(image: &NativeImage, u: f32, v: f32, linear: bool, clamp: bool) -> [f32; 4] {
    sample_rgba(
        image.width(),
        image.height(),
        image.rgba(),
        u,
        v,
        linear,
        clamp,
    )
}

fn sample_cpu_frame(image: &CpuFrame, u: f32, v: f32, linear: bool, clamp: bool) -> [f32; 4] {
    sample_rgba(
        image.width(),
        image.height(),
        image.rgba(),
        u,
        v,
        linear,
        clamp,
    )
}

fn sample_rgba(
    width: u32,
    height: u32,
    rgba: &[u8],
    u: f32,
    v: f32,
    linear: bool,
    clamp: bool,
) -> [f32; 4] {
    if width == 0 || height == 0 {
        return [0.0, 0.0, 0.0, 0.0];
    }
    let normalizedU = normalize_coordinate(u, clamp);
    let normalizedV = normalize_coordinate(v, clamp);
    if !linear {
        let x = (normalizedU * width as f32)
            .floor()
            .clamp(0.0, (width - 1) as f32) as u32;
        let y = (normalizedV * height as f32)
            .floor()
            .clamp(0.0, (height - 1) as f32) as u32;
        return rgba_pixel(width, rgba, x, y);
    }

    let x = normalizedU * width as f32 - 0.5;
    let y = normalizedV * height as f32 - 0.5;
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let fractionX = x - x.floor();
    let fractionY = y - y.floor();
    let p00 = rgba_pixel_wrapped(width, height, rgba, x0, y0, clamp);
    let p10 = rgba_pixel_wrapped(width, height, rgba, x0 + 1, y0, clamp);
    let p01 = rgba_pixel_wrapped(width, height, rgba, x0, y0 + 1, clamp);
    let p11 = rgba_pixel_wrapped(width, height, rgba, x0 + 1, y0 + 1, clamp);
    let mut result = [0.0; 4];
    for channel in 0..4 {
        let top = p00[channel] + (p10[channel] - p00[channel]) * fractionX;
        let bottom = p01[channel] + (p11[channel] - p01[channel]) * fractionX;
        result[channel] = top + (bottom - top) * fractionY;
    }
    result
}

fn normalize_coordinate(value: f32, clamp: bool) -> f32 {
    if clamp {
        value.clamp(0.0, 1.0 - f32::EPSILON)
    } else {
        value.rem_euclid(1.0)
    }
}

fn rgba_pixel(width: u32, rgba: &[u8], x: u32, y: u32) -> [f32; 4] {
    let offset = ((y * width + x) * 4) as usize;
    [
        rgba[offset] as f32 / 255.0,
        rgba[offset + 1] as f32 / 255.0,
        rgba[offset + 2] as f32 / 255.0,
        rgba[offset + 3] as f32 / 255.0,
    ]
}

fn rgba_pixel_wrapped(
    width: u32,
    height: u32,
    rgba: &[u8],
    x: i32,
    y: i32,
    clamp: bool,
) -> [f32; 4] {
    let (x, y) = if clamp {
        (
            x.clamp(0, width as i32 - 1) as u32,
            y.clamp(0, height as i32 - 1) as u32,
        )
    } else {
        (
            x.rem_euclid(width as i32) as u32,
            y.rem_euclid(height as i32) as u32,
        )
    };
    rgba_pixel(width, rgba, x, y)
}

fn to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0 + 0.5).floor() as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_ray_hits_a_cube_face_with_centered_uv() {
        let (face, u, v) =
            intersect_cube_prepared([0.0, 0.0, 0.0], PreparedPanoramaRay::new([0.0, 0.0, 1.0]));
        assert_eq!(face, 0);
        assert!((u - 0.5).abs() < f32::EPSILON);
        assert!((v - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn positive_x_face_orientation_matches_gui_main_menu_rotation() {
        let (face, u, v) =
            intersect_cube_prepared([0.0, 0.0, 0.0], PreparedPanoramaRay::new([1.0, 0.25, 0.5]));
        assert_eq!(face, 1);
        assert!((u - 0.25).abs() < 0.0001);
        assert!((v - 0.625).abs() < 0.0001);
    }

    #[test]
    fn nearest_sampler_uses_top_left_png_coordinates() {
        let image = NativeImage::from_rgba(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
        )
        .unwrap();
        assert_eq!(
            sample_texture(&image, 0.1, 0.1, false, true),
            [1.0, 0.0, 0.0, 1.0]
        );
        assert_eq!(
            sample_texture(&image, 0.9, 0.9, false, true),
            [1.0, 1.0, 1.0, 1.0]
        );
    }
}
