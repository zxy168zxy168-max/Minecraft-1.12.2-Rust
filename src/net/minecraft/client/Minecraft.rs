use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Context;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{DeviceEvent, DeviceId, ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, ModifiersState, PhysicalKey},
    window::{CursorGrabMode, Fullscreen, Window, WindowId},
};

use crate::compat::Java::{math_random_f64, JavaRandom};
use crate::net::minecraft::client::audio::PositionedSoundRecord::{
    AttenuationType, PositionedSoundRecord,
};
use crate::net::minecraft::client::audio::SoundHandler::SoundHandler;
use crate::net::minecraft::client::audio::MusicTicker::{MusicTicker, MusicType};
use crate::net::minecraft::client::audio::ElytraSound::ElytraSound;
use crate::net::minecraft::crash::CrashReport::CrashReport;
use crate::net::minecraft::crash::CrashReportCategory::CrashReportCategory;
use crate::net::minecraft::util::ReportedException::ReportedException;
use crate::net::minecraft::block::Block::Block;
use crate::net::minecraft::block::SoundType::SoundType;
use crate::launcher::AssetRoot::AssetRoot;
use crate::net::minecraft::client::gui::FontRenderer::FontRenderer;
use crate::net::minecraft::client::gui::inventory::GuiInventory::GuiInventory;
use crate::net::minecraft::client::gui::inventory::GuiChest::GuiChest;
use crate::net::minecraft::client::gui::inventory::GuiShulkerBox::GuiShulkerBox;
use crate::net::minecraft::client::gui::inventory::GuiScreenHorseInventory::GuiScreenHorseInventory;
use crate::net::minecraft::client::gui::inventory::GuiCrafting::GuiCrafting;
use crate::net::minecraft::client::gui::inventory::GuiFurnace::GuiFurnace;
use crate::net::minecraft::client::gui::inventory::GuiBrewingStand::GuiBrewingStand;
use crate::net::minecraft::client::gui::inventory::GuiDispenser::GuiDispenser;
use crate::net::minecraft::client::gui::inventory::GuiBeacon::GuiBeacon;
use crate::net::minecraft::client::gui::inventory::GuiEditSign::{GuiEditSign, SignEditKey};
use crate::net::minecraft::client::gui::GuiMerchant::GuiMerchant;
use crate::net::minecraft::client::gui::recipebook::GuiRecipeBook::{RecipeBookClick, RecipeBookRenderState};
use crate::net::minecraft::client::gui::GuiHopper::GuiHopper;
use crate::net::minecraft::client::gui::GuiRepair::GuiRepair;
use crate::net::minecraft::client::gui::GuiEnchantment::{EnchantmentBookRenderState, GuiEnchantment};
use crate::net::minecraft::client::gui::inventory::GuiContainer::GuiContainer;
use crate::net::minecraft::client::gui::inventory::GuiContainerCreative::{CreativeSlotKind, GuiContainerCreative};
use crate::net::minecraft::client::gui::GuiLanguage::{GuiLanguage, GuiLanguageAction};
use crate::net::minecraft::client::gui::GuiDisconnected::{GuiDisconnected, GuiDisconnectedAction};
use crate::net::minecraft::client::gui::GuiDownloadTerrain::GuiDownloadTerrain;
use crate::net::minecraft::client::gui::GuiMainMenu::{GuiMainMenu, MainMenuAction, MainMenuDate};
use crate::net::minecraft::client::account::AccountConfig::AccountConfig;
use crate::net::minecraft::client::gui::GuiAccountManager::{AccountManagerKey, GuiAccountManager, GuiAccountManagerAction};
use crate::net::minecraft::client::gui::GuiMicrosoftAuth::{GuiMicrosoftAuth, GuiMicrosoftAuthAction};
use crate::net::minecraft::client::gui::GuiSessionLogin::{GuiSessionLogin, GuiSessionLoginAction};
use crate::net::minecraft::client::gui::GuiAltCracked::{GuiAltCracked, GuiAltCrackedAction};
use crate::net::minecraft::client::gui::GuiMultiplayer::{GuiMultiplayer, GuiMultiplayerAction};
use crate::net::minecraft::client::gui::GuiScreenAddServer::{GuiScreenAddServer, GuiScreenAddServerAction};
use crate::net::minecraft::client::gui::GuiScreenServerList::{GuiScreenServerList, GuiScreenServerListAction};
use crate::net::minecraft::client::gui::GuiTextField::{GuiTextField, GuiTextFieldKey, GuiTextFieldModifiers};
use crate::net::minecraft::client::gui::GuiChat::GuiChat;
use crate::net::minecraft::client::gui::GuiIngameMenu::{GuiIngameMenu, GuiIngameMenuAction};
use crate::net::minecraft::client::gui::GuiGameOver::{GuiGameOver, GuiGameOverAction};
use crate::net::minecraft::client::gui::GuiNewChat::GuiNewChat;
use crate::net::minecraft::client::gui::GuiYesNo::GuiYesNo;
use crate::net::minecraft::client::multiplayer::GuiConnecting::{GuiConnecting, GuiConnectingAction, GuiConnectingEvent};
use crate::net::minecraft::client::multiplayer::PlayerControllerMP::PlayerControllerMP;
use crate::net::minecraft::client::particle::ParticleManager::ParticleManager;
use crate::net::minecraft::client::renderer::color::BlockColors::BlockColors;
use crate::net::minecraft::world::ColorizerGrass::ColorizerGrass;
use crate::net::minecraft::world::ColorizerFoliage::ColorizerFoliage;
use crate::net::minecraft::client::multiplayer::ServerData::ServerData;
use crate::net::minecraft::client::gui::GuiOptions::{GuiOptions, GuiOptionsAction};
use crate::net::minecraft::client::gui::GuiControls::{GuiControls, GuiControlsAction};
use crate::net::minecraft::client::gui::GuiWorldSelection::{GuiWorldSelection, GuiWorldSelectionAction};
use crate::net::minecraft::client::gui::GuiCreateWorld::{GuiCreateWorld, GuiCreateWorldAction, WorldCreationRequest};
use crate::net::minecraft::client::gui::GuiVideoSettings::{GuiVideoSettings, GuiVideoSettingsAction};
use crate::net::minecraft::client::gui::GuiScreenOptionsSounds::{GuiScreenOptionsSounds, GuiScreenOptionsSoundsAction};
use crate::net::minecraft::client::gui::ScreenChatOptions::{ScreenChatOptions, ScreenChatOptionsAction};
use crate::net::minecraft::client::gui::GuiCustomizeSkin::{GuiCustomizeSkin, GuiCustomizeSkinAction};
use crate::net::minecraft::client::gui::GuiScreenResourcePacks::{GuiScreenResourcePacks, GuiScreenResourcePacksAction};
use crate::net::minecraft::client::gui::ScaledResolution::ScaledResolution;
use crate::net::minecraft::client::main::GameConfiguration::{GameConfiguration, PropertyMap, Proxy};
use crate::net::minecraft::client::renderer::ItemRenderer::ItemRenderer;
use crate::net::minecraft::client::resources::Locale::Locale;
use crate::net::minecraft::client::particle::ParticleSpawnRequest::ParticleSpawnRequest;
use crate::net::minecraft::client::resources::LanguageManager::LanguageManager;
use crate::net::minecraft::client::resources::SimpleReloadableResourceManager::ResourceManager;
use crate::net::minecraft::client::resources::ResourcePackRepository::{
    defaultPackIconBytes, defaultPackIconLocation, folder_assets_root, ResourcePackKind,
    ResourcePackRepository,
};
use crate::net::minecraft::client::settings::GameSettings::{GameSettings, FRAMERATE_LIMIT_MAX};
use crate::net::minecraft::client::settings::InputKeyCodes::{
    lwjgl_from_winit,
    mouse_button_from_index, mouse_button_index, mouse_code,
};
use crate::net::minecraft::client::settings::KeyBinding::KeyBindingId;
use crate::net::minecraft::util::MovementInputFromOptions::MovementKeyState;
use crate::net::minecraft::util::EnumHand::EnumHand;
use crate::net::minecraft::util::EnumActionResult::EnumActionResult;
use crate::net::minecraft::network::play::client::CPacketAnimation::CPacketAnimation;
use crate::net::minecraft::network::play::client::CPacketChatMessage::CPacketChatMessage;
use crate::net::minecraft::inventory::ClickType::ClickType;
use crate::net::minecraft::entity::player::EntityPlayer::EnumChatVisibility;
use crate::net::minecraft::entity::player::EnumPlayerModelParts::EnumPlayerModelParts;
use crate::net::minecraft::inventory::Container::Container;
use crate::net::minecraft::inventory::ContainerWindow::ContainerWindowKind;
use crate::net::minecraft::item::ItemStack::ItemStack;
use crate::net::minecraft::item::ItemTooltip;
use crate::net::minecraft::item::crafting::CraftingManager::CraftingManager;
use crate::net::minecraft::network::play::client::CPacketCloseWindow::CPacketCloseWindow;
use crate::net::minecraft::network::play::client::CPacketClientStatus::{CPacketClientStatus, State as ClientStatusState};
use crate::net::minecraft::network::play::client::CPacketClientSettings::CPacketClientSettings;
use crate::net::minecraft::network::play::client::CPacketCreativeInventoryAction::CPacketCreativeInventoryAction;
use crate::net::minecraft::network::play::client::CPacketCustomPayload::CPacketCustomPayload;
use crate::net::minecraft::network::play::client::CPacketPlaceRecipe::CPacketPlaceRecipe;
use crate::net::minecraft::network::play::client::CPacketRecipeInfo::CPacketRecipeInfo;
use crate::net::minecraft::network::play::client::CPacketUpdateSign::CPacketUpdateSign;
use crate::net::minecraft::network::PacketBuffer::{write_i32_be, write_string};
use crate::net::minecraft::network::play::client::CPacketPlayerDigging::{Action as DiggingAction, CPacketPlayerDigging};
use crate::net::minecraft::network::play::client::CPacketUseEntity::CPacketUseEntity;
use crate::net::minecraft::util::math::RayTraceResult::Type as RayTraceType;
use crate::net::minecraft::util::math::BlockPos::BlockPos;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;
use crate::net::minecraft::util::SoundCategory::SoundCategory;
use crate::net::minecraft::util::EnumParticleTypes::EnumParticleTypes;
use crate::net::minecraft::util::Session::Session;
use crate::net::minecraft::world::GameType::GameType;
use crate::net::minecraft::world::WorldSettings::WorldSettings;
use crate::net::minecraft::world::chunk::storage::AnvilSaveConverter::AnvilSaveConverter;
use crate::net::minecraft::world::storage::WorldInfo::WorldInfo;
use crate::net::minecraft::server::integrated::IntegratedServer::{IntegratedServer, IntegratedServerHandle};
use crate::net::optifine::CustomPanorama::select_custom_panorama;
use crate::net::optifine::shader::gui::GuiShader::{GuiShader, GuiShaderAction};
use crate::net::optifine::shader::Shaders::Shaders;
use crate::vulkan::CpuFrame::CpuFrame;
use crate::vulkan::GuiDrawList::GuiDrawList;
use crate::vulkan::GuiRenderFrame::GuiRenderFrame;
use crate::vulkan::NativeImage::NativeImage;
use crate::vulkan::SoftwareGuiRenderer::SoftwareGuiRenderer;
use crate::vulkan::VulkanWorldRenderer::{VulkanWorldRenderer, WorldRenderFrame};
use crate::renderer::DesktopRenderer::DesktopRenderer;

// Minecraft 1.12.2 caps GUI-only screens at 30 FPS through
// `Minecraft#getLimitFramerate`; loaded worlds continue to use the configured
// `GameSettings.limitFramerate`.
const RESIZE_DEBOUNCE: Duration = Duration::from_millis(75);
const CLIENT_TICK_INTERVAL: Duration = Duration::from_millis(50);
const RIGHT_CLICK_DELAY_TICKS: i32 = 4;

/// MCP-facing client owner. Field names deliberately follow
/// `net.minecraft.client.Minecraft` where the corresponding state already
/// exists in this port.
pub struct Minecraft {
    pub gameDir: PathBuf,
    fileAssets: PathBuf,
    fileResourcepacks: PathBuf,
    launchedVersion: String,
    versionType: String,
    profileProperties: PropertyMap,
    proxy: Proxy,
    session: Session,
    isDemo: bool,
    fullscreen: bool,
    serverName: Option<String>,
    serverPort: u16,
    tempDisplayWidth: i32,
    tempDisplayHeight: i32,
    pub displayWidth: i32,
    pub displayHeight: i32,
    pub gameSettings: GameSettings,
    pub resourceManager: ResourceManager,
    /// MCP `Minecraft#saveLoader` (`AnvilSaveConverter` in 1.12.2).
    saveLoader: AnvilSaveConverter,
    assetRoot: AssetRoot,
}

impl Minecraft {
    pub fn new(gameConfiguration: GameConfiguration) -> anyhow::Result<Self> {
        let assetRoot = AssetRoot::open(&gameConfiguration.folderInfo.assetsDir)
            .context("Minecraft 1.12.2 runtime assets are incomplete")?;
        let mut resourceManager = ResourceManager::new();
        resourceManager.add_directory_pack("runtime", assetRoot.root());
        let displayWidth = gameConfiguration.displayInfo.width.max(1);
        let displayHeight = gameConfiguration.displayInfo.height.max(1);
        let gameSettings = GameSettings::loadFromGameDir(&gameConfiguration.folderInfo.mcDataDir)
            .unwrap_or_else(|error| {
                log::warn!("failed loading options.txt; using 1.12.2 defaults: {error}");
                GameSettings::default()
            });
        match ResourcePackRepository::scan(&gameConfiguration.folderInfo.resourcePacksDir) {
            Ok(repository) => {
                for selectedName in &gameSettings.resourcePacks {
                    let Some(entry) = repository.findByName(selectedName) else {
                        log::warn!("selected resource pack is no longer present: {selectedName}");
                        continue;
                    };
                    if !entry.isCompatibleWith1122()
                        && !gameSettings.incompatibleResourcePacks.contains(selectedName)
                    {
                        log::warn!(
                            "resource pack {} uses pack_format {} and has not been confirmed as incompatible",
                            entry.resourcePackName,
                            entry.packFormat,
                        );
                        continue;
                    }
                    let result = match entry.kind {
                        ResourcePackKind::Folder => {
                            resourceManager.add_directory_pack(
                                entry.resourcePackName.clone(),
                                folder_assets_root(&entry.resourcePackFile),
                            );
                            Ok(())
                        }
                        ResourcePackKind::File => resourceManager.add_zip_pack(
                            entry.resourcePackName.clone(),
                            entry.resourcePackFile.clone(),
                        ),
                    };
                    if let Err(error) = result {
                        log::warn!("failed enabling resource pack {}: {error}", entry.resourcePackName);
                    } else {
                        log::info!(
                            "enabled Minecraft 1.12.2 resource pack {} ({})",
                            entry.resourcePackName,
                            entry.description,
                        );
                    }
                }
            }
            Err(error) => log::warn!("failed scanning resourcepacks directory: {error}"),
        }
        let saveLoader = AnvilSaveConverter::new(gameConfiguration.folderInfo.mcDataDir.join("saves"));
        Ok(Self {
            gameDir: gameConfiguration.folderInfo.mcDataDir.clone(),
            fileAssets: gameConfiguration.folderInfo.assetsDir.clone(),
            fileResourcepacks: gameConfiguration.folderInfo.resourcePacksDir.clone(),
            launchedVersion: gameConfiguration.gameInfo.version.clone(),
            versionType: gameConfiguration.gameInfo.versionType.clone(),
            profileProperties: gameConfiguration.userInfo.profileProperties.clone(),
            proxy: gameConfiguration.userInfo.proxy.clone(),
            session: gameConfiguration.userInfo.session.clone(),
            isDemo: gameConfiguration.gameInfo.isDemo,
            fullscreen: gameConfiguration.displayInfo.fullscreen || gameSettings.fullScreen,
            serverName: gameConfiguration.serverInfo.serverName.clone(),
            serverPort: gameConfiguration.serverInfo.serverPort,
            tempDisplayWidth: gameConfiguration.displayInfo.width,
            tempDisplayHeight: gameConfiguration.displayInfo.height,
            displayWidth,
            displayHeight,
            gameSettings,
            resourceManager,
            saveLoader,
            assetRoot,
        })
    }

    /// Initial source-shaped tranche of MCP `Minecraft#launchIntegratedServer`.
    ///
    /// This deliberately stops at the boundary immediately before the
    /// Yggdrasil/session-service setup and `IntegratedServer` construction.
    /// Lines before that boundary are real 1.12.2 responsibilities: obtain the
    /// save handler, load/create WorldInfo, persist a newly created level.dat,
    /// and recover WorldSettings when launching an existing save.  The next
    /// single-player tranche can continue from the returned settings without
    /// moving storage responsibilities into `GuiCreateWorld`.
    pub fn prepareIntegratedServerLaunch(
        &self,
        folderName: &str,
        worldName: &str,
        worldSettingsIn: Option<WorldSettings>,
    ) -> anyhow::Result<WorldSettings> {
        // `worldName` becomes the server/world display name in the subsequent
        // IntegratedServer/MinecraftServer stage.  It is intentionally not
        // substituted for `folderName` in the pre-server WorldInfo write.
        let _worldName = worldName;
        let saveHandler = self.saveLoader.getSaveLoader(folderName, false)?;
        let mut worldInfo = saveHandler.loadWorldInfo()?;

        if worldInfo.is_none() {
            if let Some(settings) = worldSettingsIn.as_ref() {
                // MCP uses the folder id at this pre-server checkpoint.  The
                // IntegratedServer/MinecraftServer phase later replaces the
                // display name with the user-entered world name.
                let mut info = WorldInfo::new(settings, folderName);
                saveHandler.saveWorldInfo(&mut info)?;
                worldInfo = Some(info);
            }
        }

        if let Some(settings) = worldSettingsIn {
            Ok(settings)
        } else {
            let info = worldInfo.as_ref().ok_or_else(|| {
                anyhow::anyhow!("singleplayer world {folderName:?} has no level.dat")
            })?;
            Ok(WorldSettings::fromWorldInfo(info))
        }
    }

    /// Rebuilds the reloadable manager in vanilla pack priority order:
    /// default assets first, then options.txt selected packs from low to high.
    pub fn rebuildSelectedResourcePacks(&mut self) -> Result<(), String> {
        let repository = ResourcePackRepository::scan(&self.fileResourcepacks)
            .map_err(|error| error.to_string())?;
        self.rebuildSelectedResourcePacksFromRepository(&repository)
    }

    fn rebuildSelectedResourcePacksFromRepository(
        &mut self,
        repository: &ResourcePackRepository,
    ) -> Result<(), String> {
        let mut manager = ResourceManager::new();
        manager.add_directory_pack("runtime", self.assetRoot.root());
        let mut retained = Vec::new();
        for selectedName in self.gameSettings.resourcePacks.clone() {
            let Some(entry) = repository.findByName(&selectedName) else { continue; };
            if !entry.isCompatibleWith1122()
                && !self.gameSettings.incompatibleResourcePacks.contains(&selectedName)
            {
                continue;
            }
            match entry.kind {
                ResourcePackKind::Folder => manager.add_directory_pack(
                    entry.resourcePackName.clone(),
                    folder_assets_root(&entry.resourcePackFile),
                ),
                ResourcePackKind::File => manager.add_zip_pack(
                    entry.resourcePackName.clone(),
                    entry.resourcePackFile.clone(),
                ).map_err(|error| error.to_string())?,
            }
            retained.push(selectedName);
        }
        self.gameSettings.resourcePacks = retained;
        self.resourceManager = manager;
        Ok(())
    }

    pub fn run(self) -> anyhow::Result<()> {
        let eventLoop = EventLoop::new().context("failed to create the desktop event loop")?;
        eventLoop.set_control_flow(ControlFlow::Wait);
        let mut application = MinecraftApplication::new(self);
        eventLoop.run_app(&mut application).context("Minecraft window event loop failed")?;
        if let Some(error) = application.fatalError { return Err(error); }
        Ok(())
    }

    pub fn getVersion(&self) -> &str { &self.launchedVersion }
    pub fn getVersionType(&self) -> &str { &self.versionType }
    pub const fn isDemo(&self) -> bool { self.isDemo }
    pub const fn isFullScreen(&self) -> bool { self.fullscreen }
    pub fn getSession(&self) -> &Session { &self.session }
    pub fn setSession(&mut self, session: Session) { self.session = session; }
    pub fn assetRoot(&self) -> &AssetRoot { &self.assetRoot }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScreenId { MainMenu, Options, Multiplayer, WorldSelection }

enum ActiveGuiScreen {
    Empty,
    MainMenu(GuiMainMenu),
    AccountManager(GuiAccountManager),
    MicrosoftAuth(GuiMicrosoftAuth),
    SessionLogin(GuiSessionLogin),
    OfflineLogin(GuiAltCracked),
    Options(GuiOptions),
    Controls(GuiControls),
    VideoSettings(GuiVideoSettings),
    ShaderSettings(GuiShader),
    SoundSettings(GuiScreenOptionsSounds),
    ChatSettings(ScreenChatOptions),
    SkinSettings(GuiCustomizeSkin),
    ResourcePacks(GuiScreenResourcePacks),
    Multiplayer(GuiMultiplayer),
    WorldSelection(GuiWorldSelection),
    CreateWorld(GuiCreateWorld),
    Language { screen: GuiLanguage, parent: ScreenId },
    AddServer { screen: GuiScreenAddServer, parent: Box<GuiMultiplayer>, editingIndex: Option<usize> },
    DirectConnect { screen: GuiScreenServerList, parent: Box<GuiMultiplayer> },
    ConfirmDelete { screen: GuiYesNo, parent: Box<GuiMultiplayer>, serverIndex: usize },
    Connecting { screen: GuiConnecting, parent: Box<GuiMultiplayer> },
    Disconnected { screen: GuiDisconnected, parent: Box<GuiMultiplayer> },
    DownloadTerrain { screen: GuiDownloadTerrain, connection: GuiConnecting, parent: Box<GuiMultiplayer> },
    World { connection: GuiConnecting, parent: Box<GuiMultiplayer> },
}

impl ActiveGuiScreen {
    fn isAnimated(&self) -> bool {
        matches!(self, Self::MainMenu(_)) || matches!(self, Self::Multiplayer(screen) if screen.isPinging())
    }
}

#[derive(Debug, Clone)]
enum WorldGuiScreen {
    IngameMenu(GuiIngameMenu),
    Options(GuiOptions),
    Controls(GuiControls),
    VideoSettings(GuiVideoSettings),
    ShaderSettings(GuiShader),
    SoundSettings(GuiScreenOptionsSounds),
    ChatSettings(ScreenChatOptions),
    SkinSettings(GuiCustomizeSkin),
    ResourcePacks(GuiScreenResourcePacks),
    Language(GuiLanguage),
    EditSign(GuiEditSign),
    GameOver(GuiGameOver),
    GameOverConfirm {
        parent: Box<GuiGameOver>,
        screen: GuiYesNo,
    },
}

#[derive(Debug, Clone, PartialEq)]
enum RuntimeGuiAction {
    None,
    Shutdown,
    Switch(ScreenId),
    OpenCreateWorld,
    CreateWorld(WorldCreationRequest),
    JoinWorld { folderName: String, worldName: String },
    OpenLanguage(ScreenId),
    OpenWorldLanguage,
    OpenAccountManager { notification: Option<String> },
    OpenMicrosoftAuth,
    OpenSessionLogin,
    OpenOfflineLogin,
    AccountAuthenticated { session: Session, returnToManager: bool },
    ToggleUnicode,
    SetLanguage(String),
    SetFov(f32),
    ToggleForceSprint,
    OpenControls,
    OpenWorldControls,
    ReturnToControlsParent { world: bool },
    SetMouseSensitivity(f32),
    ToggleInvertMouse,
    ToggleTouchscreen,
    ToggleAutoJump,
    SelectKeyBinding,
    SetKeyBinding { binding: KeyBindingId, code: i32 },
    ResetKeyBinding(KeyBindingId),
    ResetAllKeyBindings,
    OpenVideoSettings,
    OpenShaderSettings,
    OpenWorldShaderSettings,
    ReturnToVideoSettings,
    ReturnToWorldVideoSettings,
    SelectShaderPack(String),
    ReloadShaderPack,
    OpenShaderPackFolder,
    OpenSoundSettings,
    OpenChatSettings,
    OpenSkinSettings,
    OpenResourcePacks,
    OpenWorldSoundSettings,
    OpenWorldChatSettings,
    OpenWorldSkinSettings,
    OpenWorldResourcePacks,
    ReturnToOptions,
    ReturnToWorldOptions,
    SetSoundLevel(SoundCategory, f32),
    ToggleSubtitles,
    CycleChatVisibility,
    ToggleChatColours,
    ToggleChatLinks,
    ToggleChatLinksPrompt,
    ToggleReducedDebugInfo,
    SetChatOpacity(f32),
    SetChatScale(f32),
    SetChatWidth(f32),
    SetChatHeightFocused(f32),
    SetChatHeightUnfocused(f32),
    ToggleModelPart(EnumPlayerModelParts),
    ToggleMainHand,
    ApplyResourcePacks { selected: Vec<String>, world: bool },
    OpenResourcePackFolder,
    SetGamma(f32),
    SetRenderDistance(i32),
    SetFramerate { limit: i32, enableVsync: bool },
    ToggleGraphics,
    CycleAmbientOcclusion,
    CycleGuiScale,
    ToggleFullscreen,
    ToggleRenderBackend,
    CloseVideoSettings,
    OpenIngameMenu,
    ResumeWorld,
    ResumeWorldSaveOptions,
    FinishSignEditor,
    OpenGameOver(crate::net::minecraft::util::text::ITextComponent::ITextComponent),
    RespawnPlayer,
    OpenDeathQuitConfirm,
    ConfirmDeathQuit(bool),
    LeaveWorldToMainMenu,
    OpenWorldOptions,
    OpenWorldVideoSettings,
    ReturnToIngameMenu,
    DisconnectWorld,
    OpenDirectConnect,
    OpenAddServer,
    OpenEditServer { index: usize, server: ServerData },
    OpenDeleteConfirm { index: usize, serverName: String },
    SaveServer { editingIndex: Option<usize>, server: ServerData },
    DeleteServer { index: usize },
    Connect(ServerData),
    CancelConnecting,
    OpenDisconnected { reasonKey: &'static str, message: String },
    OpenDownloadTerrain,
    OpenWorld,
    ReturnToMultiplayer { lastServer: Option<String> },
    NotConnected(&'static str),
}

enum RuntimeFrame {
    /// Explicit software/offline fallback retained for deterministic checks.
    /// Normal OpenGL and Vulkan GUI-only screens use `NativeGui` below.
    Gui(CpuFrame),
    /// Shared MCP Gui/Tessellator command stream submitted directly by the
    /// selected native GPU backend without a full-window CPU raster roundtrip.
    NativeGui(GuiRenderFrame),
    /// In-world frames contain indexed chunk geometry and camera constants;
    /// the selected native backend performs texture sampling, depth testing
    /// and pixel rasterization.
    World(WorldRenderFrame),
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BeaconGuiAction {
    SelectPower { tier: i32, effectId: i32 },
    Confirm,
    Cancel,
}

#[derive(Debug, Clone)]
enum DedicatedContainerGui {
    Crafting(GuiCrafting),
    Furnace(GuiFurnace),
    Repair {
        gui: GuiRepair,
        nameField: GuiTextField,
        lastInputStack: ItemStack,
    },
    Enchantment(GuiEnchantment),
    Hopper(GuiHopper),
    BrewingStand(GuiBrewingStand),
    Dispenser(GuiDispenser),
    Dropper(GuiDispenser),
    Beacon(GuiBeacon),
    Merchant(GuiMerchant),
}

impl DedicatedContainerGui {
    fn new(kind: ContainerWindowKind) -> Self {
        match kind {
            ContainerWindowKind::Workbench => Self::Crafting(GuiCrafting::new()),
            ContainerWindowKind::Furnace => Self::Furnace(GuiFurnace::new()),
            ContainerWindowKind::Repair => {
                let gui = GuiRepair::new();
                let mut nameField = GuiTextField::new(
                    0,
                    GuiRepair::NAME_X,
                    GuiRepair::NAME_Y,
                    GuiRepair::NAME_WIDTH,
                    GuiRepair::NAME_HEIGHT,
                );
                nameField.setTextColor(-1);
                nameField.setDisabledTextColour(-1);
                nameField.setEnableBackgroundDrawing(false);
                nameField.setMaxStringLength(GuiRepair::NAME_MAX_LENGTH);
                nameField.setEnabled(false);
                Self::Repair {
                    gui,
                    nameField,
                    lastInputStack: ItemStack::EMPTY,
                }
            }
            ContainerWindowKind::Enchantment => Self::Enchantment(GuiEnchantment::new()),
            ContainerWindowKind::Hopper => Self::Hopper(GuiHopper::new()),
            ContainerWindowKind::BrewingStand => Self::BrewingStand(GuiBrewingStand::new()),
            ContainerWindowKind::Dispenser => Self::Dispenser(GuiDispenser::new()),
            ContainerWindowKind::Dropper => Self::Dropper(GuiDispenser::new()),
            ContainerWindowKind::Beacon => Self::Beacon(GuiBeacon::new()),
            ContainerWindowKind::Merchant => Self::Merchant(GuiMerchant::new()),
        }
    }

    const fn kind(&self) -> ContainerWindowKind {
        match self {
            Self::Crafting(_) => ContainerWindowKind::Workbench,
            Self::Furnace(_) => ContainerWindowKind::Furnace,
            Self::Repair { .. } => ContainerWindowKind::Repair,
            Self::Enchantment(_) => ContainerWindowKind::Enchantment,
            Self::Hopper(_) => ContainerWindowKind::Hopper,
            Self::BrewingStand(_) => ContainerWindowKind::BrewingStand,
            Self::Dispenser(_) => ContainerWindowKind::Dispenser,
            Self::Dropper(_) => ContainerWindowKind::Dropper,
            Self::Beacon(_) => ContainerWindowKind::Beacon,
            Self::Merchant(_) => ContainerWindowKind::Merchant,
        }
    }

    fn container(&self) -> &GuiContainer {
        match self {
            Self::Crafting(gui) => &gui.container,
            Self::Furnace(gui) => &gui.container,
            Self::Repair { gui, .. } => &gui.container,
            Self::Enchantment(gui) => &gui.container,
            Self::Hopper(gui) => &gui.container,
            Self::BrewingStand(gui) => &gui.container,
            Self::Dispenser(gui) | Self::Dropper(gui) => &gui.container,
            Self::Beacon(gui) => &gui.container,
            Self::Merchant(gui) => &gui.container,
        }
    }

    fn containerMut(&mut self) -> &mut GuiContainer {
        match self {
            Self::Crafting(gui) => &mut gui.container,
            Self::Furnace(gui) => &mut gui.container,
            Self::Repair { gui, .. } => &mut gui.container,
            Self::Enchantment(gui) => &mut gui.container,
            Self::Hopper(gui) => &mut gui.container,
            Self::BrewingStand(gui) => &mut gui.container,
            Self::Dispenser(gui) | Self::Dropper(gui) => &mut gui.container,
            Self::Beacon(gui) => &mut gui.container,
            Self::Merchant(gui) => &mut gui.container,
        }
    }

    fn initGui(&mut self, width: i32, height: i32) {
        match self {
            Self::Crafting(gui) => gui.initGui(width, height),
            Self::Furnace(gui) => gui.initGui(width, height),
            Self::Repair { gui, nameField, .. } => {
                gui.initGui(width, height);
                nameField.xPosition = gui.container.guiLeft + GuiRepair::NAME_X;
                nameField.yPosition = gui.container.guiTop + GuiRepair::NAME_Y;
            }
            Self::Enchantment(gui) => gui.initGui(width, height),
            Self::Hopper(gui) => gui.initGui(width, height),
            Self::BrewingStand(gui) => gui.initGui(width, height),
            Self::Dispenser(gui) | Self::Dropper(gui) => gui.initGui(width, height),
            Self::Beacon(gui) => gui.initGui(width, height),
            Self::Merchant(gui) => gui.initGui(width, height),
        }
    }

    fn syncRepairInput(&mut self, stack: &ItemStack, locale: &Locale) -> Option<String> {
        let Self::Repair { nameField, lastInputStack, .. } = self else {
            return None;
        };
        if lastInputStack == stack {
            return None;
        }
        *lastInputStack = stack.clone();
        if stack.isEmpty() {
            nameField.setText("");
            nameField.setEnabled(false);
            nameField.setFocused(false);
            None
        } else {
            let name = ItemTooltip::displayName(stack, locale);
            nameField.setText(&name);
            nameField.setEnabled(true);
            Some(name)
        }
    }

    fn repairNameField(&self) -> Option<&GuiTextField> {
        match self {
            Self::Repair { nameField, .. } => Some(nameField),
            _ => None,
        }
    }

    fn repairNameFieldMut(&mut self) -> Option<&mut GuiTextField> {
        match self {
            Self::Repair { nameField, .. } => Some(nameField),
            _ => None,
        }
    }

    fn repairPacketName(&self, locale: &Locale) -> Option<String> {
        let Self::Repair { nameField, lastInputStack, .. } = self else {
            return None;
        };
        let name = nameField.getText();
        if !lastInputStack.isEmpty()
            && !ItemTooltip::hasDisplayName(lastInputStack)
            && name == ItemTooltip::displayName(lastInputStack, locale)
        {
            Some(String::new())
        } else {
            Some(name)
        }
    }


    fn tickEnchantmentBook(&mut self, inputStack: &ItemStack, enchantLevels: &[i32]) {
        if let Self::Enchantment(gui) = self {
            gui.tickBook(inputStack, enchantLevels);
        }
    }

    fn enchantmentBookRenderState(&self, partialTicks: f32) -> Option<EnchantmentBookRenderState> {
        match self {
            Self::Enchantment(gui) => Some(gui.bookRenderState(partialTicks)),
            _ => None,
        }
    }
}

struct MainMenuRuntime {
    locale: Locale,
    /// MCP `Minecraft#mcLanguageManager`.
    languageManager: LanguageManager,
    fontRendererObj: FontRenderer,
    accountConfig: AccountConfig,
    currentScreen: ActiveGuiScreen,
    guiRenderer: SoftwareGuiRenderer,
    worldRenderer: VulkanWorldRenderer,
    itemRenderer: ItemRenderer,
    scaledResolution: ScaledResolution,
    mousePosition: PhysicalPosition<f64>,
    mouseInsideWindow: bool,
    lastGuiFrame: Instant,
    lastWorldRevision: u64,
    movementKeys: MovementKeyState,
    playerController: PlayerControllerMP,
    particleManager: ParticleManager,
    soundHandler: SoundHandler,
    musicTicker: MusicTicker,
    elytraSounds: Vec<ElytraSound>,
    wasElytraFlying: bool,
    worldEventRandom: JavaRandom,
    attackButtonDown: bool,
    useButtonDown: bool,
    /// Direct equivalent of Minecraft#rightClickDelayTimer. It is decremented
    /// once per 20 TPS client tick and reset to four whenever rightClickMouse
    /// actually enters its interaction branch.
    rightClickDelayTimer: i32,
    playerListKeyDown: bool,
    guiChat: Option<GuiChat>,
    worldGuiScreen: Option<WorldGuiScreen>,
    inventoryOpen: bool,
    creativeInventoryOpen: bool,
    guiInventory: GuiInventory,
    guiCreative: GuiContainerCreative,
    guiChest: Option<GuiChest>,
    guiShulkerBox: Option<GuiShulkerBox>,
    guiHorse: Option<GuiScreenHorseInventory>,
    guiDedicated: Option<DedicatedContainerGui>,
    /// Window ID mirrored from EntityPlayerSP.openContainer. Tracking it
    /// separately from row geometry preserves displayGuiScreen transitions
    /// when a server replaces one same-sized plugin menu with another.
    guiContainerWindowId: Option<i32>,
    inventoryOldMouseX: f32,
    inventoryOldMouseY: f32,
    lastInventoryClick: Option<(Instant, i32, i32)>,
    inventoryShiftClickedStack: ItemStack,
    /// Pending equivalent of Minecraft#setIngameFocus / setIngameNotInFocus
    /// requested by a server-driven GuiContainer transition. The winit
    /// application owns the actual OS cursor and consumes this request.
    pendingWorldMouseFocus: Option<bool>,
}

impl MainMenuRuntime {
    fn new(minecraft: &Minecraft, framebufferWidth: u32, framebufferHeight: u32) -> anyhow::Result<Self> {
        let language = minecraft.gameSettings.language.as_str();
        let languageCodes = if language.eq_ignore_ascii_case("en_us") { vec!["en_us"] } else { vec!["en_us", language] };
        let locale = Locale::load(&minecraft.resourceManager, &languageCodes, &["minecraft"]);
        let mut languageManager = LanguageManager::new(minecraft.gameSettings.language.clone());
        languageManager.parseLanguageMetadata(&minecraft.resourceManager.read_pack_metadatas("pack"));
        let unicode = minecraft.gameSettings.forceUnicodeFont || locale.is_unicode();
        let mut fontRendererObj = FontRenderer::load(
            &minecraft.resourceManager,
            ResourceLocation::parse("textures/font/ascii.png"),
            unicode,
            minecraft.gameSettings.anaglyph,
            minecraft.gameSettings.ofCustomFonts,
        ).context("failed loading Minecraft FontRenderer resources")?;
        fontRendererObj.set_bidi_flag(languageManager.isCurrentLanguageBidirectional());
        // Clone the MCP-facing resources before moving their primary owners
        // into `MainMenuRuntime`; the world renderer keeps its own mutable
        // FontRenderer state just as GuiIngame does in the Java client.
        let worldRenderer = VulkanWorldRenderer::new(
            minecraft.resourceManager.clone(),
            fontRendererObj.clone(),
            locale.clone(),
            minecraft.gameDir.join("skins"),
        );
        let particleManager = ParticleManager::new(BlockColors::new(
            ColorizerGrass::load(&minecraft.resourceManager),
            ColorizerFoliage::load(&minecraft.resourceManager),
        ));
        let mut soundHandler = SoundHandler::new(minecraft.resourceManager.clone());
        soundHandler.setSoundLevels(minecraft.gameSettings.soundLevels);
        let soundSeed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as i64;
        let mut runtime = Self {
            locale,
            languageManager,
            fontRendererObj,
            accountConfig: AccountConfig::load(&minecraft.gameDir),
            currentScreen: ActiveGuiScreen::MainMenu(Self::createMainMenu(minecraft)?),
            guiRenderer: SoftwareGuiRenderer::new(minecraft.resourceManager.clone()),
            worldRenderer,
            itemRenderer: ItemRenderer::new(),
            scaledResolution: ScaledResolution::new(1, 1, 1, unicode),
            mousePosition: PhysicalPosition::new(-1.0, -1.0),
            mouseInsideWindow: false,
            lastGuiFrame: Instant::now(),
            lastWorldRevision: 0,
            movementKeys: MovementKeyState::default(),
            playerController: PlayerControllerMP::new(),
            particleManager,
            soundHandler,
            musicTicker: MusicTicker::new(),
            elytraSounds: Vec::new(),
            wasElytraFlying: false,
            worldEventRandom: JavaRandom::new(soundSeed),
            attackButtonDown: false,
            useButtonDown: false,
            rightClickDelayTimer: 0,
            playerListKeyDown: false,
            guiChat: None,
            worldGuiScreen: None,
            inventoryOpen: false,
            creativeInventoryOpen: false,
            guiInventory: GuiInventory::new(),
            guiCreative: GuiContainerCreative::new(),
            guiChest: None,
            guiShulkerBox: None,
            guiHorse: None,
            guiDedicated: None,
            guiContainerWindowId: None,
            inventoryOldMouseX: 0.0,
            inventoryOldMouseY: 0.0,
            lastInventoryClick: None,
            inventoryShiftClickedStack: ItemStack::EMPTY,
            pendingWorldMouseFocus: None,
        };
        runtime.resize(minecraft, framebufferWidth, framebufferHeight);
        Ok(runtime)
    }

    fn createMainMenu(minecraft: &Minecraft) -> anyhow::Result<GuiMainMenu> {
        let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos() as i64;
        let mut random = JavaRandom::new(seed);
        let customPanorama = select_custom_panorama(&minecraft.resourceManager, &mut random)
            .context("invalid OptiFine custom panorama configuration")?;
        Ok(GuiMainMenu::new(&minecraft.resourceManager, &mut random, customPanorama))
    }

    fn isAnimated(&self) -> bool { self.currentScreen.isAnimated() }

    fn isWorld(&self) -> bool { matches!(self.currentScreen, ActiveGuiScreen::World { .. }) }

    /// MCP `Minecraft.world != null`. `GuiDownloadTerrain` is displayed only
    /// after `NetHandlerPlayClient#handleJoinGame` has created and loaded the
    /// `WorldClient`, so it must use the configured world framerate rather than
    /// the 30 FPS GUI-only cap.
    fn hasLoadedWorld(&self) -> bool {
        matches!(
            self.currentScreen,
            ActiveGuiScreen::DownloadTerrain { .. } | ActiveGuiScreen::World { .. }
        )
    }

    /// `currentScreen instanceof GuiContainer` for the two concrete container
    /// screens currently migrated: `GuiInventory` and `GuiChest`.
    fn isInventoryOpen(&self) -> bool {
        self.isWorld() && (self.inventoryOpen || self.creativeInventoryOpen || self.guiChest.is_some() || self.guiShulkerBox.is_some() || self.guiHorse.is_some() || self.guiDedicated.is_some())
    }

    /// Equivalent of `Minecraft.currentScreen != null` for the world-side
    /// screens currently migrated. A modal screen must own mouse focus and
    /// therefore suppress relative camera motion and gameplay clicks.
    fn isModalWorldGuiOpen(&self) -> bool {
        self.isInventoryOpen() || self.isChatOpen() || self.worldGuiScreen.is_some()
    }

    fn isWorldGuiOpen(&self) -> bool {
        self.isWorld() && self.worldGuiScreen.is_some()
    }

    fn controlsAwaitingBinding(&self) -> bool {
        match self.worldGuiScreen.as_ref() {
            Some(WorldGuiScreen::Controls(screen)) if screen.buttonId.is_some() => return true,
            _ => {}
        }
        matches!(&self.currentScreen, ActiveGuiScreen::Controls(screen) if screen.buttonId.is_some())
    }

    fn takeWorldMouseFocusRequest(&mut self) -> Option<bool> {
        self.pendingWorldMouseFocus.take()
    }

    /// Initializes the current world-side GuiScreen with the same scaled
    /// dimensions used by every other 1.12.2 GUI.
    fn initWorldGui(&mut self, minecraft: &Minecraft) {
        let width = self.scaledResolution.scaled_width();
        let height = self.scaledResolution.scaled_height();
        match &mut self.worldGuiScreen {
            Some(WorldGuiScreen::IngameMenu(screen)) => {
                screen.initGui(width, height, &self.locale);
            }
            Some(WorldGuiScreen::Options(screen)) => {
                screen.initGui(width, height, &self.locale, &minecraft.gameSettings);
            }
            Some(WorldGuiScreen::Controls(screen)) => {
                screen.initGui(width, height, &self.locale, &minecraft.gameSettings, &self.fontRendererObj);
            }
            Some(WorldGuiScreen::VideoSettings(screen)) => {
                screen.initGui(width, height, &self.locale, &minecraft.gameSettings);
            }
            Some(WorldGuiScreen::ShaderSettings(screen)) => screen.initGui(width, height),
            Some(WorldGuiScreen::SoundSettings(screen)) => screen.initGui(width, height, &self.locale, &minecraft.gameSettings),
            Some(WorldGuiScreen::ChatSettings(screen)) => screen.initGui(width, height, &self.locale, &minecraft.gameSettings),
            Some(WorldGuiScreen::SkinSettings(screen)) => screen.initGui(width, height, &self.locale, &minecraft.gameSettings),
            Some(WorldGuiScreen::ResourcePacks(screen)) => screen.initGui(width, height, &self.locale),
            Some(WorldGuiScreen::Language(screen)) => screen.initGui(
                width, height, &self.locale, &minecraft.gameSettings, &self.languageManager,
            ),
            Some(WorldGuiScreen::EditSign(screen)) => {
                screen.initGui(width, height, &self.locale);
            }
            Some(WorldGuiScreen::GameOver(screen)) => {
                screen.initGui(width, height, &self.locale);
            }
            Some(WorldGuiScreen::GameOverConfirm { screen, .. }) => {
                screen.initGui(width, height, &self.fontRendererObj);
            }
            None => {}
        }
    }

    /// Equivalent of `Minecraft#displayInGameMenu` for the remote-world path.
    fn openIngameMenu(&mut self, minecraft: &Minecraft) -> bool {
        if !self.isWorld() || self.isInventoryOpen() || self.isChatOpen() {
            return false;
        }
        self.worldGuiScreen = Some(WorldGuiScreen::IngameMenu(GuiIngameMenu::new()));
        self.initWorldGui(minecraft);
        self.clearMovementKeys();
        true
    }

    fn resumeWorld(&mut self) {
        self.worldGuiScreen = None;
    }

    fn openWorldOptions(&mut self, minecraft: &Minecraft) {
        if !self.isWorld() { return; }
        self.worldGuiScreen = Some(WorldGuiScreen::Options(GuiOptions::new()));
        self.initWorldGui(minecraft);
    }

    fn openWorldLanguage(&mut self, minecraft: &Minecraft) {
        if !self.isWorld() { return; }
        self.worldGuiScreen = Some(WorldGuiScreen::Language(GuiLanguage::new(
            minecraft.gameSettings.language.clone(),
        )));
        self.initWorldGui(minecraft);
    }

    fn openWorldControls(&mut self, minecraft: &Minecraft) {
        if !self.isWorld() { return; }
        self.worldGuiScreen = Some(WorldGuiScreen::Controls(GuiControls::new()));
        self.initWorldGui(minecraft);
    }

    fn openWorldVideoSettings(&mut self, minecraft: &Minecraft) {
        if !self.isWorld() { return; }
        self.worldGuiScreen = Some(WorldGuiScreen::VideoSettings(GuiVideoSettings::new()));
        self.initWorldGui(minecraft);
    }

    fn openWorldShaderSettings(&mut self, minecraft: &Minecraft, rendererDescription: String) {
        if !self.isWorld() { return; }
        self.worldGuiScreen = Some(WorldGuiScreen::ShaderSettings(GuiShader::newWithSettings(
            minecraft.gameDir.clone(),
            rendererDescription,
            minecraft.gameSettings.language.clone(),
            minecraft.gameSettings.advancedItemTooltips,
        )));
        self.initWorldGui(minecraft);
    }

    fn returnToWorldVideoSettings(&mut self, minecraft: &Minecraft) {
        if !self.isWorld() { return; }
        self.worldGuiScreen = Some(WorldGuiScreen::VideoSettings(GuiVideoSettings::new()));
        self.initWorldGui(minecraft);
    }

    fn openControls(&mut self, minecraft: &Minecraft) {
        self.currentScreen = ActiveGuiScreen::Controls(GuiControls::new());
        self.initCurrentScreen(minecraft);
    }

    fn openSoundSettings(&mut self, minecraft: &Minecraft) {
        self.currentScreen = ActiveGuiScreen::SoundSettings(GuiScreenOptionsSounds::new());
        self.initCurrentScreen(minecraft);
    }
    fn openChatSettings(&mut self, minecraft: &Minecraft) {
        self.currentScreen = ActiveGuiScreen::ChatSettings(ScreenChatOptions::new());
        self.initCurrentScreen(minecraft);
    }
    fn openSkinSettings(&mut self, minecraft: &Minecraft) {
        self.currentScreen = ActiveGuiScreen::SkinSettings(GuiCustomizeSkin::new());
        self.initCurrentScreen(minecraft);
    }
    fn decodedResourcePackIcons(repository: &ResourcePackRepository) -> Vec<(ResourceLocation, NativeImage)> {
        let mut icons = Vec::new();
        match NativeImage::decode_png(defaultPackIconBytes()) {
            Ok(image) => icons.push((defaultPackIconLocation(), image)),
            Err(error) => log::warn!("failed decoding vanilla default resource-pack icon: {error}"),
        }
        for entry in repository.getRepositoryEntriesAll() {
            let Some(bytes) = entry.iconBytes.as_deref() else { continue; };
            match NativeImage::decode_png(bytes) {
                Ok(image) => icons.push((entry.iconLocation.clone(), image)),
                Err(error) => log::warn!(
                    "failed decoding resource-pack icon for {}: {error}",
                    entry.resourcePackName,
                ),
            }
        }
        icons
    }

    fn registerMenuResourcePackIcons(&mut self, repository: &ResourcePackRepository) {
        for (location, image) in Self::decodedResourcePackIcons(repository) {
            self.guiRenderer.registerDynamicTexture(location, image);
        }
    }

    fn registerWorldResourcePackIcons(&mut self, repository: &ResourcePackRepository) {
        self.worldRenderer.setResourcePackIconTextures(Self::decodedResourcePackIcons(repository));
    }

    fn openResourcePacks(&mut self, minecraft: &Minecraft) {
        let repository = ResourcePackRepository::scan(&minecraft.fileResourcepacks).unwrap_or_default();
        self.registerMenuResourcePackIcons(&repository);
        self.currentScreen = ActiveGuiScreen::ResourcePacks(GuiScreenResourcePacks::new(
            repository, minecraft.gameSettings.resourcePacks.clone(), minecraft.fileResourcepacks.clone(),
        ));
        self.initCurrentScreen(minecraft);
    }
    fn openWorldSoundSettings(&mut self, minecraft: &Minecraft) { self.worldGuiScreen = Some(WorldGuiScreen::SoundSettings(GuiScreenOptionsSounds::new())); self.initWorldGui(minecraft); }
    fn openWorldChatSettings(&mut self, minecraft: &Minecraft) { self.worldGuiScreen = Some(WorldGuiScreen::ChatSettings(ScreenChatOptions::new())); self.initWorldGui(minecraft); }
    fn openWorldSkinSettings(&mut self, minecraft: &Minecraft) { self.worldGuiScreen = Some(WorldGuiScreen::SkinSettings(GuiCustomizeSkin::new())); self.initWorldGui(minecraft); }
    fn openWorldResourcePacks(&mut self, minecraft: &Minecraft) {
        let repository = ResourcePackRepository::scan(&minecraft.fileResourcepacks).unwrap_or_default();
        self.registerWorldResourcePackIcons(&repository);
        self.worldGuiScreen = Some(WorldGuiScreen::ResourcePacks(GuiScreenResourcePacks::new(
            repository, minecraft.gameSettings.resourcePacks.clone(), minecraft.fileResourcepacks.clone(),
        )));
        self.initWorldGui(minecraft);
    }

    fn returnToIngameMenu(&mut self, minecraft: &Minecraft) {
        let _ = self.openIngameMenu(minecraft);
    }

    /// Consumes MCP `NetHandlerPlayClient#handleSignEditorOpen`'s hand-off.
    /// Vanilla creates a temporary TileEntitySign if its chunk tile packet has
    /// not arrived yet; the same fallback is kept here.
    fn openPendingSignEditor(&mut self) {
        if !self.isWorld() { return; }
        let shared = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState(),
            _ => return,
        };
        let Some(position) = shared.takePendingSignEditorPosition() else { return; };
        let Some((sign, blockId, metadata)) = shared.withWrite(|state| {
            let world = state.worldClient.as_mut()?;
            let blockState = world.getBlockState(position);
            let sign = world.getOrCreateSignTileEntity(position);
            sign.setEditable(false);
            let sign = sign.clone();
            state.revision = state.revision.wrapping_add(1);
            Some((sign, blockState.getBlockId(), blockState.getMetadata()))
        }) else {
            return;
        };
        let mut screen = GuiEditSign::new(&sign, blockId, metadata);
        screen.initGui(
            self.scaledResolution.scaled_width(),
            self.scaledResolution.scaled_height(),
            &self.locale,
        );
        self.worldGuiScreen = Some(WorldGuiScreen::EditSign(screen));
        self.clearMovementKeys();
        self.pendingWorldMouseFocus = Some(false);
    }


    fn finishSignEditor(&mut self) -> Result<(), String> {
        let Some(WorldGuiScreen::EditSign(screen)) = self.worldGuiScreen.take() else {
            return Ok(());
        };
        let packet = CPacketUpdateSign::new(screen.getPosition(), (*screen.getLines()).clone())
            .writePacketData()
            .map_err(|error| error.to_string())?;
        let connection = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection,
            _ => return Ok(()),
        };
        let shared = connection.getSharedPlayState();
        shared.withWrite(|state| {
            if let Some(world) = state.worldClient.as_mut() {
                let sign = world.getOrCreateSignTileEntity(screen.getPosition());
                screen.finishTileEntity(sign);
                state.revision = state.revision.wrapping_add(1);
            }
        });
        connection.sendPlayPackets(vec![packet])?;
        self.pendingWorldMouseFocus = Some(true);
        Ok(())
    }

    fn openGameOver(&mut self, message: crate::net::minecraft::util::text::ITextComponent::ITextComponent) {
        if !self.isWorld() { return; }
        let (hardcore, score) = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState().withRead(|state| {
                (
                    state.hardcoreMode,
                    state.thePlayer.as_ref().map_or(0, |player| player.getScore()),
                )
            }),
            _ => return,
        };
        let mut screen = GuiGameOver::new(Some(message), hardcore, score);
        screen.initGui(
            self.scaledResolution.scaled_width(),
            self.scaledResolution.scaled_height(),
            &self.locale,
        );
        self.worldGuiScreen = Some(WorldGuiScreen::GameOver(screen));
        self.clearMovementKeys();
        self.pendingWorldMouseFocus = Some(false);
    }

    fn respawnPlayer(&mut self) -> Result<(), String> {
        let connection = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection,
            _ => return Ok(()),
        };
        connection.sendPlayPackets(vec![
            CPacketClientStatus::new(ClientStatusState::PerformRespawn).writePacketData(),
        ])?;
        self.worldGuiScreen = None;
        self.pendingWorldMouseFocus = Some(true);
        Ok(())
    }

    fn openDeathQuitConfirm(&mut self) {
        let Some(WorldGuiScreen::GameOver(parent)) = self.worldGuiScreen.take() else { return; };
        if parent.isHardcore() {
            self.worldGuiScreen = Some(WorldGuiScreen::GameOver(parent));
            return;
        }
        let mut screen = GuiYesNo::new(
            self.locale.translate_key("deathScreen.quit.confirm").to_owned(),
            String::new(),
            self.locale.translate_key("deathScreen.titleScreen").to_owned(),
            self.locale.translate_key("deathScreen.respawn").to_owned(),
            0,
        );
        screen.initGui(
            self.scaledResolution.scaled_width(),
            self.scaledResolution.scaled_height(),
            &self.fontRendererObj,
        );
        screen.setButtonDelay(20);
        self.worldGuiScreen = Some(WorldGuiScreen::GameOverConfirm {
            parent: Box::new(parent),
            screen,
        });
    }

    fn cancelDeathQuitAndRespawn(&mut self) -> Result<(), String> {
        self.respawnPlayer()
    }

    fn activeGuiContainer(&self) -> Option<&GuiContainer> {
        if self.creativeInventoryOpen {
            Some(&self.guiCreative.container)
        } else if self.inventoryOpen {
            Some(&self.guiInventory.container)
        } else if let Some(gui) = self.guiChest.as_ref() {
            Some(&gui.container)
        } else if let Some(gui) = self.guiShulkerBox.as_ref() {
            Some(&gui.container)
        } else if let Some(gui) = self.guiHorse.as_ref() {
            Some(&gui.container)
        } else {
            self.guiDedicated.as_ref().map(DedicatedContainerGui::container)
        }
    }

    fn activeGuiContainerMut(&mut self) -> Option<&mut GuiContainer> {
        if self.creativeInventoryOpen {
            Some(&mut self.guiCreative.container)
        } else if self.inventoryOpen {
            Some(&mut self.guiInventory.container)
        } else if let Some(gui) = self.guiChest.as_mut() {
            Some(&mut gui.container)
        } else if let Some(gui) = self.guiShulkerBox.as_mut() {
            Some(&mut gui.container)
        } else if let Some(gui) = self.guiHorse.as_mut() {
            Some(&mut gui.container)
        } else {
            self.guiDedicated.as_mut().map(DedicatedContainerGui::containerMut)
        }
    }

    fn activeSlotAt(&self, mouseX: i32, mouseY: i32) -> Option<i32> {
        self.activeGuiContainer()?.slotAt(mouseX, mouseY)
    }

    fn activeProtocolSlotAt(&self, mouseX: i32, mouseY: i32) -> i32 {
        let Some(container) = self.activeGuiContainer() else { return -1; };
        if let Some(slot) = container.slotAt(mouseX, mouseY) { return slot; }
        let recipeOutside = if self.inventoryOpen {
            self.guiInventory.recipeBook.isPointOutside(
                mouseX, mouseY, container.guiLeft, container.guiTop, container.xSize, container.ySize,
            )
        } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_ref() {
            gui.recipeBook.isPointOutside(
                mouseX, mouseY, container.guiLeft, container.guiTop, container.xSize, container.ySize,
            )
        } else {
            container.isOutsideGui(mouseX, mouseY)
        };
        if recipeOutside { -999 } else { -1 }
    }

    /// Synchronizes the concrete `GuiRecipeBook` from `EntityPlayerSP` and the
    /// active crafting matrix. Network state remains authoritative; this only
    /// rebuilds the client-side availability/search/page model and consumes a
    /// server-requested ghost recipe for the matching window.
    fn syncRecipeBookGui(&mut self, resetPage: bool) {
        let shared = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState(),
            _ => return,
        };
        let inventoryScreen = self.inventoryOpen;
        let workbenchScreen = matches!(self.guiDedicated.as_ref(), Some(DedicatedContainerGui::Crafting(_)));
        if !inventoryScreen && !workbenchScreen { return; }
        let Some((book, inventory, craftingStacks, windowId)) = shared.withRead(|state| {
            let player = state.thePlayer.as_ref()?;
            if inventoryScreen {
                let stacks = (1..=4)
                    .map(|slot| player.inventoryContainer.getSlot(slot).cloned().unwrap_or(ItemStack::EMPTY))
                    .collect::<Vec<_>>();
                Some((player.recipeBook.clone(), player.inventory.clone(), stacks, 0))
            } else {
                let container = player.openContainer.as_ref()?;
                if container.windowKind() != Some(ContainerWindowKind::Workbench) { return None; }
                let stacks = (1..=9)
                    .map(|slot| container.getSlot(slot).cloned().unwrap_or(ItemStack::EMPTY))
                    .collect::<Vec<_>>();
                Some((player.recipeBook.clone(), player.inventory.clone(), stacks, container.windowId()))
            }
        }) else { return; };

        if inventoryScreen {
            self.guiInventory.rebuildRecipeBook(
                &book, &inventory, &craftingStacks, resetPage, &self.locale,
            );
        } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
            gui.rebuildRecipeBook(&book, &inventory, &craftingStacks, resetPage, &self.locale);
        }

        // MCP `GuiButtonRecipe#func_193928_a` notifies `GuiRecipeBook` when a
        // current-page button contains a newly unlocked recipe. Vanilla then
        // calls `EntityPlayerSP#func_193103_a`: mark the recipe seen locally
        // and send one `CPacketRecipeInfo(SHOWN)` packet. Without this step the
        // server keeps resending the same recipes as new on later sessions.
        let newlyDisplayed = if inventoryScreen {
            self.guiInventory.recipeBook.newlyDisplayedRecipeIds(&book)
        } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_ref() {
            gui.recipeBook.newlyDisplayedRecipeIds(&book)
        } else {
            Vec::new()
        };
        if !newlyDisplayed.is_empty() {
            let packets = shared.withWrite(|state| {
                let Some(player) = state.thePlayer.as_mut() else { return Vec::new(); };
                let mut packets = Vec::new();
                for recipeId in &newlyDisplayed {
                    let Some(recipe) = CraftingManager::getRecipe(*recipeId) else { continue; };
                    if !player.recipeBook.isNew(recipe) { continue; }
                    player.recipeBook.markSeen(recipe);
                    if let Ok(packet) = CPacketRecipeInfo::shown(*recipeId) {
                        packets.push(packet.writePacketData());
                    }
                }
                packets
            });
            if !packets.is_empty() {
                if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
                    if let Err(error) = connection.sendPlayPackets(packets) {
                        log::warn!("failed to acknowledge displayed recipe-book entries: {error}");
                    }
                }
            }
        }

        let pending = shared.withWrite(|state| {
            let pending = state.pendingGhostRecipe?;
            if i32::from(pending.0) != windowId { return None; }
            state.pendingGhostRecipe.take()
        });
        if let Some((_window, recipeId)) = pending {
            if inventoryScreen {
                let positions = self.guiInventory.craftingSlotPositions();
                self.guiInventory.recipeBook.placeGhostRecipe(recipeId, &positions, 2, 2);
            } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
                let positions = gui.craftingSlotPositions();
                gui.recipeBook.placeGhostRecipe(recipeId, &positions, 3, 3);
            }
        }
    }

    fn recipeBookRenderState(&mut self) -> Option<RecipeBookRenderState> {
        self.syncRecipeBookGui(false);
        let shared = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState(),
            _ => return None,
        };
        let book = shared.withRead(|state| state.thePlayer.as_ref().map(|player| player.recipeBook.clone()))?;
        if self.inventoryOpen {
            Some(self.guiInventory.recipeBook.renderState(true, &book, GuiInventory::X_SIZE, &self.fontRendererObj))
        } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_ref() {
            Some(gui.recipeBook.renderState(false, &book, GuiCrafting::X_SIZE, &self.fontRendererObj))
        } else {
            None
        }
    }

    fn recipeBookMouseClicked(
        &mut self,
        mouseX: i32,
        mouseY: i32,
        mouseButton: i32,
        shiftHeld: bool,
    ) -> Result<Option<bool>, String> {
        self.syncRecipeBookGui(false);
        let inventoryScreen = self.inventoryOpen;
        let workbenchScreen = matches!(self.guiDedicated.as_ref(), Some(DedicatedContainerGui::Crafting(_)));
        if !inventoryScreen && !workbenchScreen { return Ok(None); }
        let shared = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState(),
            _ => return Ok(None),
        };
        let Some((mut book, windowId)) = shared.withRead(|state| {
            let player = state.thePlayer.as_ref()?;
            let windowId = if inventoryScreen { 0 } else { player.openContainer.as_ref()?.windowId() };
            Some((player.recipeBook.clone(), windowId))
        }) else { return Ok(None); };

        let click = if inventoryScreen {
            self.guiInventory.recipeBook.click(
                true, mouseX, mouseY, mouseButton, shiftHeld, &mut book, &self.fontRendererObj,
            )
        } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
            gui.recipeBook.click(false, mouseX, mouseY, mouseButton, shiftHeld, &mut book, &self.fontRendererObj)
        } else {
            RecipeBookClick::None
        };
        let narrowOpen = if inventoryScreen {
            self.guiInventory.widthTooNarrow && self.guiInventory.recipeBook.isOpen()
        } else {
            self.guiDedicated.as_ref().is_some_and(|gui| matches!(gui, DedicatedContainerGui::Crafting(crafting) if crafting.widthTooNarrow && crafting.recipeBook.isOpen()))
        };

        match click {
            RecipeBookClick::None => return Ok(narrowOpen.then_some(true)),
            RecipeBookClick::Consumed => {
                self.syncRecipeBookGui(false);
                return Ok(Some(true));
            }
            RecipeBookClick::SettingsChanged { open, filtering } => {
                shared.withWrite(|state| {
                    if let Some(player) = state.thePlayer.as_mut() {
                        player.recipeBook.setGuiOpen(open);
                        player.recipeBook.setFilteringCraftable(filtering);
                    }
                });
                let packet = CPacketRecipeInfo::settings(open, filtering).writePacketData();
                if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
                    connection.sendPlayPackets(vec![packet])?;
                }
                self.syncRecipeBookGui(false);
                return Ok(Some(true));
            }
            RecipeBookClick::PlaceRecipe { recipeId, placeAll, closeBook } => {
                let packet = CPacketPlaceRecipe::new(windowId, recipeId, placeAll)
                    .map_err(|error| error.to_string())?
                    .writePacketData();
                if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
                    connection.sendPlayPackets(vec![packet])?;
                }
                if closeBook {
                    let filtering = book.isFilteringCraftable();
                    shared.withWrite(|state| {
                        if let Some(player) = state.thePlayer.as_mut() {
                            player.recipeBook.setGuiOpen(false);
                        }
                    });
                    let settings = CPacketRecipeInfo::settings(false, filtering).writePacketData();
                    if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
                        connection.sendPlayPackets(vec![settings])?;
                    }
                    self.syncRecipeBookGui(false);
                }
                return Ok(Some(true));
            }
        }
    }

    fn recipeBookKeyPressed(
        &mut self,
        key: KeyCode,
        binding: Option<KeyBindingId>,
        modifiers: ModifiersState,
        eventText: Option<&str>,
    ) -> Result<bool, String> {
        let inventoryScreen = self.inventoryOpen;
        let workbenchScreen = matches!(
            self.guiDedicated.as_ref(),
            Some(DedicatedContainerGui::Crafting(_))
        );
        if !inventoryScreen && !workbenchScreen { return Ok(false); }
        self.syncRecipeBookGui(false);

        let shared = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState(),
            _ => return Ok(false),
        };
        let Some(mut book) = shared.withRead(|state| {
            state.thePlayer.as_ref().map(|player| player.recipeBook.clone())
        }) else { return Ok(false); };

        if key == KeyCode::Escape {
            let action = if inventoryScreen {
                self.guiInventory.recipeBook.closeOnEscape(&mut book)
            } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
                gui.recipeBook.closeOnEscape(&mut book)
            } else {
                None
            };
            if let Some(RecipeBookClick::SettingsChanged { open, filtering }) = action {
                shared.withWrite(|state| {
                    if let Some(player) = state.thePlayer.as_mut() {
                        player.recipeBook.setGuiOpen(open);
                        player.recipeBook.setFilteringCraftable(filtering);
                    }
                });
                let packet = CPacketRecipeInfo::settings(open, filtering).writePacketData();
                if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
                    connection.sendPlayPackets(vec![packet])?;
                }
                self.syncRecipeBookGui(false);
                return Ok(true);
            }
        }

        if binding == Some(KeyBindingId::Chat) {
            let focused = if inventoryScreen {
                self.guiInventory.recipeBook.focusSearchFromChatKey()
            } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
                gui.recipeBook.focusSearchFromChatKey()
            } else {
                false
            };
            if focused { return Ok(true); }
        }

        let textModifiers = GuiTextFieldModifiers {
            control: modifiers.control_key(),
            shift: modifiers.shift_key(),
        };
        if key == KeyCode::KeyA && modifiers.control_key() {
            let handled = if inventoryScreen {
                self.guiInventory.recipeBook.selectAllSearch(&self.fontRendererObj)
            } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
                gui.recipeBook.selectAllSearch(&self.fontRendererObj)
            } else {
                false
            };
            if handled { return Ok(true); }
        }

        let textKey = match key {
            KeyCode::Backspace => Some(GuiTextFieldKey::Backspace),
            KeyCode::Delete => Some(GuiTextFieldKey::Delete),
            KeyCode::ArrowLeft => Some(GuiTextFieldKey::Left),
            KeyCode::ArrowRight => Some(GuiTextFieldKey::Right),
            KeyCode::Home => Some(GuiTextFieldKey::Home),
            KeyCode::End => Some(GuiTextFieldKey::End),
            _ => None,
        };
        if let Some(textKey) = textKey {
            let (handled, changed) = if inventoryScreen {
                self.guiInventory.recipeBook.keyPressed(
                    textKey, textModifiers, &self.fontRendererObj,
                )
            } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
                gui.recipeBook.keyPressed(textKey, textModifiers, &self.fontRendererObj)
            } else {
                (false, false)
            };
            if changed { self.syncRecipeBookGui(false); }
            if handled { return Ok(true); }
        }

        if !modifiers.control_key() && !modifiers.alt_key() {
            if let Some(text) = eventText {
                let changed = if inventoryScreen {
                    self.guiInventory.recipeBook.typedText(text, &self.fontRendererObj)
                } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
                    gui.recipeBook.typedText(text, &self.fontRendererObj)
                } else {
                    false
                };
                if changed {
                    self.syncRecipeBookGui(false);
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn sendAnvilName(&self, name: &str) -> Result<bool, String> {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
            return Ok(false);
        };
        let mut data = Vec::new();
        write_string(name, 32_767, &mut data).map_err(|error| error.to_string())?;
        let packet = CPacketCustomPayload::new("MC|ItemName", data)
            .and_then(|payload| payload.writePacketData())
            .map_err(|error| error.to_string())?;
        connection.sendPlayPackets(vec![packet])?;
        Ok(true)
    }

    fn repairTypedText(&mut self, text: &str) -> Result<bool, String> {
        let changed = self
            .guiDedicated
            .as_mut()
            .and_then(DedicatedContainerGui::repairNameFieldMut)
            .is_some_and(|field| {
                field.isFocused() && field.writeText(text, Some(&self.fontRendererObj))
            });
        if !changed {
            return Ok(false);
        }
        let name = self
            .guiDedicated
            .as_ref()
            .and_then(|gui| gui.repairPacketName(&self.locale))
            .unwrap_or_default();
        self.sendAnvilName(&name)
    }

    fn repairKeyPressed(
        &mut self,
        key: KeyCode,
        modifiers: ModifiersState,
    ) -> Result<bool, String> {
        if !matches!(self.guiDedicated.as_ref(), Some(DedicatedContainerGui::Repair { .. })) {
            return Ok(false);
        }
        let textModifiers = GuiTextFieldModifiers {
            control: modifiers.control_key(),
            shift: modifiers.shift_key(),
        };
        let changed = if key == KeyCode::KeyA && modifiers.control_key() {
            if let Some(field) = self
                .guiDedicated
                .as_mut()
                .and_then(DedicatedContainerGui::repairNameFieldMut)
            {
                if field.isFocused() {
                    field.selectAll(&self.fontRendererObj);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            let textKey = match key {
                KeyCode::Backspace => Some(GuiTextFieldKey::Backspace),
                KeyCode::Delete => Some(GuiTextFieldKey::Delete),
                KeyCode::ArrowLeft => Some(GuiTextFieldKey::Left),
                KeyCode::ArrowRight => Some(GuiTextFieldKey::Right),
                KeyCode::Home => Some(GuiTextFieldKey::Home),
                KeyCode::End => Some(GuiTextFieldKey::End),
                _ => None,
            };
            textKey.is_some_and(|textKey| {
                self.guiDedicated
                    .as_mut()
                    .and_then(DedicatedContainerGui::repairNameFieldMut)
                    .is_some_and(|field| {
                        field.keyPressed(textKey, textModifiers, &self.fontRendererObj)
                    })
            })
        };
        if !changed {
            return Ok(false);
        }
        let name = self
            .guiDedicated
            .as_ref()
            .and_then(|gui| gui.repairPacketName(&self.locale))
            .unwrap_or_default();
        self.sendAnvilName(&name)
    }

    /// Reflect `EntityPlayer.openContainer` into the concrete client GUI.
    /// `SPacketOpenWindow` is processed on the network thread, while this
    /// runtime owns screen geometry and therefore synchronizes at frame/input
    /// boundaries just as Minecraft schedules packet work onto its main thread.
    fn syncOpenContainerGui(&mut self) {
        let sharedState = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState(),
            _ => {
                self.guiChest = None;
                self.guiShulkerBox = None;
                self.guiHorse = None;
                self.guiDedicated = None;
                self.guiContainerWindowId = None;
                self.pendingWorldMouseFocus = None;
                return;
            }
        };
        let openWindow = sharedState.withRead(|state| {
            state.thePlayer.as_ref()?.openContainer.as_ref().map(|container| {
                (
                    container.windowId(),
                    container.getNumRows() as i32,
                    container.isShulkerBox(),
                    container.horseInventorySpec(),
                    container.windowKind(),
                )
            })
        });
        match openWindow {
            Some((windowId, rows, shulkerBox, horseSpec, dedicatedKind)) => {
                if self.inventoryOpen {
                    self.inventoryOpen = false;
                    self.guiInventory.container.resetInteraction();
                }
                if self.creativeInventoryOpen {
                    self.creativeInventoryOpen = false;
                    self.guiCreative.container.resetInteraction();
                }
                self.closeChat();

                let currentKindMatches = if let Some(kind) = dedicatedKind {
                    self.guiDedicated.as_ref().is_some_and(|gui| gui.kind() == kind)
                        && self.guiChest.is_none()
                        && self.guiShulkerBox.is_none()
                        && self.guiHorse.is_none()
                } else if shulkerBox {
                    self.guiShulkerBox.is_some()
                        && self.guiChest.is_none()
                        && self.guiHorse.is_none()
                        && self.guiDedicated.is_none()
                } else if let Some(spec) = horseSpec {
                    self.guiHorse.as_ref().is_some_and(|gui| gui.spec == spec)
                        && self.guiChest.is_none()
                        && self.guiShulkerBox.is_none()
                        && self.guiDedicated.is_none()
                } else {
                    self.guiChest.as_ref().is_some_and(|gui| gui.inventoryRows == rows)
                        && self.guiShulkerBox.is_none()
                        && self.guiHorse.is_none()
                        && self.guiDedicated.is_none()
                };
                let needsCreate = !currentKindMatches || self.guiContainerWindowId != Some(windowId);
                if needsCreate {
                    if let Some(mut previous) = self.guiChest.take() {
                        previous.container.resetInteraction();
                    }
                    if let Some(mut previous) = self.guiShulkerBox.take() {
                        previous.container.resetInteraction();
                    }
                    if let Some(mut previous) = self.guiHorse.take() {
                        previous.container.resetInteraction();
                    }
                    if let Some(mut previous) = self.guiDedicated.take() {
                        previous.containerMut().resetInteraction();
                    }
                    if let Some(kind) = dedicatedKind {
                        let mut gui = DedicatedContainerGui::new(kind);
                        gui.initGui(
                            self.scaledResolution.scaled_width(),
                            self.scaledResolution.scaled_height(),
                        );
                        self.guiDedicated = Some(gui);
                    } else if shulkerBox {
                        let mut gui = GuiShulkerBox::new();
                        gui.initGui(
                            self.scaledResolution.scaled_width(),
                            self.scaledResolution.scaled_height(),
                        );
                        self.guiShulkerBox = Some(gui);
                    } else if let Some(spec) = horseSpec {
                        let mut gui = GuiScreenHorseInventory::new(spec);
                        gui.initGui(
                            self.scaledResolution.scaled_width(),
                            self.scaledResolution.scaled_height(),
                        );
                        self.guiHorse = Some(gui);
                    } else {
                        let mut gui = GuiChest::new(rows);
                        gui.initGui(
                            self.scaledResolution.scaled_width(),
                            self.scaledResolution.scaled_height(),
                        );
                        self.guiChest = Some(gui);
                    }
                    self.guiContainerWindowId = Some(windowId);
                    self.lastInventoryClick = None;
                    self.inventoryShiftClickedStack = ItemStack::EMPTY;
                    self.clearMovementKeys();
                    self.pendingWorldMouseFocus = Some(false);
                }

                if dedicatedKind == Some(ContainerWindowKind::Repair) {
                    let inputStack = sharedState.withRead(|state| {
                        state
                            .thePlayer
                            .as_ref()
                            .and_then(|player| player.openContainer.as_ref())
                            .and_then(|container| container.getSlot(0))
                            .cloned()
                            .unwrap_or(ItemStack::EMPTY)
                    });
                    let renamed = self
                        .guiDedicated
                        .as_mut()
                        .and_then(|gui| gui.syncRepairInput(&inputStack, &self.locale))
                        .is_some();
                    if renamed {
                        let name = self
                            .guiDedicated
                            .as_ref()
                            .and_then(|gui| gui.repairPacketName(&self.locale))
                            .unwrap_or_default();
                        if let Err(error) = self.sendAnvilName(&name) {
                            log::error!("failed sending MC|ItemName after anvil input sync: {error}");
                        }
                    }
                }
            }
            None => {
                let mut closed = false;
                if let Some(mut gui) = self.guiChest.take() {
                    gui.container.resetInteraction();
                    closed = true;
                }
                if let Some(mut gui) = self.guiShulkerBox.take() {
                    gui.container.resetInteraction();
                    closed = true;
                }
                if let Some(mut gui) = self.guiHorse.take() {
                    gui.container.resetInteraction();
                    closed = true;
                }
                if let Some(mut gui) = self.guiDedicated.take() {
                    gui.containerMut().resetInteraction();
                    closed = true;
                }
                if closed {
                    self.guiContainerWindowId = None;
                    self.lastInventoryClick = None;
                    self.inventoryShiftClickedStack = ItemStack::EMPTY;
                    self.pendingWorldMouseFocus = Some(true);
                }
            }
        }
    }

    /// `GuiInventory#updateScreen` and `GuiContainerCreative#updateScreen`
    /// replace one another as soon as PlayerControllerMP changes game type.
    /// This also covers server `/gamemode` changes while the screen remains
    /// open rather than requiring the player to close and reopen it.
    fn syncInventoryGameType(&mut self) {
        if !self.inventoryOpen && !self.creativeInventoryOpen { return; }
        let creative = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection
                .getSharedPlayState()
                .withRead(|state| state.gameType == GameType::Creative),
            _ => return,
        };
        if creative == self.creativeInventoryOpen { return; }

        self.guiInventory.container.resetInteraction();
        self.guiCreative.container.resetInteraction();
        self.inventoryOpen = !creative;
        self.creativeInventoryOpen = creative;
        self.lastInventoryClick = None;
        self.inventoryShiftClickedStack = ItemStack::EMPTY;
        if creative {
            self.guiCreative.initGui(
                self.scaledResolution.scaled_width(),
                self.scaledResolution.scaled_height(),
            );
        } else {
            self.guiInventory.initGui(
                self.scaledResolution.scaled_width(),
                self.scaledResolution.scaled_height(),
            );
        }
    }

    fn openInventory(&mut self) -> bool {
        if !self.isWorld() || self.isInventoryOpen() { return false; }
        let creative = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection
                .getSharedPlayState()
                .withRead(|state| state.gameType == GameType::Creative),
            _ => false,
        };
        self.inventoryOpen = !creative;
        self.creativeInventoryOpen = creative;
        self.inventoryOldMouseX = 0.0;
        self.inventoryOldMouseY = 0.0;
        self.lastInventoryClick = None;
        self.inventoryShiftClickedStack = ItemStack::EMPTY;
        if creative {
            self.guiCreative.container.resetInteraction();
            self.guiCreative.initGui(
                self.scaledResolution.scaled_width(),
                self.scaledResolution.scaled_height(),
            );
        } else {
            self.guiInventory.container.resetInteraction();
            self.guiInventory.initGui(
                self.scaledResolution.scaled_width(),
                self.scaledResolution.scaled_height(),
            );
        }
        self.clearMovementKeys();
        true
    }

    fn closeInventory(&mut self) -> Result<bool, String> {
        if !self.isInventoryOpen() { return Ok(false); }
        let mut windowId = 0;
        if self.creativeInventoryOpen {
            self.creativeInventoryOpen = false;
            self.guiCreative.container.resetInteraction();
        } else if self.inventoryOpen {
            self.inventoryOpen = false;
            self.guiInventory.container.resetInteraction();
        } else if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
            let shared = connection.getSharedPlayState();
            windowId = shared.withRead(|state| {
                state.thePlayer.as_ref()
                    .and_then(|player| player.openContainer.as_ref())
                    .map_or(0, |container| container.windowId())
            });
            shared.closeOpenContainer(windowId);
            if let Some(mut gui) = self.guiChest.take() {
                gui.container.resetInteraction();
            }
            if let Some(mut gui) = self.guiShulkerBox.take() {
                gui.container.resetInteraction();
            }
            if let Some(mut gui) = self.guiHorse.take() {
                gui.container.resetInteraction();
            }
            if let Some(mut gui) = self.guiDedicated.take() {
                gui.containerMut().resetInteraction();
            }
            self.guiContainerWindowId = None;
        }
        self.lastInventoryClick = None;
        self.inventoryShiftClickedStack = ItemStack::EMPTY;
        if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
            connection.sendPlayPackets(vec![CPacketCloseWindow::new(windowId).writePacketData()])?;
        }
        Ok(true)
    }

    fn setMovementBinding(&mut self, binding: KeyBindingId, pressed: bool) -> bool {
        let target = match binding {
            KeyBindingId::Forward => &mut self.movementKeys.forward,
            KeyBindingId::Back => &mut self.movementKeys.back,
            KeyBindingId::Left => &mut self.movementKeys.left,
            KeyBindingId::Right => &mut self.movementKeys.right,
            KeyBindingId::Jump => &mut self.movementKeys.jump,
            KeyBindingId::Sneak => &mut self.movementKeys.sneak,
            KeyBindingId::Sprint => &mut self.movementKeys.sprint,
            _ => return false,
        };
        *target = pressed;
        true
    }

    fn clearMovementKeys(&mut self) {
        self.movementKeys = MovementKeyState::default();
        self.attackButtonDown = false;
        self.useButtonDown = false;
        self.playerListKeyDown = false;
    }

    fn setPlayerListKeyDown(&mut self, pressed: bool) -> bool {
        if !self.isWorld() || self.isInventoryOpen() || self.isChatOpen() { return false; }
        let changed = self.playerListKeyDown != pressed;
        self.playerListKeyDown = pressed;
        changed
    }

    fn currentPlayerTicks(&self) -> i32 {
        match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection
                .getSharedPlayState()
                .withRead(|state| state.thePlayer.as_ref().map_or(0, |player| player.entity.ticksExisted)),
            _ => 0,
        }
    }

    fn debugChatWidth(gameSettings: &GameSettings) -> i32 {
        ((GuiNewChat::calculateChatboxWidth(gameSettings.chatWidth.clamp(0.0, 1.0)) as f32)
            / gameSettings.chatScale.max(0.01))
            .floor() as i32
    }

    fn printDebugMessage(&mut self, key: &str, gameSettings: &GameSettings) {
        let prefix = self.locale.translate_key("debug.prefix");
        let message = self.locale.translate_key(key);
        self.worldRenderer.printDebugMessage(
            format!("§e§l{prefix}§r {message}"),
            self.currentPlayerTicks(),
            Self::debugChatWidth(gameSettings),
        );
    }

    fn printDebugValue(
        &mut self,
        key: &str,
        value: impl AsRef<str>,
        gameSettings: &GameSettings,
    ) {
        let template = self.locale.translate_key(key);
        let rendered = template.replacen("%s", value.as_ref(), 1);
        let prefix = self.locale.translate_key("debug.prefix");
        self.worldRenderer.printDebugMessage(
            format!("§e§l{prefix}§r {rendered}"),
            self.currentPlayerTicks(),
            Self::debugChatWidth(gameSettings),
        );
    }

    fn printDebugHelp(&mut self, gameSettings: &GameSettings) {
        self.printDebugMessage("debug.help.message", gameSettings);
        for key in [
            "debug.reload_chunks.help",
            "debug.show_hitboxes.help",
            "debug.clear_chat.help",
            "debug.cycle_renderdistance.help",
            "debug.chunk_boundaries.help",
            "debug.advanced_tooltips.help",
            "debug.creative_spectator.help",
            "debug.pause_focus.help",
            "debug.help.help",
            "debug.reload_resourcepacks.help",
        ] {
            let line = self.locale.translate_key(key).to_owned();
            self.worldRenderer.printDebugMessage(
                line,
                self.currentPlayerTicks(),
                Self::debugChatWidth(gameSettings),
            );
        }
    }

    fn sendClientSettings(&self, settings: &GameSettings) -> Result<(), String> {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else { return Ok(()); };
        let packet = CPacketClientSettings::new(
            settings.language.clone(),
            settings.renderDistanceChunks,
            settings.chatVisibility,
            settings.chatColours,
            settings.modelPartFlags,
            settings.mainHand,
        ).writePacketData().map_err(|error| error.to_string())?;
        connection.sendPlayPackets(vec![packet])
    }

    fn toggleCreativeSpectator(&self) -> Result<Option<bool>, String> {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
            return Ok(None);
        };
        let (permissionLevel, gameType) = connection.getSharedPlayState().withRead(|state| {
            (
                state.thePlayer.as_ref().map_or(0, |player| player.permissionLevel),
                state.gameType,
            )
        });
        if permissionLevel < 2 { return Ok(Some(false)); }
        let command = match gameType {
            GameType::Creative => "/gamemode spectator",
            GameType::Spectator => "/gamemode creative",
            _ => return Ok(None),
        };
        let packet = CPacketChatMessage::new(command)
            .writePacketData()
            .map_err(|error| error.to_string())?;
        connection.sendPlayPackets(vec![packet])?;
        Ok(Some(true))
    }

    fn reloadResources(&mut self, minecraft: &Minecraft) -> Result<(), String> {
        let language = minecraft.gameSettings.language.as_str();
        let languageCodes = if language.eq_ignore_ascii_case("en_us") {
            vec!["en_us"]
        } else {
            vec!["en_us", language]
        };
        let locale = Locale::load(&minecraft.resourceManager, &languageCodes, &["minecraft"]);
        self.languageManager = LanguageManager::new(minecraft.gameSettings.language.clone());
        self.languageManager.parseLanguageMetadata(&minecraft.resourceManager.read_pack_metadatas("pack"));
        let unicode = minecraft.gameSettings.forceUnicodeFont || locale.is_unicode();
        let mut fontRenderer = FontRenderer::load(
            &minecraft.resourceManager,
            ResourceLocation::parse("textures/font/ascii.png"),
            unicode,
            minecraft.gameSettings.anaglyph,
            minecraft.gameSettings.ofCustomFonts,
        ).map_err(|error| error.to_string())?;
        fontRenderer.set_bidi_flag(self.languageManager.isCurrentLanguageBidirectional());
        self.locale = locale.clone();
        self.fontRendererObj = fontRenderer.clone();
        self.guiRenderer.setResourceManager(minecraft.resourceManager.clone());
        self.worldRenderer.reloadResources(
            minecraft.resourceManager.clone(),
            fontRenderer,
            locale,
        );
        self.soundHandler.setResourceManager(minecraft.resourceManager.clone());
        self.musicTicker = MusicTicker::new();
        self.elytraSounds.clear();
        Ok(())
    }

    fn isChatOpen(&self) -> bool { self.guiChat.is_some() }

    fn openChat(&mut self, defaultText: &str) -> bool {
        if !self.isWorld() || self.isInventoryOpen() || self.guiChat.is_some() { return false; }
        let sentCount = self.worldRenderer.sentChatMessages().len();
        self.guiChat = Some(GuiChat::new(
            defaultText,
            self.scaledResolution.scaled_width(),
            self.scaledResolution.scaled_height(),
            sentCount,
        ));
        self.clearMovementKeys();
        true
    }

    fn closeChat(&mut self) -> bool {
        if self.guiChat.take().is_none() { return false; }
        self.worldRenderer.resetChatScroll();
        true
    }

    fn chatLineCount(chatHeightFocused: f32) -> i32 {
        (crate::net::minecraft::client::gui::GuiNewChat::GuiNewChat::calculateChatboxHeight(
            chatHeightFocused.clamp(0.0, 1.0),
        ) / 9).max(1)
    }

    fn scrollChat(&mut self, amount: i32, chatHeightFocused: f32) -> bool {
        if !self.isChatOpen() || amount == 0 { return false; }
        self.worldRenderer.scrollChat(amount, Self::chatLineCount(chatHeightFocused));
        true
    }

    fn chatTypedText(&mut self, text: &str) -> bool {
        self.guiChat.as_mut().is_some_and(|chat| chat.typedText(text, &self.fontRendererObj))
    }

    fn chatKeyPressed(
        &mut self,
        key: KeyCode,
        modifiers: ModifiersState,
        chatHeightFocused: f32,
        chatWidth: f32,
        chatScale: f32,
    ) -> Result<(bool, bool), String> {
        if self.guiChat.is_none() { return Ok((false, false)); }
        let textModifiers = GuiTextFieldModifiers {
            control: modifiers.control_key(),
            shift: modifiers.shift_key(),
        };
        match key {
            KeyCode::Tab => {
                let (request, completionLine) = if let Some(chat) = self.guiChat.as_mut() {
                    let request = chat.complete(&self.fontRendererObj);
                    let completionLine = chat.takeCompletionDisplayLine();
                    (request, completionLine)
                } else {
                    (None, None)
                };
                if let Some(request) = request {
                    let packet = request.writePacketData().map_err(|error| error.to_string())?;
                    let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
                        return Ok((true, false));
                    };
                    connection.sendPlayPackets(vec![packet])?;
                }
                if let Some(line) = completionLine {
                    let wrapWidth = ((GuiNewChat::calculateChatboxWidth(chatWidth.clamp(0.0, 1.0)) as f32)
                        / chatScale.max(0.01)).floor() as i32;
                    let ticks = match &self.currentScreen {
                        ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState().withRead(|state| {
                            state.thePlayer.as_ref().map_or(0, |player| player.entity.ticksExisted)
                        }),
                        _ => 0,
                    };
                    self.worldRenderer.showChatCompletionCandidates(line, ticks, wrapWidth);
                }
                return Ok((true, false));
            }
            KeyCode::Escape => { self.closeChat(); return Ok((true, true)); }
            KeyCode::Enter | KeyCode::NumpadEnter => {
                let message = self.guiChat.as_ref().map_or_else(String::new, GuiChat::getTrimmedText);
                if !message.is_empty() {
                    let packet = CPacketChatMessage::new(message.clone()).writePacketData()
                        .map_err(|error| error.to_string())?;
                    let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
                        return Ok((true, false));
                    };
                    connection.sendPlayPackets(vec![packet])?;
                    self.worldRenderer.addSentChatMessage(message);
                }
                self.closeChat();
                return Ok((true, true));
            }
            KeyCode::ArrowUp | KeyCode::ArrowDown => {
                let history = self.worldRenderer.sentChatMessages();
                if let Some(chat) = self.guiChat.as_mut() {
                    chat.getSentHistory(if key == KeyCode::ArrowUp { -1 } else { 1 }, &history);
                }
                return Ok((true, false));
            }
            KeyCode::PageUp | KeyCode::PageDown => {
                let amount = Self::chatLineCount(chatHeightFocused) - 1;
                self.worldRenderer.scrollChat(if key == KeyCode::PageUp { amount } else { -amount }, Self::chatLineCount(chatHeightFocused));
                return Ok((true, false));
            }
            KeyCode::KeyA if modifiers.control_key() => {
                if let Some(chat) = self.guiChat.as_mut() { chat.selectAll(&self.fontRendererObj); }
                return Ok((true, false));
            }
            _ => {}
        }
        let fieldKey = match key {
            KeyCode::Backspace => Some(GuiTextFieldKey::Backspace),
            KeyCode::Delete => Some(GuiTextFieldKey::Delete),
            KeyCode::ArrowLeft => Some(GuiTextFieldKey::Left),
            KeyCode::ArrowRight => Some(GuiTextFieldKey::Right),
            KeyCode::Home => Some(GuiTextFieldKey::Home),
            KeyCode::End => Some(GuiTextFieldKey::End),
            _ => None,
        };
        Ok((fieldKey.is_some_and(|fieldKey| {
            self.guiChat.as_mut().is_some_and(|chat| chat.keyPressed(fieldKey, textModifiers, &self.fontRendererObj))
        }), false))
    }

    fn worldHotbarBinding(&mut self, binding: KeyBindingId, modifiers: ModifiersState) -> Result<bool, String> {
        let slot = match binding {
            KeyBindingId::Hotbar1 => Some(0),
            KeyBindingId::Hotbar2 => Some(1),
            KeyBindingId::Hotbar3 => Some(2),
            KeyBindingId::Hotbar4 => Some(3),
            KeyBindingId::Hotbar5 => Some(4),
            KeyBindingId::Hotbar6 => Some(5),
            KeyBindingId::Hotbar7 => Some(6),
            KeyBindingId::Hotbar8 => Some(7),
            KeyBindingId::Hotbar9 => Some(8),
            _ => None,
        };
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
            return Ok(false);
        };
        let shared = connection.getSharedPlayState();
        let gameType = shared.withRead(|state| state.gameType);
        if let Some(slot) = slot {
            // The vanilla spectator branch delegates number keys to
            // GuiSpectator. Until that concrete GUI exists, consume the key
            // without mutating InventoryPlayer.
            return Ok(if gameType == GameType::Spectator {
                true
            } else {
                shared.setCurrentHotbarSlot(slot)
            });
        }

        let action = match binding {
            KeyBindingId::Drop => Some(if modifiers.control_key() {
                DiggingAction::DropAllItems
            } else {
                DiggingAction::DropItem
            }),
            KeyBindingId::SwapHands => Some(DiggingAction::SwapHeldItems),
            _ => None,
        };
        let Some(action) = action else { return Ok(false); };
        if gameType == GameType::Spectator {
            return Ok(true);
        }
        // Minecraft.processKeyBinds sends SWAP_HELD_ITEMS as one digging
        // packet. It does not prepend CPacketHeldItemChange; the server swaps
        // the two inventory slots and synchronizes them back with SetSlot.
        let mut packets = Vec::with_capacity(2);
        if action != DiggingAction::SwapHeldItems {
            if let Some(slot) = shared.currentHotbarSlot() {
                if let Some(packet) = self.playerController.syncCurrentPlayItem(slot) {
                    packets.push(packet);
                }
            }
        }
        packets.push(CPacketPlayerDigging::new(
            action,
            crate::net::minecraft::util::math::BlockPos::BlockPos::new(0, 0, 0),
            crate::net::minecraft::util::EnumFacing::EnumFacing::Down,
        ).writePacketData());
        connection.sendPlayPackets(packets)?;
        if action == DiggingAction::SwapHeldItems {
            log::info!("sent CPacketPlayerDigging SWAP_HELD_ITEMS (protocol 340 action 6)");
            // Server-authoritative stacks still arrive through SetSlot, but
            // ItemRenderer must lower both hands while that swap is applied.
            self.itemRenderer.resetEquippedProgressMainHand();
            self.itemRenderer.resetEquippedProgressOffHand();
        }
        Ok(true)
    }

    fn inventoryUiState(&self) -> Option<(ItemStack, Vec<ItemStack>, GameType)> {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else { return None; };
        let playerInventory = self.inventoryOpen || self.creativeInventoryOpen;
        connection.getSharedPlayState().withRead(|state| {
            let player = state.thePlayer.as_ref()?;
            let slots = if playerInventory {
                player.inventoryContainer.slots().to_vec()
            } else {
                player.openContainer.as_ref()?.slots().to_vec()
            };
            Some((player.inventory.getItemStack().clone(), slots, state.gameType))
        })
    }

    fn creativeInventoryUiState(&self) -> Option<(ItemStack, Vec<ItemStack>)> {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else { return None; };
        connection.getSharedPlayState().creativePlayerContainerSnapshot()
    }

    fn sendCreativePlayerSlotClick(
        &self,
        playerSlot: i32,
        mouseButton: i32,
        clickType: ClickType,
    ) -> Result<bool, String> {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
            return Ok(false);
        };
        let inventoryTab = self.guiCreative.selectedTabIndex
            == crate::net::minecraft::creativetab::CreativeTabs::INVENTORY.tabIndex;
        let shared = connection.getSharedPlayState();
        let Some(result) = shared
            .clickCreativePlayerInventorySlot(
                playerSlot,
                mouseButton,
                clickType,
                inventoryTab,
            )
            .map_err(|error| error.to_string())?
        else {
            return Ok(false);
        };

        let changed = result.beforeCursor != result.afterCursor
            || result.beforeSlots != result.afterSlots;
        let mut packets = Vec::new();

        if inventoryTab {
            // The inventory tab installs CreativeCrafting as a listener on the
            // real ContainerPlayer. detectAndSendChanges therefore reports
            // each changed real slot using its unwrapped ContainerPlayer ID.
            if clickType == ClickType::Throw && playerSlot >= 0 && !result.originalSlotStack.isEmpty() {
                let mut dropped = result.originalSlotStack.copy();
                dropped.setCount(if mouseButton == 0 { 1 } else { dropped.getMaxStackSize() });
                packets.push(
                    CPacketCreativeInventoryAction::new(-1, &dropped)
                        .writePacketData()
                        .map_err(|error| error.to_string())?,
                );
                let after = result.afterSlots
                    .get(playerSlot as usize)
                    .cloned()
                    .unwrap_or(ItemStack::EMPTY);
                packets.push(
                    CPacketCreativeInventoryAction::new(playerSlot, &after)
                        .writePacketData()
                        .map_err(|error| error.to_string())?,
                );
            } else {
                for slot in 0..=45 {
                    let before = result.beforeSlots
                        .get(slot as usize)
                        .cloned()
                        .unwrap_or(ItemStack::EMPTY);
                    let after = result.afterSlots
                        .get(slot as usize)
                        .cloned()
                        .unwrap_or(ItemStack::EMPTY);
                    if before != after {
                        packets.push(
                            CPacketCreativeInventoryAction::new(slot, &after)
                                .writePacketData()
                                .map_err(|error| error.to_string())?,
                        );
                    }
                }
            }
        } else if result.quickCraftFinished {
            // MCP sends every hotbar slot after QUICK_CRAFT event 2, even if a
            // particular slot retained the same stack.
            for slot in 36..=44 {
                let after = result.afterSlots
                    .get(slot as usize)
                    .cloned()
                    .unwrap_or(ItemStack::EMPTY);
                packets.push(
                    CPacketCreativeInventoryAction::new(slot, &after)
                        .writePacketData()
                        .map_err(|error| error.to_string())?,
                );
            }
        } else if playerSlot >= 0 {
            // ContainerCreative's visible slot ID 45..53 maps onto protocol
            // player slots 36..44. Vanilla sends the clicked slot after every
            // non-final event, plus the swapped hotbar counterpart or drop.
            let after = result.afterSlots
                .get(playerSlot as usize)
                .cloned()
                .unwrap_or(ItemStack::EMPTY);
            packets.push(
                CPacketCreativeInventoryAction::new(playerSlot, &after)
                    .writePacketData()
                    .map_err(|error| error.to_string())?,
            );
            if clickType == ClickType::Swap {
                let counterpart = 36 + mouseButton;
                packets.push(
                    CPacketCreativeInventoryAction::new(counterpart, &result.originalSlotStack)
                        .writePacketData()
                        .map_err(|error| error.to_string())?,
                );
            } else if clickType == ClickType::Throw && !result.originalSlotStack.isEmpty() {
                let mut dropped = result.originalSlotStack.copy();
                dropped.setCount(if mouseButton == 0 { 1 } else { dropped.getMaxStackSize() });
                packets.push(
                    CPacketCreativeInventoryAction::new(-1, &dropped)
                        .writePacketData()
                        .map_err(|error| error.to_string())?,
                );
            }
        }

        if !packets.is_empty() {
            connection.sendPlayPackets(packets)?;
        }
        Ok(changed || playerSlot >= 0 || result.quickCraftFinished)
    }

    fn sendWorldPlayPackets(&self, packets: Vec<crate::net::minecraft::network::Packet::RawPacket>) -> Result<(), String> {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
            return Err("creative inventory is not attached to a world connection".to_owned());
        };
        connection.sendPlayPackets(packets)
    }

    fn creativeHandleClick(
        &mut self,
        slotId: i32,
        mouseButton: i32,
        clickType: ClickType,
        modifiers: ModifiersState,
    ) -> Result<bool, String> {
        // Clone the Arc-backed play state before mutating GUI fields. Keeping a
        // borrow of ActiveGuiScreen::World alive here would incorrectly couple
        // the network owner to GuiContainerCreative's local state machine.
        let shared = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState(),
            _ => return Ok(false),
        };
        self.guiCreative.clearSearch = true;
        if slotId == -999 {
            let packet = shared.dropCreativeCursor(mouseButton).map_err(|error| error.to_string())?;
            if let Some(packet) = packet {
                self.sendWorldPlayPackets(vec![packet])?;
                return Ok(true);
            }
            return Ok(false);
        }
        let Some(kind) = self.guiCreative.slotKind(slotId) else { return Ok(false); };
        match kind {
            CreativeSlotKind::Catalog { .. } => {
                let playerSlots = self.creativeInventoryUiState().map_or_else(Vec::new, |(_, slots)| slots);
                let stack = self.guiCreative.stackForSlot(slotId, &playerSlots);
                let packets = shared
                    .clickCreativeCatalogStack(&stack, mouseButton, clickType)
                    .map_err(|error| error.to_string())?;
                if !packets.is_empty() { self.sendWorldPlayPackets(packets)?; }
                Ok(!stack.isEmpty() || clickType == ClickType::Pickup)
            }
            CreativeSlotKind::Hotbar { playerContainerSlot }
            | CreativeSlotKind::Player { playerContainerSlot } => {
                self.sendCreativePlayerSlotClick(playerContainerSlot, mouseButton, clickType)
            }
            CreativeSlotKind::Destroy => {
                if clickType == ClickType::QuickMove || modifiers.shift_key() {
                    let packets = shared.clearCreativePlayerContainer().map_err(|error| error.to_string())?;
                    if !packets.is_empty() { self.sendWorldPlayPackets(packets)?; }
                    Ok(true)
                } else {
                    Ok(shared.clearCreativeCursor())
                }
            }
        }
    }

    fn creativeInventoryMouseClicked(
        &mut self,
        framebufferWidth: u32,
        framebufferHeight: u32,
        button: MouseButton,
        modifiers: ModifiersState,
    ) -> Result<bool, String> {
        let mouseButton = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            _ => return Ok(false),
        };
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        if mouseButton == 0 && self.guiCreative.updateScrollbarDrag(mouseX, mouseY, true) {
            return Ok(true);
        }
        if mouseButton == 0 && self.guiCreative.tabAt(mouseX, mouseY).is_some() {
            return Ok(true);
        }
        if self.guiCreative.searchMouseClicked(mouseX, mouseY, mouseButton, &self.fontRendererObj) {
            return Ok(true);
        }
        let hoveredSlot = self.guiCreative.container.slotAt(mouseX, mouseY);
        let slotId = self.guiCreative.container.protocolSlotAt(mouseX, mouseY);
        let now = Instant::now();
        let clickIdentity = hoveredSlot.unwrap_or(-1);
        let doubleClick = self.lastInventoryClick.is_some_and(|(last, lastSlot, lastButton)| {
            lastSlot == clickIdentity
                && lastButton == mouseButton
                && now.duration_since(last) < Duration::from_millis(250)
        });
        self.lastInventoryClick = Some((now, clickIdentity, mouseButton));
        self.guiCreative.container.doubleClick = doubleClick;
        self.guiCreative.container.ignoreMouseUp = false;
        if slotId == -1 { return Ok(false); }
        let Some((cursor, _)) = self.creativeInventoryUiState() else { return Ok(false); };
        if !self.guiCreative.container.dragSplitting {
            if cursor.isEmpty() {
                let clickType = if mouseButton == 2 {
                    ClickType::Clone
                } else if modifiers.shift_key() && slotId >= 0 {
                    ClickType::QuickMove
                } else if slotId == -999 {
                    ClickType::Throw
                } else {
                    ClickType::Pickup
                };
                let changed = self.creativeHandleClick(slotId, mouseButton, clickType, modifiers)?;
                self.guiCreative.container.ignoreMouseUp = true;
                return Ok(changed);
            }
            return Ok(self.guiCreative.container.beginDragSplitting(mouseButton));
        }
        Ok(false)
    }

    fn creativeInventoryMouseDragged(
        &mut self,
        framebufferWidth: u32,
        framebufferHeight: u32,
    ) -> bool {
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        let leftButtonDown = self.guiCreative.wasClicking;
        if self.guiCreative.updateScrollbarDrag(mouseX, mouseY, leftButtonDown) {
            return true;
        }
        if !self.guiCreative.container.dragSplitting { return false; }
        let Some(slotId) = self.guiCreative.container.slotAt(mouseX, mouseY) else { return false; };
        let Some((cursor, playerSlots)) = self.creativeInventoryUiState() else { return false; };
        let displaySlots = self.guiCreative.displayStacks(&playerSlots);
        let slotKind = self.guiCreative.slotKind(slotId);
        self.guiCreative.container.tryAddDragSplittingSlotWithRules(
            slotId,
            &cursor,
            &displaySlots,
            move |_slot, _stack| matches!(slotKind, Some(CreativeSlotKind::Hotbar { .. }) | Some(CreativeSlotKind::Player { .. })),
            |_slot, stack| stack.getMaxStackSize(),
        )
    }

    fn creativeInventoryMouseReleased(
        &mut self,
        framebufferWidth: u32,
        framebufferHeight: u32,
        button: MouseButton,
        modifiers: ModifiersState,
    ) -> Result<bool, String> {
        let mouseButton = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            _ => return Ok(false),
        };
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        if mouseButton == 0 {
            self.guiCreative.updateScrollbarDrag(mouseX, mouseY, false);
            if let Some(tabIndex) = self.guiCreative.tabAt(mouseX, mouseY) {
                let changed = self.guiCreative.setCurrentCreativeTab(tabIndex);
                self.lastInventoryClick = None;
                return Ok(changed);
            }
        }
        let hoveredSlot = self.guiCreative.container.slotAt(mouseX, mouseY);
        let slotId = self.guiCreative.container.protocolSlotAt(mouseX, mouseY);
        let doubleClick = self.guiCreative.container.doubleClick;
        if doubleClick && mouseButton == 0 {
            if let Some(slot) = hoveredSlot {
                if let Some(CreativeSlotKind::Player { playerContainerSlot }
                    | CreativeSlotKind::Hotbar { playerContainerSlot }) = self.guiCreative.slotKind(slot)
                {
                    let changed = self.sendCreativePlayerSlotClick(
                        playerContainerSlot,
                        0,
                        if modifiers.shift_key() { ClickType::QuickMove } else { ClickType::PickupAll },
                    )?;
                    self.guiCreative.container.doubleClick = false;
                    self.guiCreative.container.cancelDragSplitting();
                    self.lastInventoryClick = None;
                    return Ok(changed);
                }
            }
        }
        let dragSplitting = self.guiCreative.container.dragSplitting;
        let dragButton = self.guiCreative.container.dragSplittingButton;
        let ignoreMouseUp = self.guiCreative.container.ignoreMouseUp;
        let selected = self.guiCreative.container.dragSplittingSlots.iter().copied().collect::<Vec<_>>();
        let mode = self.guiCreative.container.dragSplittingLimit;
        if dragSplitting && dragButton != mouseButton {
            self.guiCreative.container.cancelDragSplitting();
            self.guiCreative.container.ignoreMouseUp = true;
            return Ok(true);
        }
        if ignoreMouseUp {
            self.guiCreative.container.ignoreMouseUp = false;
            self.guiCreative.container.cancelDragSplitting();
            return Ok(false);
        }
        let mut changed = false;
        if dragSplitting && !selected.is_empty() {
            let mut mapped = Vec::new();
            for slot in selected {
                match self.guiCreative.slotKind(slot) {
                    Some(CreativeSlotKind::Hotbar { playerContainerSlot })
                    | Some(CreativeSlotKind::Player { playerContainerSlot }) => mapped.push(playerContainerSlot),
                    _ => {}
                }
            }
            if !mapped.is_empty() {
                changed |= self.sendCreativePlayerSlotClick(
                    -999,
                    Container::getQuickcraftMask(0, mode),
                    ClickType::QuickCraft,
                )?;
                for slot in mapped {
                    changed |= self.sendCreativePlayerSlotClick(
                        slot,
                        Container::getQuickcraftMask(1, mode),
                        ClickType::QuickCraft,
                    )?;
                }
                changed |= self.sendCreativePlayerSlotClick(
                    -999,
                    Container::getQuickcraftMask(2, mode),
                    ClickType::QuickCraft,
                )?;
            }
        } else if let Some((cursor, _)) = self.creativeInventoryUiState() {
            if !cursor.isEmpty() && slotId != -1 {
                let clickType = if mouseButton == 2 {
                    ClickType::Clone
                } else if modifiers.shift_key() && slotId >= 0 {
                    ClickType::QuickMove
                } else {
                    ClickType::Pickup
                };
                changed |= self.creativeHandleClick(slotId, mouseButton, clickType, modifiers)?;
            }
        }
        if self.creativeInventoryUiState().is_some_and(|(cursor, _)| cursor.isEmpty()) {
            self.lastInventoryClick = None;
        }
        self.guiCreative.container.cancelDragSplitting();
        Ok(changed)
    }

    fn creativeInventoryKeyPressed(
        &mut self,
        framebufferWidth: u32,
        framebufferHeight: u32,
        key: KeyCode,
        binding: Option<KeyBindingId>,
        modifiers: ModifiersState,
    ) -> Result<bool, String> {
        if self.guiCreative.selectedTabIndex != crate::net::minecraft::creativetab::CreativeTabs::SEARCH.tabIndex
            && binding == Some(KeyBindingId::Chat)
        {
            return Ok(self.guiCreative.setCurrentCreativeTab(
                crate::net::minecraft::creativetab::CreativeTabs::SEARCH.tabIndex,
            ));
        }
        let textModifiers = GuiTextFieldModifiers {
            control: modifiers.control_key(),
            shift: modifiers.shift_key(),
        };
        let textKey = match key {
            KeyCode::Backspace => Some(GuiTextFieldKey::Backspace),
            KeyCode::Delete => Some(GuiTextFieldKey::Delete),
            KeyCode::ArrowLeft => Some(GuiTextFieldKey::Left),
            KeyCode::ArrowRight => Some(GuiTextFieldKey::Right),
            KeyCode::Home => Some(GuiTextFieldKey::Home),
            KeyCode::End => Some(GuiTextFieldKey::End),
            KeyCode::KeyA if modifiers.control_key() => {
                self.guiCreative.searchField.selectAll(&self.fontRendererObj);
                return Ok(true);
            }
            _ => None,
        };
        if let Some(textKey) = textKey {
            if self.guiCreative.searchKeyPressed(textKey, textModifiers, &self.fontRendererObj) {
                return Ok(true);
            }
        }
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        let Some(slotId) = self.guiCreative.container.slotAt(mouseX, mouseY) else { return Ok(false); };
        let (mouseButton, clickType) = match binding {
            Some(KeyBindingId::Hotbar1) => (0, ClickType::Swap),
            Some(KeyBindingId::Hotbar2) => (1, ClickType::Swap),
            Some(KeyBindingId::Hotbar3) => (2, ClickType::Swap),
            Some(KeyBindingId::Hotbar4) => (3, ClickType::Swap),
            Some(KeyBindingId::Hotbar5) => (4, ClickType::Swap),
            Some(KeyBindingId::Hotbar6) => (5, ClickType::Swap),
            Some(KeyBindingId::Hotbar7) => (6, ClickType::Swap),
            Some(KeyBindingId::Hotbar8) => (7, ClickType::Swap),
            Some(KeyBindingId::Hotbar9) => (8, ClickType::Swap),
            Some(KeyBindingId::PickBlock) => (2, ClickType::Clone),
            Some(KeyBindingId::Drop) => (if modifiers.control_key() { 1 } else { 0 }, ClickType::Throw),
            _ => return Ok(false),
        };
        self.creativeHandleClick(slotId, mouseButton, clickType, modifiers)
    }

    fn creativeInventoryTypedText(&mut self, text: &str) -> bool {
        self.guiCreative.searchTypedText(text, &self.fontRendererObj)
    }

    fn creativeInventoryScroll(&mut self, wheelDelta: i32) -> bool {
        self.guiCreative.handleMouseWheel(wheelDelta)
    }

    fn sendPlayerInventoryClick(
        &self,
        slotId: i32,
        mouseButton: i32,
        clickType: ClickType,
    ) -> Result<bool, String> {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
            return Ok(false);
        };
        let shared = connection.getSharedPlayState();
        let packet = if self.inventoryOpen {
            shared.clickPlayerInventorySlot(slotId, mouseButton, clickType)
        } else {
            shared.clickOpenContainerSlot(slotId, mouseButton, clickType)
        }.map_err(|error| error.to_string())?;
        if let Some(packet) = packet {
            connection.sendPlayPackets(vec![packet])?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn activeContainerInventoryGroup(&self, slotId: i32) -> i32 {
        if self.inventoryOpen {
            player_container_inventory_group(slotId)
        } else {
            let lower = if let Some(gui) = self.guiHorse.as_ref() {
                gui.spec.lowerSlotCount() as i32
            } else if self.guiShulkerBox.is_some() {
                27
            } else if let Some(gui) = self.guiChest.as_ref() {
                gui.inventoryRows * 9
            } else {
                self.guiDedicated.as_ref().map_or(0, |gui| match gui.kind() {
                    ContainerWindowKind::Workbench => 10,
                    ContainerWindowKind::Furnace | ContainerWindowKind::Repair | ContainerWindowKind::Merchant => 3,
                    ContainerWindowKind::Enchantment => 2,
                    ContainerWindowKind::Hopper | ContainerWindowKind::BrewingStand => 5,
                    ContainerWindowKind::Dispenser | ContainerWindowKind::Dropper => 9,
                    ContainerWindowKind::Beacon => 1,
                })
            };
            if slotId < lower { 0 } else { 1 }
        }
    }

    /// Desktop branch of MCP `GuiContainer.mouseClicked`. Empty-cursor clicks
    /// execute immediately and suppress mouse-up. A non-empty cursor starts the
    /// three-phase QUICK_CRAFT gesture and is committed by `mouseReleased`.
    fn inventoryMouseClicked(
        &mut self,
        framebufferWidth: u32,
        framebufferHeight: u32,
        button: MouseButton,
        modifiers: ModifiersState,
    ) -> Result<bool, String> {
        self.syncOpenContainerGui();
        if !self.isInventoryOpen() { return Ok(false); }
        if self.creativeInventoryOpen {
            return self.creativeInventoryMouseClicked(framebufferWidth, framebufferHeight, button, modifiers);
        }
        let mouseButton = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            _ => return Ok(false),
        };
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        if let Some(consumed) = self.recipeBookMouseClicked(
            mouseX, mouseY, mouseButton, modifiers.shift_key(),
        )? {
            return Ok(consumed);
        }
        let repairFieldClicked = self
            .guiDedicated
            .as_mut()
            .and_then(DedicatedContainerGui::repairNameFieldMut)
            .is_some_and(|field| {
                field.mouseClicked(mouseX, mouseY, mouseButton, &self.fontRendererObj)
            });
        if repairFieldClicked {
            return Ok(true);
        }

        let beaconAction = if mouseButton == 0 {
            let beaconState = match &self.currentScreen {
                ActiveGuiScreen::World { connection, .. } => connection
                    .getSharedPlayState()
                    .withRead(|state| {
                        let container = state.thePlayer.as_ref()?.openContainer.as_ref()?;
                        if container.windowKind() != Some(ContainerWindowKind::Beacon) {
                            return None;
                        }
                        Some((
                            container.properties().first().copied().unwrap_or(0),
                            container.properties().get(1).copied().unwrap_or(0),
                            GuiBeacon::confirmEnabled(
                                container.getSlot(0),
                                container.properties().get(1).copied().unwrap_or(0),
                            ),
                        ))
                    }),
                _ => None,
            };
            match (self.guiDedicated.as_ref(), beaconState) {
                (Some(DedicatedContainerGui::Beacon(gui)), Some((levels, primary, confirmEnabled))) => {
                    if confirmEnabled && gui.confirmAt(mouseX, mouseY) {
                        Some(BeaconGuiAction::Confirm)
                    } else if gui.cancelAt(mouseX, mouseY) {
                        Some(BeaconGuiAction::Cancel)
                    } else {
                        gui.powerButtonAt(mouseX, mouseY, levels, primary).map(|button| {
                            BeaconGuiAction::SelectPower {
                                tier: button.tier,
                                effectId: button.effectId,
                            }
                        })
                    }
                }
                _ => None,
            }
        } else {
            None
        };
        if let Some(action) = beaconAction {
            match action {
                BeaconGuiAction::SelectPower { tier, effectId } => {
                    let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
                        return Ok(false);
                    };
                    let updated = connection.getSharedPlayState().withWrite(|state| {
                        let Some(container) = state
                            .thePlayer
                            .as_mut()
                            .and_then(|player| player.openContainer.as_mut())
                        else {
                            return false;
                        };
                        if container.windowKind() != Some(ContainerWindowKind::Beacon) {
                            return false;
                        }
                        let property = if tier < 3 { 1 } else { 2 };
                        if container.properties().get(property as usize).copied() == Some(effectId) {
                            return true;
                        }
                        container.updateProgressBar(property, effectId).is_ok()
                    });
                    return Ok(updated);
                }
                BeaconGuiAction::Confirm => {
                    let permitted = match &self.currentScreen {
                        ActiveGuiScreen::World { connection, .. } => connection
                            .getSharedPlayState()
                            .withRead(|state| {
                                let container = state.thePlayer.as_ref()?.openContainer.as_ref()?;
                                if container.windowKind() != Some(ContainerWindowKind::Beacon) {
                                    return None;
                                }
                                let primary = container.properties().get(1).copied().unwrap_or(0);
                                let secondary = container.properties().get(2).copied().unwrap_or(0);
                                GuiBeacon::confirmEnabled(container.getSlot(0), primary)
                                    .then_some((primary, secondary))
                            }),
                        _ => None,
                    };
                    let Some((primary, secondary)) = permitted else {
                        return Ok(true);
                    };
                    let mut data = Vec::with_capacity(8);
                    write_i32_be(primary, &mut data);
                    write_i32_be(secondary, &mut data);
                    let packet = CPacketCustomPayload::new("MC|Beacon", data)
                        .and_then(|payload| payload.writePacketData())
                        .map_err(|error| error.to_string())?;
                    if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
                        connection.sendPlayPackets(vec![packet])?;
                    }
                    self.closeInventory()?;
                    return Ok(true);
                }
                BeaconGuiAction::Cancel => {
                    self.closeInventory()?;
                    return Ok(true);
                }
            }
        }

        if mouseButton == 0 {
            let merchantState = match &self.currentScreen {
                ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState().withRead(|state| {
                    let container=state.thePlayer.as_ref()?.openContainer.as_ref()?;
                    if container.windowKind()!=Some(ContainerWindowKind::Merchant){return None;}
                    Some((container.windowId(),container.merchantRecipes().cloned(),container.merchantRecipeIndex().unwrap_or(0)))
                }),
                _=>None,
            };
            if let (Some(DedicatedContainerGui::Merchant(gui)),Some((windowId,recipes,current)))=(self.guiDedicated.as_mut(),merchantState){
                gui.setSelectedMerchantRecipe(current,recipes.as_ref());
                if let Some(delta)=gui.buttonDeltaAt(mouseX,mouseY,recipes.as_ref()){
                    let next=(gui.selectedMerchantRecipe()+delta).max(0);
                    gui.setSelectedMerchantRecipe(next,recipes.as_ref());
                    let selected=gui.selectedMerchantRecipe();
                    if let ActiveGuiScreen::World{connection,..}=&self.currentScreen{
                        connection.getSharedPlayState().withWrite(|state|{
                            if let Some(container)=state.thePlayer.as_mut().and_then(|p|p.openContainer.as_mut()){
                                if container.windowId()==windowId{container.setMerchantRecipeIndex(selected);}
                            }
                        });
                        let mut data=Vec::with_capacity(4); write_i32_be(selected,&mut data);
                        let packet=CPacketCustomPayload::new("MC|TrSel",data).and_then(|p|p.writePacketData()).map_err(|e|e.to_string())?;
                        connection.sendPlayPackets(vec![packet])?;
                    }
                    return Ok(true);
                }
            }
        }

        let enchantOption = if mouseButton == 0 {
            match self.guiDedicated.as_ref() {
                Some(DedicatedContainerGui::Enchantment(gui)) => gui.optionAt(mouseX, mouseY),
                _ => None,
            }
        } else {
            None
        };
        if let Some(option) = enchantOption {
            let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
                return Ok(false);
            };
            let windowId = connection.getSharedPlayState().withRead(|state| {
                let player = state.thePlayer.as_ref()?;
                let container = player.openContainer.as_ref()?;
                if container.windowKind() != Some(ContainerWindowKind::Enchantment) {
                    return None;
                }
                let level = container.properties().get(option as usize).copied().unwrap_or(0);
                let lapis = container.getSlot(1).map_or(0, ItemStack::getCount);
                let itemPresent = container.getSlot(0).is_some_and(|stack| !stack.isEmpty());
                let permitted = itemPresent
                    && level > 0
                    && (state.gameType == GameType::Creative
                        || (lapis >= option + 1 && player.experienceLevel >= level));
                permitted.then(|| container.windowId())
            });
            if let Some(windowId) = windowId {
                let packet = self.playerController.sendEnchantPacket(windowId, option);
                connection.sendPlayPackets(vec![packet])?;
                return Ok(true);
            }
        }
        let hoveredSlot = self.activeSlotAt(mouseX, mouseY);
        let slotId = self.activeProtocolSlotAt(mouseX, mouseY);
        let now = Instant::now();
        let clickIdentity = hoveredSlot.unwrap_or(-1);
        let doubleClick = self.lastInventoryClick.is_some_and(|(last, lastSlot, lastButton)| {
            lastSlot == clickIdentity
                && lastButton == mouseButton
                && now.duration_since(last) < Duration::from_millis(250)
        });
        self.lastInventoryClick = Some((now, clickIdentity, mouseButton));
        if let Some(container) = self.activeGuiContainerMut() {
            container.doubleClick = doubleClick;
            container.ignoreMouseUp = false;
        }

        if slotId == -1 {
            return Ok(false);
        }
        let Some((cursor, slots, _gameType)) = self.inventoryUiState() else {
            return Ok(false);
        };
        let dragSplitting = self.activeGuiContainer().is_some_and(|container| container.dragSplitting);
        if !dragSplitting {
            if cursor.isEmpty() {
                let clickType = if mouseButton == 2 {
                    ClickType::Clone
                } else if modifiers.shift_key() && slotId >= 0 {
                    if let Some(stack) = slots.get(slotId as usize) {
                        self.inventoryShiftClickedStack = stack.clone();
                    }
                    ClickType::QuickMove
                } else if slotId == -999 {
                    ClickType::Throw
                } else {
                    ClickType::Pickup
                };
                let sent = self.sendPlayerInventoryClick(slotId, mouseButton, clickType)?;
                if sent {
                    if self.inventoryOpen {
                        self.guiInventory.recipeBook.clearGhost();
                    } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
                        gui.recipeBook.clearGhost();
                    }
                }
                if let Some(container) = self.activeGuiContainerMut() {
                    container.ignoreMouseUp = true;
                }
                return Ok(sent);
            }
            return Ok(self.activeGuiContainerMut().is_some_and(|container| {
                container.beginDragSplitting(mouseButton)
            }));
        }
        Ok(false)
    }

    /// MCP `GuiContainer.mouseClickMove` desktop QUICK_CRAFT branch.
    fn inventoryMouseDragged(
        &mut self,
        framebufferWidth: u32,
        framebufferHeight: u32,
    ) -> bool {
        self.syncOpenContainerGui();
        if self.creativeInventoryOpen {
            return self.creativeInventoryMouseDragged(framebufferWidth, framebufferHeight);
        }
        if !self.isInventoryOpen()
            || !self.activeGuiContainer().is_some_and(|container| container.dragSplitting)
        {
            return false;
        }
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        let Some(slotId) = self.activeSlotAt(mouseX, mouseY) else { return false; };
        let Some((cursor, slots, _)) = self.inventoryUiState() else { return false; };
        let playerInventory = self.inventoryOpen;
        if playerInventory {
            self.activeGuiContainerMut().is_some_and(|container| {
                container.tryAddDragSplittingSlot(slotId, &cursor, &slots)
            })
        } else {
            let (accepts, limits) = match &self.currentScreen {
                ActiveGuiScreen::World { connection, .. } => connection
                    .getSharedPlayState()
                    .withRead(|state| {
                        let container = state.thePlayer.as_ref()?.openContainer.as_ref()?;
                        let accepts = (0..container.slotCount() as i32)
                            .map(|candidate| container.isItemValidForSlot(candidate, &cursor))
                            .collect::<Vec<_>>();
                        let limits = (0..container.slotCount() as i32)
                            .map(|candidate| container.slotLimit(candidate, &cursor))
                            .collect::<Vec<_>>();
                        Some((accepts, limits))
                    }),
                _ => None,
            }
            .unwrap_or_else(|| {
                (
                    vec![true; slots.len()],
                    vec![cursor.getMaxStackSize(); slots.len()],
                )
            });
            self.activeGuiContainerMut().is_some_and(|container| {
                container.tryAddDragSplittingSlotWithRules(
                    slotId,
                    &cursor,
                    &slots,
                    |candidate, _stack| {
                        usize::try_from(candidate)
                            .ok()
                            .and_then(|index| accepts.get(index))
                            .copied()
                            .unwrap_or(false)
                    },
                    |candidate, stack| {
                        usize::try_from(candidate)
                            .ok()
                            .and_then(|index| limits.get(index))
                            .copied()
                            .unwrap_or_else(|| stack.getMaxStackSize())
                    },
                )
            })
        }
    }

    /// MCP `GuiContainer.mouseReleased`, including double-click collection,
    /// shift-double-click transfer, and the start/add/end QUICK_CRAFT packet
    /// sequence used by protocol 340.
    fn inventoryMouseReleased(
        &mut self,
        framebufferWidth: u32,
        framebufferHeight: u32,
        button: MouseButton,
        modifiers: ModifiersState,
    ) -> Result<bool, String> {
        self.syncOpenContainerGui();
        if !self.isInventoryOpen() { return Ok(false); }
        if self.creativeInventoryOpen {
            return self.creativeInventoryMouseReleased(framebufferWidth, framebufferHeight, button, modifiers);
        }
        let mouseButton = match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            _ => return Ok(false),
        };
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        let hoveredSlot = self.activeSlotAt(mouseX, mouseY);
        let slotId = self.activeProtocolSlotAt(mouseX, mouseY);
        let mut changed = false;
        let isPlayerInventory = self.inventoryOpen;
        let doubleClick = self.activeGuiContainer().is_some_and(|container| container.doubleClick);

        if doubleClick
            && hoveredSlot.is_some_and(|slot| !(isPlayerInventory && slot == 0))
            && mouseButton == 0
        {
            let clickedSlot = hoveredSlot.expect("checked above");
            if modifiers.shift_key() && !self.inventoryShiftClickedStack.isEmpty() {
                if let Some((_cursor, slots, _)) = self.inventoryUiState() {
                    let group = self.activeContainerInventoryGroup(clickedSlot);
                    for candidate in 0..slots.len() as i32 {
                        if self.activeContainerInventoryGroup(candidate) != group {
                            continue;
                        }
                        let Some(stack) = slots.get(candidate as usize) else { continue; };
                        if !stack.isEmpty() && stack.canStackWith(&self.inventoryShiftClickedStack) {
                            changed |= self.sendPlayerInventoryClick(candidate, 0, ClickType::QuickMove)?;
                        }
                    }
                }
            } else {
                changed |= self.sendPlayerInventoryClick(clickedSlot, 0, ClickType::PickupAll)?;
            }
            if let Some(container) = self.activeGuiContainerMut() {
                container.doubleClick = false;
                container.cancelDragSplitting();
            }
            self.lastInventoryClick = None;
            return Ok(changed);
        }

        let (dragSplitting, dragButton, ignoreMouseUp, selected, mode) = self
            .activeGuiContainer()
            .map(|container| (
                container.dragSplitting,
                container.dragSplittingButton,
                container.ignoreMouseUp,
                container.dragSplittingSlots.iter().copied().collect::<Vec<_>>(),
                container.dragSplittingLimit,
            ))
            .unwrap_or((false, -1, false, Vec::new(), 0));

        if dragSplitting && dragButton != mouseButton {
            if let Some(container) = self.activeGuiContainerMut() {
                container.cancelDragSplitting();
                container.ignoreMouseUp = true;
            }
            return Ok(true);
        }

        if ignoreMouseUp {
            if let Some(container) = self.activeGuiContainerMut() {
                container.ignoreMouseUp = false;
                container.cancelDragSplitting();
            }
            return Ok(false);
        }

        if dragSplitting && !selected.is_empty() {
            changed |= self.sendPlayerInventoryClick(
                -999,
                Container::getQuickcraftMask(0, mode),
                ClickType::QuickCraft,
            )?;
            for selectedSlot in selected {
                changed |= self.sendPlayerInventoryClick(
                    selectedSlot,
                    Container::getQuickcraftMask(1, mode),
                    ClickType::QuickCraft,
                )?;
            }
            changed |= self.sendPlayerInventoryClick(
                -999,
                Container::getQuickcraftMask(2, mode),
                ClickType::QuickCraft,
            )?;
        } else if let Some((cursor, slots, _)) = self.inventoryUiState() {
            if !cursor.isEmpty() && slotId != -1 {
                let clickType = if mouseButton == 2 {
                    ClickType::Clone
                } else if modifiers.shift_key() && slotId >= 0 {
                    self.inventoryShiftClickedStack = slots
                        .get(slotId as usize)
                        .cloned()
                        .unwrap_or(ItemStack::EMPTY);
                    ClickType::QuickMove
                } else {
                    ClickType::Pickup
                };
                changed |= self.sendPlayerInventoryClick(slotId, mouseButton, clickType)?;
            }
        }

        if self.inventoryUiState().is_some_and(|(cursor, _, _)| cursor.isEmpty()) {
            self.lastInventoryClick = None;
        }
        if let Some(container) = self.activeGuiContainerMut() {
            container.cancelDragSplitting();
        }
        Ok(changed)
    }

    /// `GuiContainer.keyTyped` / `checkHotbarKeys` while a container screen
    /// owns keyboard input.
    fn inventoryKeyPressed(
        &mut self,
        framebufferWidth: u32,
        framebufferHeight: u32,
        key: KeyCode,
        binding: Option<KeyBindingId>,
        modifiers: ModifiersState,
        eventText: Option<&str>,
    ) -> Result<bool, String> {
        self.syncOpenContainerGui();
        if !self.isInventoryOpen() { return Ok(false); }
        if self.creativeInventoryOpen {
            return self.creativeInventoryKeyPressed(framebufferWidth, framebufferHeight, key, binding, modifiers);
        }
        if self.recipeBookKeyPressed(key, binding, modifiers, eventText)? {
            return Ok(true);
        }
        if self.repairKeyPressed(key, modifiers)? {
            return Ok(true);
        }
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        let Some(slotId) = self.activeSlotAt(mouseX, mouseY) else { return Ok(false); };
        let (mouseButton, clickType) = match binding {
            Some(KeyBindingId::Hotbar1) => (0, ClickType::Swap),
            Some(KeyBindingId::Hotbar2) => (1, ClickType::Swap),
            Some(KeyBindingId::Hotbar3) => (2, ClickType::Swap),
            Some(KeyBindingId::Hotbar4) => (3, ClickType::Swap),
            Some(KeyBindingId::Hotbar5) => (4, ClickType::Swap),
            Some(KeyBindingId::Hotbar6) => (5, ClickType::Swap),
            Some(KeyBindingId::Hotbar7) => (6, ClickType::Swap),
            Some(KeyBindingId::Hotbar8) => (7, ClickType::Swap),
            Some(KeyBindingId::Hotbar9) => (8, ClickType::Swap),
            Some(KeyBindingId::PickBlock) => (2, ClickType::Clone),
            Some(KeyBindingId::Drop) => (if modifiers.control_key() { 1 } else { 0 }, ClickType::Throw),
            _ => return Ok(false),
        };
        let Some((cursor, slots, _)) = self.inventoryUiState() else { return Ok(false); };
        if clickType == ClickType::Swap && !cursor.isEmpty() {
            return Ok(false);
        }
        if clickType == ClickType::Throw
            && slots.get(slotId as usize).map_or(true, ItemStack::isEmpty)
        {
            return Ok(false);
        }
        self.sendPlayerInventoryClick(slotId, mouseButton, clickType)
    }

    fn worldScroll(&mut self, wheelDelta: i32) -> bool {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
            return false;
        };
        let shared = connection.getSharedPlayState();
        if shared.withRead(|state| state.gameType) == GameType::Spectator {
            // Vanilla routes this to GuiSpectator or flight-speed control.
            // Neither branch is represented by InventoryPlayer selection.
            return false;
        }
        shared.changeCurrentHotbarItem(wheelDelta)
    }

    /// MCP `Minecraft.clickMouse` / `rightClickMouse` block-interaction branch.
    /// Entity targeting is connected when the multiplayer entity list exists.
    fn worldActionButton(&mut self, binding: KeyBindingId, pressed: bool) -> Result<bool, String> {
        if binding == KeyBindingId::Attack {
            self.attackButtonDown = pressed;
        } else if binding == KeyBindingId::UseItem {
            self.useButtonDown = pressed;
        }
        let shared = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => connection.getSharedPlayState(),
            _ => return Ok(false),
        };

        if binding == KeyBindingId::UseItem && pressed {
            // MCP runTickKeyboard never calls rightClickMouse while a timed use
            // is active, and rightClickMouse itself exits before assigning the
            // four-tick delay while PlayerControllerMP is mining a block.
            let handActive = shared.withRead(|state| {
                state.thePlayer.as_ref().is_some_and(|player| player.isHandActive())
            });
            if handActive || self.playerController.getIsHittingBlock() {
                return Ok(false);
            }
            self.rightClickDelayTimer = RIGHT_CLICK_DELAY_TICKS;
        }

        // The final three hand values preserve distinct MCP effects:
        // swingHand -> EntityPlayerSP.swingArm + CPacketAnimation,
        // startUseHand -> a continuous Item use action,
        // resetEquipHand -> ItemRenderer.resetEquippedProgress(hand).
        let (
            packets,
            attackedEntity,
            attackKnockbackSlowdown,
            swingHand,
            startUseHand,
            resetEquipHand,
            predictedPlacement,
            predictedBlockState,
        ) =
            if binding == KeyBindingId::Attack && !pressed {
                (
                    self.playerController.resetBlockRemoving(),
                    false,
                    false,
                    None,
                    None,
                    None,
                    None,
                    None,
                )
            } else if binding == KeyBindingId::UseItem && !pressed {
                let wasUsing = shared.stopUsingItem();
                let packets = if wasUsing {
                    vec![CPacketPlayerDigging::new(
                        DiggingAction::ReleaseUseItem,
                        crate::net::minecraft::util::math::BlockPos::BlockPos::new(0, 0, 0),
                        crate::net::minecraft::util::EnumFacing::EnumFacing::Down,
                    ).writePacketData()]
                } else {
                    Vec::new()
                };
                (packets, false, false, None, None, None, None, None)
            } else if !pressed {
                (Vec::new(), false, false, None, None, None, None, None)
            } else {
                shared.withWrite(|state| {
                    let gameType = state.gameType;
                    self.playerController.setGameType(gameType);
                    let (Some(world), Some(player)) = (
                        state.worldClient.as_mut(),
                        state.thePlayer.as_mut(),
                    ) else {
                        return (Vec::new(), false, false, None, None, None, None, None);
                    };
                    let reach = self.playerController.getBlockReachDistance();
                    let blockHit = player.rayTrace(world, reach, 1.0);
                    let eye = player.getPositionEyes(1.0);
                    let look = player.getLook(1.0);
                    let blockDistance = blockHit
                        .as_ref()
                        .map_or(reach, |hit| eye.distance_to(hit.hitVec));
                    let entityHit = world.rayTraceEntities(
                        player.entityId,
                        player.entity.ridingEntityId,
                        player.entity.boundingBox,
                        eye,
                        look,
                        if self.playerController.extendedReach() { 6.0 } else { reach },
                        blockDistance,
                        self.playerController.extendedReach(),
                    );
                    match binding {
                        KeyBindingId::Attack => {
                            let attackedEntity = entityHit.is_some();
                            let attackKnockbackSlowdown = entityHit.as_ref().is_some_and(|entity| {
                                let cooledStrength = player.getCooledAttackStrength(0.5);
                                let knockbackLevel = player
                                    .inventory
                                    .getCurrentItem()
                                    .getEnchantmentLevel(19);
                                let sprintKnockback = player.isSprinting && cooledStrength > 0.9;
                                let knockbackModifier =
                                    knockbackLevel + if sprintKnockback { 1 } else { 0 };
                                knockbackModifier > 0
                                    && world.clientAttackEntityFromReturnsTrue(entity.entityId)
                            });
                            let mut packets = if let Some(entity) = entityHit {
                                vec![CPacketUseEntity::attack(entity.entityId).writePacketData()]
                            } else {
                                blockHit
                                    .filter(|result| result.typeOfHit == RayTraceType::Block)
                                    .map(|result| {
                                        self.playerController.clickBlock(
                                            world,
                                            player,
                                            result.getBlockPos(),
                                            result.sideHit,
                                        )
                                    })
                                    .unwrap_or_default()
                            };
                            packets.push(CPacketAnimation::new(EnumHand::MainHand).writePacketData());
                            (
                                packets,
                                attackedEntity,
                                attackKnockbackSlowdown,
                                Some(EnumHand::MainHand),
                                None,
                                None,
                                None,
                                None,
                            )
                        }
                        KeyBindingId::UseItem => {
                            if let Some(entity) = entityHit {
                                // Full entity-side EnumActionResult dispatch remains a
                                // separate port. Preserve the existing server packet order
                                // until each entity's applyPlayerInteraction is available.
                                let target = world
                                    .entityPosition(entity.entityId)
                                    .expect("selected entity remains in WorldClient");
                                let relative = entity.hitVec.subtract_vector(
                                    target[0],
                                    target[1],
                                    target[2],
                                );
                                return (
                                    vec![
                                        CPacketUseEntity::interactAt(
                                            entity.entityId,
                                            EnumHand::MainHand,
                                            relative,
                                        )
                                        .writePacketData(),
                                        CPacketUseEntity::interact(
                                            entity.entityId,
                                            EnumHand::MainHand,
                                        )
                                        .writePacketData(),
                                    ],
                                    false,
                                    false,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                );
                            }

                            let mut packets = Vec::new();
                            let blockResult = blockHit
                                .filter(|result| result.typeOfHit == RayTraceType::Block);

                            // MCP `Minecraft#rightClickMouse` iterates EnumHand.values():
                            // MAIN_HAND first, then OFF_HAND. PASS/FAIL continue; SUCCESS
                            // owns the click and terminates the loop.
                            for hand in [EnumHand::MainHand, EnumHand::OffHand] {
                                let stack = player.getHeldItem(hand).clone();

                                if let Some(hit) = blockResult {
                                    if !world.getBlockState(hit.getBlockPos()).isAir() {
                                        let result = self.playerController.processRightClickBlock(
                                            world,
                                            player,
                                            hit,
                                            hand,
                                        );
                                        let predictedPlacement = result.predictedPlacement;
                                        let predictedBlockState = result.predictedBlockState;
                                        if let Some(packet) = result.packet {
                                            packets.push(packet);
                                        }
                                        if let Some((name, volume, pitch)) = result.sound {
                                            player.queueSoundAtPlayer(
                                                name, SoundCategory::Blocks, volume, pitch,
                                            );
                                        }
                                        if result.result == EnumActionResult::Success {
                                            packets.push(CPacketAnimation::new(hand).writePacketData());
                                            let reset = if !stack.isEmpty()
                                                && (result.usedItemBlock
                                                    || gameType == GameType::Creative)
                                            {
                                                Some(hand)
                                            } else {
                                                None
                                            };
                                            return (
                                                packets,
                                                false,
                                                false,
                                                Some(hand),
                                                None,
                                                reset,
                                                predictedPlacement,
                                                predictedBlockState,
                                            );
                                        }
                                    }
                                }

                                if stack.isEmpty() || gameType == GameType::Spectator {
                                    continue;
                                }

                                // `PlayerControllerMP#processRightClick` sends the hand
                                // packet before evaluating Item#onItemRightClick.
                                let airResult = self.playerController.processRightClick(world, player, hand);
                                if let Some(packet) = airResult.packet {
                                    packets.push(packet);
                                }

                                // Source-equivalent client-side item effects. The remote
                                // server remains authoritative and subsequent packets correct
                                // the predicted inventory/world state.
                                let mut airConsumed = false;
                                if let Some(fill) = airResult.fillBucket {
                                    if !player.capabilities.isCreativeMode {
                                        let heldIndex = if hand == EnumHand::MainHand {
                                            player.inventory.currentItem as i32
                                        } else { 40 };
                                        let filled = ItemStack {
                                            itemId: fill.bucket, count: 1, itemDamage: 0, tagCompound: None,
                                        };
                                        let held = player.inventory.getStackInSlot(heldIndex)
                                            .cloned().unwrap_or(ItemStack::EMPTY);
                                        if held.count - 1 <= 0 {
                                            let _ = player.inventory.setInventorySlotContents(heldIndex, filled);
                                        } else {
                                            let mut remaining = held;
                                            remaining.shrink(1);
                                            let _ = player.inventory.setInventorySlotContents(heldIndex, remaining);
                                            if let Some(emptySlot) = (0..player.inventory.mainInventory.len())
                                                .find(|slot| player.inventory.mainInventory[*slot].isEmpty())
                                            {
                                                let _ = player.inventory.setInventorySlotContents(emptySlot as i32, filled);
                                            }
                                        }
                                    }
                                    let _ = world.invalidateRegionAndSetBlock(
                                        fill.source,
                                        crate::net::minecraft::block::state::IBlockState::IBlockState::default(),
                                    );
                                    player.queueSoundAtPlayer(
                                        fill.sound, SoundCategory::Players, 1.0, 1.0,
                                    );
                                    airConsumed = true;
                                }
                                if let Some(empty) = airResult.emptyBucket {
                                    if !player.capabilities.isCreativeMode {
                                        let heldIndex = if hand == EnumHand::MainHand {
                                            player.inventory.currentItem as i32
                                        } else { 40 };
                                        let _ = player.inventory.setInventorySlotContents(
                                            heldIndex,
                                            ItemStack {
                                                itemId: crate::net::minecraft::item::ItemBucket::BUCKET,
                                                count: 1, itemDamage: 0, tagCompound: None,
                                            },
                                        );
                                    }
                                    let soundPosition = [
                                        empty.destination.x as f32 + 0.5,
                                        empty.destination.y as f32 + 0.5,
                                        empty.destination.z as f32 + 0.5,
                                    ];
                                    if empty.vaporizesWater {
                                        // MCP `ItemBucket#tryPlaceContainedLiquid`: Nether water
                                        // succeeds without placing FLOWING_WATER. It plays the
                                        // fire-extinguish sound at 0.5 volume with World#rand pitch
                                        // and emits eight SMOKE_LARGE particles inside the cell.
                                        let pitch = 2.6
                                            + (world.nextWorldRandomF32() - world.nextWorldRandomF32()) * 0.8;
                                        player.queueSoundAt(
                                            empty.sound, SoundCategory::Blocks, soundPosition, 0.5, pitch,
                                        );
                                        let mut requests = Vec::with_capacity(8);
                                        for _ in 0..8 {
                                            requests.push(ParticleSpawnRequest::new(
                                                EnumParticleTypes::SmokeLarge,
                                                [
                                                    empty.destination.x as f64 + math_random_f64(),
                                                    empty.destination.y as f64 + math_random_f64(),
                                                    empty.destination.z as f64 + math_random_f64(),
                                                ],
                                                [0.0, 0.0, 0.0],
                                                [0, 0],
                                            ));
                                        }
                                        world.queueParticleSpawns(requests);
                                    } else {
                                        // Source `tryPlaceContainedLiquid` sets the contained
                                        // flowing block on both logical sides; the server remains
                                        // authoritative and may overwrite this local prediction.
                                        let liquidId = if stack.itemId == crate::net::minecraft::item::ItemBucket::LAVA_BUCKET { 10 } else { 8 };
                                        let _ = world.invalidateRegionAndSetBlock(
                                            empty.destination,
                                            crate::net::minecraft::block::state::IBlockState::IBlockState::fromGlobalStateId(liquidId << 4),
                                        );
                                        player.queueSoundAt(
                                            empty.sound, SoundCategory::Blocks, soundPosition, 1.0, 1.0,
                                        );
                                    }
                                    airConsumed = true;
                                }
                                if let Some(thrown) = airResult.thrown {
                                    if !player.capabilities.isCreativeMode {
                                        let heldIndex = if hand == EnumHand::MainHand {
                                            player.inventory.currentItem as i32
                                        } else { 40 };
                                        let mut held = player.inventory.getStackInSlot(heldIndex)
                                            .cloned().unwrap_or(ItemStack::EMPTY);
                                        held.shrink(1);
                                        let _ = player.inventory.setInventorySlotContents(heldIndex, held);
                                    }
                                    player.queueSoundAtPlayer(
                                        thrown.sound, thrown.category, 0.5, thrown.pitch,
                                    );
                                    airConsumed = true;
                                }
                                if airConsumed {
                                    return (packets, false, false, None, Some(hand), Some(hand), None, None);
                                }

                                // Timed use actions (eat/drink, bow, shield). Ordinary
                                // PASS results continue to the off hand.
                                let canStart = stack.getItemUseAction()
                                    != crate::net::minecraft::item::EnumAction::EnumAction::None
                                    && (!stack.isFood()
                                        || ((player.getFoodStats().getFoodLevel() < 20
                                            || stack.isAlwaysEdible())
                                            && !player.capabilities.disableDamage))
                                    && (stack.itemId != 261
                                        || gameType == GameType::Creative
                                        || player
                                            .inventory
                                            .offHandInventory
                                            .iter()
                                            .chain(std::iter::once(
                                                player.inventory.getCurrentItem(),
                                            ))
                                            .chain(player.inventory.mainInventory.iter())
                                            .any(|candidate| {
                                                matches!(candidate.itemId, 262 | 439 | 440)
                                                    && !candidate.isEmpty()
                                            }));
                                if canStart {
                                    return (
                                        packets,
                                        false,
                                        false,
                                        None,
                                        Some(hand),
                                        Some(hand),
                                        None,
                                        None,
                                    );
                                }
                            }
                            (packets, false, false, None, None, None, None, None)
                        }
                        _ => (Vec::new(), false, false, None, None, None, None, None),
                    }
                })
            };

        let pendingBlockDestruction = if binding == KeyBindingId::Attack && pressed {
            self.playerController.takeBlockDestroyEffect()
        } else {
            None
        };
        if let Some(sound) = self.playerController.takeBlockHitSound() {
            let _ = shared.queueLocalPlayerSound(sound);
        }

        let mut packets = packets;
        if let Some(hand) = swingHand {
            shared.swingLocalArm(hand);
        }
        if binding == KeyBindingId::Attack && pressed && attackedEntity {
            // `EntityPlayer#attackTargetEntityWithCurrentItem` calculates
            // knockback from the pre-reset attack strength, then applies the
            // 0.6 horizontal slowdown and sprint cancellation on success.
            if attackKnockbackSlowdown {
                let _ = shared.applyLocalAttackKnockbackSlowdown();
            }
            // PlayerControllerMP resets the attack-strength ticker after the
            // local attack call; block mining does not enter this branch.
            shared.resetLocalAttackCooldown();
        }
        if let Some(hand) = startUseHand {
            // Revalidate while taking the write lock. Input/network updates can
            // race the read snapshot, so never force an invalid active hand.
            let _ = shared.startUsingHeldItemExact(hand);
        }
        if let Some(hand) = resetEquipHand {
            match hand {
                EnumHand::MainHand => self.itemRenderer.resetEquippedProgressMainHand(),
                EnumHand::OffHand => self.itemRenderer.resetEquippedProgressOffHand(),
            }
        }
        if let Some(slot) = shared.currentHotbarSlot() {
            if let Some(packet) = self.playerController.syncCurrentPlayItem(slot) {
                packets.insert(0, packet);
            }
        }
        if packets.is_empty() {
            return Ok(false);
        }
        if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
            connection.sendPlayPackets(packets)?;
            if let Some((position, blockState)) = pendingBlockDestruction {
                // MCP sends START/STOP before `onPlayerDestroyBlock`, whose
                // event 2001 observes the pre-removal actual state and whose
                // setBlockState(AIR, 11) then updates the remote world.
                shared.withRead(|state| {
                    if let Some(world) = state.worldClient.as_ref() {
                        self.particleManager
                            .addBlockDestroyEffects(world, position, blockState);
                    }
                });
                let _ = shared.applyPredictedBlockDestruction(position, blockState);
            }
            if let (Some(placement), Some(hand)) = (predictedPlacement, swingHand) {
                // `PlayerControllerMP#processRightClickBlock` queues the packet
                // before invoking ItemBlock#onItemUse on the remote world.
                // Apply the guarded prediction only after the send succeeds.
                let _ = shared.applyPredictedItemBlockPlacement(placement, hand);
            }
            if let Some(prediction) = predictedBlockState {
                // Concrete `Block#onBlockActivated` implementations such as
                // wooden doors, trapdoors, gates and buttons mutate the remote
                // WorldClient after the use-on-block packet is queued. Blocks
                // that explicitly return on `world.isRemote` never enter here.
                let _ = shared.applyPredictedBlockState(
                    prediction.pos,
                    prediction.expectedState,
                    prediction.state,
                );
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn turnPlayer(&mut self, deltaX: f64, deltaY: f64, settings: &GameSettings) -> bool {
        let ActiveGuiScreen::World { connection, .. } = &self.currentScreen else {
            return false;
        };
        let sensitivity = settings.mouseSensitivity * 0.6 + 0.2;
        let factor = sensitivity * sensitivity * sensitivity * 8.0;
        let yaw = deltaX as f32 * factor;
        // winit reports positive Y down; LWJGL Mouse.getDY used by 1.12.2
        // reports positive Y up. Convert before applying invertMouse.
        let lwjglPitch = -(deltaY as f32) * factor;
        let invert = if settings.invertMouse { -1.0 } else { 1.0 };
        connection
            .getSharedPlayState()
            .turnLocalPlayer(yaw, lwjglPitch * invert)
    }

    fn resize(&mut self, minecraft: &Minecraft, framebufferWidth: u32, framebufferHeight: u32) {
        let unicode = minecraft.gameSettings.forceUnicodeFont || self.locale.is_unicode();
        self.scaledResolution = ScaledResolution::new(
            framebufferWidth.max(1) as i32,
            framebufferHeight.max(1) as i32,
            minecraft.gameSettings.guiScale,
            unicode,
        );
        self.guiInventory.initGui(self.scaledResolution.scaled_width(), self.scaledResolution.scaled_height());
        self.guiCreative.initGui(self.scaledResolution.scaled_width(), self.scaledResolution.scaled_height());
        if let Some(gui) = self.guiChest.as_mut() {
            gui.initGui(self.scaledResolution.scaled_width(), self.scaledResolution.scaled_height());
        }
        if let Some(gui) = self.guiShulkerBox.as_mut() {
            gui.initGui(self.scaledResolution.scaled_width(), self.scaledResolution.scaled_height());
        }
        if let Some(gui) = self.guiHorse.as_mut() {
            gui.initGui(self.scaledResolution.scaled_width(), self.scaledResolution.scaled_height());
        }
        if let Some(gui) = self.guiDedicated.as_mut() {
            gui.initGui(self.scaledResolution.scaled_width(), self.scaledResolution.scaled_height());
        }
        if let Some(chat) = self.guiChat.as_mut() {
            chat.resize(self.scaledResolution.scaled_width(), self.scaledResolution.scaled_height());
        }
        self.initCurrentScreen(minecraft);
        self.initWorldGui(minecraft);
    }

    fn initCurrentScreen(&mut self, minecraft: &Minecraft) {
        if !matches!(self.currentScreen, ActiveGuiScreen::World { .. }) {
            self.guiChat = None;
            self.worldRenderer.resetChatScroll();
        }
        let width = self.scaledResolution.scaled_width();
        let height = self.scaledResolution.scaled_height();
        match &mut self.currentScreen {
            ActiveGuiScreen::Empty => {}
            ActiveGuiScreen::MainMenu(screen) => screen.initGui(width, height, current_menu_date(), &self.locale, &self.fontRendererObj),
            ActiveGuiScreen::AccountManager(screen) => screen.initGui(width, height, &self.accountConfig),
            ActiveGuiScreen::MicrosoftAuth(screen) => screen.initGui(width, height),
            ActiveGuiScreen::SessionLogin(screen) => screen.initGui(width, height),
            ActiveGuiScreen::OfflineLogin(screen) => screen.initGui(width, height),
            ActiveGuiScreen::Options(screen) => screen.initGui(width, height, &self.locale, &minecraft.gameSettings),
            ActiveGuiScreen::Controls(screen) => screen.initGui(width, height, &self.locale, &minecraft.gameSettings, &self.fontRendererObj),
            ActiveGuiScreen::VideoSettings(screen) => screen.initGui(width, height, &self.locale, &minecraft.gameSettings),
            ActiveGuiScreen::ShaderSettings(screen) => screen.initGui(width, height),
            ActiveGuiScreen::SoundSettings(screen) => screen.initGui(width, height, &self.locale, &minecraft.gameSettings),
            ActiveGuiScreen::ChatSettings(screen) => screen.initGui(width, height, &self.locale, &minecraft.gameSettings),
            ActiveGuiScreen::SkinSettings(screen) => screen.initGui(width, height, &self.locale, &minecraft.gameSettings),
            ActiveGuiScreen::ResourcePacks(screen) => screen.initGui(width, height, &self.locale),
            ActiveGuiScreen::Multiplayer(screen) => screen.initGui(width, height, &self.locale),
            ActiveGuiScreen::WorldSelection(screen) => screen.initGui(width, height, &self.locale),
            ActiveGuiScreen::CreateWorld(screen) => screen.initGui(width, height, &self.locale, &self.fontRendererObj),
            ActiveGuiScreen::Language { screen, .. } => screen.initGui(width, height, &self.locale, &minecraft.gameSettings, &self.languageManager),
            ActiveGuiScreen::AddServer { screen, .. } => screen.initGui(width, height, &self.locale, &self.fontRendererObj),
            ActiveGuiScreen::DirectConnect { screen, .. } => screen.initGui(width, height, &self.locale, &self.fontRendererObj, &minecraft.gameSettings.lastServer),
            ActiveGuiScreen::ConfirmDelete { screen, .. } => screen.initGui(width, height, &self.fontRendererObj),
            ActiveGuiScreen::Connecting { screen, .. } => screen.initGui(width, height, &self.locale),
            ActiveGuiScreen::Disconnected { screen, .. } => screen.initGui(width, height, &self.fontRendererObj),
            ActiveGuiScreen::DownloadTerrain { screen, .. } => screen.initGui(width, height),
            ActiveGuiScreen::World { .. } => {},
        }
    }

    fn switchTo(&mut self, minecraft: &Minecraft, screenId: ScreenId) -> anyhow::Result<()> {
        self.currentScreen = match screenId {
            ScreenId::MainMenu => ActiveGuiScreen::MainMenu(Self::createMainMenu(minecraft)?),
            ScreenId::Options => ActiveGuiScreen::Options(GuiOptions::new()),
            ScreenId::Multiplayer => ActiveGuiScreen::Multiplayer(GuiMultiplayer::new(minecraft.gameDir.clone())),
            ScreenId::WorldSelection => ActiveGuiScreen::WorldSelection(GuiWorldSelection::new(minecraft.gameDir.join("saves"))),
        };
        self.lastGuiFrame = Instant::now();
        self.initCurrentScreen(minecraft);
        Ok(())
    }

    fn openAccountManager(&mut self, minecraft: &Minecraft, notification: Option<String>) {
        self.currentScreen = ActiveGuiScreen::AccountManager(match notification {
            Some(message) => GuiAccountManager::withNotification(message),
            None => GuiAccountManager::new(),
        });
        self.initCurrentScreen(minecraft);
    }

    fn openMicrosoftAuth(&mut self, minecraft: &Minecraft) {
        self.currentScreen = ActiveGuiScreen::MicrosoftAuth(GuiMicrosoftAuth::new());
        self.initCurrentScreen(minecraft);
    }

    fn openSessionLogin(&mut self, minecraft: &Minecraft) {
        self.currentScreen = ActiveGuiScreen::SessionLogin(GuiSessionLogin::new());
        self.initCurrentScreen(minecraft);
    }

    fn openOfflineLogin(&mut self, minecraft: &Minecraft) {
        self.currentScreen = ActiveGuiScreen::OfflineLogin(GuiAltCracked::new());
        self.initCurrentScreen(minecraft);
    }

    fn openVideoSettings(&mut self, minecraft: &Minecraft) {
        self.currentScreen = ActiveGuiScreen::VideoSettings(GuiVideoSettings::new());
        self.initCurrentScreen(minecraft);
    }

    fn openShaderSettings(&mut self, minecraft: &Minecraft, rendererDescription: String) {
        self.currentScreen = ActiveGuiScreen::ShaderSettings(GuiShader::newWithSettings(
            minecraft.gameDir.clone(),
            rendererDescription,
            minecraft.gameSettings.language.clone(),
            minecraft.gameSettings.advancedItemTooltips,
        ));
        self.initCurrentScreen(minecraft);
    }

    fn returnToVideoSettings(&mut self, minecraft: &Minecraft) {
        self.currentScreen = ActiveGuiScreen::VideoSettings(GuiVideoSettings::new());
        self.initCurrentScreen(minecraft);
    }

    fn openLanguage(&mut self, minecraft: &Minecraft, parent: ScreenId) {
        self.currentScreen = ActiveGuiScreen::Language { screen: GuiLanguage::new(minecraft.gameSettings.language.clone()), parent };
        self.initCurrentScreen(minecraft);
    }

    /// MCP `GuiLanguage.List#elementClicked` -> `Minecraft#refreshResources`.
    /// The settings language has already been updated by the caller. Reuse the
    /// runtime's complete resource-reload chain rather than refreshing only
    /// Locale/FontRenderer, then re-init the visible language screen.
    fn setLanguage(&mut self, minecraft: &Minecraft, languageCode: &str) -> Result<(), String> {
        debug_assert_eq!(minecraft.gameSettings.language, languageCode);
        self.reloadResources(minecraft)?;
        self.initCurrentScreen(minecraft);
        self.initWorldGui(minecraft);
        self.lastGuiFrame = Instant::now();
        Ok(())
    }

    fn openDirectConnect(&mut self, minecraft: &Minecraft) {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        if let ActiveGuiScreen::Multiplayer(parent) = current {
            let defaultName = self.locale.translate_key("selectServer.defaultName").to_owned();
            self.currentScreen = ActiveGuiScreen::DirectConnect {
                screen: GuiScreenServerList::new(ServerData::new(defaultName, "", false)),
                parent: Box::new(parent),
            };
        } else { self.currentScreen = current; }
        self.initCurrentScreen(minecraft);
    }

    fn openAddServer(&mut self, minecraft: &Minecraft, editingIndex: Option<usize>, server: Option<ServerData>) {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        if let ActiveGuiScreen::Multiplayer(parent) = current {
            let server = server.unwrap_or_else(|| ServerData::new(self.locale.translate_key("selectServer.defaultName"), "", false));
            self.currentScreen = ActiveGuiScreen::AddServer { screen: GuiScreenAddServer::new(server), parent: Box::new(parent), editingIndex };
        } else { self.currentScreen = current; }
        self.initCurrentScreen(minecraft);
    }

    fn openDeleteConfirm(&mut self, minecraft: &Minecraft, serverIndex: usize, serverName: String) {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        if let ActiveGuiScreen::Multiplayer(parent) = current {
            let line1 = self.locale.translate_key("selectServer.deleteQuestion").to_owned();
            let line2 = format!("'{serverName}' {}", self.locale.translate_key("selectServer.deleteWarning"));
            let screen = GuiYesNo::new(
                line1, line2, self.locale.translate_key("selectServer.deleteButton").to_owned(),
                self.locale.translate_key("gui.cancel").to_owned(), serverIndex as i32,
            );
            self.currentScreen = ActiveGuiScreen::ConfirmDelete { screen, parent: Box::new(parent), serverIndex };
        } else { self.currentScreen = current; }
        self.initCurrentScreen(minecraft);
    }

    fn returnToMultiplayer(&mut self, minecraft: &Minecraft) {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        self.currentScreen = match current {
            ActiveGuiScreen::AddServer { parent, .. }
            | ActiveGuiScreen::DirectConnect { parent, .. }
            | ActiveGuiScreen::ConfirmDelete { parent, .. }
            | ActiveGuiScreen::Connecting { parent, .. }
            | ActiveGuiScreen::Disconnected { parent, .. }
            | ActiveGuiScreen::DownloadTerrain { parent, .. }
            | ActiveGuiScreen::World { parent, .. } => ActiveGuiScreen::Multiplayer(*parent),
            other => other,
        };
        self.inventoryOpen = false;
        self.creativeInventoryOpen = false;
        self.guiChest = None;
        self.guiShulkerBox = None;
        self.guiHorse = None;
        self.guiDedicated = None;
        self.guiContainerWindowId = None;
        self.worldGuiScreen = None;
        self.initCurrentScreen(minecraft);
    }

    fn leaveWorldToMainMenu(&mut self, minecraft: &Minecraft) -> anyhow::Result<()> {
        // Dropping `GuiConnecting` closes the active network thread through
        // its Drop implementation, matching WorldClient#sendQuittingDisconnectingPacket
        // followed by Minecraft#loadWorld(null) for this multiplayer-only port.
        self.currentScreen = ActiveGuiScreen::Empty;
        self.inventoryOpen = false;
        self.creativeInventoryOpen = false;
        self.guiChest = None;
        self.guiShulkerBox = None;
        self.guiHorse = None;
        self.guiDedicated = None;
        self.guiContainerWindowId = None;
        self.worldGuiScreen = None;
        self.currentScreen = ActiveGuiScreen::MainMenu(Self::createMainMenu(minecraft)?);
        self.initCurrentScreen(minecraft);
        Ok(())
    }

    fn openConnecting(&mut self, minecraft: &Minecraft, server: ServerData) {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        let parent = match current {
            ActiveGuiScreen::Multiplayer(parent) => Some(Box::new(parent)),
            ActiveGuiScreen::DirectConnect { parent, .. } => Some(parent),
            other => { self.currentScreen = other; None }
        };
        if let Some(parent) = parent {
            self.currentScreen = ActiveGuiScreen::Connecting {
                screen: GuiConnecting::new(
                    server,
                    minecraft.getSession().clone(),
                    minecraft.gameSettings.language.clone(),
                    minecraft.gameSettings.renderDistanceChunks,
                    minecraft.gameSettings.chatVisibility,
                    minecraft.gameSettings.chatColours,
                    minecraft.gameSettings.modelPartFlags,
                    minecraft.gameSettings.mainHand,
                ), parent,
            };
            self.initCurrentScreen(minecraft);
        }
    }

    fn cancelConnecting(&mut self, minecraft: &Minecraft) {
        if let ActiveGuiScreen::Connecting { screen, .. } = &mut self.currentScreen { screen.cancel(); }
        self.returnToMultiplayer(minecraft);
    }

    fn openDisconnected(&mut self, minecraft: &Minecraft, reasonKey: &str, message: String) {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        self.currentScreen = match current {
            ActiveGuiScreen::Connecting { parent, .. } | ActiveGuiScreen::DownloadTerrain { parent, .. } | ActiveGuiScreen::World { parent, .. } => {
                let reason = self.locale.translate_key(reasonKey).to_owned();
                let message = match message.as_str() {
                    "disconnect.loginFailedInfo.serversUnavailable" => self.locale.translate_key("disconnect.loginFailedInfo.serversUnavailable").to_owned(),
                    "disconnect.loginFailedInfo.invalidSession" => self.locale.translate_key("disconnect.loginFailedInfo.invalidSession").to_owned(),
                    _ => message,
                };
                ActiveGuiScreen::Disconnected { screen: GuiDisconnected::new(reason, message, self.locale.translate_key("gui.toMenu").to_owned()), parent }
            }
            other => other,
        };
        self.worldGuiScreen = None;
        self.initCurrentScreen(minecraft);
    }

    fn openDownloadTerrain(&mut self, minecraft: &Minecraft) {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        self.currentScreen = match current {
            ActiveGuiScreen::Connecting { screen: connection, parent }
            | ActiveGuiScreen::World { connection, parent } => ActiveGuiScreen::DownloadTerrain {
                screen: GuiDownloadTerrain::new(self.locale.translate_key("multiplayer.downloadingTerrain").to_owned()), connection, parent,
            },
            other => other,
        };
        self.worldGuiScreen = None;
        self.pendingWorldMouseFocus = Some(false);
        self.inventoryOpen = false;
        self.creativeInventoryOpen = false;
        self.guiChest = None;
        self.guiShulkerBox = None;
        self.guiHorse = None;
        self.guiDedicated = None;
        self.guiContainerWindowId = None;
        self.initCurrentScreen(minecraft);
    }

    fn openWorld(&mut self, minecraft: &Minecraft) {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        self.currentScreen = match current {
            ActiveGuiScreen::DownloadTerrain { connection, parent, .. } => ActiveGuiScreen::World { connection, parent },
            other => other,
        };
        self.lastWorldRevision = 0;
        self.worldRenderer.clearCaches();
        self.particleManager.clearEffects();
        self.itemRenderer.clear();
        self.inventoryOpen = false;
        self.creativeInventoryOpen = false;
        self.guiChest = None;
        self.guiShulkerBox = None;
        self.guiHorse = None;
        self.guiDedicated = None;
        self.guiContainerWindowId = None;
        self.worldGuiScreen = None;
        self.pendingWorldMouseFocus = Some(true);
        self.initCurrentScreen(minecraft);
    }

    fn saveServerAndReturn(&mut self, minecraft: &Minecraft, editingIndex: Option<usize>, server: ServerData) -> anyhow::Result<()> {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        self.currentScreen = match current {
            ActiveGuiScreen::AddServer { mut parent, .. } => {
                if let Some(index) = editingIndex { parent.editServer(index, server)?; } else { parent.addServer(server)?; }
                ActiveGuiScreen::Multiplayer(*parent)
            }
            other => other,
        };
        self.initCurrentScreen(minecraft);
        Ok(())
    }

    fn deleteServerAndReturn(&mut self, minecraft: &Minecraft, index: usize) -> anyhow::Result<()> {
        let current = std::mem::replace(&mut self.currentScreen, ActiveGuiScreen::Empty);
        self.currentScreen = match current {
            ActiveGuiScreen::ConfirmDelete { mut parent, .. } => { parent.deleteServer(index)?; ActiveGuiScreen::Multiplayer(*parent) }
            other => other,
        };
        self.initCurrentScreen(minecraft);
        Ok(())
    }

    fn cursorGuiPosition(&self, framebufferWidth: u32, framebufferHeight: u32) -> (i32, i32) {
        if !self.mouseInsideWindow || framebufferWidth == 0 || framebufferHeight == 0 { return (-1, -1); }
        let mouseX = (self.mousePosition.x * self.scaledResolution.scaled_width() as f64 / framebufferWidth as f64).floor() as i32;
        let mouseY = (self.mousePosition.y * self.scaledResolution.scaled_height() as f64 / framebufferHeight as f64).floor() as i32;
        (mouseX, mouseY)
    }

    fn draw(
        &mut self,
        minecraft: &Minecraft,
        framebufferWidth: u32,
        framebufferHeight: u32,
        partialTicks: f32,
        debugFps: i32,
        graphicsDevice: &str,
        renderBackend: crate::launcher::RenderBackend::RenderBackend,
    ) -> anyhow::Result<RuntimeFrame> {
        self.syncOpenContainerGui();
        self.syncInventoryGameType();
        let partialTicks = partialTicks.clamp(0.0, 1.0);
        let systemTimeMillis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        if let ActiveGuiScreen::World { connection, .. } = &self.currentScreen {
            let sharedState = connection.getSharedPlayState();
            if let Some((position, yaw, pitch)) = sharedState.withRead(|state| {
                let player = state.thePlayer.as_ref()?;
                let eyes = player.getPositionEyes(partialTicks);
                let yaw = player.entity.prevRotationYaw
                    + (player.entity.rotationYaw - player.entity.prevRotationYaw) * partialTicks;
                let pitch = player.entity.prevRotationPitch
                    + (player.entity.rotationPitch - player.entity.prevRotationPitch) * partialTicks;
                Some(([eyes.x as f32, eyes.y as f32, eyes.z as f32], yaw, pitch))
            }) {
                self.soundHandler.setListener(position, yaw, pitch);
            }
            sharedState.pruneDamagedBlocksForRender(partialTicks);
            let firstPersonItemState = self.itemRenderer.renderState(partialTicks);
            let playerInventoryOpen = self.inventoryOpen;
            let creativeInventoryOpen = self.creativeInventoryOpen;
            let creativeSelectedTab = self.guiCreative.selectedTabIndex;
            let creativeCurrentScroll = self.guiCreative.currentScroll;
            let creativeCanScroll = self.guiCreative.needsScrollBars();
            let creativeContainer = creativeInventoryOpen.then(|| self.guiCreative.container.clone());
            let creativeSearchInput = if creativeInventoryOpen {
                self.guiCreative.searchRenderState(&self.fontRendererObj)
            } else {
                None
            };
            let anvilNameInput = self
                .guiDedicated
                .as_ref()
                .and_then(DedicatedContainerGui::repairNameField)
                .map(|field| field.buildRenderState(&self.fontRendererObj));
            let anvilCostFormat = self.locale.translate_key("container.repair.cost").to_owned();
            let anvilTooExpensive = self.locale.translate_key("container.repair.expensive").to_owned();
            let creativeTabTitle = crate::net::minecraft::creativetab::CreativeTabs::byIndex(creativeSelectedTab)
                .map(|tab| self.locale.translate_key(tab.getTranslatedTabLabel()).to_owned())
                .unwrap_or_default();
            let creativeDisplaySlots = if creativeInventoryOpen {
                sharedState.withRead(|state| {
                    let playerSlots = state.thePlayer.as_ref()
                        .map(|player| player.inventoryContainer.slots())
                        .unwrap_or(&[]);
                    self.guiCreative.displayStacks(playerSlots)
                })
            } else {
                Vec::new()
            };
            let (dragSplitting, dragLimit, dragRemnant, dragSlots) = self.activeGuiContainer()
                .map(|container| (
                    container.dragSplitting,
                    container.dragSplittingLimit,
                    container.dragSplittingRemnant,
                    container.dragSplittingSlots.iter().copied().collect::<Vec<_>>(),
                ))
                .unwrap_or((false, 0, 0, Vec::new()));
            let rawInventoryTitle = sharedState.withRead(|state| {
                state.thePlayer.as_ref()
                    .and_then(|player| player.openContainer.as_ref())
                    .map(|container| {
                        container.title()
                            .resolveWithLocale(&self.locale)
                            .getUnformattedText()
                            .to_owned()
                    })
                    .unwrap_or_default()
            });
            // MCP `GuiContainer#drawGuiContainerForegroundLayer` receives an
            // `ITextComponent`. JSON translation components are resolved above;
            // legacy open-window packets may instead carry a plain localization
            // key such as `tile.workbench.name` or `tile.anvil.name`.
            let inventoryTitle = if self.locale.has_key(&rawInventoryTitle) {
                self.locale.translate_key(&rawInventoryTitle).to_owned()
            } else {
                rawInventoryTitle
            };
            let playerInventoryTitle = self.locale.translate_key("container.inventory").to_owned();
            // Match RenderGlobal/ChunkRenderDispatcher ownership: hold the
            // network WorldClient lock only while copying the small immutable
            // chunk neighbourhood selected for this frame. JSON/model lookup,
            // tessellation and GPU upload happen after the lock is released so
            // keep-alive and incoming chunk packets cannot be starved by render work.
            let chatOpen = self.isChatOpen();
            let chatInput = self.guiChat.as_ref().map(|chat| chat.renderState(&self.fontRendererObj));
            let particleStates = self.particleManager.renderStates();
            let miscParticleStates = self.particleManager.miscRenderStates(partialTicks);
            let localDestroyProgress = sharedState.withRead(|state| {
                state.thePlayer.as_ref().and_then(|player| {
                    self.playerController.getDestroyBlockProgress(
                        player.entityId,
                        state.cloudTickCounter,
                    )
                })
            });
            let worldGuiDrawList = self.worldGuiScreen.as_mut().map(|screen| {
                let mut drawList = GuiDrawList::new();
                match screen {
                    WorldGuiScreen::IngameMenu(screen) => screen.drawScreen(
                        &mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks,
                    ),
                    WorldGuiScreen::Options(screen) => screen.drawScreenInWorld(
                        &mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks,
                    ),
                    WorldGuiScreen::Controls(screen) => screen.drawScreen(
                        &mut drawList, &mut self.fontRendererObj, &self.locale,
                        &minecraft.gameSettings, mouseX, mouseY, partialTicks, true,
                    ),
                    WorldGuiScreen::VideoSettings(screen) => screen.drawScreenInWorld(
                        &mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks,
                    ),
                    WorldGuiScreen::ShaderSettings(screen) => screen.drawScreenInWorld(
                        &mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks,
                    ),
                    WorldGuiScreen::SoundSettings(screen) => screen.drawScreenInWorld(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
                    WorldGuiScreen::ChatSettings(screen) => screen.drawScreenInWorld(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
                    WorldGuiScreen::SkinSettings(screen) => screen.drawScreenInWorld(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
                    WorldGuiScreen::ResourcePacks(screen) => screen.drawScreenInWorld(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
                    WorldGuiScreen::Language(screen) => screen.drawScreenInWorld(
                        &mut drawList, &mut self.fontRendererObj, &self.locale, mouseX, mouseY, partialTicks,
                    ),
                    WorldGuiScreen::EditSign(screen) => screen.drawScreen(
                        &mut drawList, &mut self.fontRendererObj, &self.locale,
                        mouseX, mouseY, partialTicks,
                    ),
                    WorldGuiScreen::GameOver(screen) => screen.drawScreen(
                        &mut drawList, &mut self.fontRendererObj, &self.locale,
                        mouseX, mouseY, partialTicks,
                    ),
                    WorldGuiScreen::GameOverConfirm { screen, .. } => screen.drawScreenInWorld(
                        &mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks,
                    ),
                }
                drawList
            });
            let enchantmentBookState = self
                .guiDedicated
                .as_ref()
                .and_then(|gui| gui.enchantmentBookRenderState(partialTicks));
            let recipeBookState = self.recipeBookRenderState();
            let capture = sharedState.withRead(|state| {
                self.worldRenderer.capture(
                    state,
                    framebufferWidth,
                    framebufferHeight,
                    self.scaledResolution.scaled_width(),
                    self.scaledResolution.scaled_height(),
                    minecraft.gameSettings.mainHand,
                    minecraft.session.getProfile().getId().unwrap_or_else(uuid::Uuid::nil),
                    minecraft.gameSettings.modelPartFlags,
                    firstPersonItemState.clone(),
                    self.playerListKeyDown,
                    chatOpen,
                    minecraft.gameSettings.chatVisibility != EnumChatVisibility::Hidden,
                    minecraft.gameSettings.showSubtitles,
                    minecraft.gameSettings.showDebugInfo,
                    minecraft.gameSettings.reducedDebugInfo,
                    minecraft.gameSettings.advancedItemTooltips,
                    minecraft.gameSettings.showDebugProfilerChart,
                    minecraft.gameSettings.showLagometer,
                    debugFps,
                    graphicsDevice.to_owned(),
                    chatInput.clone(),
                    worldGuiDrawList.clone(),
                    minecraft.gameSettings.chatOpacity,
                    minecraft.gameSettings.chatScale,
                    minecraft.gameSettings.chatWidth,
                    minecraft.gameSettings.chatHeightFocused,
                    minecraft.gameSettings.chatHeightUnfocused,
                    minecraft.session.getProfile().getName().to_owned(),
                    playerInventoryOpen,
                    creativeInventoryOpen,
                    creativeSelectedTab,
                    creativeCurrentScroll,
                    creativeCanScroll,
                    creativeContainer.clone(),
                    creativeDisplaySlots.clone(),
                    creativeSearchInput.clone(),
                    anvilNameInput.clone(),
                    enchantmentBookState,
                    recipeBookState.clone(),
                    anvilCostFormat.clone(),
                    anvilTooExpensive.clone(),
                    creativeTabTitle.clone(),
                    inventoryTitle.clone(),
                    playerInventoryTitle.clone(),
                    mouseX,
                    mouseY,
                    self.inventoryOldMouseX,
                    self.inventoryOldMouseY,
                    dragSplitting,
                    dragLimit,
                    dragRemnant,
                    dragSlots.clone(),
                    particleStates,
                    miscParticleStates,
                    localDestroyProgress,
                    minecraft.gameSettings.thirdPersonView,
                    minecraft.gameSettings.fovSetting,
                    minecraft.gameSettings.renderDistanceChunks,
                    minecraft.gameSettings.fancyGraphics,
                    minecraft.gameSettings.clouds,
                    minecraft.gameSettings.ofClouds,
                    minecraft.gameSettings.ofCloudsHeight,
                    minecraft.gameSettings.ambientOcclusion,
                    partialTicks,
                    minecraft.gameSettings.gammaSetting,
                )
            });
            let frame = self
                .worldRenderer
                .render(capture)
                .map(RuntimeFrame::World);
            // GuiInventory stores the cursor after drawing and uses the prior
            // frame's coordinates for drawEntityOnScreen, exactly as MCP.
            if self.inventoryOpen || self.creativeInventoryOpen {
                self.inventoryOldMouseX = mouseX as f32;
                self.inventoryOldMouseY = mouseY as f32;
            }
            return frame;
        }

        // GUI-only screens are scheduled by `getLimitFramerate()` at the MCP
        // 30 FPS cap. Keep animation interpolation based on elapsed GUI-frame
        // time rather than the world tick fraction so the panorama speed is
        // stable across scheduling jitter.
        let now = Instant::now();
        let partialTicks = (now.duration_since(self.lastGuiFrame).as_secs_f32() * 20.0)
            .clamp(0.0, 1.0);
        self.lastGuiFrame = now;
        let mut drawList = GuiDrawList::new();
        match &mut self.currentScreen {
            ActiveGuiScreen::Empty => {}
            ActiveGuiScreen::MainMenu(screen) => screen.drawScreen(
                &mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks,
                systemTimeMillis, minecraft.getVersionType(), self.mouseInsideWindow,
            ),
            ActiveGuiScreen::AccountManager(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks, &self.accountConfig, minecraft.getSession().getUsername()),
            ActiveGuiScreen::MicrosoftAuth(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::SessionLogin(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::OfflineLogin(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::Options(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::Controls(screen) => screen.drawScreen(
                &mut drawList, &mut self.fontRendererObj, &self.locale,
                &minecraft.gameSettings, mouseX, mouseY, partialTicks, false,
            ),
            ActiveGuiScreen::VideoSettings(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::ShaderSettings(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::SoundSettings(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::ChatSettings(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::SkinSettings(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::ResourcePacks(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::Multiplayer(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::WorldSelection(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, &self.locale, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::CreateWorld(screen) => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, &self.locale, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::Language { screen, .. } => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, &self.locale, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::AddServer { screen, .. } => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::DirectConnect { screen, .. } => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::ConfirmDelete { screen, .. } => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::Connecting { screen, .. } => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::Disconnected { screen, .. } => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::DownloadTerrain { screen, .. } => screen.drawScreen(&mut drawList, &mut self.fontRendererObj, mouseX, mouseY, partialTicks),
            ActiveGuiScreen::World { .. } => unreachable!("world rendering returned before GUI draw"),
        }
        let _ = renderBackend;
        self.guiRenderer
            .prepareNativeFrame(
                &drawList,
                self.scaledResolution.scaled_width(),
                self.scaledResolution.scaled_height(),
                framebufferWidth,
                framebufferHeight,
            )
            .map(RuntimeFrame::NativeGui)
    }

    fn updateElytraSounds(&mut self) {
        let snapshot = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. }
            | ActiveGuiScreen::DownloadTerrain { connection, .. } => connection
                .getSharedPlayState()
                .withRead(|state| state.thePlayer.as_ref().map(|player| (
                    player.entity.isDead,
                    player.isElytraFlying(),
                    [
                        player.entity.posX as f32,
                        player.entity.posY as f32,
                        player.entity.posZ as f32,
                    ],
                    [player.entity.motionX, player.entity.motionY, player.entity.motionZ],
                ))),
            _ => None,
        };

        let (playerDead, elytraFlying, position, motion) = snapshot.unwrap_or((
            true, false, [0.0; 3], [0.0; 3],
        ));
        if elytraFlying && !self.wasElytraFlying {
            self.elytraSounds.push(ElytraSound::new(&mut self.soundHandler, position));
        }
        self.wasElytraFlying = elytraFlying;

        for sound in &mut self.elytraSounds {
            sound.update(
                playerDead,
                elytraFlying,
                position,
                motion,
                &mut self.soundHandler,
            );
        }
        self.elytraSounds.retain(|sound| !sound.isDonePlaying());
    }

    fn ambientMusicType(&self) -> MusicType {
        let connection = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. }
            | ActiveGuiScreen::DownloadTerrain { connection, .. } => connection,
            _ => return MusicType::Menu,
        };
        connection.getSharedPlayState().withRead(|state| {
            let Some(world) = state.worldClient.as_ref() else {
                return MusicType::Menu;
            };
            match world.getDimension() {
                -1 => MusicType::Nether,
                1 if self.worldRenderer.shouldPlayEndBossMusic() => MusicType::EndBoss,
                1 => MusicType::End,
                _ => state.thePlayer.as_ref().map_or(MusicType::Game, |player| {
                    if player.capabilities.isCreativeMode && player.capabilities.allowFlying {
                        MusicType::Creative
                    } else {
                        MusicType::Game
                    }
                }),
            }
        })
    }

    fn updateScreen(
        &mut self,
        forceSprint: bool,
        chatWidth: f32,
        chatScale: f32,
        particleSetting: i32,
        controlHeld: bool,
        showSubtitles: bool,
    ) -> (bool, Option<RuntimeGuiAction>) {
        // Minecraft#runTick decrements this before processing GUI, network and
        // key bindings. Keep the same order so a held use binding becomes due
        // on the exact fourth client tick after the prior rightClickMouse call.
        self.rightClickDelayTimer = tick_right_click_delay(self.rightClickDelayTimer);

        let ambientMusicType = self.ambientMusicType();
        self.musicTicker.update(ambientMusicType, &mut self.soundHandler);
        self.soundHandler.update();

        // Packet handlers mutate EntityPlayer.openContainer independently of
        // this winit-owned GUI runtime. Synchronize before ticking input so a
        // server-opened menu immediately becomes the current input owner.
        self.syncOpenContainerGui();
        self.syncInventoryGameType();
        self.syncRecipeBookGui(false);
        self.openPendingSignEditor();
        if self.inventoryOpen {
            self.guiInventory.recipeBook.tick(1.0, controlHeld);
        } else if let Some(DedicatedContainerGui::Crafting(gui)) = self.guiDedicated.as_mut() {
            gui.recipeBook.tick(1.0, controlHeld);
        }
        if matches!(self.guiDedicated.as_ref(), Some(DedicatedContainerGui::Enchantment(_))) {
            let enchantmentState = match &self.currentScreen {
                ActiveGuiScreen::World { connection, .. } => connection
                    .getSharedPlayState()
                    .withRead(|state| {
                        let container = state.thePlayer.as_ref()?.openContainer.as_ref()?;
                        if container.windowKind() != Some(ContainerWindowKind::Enchantment) {
                            return None;
                        }
                        Some((
                            container.getSlot(0).cloned().unwrap_or(ItemStack::EMPTY),
                            [
                                container.properties().get(0).copied().unwrap_or(0),
                                container.properties().get(1).copied().unwrap_or(0),
                                container.properties().get(2).copied().unwrap_or(0),
                            ],
                        ))
                    }),
                _ => None,
            };
            if let (Some(gui), Some((inputStack, enchantLevels))) =
                (self.guiDedicated.as_mut(), enchantmentState)
            {
                gui.tickEnchantmentBook(&inputStack, &enchantLevels);
            }
        }
        let modalWorldGuiOpen = self.isModalWorldGuiOpen();

        // Minecraft#processKeyBinds performs the held-use repeat before
        // sendClickBlockToController and before the player/world tick can finish
        // an active item. Preserve that ordering so a completed food/bow use is
        // not restarted one client tick too early.
        let (inWorld, handActive) = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. } => {
                let active = connection.getSharedPlayState().withRead(|state| {
                    state.thePlayer.as_ref().is_some_and(|player| player.isHandActive())
                });
                (true, active)
            }
            _ => (false, false),
        };
        let mut heldUseRedraw = false;
        if held_right_click_due(
            inWorld,
            self.useButtonDown,
            self.rightClickDelayTimer,
            handActive,
            self.playerController.getIsHittingBlock(),
            modalWorldGuiOpen,
        ) {
            match self.worldActionButton(KeyBindingId::UseItem, true) {
                Ok(sent) => heldUseRedraw = sent,
                Err(message) => {
                    return (true, Some(RuntimeGuiAction::OpenDisconnected {
                        reasonKey: "connect.failed",
                        message,
                    }));
                }
            }
        }

        let mut pendingNetworkSounds = Vec::new();
        let mut pendingLocalSounds = Vec::new();
        let mut pendingWorldEffects = Vec::new();
        let (mut redraw, action) = match &mut self.currentScreen {
            ActiveGuiScreen::AccountManager(screen) => {
                let session = screen.updateScreen(&mut self.accountConfig);
                let avatarTextures = screen.takePendingAvatarTextures();
                for (location, image) in avatarTextures {
                    self.guiRenderer.registerDynamicTexture(location, image);
                }
                (true, session.map(|session| RuntimeGuiAction::AccountAuthenticated { session, returnToManager: false }))
            }
            ActiveGuiScreen::MicrosoftAuth(screen) => {
                let action = screen.updateScreen(&mut self.accountConfig).and_then(|action| match action {
                    GuiMicrosoftAuthAction::Authenticated(session) => Some(RuntimeGuiAction::AccountAuthenticated { session, returnToManager: true }),
                    _ => None,
                });
                (true, action)
            }
            ActiveGuiScreen::SessionLogin(screen) => {
                let action = screen.updateScreen(&mut self.accountConfig).and_then(|action| match action {
                    GuiSessionLoginAction::Authenticated(session) => Some(RuntimeGuiAction::AccountAuthenticated { session, returnToManager: false }),
                    _ => None,
                });
                (true, action)
            }
            ActiveGuiScreen::OfflineLogin(screen) => {
                let session = screen.updateScreen();
                (true, session.map(|session| RuntimeGuiAction::AccountAuthenticated { session, returnToManager: false }))
            },
            ActiveGuiScreen::ShaderSettings(screen) => (screen.updateScreen(), None),
            ActiveGuiScreen::Multiplayer(screen) => (screen.updateScreen(), None),
            ActiveGuiScreen::CreateWorld(screen) => { screen.updateScreen(); (true, None) }
            ActiveGuiScreen::AddServer { screen, .. } => { screen.updateScreen(); (true, None) }
            ActiveGuiScreen::DirectConnect { screen, .. } => { screen.updateScreen(); (true, None) }
            ActiveGuiScreen::Connecting { screen, .. } => {
                let mut redraw = false;
                let mut action = None;
                for event in screen.updateScreen() {
                    redraw = true;
                    match event {
                        GuiConnectingEvent::Authorizing
                        | GuiConnectingEvent::CompressionEnabled(_)
                        | GuiConnectingEvent::LoginSuccess(_)
                        | GuiConnectingEvent::TerrainReady
                        | GuiConnectingEvent::Respawn { .. }
                        | GuiConnectingEvent::PlayerDied(_) => {}
                        GuiConnectingEvent::WorldEffect { effectType, position, data, serverWide } => {
                            pendingWorldEffects.push((effectType, position, data, serverWide));
                        }
                        GuiConnectingEvent::Sound { sound, category, x, y, z, volume, pitch } => {
                            pendingNetworkSounds.push((sound, category, x, y, z, volume, pitch));
                        }
                        GuiConnectingEvent::JoinGame(_) => { action = Some(RuntimeGuiAction::OpenDownloadTerrain); break; }
                        GuiConnectingEvent::Disconnected(message) => { action = Some(RuntimeGuiAction::OpenDisconnected { reasonKey: "connect.failed", message }); break; }
                        GuiConnectingEvent::Failed { reasonKey, message } => { action = Some(RuntimeGuiAction::OpenDisconnected { reasonKey, message }); break; }
                        GuiConnectingEvent::Cancelled => { action = Some(RuntimeGuiAction::ReturnToMultiplayer { lastServer: None }); break; }
                    }
                }
                (redraw, action)
            }
            ActiveGuiScreen::DownloadTerrain { connection, .. } => {
                let mut action = None;
                for event in connection.updateScreen() {
                    match event {
                        GuiConnectingEvent::Disconnected(message) => { action = Some(RuntimeGuiAction::OpenDisconnected { reasonKey: "connect.failed", message }); break; }
                        GuiConnectingEvent::Failed { reasonKey, message } => { action = Some(RuntimeGuiAction::OpenDisconnected { reasonKey, message }); break; }
                        GuiConnectingEvent::TerrainReady => { action = Some(RuntimeGuiAction::OpenWorld); break; }
                        GuiConnectingEvent::Cancelled => { action = Some(RuntimeGuiAction::ReturnToMultiplayer { lastServer: None }); break; }
                        GuiConnectingEvent::Sound { sound, category, x, y, z, volume, pitch } => {
                            pendingNetworkSounds.push((sound, category, x, y, z, volume, pitch));
                        }
                        GuiConnectingEvent::WorldEffect { effectType, position, data, serverWide } => {
                            pendingWorldEffects.push((effectType, position, data, serverWide));
                        }
                        _ => {}
                    }
                }
                (action.is_some(), action)
            }
            ActiveGuiScreen::World { connection, .. } => {
                let mut action = None;
                for event in connection.updateScreen() {
                    match event {
                        GuiConnectingEvent::Disconnected(message) => { action = Some(RuntimeGuiAction::OpenDisconnected { reasonKey: "connect.failed", message }); break; }
                        GuiConnectingEvent::Failed { reasonKey, message } => { action = Some(RuntimeGuiAction::OpenDisconnected { reasonKey, message }); break; }
                        GuiConnectingEvent::Cancelled => { action = Some(RuntimeGuiAction::ReturnToMultiplayer { lastServer: None }); break; }
                        GuiConnectingEvent::Respawn { dimensionChanged: true, .. } => { action = Some(RuntimeGuiAction::OpenDownloadTerrain); break; }
                        GuiConnectingEvent::PlayerDied(message) => {
                            action = Some(RuntimeGuiAction::OpenGameOver(message));
                            break;
                        }
                        GuiConnectingEvent::Sound { sound, category, x, y, z, volume, pitch } => {
                            pendingNetworkSounds.push((sound, category, x, y, z, volume, pitch));
                        }
                        GuiConnectingEvent::WorldEffect { effectType, position, data, serverWide } => {
                            pendingWorldEffects.push((effectType, position, data, serverWide));
                        }
                        _ => {}
                    }
                }

                if action.is_none() {
                    match self.worldGuiScreen.as_mut() {
                        Some(WorldGuiScreen::IngameMenu(screen)) => screen.updateScreen(),
                        Some(WorldGuiScreen::ShaderSettings(screen)) => { screen.updateScreen(); }
                        Some(WorldGuiScreen::EditSign(screen)) => screen.updateScreen(),
                        Some(WorldGuiScreen::GameOver(screen)) => screen.updateScreen(),
                        Some(WorldGuiScreen::GameOverConfirm { screen, .. }) => screen.updateScreen(),
                        _ => {}
                    }
                    // Keep the edited text mirrored into the live tile entity after
                    // `GuiEditSign#updateScreen`, without re-borrowing all of `self`
                    // while `currentScreen` is mutably matched.
                    if let Some(WorldGuiScreen::EditSign(screen)) = self.worldGuiScreen.as_ref() {
                        let snapshot = screen.clone();
                        let shared = connection.getSharedPlayState();
                        shared.withWrite(|state| {
                            let Some(world) = state.worldClient.as_mut() else { return; };
                            let sign = world.getOrCreateSignTileEntity(snapshot.getPosition());
                            snapshot.applyToTileEntity(sign);
                            state.revision = state.revision.wrapping_add(1);
                        });
                    }
                    // `Minecraft.runTick` updates GuiIngame before packet work.
                    // Doing this first prevents a freshly received title or
                    // ActionBar packet from losing one tick immediately.
                    self.worldRenderer.tickIngameGui();
                    let shared = connection.getSharedPlayState();
                    let systemTimeMillis = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
                    for packet in shared.takeTitlePackets() {
                        self.worldRenderer.handleTitle(&packet);
                    }
                    for packet in shared.takeBossInfoPackets() {
                        self.worldRenderer.handleBossInfo(&packet, systemTimeMillis);
                    }
                    let completionBatches = shared.takeTabCompleteMatches();
                    if let Some(chat) = self.guiChat.as_mut() {
                        for matches in completionBatches {
                            if let Some(line) = chat.setCompletions(&matches, &self.fontRendererObj) {
                                let wrapWidth = ((GuiNewChat::calculateChatboxWidth(chatWidth.clamp(0.0, 1.0)) as f32)
                                    / chatScale.max(0.01)).floor() as i32;
                                let ticks = shared.withRead(|state| state.thePlayer.as_ref().map_or(0, |player| player.entity.ticksExisted));
                                self.worldRenderer.showChatCompletionCandidates(line, ticks, wrapWidth);
                            }
                        }
                        chat.updateScreen();
                    }
                    if self.creativeInventoryOpen {
                        self.guiCreative.updateCursorCounter();
                    }
                    if let Some(field) = self
                        .guiDedicated
                        .as_mut()
                        .and_then(DedicatedContainerGui::repairNameFieldMut)
                    {
                        field.updateCursorCounter();
                    }
                    let shared = connection.getSharedPlayState();
                    // GuiContainer/GuiChat do not allow user input in 1.12.2.
                    // The player still receives its normal client tick, but
                    // no movement key (including the opt-in force-sprint
                    // binding) is allowed to leak through the current screen.
                    let mut movementKeys = if modalWorldGuiOpen {
                        MovementKeyState::default()
                    } else {
                        self.movementKeys
                    };
                    // User-requested extension: logically hold the existing
                    // vanilla sprint binding. EntityPlayerSP still executes
                    // every original food, collision and item-use condition.
                    if !modalWorldGuiOpen { movementKeys.sprint |= forceSprint; }
                    let mut packets = shared.tickLocalPlayer(movementKeys);
                    let (particleSpawns, particleViewPosition) = shared.withWrite(|state| {
                        let requests = state.worldClient.as_mut().map_or_else(Vec::new, |world| world.takeParticleSpawns());
                        let position = [state.playerPosition.posX, state.playerPosition.posY, state.playerPosition.posZ];
                        (requests, position)
                    });
                    self.particleManager.spawnEffects(
                        particleSpawns,
                        particleViewPosition,
                        particleSetting,
                    );
                    shared.withRead(|state| {
                        if let Some(world) = state.worldClient.as_ref() {
                            self.particleManager.updateEffects(world);
                        }
                    });
                    if let Some((mainHand, offHand, cooledAttackStrength)) = shared.heldItemState() {
                        self.itemRenderer.updateEquippedItem(
                            &mainHand,
                            &offHand,
                            shared.localPlayerIsRowingBoat(),
                            cooledAttackStrength,
                        );
                        if let Some((active, hand, activeStack, useCount)) = shared.activeItemState() {
                            self.itemRenderer.setActiveItemState(active, hand, &activeStack, useCount);
                        }
                    } else {
                        self.itemRenderer.clear();
                    }

                    // MCP `Minecraft.sendClickBlockToController`: while the
                    // attack binding remains held, advance block damage once
                    // per 20 TPS client tick. Entity hits remain click-only.
                    let mut pendingContinuousBlockDestruction = None;
                    if self.attackButtonDown && !modalWorldGuiOpen {
                        let (mut miningPackets, swing, hitEffect) = shared.withRead(|state| {
                            self.playerController.setGameType(state.gameType);
                            let (Some(world), Some(player)) = (&state.worldClient, &state.thePlayer) else {
                                return (Vec::new(), false, None);
                            };
                            let reach = self.playerController.getBlockReachDistance();
                            let blockHit = player.rayTrace(world, reach, 1.0);
                            let eye = player.getPositionEyes(1.0);
                            let look = player.getLook(1.0);
                            let blockDistance = blockHit.as_ref().map_or(reach, |hit| eye.distance_to(hit.hitVec));
                            let entityHit = world.rayTraceEntities(
                                player.entityId, player.entity.ridingEntityId,
                                player.entity.boundingBox, eye, look,
                                if self.playerController.extendedReach() { 6.0 } else { reach },
                                blockDistance, self.playerController.extendedReach(),
                            );
                            if entityHit.is_some() {
                                return (self.playerController.resetBlockRemoving(), false, None);
                            }
                            match blockHit {
                                Some(result) if result.typeOfHit == RayTraceType::Block => {
                                    let position = result.getBlockPos();
                                    let side = result.sideHit;
                                    let (packets, swing) = self.playerController.onPlayerDamageBlock(
                                        world, player, position, side,
                                    );
                                    (packets, swing, swing.then_some((position, side)))
                                }
                                _ => {
                                    let packets = self.playerController.resetBlockRemoving();
                                    (packets, false, None)
                                }
                            }
                        });
                        packets.append(&mut miningPackets);
                        if let Some((position, side)) = hitEffect {
                            shared.withRead(|state| {
                                if let Some(world) = state.worldClient.as_ref() {
                                    self.particleManager.addBlockHitEffects(world, position, side);
                                }
                            });
                        }
                        pendingContinuousBlockDestruction =
                            self.playerController.takeBlockDestroyEffect();
                        if let Some(sound) = self.playerController.takeBlockHitSound() {
                            let _ = shared.queueLocalPlayerSound(sound);
                        }
                        if swing {
                            shared.swingLocalArm(EnumHand::MainHand);
                            packets.push(CPacketAnimation::new(EnumHand::MainHand).writePacketData());
                        }
                    }

                    if let Some(slot) = shared.currentHotbarSlot() {
                        if let Some(packet) = self.playerController.syncCurrentPlayItem(slot) {
                            packets.insert(0, packet);
                        }
                    }
                    match connection.sendPlayPackets(packets) {
                        Err(message) => {
                            action = Some(RuntimeGuiAction::OpenDisconnected {
                                reasonKey: "connect.failed",
                                message,
                            });
                        }
                        Ok(()) => {
                            if let Some((position, blockState)) = pendingContinuousBlockDestruction {
                                // `onPlayerDamageBlock` queues STOP_DESTROY_BLOCK
                                // before calling `onPlayerDestroyBlock`. Reproduce
                                // that ordering for held mining as well as the
                                // initial-click hardness>=1 path.
                                shared.withRead(|state| {
                                    if let Some(world) = state.worldClient.as_ref() {
                                        self.particleManager.addBlockDestroyEffects(
                                            world,
                                            position,
                                            blockState,
                                        );
                                    }
                                });
                                let _ = shared.applyPredictedBlockDestruction(position, blockState);
                            }
                        }
                    }
                }

                pendingLocalSounds.extend(
                    connection.getSharedPlayState().takeLocalPlayerSoundEvents()
                );
                let revision = connection.getSharedPlayState().revision();
                let redraw = revision != self.lastWorldRevision
                    || action.is_some()
                    || self.guiChat.is_some()
                    || self.worldGuiScreen.is_some();
                self.lastWorldRevision = revision;
                (redraw, action)
            }
            _ => (false, None),
        };
        for sound in pendingLocalSounds {
            self.soundHandler.playSound(sound.intoRecord());
        }
        for (sound, category, x, y, z, volume, pitch) in pendingNetworkSounds {
            self.soundHandler.playSound(PositionedSoundRecord::new(
                sound,
                category,
                volume,
                pitch,
                false,
                0,
                AttenuationType::Linear,
                [x as f32, y as f32, z as f32],
            ));
        }
        let viewPosition = match &self.currentScreen {
            ActiveGuiScreen::World { connection, .. }
            | ActiveGuiScreen::DownloadTerrain { connection, .. } => connection
                .getSharedPlayState()
                .withRead(|state| state.thePlayer.as_ref().map(|player| [
                    player.entity.posX, player.entity.posY, player.entity.posZ,
                ])),
            _ => None,
        };
        for (effectType, position, data, serverWide) in pendingWorldEffects {
            if let Some(action) = world_event_audio(
                effectType, position, data, serverWide, viewPosition, &mut self.worldEventRandom,
            ) {
                match action {
                    WorldEventAudio::Play(record) => {
                        if record.category == SoundCategory::Records {
                            self.soundHandler.stopRecordAt(record.position);
                        }
                        self.soundHandler.playSound(record);
                    }
                    WorldEventAudio::StopRecord(position) => self.soundHandler.stopRecordAt(position),
                }
            }
        }
        self.updateElytraSounds();

        let soundEvents = self.soundHandler.takeSoundPlayEvents();
        if showSubtitles && self.isWorld() {
            for event in soundEvents {
                let subtitle = self.locale.translate_key(&event.subtitle).to_owned();
                let startedAtMillis = event.startedAtMillis.min(u64::MAX as u128) as u64;
                self.worldRenderer.soundPlay(
                    subtitle,
                    event.position,
                    startedAtMillis,
                );
            }
        }

        redraw |= heldUseRedraw;
        (redraw, action)
    }

    fn mouseClicked(
        &mut self,
        framebufferWidth: u32,
        framebufferHeight: u32,
        mouseButton: i32,
        shiftDown: bool,
        settings: &GameSettings,
        currentAccessToken: &str,
    ) -> Option<RuntimeGuiAction> {
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        let worldGuiOpen = self.isWorldGuiOpen();
        if mouseButton != 0 {
            let multiButtonGuiOpen = if worldGuiOpen {
                matches!(
                    self.worldGuiScreen.as_ref(),
                    Some(WorldGuiScreen::ShaderSettings(_)) | Some(WorldGuiScreen::Controls(_))
                )
            } else {
                matches!(
                    &self.currentScreen,
                    ActiveGuiScreen::ShaderSettings(_) | ActiveGuiScreen::Controls(_)
                )
            };
            if !multiButtonGuiOpen {
                return None;
            }
        }
        let soundHandler = &mut self.soundHandler;
        if worldGuiOpen {
            return match self.worldGuiScreen.as_mut() {
                Some(WorldGuiScreen::IngameMenu(screen)) => screen
                    .mouseClicked(mouseX, mouseY, 0)
                    .map(|interaction| {
                        playGuiSound(soundHandler, Some(&interaction.sound));
                        match interaction.action {
                            GuiIngameMenuAction::ReturnToGame => RuntimeGuiAction::ResumeWorld,
                            GuiIngameMenuAction::Options => RuntimeGuiAction::OpenWorldOptions,
                            GuiIngameMenuAction::Disconnect => RuntimeGuiAction::DisconnectWorld,
                            GuiIngameMenuAction::Advancements => RuntimeGuiAction::NotConnected("GuiScreenAdvancements"),
                            GuiIngameMenuAction::Statistics => RuntimeGuiAction::NotConnected("GuiStats"),
                            GuiIngameMenuAction::ShareToLan => RuntimeGuiAction::NotConnected("GuiShareToLan"),
                        }
                    }),
                Some(WorldGuiScreen::Options(screen)) => screen
                    .mouseClicked(mouseX, mouseY, 0, &self.locale)
                    .map(|interaction| {
                        playGuiSound(soundHandler, interaction.sound.as_ref());
                        match interaction.action {
                            GuiOptionsAction::Done => RuntimeGuiAction::ReturnToIngameMenu,
                            GuiOptionsAction::SetFov(value) => RuntimeGuiAction::SetFov(value),
                            GuiOptionsAction::ToggleForceSprint => RuntimeGuiAction::ToggleForceSprint,
                            GuiOptionsAction::OpenVideoSettings => RuntimeGuiAction::OpenWorldVideoSettings,
                            GuiOptionsAction::OpenLanguage => RuntimeGuiAction::OpenWorldLanguage,
                            GuiOptionsAction::OpenControls => RuntimeGuiAction::OpenWorldControls,
                            GuiOptionsAction::OpenSkinCustomisation => RuntimeGuiAction::OpenWorldSkinSettings,
                            GuiOptionsAction::OpenSounds => RuntimeGuiAction::OpenWorldSoundSettings,
                            GuiOptionsAction::OpenChatSettings => RuntimeGuiAction::OpenWorldChatSettings,
                            GuiOptionsAction::OpenResourcePacks => RuntimeGuiAction::OpenWorldResourcePacks,
                            GuiOptionsAction::OpenSnooper => RuntimeGuiAction::NotConnected("GuiSnooper"),
                        }
                    }),
                Some(WorldGuiScreen::Controls(screen)) => mouse_button_from_index(mouseButton)
                    .and_then(|button| screen.mouseClicked(mouseX, mouseY, button, &self.locale, settings))
                    .map(|interaction| {
                        playGuiSound(soundHandler, interaction.sound.as_ref());
                        mapControlsAction(interaction.action, true)
                    }),
                Some(WorldGuiScreen::VideoSettings(screen)) => screen
                    .mouseClicked(mouseX, mouseY, 0, &self.locale, settings)
                    .map(|interaction| {
                        playGuiSound(soundHandler, interaction.sound.as_ref());
                        match interaction.action {
                            GuiVideoSettingsAction::Done => RuntimeGuiAction::ReturnToWorldOptions,
                            GuiVideoSettingsAction::OpenShaders => RuntimeGuiAction::OpenWorldShaderSettings,
                            other => mapVideoSettingsAction(other),
                        }
                    }),
                Some(WorldGuiScreen::ShaderSettings(screen)) => screen
                    .mouseClicked(mouseX, mouseY, mouseButton, shiftDown)
                    .map(|interaction| {
                        playGuiSound(soundHandler, Some(&interaction.sound));
                        match interaction.action {
                            GuiShaderAction::None => RuntimeGuiAction::None,
                            GuiShaderAction::SelectShaderPack(name) => RuntimeGuiAction::SelectShaderPack(name),
                            GuiShaderAction::ReloadShaderPack => RuntimeGuiAction::ReloadShaderPack,
                            GuiShaderAction::OpenShaderPacksFolder => RuntimeGuiAction::OpenShaderPackFolder,
                            GuiShaderAction::Done => RuntimeGuiAction::ReturnToWorldVideoSettings,
                        }
                    }),
                Some(WorldGuiScreen::SoundSettings(screen)) => screen.mouseClicked(mouseX, mouseY, 0, &self.locale, settings).map(|interaction| {
                    playGuiSound(soundHandler, interaction.sound.as_ref());
                    match interaction.action { GuiScreenOptionsSoundsAction::Done => RuntimeGuiAction::ReturnToWorldOptions, other => mapSoundSettingsAction(other) }
                }),
                Some(WorldGuiScreen::ChatSettings(screen)) => screen.mouseClicked(mouseX, mouseY, 0, &self.locale, settings).map(|interaction| {
                    playGuiSound(soundHandler, interaction.sound.as_ref());
                    match interaction.action { ScreenChatOptionsAction::Done => RuntimeGuiAction::ReturnToWorldOptions, other => mapChatSettingsAction(other) }
                }),
                Some(WorldGuiScreen::SkinSettings(screen)) => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                    playGuiSound(soundHandler, Some(&interaction.sound));
                    match interaction.action { GuiCustomizeSkinAction::Done => RuntimeGuiAction::ReturnToWorldOptions, GuiCustomizeSkinAction::TogglePart(part) => RuntimeGuiAction::ToggleModelPart(part), GuiCustomizeSkinAction::ToggleMainHand => RuntimeGuiAction::ToggleMainHand }
                }),
                Some(WorldGuiScreen::ResourcePacks(screen)) => {
                    let interaction = screen.mouseClicked(mouseX, mouseY, 0);
                    if let Some(interaction) = interaction {
                        playGuiSound(soundHandler, Some(&interaction.sound));
                        Some(match interaction.action {
                            GuiScreenResourcePacksAction::Done if screen.hasChanges() => RuntimeGuiAction::ApplyResourcePacks { selected: screen.selected(), world: true },
                            GuiScreenResourcePacksAction::Done => RuntimeGuiAction::ReturnToWorldOptions,
                            GuiScreenResourcePacksAction::OpenFolder => RuntimeGuiAction::OpenResourcePackFolder,
                            GuiScreenResourcePacksAction::Toggle(_) | GuiScreenResourcePacksAction::MoveSelected { .. } => RuntimeGuiAction::None,
                        })
                    } else if screen.isDraggingScrollbar() {
                        Some(RuntimeGuiAction::None)
                    } else {
                        None
                    }
                },
                Some(WorldGuiScreen::Language(screen)) => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                    playGuiSound(soundHandler, interaction.sound.as_ref());
                    match interaction.action {
                        GuiLanguageAction::Done => RuntimeGuiAction::ReturnToWorldOptions,
                        GuiLanguageAction::ToggleUnicode => RuntimeGuiAction::ToggleUnicode,
                        GuiLanguageAction::SelectLanguage(code) => RuntimeGuiAction::SetLanguage(code),
                    }
                }),
                Some(WorldGuiScreen::EditSign(screen)) => screen
                    .mouseClicked(mouseX, mouseY, 0)
                    .map(|sound| {
                        playGuiSound(soundHandler, Some(&sound));
                        RuntimeGuiAction::FinishSignEditor
                    }),
                Some(WorldGuiScreen::GameOver(screen)) => screen
                    .mouseClicked(mouseX, mouseY, 0)
                    .map(|interaction| {
                        playGuiSound(soundHandler, Some(&interaction.sound));
                        match interaction.action {
                            GuiGameOverAction::Respawn => RuntimeGuiAction::RespawnPlayer,
                            GuiGameOverAction::Quit if screen.isHardcore() => RuntimeGuiAction::LeaveWorldToMainMenu,
                            GuiGameOverAction::Quit => RuntimeGuiAction::OpenDeathQuitConfirm,
                        }
                    }),
                Some(WorldGuiScreen::GameOverConfirm { screen, .. }) => screen
                    .mouseClicked(mouseX, mouseY, 0)
                    .map(|interaction| {
                        playGuiSound(soundHandler, Some(&interaction.sound));
                        RuntimeGuiAction::ConfirmDeathQuit(interaction.result)
                    }),
                None => None,
            };
        }
        match &mut self.currentScreen {
            ActiveGuiScreen::Empty | ActiveGuiScreen::World { .. } => None,
            ActiveGuiScreen::MainMenu(screen) => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    MainMenuAction::OpenOptions => RuntimeGuiAction::Switch(ScreenId::Options),
                    MainMenuAction::OpenLanguage => RuntimeGuiAction::OpenLanguage(ScreenId::MainMenu),
                    MainMenuAction::OpenWorldSelection => RuntimeGuiAction::Switch(ScreenId::WorldSelection),
                    MainMenuAction::OpenMultiplayer => RuntimeGuiAction::Switch(ScreenId::Multiplayer),
                    MainMenuAction::OpenAccounts => RuntimeGuiAction::OpenAccountManager { notification: None },
                    MainMenuAction::Shutdown => RuntimeGuiAction::Shutdown,
                    MainMenuAction::OpenCopyrightCredits => RuntimeGuiAction::NotConnected("GuiWinGame"),
                    MainMenuAction::OpenCompatibilityWarning { .. } => RuntimeGuiAction::NotConnected("GuiConfirmOpenLink"),
                }
            }),
            ActiveGuiScreen::AccountManager(screen) => screen.mouseClicked(mouseX, mouseY, mouseButton, &mut self.accountConfig, currentAccessToken).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    GuiAccountManagerAction::Back => RuntimeGuiAction::Switch(ScreenId::MainMenu),
                    GuiAccountManagerAction::OpenMicrosoft => RuntimeGuiAction::OpenMicrosoftAuth,
                    GuiAccountManagerAction::OpenOffline => RuntimeGuiAction::OpenOfflineLogin,
                    GuiAccountManagerAction::OpenToken => RuntimeGuiAction::OpenSessionLogin,
                    GuiAccountManagerAction::None => RuntimeGuiAction::None,
                }
            }),
            ActiveGuiScreen::MicrosoftAuth(screen) => screen.mouseClicked(mouseX, mouseY, mouseButton).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    GuiMicrosoftAuthAction::Cancel => RuntimeGuiAction::OpenAccountManager { notification: None },
                    GuiMicrosoftAuthAction::Authenticated(session) => RuntimeGuiAction::AccountAuthenticated { session, returnToManager: true },
                    GuiMicrosoftAuthAction::None => RuntimeGuiAction::None,
                }
            }),
            ActiveGuiScreen::SessionLogin(screen) => screen.mouseClicked(mouseX, mouseY, mouseButton, &self.fontRendererObj).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    GuiSessionLoginAction::Cancel => RuntimeGuiAction::OpenAccountManager { notification: None },
                    GuiSessionLoginAction::Authenticated(session) => RuntimeGuiAction::AccountAuthenticated { session, returnToManager: false },
                    GuiSessionLoginAction::None => RuntimeGuiAction::None,
                }
            }),
            ActiveGuiScreen::OfflineLogin(screen) => screen.mouseClicked(mouseX, mouseY, mouseButton, &self.fontRendererObj).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    GuiAltCrackedAction::Cancel => RuntimeGuiAction::OpenAccountManager { notification: None },
                    GuiAltCrackedAction::Authenticated(session) => RuntimeGuiAction::AccountAuthenticated { session, returnToManager: false },
                    GuiAltCrackedAction::None => RuntimeGuiAction::None,
                }
            }),
            ActiveGuiScreen::Options(screen) => screen.mouseClicked(mouseX, mouseY, 0, &self.locale).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    GuiOptionsAction::Done => RuntimeGuiAction::Switch(ScreenId::MainMenu),
                    GuiOptionsAction::OpenLanguage => RuntimeGuiAction::OpenLanguage(ScreenId::Options),
                    GuiOptionsAction::SetFov(value) => RuntimeGuiAction::SetFov(value),
                    GuiOptionsAction::ToggleForceSprint => RuntimeGuiAction::ToggleForceSprint,
                    GuiOptionsAction::OpenVideoSettings => RuntimeGuiAction::OpenVideoSettings,
                    GuiOptionsAction::OpenControls => RuntimeGuiAction::OpenControls,
                    GuiOptionsAction::OpenSkinCustomisation => RuntimeGuiAction::OpenSkinSettings,
                    GuiOptionsAction::OpenSounds => RuntimeGuiAction::OpenSoundSettings,
                    GuiOptionsAction::OpenChatSettings => RuntimeGuiAction::OpenChatSettings,
                    GuiOptionsAction::OpenResourcePacks => RuntimeGuiAction::OpenResourcePacks,
                    GuiOptionsAction::OpenSnooper => RuntimeGuiAction::NotConnected("GuiSnooper"),
                }
            }),
            ActiveGuiScreen::Controls(screen) => mouse_button_from_index(mouseButton)
                .and_then(|button| screen.mouseClicked(mouseX, mouseY, button, &self.locale, settings))
                .map(|interaction| {
                    playGuiSound(soundHandler, interaction.sound.as_ref());
                    mapControlsAction(interaction.action, false)
                }),
            ActiveGuiScreen::VideoSettings(screen) => screen
                .mouseClicked(mouseX, mouseY, 0, &self.locale, settings)
                .map(|interaction| {
                    playGuiSound(soundHandler, interaction.sound.as_ref());
                    mapVideoSettingsAction(interaction.action)
                }),
            ActiveGuiScreen::ShaderSettings(screen) => screen
                .mouseClicked(mouseX, mouseY, mouseButton, shiftDown)
                .map(|interaction| {
                    playGuiSound(soundHandler, Some(&interaction.sound));
                    match interaction.action {
                        GuiShaderAction::None => RuntimeGuiAction::None,
                        GuiShaderAction::SelectShaderPack(name) => RuntimeGuiAction::SelectShaderPack(name),
                        GuiShaderAction::ReloadShaderPack => RuntimeGuiAction::ReloadShaderPack,
                        GuiShaderAction::OpenShaderPacksFolder => RuntimeGuiAction::OpenShaderPackFolder,
                        GuiShaderAction::Done => RuntimeGuiAction::ReturnToVideoSettings,
                    }
                }),
            ActiveGuiScreen::SoundSettings(screen) => screen.mouseClicked(mouseX, mouseY, 0, &self.locale, settings).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action { GuiScreenOptionsSoundsAction::Done => RuntimeGuiAction::ReturnToOptions, other => mapSoundSettingsAction(other) }
            }),
            ActiveGuiScreen::ChatSettings(screen) => screen.mouseClicked(mouseX, mouseY, 0, &self.locale, settings).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action { ScreenChatOptionsAction::Done => RuntimeGuiAction::ReturnToOptions, other => mapChatSettingsAction(other) }
            }),
            ActiveGuiScreen::SkinSettings(screen) => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                playGuiSound(soundHandler, Some(&interaction.sound));
                match interaction.action { GuiCustomizeSkinAction::Done => RuntimeGuiAction::ReturnToOptions, GuiCustomizeSkinAction::TogglePart(part) => RuntimeGuiAction::ToggleModelPart(part), GuiCustomizeSkinAction::ToggleMainHand => RuntimeGuiAction::ToggleMainHand }
            }),
            ActiveGuiScreen::ResourcePacks(screen) => {
                let interaction = screen.mouseClicked(mouseX, mouseY, 0);
                if let Some(interaction) = interaction {
                    playGuiSound(soundHandler, Some(&interaction.sound));
                    Some(match interaction.action {
                        GuiScreenResourcePacksAction::Done if screen.hasChanges() => RuntimeGuiAction::ApplyResourcePacks { selected: screen.selected(), world: false },
                        GuiScreenResourcePacksAction::Done => RuntimeGuiAction::ReturnToOptions,
                        GuiScreenResourcePacksAction::OpenFolder => RuntimeGuiAction::OpenResourcePackFolder,
                        GuiScreenResourcePacksAction::Toggle(_) | GuiScreenResourcePacksAction::MoveSelected { .. } => RuntimeGuiAction::None,
                    })
                } else if screen.isDraggingScrollbar() {
                    Some(RuntimeGuiAction::None)
                } else {
                    None
                }
            },
            ActiveGuiScreen::Multiplayer(screen) => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    GuiMultiplayerAction::Cancel => RuntimeGuiAction::Switch(ScreenId::MainMenu),
                    GuiMultiplayerAction::Refresh => { screen.refreshServerList(); RuntimeGuiAction::None }
                    GuiMultiplayerAction::DirectConnect => RuntimeGuiAction::OpenDirectConnect,
                    GuiMultiplayerAction::AddServer => RuntimeGuiAction::OpenAddServer,
                    GuiMultiplayerAction::Select(server) => RuntimeGuiAction::Connect(server),
                    GuiMultiplayerAction::Edit { index, server } => RuntimeGuiAction::OpenEditServer { index, server },
                    GuiMultiplayerAction::Delete { index, serverName } => RuntimeGuiAction::OpenDeleteConfirm { index, serverName },
                    GuiMultiplayerAction::SelectionChanged => RuntimeGuiAction::None,
                }
            }),
            ActiveGuiScreen::WorldSelection(screen) => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                playGuiSound(soundHandler, Some(&interaction.sound));
                match interaction.action {
                    GuiWorldSelectionAction::Cancel => RuntimeGuiAction::Switch(ScreenId::MainMenu),
                    GuiWorldSelectionAction::Create => RuntimeGuiAction::OpenCreateWorld,
                    GuiWorldSelectionAction::Select => {
                        if let Some(world)=screen.selectedWorld() { RuntimeGuiAction::JoinWorld { folderName: world.getFileName().to_owned(), worldName: world.getDisplayName().to_owned() } } else { RuntimeGuiAction::None }
                    },
                    GuiWorldSelectionAction::Edit => RuntimeGuiAction::NotConnected("GuiWorldEdit"),
                    GuiWorldSelectionAction::Delete => RuntimeGuiAction::NotConnected("GuiYesNo world delete"),
                    GuiWorldSelectionAction::Recreate => RuntimeGuiAction::NotConnected("GuiCreateWorld.recreateFromExistingWorld"),
                }
            }),
            ActiveGuiScreen::CreateWorld(screen) => screen.mouseClicked(
                mouseX, mouseY, 0, &self.fontRendererObj, &self.locale, shiftDown,
            ).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    GuiCreateWorldAction::None => RuntimeGuiAction::None,
                    GuiCreateWorldAction::Cancel => RuntimeGuiAction::Switch(ScreenId::WorldSelection),
                    GuiCreateWorldAction::Create(request) => RuntimeGuiAction::CreateWorld(request),
                    GuiCreateWorldAction::CustomizeFlat => RuntimeGuiAction::NotConnected("GuiCreateFlatWorld"),
                    GuiCreateWorldAction::CustomizeWorld => RuntimeGuiAction::NotConnected("GuiCustomizeWorldScreen"),
                }
            }),
            ActiveGuiScreen::Language { screen, parent } => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    GuiLanguageAction::Done => RuntimeGuiAction::Switch(*parent),
                    GuiLanguageAction::ToggleUnicode => RuntimeGuiAction::ToggleUnicode,
                    GuiLanguageAction::SelectLanguage(code) => RuntimeGuiAction::SetLanguage(code),
                }
            }),
            ActiveGuiScreen::AddServer { screen, editingIndex, .. } => screen.mouseClicked(mouseX, mouseY, 0, &self.fontRendererObj, &self.locale).map(|interaction| {
                playGuiSound(soundHandler, interaction.sound.as_ref());
                match interaction.action {
                    GuiScreenAddServerAction::Confirm(server) => RuntimeGuiAction::SaveServer { editingIndex: *editingIndex, server },
                    GuiScreenAddServerAction::Cancel => RuntimeGuiAction::ReturnToMultiplayer { lastServer: None },
                    GuiScreenAddServerAction::CycleResourceMode => RuntimeGuiAction::None,
                }
            }),
            ActiveGuiScreen::DirectConnect { screen, .. } => screen.mouseClicked(mouseX, mouseY, 0, &self.fontRendererObj).map(|interaction| {
                playGuiSound(soundHandler, Some(&interaction.sound));
                match interaction.action {
                    GuiScreenServerListAction::Confirm(server) => RuntimeGuiAction::Connect(server),
                    GuiScreenServerListAction::Cancel => RuntimeGuiAction::ReturnToMultiplayer { lastServer: Some(screen.getAddress()) },
                }
            }),
            ActiveGuiScreen::ConfirmDelete { screen, serverIndex, .. } => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                playGuiSound(soundHandler, Some(&interaction.sound));
                if interaction.result { RuntimeGuiAction::DeleteServer { index: *serverIndex } } else { RuntimeGuiAction::ReturnToMultiplayer { lastServer: None } }
            }),
            ActiveGuiScreen::Connecting { screen, .. } => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                playGuiSound(soundHandler, Some(&interaction.sound));
                match interaction.action { GuiConnectingAction::Cancel => RuntimeGuiAction::CancelConnecting }
            }),
            ActiveGuiScreen::Disconnected { screen, .. } => screen.mouseClicked(mouseX, mouseY, 0).map(|interaction| {
                playGuiSound(soundHandler, Some(&interaction.sound));
                match interaction.action { GuiDisconnectedAction::ToMenu => RuntimeGuiAction::ReturnToMultiplayer { lastServer: None } }
            }),
            ActiveGuiScreen::DownloadTerrain { .. } => None,
        }
    }

    fn mouseDragged(&mut self, framebufferWidth: u32, framebufferHeight: u32) -> Option<RuntimeGuiAction> {
        self.syncOpenContainerGui();
        if self.isInventoryOpen() {
            self.inventoryMouseDragged(framebufferWidth, framebufferHeight);
            return None;
        }
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        match self.worldGuiScreen.as_mut() {
            Some(WorldGuiScreen::Options(screen)) => return screen.mouseDragged(mouseX, &self.locale).map(|interaction| match interaction.action {
                GuiOptionsAction::SetFov(value) => RuntimeGuiAction::SetFov(value),
                _ => RuntimeGuiAction::None,
            }),
            Some(WorldGuiScreen::Controls(screen)) => return screen.mouseDragged(mouseX, mouseY, &self.locale)
                .map(|interaction| mapControlsAction(interaction.action, true)),
            Some(WorldGuiScreen::VideoSettings(screen)) => return screen.mouseDragged(mouseX, &self.locale).map(mapVideoSettingsAction),
            Some(WorldGuiScreen::SoundSettings(screen)) => return screen.mouseDragged(mouseX, &self.locale).map(mapSoundSettingsAction),
            Some(WorldGuiScreen::ChatSettings(screen)) => return screen.mouseDragged(mouseX, &self.locale).map(mapChatSettingsAction),
            Some(WorldGuiScreen::ShaderSettings(screen)) => {
                if screen.mouseDragged(mouseX) { return Some(RuntimeGuiAction::None); }
            }
            Some(WorldGuiScreen::ResourcePacks(screen)) => {
                if screen.mouseDragged(mouseY) {
                    return Some(RuntimeGuiAction::None);
                }
            }
            Some(WorldGuiScreen::Language(screen)) => {
                if screen.mouseDragged(mouseY) {
                    return Some(RuntimeGuiAction::None);
                }
            }
            _ => {}
        }
        match &mut self.currentScreen {
            ActiveGuiScreen::AccountManager(screen) => {
                if screen.mouseDragged(mouseY, self.accountConfig.len()) {
                    Some(RuntimeGuiAction::None)
                } else {
                    None
                }
            }
            ActiveGuiScreen::Options(screen) => screen.mouseDragged(mouseX, &self.locale).map(|interaction| match interaction.action {
                GuiOptionsAction::SetFov(value) => RuntimeGuiAction::SetFov(value),
                _ => RuntimeGuiAction::None,
            }),
            ActiveGuiScreen::Controls(screen) => screen.mouseDragged(mouseX, mouseY, &self.locale)
                .map(|interaction| mapControlsAction(interaction.action, false)),
            ActiveGuiScreen::VideoSettings(screen) => screen.mouseDragged(mouseX, &self.locale).map(mapVideoSettingsAction),
            ActiveGuiScreen::SoundSettings(screen) => screen.mouseDragged(mouseX, &self.locale).map(mapSoundSettingsAction),
            ActiveGuiScreen::ChatSettings(screen) => screen.mouseDragged(mouseX, &self.locale).map(mapChatSettingsAction),
            ActiveGuiScreen::ShaderSettings(screen) => {
                if screen.mouseDragged(mouseX) { Some(RuntimeGuiAction::None) } else { None }
            }
            ActiveGuiScreen::ResourcePacks(screen) => {
                if screen.mouseDragged(mouseY) {
                    Some(RuntimeGuiAction::None)
                } else {
                    None
                }
            }
            ActiveGuiScreen::Language { screen, .. } => {
                if screen.mouseDragged(mouseY) { Some(RuntimeGuiAction::None) } else { None }
            }
            _ => None,
        }
    }

    fn mouseReleased(&mut self, framebufferWidth: u32, framebufferHeight: u32) {
        let (mouseX, mouseY) = self.cursorGuiPosition(framebufferWidth, framebufferHeight);
        match self.worldGuiScreen.as_mut() {
            Some(WorldGuiScreen::Options(screen)) => { screen.mouseReleased(mouseX, mouseY); return; }
            Some(WorldGuiScreen::Controls(screen)) => { screen.mouseReleased(mouseX, mouseY); return; }
            Some(WorldGuiScreen::VideoSettings(screen)) => { screen.mouseReleased(mouseX, mouseY); return; }
            Some(WorldGuiScreen::SoundSettings(screen)) => {
                let sound = screen.mouseReleased(mouseX, mouseY);
                playGuiSound(&mut self.soundHandler, sound.as_ref());
                return;
            }
            Some(WorldGuiScreen::ChatSettings(screen)) => { screen.mouseReleased(mouseX, mouseY); return; }
            Some(WorldGuiScreen::ShaderSettings(screen)) => { screen.mouseReleased(); return; }
            Some(WorldGuiScreen::ResourcePacks(screen)) => { screen.mouseReleased(); return; }
            Some(WorldGuiScreen::Language(screen)) => { screen.mouseReleased(); return; }
            _ => {}
        }
        match &mut self.currentScreen {
            ActiveGuiScreen::AccountManager(screen) => screen.mouseReleased(),
            ActiveGuiScreen::Options(screen) => screen.mouseReleased(mouseX, mouseY),
            ActiveGuiScreen::Controls(screen) => screen.mouseReleased(mouseX, mouseY),
            ActiveGuiScreen::VideoSettings(screen) => screen.mouseReleased(mouseX, mouseY),
            ActiveGuiScreen::SoundSettings(screen) => {
                let sound = screen.mouseReleased(mouseX, mouseY);
                playGuiSound(&mut self.soundHandler, sound.as_ref());
            },
            ActiveGuiScreen::ChatSettings(screen) => screen.mouseReleased(mouseX, mouseY),
            ActiveGuiScreen::ShaderSettings(screen) => screen.mouseReleased(),
            ActiveGuiScreen::ResourcePacks(screen) => screen.mouseReleased(),
            ActiveGuiScreen::Language { screen, .. } => screen.mouseReleased(),
            _ => {}
        }
    }

    fn typedText(&mut self, text: &str) -> bool {
        if let Some(WorldGuiScreen::EditSign(screen)) = self.worldGuiScreen.as_mut() {
            return screen.typedText(text, &self.fontRendererObj);
        }
        match &mut self.currentScreen {
            ActiveGuiScreen::SessionLogin(screen) => screen.typedText(text, &self.fontRendererObj),
            ActiveGuiScreen::OfflineLogin(screen) => screen.typedText(text, &self.fontRendererObj),
            ActiveGuiScreen::AddServer { screen, .. } => screen.typedText(text, &self.fontRendererObj),
            ActiveGuiScreen::DirectConnect { screen, .. } => screen.typedText(text, &self.fontRendererObj),
            ActiveGuiScreen::CreateWorld(screen) => screen.typedText(text, &self.fontRendererObj),
            _ => false,
        }
    }

    fn keyPressed(&mut self, key: KeyCode, modifiers: ModifiersState, eventText: Option<&str>) -> Option<RuntimeGuiAction> {
        if let Some(WorldGuiScreen::Controls(screen)) = self.worldGuiScreen.as_mut() {
            return screen.keyPressed(key, eventText).map(|interaction| mapControlsAction(interaction.action, true));
        }
        if let Some(WorldGuiScreen::EditSign(screen)) = self.worldGuiScreen.as_mut() {
            let signKey = match key {
                KeyCode::ArrowUp => Some(SignEditKey::Up),
                KeyCode::ArrowDown => Some(SignEditKey::Down),
                KeyCode::Enter | KeyCode::NumpadEnter => Some(SignEditKey::Enter),
                KeyCode::Backspace => Some(SignEditKey::Backspace),
                KeyCode::Escape => Some(SignEditKey::Escape),
                _ => None,
            };
            if let Some(signKey) = signKey {
                screen.keyPressed(signKey);
                return Some(if screen.isDoneRequested() {
                    RuntimeGuiAction::FinishSignEditor
                } else {
                    RuntimeGuiAction::None
                });
            }
            return None;
        }
        let fieldModifiers = GuiTextFieldModifiers { control: modifiers.control_key(), shift: modifiers.shift_key() };
        let textKey = match key {
            KeyCode::Backspace => Some(GuiTextFieldKey::Backspace), KeyCode::Delete => Some(GuiTextFieldKey::Delete),
            KeyCode::ArrowLeft => Some(GuiTextFieldKey::Left), KeyCode::ArrowRight => Some(GuiTextFieldKey::Right),
            KeyCode::Home => Some(GuiTextFieldKey::Home), KeyCode::End => Some(GuiTextFieldKey::End), _ => None,
        };
        match &mut self.currentScreen {
            ActiveGuiScreen::Controls(screen) => screen.keyPressed(key, eventText)
                .map(|interaction| mapControlsAction(interaction.action, false)),
            ActiveGuiScreen::AccountManager(screen) => {
                let accountKey = match key {
                    KeyCode::ArrowUp => Some(AccountManagerKey::Up),
                    KeyCode::ArrowDown => Some(AccountManagerKey::Down),
                    KeyCode::Enter | KeyCode::NumpadEnter => Some(AccountManagerKey::Enter),
                    KeyCode::Delete => Some(AccountManagerKey::Delete),
                    KeyCode::KeyC if modifiers.control_key() => Some(AccountManagerKey::Copy),
                    _ => None,
                };
                accountKey.and_then(|value| screen.keyPressed(value, modifiers.control_key(), &mut self.accountConfig).map(|_| RuntimeGuiAction::None))
            }
            ActiveGuiScreen::SessionLogin(screen) => {
                if key == KeyCode::Enter || key == KeyCode::NumpadEnter { screen.enterPressed(); return Some(RuntimeGuiAction::None); }
                if modifiers.control_key() && key == KeyCode::KeyA { screen.selectAll(&self.fontRendererObj); return Some(RuntimeGuiAction::None); }
                textKey.and_then(|value| screen.keyPressed(value, fieldModifiers, &self.fontRendererObj).then_some(RuntimeGuiAction::None))
            }
            ActiveGuiScreen::OfflineLogin(screen) => {
                if key == KeyCode::Tab { screen.tabPressed(); return Some(RuntimeGuiAction::None); }
                if key == KeyCode::Enter || key == KeyCode::NumpadEnter {
                    return Some(match screen.enterPressed() {
                        GuiAltCrackedAction::Authenticated(session) => RuntimeGuiAction::AccountAuthenticated { session, returnToManager: false },
                        _ => RuntimeGuiAction::None,
                    });
                }
                if modifiers.control_key() && key == KeyCode::KeyA { screen.selectAll(&self.fontRendererObj); return Some(RuntimeGuiAction::None); }
                textKey.and_then(|value| screen.keyPressed(value, fieldModifiers, &self.fontRendererObj).then_some(RuntimeGuiAction::None))
            }
            ActiveGuiScreen::CreateWorld(screen) => {
                if key == KeyCode::Enter || key == KeyCode::NumpadEnter {
                    return screen.enterPressed().map(RuntimeGuiAction::CreateWorld);
                }
                if modifiers.control_key() && key == KeyCode::KeyA {
                    return screen.selectAll(&self.fontRendererObj).then_some(RuntimeGuiAction::None);
                }
                textKey.and_then(|value| screen.keyPressed(value, fieldModifiers, &self.fontRendererObj).then_some(RuntimeGuiAction::None))
            }
            ActiveGuiScreen::AddServer { screen, editingIndex, .. } => {
                if key == KeyCode::Tab { screen.tabPressed(); return Some(RuntimeGuiAction::None); }
                if key == KeyCode::Enter || key == KeyCode::NumpadEnter {
                    return screen.enterPressed().map(|action| match action { GuiScreenAddServerAction::Confirm(server) => RuntimeGuiAction::SaveServer { editingIndex: *editingIndex, server }, _ => RuntimeGuiAction::None });
                }
                if modifiers.control_key() && key == KeyCode::KeyA { screen.selectAll(&self.fontRendererObj); return Some(RuntimeGuiAction::None); }
                textKey.and_then(|value| screen.keyPressed(value, fieldModifiers, &self.fontRendererObj).then_some(RuntimeGuiAction::None))
            }
            ActiveGuiScreen::DirectConnect { screen, .. } => {
                if key == KeyCode::Enter || key == KeyCode::NumpadEnter {
                    return screen.enterPressed().map(|action| match action { GuiScreenServerListAction::Confirm(server) => RuntimeGuiAction::Connect(server), _ => RuntimeGuiAction::None });
                }
                if modifiers.control_key() && key == KeyCode::KeyA { screen.selectAll(&self.fontRendererObj); return Some(RuntimeGuiAction::None); }
                textKey.and_then(|value| screen.keyPressed(value, fieldModifiers, &self.fontRendererObj).then_some(RuntimeGuiAction::None))
            }
            ActiveGuiScreen::Multiplayer(screen) => match key {
                KeyCode::ArrowUp if modifiers.shift_key() => {
                    if let Err(error) = screen.moveSelectedServer(-1) { log::error!("Couldn't move server up: {error}"); }
                    Some(RuntimeGuiAction::None)
                }
                KeyCode::ArrowDown if modifiers.shift_key() => {
                    if let Err(error) = screen.moveSelectedServer(1) { log::error!("Couldn't move server down: {error}"); }
                    Some(RuntimeGuiAction::None)
                }
                KeyCode::ArrowUp => { screen.moveSelection(-1); Some(RuntimeGuiAction::None) }
                KeyCode::ArrowDown => { screen.moveSelection(1); Some(RuntimeGuiAction::None) }
                KeyCode::F5 => { screen.refreshServerList(); Some(RuntimeGuiAction::None) }
                KeyCode::Enter | KeyCode::NumpadEnter => screen.selectedServer().map(|(_, server)| RuntimeGuiAction::Connect(server)),
                _ => None,
            },
            _ => None,
        }
    }

    fn scroll(&mut self, lines: f32) -> bool {
        match self.worldGuiScreen.as_mut() {
            Some(WorldGuiScreen::Controls(screen)) => return screen.scroll(lines),
            Some(WorldGuiScreen::ResourcePacks(screen)) => return screen.scroll(lines),
            Some(WorldGuiScreen::ShaderSettings(screen)) => return screen.scroll(lines),
            Some(WorldGuiScreen::Language(screen)) => return screen.scroll(lines),
            _ => {}
        }
        match &mut self.currentScreen {
            ActiveGuiScreen::AccountManager(screen) => screen.scroll(lines, self.accountConfig.len()),
            ActiveGuiScreen::Multiplayer(screen) => {
                screen.scroll(lines);
                true
            }
            ActiveGuiScreen::WorldSelection(screen) => screen.scroll(lines),
            ActiveGuiScreen::ResourcePacks(screen) => screen.scroll(lines),
            ActiveGuiScreen::Controls(screen) => screen.scroll(lines),
            ActiveGuiScreen::ShaderSettings(screen) => screen.scroll(lines),
            ActiveGuiScreen::Language { screen, .. } => screen.scroll(lines),
            _ => false,
        }
    }

    fn escapeAction(&mut self) -> Option<RuntimeGuiAction> {
        match &mut self.currentScreen {
            ActiveGuiScreen::Empty | ActiveGuiScreen::MainMenu(_) => None,
            ActiveGuiScreen::AccountManager(_) => Some(RuntimeGuiAction::Switch(ScreenId::MainMenu)),
            ActiveGuiScreen::MicrosoftAuth(screen) => {
                screen.cancel();
                Some(RuntimeGuiAction::OpenAccountManager { notification: None })
            }
            ActiveGuiScreen::SessionLogin(_) | ActiveGuiScreen::OfflineLogin(_) => Some(RuntimeGuiAction::OpenAccountManager { notification: None }),
            ActiveGuiScreen::Options(_) | ActiveGuiScreen::Multiplayer(_) | ActiveGuiScreen::WorldSelection(_) => Some(RuntimeGuiAction::Switch(ScreenId::MainMenu)),
            ActiveGuiScreen::CreateWorld(_) => Some(RuntimeGuiAction::Switch(ScreenId::WorldSelection)),
            ActiveGuiScreen::Controls(_) => Some(RuntimeGuiAction::Switch(ScreenId::MainMenu)),
            ActiveGuiScreen::VideoSettings(_) => Some(RuntimeGuiAction::CloseVideoSettings),
            ActiveGuiScreen::ShaderSettings(screen) if screen.isOptionsView() => {
                Some(if screen.closeOptionsView() {
                    RuntimeGuiAction::ReloadShaderPack
                } else {
                    RuntimeGuiAction::None
                })
            }
            ActiveGuiScreen::ShaderSettings(_) => Some(RuntimeGuiAction::ReturnToVideoSettings),
            ActiveGuiScreen::ResourcePacks(screen) => {
                if screen.cancelConfirmation() {
                    Some(RuntimeGuiAction::None)
                } else {
                    Some(RuntimeGuiAction::ReturnToOptions)
                }
            }
            ActiveGuiScreen::SoundSettings(_) | ActiveGuiScreen::ChatSettings(_) | ActiveGuiScreen::SkinSettings(_) => Some(RuntimeGuiAction::ReturnToOptions),
            ActiveGuiScreen::Language { parent, .. } => Some(RuntimeGuiAction::Switch(*parent)),
            ActiveGuiScreen::AddServer { .. } | ActiveGuiScreen::ConfirmDelete { .. } => Some(RuntimeGuiAction::ReturnToMultiplayer { lastServer: None }),
            ActiveGuiScreen::DirectConnect { screen, .. } => Some(RuntimeGuiAction::ReturnToMultiplayer { lastServer: Some(screen.getAddress()) }),
            ActiveGuiScreen::DownloadTerrain { .. } => Some(RuntimeGuiAction::OpenWorld),
            ActiveGuiScreen::Connecting { .. } | ActiveGuiScreen::Disconnected { .. } | ActiveGuiScreen::World { .. } => None,
        }
    }
}

fn player_container_inventory_group(slotId: i32) -> i32 {
    match slotId {
        0 => 0,       // InventoryCraftResult
        1..=4 => 1,  // InventoryCrafting
        5..=45 => 2, // InventoryPlayer
        _ => -1,
    }
}

fn mapControlsAction(action: GuiControlsAction, world: bool) -> RuntimeGuiAction {
    match action {
        GuiControlsAction::None => RuntimeGuiAction::None,
        GuiControlsAction::Done => RuntimeGuiAction::ReturnToControlsParent { world },
        GuiControlsAction::SetSensitivity(value) => RuntimeGuiAction::SetMouseSensitivity(value),
        GuiControlsAction::ToggleInvertMouse => RuntimeGuiAction::ToggleInvertMouse,
        GuiControlsAction::ToggleTouchscreen => RuntimeGuiAction::ToggleTouchscreen,
        GuiControlsAction::ToggleAutoJump => RuntimeGuiAction::ToggleAutoJump,
        GuiControlsAction::SelectKeyBinding(_) => RuntimeGuiAction::SelectKeyBinding,
        GuiControlsAction::SetKeyBinding { binding, code } => RuntimeGuiAction::SetKeyBinding { binding, code },
        GuiControlsAction::ResetKeyBinding(binding) => RuntimeGuiAction::ResetKeyBinding(binding),
        GuiControlsAction::ResetAll => RuntimeGuiAction::ResetAllKeyBindings,
    }
}

fn mapVideoSettingsAction(action: GuiVideoSettingsAction) -> RuntimeGuiAction {
    match action {
        GuiVideoSettingsAction::SetGamma(value) => RuntimeGuiAction::SetGamma(value),
        GuiVideoSettingsAction::SetRenderDistance(value) => RuntimeGuiAction::SetRenderDistance(value),
        GuiVideoSettingsAction::SetFramerate { limit, enableVsync } => {
            RuntimeGuiAction::SetFramerate { limit, enableVsync }
        }
        GuiVideoSettingsAction::ToggleGraphics => RuntimeGuiAction::ToggleGraphics,
        GuiVideoSettingsAction::CycleAmbientOcclusion => RuntimeGuiAction::CycleAmbientOcclusion,
        GuiVideoSettingsAction::CycleGuiScale => RuntimeGuiAction::CycleGuiScale,
        GuiVideoSettingsAction::ToggleFullscreen => RuntimeGuiAction::ToggleFullscreen,
        GuiVideoSettingsAction::ToggleRenderBackend => RuntimeGuiAction::ToggleRenderBackend,
        GuiVideoSettingsAction::OpenShaders => RuntimeGuiAction::OpenShaderSettings,
        GuiVideoSettingsAction::Done => RuntimeGuiAction::CloseVideoSettings,
    }
}

fn mapSoundSettingsAction(action: GuiScreenOptionsSoundsAction) -> RuntimeGuiAction {
    match action {
        GuiScreenOptionsSoundsAction::SetSoundLevel(category, value) => RuntimeGuiAction::SetSoundLevel(category, value),
        GuiScreenOptionsSoundsAction::ToggleSubtitles => RuntimeGuiAction::ToggleSubtitles,
        GuiScreenOptionsSoundsAction::Done => RuntimeGuiAction::ReturnToOptions,
    }
}

fn mapChatSettingsAction(action: ScreenChatOptionsAction) -> RuntimeGuiAction {
    match action {
        ScreenChatOptionsAction::CycleVisibility => RuntimeGuiAction::CycleChatVisibility,
        ScreenChatOptionsAction::ToggleColours => RuntimeGuiAction::ToggleChatColours,
        ScreenChatOptionsAction::ToggleLinks => RuntimeGuiAction::ToggleChatLinks,
        ScreenChatOptionsAction::ToggleLinksPrompt => RuntimeGuiAction::ToggleChatLinksPrompt,
        ScreenChatOptionsAction::ToggleReducedDebugInfo => RuntimeGuiAction::ToggleReducedDebugInfo,
        ScreenChatOptionsAction::SetOpacity(value) => RuntimeGuiAction::SetChatOpacity(value),
        ScreenChatOptionsAction::SetScale(value) => RuntimeGuiAction::SetChatScale(value),
        ScreenChatOptionsAction::SetWidth(value) => RuntimeGuiAction::SetChatWidth(value),
        ScreenChatOptionsAction::SetHeightFocused(value) => RuntimeGuiAction::SetChatHeightFocused(value),
        ScreenChatOptionsAction::SetHeightUnfocused(value) => RuntimeGuiAction::SetChatHeightUnfocused(value),
        ScreenChatOptionsAction::Done => RuntimeGuiAction::ReturnToOptions,
    }
}

enum WorldEventAudio {
    Play(PositionedSoundRecord),
    StopRecord([f32; 3]),
}

fn world_event_audio(
    effectType: i32,
    position: BlockPos,
    data: i32,
    serverWide: bool,
    viewPosition: Option<[f64; 3]>,
    random: &mut JavaRandom,
) -> Option<WorldEventAudio> {
    let center = [
        position.x as f32 + 0.5,
        position.y as f32 + 0.5,
        position.z as f32 + 0.5,
    ];
    let play = |name: &str, category, volume, pitch| {
        WorldEventAudio::Play(PositionedSoundRecord::new(
            ResourceLocation::parse(name),
            category,
            volume,
            pitch,
            false,
            0,
            AttenuationType::Linear,
            center,
        ))
    };

    if serverWide {
        let (name, volume) = match effectType {
            1023 => ("entity.wither.spawn", 1.0),
            1028 => ("entity.enderdragon.death", 5.0),
            1038 => ("block.end_portal.spawn", 1.0),
            _ => return None,
        };
        let view = viewPosition?;
        let delta = [
            position.x as f64 - view[0],
            position.y as f64 - view[1],
            position.z as f64 - view[2],
        ];
        let distance = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
        let soundPosition = if distance > 0.0 {
            [
                (view[0] + delta[0] / distance * 2.0) as f32,
                (view[1] + delta[1] / distance * 2.0) as f32,
                (view[2] + delta[2] / distance * 2.0) as f32,
            ]
        } else {
            [view[0] as f32, view[1] as f32, view[2] as f32]
        };
        return Some(WorldEventAudio::Play(PositionedSoundRecord::new(
            ResourceLocation::parse(name),
            SoundCategory::Hostile,
            volume,
            1.0,
            false,
            0,
            AttenuationType::Linear,
            soundPosition,
        )));
    }

    let doorPitch = |random: &mut JavaRandom| random.next_f32() * 0.1 + 0.9;
    let hostilePitch = |random: &mut JavaRandom| {
        (random.next_f32() - random.next_f32()) * 0.2 + 1.0
    };
    Some(match effectType {
        1000 => play("block.dispenser.dispense", SoundCategory::Blocks, 1.0, 1.0),
        1001 => play("block.dispenser.fail", SoundCategory::Blocks, 1.0, 1.2),
        1002 => play("block.dispenser.launch", SoundCategory::Blocks, 1.0, 1.2),
        1003 => play("entity.endereye.launch", SoundCategory::Neutral, 1.0, 1.2),
        1004 => play("entity.firework.shoot", SoundCategory::Neutral, 1.0, 1.2),
        1005 => play("block.iron_door.open", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1006 => play("block.wooden_door.open", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1007 => play("block.wooden_trapdoor.open", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1008 => play("block.fence_gate.open", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1009 => play(
            "block.fire.extinguish",
            SoundCategory::Blocks,
            0.5,
            2.6 + (random.next_f32() - random.next_f32()) * 0.8,
        ),
        1010 => {
            let recordName = match data {
                2256 => Some("record.13"),
                2257 => Some("record.cat"),
                2258 => Some("record.blocks"),
                2259 => Some("record.chirp"),
                2260 => Some("record.far"),
                2261 => Some("record.mall"),
                2262 => Some("record.mellohi"),
                2263 => Some("record.stal"),
                2264 => Some("record.strad"),
                2265 => Some("record.ward"),
                2266 => Some("record.11"),
                2267 => Some("record.wait"),
                _ => None,
            };
            match recordName {
                Some(name) => WorldEventAudio::Play(PositionedSoundRecord::getRecordSoundRecord(
                    ResourceLocation::parse(name), center,
                )),
                None => WorldEventAudio::StopRecord(center),
            }
        }
        1011 => play("block.iron_door.close", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1012 => play("block.wooden_door.close", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1013 => play("block.wooden_trapdoor.close", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1014 => play("block.fence_gate.close", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1015 => play("entity.ghast.warn", SoundCategory::Hostile, 10.0, hostilePitch(random)),
        1016 => play("entity.ghast.shoot", SoundCategory::Hostile, 10.0, hostilePitch(random)),
        1017 => play("entity.enderdragon.shoot", SoundCategory::Hostile, 10.0, hostilePitch(random)),
        1018 => play("entity.blaze.shoot", SoundCategory::Hostile, 2.0, hostilePitch(random)),
        1019 => play("entity.zombie.attack_door_wood", SoundCategory::Hostile, 2.0, hostilePitch(random)),
        1020 => play("entity.zombie.attack_iron_door", SoundCategory::Hostile, 2.0, hostilePitch(random)),
        1021 => play("entity.zombie.break_door_wood", SoundCategory::Hostile, 2.0, hostilePitch(random)),
        1022 => play("entity.wither.break_block", SoundCategory::Hostile, 2.0, hostilePitch(random)),
        1024 => play("entity.wither.shoot", SoundCategory::Hostile, 2.0, hostilePitch(random)),
        1025 => play("entity.bat.takeoff", SoundCategory::Neutral, 0.05, hostilePitch(random)),
        1026 => play("entity.zombie.infect", SoundCategory::Hostile, 2.0, hostilePitch(random)),
        1027 => play("entity.zombie_villager.converted", SoundCategory::Neutral, 2.0, hostilePitch(random)),
        1029 => play("block.anvil.destroy", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1030 => play("block.anvil.use", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1031 => play("block.anvil.land", SoundCategory::Blocks, 0.3, doorPitch(random)),
        1032 => WorldEventAudio::Play(PositionedSoundRecord::getMasterRecord(
            ResourceLocation::parse("block.portal.travel"),
            random.next_f32() * 0.4 + 0.8,
        )),
        1033 => play("block.chorus_flower.grow", SoundCategory::Blocks, 1.0, 1.0),
        1034 => play("block.chorus_flower.death", SoundCategory::Blocks, 1.0, 1.0),
        1035 => play("block.brewing_stand.brew", SoundCategory::Blocks, 1.0, 1.0),
        1036 => play("block.iron_trapdoor.close", SoundCategory::Blocks, 1.0, doorPitch(random)),
        1037 => play("block.iron_trapdoor.open", SoundCategory::Blocks, 1.0, doorPitch(random)),
        2001 => {
            let block = Block::getBlockById(data & 4095);
            if block.isAir() {
                return None;
            }
            let soundType = SoundType::forBlockId(Block::getIdFromBlock(block));
            WorldEventAudio::Play(PositionedSoundRecord::new(
                soundType.getBreakSound(),
                SoundCategory::Blocks,
                (soundType.getVolume() + 1.0) / 2.0,
                soundType.getPitch() * 0.8,
                false,
                0,
                AttenuationType::Linear,
                center,
            ))
        }
        2002 | 2007 => play(
            "entity.splash_potion.break",
            SoundCategory::Neutral,
            1.0,
            doorPitch(random),
        ),
        2006 => play(
            "entity.enderdragon_fireball.explode",
            SoundCategory::Hostile,
            1.0,
            doorPitch(random),
        ),
        3000 => play(
            "block.end_gateway.spawn",
            SoundCategory::Blocks,
            10.0,
            (1.0 + (random.next_f32() - random.next_f32()) * 0.2) * 0.7,
        ),
        3001 => play(
            "entity.enderdragon.growl",
            SoundCategory::Hostile,
            64.0,
            0.8 + random.next_f32() * 0.3,
        ),
        _ => return None,
    })
}

fn playGuiSound(
    soundHandler: &mut SoundHandler,
    sound: Option<&crate::net::minecraft::client::gui::GuiButton::GuiSoundCommand>,
) {
    if let Some(sound) = sound {
        soundHandler.playSound(PositionedSoundRecord::getMasterRecord(
            sound.event.clone(),
            sound.pitch,
        ));
    }
}

struct MinecraftApplication {
    minecraft: Option<Minecraft>,
    renderer: Option<DesktopRenderer>,
    window: Option<Window>,
    mainMenu: Option<MainMenuRuntime>,
    fatalError: Option<anyhow::Error>,
    redrawPending: bool,
    nextFrameDeadline: Instant,
    /// MCP `Timer#field_194147_b`: accumulated frame-interval ticks. The
    /// integer part is the number of ticks to run (`elapsedTicks`), the
    /// fraction stays as the render partial-ticks budget.
    timerAccumulator: f32,
    /// MCP `Timer#lastSyncSysClock`: time of the last `updateTimer`.
    lastTimerSync: Instant,
    pendingResizeSince: Option<Instant>,
    keyboardModifiers: ModifiersState,
    worldMouseGrabbed: bool,
    windowFocused: bool,
    debugFps: i32,
    framesThisSecond: i32,
    lastFpsUpdate: Instant,
    debugKeyDown: bool,
    debugActionUsed: bool,
    debugCrashKeyPressTime: Option<Instant>,
    debugCrashKeyDown: bool,
    frameProfileStarted: Instant,
    frameProfileFrames: u64,
    frameProfileWorldFrames: u64,
    frameProfilePrepareNanos: u128,
    frameProfileRenderNanos: u128,
    integratedServer: Option<IntegratedServerHandle>,
}

impl MinecraftApplication {
    fn new(minecraft: Minecraft) -> Self {
        Self {
            minecraft: Some(minecraft), renderer: None, window: None, mainMenu: None, fatalError: None,
            redrawPending: false, nextFrameDeadline: Instant::now(), timerAccumulator: 0.0,
            lastTimerSync: Instant::now(),
            pendingResizeSince: None, keyboardModifiers: ModifiersState::empty(), worldMouseGrabbed: false,
            windowFocused: true,
            debugFps: 0,
            framesThisSecond: 0,
            lastFpsUpdate: Instant::now(),
            debugKeyDown: false,
            debugActionUsed: false,
            debugCrashKeyPressTime: None,
            debugCrashKeyDown: false,
            frameProfileStarted: Instant::now(),
            frameProfileFrames: 0,
            frameProfileWorldFrames: 0,
            frameProfilePrepareNanos: 0,
            frameProfileRenderNanos: 0,
            integratedServer: None,
        }
    }

    fn recordFrameProfile(
        &mut self,
        prepare: Duration,
        render: Duration,
        worldFrame: bool,
    ) {
        self.frameProfileFrames = self.frameProfileFrames.saturating_add(1);
        self.frameProfileWorldFrames = self
            .frameProfileWorldFrames
            .saturating_add(if worldFrame { 1 } else { 0 });
        self.frameProfilePrepareNanos = self
            .frameProfilePrepareNanos
            .saturating_add(prepare.as_nanos());
        self.frameProfileRenderNanos = self
            .frameProfileRenderNanos
            .saturating_add(render.as_nanos());

        let elapsed = self.frameProfileStarted.elapsed();
        if elapsed < Duration::from_secs(5) {
            return;
        }
        let frames = self.frameProfileFrames.max(1) as f64;
        let prepareMs = self.frameProfilePrepareNanos as f64 / frames / 1_000_000.0;
        let renderMs = self.frameProfileRenderNanos as f64 / frames / 1_000_000.0;
        log::info!(
            "Client frame stages: {:.1} fps, prepare={:.3} ms, render/present={:.3} ms, world_frames={}, gui_frames={}",
            self.frameProfileFrames as f64 / elapsed.as_secs_f64().max(0.001),
            prepareMs,
            renderMs,
            self.frameProfileWorldFrames,
            self.frameProfileFrames.saturating_sub(self.frameProfileWorldFrames),
        );
        self.frameProfileStarted = Instant::now();
        self.frameProfileFrames = 0;
        self.frameProfileWorldFrames = 0;
        self.frameProfilePrepareNanos = 0;
        self.frameProfileRenderNanos = 0;
    }

    fn processDebugChord(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::KeyA => {
                let settings = self.minecraft.as_ref().map(|minecraft| minecraft.gameSettings.clone());
                if let (Some(runtime), Some(settings)) = (self.mainMenu.as_mut(), settings.as_ref()) {
                    runtime.worldRenderer.reloadChunks();
                    runtime.printDebugMessage("debug.reload_chunks.message", settings);
                }
                true
            }
            KeyCode::KeyB => {
                let settings = self.minecraft.as_ref().map(|minecraft| minecraft.gameSettings.clone());
                if let (Some(runtime), Some(settings)) = (self.mainMenu.as_mut(), settings.as_ref()) {
                    let enabled = runtime.worldRenderer.toggleDebugHitboxes();
                    runtime.printDebugMessage(
                        if enabled { "debug.show_hitboxes.on" } else { "debug.show_hitboxes.off" },
                        settings,
                    );
                }
                true
            }
            KeyCode::KeyC => {
                if self.debugCrashKeyPressTime.is_none() {
                    self.debugCrashKeyPressTime = Some(Instant::now());
                }
                true
            }
            KeyCode::KeyD => {
                if let Some(runtime) = self.mainMenu.as_mut() {
                    runtime.worldRenderer.clearChatMessages();
                }
                true
            }
            KeyCode::KeyF => {
                let settings = if let Some(minecraft) = self.minecraft.as_mut() {
                    let step = if self.keyboardModifiers.shift_key() { -1 } else { 1 };
                    minecraft.gameSettings.renderDistanceChunks =
                        (minecraft.gameSettings.renderDistanceChunks + step).clamp(2, 32);
                    if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                        log::error!("Couldn't save options.txt: {error}");
                    }
                    Some(minecraft.gameSettings.clone())
                } else {
                    None
                };
                if let (Some(runtime), Some(settings)) = (self.mainMenu.as_mut(), settings.as_ref()) {
                    runtime.printDebugValue(
                        "debug.cycle_renderdistance.message",
                        settings.renderDistanceChunks.to_string(),
                        settings,
                    );
                }
                true
            }
            KeyCode::KeyG => {
                let settings = self.minecraft.as_ref().map(|minecraft| minecraft.gameSettings.clone());
                if let (Some(runtime), Some(settings)) = (self.mainMenu.as_mut(), settings.as_ref()) {
                    let enabled = runtime.worldRenderer.toggleChunkBoundaries();
                    runtime.printDebugMessage(
                        if enabled { "debug.chunk_boundaries.on" } else { "debug.chunk_boundaries.off" },
                        settings,
                    );
                }
                true
            }
            KeyCode::KeyH => {
                let settings = if let Some(minecraft) = self.minecraft.as_mut() {
                    minecraft.gameSettings.advancedItemTooltips =
                        !minecraft.gameSettings.advancedItemTooltips;
                    if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                        log::error!("Couldn't save options.txt: {error}");
                    }
                    Some(minecraft.gameSettings.clone())
                } else {
                    None
                };
                if let (Some(runtime), Some(settings)) = (self.mainMenu.as_mut(), settings.as_ref()) {
                    runtime.printDebugMessage(
                        if settings.advancedItemTooltips {
                            "debug.advanced_tooltips.on"
                        } else {
                            "debug.advanced_tooltips.off"
                        },
                        settings,
                    );
                }
                true
            }
            KeyCode::KeyN => {
                let result = self.mainMenu.as_ref()
                    .map(MainMenuRuntime::toggleCreativeSpectator)
                    .unwrap_or(Ok(None));
                match result {
                    Ok(Some(false)) => {
                        let settings = self.minecraft.as_ref().map(|minecraft| minecraft.gameSettings.clone());
                        if let (Some(runtime), Some(settings)) = (self.mainMenu.as_mut(), settings.as_ref()) {
                            runtime.printDebugMessage("debug.creative_spectator.error", settings);
                        }
                    }
                    Ok(_) => {}
                    Err(error) => log::error!("failed F3+N game-mode request: {error}"),
                }
                true
            }
            KeyCode::KeyP => {
                let settings = if let Some(minecraft) = self.minecraft.as_mut() {
                    minecraft.gameSettings.pauseOnLostFocus =
                        !minecraft.gameSettings.pauseOnLostFocus;
                    if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                        log::error!("Couldn't save options.txt: {error}");
                    }
                    Some(minecraft.gameSettings.clone())
                } else {
                    None
                };
                if let (Some(runtime), Some(settings)) = (self.mainMenu.as_mut(), settings.as_ref()) {
                    runtime.printDebugMessage(
                        if settings.pauseOnLostFocus {
                            "debug.pause_focus.on"
                        } else {
                            "debug.pause_focus.off"
                        },
                        settings,
                    );
                }
                true
            }
            KeyCode::KeyQ => {
                let settings = self.minecraft.as_ref().map(|minecraft| minecraft.gameSettings.clone());
                if let (Some(runtime), Some(settings)) = (self.mainMenu.as_mut(), settings.as_ref()) {
                    runtime.printDebugHelp(settings);
                }
                true
            }
            KeyCode::KeyT => {
                let result = match (self.mainMenu.as_mut(), self.minecraft.as_ref()) {
                    (Some(runtime), Some(minecraft)) => runtime.reloadResources(minecraft),
                    _ => Ok(()),
                };
                if let Err(error) = result {
                    log::error!("Failed to reload resource packs: {error}");
                } else {
                    let settings = self.minecraft.as_ref().map(|minecraft| minecraft.gameSettings.clone());
                    if let (Some(runtime), Some(settings)) = (self.mainMenu.as_mut(), settings.as_ref()) {
                        runtime.printDebugMessage("debug.reload_resourcepacks.message", settings);
                    }
                }
                true
            }
            _ => false,
        }
    }

    fn triggerManualDebugCrash(&mut self, eventLoop: &ActiveEventLoop) {
        let mut report = CrashReport::new(
            "Manually triggered debug crash",
            "Manually triggered debug crash",
        );

        if let Some(renderer) = self.renderer.as_ref() {
            report
                .getCategory()
                .addCrashSection("Graphics device", renderer.deviceName());
        }
        if let Some(window) = self.window.as_ref() {
            let size = window.inner_size();
            report
                .getCategory()
                .addCrashSection("Window size", format!("{}x{}", size.width, size.height));
        }
        if let Some(runtime) = self.mainMenu.as_ref() {
            if let ActiveGuiScreen::World { connection, .. } = &runtime.currentScreen {
                let (position, dimension, gameType, entityId) = connection
                    .getSharedPlayState()
                    .withRead(|state| {
                        (
                            state.playerPosition.clone(),
                            state.worldClient.as_ref().map(|world| world.getDimension()),
                            state.gameType,
                            state.thePlayer.as_ref().map(|player| player.entityId),
                        )
                    });
                let category = report.makeCategory("Affected level");
                category.addCrashSection(
                    "Level dimension",
                    dimension.map_or_else(|| "unknown".to_owned(), |value| value.to_string()),
                );
                category.addCrashSection("Game type", format!("{:?}", gameType));
                category.addCrashSection(
                    "Player entity ID",
                    entityId.map_or_else(|| "unknown".to_owned(), |value| value.to_string()),
                );
                category.addCrashSection(
                    "Player location",
                    CrashReportCategory::getCoordinateInfoXYZ(
                        position.posX.floor() as i32,
                        position.posY.floor() as i32,
                        position.posZ.floor() as i32,
                    ),
                );
            }
        }

        let gameDir = self
            .minecraft
            .as_ref()
            .map(|minecraft| minecraft.gameDir.clone());
        let path = gameDir
            .as_ref()
            .map(|gameDir| report.defaultClientReportPath(gameDir));
        if let Some(path) = path.as_ref() {
            if report.saveToFile(path) {
                log::error!("Crash report saved to {}", path.display());
            }
        }
        let reported = ReportedException::new(report);
        let savedPath = reported
            .getCrashReport()
            .getFile()
            .map(|path| path.display().to_string());
        let error = savedPath.map_or_else(
            || anyhow::anyhow!("{}", reported),
            |path| anyhow::anyhow!("{}; crash report: {path}", reported),
        );
        self.fail(eventLoop, error);
    }

    /// Rust event-loop equivalent of `Minecraft#getLimitFramerate`.
    ///
    /// MCP 1.12.2 returns 30 while no `WorldClient` is loaded and a `GuiScreen`
    /// is visible. Once a world exists, the configured framerate limit applies
    /// even when an in-world GUI is open.
    fn getLimitFramerate(&self) -> i32 {
        let worldLoaded = self
            .mainMenu
            .as_ref()
            .is_some_and(MainMenuRuntime::hasLoadedWorld);
        if !worldLoaded && self.mainMenu.is_some() {
            30
        } else {
            self.minecraft
                .as_ref()
                .map_or(FRAMERATE_LIMIT_MAX, |minecraft| minecraft.gameSettings.limitFramerate)
        }
    }

    /// Equivalent of `Minecraft.isFramerateLimitBelowMax()`. In 1.12.2 the
    /// slider maximum (260) means Unlimited and therefore skips `Display.sync`.
    fn isFramerateLimitBelowMax(&self) -> bool {
        self.getLimitFramerate() < FRAMERATE_LIMIT_MAX
    }

    fn currentFrameInterval(&self) -> Option<Duration> {
        frame_interval_for_limit(self.getLimitFramerate())
    }

    fn fail(&mut self, eventLoop: &ActiveEventLoop, error: anyhow::Error) {
        self.fatalError = Some(error);
        eventLoop.exit();
    }

    fn requestRedraw(&mut self) {
        if self.redrawPending { return; }
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
            self.redrawPending = true;
        }
    }

    fn setWorldMouseGrabbed(&mut self, grabbed: bool) {
        // `Minecraft#setIngameFocus` only succeeds while the display is active,
        // and no GuiScreen may coexist with gameplay focus. Centralize both
        // invariants here so packet-driven screens cannot leave relative mouse
        // motion enabled behind a visible menu.
        if grabbed {
            let modalWorldGuiOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isModalWorldGuiOpen);
            if !can_grab_world_mouse(self.windowFocused, modalWorldGuiOpen) { return; }
        }
        if self.worldMouseGrabbed == grabbed {
            return;
        }
        let Some(window) = self.window.as_ref() else {
            self.worldMouseGrabbed = false;
            return;
        };

        if grabbed {
            let result = window
                .set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            match result {
                Ok(()) => {
                    window.set_cursor_visible(false);
                    self.worldMouseGrabbed = true;
                }
                Err(error) => {
                    log::warn!("Unable to grab the Minecraft mouse cursor: {error}");
                    self.worldMouseGrabbed = false;
                }
            }
        } else {
            if let Err(error) = window.set_cursor_grab(CursorGrabMode::None) {
                log::warn!("Unable to release the Minecraft mouse cursor: {error}");
            }
            window.set_cursor_visible(true);
            self.worldMouseGrabbed = false;
            if let Some(runtime) = self.mainMenu.as_mut() {
                // Vanilla calls sendClickBlockToController(false) when gameplay
                // focus is lost, which aborts any in-progress dig.
                if let Err(message) = runtime.worldActionButton(KeyBindingId::Attack, false) {
                    log::error!("failed aborting block removal while releasing mouse: {message}");
                }
                if let Err(message) = runtime.worldActionButton(KeyBindingId::UseItem, false) {
                    log::error!("failed releasing active item use while releasing mouse: {message}");
                }
                runtime.clearMovementKeys();
            }
        }
    }

    fn applyRuntimeMouseFocusRequest(&mut self) {
        let request = self.mainMenu.as_mut().and_then(MainMenuRuntime::takeWorldMouseFocusRequest);
        match request {
            Some(true) if !self.windowFocused => {
                // Display.isActive() is false: vanilla defers setIngameFocus.
                // Retain the request until the window regains focus rather
                // than silently consuming the server-close transition.
                if let Some(runtime) = self.mainMenu.as_mut() {
                    runtime.pendingWorldMouseFocus = Some(true);
                }
            }
            Some(grabbed) => self.setWorldMouseGrabbed(grabbed),
            None => {}
        }
    }

    fn applyPendingResize(&mut self) -> anyhow::Result<bool> {
        let Some(since) = self.pendingResizeSince else { return Ok(false); };
        if since.elapsed() < RESIZE_DEBOUNCE { return Ok(false); }
        let (Some(window), Some(renderer), Some(mainMenu), Some(minecraft)) = (
            self.window.as_ref(), self.renderer.as_mut(), self.mainMenu.as_mut(), self.minecraft.as_ref(),
        ) else { return Ok(false); };
        renderer.resize(window).context("failed resizing Minecraft graphics surface")?;
        let extent = renderer.extent();
        if extent.width > 0 && extent.height > 0 { mainMenu.resize(minecraft, extent.width, extent.height); }
        self.pendingResizeSince = None;
        Ok(true)
    }

    fn launchIntegratedServer(&mut self, folderName:String, worldName:String, settings:WorldSettings) -> anyhow::Result<()> {
        // MCP `Minecraft#launchIntegratedServer`: unload any prior integrated
        // server, construct/start the new server, then connect through a local
        // in-memory NetworkManager rather than localhost TCP.
        self.integratedServer.take();
        let minecraft=self.minecraft.as_ref().expect("Minecraft state");
        let server=IntegratedServer::new(
            minecraft.gameDir.join("saves"), minecraft.getSession().getUsername(),
            folderName, worldName, settings, minecraft.isDemo(),
        );
        let (handle,address)=IntegratedServerHandle::launch(server).map_err(anyhow::Error::msg)?;
        let screen=GuiConnecting::newLocal(
            address, minecraft.getSession().clone(), minecraft.gameSettings.language.clone(),
            minecraft.gameSettings.renderDistanceChunks, minecraft.gameSettings.chatVisibility,
            minecraft.gameSettings.chatColours, minecraft.gameSettings.modelPartFlags,
            minecraft.gameSettings.mainHand,
        );
        // The existing screen-state carrier still stores the historical
        // multiplayer return object; integrated cancel/leave is intercepted by
        // MinecraftApplication and returns to WorldSelection.
        let parent=Box::new(GuiMultiplayer::new(minecraft.gameDir.clone()));
        let runtime=self.mainMenu.as_mut().expect("GUI runtime");
        runtime.currentScreen=ActiveGuiScreen::Connecting{screen,parent};
        runtime.initCurrentScreen(minecraft);
        self.integratedServer=Some(handle);
        self.setWorldMouseGrabbed(false);
        Ok(())
    }

    fn applyGuiAction(&mut self, action: RuntimeGuiAction) -> anyhow::Result<bool> {
        match action {
            RuntimeGuiAction::None => {}
            RuntimeGuiAction::Shutdown => {
                self.setWorldMouseGrabbed(false);
                return Ok(true);
            }
            RuntimeGuiAction::Switch(screen) => {
                self.setWorldMouseGrabbed(false);
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").switchTo(minecraft, screen)?;
            }
            RuntimeGuiAction::OpenCreateWorld => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                let newWorldName = self.mainMenu.as_ref().expect("GUI runtime").locale.translate_key("selectWorld.newWorld").to_owned();
                let runtime = self.mainMenu.as_mut().expect("GUI runtime");
                runtime.currentScreen = ActiveGuiScreen::CreateWorld(GuiCreateWorld::new(minecraft.gameDir.join("saves"), newWorldName));
                runtime.initCurrentScreen(minecraft);
            }
            RuntimeGuiAction::CreateWorld(request) => {
                let minecraft=self.minecraft.as_ref().expect("Minecraft state");
                let settings=minecraft.prepareIntegratedServerLaunch(&request.saveDirName,&request.worldName,Some(request.settings.clone()))?;
                if let Err(error)=self.launchIntegratedServer(request.saveDirName,request.worldName,settings) {
                    // Development-boundary guard only: until every vanilla
                    // generator exists, a missing generator must not turn a
                    // GUI action into process termination. The screen remains
                    // on Create World and the exact missing MCP tranche is logged.
                    log::error!("IntegratedServer failed to start: {error}");
                }
            }
            RuntimeGuiAction::JoinWorld { folderName, worldName } => {
                let minecraft=self.minecraft.as_ref().expect("Minecraft state");
                let settings=minecraft.prepareIntegratedServerLaunch(&folderName,&worldName,None)?;
                if let Err(error)=self.launchIntegratedServer(folderName,worldName,settings) {
                    log::error!("IntegratedServer failed to open world: {error}");
                }
            }
            RuntimeGuiAction::OpenLanguage(parent) => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openLanguage(minecraft, parent);
            }
            RuntimeGuiAction::OpenWorldLanguage => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openWorldLanguage(minecraft);
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::OpenAccountManager { notification } => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openAccountManager(minecraft, notification);
            }
            RuntimeGuiAction::OpenMicrosoftAuth => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openMicrosoftAuth(minecraft);
            }
            RuntimeGuiAction::OpenSessionLogin => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openSessionLogin(minecraft);
            }
            RuntimeGuiAction::OpenOfflineLogin => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openOfflineLogin(minecraft);
            }
            RuntimeGuiAction::AccountAuthenticated { session, returnToManager } => {
                let username = session.getUsername().to_owned();
                self.minecraft.as_mut().expect("Minecraft state").setSession(session);
                if returnToManager {
                    let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                    self.mainMenu.as_mut().expect("GUI runtime").openAccountManager(
                        minecraft,
                        Some(format!("§aSuccessful login! ({username})§r")),
                    );
                }
            }
            RuntimeGuiAction::ToggleUnicode => {
                let extent = self.renderer.as_ref().expect("renderer").extent();
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.forceUnicodeFont = !minecraft.gameSettings.forceUnicodeFont;
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) { log::error!("Couldn't save options.txt: {error}"); }
                let runtime = self.mainMenu.as_mut().expect("GUI runtime");
                let unicode = minecraft.gameSettings.forceUnicodeFont || runtime.locale.is_unicode();
                runtime.fontRendererObj.set_unicode_flag(unicode);
                runtime.worldRenderer.setUnicodeFlag(unicode);
                runtime.resize(minecraft, extent.width, extent.height);
            }
            RuntimeGuiAction::SetLanguage(code) => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.language = code.clone();
                let runtime = self.mainMenu.as_mut().expect("GUI runtime");
                if let Err(error) = runtime.setLanguage(minecraft, &code) {
                    log::error!("Couldn't refresh resources after language switch to {code}: {error}");
                } else {
                    // MCP GuiLanguage.List#elementClicked saves only after
                    // refreshResources and the font/screen refresh complete.
                    if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                        log::error!("Couldn't save options.txt after language switch: {error}");
                    }
                    // MCP `GameSettings#saveOptions` finishes by sending
                    // CPacketClientSettings when a player/world is present.
                    if let Err(error) = runtime.sendClientSettings(&minecraft.gameSettings) {
                        log::error!("Couldn't send client settings after language switch: {error}");
                    }
                    log::info!("switched game language to {code}");
                }
            }
            RuntimeGuiAction::SetFov(value) => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.fovSetting = value.clamp(30.0, 110.0);
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save options.txt after changing FOV: {error}");
                }
            }
            RuntimeGuiAction::ToggleForceSprint => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.forceSprint = !minecraft.gameSettings.forceSprint;
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save options.txt: {error}");
                }
                let runtime = self.mainMenu.as_mut().expect("GUI runtime");
                runtime.initCurrentScreen(minecraft);
                runtime.initWorldGui(minecraft);
            }
            RuntimeGuiAction::OpenControls => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openControls(minecraft);
            }
            RuntimeGuiAction::OpenWorldControls => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openWorldControls(minecraft);
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::ReturnToControlsParent { world } => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save controls to options.txt: {error}");
                }
                if world {
                    self.mainMenu.as_mut().expect("GUI runtime").openWorldOptions(minecraft);
                    self.setWorldMouseGrabbed(false);
                } else {
                    self.mainMenu.as_mut().expect("GUI runtime").switchTo(minecraft, ScreenId::Options)?;
                }
            }
            RuntimeGuiAction::SetMouseSensitivity(value) => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.mouseSensitivity = value.clamp(0.0, 1.0);
            }
            RuntimeGuiAction::ToggleInvertMouse => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.invertMouse = !minecraft.gameSettings.invertMouse;
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save invert mouse setting: {error}");
                }
            }
            RuntimeGuiAction::ToggleTouchscreen => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.touchscreen = !minecraft.gameSettings.touchscreen;
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save touchscreen setting: {error}");
                }
            }
            RuntimeGuiAction::ToggleAutoJump => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.autoJump = !minecraft.gameSettings.autoJump;
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save auto jump setting: {error}");
                }
            }
            RuntimeGuiAction::SelectKeyBinding => {}
            RuntimeGuiAction::SetKeyBinding { binding, code } => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.setOptionKeyBinding(binding, code);
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save key binding: {error}");
                }
            }
            RuntimeGuiAction::ResetKeyBinding(binding) => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                let default = minecraft.gameSettings.keyBinding(binding).keyCodeDefault;
                minecraft.gameSettings.setOptionKeyBinding(binding, default);
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save reset key binding: {error}");
                }
            }
            RuntimeGuiAction::ResetAllKeyBindings => {
                self.minecraft.as_mut().expect("Minecraft state").gameSettings.resetAllKeyBindings();
            }
            RuntimeGuiAction::OpenSoundSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openSoundSettings(minecraft);
            }
            RuntimeGuiAction::OpenChatSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openChatSettings(minecraft);
            }
            RuntimeGuiAction::OpenSkinSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openSkinSettings(minecraft);
            }
            RuntimeGuiAction::OpenResourcePacks => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openResourcePacks(minecraft);
            }
            RuntimeGuiAction::OpenWorldSoundSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openWorldSoundSettings(minecraft);
            }
            RuntimeGuiAction::OpenWorldChatSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openWorldChatSettings(minecraft);
            }
            RuntimeGuiAction::OpenWorldSkinSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openWorldSkinSettings(minecraft);
            }
            RuntimeGuiAction::OpenWorldResourcePacks => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openWorldResourcePacks(minecraft);
            }
            RuntimeGuiAction::ReturnToOptions => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) { log::error!("Couldn't save options.txt: {error}"); }
                self.mainMenu.as_mut().expect("GUI runtime").switchTo(minecraft, ScreenId::Options)?;
            }
            RuntimeGuiAction::SetSoundLevel(category, value) => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.setSoundLevel(category, value);
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save options.txt after changing sound level: {error}");
                }
                self.mainMenu.as_mut().expect("GUI runtime").soundHandler.setSoundLevel(category, value);
            }
            RuntimeGuiAction::ToggleSubtitles => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.showSubtitles = !minecraft.gameSettings.showSubtitles;
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save options.txt after changing subtitles: {error}");
                }
                let runtime = self.mainMenu.as_mut().expect("GUI runtime");
                runtime.initCurrentScreen(minecraft);
                runtime.initWorldGui(minecraft);
            }
            RuntimeGuiAction::CycleChatVisibility => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.chatVisibility = EnumChatVisibility::getChatVisibility((minecraft.gameSettings.chatVisibility.getChatVisibilityId()+1)%3);
                let runtime = self.mainMenu.as_mut().expect("GUI runtime"); let _=runtime.sendClientSettings(&minecraft.gameSettings); runtime.initCurrentScreen(minecraft); runtime.initWorldGui(minecraft);
            }
            RuntimeGuiAction::ToggleChatColours => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state"); minecraft.gameSettings.chatColours=!minecraft.gameSettings.chatColours;
                let runtime=self.mainMenu.as_mut().expect("GUI runtime"); let _=runtime.sendClientSettings(&minecraft.gameSettings); runtime.initCurrentScreen(minecraft); runtime.initWorldGui(minecraft);
            }
            RuntimeGuiAction::ToggleChatLinks => { let minecraft=self.minecraft.as_mut().expect("Minecraft state"); minecraft.gameSettings.chatLinks=!minecraft.gameSettings.chatLinks; let runtime=self.mainMenu.as_mut().expect("GUI runtime"); runtime.initCurrentScreen(minecraft); runtime.initWorldGui(minecraft); }
            RuntimeGuiAction::ToggleChatLinksPrompt => { let minecraft=self.minecraft.as_mut().expect("Minecraft state"); minecraft.gameSettings.chatLinksPrompt=!minecraft.gameSettings.chatLinksPrompt; let runtime=self.mainMenu.as_mut().expect("GUI runtime"); runtime.initCurrentScreen(minecraft); runtime.initWorldGui(minecraft); }
            RuntimeGuiAction::ToggleReducedDebugInfo => { let minecraft=self.minecraft.as_mut().expect("Minecraft state"); minecraft.gameSettings.reducedDebugInfo=!minecraft.gameSettings.reducedDebugInfo; let runtime=self.mainMenu.as_mut().expect("GUI runtime"); runtime.initCurrentScreen(minecraft); runtime.initWorldGui(minecraft); }
            RuntimeGuiAction::SetChatOpacity(value) => self.minecraft.as_mut().expect("Minecraft state").gameSettings.chatOpacity=value.clamp(0.0,1.0),
            RuntimeGuiAction::SetChatScale(value) => self.minecraft.as_mut().expect("Minecraft state").gameSettings.chatScale=value.clamp(0.0,1.0),
            RuntimeGuiAction::SetChatWidth(value) => self.minecraft.as_mut().expect("Minecraft state").gameSettings.chatWidth=value.clamp(0.0,1.0),
            RuntimeGuiAction::SetChatHeightFocused(value) => self.minecraft.as_mut().expect("Minecraft state").gameSettings.chatHeightFocused=value.clamp(0.0,1.0),
            RuntimeGuiAction::SetChatHeightUnfocused(value) => self.minecraft.as_mut().expect("Minecraft state").gameSettings.chatHeightUnfocused=value.clamp(0.0,1.0),
            RuntimeGuiAction::ToggleModelPart(part) => {
                let minecraft=self.minecraft.as_mut().expect("Minecraft state"); minecraft.gameSettings.modelPartFlags ^= part.getPartMask();
                let runtime=self.mainMenu.as_mut().expect("GUI runtime"); let _=runtime.sendClientSettings(&minecraft.gameSettings); runtime.initCurrentScreen(minecraft); runtime.initWorldGui(minecraft);
            }
            RuntimeGuiAction::ToggleMainHand => {
                let minecraft=self.minecraft.as_mut().expect("Minecraft state"); minecraft.gameSettings.mainHand=minecraft.gameSettings.mainHand.opposite();
                let runtime=self.mainMenu.as_mut().expect("GUI runtime"); let _=runtime.sendClientSettings(&minecraft.gameSettings); runtime.initCurrentScreen(minecraft); runtime.initWorldGui(minecraft);
            }
            RuntimeGuiAction::ApplyResourcePacks { selected, world } => {
                let minecraft=self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.resourcePacks=selected;
                let repository=ResourcePackRepository::scan(&minecraft.fileResourcepacks).unwrap_or_default();
                minecraft.gameSettings.incompatibleResourcePacks=minecraft.gameSettings.resourcePacks.iter().filter(|name| repository.findByName(name).is_some_and(|entry| !entry.isCompatibleWith1122())).cloned().collect();
                if let Err(error)=minecraft.rebuildSelectedResourcePacksFromRepository(&repository){log::error!("Couldn't apply resource packs: {error}");}
                if let Err(error)=minecraft.gameSettings.saveOptions(&minecraft.gameDir){log::error!("Couldn't save options.txt: {error}");}
                let runtime=self.mainMenu.as_mut().expect("GUI runtime");
                if let Err(error)=runtime.reloadResources(minecraft){log::error!("Couldn't reload resource packs: {error}");}
                if world { runtime.openWorldOptions(minecraft); } else { runtime.switchTo(minecraft,ScreenId::Options)?; }
            }
            RuntimeGuiAction::OpenResourcePackFolder => {
                let path=self.minecraft.as_ref().expect("Minecraft state").fileResourcepacks.clone();
                #[cfg(target_os="windows")] let result=std::process::Command::new("explorer").arg(&path).spawn();
                #[cfg(target_os="macos")] let result=std::process::Command::new("open").arg(&path).spawn();
                #[cfg(all(not(target_os="windows"),not(target_os="macos")))] let result=std::process::Command::new("xdg-open").arg(&path).spawn();
                if let Err(error)=result{log::error!("Couldn't open resource pack folder {}: {error}",path.display());}
            }
            RuntimeGuiAction::OpenVideoSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openVideoSettings(minecraft);
            }
            RuntimeGuiAction::OpenShaderSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if minecraft.gameSettings.activeRenderBackend
                    != crate::launcher::RenderBackend::RenderBackend::OpenGl
                {
                    log::warn!("Ignored shader-settings request while Vulkan is the active renderer");
                } else {
                    let description = self.renderer.as_ref().expect("renderer").deviceName().to_owned();
                    self.mainMenu.as_mut().expect("GUI runtime").openShaderSettings(minecraft, description);
                }
            }
            RuntimeGuiAction::OpenWorldShaderSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if minecraft.gameSettings.activeRenderBackend
                    != crate::launcher::RenderBackend::RenderBackend::OpenGl
                {
                    log::warn!("Ignored in-world shader-settings request while Vulkan is the active renderer");
                } else {
                    let description = self.renderer.as_ref().expect("renderer").deviceName().to_owned();
                    self.mainMenu.as_mut().expect("GUI runtime").openWorldShaderSettings(minecraft, description);
                    self.setWorldMouseGrabbed(false);
                }
            }
            RuntimeGuiAction::ReturnToVideoSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").returnToVideoSettings(minecraft);
            }
            RuntimeGuiAction::ReturnToWorldVideoSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").returnToWorldVideoSettings(minecraft);
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::SelectShaderPack(name) => {
                let gameDir = self.minecraft.as_ref().expect("Minecraft state").gameDir.clone();
                let mut shaders = Shaders::loadConfig(gameDir);
                if let Err(error) = shaders.setShaderPack(name.clone()) {
                    log::error!("Couldn't save selected shader pack {name:?}: {error}");
                } else {
                    log::info!("Selected OptiFine shader pack resource set: {name}");
                    self.renderer.as_mut().expect("desktop renderer").reloadShaderPack();
                }
            }
            RuntimeGuiAction::ReloadShaderPack => {
                log::info!("Reloading OptiFine shader pack after option changes");
                self.renderer.as_mut().expect("desktop renderer").reloadShaderPack();
            }
            RuntimeGuiAction::OpenShaderPackFolder => {
                let gameDir = self.minecraft.as_ref().expect("Minecraft state").gameDir.clone();
                let shaders = Shaders::loadConfig(gameDir);
                let path = shaders.shaderpacksdir;
                #[cfg(target_os="windows")] let result=std::process::Command::new("explorer").arg(&path).spawn();
                #[cfg(target_os="macos")] let result=std::process::Command::new("open").arg(&path).spawn();
                #[cfg(all(not(target_os="windows"),not(target_os="macos")))] let result=std::process::Command::new("xdg-open").arg(&path).spawn();
                if let Err(error)=result{log::error!("Couldn't open shader pack folder {}: {error}",path.display());}
            }
            RuntimeGuiAction::SetGamma(value) => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.gammaSetting = value.clamp(0.0, 1.0);
            }
            RuntimeGuiAction::SetRenderDistance(value) => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.renderDistanceChunks = value.clamp(2, 32);
            }
            RuntimeGuiAction::SetFramerate { limit, enableVsync } => {
                let limit = limit.clamp(5, FRAMERATE_LIMIT_MAX);
                let window = self.window.as_ref().expect("Minecraft window");
                let renderer = self.renderer.as_mut().expect("desktop renderer");
                renderer
                    .setVsync(window, enableVsync)
                    .context("failed applying the video-settings VSync change")?;
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.limitFramerate = limit;
                minecraft.gameSettings.enableVsync = enableVsync;
                self.nextFrameDeadline = Instant::now();
            }
            RuntimeGuiAction::ToggleGraphics => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.fancyGraphics = !minecraft.gameSettings.fancyGraphics;
            }
            RuntimeGuiAction::CycleAmbientOcclusion => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.ambientOcclusion =
                    (minecraft.gameSettings.ambientOcclusion + 1).rem_euclid(3);
            }
            RuntimeGuiAction::CycleGuiScale => {
                let window = self.window.as_ref().expect("Minecraft window");
                let displaySize = window
                    .current_monitor()
                    .map(|monitor| monitor.size())
                    .unwrap_or_else(|| window.inner_size());
                let maxScale = ((displaySize.width / 320).min(displaySize.height / 240) as i32).max(1);
                let extent = self.renderer.as_ref().expect("renderer").extent();
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                let unicode = minecraft.gameSettings.forceUnicodeFont
                    || self.mainMenu.as_ref().expect("GUI runtime").locale.is_unicode();
                minecraft.gameSettings.guiScale += 1;
                if unicode && minecraft.gameSettings.guiScale % 2 != 0 && minecraft.gameSettings.guiScale != 1 {
                    minecraft.gameSettings.guiScale += 1;
                }
                if minecraft.gameSettings.guiScale < 0 || minecraft.gameSettings.guiScale >= maxScale {
                    minecraft.gameSettings.guiScale = 0;
                }
                self.mainMenu.as_mut().expect("GUI runtime").resize(minecraft, extent.width, extent.height);
            }
            RuntimeGuiAction::ToggleFullscreen => {
                let window = self.window.as_ref().expect("Minecraft window");
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.fullScreen = !minecraft.gameSettings.fullScreen;
                minecraft.fullscreen = minecraft.gameSettings.fullScreen;
                window.set_fullscreen(
                    minecraft
                        .gameSettings
                        .fullScreen
                        .then(|| Fullscreen::Borderless(None)),
                );
                self.pendingResizeSince = Some(Instant::now());
            }
            RuntimeGuiAction::ToggleRenderBackend => {
                {
                    let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                    minecraft.gameSettings.renderBackend = minecraft.gameSettings.renderBackend.toggled();
                }
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                let runtime = self.mainMenu.as_mut().expect("GUI runtime");
                runtime.initCurrentScreen(minecraft);
                runtime.initWorldGui(minecraft);
            }
            RuntimeGuiAction::CloseVideoSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save options.txt: {error}");
                }
                self.mainMenu.as_mut().expect("GUI runtime").switchTo(minecraft, ScreenId::Options)?;
            }
            RuntimeGuiAction::OpenIngameMenu => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                let opened = self.mainMenu.as_mut().expect("GUI runtime").openIngameMenu(minecraft);
                if opened { self.setWorldMouseGrabbed(false); }
            }
            RuntimeGuiAction::ResumeWorld => {
                self.mainMenu.as_mut().expect("GUI runtime").resumeWorld();
                self.setWorldMouseGrabbed(true);
            }
            RuntimeGuiAction::ResumeWorldSaveOptions => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save options.txt: {error}");
                }
                self.mainMenu.as_mut().expect("GUI runtime").resumeWorld();
                self.setWorldMouseGrabbed(true);
            }
            RuntimeGuiAction::FinishSignEditor => {
                let result = self.mainMenu.as_mut().expect("GUI runtime").finishSignEditor();
                if let Err(message) = result {
                    log::error!("failed submitting CPacketUpdateSign: {message}");
                }
                self.setWorldMouseGrabbed(true);
            }
            RuntimeGuiAction::OpenGameOver(message) => {
                self.mainMenu.as_mut().expect("GUI runtime").openGameOver(message);
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::RespawnPlayer => {
                let result = self.mainMenu.as_mut().expect("GUI runtime").respawnPlayer();
                if let Err(message) = result {
                    log::error!("failed sending CPacketClientStatus(PERFORM_RESPAWN): {message}");
                }
                self.setWorldMouseGrabbed(true);
            }
            RuntimeGuiAction::OpenDeathQuitConfirm => {
                self.mainMenu.as_mut().expect("GUI runtime").openDeathQuitConfirm();
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::ConfirmDeathQuit(result) => {
                if result {
                    self.integratedServer.take();
                    let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                    self.mainMenu.as_mut().expect("GUI runtime").leaveWorldToMainMenu(minecraft)?;
                    self.setWorldMouseGrabbed(false);
                } else {
                    let result = self.mainMenu.as_mut().expect("GUI runtime").cancelDeathQuitAndRespawn();
                    if let Err(message) = result {
                        log::error!("failed sending CPacketClientStatus(PERFORM_RESPAWN): {message}");
                    }
                    self.setWorldMouseGrabbed(true);
                }
            }
            RuntimeGuiAction::LeaveWorldToMainMenu => {
                self.integratedServer.take();
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").leaveWorldToMainMenu(minecraft)?;
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::OpenWorldOptions => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openWorldOptions(minecraft);
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::OpenWorldVideoSettings => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save options.txt: {error}");
                }
                self.mainMenu.as_mut().expect("GUI runtime").openWorldVideoSettings(minecraft);
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::ReturnToWorldOptions => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save options.txt: {error}");
                }
                self.mainMenu.as_mut().expect("GUI runtime").openWorldOptions(minecraft);
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::ReturnToIngameMenu => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) {
                    log::error!("Couldn't save options.txt: {error}");
                }
                self.mainMenu.as_mut().expect("GUI runtime").returnToIngameMenu(minecraft);
                self.setWorldMouseGrabbed(false);
            }
            RuntimeGuiAction::DisconnectWorld => {
                let wasIntegrated=self.integratedServer.take().is_some();
                self.setWorldMouseGrabbed(false);
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if wasIntegrated {
                    self.mainMenu.as_mut().expect("GUI runtime").leaveWorldToMainMenu(minecraft)?;
                } else {
                    self.mainMenu.as_mut().expect("GUI runtime").returnToMultiplayer(minecraft);
                }
            }
            RuntimeGuiAction::OpenDirectConnect => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openDirectConnect(minecraft);
            }
            RuntimeGuiAction::OpenAddServer => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openAddServer(minecraft, None, None);
            }
            RuntimeGuiAction::OpenEditServer { index, server } => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openAddServer(minecraft, Some(index), Some(server));
            }
            RuntimeGuiAction::OpenDeleteConfirm { index, serverName } => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openDeleteConfirm(minecraft, index, serverName);
            }
            RuntimeGuiAction::SaveServer { editingIndex, server } => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").saveServerAndReturn(minecraft, editingIndex, server)?;
            }
            RuntimeGuiAction::DeleteServer { index } => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").deleteServerAndReturn(minecraft, index)?;
            }
            RuntimeGuiAction::Connect(server) => {
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                minecraft.gameSettings.lastServer = server.serverIP.clone();
                if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) { log::error!("Couldn't save options.txt: {error}"); }
                self.mainMenu.as_mut().expect("GUI runtime").openConnecting(minecraft, server);
            }
            RuntimeGuiAction::CancelConnecting => {
                self.setWorldMouseGrabbed(false);
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                if self.integratedServer.take().is_some() {
                    self.mainMenu.as_mut().expect("GUI runtime").switchTo(minecraft, ScreenId::WorldSelection)?;
                } else {
                    self.mainMenu.as_mut().expect("GUI runtime").cancelConnecting(minecraft);
                }
            }
            RuntimeGuiAction::OpenDisconnected { reasonKey, message } => {
                // A failed local connection must release the integrated server
                // and its save/session lock before the disconnected GUI owns
                // control, matching Minecraft#loadWorld(null) shutdown.
                self.integratedServer.take();
                self.setWorldMouseGrabbed(false);
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openDisconnected(minecraft, reasonKey, message);
            }
            RuntimeGuiAction::OpenDownloadTerrain => {
                self.setWorldMouseGrabbed(false);
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openDownloadTerrain(minecraft);
            }
            RuntimeGuiAction::OpenWorld => {
                let minecraft = self.minecraft.as_ref().expect("Minecraft state");
                self.mainMenu.as_mut().expect("GUI runtime").openWorld(minecraft);
                self.setWorldMouseGrabbed(true);
            }
            RuntimeGuiAction::ReturnToMultiplayer { lastServer } => {
                self.setWorldMouseGrabbed(false);
                let wasIntegrated=self.integratedServer.take().is_some();
                let minecraft = self.minecraft.as_mut().expect("Minecraft state");
                if wasIntegrated {
                    self.mainMenu.as_mut().expect("GUI runtime").switchTo(minecraft, ScreenId::WorldSelection)?;
                } else {
                    if let Some(lastServer) = lastServer {
                        minecraft.gameSettings.lastServer = lastServer;
                        if let Err(error) = minecraft.gameSettings.saveOptions(&minecraft.gameDir) { log::error!("Couldn't save options.txt: {error}"); }
                    }
                    self.mainMenu.as_mut().expect("GUI runtime").returnToMultiplayer(minecraft);
                }
            }
            RuntimeGuiAction::NotConnected(className) => log::info!("MCP screen/action not yet connected: {className}"),
        }
        Ok(false)
    }
}

impl ApplicationHandler for MinecraftApplication {
    fn resumed(&mut self, eventLoop: &ActiveEventLoop) {
        if self.window.is_some() { return; }
        let minecraft = self.minecraft.as_ref().expect("Minecraft application state");
        let mut attributes = Window::default_attributes()
            .with_title("Minecraft 1.12.2")
            .with_inner_size(LogicalSize::new(minecraft.displayWidth.max(1) as f64, minecraft.displayHeight.max(1) as f64))
            .with_min_inner_size(LogicalSize::new(320.0_f64, 240.0_f64));
        if minecraft.fullscreen { attributes = attributes.with_fullscreen(Some(Fullscreen::Borderless(None))); }
        let (window, renderer) = match DesktopRenderer::create(eventLoop, attributes, &minecraft.gameSettings, &minecraft.gameDir) {
            Ok(output) => output,
            Err(error) => { self.fail(eventLoop, error.context("failed initializing Minecraft desktop renderer")); return; }
        };
        let extent = renderer.extent();
        let activeBackend = renderer.backend();
        self.minecraft
            .as_mut()
            .expect("Minecraft application state")
            .gameSettings
            .activeRenderBackend = activeBackend;
        let minecraft = self.minecraft.as_ref().expect("Minecraft application state");
        let mainMenu = match MainMenuRuntime::new(minecraft, extent.width, extent.height) {
            Ok(mainMenu) => mainMenu,
            Err(error) => { self.fail(eventLoop, error.context("failed initializing GuiMainMenu")); return; }
        };
        self.renderer = Some(renderer); self.mainMenu = Some(mainMenu); self.window = Some(window);
        let now = Instant::now();
        self.nextFrameDeadline = now;
        self.timerAccumulator = 0.0;
        self.lastTimerSync = now;
        self.requestRedraw();
    }

    fn window_event(&mut self, eventLoop: &ActiveEventLoop, windowId: WindowId, event: WindowEvent) {
        if self.window.as_ref().map(Window::id) != Some(windowId) { return; }
        if let Some(runtime) = self.mainMenu.as_mut() {
            runtime.syncOpenContainerGui();
        }
        self.applyRuntimeMouseFocusRequest();
        let mut fatalError = None;
        match event {
            WindowEvent::CloseRequested => eventLoop.exit(),
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                self.pendingResizeSince = Some(Instant::now());
                eventLoop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + RESIZE_DEBOUNCE));
            }
            WindowEvent::Focused(false) => {
                self.windowFocused = false;
                self.setWorldMouseGrabbed(false);
                if let Some(runtime) = self.mainMenu.as_mut() {
                    runtime.clearMovementKeys();
                    // GuiScreen is no longer guaranteed to receive the
                    // matching mouse-up after focus loss. Clear the same
                    // transient GuiContainer gesture state rather than
                    // allowing a stale QUICK_CRAFT to survive Alt-Tab.
                    if let Some(container) = runtime.activeGuiContainerMut() {
                        container.resetInteraction();
                    }
                    runtime.lastInventoryClick = None;
                    runtime.inventoryShiftClickedStack = ItemStack::EMPTY;
                    runtime.guiCreative.wasClicking = false;
                    runtime.guiCreative.isScrolling = false;
                }
            }
            WindowEvent::Focused(true) => {
                self.windowFocused = true;
                self.applyRuntimeMouseFocusRequest();
            }
            WindowEvent::ModifiersChanged(modifiers) => self.keyboardModifiers = modifiers.state(),
            WindowEvent::CursorMoved { position, .. } => {
                let dragAction = if let (Some(renderer), Some(runtime)) = (self.renderer.as_ref(), self.mainMenu.as_mut()) {
                    runtime.mousePosition = position;
                    runtime.mouseInsideWindow = true;
                    let extent = renderer.extent();
                    runtime.mouseDragged(extent.width, extent.height)
                } else {
                    None
                };
                if let Some(action) = dragAction {
                    if let Err(error) = self.applyGuiAction(action) {
                        fatalError = Some(error.context("failed applying dragged video option"));
                    }
                }
                if self.mainMenu.as_ref().is_some_and(|runtime| !runtime.isAnimated()) { self.requestRedraw(); }
            }
            WindowEvent::CursorEntered { .. } => {
                if let Some(runtime) = self.mainMenu.as_mut() { runtime.mouseInsideWindow = true; }
                if self.mainMenu.as_ref().is_some_and(|runtime| !runtime.isAnimated()) { self.requestRedraw(); }
            }
            WindowEvent::CursorLeft { .. } => {
                if let Some(runtime) = self.mainMenu.as_mut() { runtime.mouseInsideWindow = false; }
                if self.mainMenu.as_ref().is_some_and(|runtime| !runtime.isAnimated()) { self.requestRedraw(); }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let lines = match delta { MouseScrollDelta::LineDelta(_, y) => y, MouseScrollDelta::PixelDelta(value) => (value.y / 20.0) as f32 };
                let inWorld = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isWorld);
                let inventoryOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isInventoryOpen);
                let chatOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isChatOpen);
                let changed = if inWorld && chatOpen {
                    // GuiNewChat#scroll uses one line while Shift is held and
                    // seven lines otherwise, matching the 1.12.2 mouse-wheel path.
                    let step = if self.keyboardModifiers.shift_key() { 1 } else { 7 };
                    let amount = if lines > 0.0 { step } else if lines < 0.0 { -step } else { 0 };
                    let focusedHeight = self.minecraft.as_ref().map_or(1.0, |minecraft| minecraft.gameSettings.chatHeightFocused);
                    self.mainMenu.as_mut().is_some_and(|runtime| runtime.scrollChat(amount, focusedHeight))
                } else if inWorld && inventoryOpen {
                    let wheelDelta = if lines > 0.0 { 1 } else if lines < 0.0 { -1 } else { 0 };
                    self.mainMenu.as_mut().is_some_and(|runtime| runtime.creativeInventoryOpen && runtime.creativeInventoryScroll(wheelDelta))
                } else if inWorld && self.worldMouseGrabbed {
                    let wheelDelta = if lines > 0.0 { 1 } else if lines < 0.0 { -1 } else { 0 };
                    self.mainMenu.as_mut().is_some_and(|runtime| runtime.worldScroll(wheelDelta))
                } else {
                    self.mainMenu.as_mut().is_some_and(|runtime| runtime.scroll(lines))
                };
                if changed { self.requestRedraw(); }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button, .. } => {
                let inWorld = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isWorld);
                let inventoryOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isInventoryOpen);
                let chatOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isChatOpen);
                let worldGuiOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isWorldGuiOpen);
                if inWorld {
                    if worldGuiOpen && mouse_button_index(button).is_some() {
                        let action = if let (Some(renderer), Some(runtime), Some(minecraft)) = (
                            self.renderer.as_ref(),
                            self.mainMenu.as_mut(),
                            self.minecraft.as_ref(),
                        ) {
                            let extent = renderer.extent();
                            runtime.mouseClicked(
                                extent.width,
                                extent.height,
                                mouse_button_index(button).unwrap_or(0),
                                self.keyboardModifiers.shift_key(),
                                &minecraft.gameSettings,
                                minecraft.getSession().getToken(),
                            )
                        } else { None };
                        if let Some(action) = action {
                            match self.applyGuiAction(action) {
                                Ok(true) => { eventLoop.exit(); return; }
                                Ok(false) => self.requestRedraw(),
                                Err(error) => fatalError = Some(error.context("failed applying world GUI action")),
                            }
                        }
                    } else if worldGuiOpen {
                        // GuiScreen owns every mouse button while displayed.
                    } else if chatOpen {
                        // GuiChat owns mouse focus while open. Clickable chat
                        // components are added with the full ITextComponent
                        // style/event port; until then never leak a click into
                        // attack/use or recapture the gameplay cursor.
                        self.requestRedraw();
                    } else if inventoryOpen && matches!(button, MouseButton::Left | MouseButton::Right | MouseButton::Middle) {
                        let interaction = if let (Some(renderer), Some(runtime)) =
                            (self.renderer.as_ref(), self.mainMenu.as_mut())
                        {
                            let extent = renderer.extent();
                            Some(runtime.inventoryMouseClicked(extent.width, extent.height, button, self.keyboardModifiers))
                        } else { None };
                        match interaction {
                            Some(Ok(true)) => self.requestRedraw(),
                            Some(Ok(false)) | None => {}
                            Some(Err(message)) => log::error!("failed sending inventory click: {message}"),
                        }
                    } else if !self.worldMouseGrabbed && button == MouseButton::Left {
                        self.setWorldMouseGrabbed(true);
                        self.requestRedraw();
                    } else if self.worldMouseGrabbed {
                        let binding = self.minecraft.as_ref().and_then(|minecraft| {
                            mouse_code(button).and_then(|code| minecraft.gameSettings.keyBindingIdForCode(code))
                        });
                        let mut redraw = false;
                        let mut releaseMouse = false;
                        let mut interactionError = None;
                        if let Some(binding) = binding {
                            match binding {
                                KeyBindingId::Attack | KeyBindingId::UseItem => {
                                    if let Some(runtime) = self.mainMenu.as_mut() {
                                        match runtime.worldActionButton(binding, true) {
                                            Ok(changed) => redraw |= changed,
                                            Err(message) => interactionError = Some(message),
                                        }
                                    }
                                }
                                KeyBindingId::Forward | KeyBindingId::Back | KeyBindingId::Left
                                | KeyBindingId::Right | KeyBindingId::Jump | KeyBindingId::Sneak
                                | KeyBindingId::Sprint => {
                                    if let Some(runtime) = self.mainMenu.as_mut() {
                                        runtime.setMovementBinding(binding, true);
                                    }
                                }
                                KeyBindingId::PlayerList => {
                                    if let Some(runtime) = self.mainMenu.as_mut() {
                                        redraw |= runtime.setPlayerListKeyDown(true);
                                    }
                                }
                                KeyBindingId::Hotbar1 | KeyBindingId::Hotbar2 | KeyBindingId::Hotbar3
                                | KeyBindingId::Hotbar4 | KeyBindingId::Hotbar5 | KeyBindingId::Hotbar6
                                | KeyBindingId::Hotbar7 | KeyBindingId::Hotbar8 | KeyBindingId::Hotbar9
                                | KeyBindingId::Drop | KeyBindingId::SwapHands => {
                                    if let Some(runtime) = self.mainMenu.as_mut() {
                                        match runtime.worldHotbarBinding(binding, self.keyboardModifiers) {
                                            Ok(changed) => redraw |= changed,
                                            Err(message) => interactionError = Some(message),
                                        }
                                    }
                                }
                                KeyBindingId::Inventory => {
                                    if self.mainMenu.as_mut().is_some_and(MainMenuRuntime::openInventory) {
                                        releaseMouse = true;
                                        redraw = true;
                                    }
                                }
                                KeyBindingId::Chat | KeyBindingId::Command => {
                                    let allowed = binding == KeyBindingId::Command
                                        || self.minecraft.as_ref().is_some_and(|minecraft| {
                                            minecraft.gameSettings.chatVisibility != EnumChatVisibility::Hidden
                                        });
                                    if allowed {
                                        let prefix = if binding == KeyBindingId::Command { "/" } else { "" };
                                        if self.mainMenu.as_mut().is_some_and(|runtime| runtime.openChat(prefix)) {
                                            releaseMouse = true;
                                            redraw = true;
                                        }
                                    }
                                }
                                KeyBindingId::TogglePerspective => {
                                    if let Some(minecraft) = self.minecraft.as_mut() {
                                        minecraft.gameSettings.thirdPersonView =
                                            (minecraft.gameSettings.thirdPersonView + 1).rem_euclid(3);
                                        redraw = true;
                                    }
                                }
                                _ => {}
                            }
                        }
                        if let Some(message) = interactionError {
                            log::error!("failed sending bound mouse action: {message}");
                        }
                        if releaseMouse { self.setWorldMouseGrabbed(false); }
                        if redraw { self.requestRedraw(); }
                    }
                } else if mouse_button_index(button).is_some() {
                    let action = if let (Some(renderer), Some(runtime), Some(minecraft)) = (
                        self.renderer.as_ref(),
                        self.mainMenu.as_mut(),
                        self.minecraft.as_ref(),
                    ) {
                        let extent = renderer.extent();
                        runtime.mouseClicked(
                            extent.width,
                            extent.height,
                            mouse_button_index(button).unwrap_or(0),
                            self.keyboardModifiers.shift_key(),
                            &minecraft.gameSettings,
                            minecraft.getSession().getToken(),
                        )
                    } else { None };
                    if let Some(action) = action {
                        match self.applyGuiAction(action) {
                            Ok(true) => { eventLoop.exit(); return; }
                            Ok(false) => self.requestRedraw(),
                            Err(error) => fatalError = Some(error.context("failed applying GUI action")),
                        }
                    }
                }
            }
            WindowEvent::MouseInput { state: ElementState::Released, button, .. } => {
                let inWorld = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isWorld);
                let inventoryOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isInventoryOpen);
                let chatOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isChatOpen);
                let worldGuiOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isWorldGuiOpen);
                if inWorld && worldGuiOpen {
                    if button == MouseButton::Left {
                        if let (Some(renderer), Some(runtime)) = (self.renderer.as_ref(), self.mainMenu.as_mut()) {
                            let extent = renderer.extent();
                            runtime.mouseReleased(extent.width, extent.height);
                        }
                    }
                } else if inWorld && chatOpen {
                    // Consumed by GuiChat; see the matching press branch.
                } else if inWorld && inventoryOpen && matches!(button, MouseButton::Left | MouseButton::Right | MouseButton::Middle) {
                    let interaction = if let (Some(renderer), Some(runtime)) =
                        (self.renderer.as_ref(), self.mainMenu.as_mut())
                    {
                        let extent = renderer.extent();
                        Some(runtime.inventoryMouseReleased(
                            extent.width,
                            extent.height,
                            button,
                            self.keyboardModifiers,
                        ))
                    } else {
                        None
                    };
                    match interaction {
                        Some(Ok(true)) => self.requestRedraw(),
                        Some(Ok(false)) | None => {}
                        Some(Err(message)) => log::error!("failed releasing inventory click: {message}"),
                    }
                } else if inWorld && self.worldMouseGrabbed {
                    let binding = self.minecraft.as_ref().and_then(|minecraft| {
                        mouse_code(button).and_then(|code| minecraft.gameSettings.keyBindingIdForCode(code))
                    });
                    if let (Some(binding), Some(runtime)) = (binding, self.mainMenu.as_mut()) {
                        match binding {
                            KeyBindingId::Attack | KeyBindingId::UseItem => {
                                if let Err(message) = runtime.worldActionButton(binding, false) {
                                    log::error!("failed releasing in-world bound action: {message}");
                                }
                            }
                            KeyBindingId::Forward | KeyBindingId::Back | KeyBindingId::Left
                            | KeyBindingId::Right | KeyBindingId::Jump | KeyBindingId::Sneak
                            | KeyBindingId::Sprint => {
                                runtime.setMovementBinding(binding, false);
                            }
                            KeyBindingId::PlayerList => {
                                runtime.setPlayerListKeyDown(false);
                            }
                            _ => {}
                        }
                    }
                } else if button == MouseButton::Left {
                    if let (Some(renderer), Some(runtime)) = (self.renderer.as_ref(), self.mainMenu.as_mut()) {
                        let extent = renderer.extent();
                        runtime.mouseReleased(extent.width, extent.height);
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let keyCode = match event.physical_key { PhysicalKey::Code(code) => Some(code), _ => None };
                let pressed = event.state == ElementState::Pressed;
                let inWorld = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isWorld);
                let bindingId = keyCode
                    .and_then(lwjgl_from_winit)
                    .and_then(|code| self.minecraft.as_ref().and_then(|minecraft| {
                        minecraft.gameSettings.keyBindingIdForCode(code)
                    }));
                let mut handled = false;

                // MCP `dispatchKeypresses`: fullscreen is a configurable key
                // binding and remains active while GUI screens are open. Do
                // not dispatch it while GuiControls itself is capturing a new
                // binding, otherwise assigning the fullscreen key would also
                // toggle the window during the capture event.
                if pressed
                    && bindingId == Some(KeyBindingId::Fullscreen)
                    && !self.mainMenu.as_ref().is_some_and(MainMenuRuntime::controlsAwaitingBinding)
                {
                    handled = true;
                    if let Err(error) = self.applyGuiAction(RuntimeGuiAction::ToggleFullscreen) {
                        fatalError = Some(error.context("failed applying bound fullscreen key"));
                    } else {
                        self.requestRedraw();
                    }
                }

                if keyCode == Some(KeyCode::KeyC) {
                    self.debugCrashKeyDown = pressed;
                    if !pressed {
                        self.debugCrashKeyPressTime = None;
                    } else if self.debugKeyDown && self.debugCrashKeyPressTime.is_none() {
                        self.debugCrashKeyPressTime = Some(Instant::now());
                    }
                }

                // MCP `Minecraft#processKeyF3`: F3 itself toggles the debug
                // overlay on release unless another supported debug chord was
                // consumed while it was held.
                if inWorld && self.worldMouseGrabbed && keyCode == Some(KeyCode::F3) {
                    if pressed {
                        if !event.repeat {
                            self.debugKeyDown = true;
                            self.debugActionUsed = false;
                            if self.debugCrashKeyDown && self.debugCrashKeyPressTime.is_none() {
                                self.debugCrashKeyPressTime = Some(Instant::now());
                                self.debugActionUsed = true;
                            }
                        }
                    } else {
                        if self.debugKeyDown && !self.debugActionUsed {
                            if let Some(minecraft) = self.minecraft.as_mut() {
                                minecraft.gameSettings.showDebugInfo =
                                    !minecraft.gameSettings.showDebugInfo;
                                minecraft.gameSettings.showDebugProfilerChart =
                                    minecraft.gameSettings.showDebugInfo
                                        && self.keyboardModifiers.shift_key();
                                minecraft.gameSettings.showLagometer =
                                    minecraft.gameSettings.showDebugInfo
                                        && self.keyboardModifiers.alt_key();
                            }
                        }
                        self.debugKeyDown = false;
                        self.debugActionUsed = false;
                        self.debugCrashKeyPressTime = None;
                    }
                    handled = true;
                    self.requestRedraw();
                }

                if inWorld
                    && pressed
                    && !handled
                    && self.debugKeyDown
                    && self.worldMouseGrabbed
                {
                    if let Some(code) = keyCode {
                        if self.processDebugChord(code) {
                            self.debugActionUsed = true;
                            handled = true;
                            self.requestRedraw();
                        }
                    }
                }

                // MCP `keyBindTogglePerspective`: F5 cycles 0 -> 1 -> 2 -> 0
                // only while gameplay owns the keyboard. The value is runtime
                // state and is intentionally not serialized to options.txt.
                if inWorld
                    && pressed
                    && self.worldMouseGrabbed
                    && bindingId == Some(KeyBindingId::TogglePerspective)
                    && !self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isModalWorldGuiOpen)
                {
                    if let Some(minecraft) = self.minecraft.as_mut() {
                        minecraft.gameSettings.thirdPersonView =
                            (minecraft.gameSettings.thirdPersonView + 1).rem_euclid(3);
                    }
                    handled = true;
                    self.requestRedraw();
                }

                if inWorld && pressed && self.worldMouseGrabbed && !handled
                    && !self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isChatOpen)
                {
                    let chatVisible = self.minecraft.as_ref().is_some_and(|minecraft| {
                        minecraft.gameSettings.chatVisibility != EnumChatVisibility::Hidden
                    });
                    let defaultText = match bindingId {
                        Some(KeyBindingId::Chat) if chatVisible => Some(""),
                        Some(KeyBindingId::Command) => Some("/"),
                        _ => None,
                    };
                    if let Some(defaultText) = defaultText {
                        if self.mainMenu.as_mut().is_some_and(|runtime| runtime.openChat(defaultText)) {
                            self.setWorldMouseGrabbed(false);
                            self.requestRedraw();
                            handled = true;
                        }
                    }
                }

                if inWorld && !handled
                    && self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isChatOpen)
                {
                    if pressed {
                        let (focusedHeight, chatWidth, chatScale) = self.minecraft.as_ref().map_or(
                            (1.0, 1.0, 1.0),
                            |minecraft| (
                                minecraft.gameSettings.chatHeightFocused,
                                minecraft.gameSettings.chatWidth,
                                minecraft.gameSettings.chatScale,
                            ),
                        );
                        let result = keyCode.and_then(|code| self.mainMenu.as_mut().map(|runtime| {
                            runtime.chatKeyPressed(
                                code,
                                self.keyboardModifiers,
                                focusedHeight,
                                chatWidth,
                                chatScale,
                            )
                        }));
                        let mut closeChat = false;
                        match result {
                            Some(Ok((keyHandled, closed))) => {
                                closeChat = closed;
                                if keyHandled { self.requestRedraw(); }
                            }
                            Some(Err(message)) => log::error!("failed handling chat key: {message}"),
                            None => {}
                        }
                        if !closeChat && !self.keyboardModifiers.control_key() && !self.keyboardModifiers.alt_key() {
                            if let Some(text) = event.text.as_ref() {
                                if self.mainMenu.as_mut().is_some_and(|runtime| runtime.chatTypedText(text.as_str())) {
                                    self.requestRedraw();
                                }
                            }
                        }
                        if closeChat {
                            self.setWorldMouseGrabbed(true);
                            self.requestRedraw();
                        }
                    }
                    // GuiChat is the current screen: even unhandled physical
                    // keys must not reach movement, inventory or hotbar input.
                    handled = true;
                }

                if inWorld
                    && pressed
                    && !handled
                    && self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isWorldGuiOpen)
                    && (keyCode != Some(KeyCode::Escape)
                        || self.mainMenu.as_ref().is_some_and(MainMenuRuntime::controlsAwaitingBinding))
                {
                    let action = keyCode.and_then(|code| {
                        self.mainMenu.as_mut().and_then(|runtime| {
                            runtime.keyPressed(code, self.keyboardModifiers, event.text.as_deref())
                        })
                    });
                    if let Some(action) = action {
                        match self.applyGuiAction(action) {
                            Ok(true) => { eventLoop.exit(); return; }
                            Ok(false) => self.requestRedraw(),
                            Err(error) => fatalError = Some(error.context("failed applying world GUI key action")),
                        }
                    } else if !self.keyboardModifiers.control_key() && !self.keyboardModifiers.alt_key() {
                        if let Some(text) = event.text.as_ref() {
                            if self.mainMenu.as_mut().is_some_and(|runtime| runtime.typedText(text.as_str())) {
                                self.requestRedraw();
                            }
                        }
                    }
                    // Every physical key belongs to the active GuiScreen even
                    // when GuiEditSign declines the character.
                    handled = true;
                }

                if inWorld && self.worldMouseGrabbed && !handled
                    && matches!(bindingId, Some(KeyBindingId::Attack | KeyBindingId::UseItem))
                {
                    if let (Some(binding), Some(runtime)) = (bindingId, self.mainMenu.as_mut()) {
                        match runtime.worldActionButton(binding, pressed) {
                            Ok(redraw) => {
                                handled = true;
                                if redraw { self.requestRedraw(); }
                            }
                            Err(message) => {
                                handled = true;
                                log::error!("failed sending bound attack/use action: {message}");
                            }
                        }
                    }
                }

                if inWorld && pressed && !handled && bindingId == Some(KeyBindingId::Inventory) {
                    let inventoryOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isInventoryOpen);
                    if inventoryOpen {
                        let result = self.mainMenu.as_mut().map(MainMenuRuntime::closeInventory);
                        match result {
                            Some(Ok(true)) => {
                                self.setWorldMouseGrabbed(true);
                                self.requestRedraw();
                                handled = true;
                            }
                            Some(Err(message)) => log::error!("failed closing inventory: {message}"),
                            _ => {}
                        }
                    } else if self.worldMouseGrabbed
                        && self.mainMenu.as_mut().is_some_and(MainMenuRuntime::openInventory)
                    {
                        self.setWorldMouseGrabbed(false);
                        self.requestRedraw();
                        handled = true;
                    }
                }

                if inWorld && pressed && !handled
                    && self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isInventoryOpen)
                {
                    let result = if let (Some(code), Some(renderer), Some(runtime)) =
                        (keyCode, self.renderer.as_ref(), self.mainMenu.as_mut())
                    {
                        let extent = renderer.extent();
                        Some(runtime.inventoryKeyPressed(
                            extent.width,
                            extent.height,
                            code,
                            bindingId,
                            self.keyboardModifiers,
                            event.text.as_ref().map(|text| text.as_str()),
                        ))
                    } else { None };
                    match result {
                        Some(Ok(true)) => { handled = true; self.requestRedraw(); }
                        Some(Ok(false)) | None => {}
                        Some(Err(message)) => log::error!("failed sending inventory key click: {message}"),
                    }
                }

                if inWorld
                    && pressed
                    && !handled
                    && self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isInventoryOpen)
                    && self.mainMenu.as_ref().is_some_and(|runtime| !runtime.creativeInventoryOpen)
                    && !self.keyboardModifiers.control_key()
                    && !self.keyboardModifiers.alt_key()
                {
                    if let Some(text) = event.text.as_ref() {
                        let result = self.mainMenu.as_mut().map(|runtime| runtime.repairTypedText(text.as_str()));
                        match result {
                            Some(Ok(true)) => {
                                handled = true;
                                self.requestRedraw();
                            }
                            Some(Ok(false)) | None => {}
                            Some(Err(message)) => log::error!("failed sending anvil rename text: {message}"),
                        }
                    }
                }

                if inWorld
                    && pressed
                    && !handled
                    && self.mainMenu.as_ref().is_some_and(|runtime| runtime.creativeInventoryOpen)
                    && !self.keyboardModifiers.control_key()
                    && !self.keyboardModifiers.alt_key()
                {
                    if let Some(text) = event.text.as_ref() {
                        if self.mainMenu.as_mut().is_some_and(|runtime| runtime.creativeInventoryTypedText(text.as_str())) {
                            handled = true;
                            self.requestRedraw();
                        }
                    }
                }

                if inWorld && self.worldMouseGrabbed && !handled && bindingId == Some(KeyBindingId::PlayerList)
                    && !self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isInventoryOpen)
                {
                    if self.mainMenu.as_mut().is_some_and(|runtime| runtime.setPlayerListKeyDown(pressed)) {
                        self.requestRedraw();
                    }
                    handled = true;
                }

                if inWorld && self.worldMouseGrabbed && !handled {
                    if let (Some(binding), Some(runtime)) = (bindingId, self.mainMenu.as_mut()) {
                        handled = runtime.setMovementBinding(binding, pressed);
                    }
                }

                if inWorld && self.worldMouseGrabbed && pressed && !handled {
                    let hotbarResult = if let (Some(binding), Some(runtime)) =
                        (bindingId, self.mainMenu.as_mut())
                    {
                        Some(runtime.worldHotbarBinding(binding, self.keyboardModifiers))
                    } else {
                        None
                    };
                    match hotbarResult {
                        Some(Ok(true)) => { handled = true; self.requestRedraw(); }
                        Some(Ok(false)) | None => {}
                        Some(Err(message)) => {
                            log::error!("failed sending in-world key action: {message}")
                        }
                    }
                }

                if pressed && !handled && keyCode == Some(KeyCode::Escape)
                    && self.mainMenu.as_ref().is_some_and(MainMenuRuntime::controlsAwaitingBinding)
                {
                    let action = self.mainMenu.as_mut().and_then(|runtime| {
                        runtime.keyPressed(KeyCode::Escape, self.keyboardModifiers, event.text.as_deref())
                    });
                    if let Some(action) = action {
                        handled = true;
                        match self.applyGuiAction(action) {
                            Ok(true) => { eventLoop.exit(); return; }
                            Ok(false) => self.requestRedraw(),
                            Err(error) => fatalError = Some(error.context("failed clearing selected key binding")),
                        }
                    }
                }

                if pressed && !handled && keyCode == Some(KeyCode::Escape) {
                    let inventoryOpen = self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isInventoryOpen);
                    if inWorld && inventoryOpen {
                        let result = self.mainMenu.as_mut().map(MainMenuRuntime::closeInventory);
                        match result {
                            Some(Ok(true)) => {
                                self.setWorldMouseGrabbed(true);
                                self.requestRedraw();
                                handled = true;
                            }
                            Some(Err(message)) => log::error!("failed closing inventory: {message}"),
                            _ => {}
                        }
                    } else if inWorld {
                        let action = match self.mainMenu.as_mut().and_then(|runtime| runtime.worldGuiScreen.as_mut()) {
                            Some(WorldGuiScreen::IngameMenu(_)) => RuntimeGuiAction::ResumeWorld,
                            Some(WorldGuiScreen::Options(_)) => RuntimeGuiAction::ResumeWorldSaveOptions,
                            Some(WorldGuiScreen::Controls(_)) => RuntimeGuiAction::ResumeWorld,
                            Some(WorldGuiScreen::VideoSettings(_))
                            | Some(WorldGuiScreen::SoundSettings(_))
                            | Some(WorldGuiScreen::ChatSettings(_))
                            | Some(WorldGuiScreen::SkinSettings(_))
                            | Some(WorldGuiScreen::Language(_)) => RuntimeGuiAction::ReturnToWorldOptions,
                            Some(WorldGuiScreen::ShaderSettings(screen)) if screen.isOptionsView() => {
                                if screen.closeOptionsView() {
                                    RuntimeGuiAction::ReloadShaderPack
                                } else {
                                    RuntimeGuiAction::None
                                }
                            }
                            Some(WorldGuiScreen::ShaderSettings(_)) => RuntimeGuiAction::ReturnToWorldVideoSettings,
                            Some(WorldGuiScreen::ResourcePacks(screen)) => {
                                if screen.cancelConfirmation() {
                                    RuntimeGuiAction::None
                                } else {
                                    RuntimeGuiAction::ReturnToWorldOptions
                                }
                            }
                            Some(WorldGuiScreen::EditSign(_)) => RuntimeGuiAction::FinishSignEditor,
                            Some(WorldGuiScreen::GameOver(_))
                            | Some(WorldGuiScreen::GameOverConfirm { .. }) => RuntimeGuiAction::None,
                            None => RuntimeGuiAction::OpenIngameMenu,
                        };
                        handled = true;
                        match self.applyGuiAction(action) {
                            Ok(true) => { eventLoop.exit(); return; }
                            Ok(false) => self.requestRedraw(),
                            Err(error) => fatalError = Some(error.context("failed applying in-world Escape action")),
                        }
                    } else {
                        let action = self.mainMenu.as_mut().and_then(MainMenuRuntime::escapeAction);
                        if let Some(action) = action {
                            handled = true;
                            match self.applyGuiAction(action) {
                                Ok(true) => { eventLoop.exit(); return; }
                                Ok(false) => self.requestRedraw(),
                                Err(error) => fatalError = Some(error.context("failed applying Escape action")),
                            }
                        }
                    }
                } else if pressed && !handled {
                    if let (Some(code), Some(runtime)) = (keyCode, self.mainMenu.as_mut()) {
                        if let Some(action) = runtime.keyPressed(code, self.keyboardModifiers, event.text.as_deref()) {
                            handled = true;
                            match self.applyGuiAction(action) {
                                Ok(true) => { eventLoop.exit(); return; }
                                Ok(false) => self.requestRedraw(),
                                Err(error) => fatalError = Some(error.context("failed applying keyboard GUI action")),
                            }
                        }
                    }
                }

                if pressed && !handled && !self.keyboardModifiers.control_key() && !self.keyboardModifiers.alt_key() {
                    if let Some(text) = event.text.as_ref() {
                        if self.mainMenu.as_mut().is_some_and(|runtime| runtime.typedText(text.as_str())) { self.requestRedraw(); }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.redrawPending = false;
                let mut frameProfileSample = None;
                if self.pendingResizeSince.is_none() {
                    let frameInterval = self.currentFrameInterval();
                    // `Display.sync` budgets the complete frame, not an extra
                    // interval after rendering. Anchor the next deadline at the
                    // start of preparation so a 30 FPS menu with 5 ms of work
                    // waits about 28 ms rather than 33 ms more.
                    let frameStarted = Instant::now();
                    if let (Some(window), Some(renderer), Some(runtime), Some(minecraft)) = (
                        self.window.as_ref(), self.renderer.as_mut(), self.mainMenu.as_mut(), self.minecraft.as_ref(),
                    ) {
                        let extent = renderer.extent();
                        if extent.width > 0 && extent.height > 0 {
                            // MCP `Timer#field_194147_b`: partialTicks is the
                            // tick-pump residual — the time since the last
                            // pumped tick in ticks — not the frame interval.
                            // `updateTimer` folds the frame interval into the
                            // accumulator and the render consumes the
                            // remainder, exactly like `Minecraft#runGameLoop`.
                            let partialTicks = render_partial_ticks(
                                self.timerAccumulator,
                                Instant::now(),
                                self.lastTimerSync,
                            );
                            let graphicsDevice = renderer.deviceName().to_owned();
                            let prepareStarted = frameStarted;
                            let preparedFrame = runtime.draw(
                                minecraft,
                                extent.width,
                                extent.height,
                                partialTicks,
                                self.debugFps,
                                &graphicsDevice,
                                renderer.backend(),
                            );
                            let prepareElapsed = prepareStarted.elapsed();
                            let renderStarted = Instant::now();
                            let mut worldFrame = false;
                            match preparedFrame {
                                Ok(RuntimeFrame::Gui(frame)) => {
                                    fatalError = renderer
                                        .drawFrame(window, &frame)
                                        .err()
                                        .map(|error| error.context("failed drawing Minecraft GUI frame"));
                                }
                                Ok(RuntimeFrame::NativeGui(frame)) => {
                                    fatalError = renderer
                                        .drawNativeGuiFrame(window, &frame)
                                        .err()
                                        .map(|error| error.context("failed drawing native Minecraft GUI frame"));
                                }
                                Ok(RuntimeFrame::World(frame)) => {
                                    worldFrame = true;
                                    fatalError = renderer
                                        .drawWorldFrame(window, &frame)
                                        .err()
                                        .map(|error| error.context("failed drawing Minecraft world frame"));
                                }
                                Err(error) => {
                                    fatalError = Some(error.context("failed preparing current Minecraft screen"));
                                }
                            }
                            let renderElapsed = renderStarted.elapsed();
                            if fatalError.is_none() {
                                frameProfileSample = Some((prepareElapsed, renderElapsed, worldFrame));
                            }
                            self.framesThisSecond = self.framesThisSecond.saturating_add(1);
                            let fpsNow = Instant::now();
                            if fpsNow.duration_since(self.lastFpsUpdate) >= Duration::from_secs(1) {
                                self.debugFps = self.framesThisSecond;
                                self.framesThisSecond = 0;
                                self.lastFpsUpdate = fpsNow;
                            }
                        }
                        let frameFinished = Instant::now();
                        self.nextFrameDeadline = frameInterval.map_or(frameFinished, |interval| {
                            frame_deadline_from_start(frameStarted, frameFinished, interval)
                        });
                    }
                    // `draw` also synchronizes packet-opened containers, so
                    // consume any resulting displayGuiScreen focus transition
                    // before another raw mouse-motion event can rotate the view.
                    self.applyRuntimeMouseFocusRequest();
                }
                if let Some((prepare, render, worldFrame)) = frameProfileSample {
                    self.recordFrameProfile(prepare, render, worldFrame);
                }
                // A Java/LWJGL Minecraft frame loop begins the next unlimited
                // frame immediately after Display.update. Waiting for Winit to
                // reach AboutToWait before posting another RedrawRequested adds
                // one native event-loop round trip per frame and can cap a very
                // fast scene around a few hundred FPS. Queue the next frame here;
                // Winit still dispatches input/window events between redraws.
                if fatalError.is_none() && !self.isFramerateLimitBelowMax() {
                    self.requestRedraw();
                }
            }
            _ => {}
        }
        if let Some(error) = fatalError { self.fail(eventLoop, error); }
    }

    fn device_event(&mut self, _eventLoop: &ActiveEventLoop, _deviceId: DeviceId, event: DeviceEvent) {
        if let Some(runtime) = self.mainMenu.as_mut() {
            runtime.syncOpenContainerGui();
        }
        self.applyRuntimeMouseFocusRequest();
        if !self.worldMouseGrabbed
            || self.mainMenu.as_ref().is_some_and(MainMenuRuntime::isModalWorldGuiOpen)
        {
            return;
        }
        let DeviceEvent::MouseMotion { delta: (deltaX, deltaY) } = event else {
            return;
        };
        let turned = match (self.mainMenu.as_mut(), self.minecraft.as_ref()) {
            (Some(runtime), Some(minecraft)) => runtime.turnPlayer(deltaX, deltaY, &minecraft.gameSettings),
            _ => false,
        };
        if turned {
            self.requestRedraw();
        }
    }

    fn suspended(&mut self, _eventLoop: &ActiveEventLoop) {
        self.setWorldMouseGrabbed(false);
        self.mainMenu = None; self.renderer = None; self.window = None; self.redrawPending = false;
    }

    fn about_to_wait(&mut self, eventLoop: &ActiveEventLoop) {
        if self.debugKeyDown
            && self.debugCrashKeyDown
            && self
                .debugCrashKeyPressTime
                .is_some_and(|started| started.elapsed() >= Duration::from_secs(6))
        {
            self.debugCrashKeyPressTime = None;
            self.triggerManualDebugCrash(eventLoop);
            return;
        }
        match self.applyPendingResize() {
            Ok(true) => self.requestRedraw(), Ok(false) => {}
            Err(error) => { self.fail(eventLoop, error); return; }
        }
        if let Some(since) = self.pendingResizeSince {
            eventLoop.set_control_flow(ControlFlow::WaitUntil(since + RESIZE_DEBOUNCE)); return;
        }

        let now = Instant::now();
        // MCP `Timer#updateTimer`: the frame interval in ticks is added to
        // the accumulator; the integer part runs that many ticks (capped at
        // 10 like the source) and the fraction is the render partial-ticks
        // residual (`field_194147_b`), consumed by `render_partial_ticks` on
        // the next RedrawRequested. The tick cadence is driven by wall time,
        // exactly like `Minecraft#runGameLoop` with `timer.elapsedTicks`.
        let frameDelta = now.duration_since(self.lastTimerSync).as_secs_f32() * 20.0;
        self.lastTimerSync = now;
        let (elapsedTicks, residual) = timer_pump(frameDelta, self.timerAccumulator);
        self.timerAccumulator = residual;
        let runTicks = elapsedTicks.min(10);
        if runTicks > 0 {
            let (forceSprint, chatWidth, chatScale, particleSetting, showSubtitles) = self.minecraft.as_ref().map_or(
                (false, 1.0, 1.0, 0, false),
                |minecraft| (
                    minecraft.gameSettings.forceSprint,
                    minecraft.gameSettings.chatWidth,
                    minecraft.gameSettings.chatScale,
                    minecraft.gameSettings.particleSetting,
                    minecraft.gameSettings.showSubtitles,
                ),
            );
            let controlHeld = self.keyboardModifiers.control_key();
            for _ in 0..runTicks {
                let (redraw, action) = self
                    .mainMenu
                    .as_mut()
                    .map(|runtime| runtime.updateScreen(
                        forceSprint,
                        chatWidth,
                        chatScale,
                        particleSetting,
                        controlHeld,
                        showSubtitles,
                    ))
                    .unwrap_or((false, None));
                let hadAction = action.is_some();
                if let Some(action) = action {
                    match self.applyGuiAction(action) {
                        Ok(true) => { eventLoop.exit(); return; }
                        Ok(false) => {}
                        Err(error) => { self.fail(eventLoop, error.context("failed applying network GUI action")); return; }
                    }
                }
                self.applyRuntimeMouseFocusRequest();
                if redraw || hadAction { self.requestRedraw(); }
            }
        }

        // Next wakeup for a tick: the fraction of the accumulator remaining.
        let nextTickAt = now + Duration::from_secs_f32(
            ((1.0 - self.timerAccumulator) / 20.0).max(0.0),
        );
        if self.window.is_some() && self.mainMenu.is_some() {
            if self.isFramerateLimitBelowMax() {
                if now >= self.nextFrameDeadline {
                    self.requestRedraw();
                }
                eventLoop.set_control_flow(ControlFlow::WaitUntil(
                    self.nextFrameDeadline.min(nextTickAt),
                ));
            } else {
                // 1.12.2 Unlimited: do not insert a frame-deadline wait. Winit
                // Poll is the `Display.sync`-free equivalent for this backend.
                self.requestRedraw();
                eventLoop.set_control_flow(ControlFlow::Poll);
            }
        } else {
            eventLoop.set_control_flow(ControlFlow::WaitUntil(nextTickAt));
        }
    }
}


/// MCP `Timer#updateTimer` accumulator step: adds the full frame interval
/// in ticks (never clamped — a stall must accumulate every tick), returns
/// the whole `elapsedTicks` (the run loop caps execution at 10) and the
/// exact source remainder as the render partial-ticks residual.
fn timer_pump(frameDelta: f32, accumulator: f32) -> (i32, f32) {
    let accumulated = accumulator + frameDelta;
    let elapsedTicks = accumulated as i32;
    (elapsedTicks, accumulated - elapsedTicks as f32)
}

/// MCP `Timer#field_194147_b`: the render partial-ticks value is the tick
/// accumulator's fractional remainder — the time since the last pumped tick,
/// in tick units — not the raw frame interval. `updateTimer` folds the whole
/// frame interval into the accumulator and consumes the whole ticks, so
/// `residual + (now - lastSync) * 20` is the exact position within the
/// current tick interval at the render instant. Rendering interpolates
/// `prev + (current - prev) * partialTicks` with this value, which keeps the
/// per-second animation speed at the fixed 20 TPS whatever the frame rate
/// (60 FPS cycles the 0.333 / 0.667 / 0.0 sawtooth, 1000 FPS ramps 0 → 1 in
/// hundredths; the per-second sum is 20). Clamped to the MCP `[0, 1)`
/// invariant: a value reaching 1.0 means a tick boundary was crossed between
/// the last pump and this render, where the interpolated position is the
/// freshly ticked position until the next pump runs the tick.
fn render_partial_ticks(residual: f32, now: Instant, lastTimerSync: Instant) -> f32 {
    (residual + now.duration_since(lastTimerSync).as_secs_f32() * 20.0).clamp(0.0, 1.0)
}

const fn tick_right_click_delay(timer: i32) -> i32 {
    if timer > 0 { timer - 1 } else { 0 }
}

const fn held_right_click_due(
    inWorld: bool,
    useButtonDown: bool,
    rightClickDelayTimer: i32,
    handActive: bool,
    isHittingBlock: bool,
    modalWorldGuiOpen: bool,
) -> bool {
    inWorld
        && useButtonDown
        && rightClickDelayTimer == 0
        && !handActive
        && !isHittingBlock
        && !modalWorldGuiOpen
}

const fn can_grab_world_mouse(windowFocused: bool, modalWorldGuiOpen: bool) -> bool {
    windowFocused && !modalWorldGuiOpen
}

fn frame_interval_for_limit(limit: i32) -> Option<Duration> {
    if limit >= FRAMERATE_LIMIT_MAX {
        None
    } else {
        Some(Duration::from_secs_f64(1.0 / limit.max(1) as f64))
    }
}

fn frame_deadline_from_start(
    frameStarted: Instant,
    frameFinished: Instant,
    interval: Duration,
) -> Instant {
    frameStarted.checked_add(interval).unwrap_or(frameFinished)
}

fn current_menu_date() -> MainMenuDate {
    let seconds = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let days = seconds.div_euclid(86_400);
    let (_, month, day) = civil_from_days(days);
    MainMenuDate { month: month as u8, day: day as u8 }
}

fn civil_from_days(daysSinceUnixEpoch: i64) -> (i32, u32, u32) {
    let z = daysSinceUnixEpoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 }.div_euclid(146_097);
    let dayOfEra = z - era * 146_097;
    let yearOfEra = (dayOfEra - dayOfEra / 1_460 + dayOfEra / 36_524 - dayOfEra / 146_096).div_euclid(365);
    let mut year = yearOfEra + era * 400;
    let dayOfYear = dayOfEra - (365 * yearOfEra + yearOfEra / 4 - yearOfEra / 100);
    let monthPrime = (5 * dayOfYear + 2).div_euclid(153);
    let day = dayOfYear - (153 * monthPrime + 2).div_euclid(5) + 1;
    let month = monthPrime + if monthPrime < 10 { 3 } else { -9 };
    year += if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn unix_epoch_date_conversion_is_1970_01_01() { assert_eq!(civil_from_days(0), (1970, 1, 1)); }
    #[test] fn minecraft_112_release_date_is_preserved() { assert_eq!(civil_from_days(17_427), (2017, 9, 18)); }
}

#[cfg(test)]
mod frame_rate_tests {
    use super::*;

    #[test]
    fn unlimited_uses_no_frame_wait() {
        assert_eq!(frame_interval_for_limit(FRAMERATE_LIMIT_MAX), None);
        assert_eq!(frame_interval_for_limit(300), None);
    }

    #[test]
    fn modal_gui_and_inactive_window_block_gameplay_mouse_grab() {
        assert!(can_grab_world_mouse(true, false));
        assert!(!can_grab_world_mouse(false, false));
        assert!(!can_grab_world_mouse(true, true));
        assert!(!can_grab_world_mouse(false, true));
    }

    #[test]
    fn right_click_delay_counts_four_client_ticks() {
        let mut timer = RIGHT_CLICK_DELAY_TICKS;
        assert!(!held_right_click_due(true, true, timer, false, false, false));
        for expected in [3, 2, 1, 0] {
            timer = tick_right_click_delay(timer);
            assert_eq!(timer, expected);
        }
        assert!(held_right_click_due(true, true, timer, false, false, false));
    }

    #[test]
    fn held_right_click_is_suppressed_by_source_guards() {
        assert!(!held_right_click_due(false, true, 0, false, false, false));
        assert!(!held_right_click_due(true, false, 0, false, false, false));
        assert!(!held_right_click_due(true, true, 1, false, false, false));
        assert!(!held_right_click_due(true, true, 0, true, false, false));
        assert!(!held_right_click_due(true, true, 0, false, true, false));
        assert!(!held_right_click_due(true, true, 0, false, false, true));
    }

    #[test]
    fn limited_deadline_includes_frame_work_in_the_budget() {
        let started = Instant::now();
        let finished = started + Duration::from_millis(5);
        let interval = Duration::from_millis(33);
        assert_eq!(
            frame_deadline_from_start(started, finished, interval),
            started + interval,
        );
        let slowFinish = started + Duration::from_millis(40);
        assert!(frame_deadline_from_start(started, slowFinish, interval) <= slowFinish);
    }

    #[test]
    fn limited_rates_produce_deadline_intervals() {
        assert_eq!(
            frame_interval_for_limit(30),
            Some(Duration::from_secs_f64(1.0 / 30.0)),
        );
        assert_eq!(
            frame_interval_for_limit(120),
            Some(Duration::from_secs_f64(1.0 / 120.0)),
        );
    }

    #[test]
    fn timer_pump_accumulates_stalls_without_clamping() {
        // MCP `Timer#updateTimer` never clamps the frame delta: a 250 ms
        // stall accumulates ~5 ticks, a 600 ms stall ~12. The run loop caps
        // execution at 10 ticks, but the accumulator subtracts the *full*
        // elapsedTicks, so the residual stays the exact source remainder
        // and no time is dropped or carried over into catch-up ticks.
        let (ticks, residual) = timer_pump(5.0, 0.0);
        assert_eq!(ticks, 5);
        assert!((residual - 0.0).abs() < 1.0e-6);

        let (ticks, residual) = timer_pump(12.0, 0.0);
        assert_eq!(ticks, 12); // run loop executes min(12, 10) = 10
        assert!((residual - 0.0).abs() < 1.0e-6);

        // A sub-tick delta keeps the whole fraction as residual.
        let (ticks, residual) = timer_pump(0.66, 0.0);
        assert_eq!(ticks, 0);
        assert!((residual - 0.66).abs() < 1.0e-6);

        // Accumulation across frames: 0.5 + 0.5 crosses one tick boundary.
        let (ticks, residual) = timer_pump(0.5, 0.5);
        assert_eq!(ticks, 1);
        assert!((residual - 0.0).abs() < 1.0e-6);

        // Long stall followed by normal frames: the next frame starts from
        // the true remainder, not a clamped budget.
        let (ticks, residual) = timer_pump(12.66, 0.0);
        assert_eq!(ticks, 12);
        assert!((residual - 0.66).abs() < 1.0e-6);
        let (ticks, residual) = timer_pump(1.0 / 3.0, residual);
        assert_eq!(ticks, 0);
        assert!((residual - 0.99333).abs() < 1.0e-5);
    }

    #[test]
    fn partial_ticks_is_the_tick_pump_residual() {
        let lastSync = Instant::now();
        // MCP `Timer#field_194147_b`: partialTicks is the accumulator
        // remainder, not the frame interval. A fresh pump leaves its
        // residual untouched...
        let fresh = render_partial_ticks(1.0 / 3.0, lastSync, lastSync);
        assert!((fresh - 1.0 / 3.0).abs() < 1.0e-6);
        // ...and only the wall time since the last pump is added.
        let later = render_partial_ticks(0.25, lastSync + Duration::from_millis(25), lastSync);
        assert!((later - 0.75).abs() < 1.0e-6);
        // A tick boundary crossed between pump and render clamps to the MCP
        // `[0, 1)` invariant: the interpolated position is the freshly
        // ticked position until the next pump runs the tick.
        let crossed = render_partial_ticks(0.9, lastSync + Duration::from_millis(20), lastSync);
        assert_eq!(crossed, 1.0);
    }

    #[test]
    fn sixty_fps_partial_ticks_cycle_is_the_java_sawtooth() {
        // MCP `Timer#updateTimer` at 60 FPS: each frame adds a third of a
        // tick to the accumulator, a whole tick runs when the sum crosses 1
        // and the render partial-ticks is the remainder: 0.333, 0.667, 0.0.
        let mut residual = 0.0;
        let mut lastSync = Instant::now();
        let frame = Duration::from_secs_f64(1.0 / 60.0);
        let mut sequence = Vec::new();
        for _ in 0..6 {
            let now = lastSync + frame;
            residual += (now.duration_since(lastSync).as_secs_f32() * 20.0).min(1.0);
            residual -= residual as i32 as f32;
            lastSync = now;
            sequence.push(render_partial_ticks(residual, lastSync, lastSync));
        }
        for (got, want) in sequence
            .iter()
            .zip([1.0 / 3.0, 2.0 / 3.0, 0.0, 1.0 / 3.0, 2.0 / 3.0, 0.0])
        {
            assert!((got - want).abs() < 1.0e-6, "partialTicks {got} != {want}");
        }
    }

    #[test]
    fn partial_ticks_sawtooth_is_continuous_at_any_frame_rate() {
        // The interpolation residual is a continuous sawtooth (0..1) that
        // ramps inside every tick and resets at the tick boundary: at any
        // frame rate each tick interval is swept monotonically, so rendered
        // positions never hold still and jump (frame-interval semantics
        // would stall at `1/n` and jump at every tick).
        fn simulate(fps: u32) -> Vec<f32> {
            let mut residual = 0.0;
            let mut lastSync = Instant::now();
            let frame = Duration::from_secs_f64(1.0 / fps as f64);
            let mut sequence = Vec::new();
            for _ in 0..fps {
                let now = lastSync + frame;
                residual += (now.duration_since(lastSync).as_secs_f32() * 20.0).min(1.0);
                residual -= residual as i32 as f32;
                lastSync = now;
                sequence.push(render_partial_ticks(residual, lastSync, lastSync));
            }
            sequence
        }
        for fps in [60_u32, 240, 1000] {
            let sequence = simulate(fps);
            // A wall second sweeps 20 ticks; ±1 tolerates the floating-point
            // carry-over at the tick boundary (an accumulator that lands on
            // 0.9999999 instead of 1.0 merges two ramps).
            let mut previous = -1.0_f32;
            let mut resets = 0_usize;
            for value in sequence {
                assert!(value < 1.0, "residual escapes the tick at {value}");
                if value < previous {
                    // Tick boundary: the next ramp restarts (floating-point
                    // carry-over makes the new start a tiny epsilon, not 0).
                    resets += 1;
                }
                previous = value;
            }
            assert!(
                (19..=21).contains(&resets),
                "tick sweep count at {fps} FPS: {resets}"
            );
        }
    }
}
