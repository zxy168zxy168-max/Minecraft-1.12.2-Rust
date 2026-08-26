use std::collections::HashMap;
use std::ffi::CString;
use std::io::Cursor;
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context};
use ash::{vk, Device};

use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::vulkan::GuiCompiler::{CompiledGuiStep, GuiBatch, VulkanGuiVertex};
use crate::vulkan::GuiDrawList::GuiTopology;
use crate::vulkan::GuiRenderFrame::GuiRenderFrame;
use crate::vulkan::PanoramaRenderer::{PanoramaCompositeVertex, PanoramaPassPlan};
use crate::vulkan::TextureSource::TextureSource;

const MAX_GUI_TEXTURES: u32 = 1024;
const PANORAMA_SIZE: u32 = 256;

#[repr(C)]
#[derive(Clone, Copy)]
struct GuiPushConstants {
    guiSize: [f32; 2],
    useTexture: i32,
    padding: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct PanoramaPushConstants {
    pitchRadians: f32,
    yawRadians: f32,
    sampleCount: i32,
    padding: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct BlurPushConstants {
    values: [i32; 4],
}

struct MappedBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    mapped: NonNull<u8>,
    capacity: vk::DeviceSize,
}

struct FrameGeometry {
    vertex: Option<MappedBuffer>,
    index: Option<MappedBuffer>,
}

struct GuiTexture {
    source: Arc<TextureSource>,
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    sampler: vk::Sampler,
    descriptorSet: vk::DescriptorSet,
}

struct PendingGuiTextureUpload {
    staging: HostBuffer,
    image: vk::Image,
    width: u32,
    height: u32,
}

struct OffscreenImage {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    sampler: vk::Sampler,
    framebuffer: vk::Framebuffer,
    guiDescriptorSet: vk::DescriptorSet,
    blurDescriptorSet: vk::DescriptorSet,
}

#[derive(Clone, Copy)]
struct PreparedDraw {
    firstIndex: u32,
    indexCount: u32,
    vertexOffset: i32,
    descriptorSet: vk::DescriptorSet,
    useTexture: bool,
}

/// Native Vulkan implementation of the MCP GUI command stream.
///
/// `GuiScreen`, `Gui`, `FontRenderer`, `GuiMainMenu` and `GuiDrawList` retain
/// ownership of layout, ordering and animation constants. This object only
/// replaces the former full-window CPU rasterization/upload with equivalent
/// Vulkan vertex/index submission, persistent texture objects and GPU
/// panorama/blur passes.
pub struct VulkanGuiPipeline {
    guiDescriptorSetLayout: vk::DescriptorSetLayout,
    panoramaDescriptorSetLayout: vk::DescriptorSetLayout,
    blurDescriptorSetLayout: vk::DescriptorSetLayout,
    descriptorPool: vk::DescriptorPool,
    guiPipelineLayout: vk::PipelineLayout,
    panoramaPipelineLayout: vk::PipelineLayout,
    blurPipelineLayout: vk::PipelineLayout,
    guiRenderPass: vk::RenderPass,
    offscreenRenderPass: vk::RenderPass,
    guiPipeline: vk::Pipeline,
    panoramaPipeline: vk::Pipeline,
    blurPipeline: vk::Pipeline,
    swapchainImageViews: Vec<vk::ImageView>,
    swapchainFramebuffers: Vec<vk::Framebuffer>,
    frameGeometry: Vec<FrameGeometry>,
    offscreen: Vec<[OffscreenImage; 2]>,
    panoramaDescriptorSets: Vec<vk::DescriptorSet>,
    textures: HashMap<ResourceLocation, GuiTexture>,
    whiteTexture: Option<GuiTexture>,
    /// GUI textures are created while compiling the current MCP draw list,
    /// but their transfer commands are recorded into the same frame command
    /// buffer before panorama/world-space GUI sampling. This removes the old
    /// per-texture command-buffer allocation and `queue_wait_idle` stall.
    pendingTextureUploads: Vec<PendingGuiTextureUpload>,
    /// Replaced textures and completed upload staging buffers are released only
    /// when this frame slot's fence signals on its next reuse. Because the
    /// graphics queue is ordered, that fence also follows every older frame
    /// which could still reference the replaced descriptor.
    retiredTextures: Vec<Vec<GuiTexture>>,
    retiredTextureStaging: Vec<Vec<HostBuffer>>,
    profileStarted: Instant,
    profileFrames: u64,
    profileDraws: u64,
    profileTextureUploads: u64,
    profileTextureUploadBytes: u64,
    profileSubmitNanos: u128,
}

impl VulkanGuiPipeline {
    pub fn new(
        device: &Device,
        memoryProperties: &vk::PhysicalDeviceMemoryProperties,
        swapchainImages: &[vk::Image],
        swapchainFormat: vk::Format,
        swapchainExtent: vk::Extent2D,
        framesInFlight: usize,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            framesInFlight > 0,
            "Vulkan GUI requires at least one frame slot"
        );

        let guiBinding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let guiBindings = [guiBinding];
        let guiLayoutInfo = vk::DescriptorSetLayoutCreateInfo::default().bindings(&guiBindings);
        let guiDescriptorSetLayout =
            unsafe { device.create_descriptor_set_layout(&guiLayoutInfo, None) }
                .context("failed creating Vulkan GUI descriptor-set layout")?;

        let panoramaBindings = (0..6)
            .map(|binding| {
                vk::DescriptorSetLayoutBinding::default()
                    .binding(binding)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            })
            .collect::<Vec<_>>();
        let panoramaLayoutInfo =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&panoramaBindings);
        let panoramaDescriptorSetLayout =
            match unsafe { device.create_descriptor_set_layout(&panoramaLayoutInfo, None) } {
                Ok(layout) => layout,
                Err(error) => {
                    unsafe { device.destroy_descriptor_set_layout(guiDescriptorSetLayout, None) };
                    return Err(anyhow!(
                        "failed creating panorama descriptor-set layout: {error:?}"
                    ));
                }
            };

        let blurBinding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);
        let blurBindings = [blurBinding];
        let blurLayoutInfo = vk::DescriptorSetLayoutCreateInfo::default().bindings(&blurBindings);
        let blurDescriptorSetLayout =
            match unsafe { device.create_descriptor_set_layout(&blurLayoutInfo, None) } {
                Ok(layout) => layout,
                Err(error) => {
                    unsafe {
                        device.destroy_descriptor_set_layout(panoramaDescriptorSetLayout, None);
                        device.destroy_descriptor_set_layout(guiDescriptorSetLayout, None);
                    }
                    return Err(anyhow!(
                        "failed creating blur descriptor-set layout: {error:?}"
                    ));
                }
            };

        // A texture replacement allocates the new descriptor immediately and
        // retires the old descriptor behind the current frame-slot fence. Keep
        // one full texture-cache generation per in-flight frame so resource
        // reloads never need `device_wait_idle` merely to recycle descriptors.
        let textureDescriptorCapacity =
            MAX_GUI_TEXTURES.saturating_mul((framesInFlight as u32).saturating_add(1));
        let descriptorCount = textureDescriptorCapacity
            .saturating_add(1)
            .saturating_add((framesInFlight as u32).saturating_mul(10));
        let poolSize = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(descriptorCount);
        let poolSizes = [poolSize];
        let descriptorPoolInfo = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .pool_sizes(&poolSizes)
            .max_sets(
                textureDescriptorCapacity
                    .saturating_add(1)
                    .saturating_add((framesInFlight as u32).saturating_mul(5)),
            );
        let descriptorPool =
            match unsafe { device.create_descriptor_pool(&descriptorPoolInfo, None) } {
                Ok(pool) => pool,
                Err(error) => {
                    unsafe {
                        device.destroy_descriptor_set_layout(blurDescriptorSetLayout, None);
                        device.destroy_descriptor_set_layout(panoramaDescriptorSetLayout, None);
                        device.destroy_descriptor_set_layout(guiDescriptorSetLayout, None);
                    }
                    return Err(anyhow!(
                        "failed creating Vulkan GUI descriptor pool: {error:?}"
                    ));
                }
            };

        let guiPipelineLayout = create_pipeline_layout::<GuiPushConstants>(
            device,
            guiDescriptorSetLayout,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
            "GUI",
        )?;
        let panoramaPipelineLayout = create_pipeline_layout::<PanoramaPushConstants>(
            device,
            panoramaDescriptorSetLayout,
            vk::ShaderStageFlags::FRAGMENT,
            "panorama",
        )?;
        let blurPipelineLayout = create_pipeline_layout::<BlurPushConstants>(
            device,
            blurDescriptorSetLayout,
            vk::ShaderStageFlags::FRAGMENT,
            "panorama blur",
        )?;

        let offscreenRenderPass = create_offscreen_render_pass(device)?;
        let panoramaPipeline = create_fullscreen_pipeline(
            device,
            offscreenRenderPass,
            panoramaPipelineLayout,
            include_bytes!(concat!(env!("OUT_DIR"), "/panorama.vert.spv")),
            include_bytes!(concat!(env!("OUT_DIR"), "/panorama.frag.spv")),
            "panorama",
        )?;
        let blurPipeline = create_fullscreen_pipeline(
            device,
            offscreenRenderPass,
            blurPipelineLayout,
            include_bytes!(concat!(env!("OUT_DIR"), "/panorama.vert.spv")),
            include_bytes!(concat!(env!("OUT_DIR"), "/panorama_blur.frag.spv")),
            "panorama blur",
        )?;

        let panoramaDescriptorSets = allocate_descriptor_sets(
            device,
            descriptorPool,
            panoramaDescriptorSetLayout,
            framesInFlight,
            "panorama",
        )?;
        let guiOffscreenSets = allocate_descriptor_sets(
            device,
            descriptorPool,
            guiDescriptorSetLayout,
            framesInFlight * 2,
            "panorama composite",
        )?;
        let blurOffscreenSets = allocate_descriptor_sets(
            device,
            descriptorPool,
            blurDescriptorSetLayout,
            framesInFlight * 2,
            "panorama blur source",
        )?;

        let mut offscreen = Vec::with_capacity(framesInFlight);
        for frameSlot in 0..framesInFlight {
            let first = create_offscreen_image(
                device,
                memoryProperties,
                offscreenRenderPass,
                guiOffscreenSets[frameSlot * 2],
                blurOffscreenSets[frameSlot * 2],
            )?;
            let second = match create_offscreen_image(
                device,
                memoryProperties,
                offscreenRenderPass,
                guiOffscreenSets[frameSlot * 2 + 1],
                blurOffscreenSets[frameSlot * 2 + 1],
            ) {
                Ok(image) => image,
                Err(error) => {
                    destroy_offscreen_image(device, first);
                    for pair in offscreen.drain(..) {
                        let [firstImage, secondImage] = pair;
                        destroy_offscreen_image(device, firstImage);
                        destroy_offscreen_image(device, secondImage);
                    }
                    return Err(error);
                }
            };
            offscreen.push([first, second]);
        }

        let mut result = Self {
            guiDescriptorSetLayout,
            panoramaDescriptorSetLayout,
            blurDescriptorSetLayout,
            descriptorPool,
            guiPipelineLayout,
            panoramaPipelineLayout,
            blurPipelineLayout,
            guiRenderPass: vk::RenderPass::null(),
            offscreenRenderPass,
            guiPipeline: vk::Pipeline::null(),
            panoramaPipeline,
            blurPipeline,
            swapchainImageViews: Vec::new(),
            swapchainFramebuffers: Vec::new(),
            frameGeometry: (0..framesInFlight)
                .map(|_| FrameGeometry {
                    vertex: None,
                    index: None,
                })
                .collect(),
            offscreen,
            panoramaDescriptorSets,
            textures: HashMap::new(),
            whiteTexture: None,
            pendingTextureUploads: Vec::new(),
            retiredTextures: (0..framesInFlight).map(|_| Vec::new()).collect(),
            retiredTextureStaging: (0..framesInFlight).map(|_| Vec::new()).collect(),
            profileStarted: Instant::now(),
            profileFrames: 0,
            profileDraws: 0,
            profileTextureUploads: 0,
            profileTextureUploadBytes: 0,
            profileSubmitNanos: 0,
        };
        if let Err(error) = result.create_swapchain_resources(
            device,
            swapchainImages,
            swapchainFormat,
            swapchainExtent,
        ) {
            result.destroy(device);
            return Err(error);
        }
        Ok(result)
    }

    pub fn recreate_swapchain_resources(
        &mut self,
        device: &Device,
        swapchainImages: &[vk::Image],
        swapchainFormat: vk::Format,
        swapchainExtent: vk::Extent2D,
    ) -> anyhow::Result<()> {
        self.destroy_swapchain_resources(device);
        self.create_swapchain_resources(device, swapchainImages, swapchainFormat, swapchainExtent)
    }

    fn create_swapchain_resources(
        &mut self,
        device: &Device,
        swapchainImages: &[vk::Image],
        swapchainFormat: vk::Format,
        swapchainExtent: vk::Extent2D,
    ) -> anyhow::Result<()> {
        self.guiRenderPass = create_gui_render_pass(device, swapchainFormat)?;
        self.guiPipeline = create_gui_pipeline(device, self.guiRenderPass, self.guiPipelineLayout)?;
        for &image in swapchainImages {
            let view = create_image_view(device, image, swapchainFormat)?;
            let attachments = [view];
            let framebufferInfo = vk::FramebufferCreateInfo::default()
                .render_pass(self.guiRenderPass)
                .attachments(&attachments)
                .width(swapchainExtent.width)
                .height(swapchainExtent.height)
                .layers(1);
            let framebuffer = match unsafe { device.create_framebuffer(&framebufferInfo, None) } {
                Ok(framebuffer) => framebuffer,
                Err(error) => {
                    unsafe { device.destroy_image_view(view, None) };
                    return Err(anyhow!("failed creating Vulkan GUI framebuffer: {error:?}"));
                }
            };
            self.swapchainImageViews.push(view);
            self.swapchainFramebuffers.push(framebuffer);
        }
        Ok(())
    }

    pub fn destroy_swapchain_resources(&mut self, device: &Device) {
        unsafe {
            for framebuffer in self.swapchainFramebuffers.drain(..) {
                device.destroy_framebuffer(framebuffer, None);
            }
            for view in self.swapchainImageViews.drain(..) {
                device.destroy_image_view(view, None);
            }
            if self.guiPipeline != vk::Pipeline::null() {
                device.destroy_pipeline(self.guiPipeline, None);
                self.guiPipeline = vk::Pipeline::null();
            }
            if self.guiRenderPass != vk::RenderPass::null() {
                device.destroy_render_pass(self.guiRenderPass, None);
                self.guiRenderPass = vk::RenderPass::null();
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        device: &Device,
        memoryProperties: &vk::PhysicalDeviceMemoryProperties,
        _commandPool: vk::CommandPool,
        _graphicsQueue: vk::Queue,
        commandBuffer: vk::CommandBuffer,
        frameSlot: usize,
        imageIndex: usize,
        swapchainExtent: vk::Extent2D,
        frame: &GuiRenderFrame,
    ) -> anyhow::Result<()> {
        let submitStarted = Instant::now();
        anyhow::ensure!(
            frameSlot < self.frameGeometry.len(),
            "Vulkan GUI frame slot out of range"
        );
        anyhow::ensure!(
            frameSlot < self.offscreen.len(),
            "Vulkan GUI panorama frame slot out of range"
        );
        anyhow::ensure!(
            frameSlot < self.panoramaDescriptorSets.len(),
            "Vulkan GUI panorama descriptor slot out of range",
        );
        anyhow::ensure!(
            imageIndex < self.swapchainFramebuffers.len(),
            "Vulkan GUI swapchain image out of range"
        );
        anyhow::ensure!(
            frame.outputWidth == swapchainExtent.width
                && frame.outputHeight == swapchainExtent.height,
            "native GUI frame {}x{} does not match Vulkan swapchain {}x{}",
            frame.outputWidth,
            frame.outputHeight,
            swapchainExtent.width,
            swapchainExtent.height,
        );

        self.collect_retired_textures(device, frameSlot)?;
        self.ensure_white_texture(device, memoryProperties)?;
        for (location, source) in &frame.textures {
            self.ensure_texture(device, memoryProperties, frameSlot, location, source)?;
        }

        let panoramaPlans = frame
            .compiled
            .steps
            .iter()
            .filter_map(|step| match step {
                CompiledGuiStep::Panorama(plan) => Some(plan),
                CompiledGuiStep::Draw(_) => None,
            })
            .collect::<Vec<_>>();
        anyhow::ensure!(
            panoramaPlans.len() <= 1,
            "MCP GUI frame contains more than one panorama command"
        );
        let panoramaFinal = if let Some(plan) = panoramaPlans.first().copied() {
            self.update_panorama_descriptor(device, frameSlot, plan)?;
            Some(plan.blur_invocations.len() & 1)
        } else {
            None
        };

        let (vertices, indices, draws) = self.prepare_geometry(frameSlot, frame, panoramaFinal)?;
        self.upload_geometry(device, memoryProperties, frameSlot, &vertices, &indices)?;

        unsafe {
            device
                .reset_command_buffer(commandBuffer, vk::CommandBufferResetFlags::empty())
                .context("failed resetting Vulkan native-GUI command buffer")?;
            let beginInfo = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            device
                .begin_command_buffer(commandBuffer, &beginInfo)
                .context("failed beginning Vulkan native-GUI command buffer")?;
        }

        self.record_pending_texture_uploads(device, commandBuffer, frameSlot)?;

        if let Some(plan) = panoramaPlans.first().copied() {
            self.record_panorama(device, commandBuffer, frameSlot, plan)?;
        }

        let clearValues = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];
        let renderPassInfo = vk::RenderPassBeginInfo::default()
            .render_pass(self.guiRenderPass)
            .framebuffer(self.swapchainFramebuffers[imageIndex])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchainExtent,
            })
            .clear_values(&clearValues);
        unsafe {
            device.cmd_begin_render_pass(
                commandBuffer,
                &renderPassInfo,
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(
                commandBuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.guiPipeline,
            );
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: swapchainExtent.width as f32,
                height: swapchainExtent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchainExtent,
            };
            device.cmd_set_viewport(commandBuffer, 0, &[viewport]);
            device.cmd_set_scissor(commandBuffer, 0, &[scissor]);
            if let Some(geometry) = self.frameGeometry.get(frameSlot) {
                let vertex = geometry
                    .vertex
                    .as_ref()
                    .context("missing Vulkan GUI vertex buffer")?;
                let index = geometry
                    .index
                    .as_ref()
                    .context("missing Vulkan GUI index buffer")?;
                device.cmd_bind_vertex_buffers(commandBuffer, 0, &[vertex.buffer], &[0]);
                device.cmd_bind_index_buffer(commandBuffer, index.buffer, 0, vk::IndexType::UINT32);
            }
            for draw in &draws {
                device.cmd_bind_descriptor_sets(
                    commandBuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.guiPipelineLayout,
                    0,
                    &[draw.descriptorSet],
                    &[],
                );
                let push = GuiPushConstants {
                    guiSize: [frame.guiWidth.max(1) as f32, frame.guiHeight.max(1) as f32],
                    useTexture: if draw.useTexture { 1 } else { 0 },
                    padding: 0,
                };
                device.cmd_push_constants(
                    commandBuffer,
                    self.guiPipelineLayout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytes_of(&push),
                );
                device.cmd_draw_indexed(
                    commandBuffer,
                    draw.indexCount,
                    1,
                    draw.firstIndex,
                    draw.vertexOffset,
                    0,
                );
            }
            device.cmd_end_render_pass(commandBuffer);
            device
                .end_command_buffer(commandBuffer)
                .context("failed ending Vulkan native-GUI command buffer")?;
        }

        self.profileFrames = self.profileFrames.saturating_add(1);
        self.profileDraws = self.profileDraws.saturating_add(draws.len() as u64);
        self.profileSubmitNanos = self
            .profileSubmitNanos
            .saturating_add(submitStarted.elapsed().as_nanos());
        let elapsed = self.profileStarted.elapsed();
        if elapsed >= Duration::from_secs(5) {
            log::info!(
                "Vulkan native GUI workload: {:.1} fps, prepare/record={:.3} ms, batches/frame={:.1}, cached_textures={}, texture_uploads={}, texture_upload={:.2} MiB",
                self.profileFrames as f64 / elapsed.as_secs_f64().max(0.001),
                self.profileSubmitNanos as f64 / self.profileFrames.max(1) as f64 / 1_000_000.0,
                self.profileDraws as f64 / self.profileFrames.max(1) as f64,
                self.textures.len(),
                self.profileTextureUploads,
                self.profileTextureUploadBytes as f64 / (1024.0 * 1024.0),
            );
            self.profileStarted = Instant::now();
            self.profileFrames = 0;
            self.profileDraws = 0;
            self.profileTextureUploads = 0;
            self.profileTextureUploadBytes = 0;
            self.profileSubmitNanos = 0;
        }
        Ok(())
    }

    fn prepare_geometry(
        &self,
        frameSlot: usize,
        frame: &GuiRenderFrame,
        panoramaFinal: Option<usize>,
    ) -> anyhow::Result<(Vec<VulkanGuiVertex>, Vec<u32>, Vec<PreparedDraw>)> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut draws = Vec::new();
        for step in &frame.compiled.steps {
            match step {
                CompiledGuiStep::Draw(batch) => {
                    let (descriptorSet, useTexture) = if let Some(location) = batch.texture.as_ref()
                    {
                        let texture = self.textures.get(location).with_context(|| {
                            format!("missing cached Vulkan GUI texture {location}")
                        })?;
                        (texture.descriptorSet, true)
                    } else {
                        (
                            self.whiteTexture
                                .as_ref()
                                .context("missing Vulkan GUI white texture")?
                                .descriptorSet,
                            false,
                        )
                    };
                    append_prepared_batch(
                        batch,
                        descriptorSet,
                        useTexture,
                        &mut vertices,
                        &mut indices,
                        &mut draws,
                    )?;
                }
                CompiledGuiStep::Panorama(plan) => {
                    let finalIndex = panoramaFinal.context("missing panorama final target")?;
                    let texture = &self.offscreen[frameSlot][finalIndex];
                    let batch = panorama_composite_batch(plan);
                    append_prepared_batch(
                        &batch,
                        texture.guiDescriptorSet,
                        true,
                        &mut vertices,
                        &mut indices,
                        &mut draws,
                    )?;
                }
            }
        }
        Ok((vertices, indices, draws))
    }

    fn upload_geometry(
        &mut self,
        device: &Device,
        memoryProperties: &vk::PhysicalDeviceMemoryProperties,
        frameSlot: usize,
        vertices: &[VulkanGuiVertex],
        indices: &[u32],
    ) -> anyhow::Result<()> {
        let geometry = self
            .frameGeometry
            .get_mut(frameSlot)
            .context("Vulkan GUI frame slot out of range")?;
        let vertexBytes = as_bytes(vertices);
        let indexBytes = as_bytes(indices);
        ensure_mapped_buffer(
            device,
            memoryProperties,
            &mut geometry.vertex,
            vertexBytes.len() as vk::DeviceSize,
            vk::BufferUsageFlags::VERTEX_BUFFER,
            "GUI vertex",
        )?;
        ensure_mapped_buffer(
            device,
            memoryProperties,
            &mut geometry.index,
            indexBytes.len() as vk::DeviceSize,
            vk::BufferUsageFlags::INDEX_BUFFER,
            "GUI index",
        )?;
        unsafe {
            std::ptr::copy_nonoverlapping(
                vertexBytes.as_ptr(),
                geometry
                    .vertex
                    .as_ref()
                    .expect("created vertex buffer")
                    .mapped
                    .as_ptr(),
                vertexBytes.len(),
            );
            std::ptr::copy_nonoverlapping(
                indexBytes.as_ptr(),
                geometry
                    .index
                    .as_ref()
                    .expect("created index buffer")
                    .mapped
                    .as_ptr(),
                indexBytes.len(),
            );
        }
        Ok(())
    }

    fn record_panorama(
        &self,
        device: &Device,
        commandBuffer: vk::CommandBuffer,
        frameSlot: usize,
        plan: &PanoramaPassPlan,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(!plan.samples.is_empty(), "panorama plan has no samples");
        anyhow::ensure!(
            plan.target_width == PANORAMA_SIZE && plan.target_height == PANORAMA_SIZE,
            "MCP panorama target must remain {PANORAMA_SIZE}x{PANORAMA_SIZE}",
        );
        let extent = vk::Extent2D {
            width: PANORAMA_SIZE,
            height: PANORAMA_SIZE,
        };
        let clearValues = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        }];
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: PANORAMA_SIZE as f32,
            height: PANORAMA_SIZE as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        let begin = vk::RenderPassBeginInfo::default()
            .render_pass(self.offscreenRenderPass)
            .framebuffer(self.offscreen[frameSlot][0].framebuffer)
            .render_area(scissor)
            .clear_values(&clearValues);
        unsafe {
            device.cmd_begin_render_pass(commandBuffer, &begin, vk::SubpassContents::INLINE);
            device.cmd_bind_pipeline(
                commandBuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.panoramaPipeline,
            );
            device.cmd_set_viewport(commandBuffer, 0, &[viewport]);
            device.cmd_set_scissor(commandBuffer, 0, &[scissor]);
            device.cmd_bind_descriptor_sets(
                commandBuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.panoramaPipelineLayout,
                0,
                &[self.panoramaDescriptorSets[frameSlot]],
                &[],
            );
            let firstSample = plan
                .samples
                .first()
                .context("panorama plan has no sample")?;
            let push = PanoramaPushConstants {
                // Consume the MCP/MathHelper-derived angles from PanoramaPassPlan
                // directly. Recomputing pitch with GLSL sin() is observably not
                // identical to Minecraft 1.12.2's lookup-table MathHelper.sin.
                pitchRadians: firstSample.pitch_degrees.to_radians(),
                yawRadians: firstSample.yaw_degrees.to_radians(),
                sampleCount: plan.samples.len().min(i32::MAX as usize) as i32,
                padding: 0,
            };
            device.cmd_push_constants(
                commandBuffer,
                self.panoramaPipelineLayout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                bytes_of(&push),
            );
            device.cmd_draw(commandBuffer, 3, 1, 0, 0);
            device.cmd_end_render_pass(commandBuffer);
        }

        let mut current = 0usize;
        for invocation in &plan.blur_invocations {
            let target = 1 - current;
            let begin = vk::RenderPassBeginInfo::default()
                .render_pass(self.offscreenRenderPass)
                .framebuffer(self.offscreen[frameSlot][target].framebuffer)
                .render_area(scissor)
                .clear_values(&clearValues);
            unsafe {
                device.cmd_begin_render_pass(commandBuffer, &begin, vk::SubpassContents::INLINE);
                device.cmd_bind_pipeline(
                    commandBuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.blurPipeline,
                );
                device.cmd_set_viewport(commandBuffer, 0, &[viewport]);
                device.cmd_set_scissor(commandBuffer, 0, &[scissor]);
                device.cmd_bind_descriptor_sets(
                    commandBuffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.blurPipelineLayout,
                    0,
                    &[self.offscreen[frameSlot][current].blurDescriptorSet],
                    &[],
                );
                let push = BlurPushConstants {
                    values: [
                        invocation.layers.len().min(i32::MAX as usize) as i32,
                        0,
                        0,
                        0,
                    ],
                };
                device.cmd_push_constants(
                    commandBuffer,
                    self.blurPipelineLayout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    bytes_of(&push),
                );
                device.cmd_draw(commandBuffer, 3, 1, 0, 0);
                device.cmd_end_render_pass(commandBuffer);
            }
            current = target;
        }
        Ok(())
    }

    fn update_panorama_descriptor(
        &self,
        device: &Device,
        frameSlot: usize,
        plan: &PanoramaPassPlan,
    ) -> anyhow::Result<()> {
        let sample = plan
            .samples
            .first()
            .context("panorama plan has no sample")?;
        let mut infos = [vk::DescriptorImageInfo::default(); 6];
        for (index, face) in sample.faces.iter().enumerate() {
            let texture = self
                .textures
                .get(&face.texture)
                .with_context(|| format!("missing Vulkan panorama texture {}", face.texture))?;
            infos[index] = vk::DescriptorImageInfo::default()
                .sampler(texture.sampler)
                .image_view(texture.view)
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        }
        let set = self.panoramaDescriptorSets[frameSlot];
        for index in 0..6 {
            let imageInfo = [infos[index]];
            let write = vk::WriteDescriptorSet::default()
                .dst_set(set)
                .dst_binding(index as u32)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(&imageInfo);
            unsafe { device.update_descriptor_sets(&[write], &[]) };
        }
        Ok(())
    }

    fn collect_retired_textures(
        &mut self,
        device: &Device,
        frameSlot: usize,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            frameSlot < self.retiredTextures.len() && frameSlot < self.retiredTextureStaging.len(),
            "Vulkan GUI retired-resource frame slot out of range",
        );
        let descriptorPool = self.descriptorPool;
        for texture in self.retiredTextures[frameSlot].drain(..) {
            unsafe {
                device
                    .free_descriptor_sets(descriptorPool, &[texture.descriptorSet])
                    .context("failed freeing retired Vulkan GUI texture descriptor")?;
            }
            destroy_gui_texture(device, texture);
        }
        for staging in self.retiredTextureStaging[frameSlot].drain(..) {
            destroy_buffer(device, staging);
        }
        Ok(())
    }

    fn record_pending_texture_uploads(
        &mut self,
        device: &Device,
        commandBuffer: vk::CommandBuffer,
        frameSlot: usize,
    ) -> anyhow::Result<()> {
        anyhow::ensure!(
            frameSlot < self.retiredTextureStaging.len(),
            "Vulkan GUI texture-upload frame slot out of range",
        );
        if self.pendingTextureUploads.is_empty() {
            return Ok(());
        }
        let pending = std::mem::take(&mut self.pendingTextureUploads);
        self.profileTextureUploads = self
            .profileTextureUploads
            .saturating_add(pending.len() as u64);
        self.profileTextureUploadBytes =
            self.profileTextureUploadBytes
                .saturating_add(pending.iter().fold(0_u64, |total, upload| {
                    total.saturating_add(
                        u64::from(upload.width)
                            .saturating_mul(u64::from(upload.height))
                            .saturating_mul(4),
                    )
                }));
        let toTransfer = pending
            .iter()
            .map(|upload| {
                vk::ImageMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::empty())
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .old_layout(vk::ImageLayout::UNDEFINED)
                    .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                    .image(upload.image)
                    .subresource_range(color_subresource_range())
            })
            .collect::<Vec<_>>();
        unsafe {
            device.cmd_pipeline_barrier(
                commandBuffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &toTransfer,
            );
            for upload in &pending {
                let copy = vk::BufferImageCopy::default()
                    .buffer_offset(0)
                    .buffer_row_length(0)
                    .buffer_image_height(0)
                    .image_subresource(
                        vk::ImageSubresourceLayers::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .mip_level(0)
                            .base_array_layer(0)
                            .layer_count(1),
                    )
                    .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
                    .image_extent(vk::Extent3D {
                        width: upload.width,
                        height: upload.height,
                        depth: 1,
                    });
                device.cmd_copy_buffer_to_image(
                    commandBuffer,
                    upload.staging.buffer,
                    upload.image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &[copy],
                );
            }
            let toShader = pending
                .iter()
                .map(|upload| {
                    vk::ImageMemoryBarrier::default()
                        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                        .dst_access_mask(vk::AccessFlags::SHADER_READ)
                        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                        .image(upload.image)
                        .subresource_range(color_subresource_range())
                })
                .collect::<Vec<_>>();
            device.cmd_pipeline_barrier(
                commandBuffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &toShader,
            );
        }
        self.retiredTextureStaging[frameSlot]
            .extend(pending.into_iter().map(|upload| upload.staging));
        Ok(())
    }

    fn ensure_white_texture(
        &mut self,
        device: &Device,
        memoryProperties: &vk::PhysicalDeviceMemoryProperties,
    ) -> anyhow::Result<()> {
        if self.whiteTexture.is_some() {
            return Ok(());
        }
        let location = ResourceLocation::parse("minecraft:internal/solid_white.png");
        let source = Arc::new(TextureSource::solid_white(location));
        let (texture, pending) = create_pending_texture(
            device,
            memoryProperties,
            self.descriptorPool,
            self.guiDescriptorSetLayout,
            source,
        )?;
        self.whiteTexture = Some(texture);
        self.pendingTextureUploads.push(pending);
        Ok(())
    }

    fn ensure_texture(
        &mut self,
        device: &Device,
        memoryProperties: &vk::PhysicalDeviceMemoryProperties,
        frameSlot: usize,
        location: &ResourceLocation,
        source: &Arc<TextureSource>,
    ) -> anyhow::Result<()> {
        if self
            .textures
            .get(location)
            .is_some_and(|texture| Arc::ptr_eq(&texture.source, source))
        {
            return Ok(());
        }
        anyhow::ensure!(
            frameSlot < self.retiredTextures.len(),
            "Vulkan GUI texture-retirement frame slot out of range",
        );
        let replacing = self.textures.contains_key(location);
        anyhow::ensure!(
            replacing || self.textures.len() < MAX_GUI_TEXTURES as usize,
            "Vulkan GUI texture cache exceeded {MAX_GUI_TEXTURES} entries"
        );
        let (texture, pending) = create_pending_texture(
            device,
            memoryProperties,
            self.descriptorPool,
            self.guiDescriptorSetLayout,
            Arc::clone(source),
        )?;
        if let Some(old) = self.textures.insert(location.clone(), texture) {
            self.retiredTextures[frameSlot].push(old);
        }
        self.pendingTextureUploads.push(pending);
        Ok(())
    }

    pub fn destroy(&mut self, device: &Device) {
        self.destroy_swapchain_resources(device);
        for upload in self.pendingTextureUploads.drain(..) {
            destroy_buffer(device, upload.staging);
        }
        for slot in &mut self.retiredTextureStaging {
            for staging in slot.drain(..) {
                destroy_buffer(device, staging);
            }
        }
        unsafe {
            for (_, texture) in self.textures.drain() {
                destroy_gui_texture(device, texture);
            }
            if let Some(texture) = self.whiteTexture.take() {
                destroy_gui_texture(device, texture);
            }
            for slot in &mut self.retiredTextures {
                for texture in slot.drain(..) {
                    destroy_gui_texture(device, texture);
                }
            }
            for geometry in &mut self.frameGeometry {
                destroy_mapped_buffer(device, geometry.vertex.take());
                destroy_mapped_buffer(device, geometry.index.take());
            }
            for pair in self.offscreen.drain(..) {
                let [firstImage, secondImage] = pair;
                destroy_offscreen_image(device, firstImage);
                destroy_offscreen_image(device, secondImage);
            }
            device.destroy_pipeline(self.blurPipeline, None);
            device.destroy_pipeline(self.panoramaPipeline, None);
            device.destroy_render_pass(self.offscreenRenderPass, None);
            device.destroy_pipeline_layout(self.blurPipelineLayout, None);
            device.destroy_pipeline_layout(self.panoramaPipelineLayout, None);
            device.destroy_pipeline_layout(self.guiPipelineLayout, None);
            device.destroy_descriptor_pool(self.descriptorPool, None);
            device.destroy_descriptor_set_layout(self.blurDescriptorSetLayout, None);
            device.destroy_descriptor_set_layout(self.panoramaDescriptorSetLayout, None);
            device.destroy_descriptor_set_layout(self.guiDescriptorSetLayout, None);
        }
    }
}

fn append_prepared_batch(
    batch: &GuiBatch,
    descriptorSet: vk::DescriptorSet,
    useTexture: bool,
    vertices: &mut Vec<VulkanGuiVertex>,
    indices: &mut Vec<u32>,
    draws: &mut Vec<PreparedDraw>,
) -> anyhow::Result<()> {
    if batch.vertices.is_empty() || batch.indices.is_empty() {
        return Ok(());
    }
    let vertexOffset =
        i32::try_from(vertices.len()).context("Vulkan GUI vertex offset overflow")?;
    let firstIndex = u32::try_from(indices.len()).context("Vulkan GUI index offset overflow")?;
    let indexCount =
        u32::try_from(batch.indices.len()).context("Vulkan GUI index count overflow")?;
    vertices.extend_from_slice(&batch.vertices);
    indices.extend_from_slice(&batch.indices);
    draws.push(PreparedDraw {
        firstIndex,
        indexCount,
        vertexOffset,
        descriptorSet,
        useTexture,
    });
    Ok(())
}

fn panorama_composite_batch(plan: &PanoramaPassPlan) -> GuiBatch {
    GuiBatch {
        texture: None,
        topology: GuiTopology::Quads,
        vertices: plan.composite.map(panorama_composite_vertex).to_vec(),
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

fn panorama_composite_vertex(vertex: PanoramaCompositeVertex) -> VulkanGuiVertex {
    VulkanGuiVertex {
        position: [vertex.x, vertex.y, 0.0],
        uv: [vertex.u, vertex.v],
        color_rgba: [1.0; 4],
    }
}

fn create_pipeline_layout<T>(
    device: &Device,
    descriptorSetLayout: vk::DescriptorSetLayout,
    stages: vk::ShaderStageFlags,
    label: &str,
) -> anyhow::Result<vk::PipelineLayout> {
    let setLayouts = [descriptorSetLayout];
    let ranges = [vk::PushConstantRange::default()
        .stage_flags(stages)
        .offset(0)
        .size(std::mem::size_of::<T>() as u32)];
    let info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&setLayouts)
        .push_constant_ranges(&ranges);
    unsafe { device.create_pipeline_layout(&info, None) }
        .with_context(|| format!("failed creating Vulkan {label} pipeline layout"))
}

fn allocate_descriptor_sets(
    device: &Device,
    pool: vk::DescriptorPool,
    layout: vk::DescriptorSetLayout,
    count: usize,
    label: &str,
) -> anyhow::Result<Vec<vk::DescriptorSet>> {
    let layouts = vec![layout; count];
    let info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(pool)
        .set_layouts(&layouts);
    unsafe { device.allocate_descriptor_sets(&info) }
        .with_context(|| format!("failed allocating Vulkan {label} descriptor sets"))
}

fn create_gui_render_pass(device: &Device, format: vk::Format) -> anyhow::Result<vk::RenderPass> {
    let attachment = vk::AttachmentDescription::default()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
    let attachments = [attachment];
    let colorReference = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };
    let colorReferences = [colorReference];
    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&colorReferences);
    let subpasses = [subpass];
    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
    let dependencies = [dependency];
    let info = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);
    unsafe { device.create_render_pass(&info, None) }
        .context("failed creating Vulkan GUI render pass")
}

fn create_offscreen_render_pass(device: &Device) -> anyhow::Result<vk::RenderPass> {
    let attachment = vk::AttachmentDescription::default()
        .format(vk::Format::R8G8B8A8_UNORM)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let attachments = [attachment];
    let colorReference = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };
    let colorReferences = [colorReference];
    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&colorReferences);
    let subpasses = [subpass];
    let dependencies = [
        vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::SHADER_READ)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE),
        vk::SubpassDependency::default()
            .src_subpass(0)
            .dst_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
            .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ),
    ];
    let info = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(&subpasses)
        .dependencies(&dependencies);
    unsafe { device.create_render_pass(&info, None) }
        .context("failed creating Vulkan panorama render pass")
}

fn create_gui_pipeline(
    device: &Device,
    renderPass: vk::RenderPass,
    pipelineLayout: vk::PipelineLayout,
) -> anyhow::Result<vk::Pipeline> {
    let binding = vk::VertexInputBindingDescription {
        binding: 0,
        stride: std::mem::size_of::<VulkanGuiVertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
    };
    let bindings = [binding];
    let attributes = [
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 12,
        },
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R32G32B32A32_SFLOAT,
            offset: 20,
        },
    ];
    create_graphics_pipeline(
        device,
        renderPass,
        pipelineLayout,
        include_bytes!(concat!(env!("OUT_DIR"), "/gui.vert.spv")),
        include_bytes!(concat!(env!("OUT_DIR"), "/gui.frag.spv")),
        &bindings,
        &attributes,
        true,
        "GUI",
    )
}

fn create_fullscreen_pipeline(
    device: &Device,
    renderPass: vk::RenderPass,
    pipelineLayout: vk::PipelineLayout,
    vertexBytes: &[u8],
    fragmentBytes: &[u8],
    label: &str,
) -> anyhow::Result<vk::Pipeline> {
    create_graphics_pipeline(
        device,
        renderPass,
        pipelineLayout,
        vertexBytes,
        fragmentBytes,
        &[],
        &[],
        false,
        label,
    )
}

#[allow(clippy::too_many_arguments)]
fn create_graphics_pipeline(
    device: &Device,
    renderPass: vk::RenderPass,
    pipelineLayout: vk::PipelineLayout,
    vertexBytes: &[u8],
    fragmentBytes: &[u8],
    bindings: &[vk::VertexInputBindingDescription],
    attributes: &[vk::VertexInputAttributeDescription],
    blend: bool,
    label: &str,
) -> anyhow::Result<vk::Pipeline> {
    let vertexCode = ash::util::read_spv(&mut Cursor::new(vertexBytes))
        .with_context(|| format!("failed reading compiled Vulkan {label} vertex shader"))?;
    let fragmentCode = ash::util::read_spv(&mut Cursor::new(fragmentBytes))
        .with_context(|| format!("failed reading compiled Vulkan {label} fragment shader"))?;
    let vertexInfo = vk::ShaderModuleCreateInfo::default().code(&vertexCode);
    let fragmentInfo = vk::ShaderModuleCreateInfo::default().code(&fragmentCode);
    let vertexModule = unsafe { device.create_shader_module(&vertexInfo, None) }
        .with_context(|| format!("failed creating Vulkan {label} vertex module"))?;
    let fragmentModule = match unsafe { device.create_shader_module(&fragmentInfo, None) } {
        Ok(module) => module,
        Err(error) => {
            unsafe { device.destroy_shader_module(vertexModule, None) };
            return Err(anyhow!(
                "failed creating Vulkan {label} fragment module: {error:?}"
            ));
        }
    };
    let entry = CString::new("main").expect("static shader entry");
    let stages = [
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertexModule)
            .name(&entry),
        vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(fragmentModule)
            .name(&entry),
    ];
    let vertexInput = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(bindings)
        .vertex_attribute_descriptions(attributes);
    let inputAssembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false);
    let viewportState = vk::PipelineViewportStateCreateInfo::default()
        .viewport_count(1)
        .scissor_count(1);
    let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);
    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);
    let blendAttachment = vk::PipelineColorBlendAttachmentState::default()
        .blend_enable(blend)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(vk::ColorComponentFlags::RGBA);
    let blendAttachments = [blendAttachment];
    let colorBlend =
        vk::PipelineColorBlendStateCreateInfo::default().attachments(&blendAttachments);
    let dynamicStates = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
    let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamicStates);
    let info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertexInput)
        .input_assembly_state(&inputAssembly)
        .viewport_state(&viewportState)
        .rasterization_state(&rasterization)
        .multisample_state(&multisample)
        .color_blend_state(&colorBlend)
        .dynamic_state(&dynamic)
        .layout(pipelineLayout)
        .render_pass(renderPass)
        .subpass(0);
    let result =
        unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None) };
    unsafe {
        device.destroy_shader_module(fragmentModule, None);
        device.destroy_shader_module(vertexModule, None);
    }
    result
        .map(|pipelines| pipelines[0])
        .map_err(|(_, error)| anyhow!("failed creating Vulkan {label} pipeline: {error:?}"))
}

fn create_offscreen_image(
    device: &Device,
    memoryProperties: &vk::PhysicalDeviceMemoryProperties,
    renderPass: vk::RenderPass,
    guiDescriptorSet: vk::DescriptorSet,
    blurDescriptorSet: vk::DescriptorSet,
) -> anyhow::Result<OffscreenImage> {
    let (image, memory) = create_image(
        device,
        memoryProperties,
        PANORAMA_SIZE,
        PANORAMA_SIZE,
        vk::Format::R8G8B8A8_UNORM,
        vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
    )?;
    let view = match create_image_view(device, image, vk::Format::R8G8B8A8_UNORM) {
        Ok(view) => view,
        Err(error) => {
            unsafe {
                device.destroy_image(image, None);
                device.free_memory(memory, None);
            }
            return Err(error);
        }
    };
    let samplerInfo = vk::SamplerCreateInfo::default()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .min_lod(0.0)
        .max_lod(0.0);
    let sampler = match unsafe { device.create_sampler(&samplerInfo, None) } {
        Ok(sampler) => sampler,
        Err(error) => {
            unsafe {
                device.destroy_image_view(view, None);
                device.destroy_image(image, None);
                device.free_memory(memory, None);
            }
            return Err(anyhow!(
                "failed creating Vulkan panorama sampler: {error:?}"
            ));
        }
    };
    let attachments = [view];
    let framebufferInfo = vk::FramebufferCreateInfo::default()
        .render_pass(renderPass)
        .attachments(&attachments)
        .width(PANORAMA_SIZE)
        .height(PANORAMA_SIZE)
        .layers(1);
    let framebuffer = match unsafe { device.create_framebuffer(&framebufferInfo, None) } {
        Ok(framebuffer) => framebuffer,
        Err(error) => {
            unsafe {
                device.destroy_sampler(sampler, None);
                device.destroy_image_view(view, None);
                device.destroy_image(image, None);
                device.free_memory(memory, None);
            }
            return Err(anyhow!(
                "failed creating Vulkan panorama framebuffer: {error:?}"
            ));
        }
    };
    let imageInfo = [vk::DescriptorImageInfo::default()
        .sampler(sampler)
        .image_view(view)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
    let writes = [
        vk::WriteDescriptorSet::default()
            .dst_set(guiDescriptorSet)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&imageInfo),
        vk::WriteDescriptorSet::default()
            .dst_set(blurDescriptorSet)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&imageInfo),
    ];
    unsafe { device.update_descriptor_sets(&writes, &[]) };
    Ok(OffscreenImage {
        image,
        memory,
        view,
        sampler,
        framebuffer,
        guiDescriptorSet,
        blurDescriptorSet,
    })
}

fn create_pending_texture(
    device: &Device,
    memoryProperties: &vk::PhysicalDeviceMemoryProperties,
    descriptorPool: vk::DescriptorPool,
    descriptorSetLayout: vk::DescriptorSetLayout,
    source: Arc<TextureSource>,
) -> anyhow::Result<(GuiTexture, PendingGuiTextureUpload)> {
    let width = source.image.width();
    let height = source.image.height();
    let bytes = source.image.rgba();
    anyhow::ensure!(
        bytes.len() == width as usize * height as usize * 4,
        "Vulkan GUI texture byte count does not match dimensions"
    );
    let staging = create_host_buffer(
        device,
        memoryProperties,
        bytes,
        vk::BufferUsageFlags::TRANSFER_SRC,
        "GUI texture staging",
    )?;
    let (image, memory) = match create_image(
        device,
        memoryProperties,
        width,
        height,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
    ) {
        Ok(value) => value,
        Err(error) => {
            destroy_buffer(device, staging);
            return Err(error);
        }
    };
    let view = match create_image_view(device, image, vk::Format::R8G8B8A8_SRGB) {
        Ok(view) => view,
        Err(error) => {
            destroy_buffer(device, staging);
            unsafe {
                device.destroy_image(image, None);
                device.free_memory(memory, None);
            }
            return Err(error);
        }
    };
    let filter = if source.sampling.blur {
        vk::Filter::LINEAR
    } else {
        vk::Filter::NEAREST
    };
    let address = if source.sampling.clamp {
        vk::SamplerAddressMode::CLAMP_TO_EDGE
    } else {
        vk::SamplerAddressMode::REPEAT
    };
    let samplerInfo = vk::SamplerCreateInfo::default()
        .mag_filter(filter)
        .min_filter(filter)
        .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
        .address_mode_u(address)
        .address_mode_v(address)
        .address_mode_w(address)
        .min_lod(0.0)
        .max_lod(0.0);
    let sampler = match unsafe { device.create_sampler(&samplerInfo, None) } {
        Ok(sampler) => sampler,
        Err(error) => {
            destroy_buffer(device, staging);
            unsafe {
                device.destroy_image_view(view, None);
                device.destroy_image(image, None);
                device.free_memory(memory, None);
            }
            return Err(anyhow!("failed creating Vulkan GUI sampler: {error:?}"));
        }
    };

    let layouts = [descriptorSetLayout];
    let allocateInfo = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptorPool)
        .set_layouts(&layouts);
    let descriptorSet = match unsafe { device.allocate_descriptor_sets(&allocateInfo) } {
        Ok(sets) => sets[0],
        Err(error) => {
            destroy_buffer(device, staging);
            unsafe {
                device.destroy_sampler(sampler, None);
                device.destroy_image_view(view, None);
                device.destroy_image(image, None);
                device.free_memory(memory, None);
            }
            return Err(anyhow!(
                "failed allocating Vulkan GUI texture descriptor: {error:?}"
            ));
        }
    };

    let imageInfo = [vk::DescriptorImageInfo::default()
        .sampler(sampler)
        .image_view(view)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
    let write = vk::WriteDescriptorSet::default()
        .dst_set(descriptorSet)
        .dst_binding(0)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(&imageInfo);
    unsafe { device.update_descriptor_sets(&[write], &[]) };
    Ok((
        GuiTexture {
            source,
            image,
            memory,
            view,
            sampler,
            descriptorSet,
        },
        PendingGuiTextureUpload {
            staging,
            image,
            width,
            height,
        },
    ))
}

struct HostBuffer {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
}

fn create_host_buffer(
    device: &Device,
    memoryProperties: &vk::PhysicalDeviceMemoryProperties,
    bytes: &[u8],
    usage: vk::BufferUsageFlags,
    label: &str,
) -> anyhow::Result<HostBuffer> {
    let size = bytes.len().max(1) as vk::DeviceSize;
    let info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buffer = unsafe { device.create_buffer(&info, None) }
        .with_context(|| format!("failed creating Vulkan {label} buffer"))?;
    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let memoryType = find_memory_type(
        memoryProperties,
        requirements.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .with_context(|| format!("no host-visible coherent memory for Vulkan {label}"))?;
    let allocationInfo = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(memoryType);
    let memory = match unsafe { device.allocate_memory(&allocationInfo, None) } {
        Ok(memory) => memory,
        Err(error) => {
            unsafe { device.destroy_buffer(buffer, None) };
            return Err(anyhow!(
                "failed allocating Vulkan {label} memory: {error:?}"
            ));
        }
    };
    if let Err(error) = unsafe { device.bind_buffer_memory(buffer, memory, 0) } {
        unsafe {
            device.free_memory(memory, None);
            device.destroy_buffer(buffer, None);
        }
        return Err(anyhow!("failed binding Vulkan {label} memory: {error:?}"));
    }
    let pointer = match unsafe { device.map_memory(memory, 0, size, vk::MemoryMapFlags::empty()) } {
        Ok(pointer) => pointer,
        Err(error) => {
            unsafe {
                device.free_memory(memory, None);
                device.destroy_buffer(buffer, None);
            }
            return Err(anyhow!("failed mapping Vulkan {label} memory: {error:?}"));
        }
    };
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), pointer.cast::<u8>(), bytes.len());
        device.unmap_memory(memory);
    }
    Ok(HostBuffer { buffer, memory })
}

fn ensure_mapped_buffer(
    device: &Device,
    memoryProperties: &vk::PhysicalDeviceMemoryProperties,
    slot: &mut Option<MappedBuffer>,
    required: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    label: &str,
) -> anyhow::Result<()> {
    if slot
        .as_ref()
        .is_some_and(|buffer| buffer.capacity >= required.max(1))
    {
        return Ok(());
    }
    destroy_mapped_buffer(device, slot.take());
    let capacity = required.max(1).next_power_of_two();
    let info = vk::BufferCreateInfo::default()
        .size(capacity)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buffer = unsafe { device.create_buffer(&info, None) }
        .with_context(|| format!("failed creating Vulkan {label} buffer"))?;
    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    let memoryType = find_memory_type(
        memoryProperties,
        requirements.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )
    .with_context(|| format!("no host-visible coherent memory for Vulkan {label}"))?;
    let allocationInfo = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(memoryType);
    let memory = match unsafe { device.allocate_memory(&allocationInfo, None) } {
        Ok(memory) => memory,
        Err(error) => {
            unsafe { device.destroy_buffer(buffer, None) };
            return Err(anyhow!(
                "failed allocating Vulkan {label} memory: {error:?}"
            ));
        }
    };
    if let Err(error) = unsafe { device.bind_buffer_memory(buffer, memory, 0) } {
        unsafe {
            device.free_memory(memory, None);
            device.destroy_buffer(buffer, None);
        }
        return Err(anyhow!("failed binding Vulkan {label} memory: {error:?}"));
    }
    let mapped =
        match unsafe { device.map_memory(memory, 0, capacity, vk::MemoryMapFlags::empty()) } {
            Ok(pointer) => NonNull::new(pointer.cast::<u8>())
                .with_context(|| format!("Vulkan returned null {label} mapping"))?,
            Err(error) => {
                unsafe {
                    device.free_memory(memory, None);
                    device.destroy_buffer(buffer, None);
                }
                return Err(anyhow!("failed mapping Vulkan {label} memory: {error:?}"));
            }
        };
    *slot = Some(MappedBuffer {
        buffer,
        memory,
        mapped,
        capacity,
    });
    Ok(())
}

fn create_image(
    device: &Device,
    memoryProperties: &vk::PhysicalDeviceMemoryProperties,
    width: u32,
    height: u32,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
) -> anyhow::Result<(vk::Image, vk::DeviceMemory)> {
    let info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    let image =
        unsafe { device.create_image(&info, None) }.context("failed creating Vulkan GUI image")?;
    let requirements = unsafe { device.get_image_memory_requirements(image) };
    let memoryType = find_memory_type(
        memoryProperties,
        requirements.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )
    .context("no device-local memory for Vulkan GUI image")?;
    let allocationInfo = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(memoryType);
    let memory = match unsafe { device.allocate_memory(&allocationInfo, None) } {
        Ok(memory) => memory,
        Err(error) => {
            unsafe { device.destroy_image(image, None) };
            return Err(anyhow!(
                "failed allocating Vulkan GUI image memory: {error:?}"
            ));
        }
    };
    if let Err(error) = unsafe { device.bind_image_memory(image, memory, 0) } {
        unsafe {
            device.free_memory(memory, None);
            device.destroy_image(image, None);
        }
        return Err(anyhow!("failed binding Vulkan GUI image memory: {error:?}"));
    }
    Ok((image, memory))
}

fn create_image_view(
    device: &Device,
    image: vk::Image,
    format: vk::Format,
) -> anyhow::Result<vk::ImageView> {
    let info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(color_subresource_range());
    unsafe { device.create_image_view(&info, None) }
        .context("failed creating Vulkan GUI image view")
}

fn find_memory_type(
    properties: &vk::PhysicalDeviceMemoryProperties,
    allowedTypes: u32,
    required: vk::MemoryPropertyFlags,
) -> Option<u32> {
    (0..properties.memory_type_count).find(|&index| {
        allowedTypes & (1 << index) != 0
            && properties.memory_types[index as usize]
                .property_flags
                .contains(required)
    })
}

fn color_subresource_range() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1)
}

fn destroy_gui_texture(device: &Device, texture: GuiTexture) {
    unsafe {
        device.destroy_sampler(texture.sampler, None);
        device.destroy_image_view(texture.view, None);
        device.destroy_image(texture.image, None);
        device.free_memory(texture.memory, None);
    }
}

fn destroy_offscreen_image(device: &Device, image: OffscreenImage) {
    unsafe {
        device.destroy_framebuffer(image.framebuffer, None);
        device.destroy_sampler(image.sampler, None);
        device.destroy_image_view(image.view, None);
        device.destroy_image(image.image, None);
        device.free_memory(image.memory, None);
    }
}

fn destroy_mapped_buffer(device: &Device, buffer: Option<MappedBuffer>) {
    if let Some(buffer) = buffer {
        unsafe {
            device.unmap_memory(buffer.memory);
            device.destroy_buffer(buffer.buffer, None);
            device.free_memory(buffer.memory, None);
        }
    }
}

fn destroy_buffer(device: &Device, buffer: HostBuffer) {
    unsafe {
        device.destroy_buffer(buffer.buffer, None);
        device.free_memory(buffer.memory, None);
    }
}

fn as_bytes<T>(values: &[T]) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(values.as_ptr().cast::<u8>(), std::mem::size_of_val(values))
    }
}

fn bytes_of<T>(value: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts((value as *const T).cast::<u8>(), std::mem::size_of::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_constant_blocks_and_gui_vertices_match_shader_layouts() {
        assert_eq!(std::mem::size_of::<GuiPushConstants>(), 16);
        assert_eq!(std::mem::size_of::<PanoramaPushConstants>(), 16);
        assert_eq!(std::mem::size_of::<BlurPushConstants>(), 16);
        assert_eq!(std::mem::size_of::<VulkanGuiVertex>(), 36);
    }

    #[test]
    fn panorama_ping_pong_parity_matches_blur_count() {
        for blurCount in 0..12 {
            let mut current = 0usize;
            for _ in 0..blurCount {
                current = 1 - current;
            }
            assert_eq!(current, blurCount & 1);
        }
    }
}
