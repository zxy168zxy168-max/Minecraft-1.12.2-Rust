use std::ffi::OsString;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use clap::{Parser, Subcommand};

use crate::compat::Java::JavaRandom;
use crate::launcher::AssetRoot::AssetRoot;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::GuiMainMenu::{GuiMainMenu, MainMenuDate};
use crate::net::minecraft::client::gui::ScaledResolution::ScaledResolution;
use crate::net::minecraft::client::main::GameConfiguration::{
    DisplayInformation, FolderInformation, GameConfiguration, GameInformation, PropertyMap, Proxy,
    ServerInformation, UserInformation,
};
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::client::settings::GameSettings::VulkanBackendSettings;
use crate::net::minecraft::client::Minecraft::Minecraft;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::Session::Session;
use crate::net::optifine::CustomPanorama::select_custom_panorama;
use crate::vulkan::GuiCompiler::{CompiledGuiFrame, CompiledGuiStep};
use crate::vulkan::GuiDrawList::GuiDrawList;
use crate::vulkan::SoftwareGuiRenderer::SoftwareGuiRenderer;
use crate::vulkan::VulkanBackend::VulkanBackend;
use crate::GAME_VERSION;
use crate::PROTOCOL_VERSION;

#[derive(Debug, Parser)]
#[command(
    name = "mc112-client",
    version,
    about = "Minecraft Java Edition 1.12.2 Rust/Vulkan client"
)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Launch the native Minecraft window. This is the default command.
    Run {
        #[arg(long, default_value = "runtime/assets")]
        assets: PathBuf,
        #[arg(long, default_value_t = 854)]
        width: i32,
        #[arg(long, default_value_t = 480)]
        height: i32,
        #[arg(long)]
        fullscreen: bool,
        #[arg(long)]
        username: Option<String>,
        #[arg(long = "uuid")]
        player_id: Option<String>,
        #[arg(long = "accessToken")]
        access_token: Option<String>,
        #[arg(long = "userType", default_value = "legacy")]
        user_type: String,
    },
    /// Validate an imported assets directory.
    ValidateAssets {
        #[arg(long)]
        path: PathBuf,
    },
    /// Build the exact 1.12.2 main-menu draw/pass plan from imported assets.
    PlanMainMenu {
        #[arg(long)]
        assets: PathBuf,
        #[arg(long, default_value_t = 854)]
        width: i32,
        #[arg(long, default_value_t = 480)]
        height: i32,
        #[arg(long, default_value = "en_us")]
        language: String,
        #[arg(long, default_value_t = 0)]
        random_seed: i64,
        #[arg(long, default_value_t = 0)]
        system_time_millis: u64,
    },
    /// Render the current 1.12.2 main menu to a PNG without opening a window.
    RenderMainMenuPreview {
        #[arg(long, default_value = "runtime/assets")]
        assets: PathBuf,
        #[arg(long, default_value = "main-menu-preview.png")]
        output: PathBuf,
        #[arg(long, default_value_t = 854)]
        width: i32,
        #[arg(long, default_value_t = 480)]
        height: i32,
        #[arg(long, default_value_t = 0)]
        gui_scale: i32,
        #[arg(long, default_value = "en_us")]
        language: String,
        #[arg(long, default_value_t = 0)]
        random_seed: i64,
        #[arg(long, default_value_t = 0)]
        system_time_millis: u64,
    },
    /// Create a Vulkan instance and logical graphics device, then print capabilities.
    ProbeVulkan,
    /// Print the compatibility target encoded by this build.
    Version,
}

/// Rust entry corresponding to `net.minecraft.client.main.Main.main`.
pub fn main<I, T>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let command = Arguments::parse_from(args).command.unwrap_or(Command::Run {
        assets: PathBuf::from("runtime/assets"),
        width: 854,
        height: 480,
        fullscreen: false,
        username: None,
        player_id: None,
        access_token: None,
        user_type: "legacy".to_owned(),
    });

    match command {
        Command::Run {
            assets,
            width,
            height,
            fullscreen,
            username,
            player_id,
            access_token,
            user_type,
        } => {
            let gameDir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let resourcePacksDir = gameDir.join("resourcepacks");
            let millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let username = username.unwrap_or_else(|| format!("Player{}", millis % 1000));
            let playerId = player_id.unwrap_or_else(|| username.clone());
            let session = Session::new(
                username,
                playerId,
                access_token.unwrap_or_default(),
                user_type,
            );
            let gameConfiguration = GameConfiguration::new(
                UserInformation::new(
                    session,
                    PropertyMap::new(),
                    PropertyMap::new(),
                    Proxy::NoProxy,
                ),
                DisplayInformation::new(width, height, fullscreen, false),
                FolderInformation::new(gameDir, resourcePacksDir, assets, None),
                GameInformation::new(false, GAME_VERSION, "release"),
                ServerInformation::new(None, 25565),
            );
            Minecraft::new(gameConfiguration)?.run()?;
        }
        Command::ValidateAssets { path } => {
            let assets = AssetRoot::open(path).context("1.12.2 asset validation failed")?;
            println!("validated asset root: {}", assets.root().display());
            println!("asset coverage: {:?}", assets.coverage());
        }
        Command::PlanMainMenu {
            assets,
            width,
            height,
            language,
            random_seed,
            system_time_millis,
        } => {
            let assets = AssetRoot::open(assets).context("1.12.2 asset validation failed")?;
            let mut resources = ResourceManager::new();
            resources.add_directory_pack("runtime", assets.root());
            let locale = Locale::load(&resources, &[language.as_str()], &["minecraft"]);
            let mut font = FontRenderer::load(
                &resources,
                ResourceLocation::parse("textures/font/ascii.png"),
                locale.is_unicode(),
                false,
                true,
            )
            .context("failed to load Minecraft 1.12.2 font resources")?;
            let mut random = JavaRandom::new(random_seed);
            let customPanorama = select_custom_panorama(&resources, &mut random)
                .context("invalid OptiFine custom panorama properties")?;
            let mut menu = GuiMainMenu::new(&resources, &mut random, customPanorama);
            menu.initGui(
                width,
                height,
                MainMenuDate { month: 7, day: 24 },
                &locale,
                &font,
            );
            let mut drawList = GuiDrawList::new();
            menu.drawScreen(
                &mut drawList,
                &mut font,
                -1,
                -1,
                0.0,
                system_time_millis,
                "release",
                true,
            );
            let compiled = CompiledGuiFrame::compile(&drawList);
            let drawSteps = compiled
                .steps
                .iter()
                .filter(|step| matches!(step, CompiledGuiStep::Draw(_)))
                .count();
            let panoramaSteps = compiled
                .steps
                .iter()
                .filter(|step| matches!(step, CompiledGuiStep::Panorama(_)))
                .count();
            let vertices = compiled
                .steps
                .iter()
                .filter_map(|step| match step {
                    CompiledGuiStep::Draw(batch) => Some(batch.vertices.len()),
                    CompiledGuiStep::Panorama(_) => None,
                })
                .sum::<usize>();
            println!("menu: {width}x{height}, locale={language}");
            println!("splash: {}", menu.getSplashText());
            println!("source commands: {}", drawList.commands().len());
            println!(
                "compiled steps: {} draw, {} panorama",
                drawSteps, panoramaSteps
            );
            println!("compiled GUI vertices: {vertices}");
        }
        Command::RenderMainMenuPreview {
            assets,
            output,
            width,
            height,
            gui_scale,
            language,
            random_seed,
            system_time_millis,
        } => {
            anyhow::ensure!(
                width > 0 && height > 0,
                "preview dimensions must be positive"
            );
            let assets = AssetRoot::open(assets).context("1.12.2 asset validation failed")?;
            let mut resources = ResourceManager::new();
            resources.add_directory_pack("runtime", assets.root());
            let languageCodes = if language.eq_ignore_ascii_case("en_us") {
                vec!["en_us"]
            } else {
                vec!["en_us", language.as_str()]
            };
            let locale = Locale::load(&resources, &languageCodes, &["minecraft"]);
            let mut font = FontRenderer::load(
                &resources,
                ResourceLocation::parse("textures/font/ascii.png"),
                locale.is_unicode(),
                false,
                true,
            )
            .context("failed to load Minecraft 1.12.2 font resources")?;
            let scaled = ScaledResolution::new(width, height, gui_scale, locale.is_unicode());
            let mut random = JavaRandom::new(random_seed);
            let customPanorama = select_custom_panorama(&resources, &mut random)
                .context("invalid OptiFine custom panorama properties")?;
            let mut menu = GuiMainMenu::new(&resources, &mut random, customPanorama);
            menu.initGui(
                scaled.scaled_width(),
                scaled.scaled_height(),
                MainMenuDate { month: 7, day: 24 },
                &locale,
                &font,
            );
            let mut drawList = GuiDrawList::new();
            menu.drawScreen(
                &mut drawList,
                &mut font,
                -1,
                -1,
                0.0,
                system_time_millis,
                "release",
                false,
            );
            let mut renderer = SoftwareGuiRenderer::new(resources);
            let frame = renderer.render(
                &drawList,
                scaled.scaled_width(),
                scaled.scaled_height(),
                width as u32,
                height as u32,
            )?;
            write_png(&output, &frame)?;
            println!("rendered main menu preview: {}", output.display());
        }
        Command::ProbeVulkan => {
            let backend = VulkanBackend::probe(&VulkanBackendSettings::default())
                .context("Vulkan backend probe failed")?;
            let name = unsafe {
                std::ffi::CStr::from_ptr(backend.physical_device.properties.device_name.as_ptr())
            };
            println!("Vulkan device: {}", name.to_string_lossy());
            println!(
                "graphics queue family: {}",
                backend.physical_device.graphics_queue_family
            );
        }
        Command::Version => {
            println!("Minecraft {GAME_VERSION}, protocol {PROTOCOL_VERSION}");
        }
    }
    Ok(())
}

fn write_png(
    path: &std::path::Path,
    frame: &crate::vulkan::CpuFrame::CpuFrame,
) -> anyhow::Result<()> {
    let file = File::create(path)
        .with_context(|| format!("failed creating preview PNG {}", path.display()))?;
    let writer = BufWriter::new(file);
    let mut encoder = png::Encoder::new(writer, frame.width(), frame.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .context("failed writing preview PNG header")?;
    writer
        .write_image_data(frame.rgba())
        .context("failed writing preview PNG pixels")?;
    Ok(())
}
