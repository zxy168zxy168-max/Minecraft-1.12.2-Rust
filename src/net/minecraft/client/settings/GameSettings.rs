use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::launcher::OptionsFile::{OptionsFile, OptionsFileError};
use crate::launcher::RenderBackend::RenderBackend;
use crate::net::minecraft::client::settings::KeyBinding::{
    vanilla_key_bindings, KeyBinding, KeyBindingId,
};

use crate::net::minecraft::entity::player::EntityPlayer::EnumChatVisibility;
use crate::net::minecraft::entity::player::EnumPlayerModelParts::EnumPlayerModelParts;
use crate::net::minecraft::util::EnumHandSide::EnumHandSide;
use crate::net::minecraft::util::SoundCategory::SoundCategory;

pub const FRAMERATE_LIMIT_MAX: i32 = 260;

/// Render-facing settings model initialized from MCP 1.12.2 + OptiFine C6 defaults.
///
/// Fields intentionally keep their MCP names where that improves source-level
/// traceability. Native-backend implementation switches belong in separate
/// backend settings structures and must not overwrite gameplay-visible options.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameSettings {
    pub mouseSensitivity: f32,
    pub invertMouse: bool,
    pub renderDistanceChunks: i32,
    pub viewBobbing: bool,
    pub anaglyph: bool,
    pub fboEnable: bool,
    pub limitFramerate: i32,
    pub clouds: i32,
    pub fancyGraphics: bool,
    pub ambientOcclusion: i32,
    pub chatVisibility: EnumChatVisibility,
    pub chatColours: bool,
    pub chatLinks: bool,
    pub chatLinksPrompt: bool,
    pub chatOpacity: f32,
    pub chatScale: f32,
    pub chatWidth: f32,
    pub chatHeightUnfocused: f32,
    pub chatHeightFocused: f32,
    pub modelPartFlags: u8,
    pub mainHand: EnumHandSide,
    pub fullScreen: bool,
    pub enableVsync: bool,
    pub useVbo: bool,
    pub pauseOnLostFocus: bool,
    pub mipmapLevels: i32,
    pub entityShadows: bool,
    pub attackIndicator: i32,
    pub autoJump: bool,
    #[serde(default)]
    pub touchscreen: bool,
    /// MCP/OptiFine 1.12.2 key binding table. Codes remain in LWJGL 2
    /// `options.txt` integer space so existing vanilla configurations load unchanged.
    #[serde(default = "vanilla_key_bindings")]
    pub keyBindings: Vec<KeyBinding>,
    pub showSubtitles: bool,
    /// MCP transient debug overlay switches. Vanilla does not serialize these three fields.
    pub showDebugInfo: bool,
    pub showDebugProfilerChart: bool,
    pub showLagometer: bool,
    pub reducedDebugInfo: bool,
    pub advancedItemTooltips: bool,
    /// MCP `soundLevels` enum map stored as a dense array in enum order.
    pub soundLevels: [f32; 10],
    /// User-requested Rust-port extension. When enabled, the client holds the
    /// vanilla sprint key logically while its normal eligibility checks remain
    /// authoritative. This is deliberately not represented as a new gameplay
    /// state on EntityPlayerSP.
    pub forceSprint: bool,
    pub fovSetting: f32,
    pub gammaSetting: f32,
    pub guiScale: i32,
    pub particleSetting: i32,
    /// MCP transient camera mode: 0 first person, 1 rear third person,
    /// 2 front-facing third person. Vanilla does not persist this field.
    pub thirdPersonView: i32,
    pub language: String,
    pub forceUnicodeFont: bool,
    pub lastServer: String,
    pub resourcePacks: Vec<String>,
    pub incompatibleResourcePacks: Vec<String>,
    /// Rust-port launcher setting. Unlike MCP video options, changing the
    /// native graphics API requires rebuilding the window/context on restart.
    pub renderBackend: RenderBackend,
    /// Native API currently owning the window. This is transient and remains
    /// unchanged when a restart-required backend preference is edited.
    #[serde(skip)]
    pub activeRenderBackend: RenderBackend,

    pub ofFogType: i32,
    pub ofFogStart: f32,
    pub ofMipmapType: i32,
    pub ofOcclusionFancy: bool,
    pub ofSmoothFps: bool,
    pub ofSmoothWorld: bool,
    pub ofLazyChunkLoading: bool,
    pub ofAoLevel: f32,
    pub ofAaLevel: i32,
    pub ofAfLevel: i32,
    pub ofClouds: i32,
    pub ofCloudsHeight: f32,
    pub ofTrees: i32,
    pub ofRain: i32,
    pub ofDroppedItems: i32,
    pub ofBetterGrass: i32,
    pub ofAutoSaveTicks: i32,
    pub ofLagometer: bool,
    pub ofProfiler: bool,
    pub ofShowFps: bool,
    pub ofWeather: bool,
    pub ofSky: bool,
    pub ofStars: bool,
    pub ofSunMoon: bool,
    pub ofVignette: i32,
    pub ofChunkUpdates: i32,
    pub ofChunkUpdatesDynamic: bool,
    pub ofTime: i32,
    pub ofClearWater: bool,
    pub ofBetterSnow: bool,
    pub ofFullscreenMode: String,
    pub ofSwampColors: bool,
    pub ofRandomMobs: bool,
    pub ofSmoothBiomes: bool,
    pub ofCustomFonts: bool,
    pub ofCustomColors: bool,
    pub ofCustomSky: bool,
    pub ofShowCapes: bool,
    pub ofConnectedTextures: i32,
    pub ofCustomItems: bool,
    pub ofNaturalTextures: bool,
    pub ofFastMath: bool,
    pub ofFastRender: bool,
    pub ofTranslucentBlocks: i32,
    pub ofDynamicFov: bool,
    pub ofAlternateBlocks: bool,
    pub ofDynamicLights: i32,
    pub ofCustomEntityModels: bool,
    pub ofCustomGuis: bool,
    pub ofScreenshotSize: i32,
    pub ofAnimatedWater: i32,
    pub ofAnimatedLava: i32,
    pub ofAnimatedFire: bool,
    pub ofAnimatedPortal: bool,
    pub ofAnimatedRedstone: bool,
    pub ofAnimatedExplosion: bool,
    pub ofAnimatedFlame: bool,
    pub ofAnimatedSmoke: bool,
    pub ofVoidParticles: bool,
    pub ofWaterParticles: bool,
    pub ofRainSplash: bool,
    pub ofPortalParticles: bool,
    pub ofPotionParticles: bool,
    pub ofFireworkParticles: bool,
    pub ofDrippingWaterLava: bool,
    pub ofAnimatedTerrain: bool,
    pub ofAnimatedTextures: bool,
}

impl GameSettings {
    /// Loads the subset of `options.txt` already consumed by the visible
    /// client bootstrap. Unknown options remain untouched and all defaults are
    /// still the MCP/OptiFine 1.12.2 values above.
    pub fn loadFromGameDir(gameDir: impl AsRef<Path>) -> Result<Self, OptionsFileError> {
        let path = gameDir.as_ref().join("options.txt");
        if !path.is_file() {
            return Ok(Self::default());
        }
        let options = OptionsFile::load(path)?;
        let mut settings = Self::default();
        settings.mouseSensitivity =
            read_f32(&options, "mouseSensitivity", settings.mouseSensitivity).clamp(0.0, 1.0);
        settings.invertMouse = read_bool(&options, "invertYMouse", settings.invertMouse);
        settings.renderDistanceChunks =
            read_i32(&options, "renderDistance", settings.renderDistanceChunks).clamp(2, 32);
        settings.viewBobbing = read_bool(&options, "bobView", settings.viewBobbing);
        settings.clouds = read_clouds(&options, settings.clouds);
        settings.fancyGraphics = read_bool(&options, "fancyGraphics", settings.fancyGraphics);
        settings.ambientOcclusion = read_ambient_occlusion(&options, settings.ambientOcclusion);
        settings.pauseOnLostFocus =
            read_bool(&options, "pauseOnLostFocus", settings.pauseOnLostFocus);
        settings.entityShadows = read_bool(&options, "entityShadows", settings.entityShadows);
        settings.attackIndicator =
            read_i32(&options, "attackIndicator", settings.attackIndicator).clamp(0, 2);
        settings.autoJump = read_bool(&options, "autoJump", settings.autoJump);
        settings.touchscreen = read_bool(&options, "touchscreen", settings.touchscreen);
        for binding in &mut settings.keyBindings {
            let optionKey = format!("key_{}", binding.keyDescription);
            binding.keyCode = read_i32(&options, &optionKey, binding.keyCode);
        }
        settings.showSubtitles = read_bool(&options, "showSubtitles", settings.showSubtitles);
        settings.reducedDebugInfo =
            read_bool(&options, "reducedDebugInfo", settings.reducedDebugInfo);
        settings.advancedItemTooltips = read_bool(
            &options,
            "advancedItemTooltips",
            settings.advancedItemTooltips,
        );
        for category in SoundCategory::ALL {
            settings.soundLevels[category.index()] = read_f32(
                &options,
                &format!("soundCategory_{}", category.getName()),
                settings.soundLevels[category.index()],
            )
            .clamp(0.0, 1.0);
        }
        settings.forceSprint = read_bool(&options, "forceSprint", settings.forceSprint);
        settings.gammaSetting = read_f32(&options, "gamma", settings.gammaSetting).clamp(0.0, 1.0);
        settings.particleSetting =
            read_i32(&options, "particles", settings.particleSetting).clamp(0, 2);
        settings.guiScale = read_i32(&options, "guiScale", settings.guiScale);
        if let Some(language) = options.get("lang") {
            settings.language = language.to_owned();
        }
        settings.forceUnicodeFont =
            read_bool(&options, "forceUnicodeFont", settings.forceUnicodeFont);
        settings.fullScreen = read_bool(&options, "fullscreen", settings.fullScreen);
        settings.anaglyph = read_bool(&options, "anaglyph3d", settings.anaglyph);
        settings.fboEnable = read_bool(&options, "fboEnable", settings.fboEnable);
        settings.useVbo = read_bool(&options, "useVbo", settings.useVbo);
        settings.mipmapLevels = read_i32(&options, "mipmapLevels", settings.mipmapLevels);
        settings.limitFramerate = read_i32(&options, "maxFps", settings.limitFramerate);
        if settings.limitFramerate <= 0 {
            settings.limitFramerate = FRAMERATE_LIMIT_MAX;
        }
        settings.enableVsync = read_bool(&options, "enableVsync", settings.enableVsync);
        if settings.enableVsync {
            settings.limitFramerate = FRAMERATE_LIMIT_MAX;
        }
        settings.chatVisibility = EnumChatVisibility::getChatVisibility(read_i32(
            &options,
            "chatVisibility",
            settings.chatVisibility.getChatVisibilityId(),
        ));
        settings.chatColours = read_bool(&options, "chatColors", settings.chatColours);
        settings.chatLinks = read_bool(&options, "chatLinks", settings.chatLinks);
        settings.chatLinksPrompt = read_bool(&options, "chatLinksPrompt", settings.chatLinksPrompt);
        settings.chatOpacity =
            read_f32(&options, "chatOpacity", settings.chatOpacity).clamp(0.0, 1.0);
        settings.chatScale = read_f32(&options, "chatScale", settings.chatScale).clamp(0.0, 1.0);
        settings.chatWidth = read_f32(&options, "chatWidth", settings.chatWidth).clamp(0.0, 1.0);
        settings.chatHeightUnfocused = read_f32(
            &options,
            "chatHeightUnfocused",
            settings.chatHeightUnfocused,
        )
        .clamp(0.0, 1.0);
        settings.chatHeightFocused =
            read_f32(&options, "chatHeightFocused", settings.chatHeightFocused).clamp(0.0, 1.0);
        if let Some(mainHand) = options.get("mainHand") {
            settings.mainHand = if mainHand.eq_ignore_ascii_case("left") {
                EnumHandSide::Left
            } else {
                EnumHandSide::Right
            };
        }
        settings.modelPartFlags = read_model_part_flags(&options, settings.modelPartFlags);
        if let Some(fov) = options
            .get("fov")
            .and_then(|value| value.parse::<f32>().ok())
        {
            settings.fovSetting = fov * 40.0 + 70.0;
        }
        if let Some(lastServer) = options.get("lastServer") {
            settings.lastServer = lastServer.to_owned();
        }
        settings.resourcePacks = read_string_list(&options, "resourcePacks");
        settings.incompatibleResourcePacks =
            read_string_list(&options, "incompatibleResourcePacks");
        if let Some(backend) = options.get("rustRenderBackend") {
            settings.renderBackend = RenderBackend::parse(backend);
        }

        settings.activeRenderBackend = settings.renderBackend;

        settings.ofFastMath = read_bool(&options, "ofFastMath", settings.ofFastMath);
        settings.ofCustomFonts = read_bool(&options, "ofCustomFonts", settings.ofCustomFonts);
        settings.ofCustomGuis = read_bool(&options, "ofCustomGuis", settings.ofCustomGuis);
        settings.ofCustomSky = read_bool(&options, "ofCustomSky", settings.ofCustomSky);
        settings.ofCustomColors = read_bool(&options, "ofCustomColors", settings.ofCustomColors);
        settings.ofAaLevel = read_i32(&options, "ofAaLevel", settings.ofAaLevel);
        settings.ofAfLevel = read_i32(&options, "ofAfLevel", settings.ofAfLevel);
        if let Some(fullscreenMode) = options.get("ofFullscreenMode") {
            settings.ofFullscreenMode = fullscreenMode.to_owned();
        }
        Ok(settings)
    }

    /// Persists the options currently implemented by this port while preserving
    /// every unknown line from the original 1.12.2/OptiFine options.txt.
    pub fn saveOptions(&self, gameDir: impl AsRef<Path>) -> io::Result<()> {
        let path = gameDir.as_ref().join("options.txt");
        let mut options = if path.is_file() {
            OptionsFile::load(&path).unwrap_or_default()
        } else {
            OptionsFile::default()
        };
        options.set("version", "1343");
        options.set("invertYMouse", self.invertMouse.to_string());
        options.set("mouseSensitivity", self.mouseSensitivity.to_string());
        options.set("renderDistance", self.renderDistanceChunks.to_string());
        options.set("bobView", self.viewBobbing.to_string());
        options.remove("clouds");
        options.set(
            "renderClouds",
            match self.clouds {
                0 => "false",
                1 => "fast",
                _ => "true",
            },
        );
        options.set("fancyGraphics", self.fancyGraphics.to_string());
        options.set("ao", self.ambientOcclusion.to_string());
        options.set("pauseOnLostFocus", self.pauseOnLostFocus.to_string());
        options.set("entityShadows", self.entityShadows.to_string());
        options.set("attackIndicator", self.attackIndicator.to_string());
        options.set("autoJump", self.autoJump.to_string());
        options.set("touchscreen", self.touchscreen.to_string());
        for binding in &self.keyBindings {
            options.set(
                format!("key_{}", binding.keyDescription),
                binding.keyCode.to_string(),
            );
        }
        options.set("showSubtitles", self.showSubtitles.to_string());
        options.set("reducedDebugInfo", self.reducedDebugInfo.to_string());
        options.set(
            "advancedItemTooltips",
            self.advancedItemTooltips.to_string(),
        );
        for category in SoundCategory::ALL {
            options.set(
                format!("soundCategory_{}", category.getName()),
                self.getSoundLevel(category).to_string(),
            );
        }
        options.set("forceSprint", self.forceSprint.to_string());
        options.set("gamma", self.gammaSetting.to_string());
        options.set("particles", self.particleSetting.to_string());
        options.set("guiScale", self.guiScale.to_string());
        options.set("lang", self.language.clone());
        options.set("forceUnicodeFont", self.forceUnicodeFont.to_string());
        options.set("enableVsync", self.enableVsync.to_string());
        options.set("fullscreen", self.fullScreen.to_string());
        options.set("anaglyph3d", self.anaglyph.to_string());
        options.set("fboEnable", self.fboEnable.to_string());
        options.set("useVbo", self.useVbo.to_string());
        options.set("mipmapLevels", self.mipmapLevels.to_string());
        options.set("maxFps", self.limitFramerate.to_string());
        options.set(
            "chatVisibility",
            self.chatVisibility.getChatVisibilityId().to_string(),
        );
        options.set("chatColors", self.chatColours.to_string());
        options.set("chatLinks", self.chatLinks.to_string());
        options.set("chatLinksPrompt", self.chatLinksPrompt.to_string());
        options.set("chatOpacity", self.chatOpacity.to_string());
        options.set("chatScale", self.chatScale.to_string());
        options.set("chatWidth", self.chatWidth.to_string());
        options.set("chatHeightUnfocused", self.chatHeightUnfocused.to_string());
        options.set("chatHeightFocused", self.chatHeightFocused.to_string());
        options.set(
            "mainHand",
            match self.mainHand {
                EnumHandSide::Left => "left",
                EnumHandSide::Right => "right",
            },
        );
        write_model_part_flags(&mut options, self.modelPartFlags);
        options.set("lastServer", self.lastServer.clone());
        options.set(
            "resourcePacks",
            serde_json::to_string(&self.resourcePacks).unwrap_or_else(|_| "[]".to_owned()),
        );
        options.set(
            "incompatibleResourcePacks",
            serde_json::to_string(&self.incompatibleResourcePacks)
                .unwrap_or_else(|_| "[]".to_owned()),
        );
        options.set("rustRenderBackend", self.renderBackend.optionValue());
        // MCP stores FOV as a normalized slider value: (degrees - 70) / 40.
        options.set("fov", ((self.fovSetting - 70.0) / 40.0).to_string());
        options.set("ofFastMath", self.ofFastMath.to_string());
        options.set("ofCustomFonts", self.ofCustomFonts.to_string());
        options.set("ofCustomGuis", self.ofCustomGuis.to_string());
        options.set("ofCustomSky", self.ofCustomSky.to_string());
        options.set("ofCustomColors", self.ofCustomColors.to_string());
        options.set("ofAaLevel", self.ofAaLevel.to_string());
        options.set("ofAfLevel", self.ofAfLevel.to_string());
        options.set("ofFullscreenMode", self.ofFullscreenMode.clone());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension("txt_tmp");
        fs::write(&temporary, options.render())?;
        if path.exists() {
            fs::remove_file(&path)?;
        }
        fs::rename(temporary, path)
    }

    pub fn keyBinding(&self, id: KeyBindingId) -> &KeyBinding {
        &self.keyBindings[id.index()]
    }

    pub fn keyBindingMut(&mut self, id: KeyBindingId) -> &mut KeyBinding {
        &mut self.keyBindings[id.index()]
    }

    pub fn setOptionKeyBinding(&mut self, id: KeyBindingId, keyCode: i32) {
        self.keyBindingMut(id).setKeyCode(keyCode);
    }

    /// Resolve a physical LWJGL key code through the current binding table.
    /// Later entries win on conflicts, matching the overwrite behavior of
    /// Minecraft 1.12.2 `KeyBinding.HASH#addKey` during registration.
    pub fn keyBindingIdForCode(&self, keyCode: i32) -> Option<KeyBindingId> {
        if keyCode == 0 {
            return None;
        }
        KeyBindingId::ALL
            .iter()
            .rev()
            .copied()
            .find(|id| self.keyBinding(*id).keyCode == keyCode)
    }

    pub fn resetAllKeyBindings(&mut self) {
        for binding in &mut self.keyBindings {
            binding.keyCode = binding.keyCodeDefault;
            binding.unpressKey();
        }
    }

    pub fn getSoundLevel(&self, category: SoundCategory) -> f32 {
        self.soundLevels[category.index()]
    }

    pub fn setSoundLevel(&mut self, category: SoundCategory, volume: f32) {
        self.soundLevels[category.index()] = volume.clamp(0.0, 1.0);
    }
}

fn read_bool(options: &OptionsFile, key: &str, default: bool) -> bool {
    options
        .get(key)
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(default)
}

fn read_i32(options: &OptionsFile, key: &str, default: i32) -> i32 {
    options
        .get(key)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn read_f32(options: &OptionsFile, key: &str, default: f32) -> f32 {
    options
        .get(key)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn read_string_list(options: &OptionsFile, key: &str) -> Vec<String> {
    options
        .get(key)
        .and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
        .unwrap_or_default()
}

fn read_ambient_occlusion(options: &OptionsFile, default: i32) -> i32 {
    match options.get("ao") {
        Some(value) if value.eq_ignore_ascii_case("true") => 2,
        Some(value) if value.eq_ignore_ascii_case("false") => 0,
        Some(value) => value.parse::<i32>().unwrap_or(default).clamp(0, 2),
        None => default,
    }
}

fn read_clouds(options: &OptionsFile, default: i32) -> i32 {
    // MCP 1.12.2 persists this option under `renderClouds` using
    // false/fast/true rather than an integer. Accept the short-lived Rust-port
    // `clouds` key only as a migration fallback, then always save the vanilla key.
    if let Some(value) = options.get("renderClouds") {
        return match value.to_ascii_lowercase().as_str() {
            "false" => 0,
            "fast" => 1,
            "true" => 2,
            _ => default,
        };
    }
    options
        .get("clouds")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(default)
        .clamp(0, 2)
}

fn read_model_part_flags(options: &OptionsFile, default: u8) -> u8 {
    let mut flags = default;
    for part in EnumPlayerModelParts::VALUES {
        let key = format!("modelPart_{}", part.getPartName());
        if options.get(&key).is_some() {
            if read_bool(options, &key, true) {
                flags |= part.getPartMask();
            } else {
                flags &= !part.getPartMask();
            }
        }
    }
    flags
}

fn write_model_part_flags(options: &mut OptionsFile, flags: u8) {
    for part in EnumPlayerModelParts::VALUES {
        options.set(
            format!("modelPart_{}", part.getPartName()),
            (flags & part.getPartMask() != 0).to_string(),
        );
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        let single_processor =
            std::thread::available_parallelism().map_or(true, |value| value.get() <= 1);
        Self {
            mouseSensitivity: 0.5,
            invertMouse: false,
            renderDistanceChunks: 8,
            viewBobbing: true,
            anaglyph: false,
            fboEnable: true,
            limitFramerate: FRAMERATE_LIMIT_MAX,
            clouds: 2,
            fancyGraphics: true,
            ambientOcclusion: 2,
            chatVisibility: EnumChatVisibility::Full,
            chatColours: true,
            chatLinks: true,
            chatLinksPrompt: true,
            chatOpacity: 1.0,
            chatScale: 1.0,
            chatWidth: 1.0,
            chatHeightUnfocused: 0.443_661_96,
            chatHeightFocused: 1.0,
            modelPartFlags: 0x7F,
            mainHand: EnumHandSide::Right,
            fullScreen: false,
            enableVsync: false,
            useVbo: true,
            pauseOnLostFocus: true,
            mipmapLevels: 4,
            entityShadows: true,
            attackIndicator: 1,
            autoJump: true,
            touchscreen: false,
            keyBindings: vanilla_key_bindings(),
            showSubtitles: false,
            showDebugInfo: false,
            showDebugProfilerChart: false,
            showLagometer: false,
            reducedDebugInfo: false,
            advancedItemTooltips: false,
            soundLevels: [1.0; 10],
            forceSprint: false,
            fovSetting: 70.0,
            gammaSetting: 0.0,
            guiScale: 0,
            particleSetting: 0,
            thirdPersonView: 0,
            language: "en_us".to_owned(),
            forceUnicodeFont: false,
            lastServer: String::new(),
            resourcePacks: Vec::new(),
            incompatibleResourcePacks: Vec::new(),
            renderBackend: RenderBackend::Vulkan,
            activeRenderBackend: RenderBackend::Vulkan,
            ofFogType: 1,
            ofFogStart: 0.8,
            ofMipmapType: 0,
            ofOcclusionFancy: false,
            ofSmoothFps: false,
            ofSmoothWorld: single_processor,
            ofLazyChunkLoading: single_processor,
            ofAoLevel: 1.0,
            ofAaLevel: 0,
            ofAfLevel: 1,
            ofClouds: 0,
            ofCloudsHeight: 0.0,
            ofTrees: 0,
            ofRain: 0,
            ofDroppedItems: 0,
            ofBetterGrass: 3,
            ofAutoSaveTicks: 4000,
            ofLagometer: false,
            ofProfiler: false,
            ofShowFps: false,
            ofWeather: true,
            ofSky: true,
            ofStars: true,
            ofSunMoon: true,
            ofVignette: 0,
            ofChunkUpdates: 1,
            ofChunkUpdatesDynamic: false,
            ofTime: 0,
            ofClearWater: false,
            ofBetterSnow: false,
            ofFullscreenMode: "Default".to_owned(),
            ofSwampColors: true,
            ofRandomMobs: true,
            ofSmoothBiomes: true,
            ofCustomFonts: true,
            ofCustomColors: true,
            ofCustomSky: true,
            ofShowCapes: true,
            ofConnectedTextures: 2,
            ofCustomItems: true,
            ofNaturalTextures: false,
            ofFastMath: false,
            ofFastRender: false,
            ofTranslucentBlocks: 0,
            ofDynamicFov: true,
            ofAlternateBlocks: true,
            ofDynamicLights: 3,
            ofCustomEntityModels: true,
            ofCustomGuis: true,
            ofScreenshotSize: 1,
            ofAnimatedWater: 0,
            ofAnimatedLava: 0,
            ofAnimatedFire: true,
            ofAnimatedPortal: true,
            ofAnimatedRedstone: true,
            ofAnimatedExplosion: true,
            ofAnimatedFlame: true,
            ofAnimatedSmoke: true,
            ofVoidParticles: true,
            ofWaterParticles: true,
            ofRainSplash: true,
            ofPortalParticles: true,
            ofPotionParticles: true,
            ofFireworkParticles: true,
            ofDrippingWaterLava: true,
            ofAnimatedTerrain: true,
            ofAnimatedTextures: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_visible_vanilla_and_optifine_options() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "mc112-game-settings-{}-{unique}",
            std::process::id(),
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("options.txt"),
            concat!(
                "invertYMouse:true\n",
                "mouseSensitivity:0.75\n",
                "renderDistance:12\n",
                "bobView:false\n",
                "renderClouds:fast\n",
                "fancyGraphics:false\n",
                "ao:1\n",
                "gamma:0.6\n",
                "particles:2\n",
                "pauseOnLostFocus:false\n",
                "entityShadows:false\n",
                "attackIndicator:2\n",
                "autoJump:false\n",
                "touchscreen:true\n",
                "key_key.forward:44\n",
                "key_key.attack:-98\n",
                "forceSprint:true\n",
                "guiScale:3\n",
                "lang:zh_cn\n",
                "forceUnicodeFont:true\n",
                "enableVsync:false\n",
                "chatVisibility:2\n",
                "chatColors:false\n",
                "chatLinks:false\n",
                "chatLinksPrompt:false\n",
                "chatOpacity:0.5\n",
                "chatScale:0.75\n",
                "chatWidth:0.6\n",
                "chatHeightUnfocused:0.25\n",
                "chatHeightFocused:0.8\n",
                "mainHand:left\n",
                "modelPart_cape:false\n",
                "ofFastMath:true\n",
                "ofCustomFonts:false\n",
                "ofFullscreenMode:1920x1080\n",
                "resourcePacks:[\"Vanilla Test.zip\",\"Folder Pack\"]\n",
                "incompatibleResourcePacks:[\"Legacy Test.zip\"]\n",
                "rustRenderBackend:opengl\n",
            ),
        )
        .unwrap();

        let settings = GameSettings::loadFromGameDir(&directory).unwrap();
        assert!(settings.invertMouse);
        assert!((settings.mouseSensitivity - 0.75).abs() < f32::EPSILON);
        assert_eq!(settings.renderDistanceChunks, 12);
        assert!(!settings.viewBobbing);
        assert_eq!(settings.clouds, 1);
        assert!(!settings.fancyGraphics);
        assert_eq!(settings.ambientOcclusion, 1);
        assert!((settings.gammaSetting - 0.6).abs() < f32::EPSILON);
        assert_eq!(settings.particleSetting, 2);
        assert!(!settings.pauseOnLostFocus);
        assert!(!settings.entityShadows);
        assert_eq!(settings.attackIndicator, 2);
        assert!(!settings.autoJump);
        assert!(settings.touchscreen);
        assert_eq!(settings.keyBinding(KeyBindingId::Forward).keyCode, 44);
        assert_eq!(settings.keyBinding(KeyBindingId::Attack).keyCode, -98);
        assert!(settings.forceSprint);
        assert_eq!(settings.guiScale, 3);
        assert_eq!(settings.language, "zh_cn");
        assert!(settings.forceUnicodeFont);
        assert!(!settings.enableVsync);
        assert_eq!(settings.chatVisibility, EnumChatVisibility::Hidden);
        assert!(!settings.chatColours);
        assert!(!settings.chatLinks);
        assert!(!settings.chatLinksPrompt);
        assert!((settings.chatOpacity - 0.5).abs() < f32::EPSILON);
        assert!((settings.chatScale - 0.75).abs() < f32::EPSILON);
        assert!((settings.chatWidth - 0.6).abs() < f32::EPSILON);
        assert!((settings.chatHeightUnfocused - 0.25).abs() < f32::EPSILON);
        assert!((settings.chatHeightFocused - 0.8).abs() < f32::EPSILON);
        assert_eq!(settings.mainHand, EnumHandSide::Left);
        assert_eq!(settings.modelPartFlags, 0x7E);
        assert!(settings.ofFastMath);
        assert!(!settings.ofCustomFonts);
        assert_eq!(settings.ofFullscreenMode, "1920x1080");
        assert_eq!(
            settings.resourcePacks,
            vec!["Vanilla Test.zip", "Folder Pack"]
        );
        assert_eq!(settings.incompatibleResourcePacks, vec!["Legacy Test.zip"]);
        assert_eq!(settings.renderBackend, RenderBackend::OpenGl);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn key_binding_resolution_matches_registration_overwrite_semantics() {
        let mut settings = GameSettings::default();
        // OptiFine 1.12.2 appends Zoom (C/46) after vanilla Save Toolbar,
        // so initial KeyBinding.HASH registration resolves C to Zoom.
        assert_eq!(
            settings.keyBindingIdForCode(46),
            Some(KeyBindingId::OptifineZoom)
        );
        settings.setOptionKeyBinding(KeyBindingId::Forward, 44);
        assert_eq!(
            settings.keyBindingIdForCode(44),
            Some(KeyBindingId::Forward)
        );
        settings.setOptionKeyBinding(KeyBindingId::Forward, 0);
        assert_eq!(settings.keyBindingIdForCode(0), None);
    }

    #[test]
    fn default_video_timing_is_unlimited_without_vsync() {
        let settings = GameSettings::default();
        assert_eq!(settings.limitFramerate, FRAMERATE_LIMIT_MAX);
        assert!(!settings.enableVsync);
    }

    #[test]
    fn load_options_preserves_vanilla_framerate_normalization() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "mc112-frame-options-{}-{unique}",
            std::process::id(),
        ));
        fs::create_dir_all(&directory).unwrap();

        fs::write(
            directory.join("options.txt"),
            "maxFps:0\nenableVsync:false\n",
        )
        .unwrap();
        let unlimited = GameSettings::loadFromGameDir(&directory).unwrap();
        assert_eq!(unlimited.limitFramerate, FRAMERATE_LIMIT_MAX);
        assert!(!unlimited.enableVsync);

        fs::write(
            directory.join("options.txt"),
            "maxFps:60\nenableVsync:true\n",
        )
        .unwrap();
        let vsync = GameSettings::loadFromGameDir(&directory).unwrap();
        assert_eq!(vsync.limitFramerate, FRAMERATE_LIMIT_MAX);
        assert!(vsync.enableVsync);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn save_options_preserves_unknown_entries_and_updates_last_server() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "mc112-save-options-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        fs::write(
            directory.join("options.txt"),
            "unknownModOption:keep-me\nlastServer:old.example\n",
        )
        .unwrap();
        let mut settings = GameSettings::loadFromGameDir(&directory).unwrap();
        settings.lastServer = "localhost:25565".to_owned();
        settings.forceSprint = true;
        settings.resourcePacks = vec!["Vanilla Test.zip".to_owned(), "Folder Pack".to_owned()];
        settings.incompatibleResourcePacks = vec!["Legacy Test.zip".to_owned()];
        settings.saveOptions(&directory).unwrap();
        let saved = fs::read_to_string(directory.join("options.txt")).unwrap();
        assert!(saved.contains("unknownModOption:keep-me\n"));
        assert!(saved.contains("lastServer:localhost:25565\n"));
        assert!(saved.contains("forceSprint:true\n"));
        assert!(saved.contains("key_key.forward:17\n"));
        assert!(saved.contains("key_key.attack:-100\n"));
        assert!(saved.contains("touchscreen:false\n"));
        assert!(saved.contains("version:1343\n"));
        assert!(saved.contains("renderClouds:true\n"));
        assert!(saved.contains("resourcePacks:[\"Vanilla Test.zip\",\"Folder Pack\"]\n"));
        assert!(saved.contains("incompatibleResourcePacks:[\"Legacy Test.zip\"]\n"));
        assert!(saved.contains("rustRenderBackend:vulkan\n"));
        assert!(!saved.contains("clouds:2\n"));
        fs::remove_dir_all(directory).unwrap();
    }
}

/// Vulkan implementation choices that do not exist in the original options.txt.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VulkanBackendSettings {
    pub frames_in_flight: u32,
    pub prefer_discrete_gpu: bool,
    pub enable_validation: bool,
    pub pipeline_cache: bool,
    pub async_chunk_uploads: bool,
}

impl Default for VulkanBackendSettings {
    fn default() -> Self {
        Self {
            frames_in_flight: 2,
            prefer_discrete_gpu: true,
            enable_validation: cfg!(debug_assertions),
            pipeline_cache: true,
            async_chunk_uploads: true,
        }
    }
}
