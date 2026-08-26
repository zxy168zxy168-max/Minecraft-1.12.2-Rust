use std::path::Path;

use anyhow::Context;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::launcher::RenderBackend::RenderBackend;
use crate::net::minecraft::client::settings::GameSettings::GameSettings;
use crate::opengl::OpenGlWindow::OpenGlWindow;
use crate::vulkan::CpuFrame::CpuFrame;
use crate::vulkan::GuiRenderFrame::GuiRenderFrame;
use crate::vulkan::VulkanWindow::VulkanWindow;
use crate::vulkan::VulkanWorldRenderer::WorldRenderFrame;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RendererExtent {
    pub width: u32,
    pub height: u32,
}

/// Native presentation backend selected before window creation. Minecraft and
/// MCP renderer classes prepare the same semantic frame for either API; only
/// native resource ownership and draw submission differ here.
pub enum DesktopRenderer {
    Vulkan(VulkanWindow),
    OpenGl(OpenGlWindow),
}

impl DesktopRenderer {
    pub fn create(
        eventLoop: &ActiveEventLoop,
        attributes: WindowAttributes,
        gameSettings: &GameSettings,
        gameDir: &Path,
    ) -> anyhow::Result<(Window, Self)> {
        match gameSettings.renderBackend {
            RenderBackend::Vulkan => {
                let window = eventLoop
                    .create_window(attributes)
                    .context("failed creating Minecraft Vulkan window")?;
                let renderer = VulkanWindow::new(&window, gameSettings)
                    .context("failed initializing Minecraft Vulkan output")?;
                Ok((window, Self::Vulkan(renderer)))
            }
            RenderBackend::OpenGl => {
                let (window, renderer) =
                    OpenGlWindow::create(eventLoop, attributes, gameSettings, gameDir)
                        .context("failed initializing Minecraft OpenGL output")?;
                Ok((window, Self::OpenGl(renderer)))
            }
        }
    }

    pub const fn backend(&self) -> RenderBackend {
        match self {
            Self::Vulkan(_) => RenderBackend::Vulkan,
            Self::OpenGl(_) => RenderBackend::OpenGl,
        }
    }

    pub fn extent(&self) -> RendererExtent {
        match self {
            Self::Vulkan(renderer) => {
                let extent = renderer.extent();
                RendererExtent {
                    width: extent.width,
                    height: extent.height,
                }
            }
            Self::OpenGl(renderer) => renderer.extent(),
        }
    }

    pub fn deviceName(&self) -> &str {
        match self {
            Self::Vulkan(renderer) => renderer.deviceName(),
            Self::OpenGl(renderer) => renderer.deviceName(),
        }
    }

    pub fn drawFrame(&mut self, window: &Window, frame: &CpuFrame) -> anyhow::Result<()> {
        match self {
            Self::Vulkan(renderer) => renderer.drawFrame(window, frame),
            Self::OpenGl(renderer) => renderer.drawFrame(window, frame),
        }
    }

    pub fn drawNativeGuiFrame(
        &mut self,
        window: &Window,
        frame: &GuiRenderFrame,
    ) -> anyhow::Result<()> {
        match self {
            Self::OpenGl(renderer) => renderer.drawNativeGuiFrame(window, frame),
            Self::Vulkan(renderer) => renderer.drawNativeGuiFrame(window, frame),
        }
    }

    pub fn drawWorldFrame(
        &mut self,
        window: &Window,
        frame: &WorldRenderFrame,
    ) -> anyhow::Result<()> {
        match self {
            Self::Vulkan(renderer) => renderer.drawWorldFrame(window, frame),
            Self::OpenGl(renderer) => renderer.drawWorldFrame(window, frame),
        }
    }

    pub fn resize(&mut self, window: &Window) -> anyhow::Result<()> {
        match self {
            Self::Vulkan(renderer) => renderer.resize(window),
            Self::OpenGl(renderer) => renderer.resize(window),
        }
    }

    pub fn setVsync(&mut self, window: &Window, enableVsync: bool) -> anyhow::Result<()> {
        match self {
            Self::Vulkan(renderer) => renderer.setVsync(window, enableVsync),
            Self::OpenGl(renderer) => renderer.setVsync(enableVsync),
        }
    }
    pub fn reloadShaderPack(&mut self) {
        if let Self::OpenGl(renderer) = self {
            renderer.reloadShaderPack();
        }
    }
}
