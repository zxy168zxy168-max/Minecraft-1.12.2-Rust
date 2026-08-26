use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use crate::net::optifine::shader::config::ShaderPackSource::{
    loadShaderOptionSource, ShaderOptionSourceCache,
};
use crate::net::optifine::shader::IShaderPack::IShaderPack;

const PROGRAM_NAMES: [&str; 32] = [
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
    "composite",
    "composite1",
    "composite2",
    "composite3",
    "composite4",
    "composite5",
    "composite6",
    "composite7",
    "final",
    "shadow",
    "shadow_solid",
    "shadow_cutout",
];

static OPTION_MODEL_CACHE: OnceLock<Mutex<HashMap<String, ShaderPackOptions>>> = OnceLock::new();

const CONST_OPTION_NAMES: [&str; 36] = [
    "shadowMapResolution",
    "shadowMapFov",
    "shadowDistance",
    "shadowDistanceRenderMul",
    "shadowIntervalSize",
    "generateShadowMipmap",
    "generateShadowColorMipmap",
    "shadowHardwareFiltering",
    "shadowHardwareFiltering0",
    "shadowHardwareFiltering1",
    "shadowtex0Mipmap",
    "shadowtexMipmap",
    "shadowtex1Mipmap",
    "shadowcolor0Mipmap",
    "shadowColor0Mipmap",
    "shadowcolor1Mipmap",
    "shadowColor1Mipmap",
    "shadowtex0Nearest",
    "shadowtexNearest",
    "shadow0MinMagNearest",
    "shadowtex1Nearest",
    "shadow1MinMagNearest",
    "shadowcolor0Nearest",
    "shadowColor0Nearest",
    "shadowColor0MinMagNearest",
    "shadowcolor1Nearest",
    "shadowColor1Nearest",
    "shadowColor1MinMagNearest",
    "wetnessHalflife",
    "drynessHalflife",
    "eyeBrightnessHalflife",
    "centerDepthHalflife",
    "sunPathRotation",
    "ambientOcclusionLevel",
    "superSamplingLevel",
    "noiseTextureResolution",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShaderOptionKind {
    Switch,
    Variable,
    ConstSwitch,
    ConstVariable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderOption {
    pub name: String,
    pub description: String,
    pub value: String,
    pub values: Vec<String>,
    pub valueDefault: String,
    pub paths: Vec<String>,
    pub enabled: bool,
    pub visible: bool,
    pub kind: ShaderOptionKind,
    pub constType: Option<String>,
}

impl ShaderOption {
    pub fn isChanged(&self) -> bool {
        self.value != self.valueDefault
    }

    pub fn resetValue(&mut self) {
        self.value = self.valueDefault.clone();
    }

    pub fn nextValue(&mut self) {
        if let Some(index) = self.values.iter().position(|value| value == &self.value) {
            self.value = self.values[(index + 1) % self.values.len()].clone();
        }
    }

    pub fn prevValue(&mut self) {
        if let Some(index) = self.values.iter().position(|value| value == &self.value) {
            self.value = self.values[(index + self.values.len() - 1) % self.values.len()].clone();
        }
    }

    pub fn setValue(&mut self, value: &str) -> bool {
        if self.values.iter().any(|candidate| candidate == value) {
            self.value = value.to_owned();
            true
        } else {
            false
        }
    }

    pub fn setIndexNormalized(&mut self, normalized: f32) -> bool {
        if self.values.is_empty() {
            return false;
        }
        let maximum = self.values.len().saturating_sub(1);
        let index = (normalized.clamp(0.0, 1.0) * maximum as f32).round() as usize;
        let value = self.values[index.min(maximum)].clone();
        if value == self.value {
            return false;
        }
        self.value = value;
        true
    }

    pub fn indexNormalized(&self) -> f32 {
        if self.values.len() <= 1 {
            return 0.0;
        }
        let index = self
            .values
            .iter()
            .position(|value| value == &self.value)
            .unwrap_or(0);
        index as f32 / (self.values.len() - 1) as f32
    }

    pub fn matchesLine(&self, line: &str) -> bool {
        match self.kind {
            ShaderOptionKind::Switch | ShaderOptionKind::Variable => {
                parse_define(line).is_some_and(|parsed| parsed.name == self.name)
            }
            ShaderOptionKind::ConstSwitch | ShaderOptionKind::ConstVariable => {
                parse_const(line).is_some_and(|parsed| parsed.name == self.name)
            }
        }
    }

    pub fn sourceLine(&self) -> String {
        match self.kind {
            ShaderOptionKind::Switch => {
                if self.value == "true" {
                    format!("#define {} // Shader option ON", self.name)
                } else {
                    format!("//#define {} // Shader option OFF", self.name)
                }
            }
            ShaderOptionKind::Variable => {
                format!(
                    "#define {} {} // Shader option {}",
                    self.name, self.value, self.value
                )
            }
            ShaderOptionKind::ConstSwitch => {
                format!(
                    "const bool {} = {}; // Shader option {}",
                    self.name, self.value, self.value
                )
            }
            ShaderOptionKind::ConstVariable => {
                let ty = self.constType.as_deref().unwrap_or("float");
                format!(
                    "const {ty} {} = {}; // Shader option {}",
                    self.name, self.value, self.value
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderProfile {
    pub name: String,
    pub optionValues: BTreeMap<String, String>,
    pub disabledPrograms: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct ShaderPackOptions {
    pub packName: String,
    pub options: Vec<ShaderOption>,
    pub sliders: HashSet<String>,
    pub screens: HashMap<String, Vec<String>>,
    pub screenColumns: HashMap<String, usize>,
    pub profiles: Vec<ShaderProfile>,
    /// `shaders.properties`: only an explicit `shadowTranslucent=false`
    /// disables the translucent terrain stage in the shadow traversal.
    pub shadowTranslucent: bool,
    /// Pack-local tri-state properties from `shaders.properties`. `None`
    /// corresponds to OptiFine's `default` value.
    pub oldHandLight: Option<bool>,
    pub oldLighting: Option<bool>,
    pub translations: HashMap<String, String>,
    /// Normalized language code represented by `translations`. OptiFine keeps
    /// the selected pack and its option model alive globally; retaining this
    /// tag lets the Rust equivalent reuse the same model without reopening a
    /// large ZIP merely to reload the same language file.
    pub translationLanguage: String,
    pub configPath: PathBuf,
    pub cacheKey: String,
}

impl Default for ShaderPackOptions {
    fn default() -> Self {
        Self {
            packName: String::new(),
            options: Vec::new(),
            sliders: HashSet::new(),
            screens: HashMap::new(),
            screenColumns: HashMap::new(),
            profiles: Vec::new(),
            shadowTranslucent: true,
            oldHandLight: None,
            oldLighting: None,
            translations: HashMap::new(),
            translationLanguage: String::new(),
            configPath: PathBuf::new(),
            cacheKey: String::new(),
        }
    }
}

impl ShaderPackOptions {
    pub fn load(
        gameDir: &Path,
        pack: &mut dyn IShaderPack,
        dimensions: &[i32],
    ) -> io::Result<Self> {
        Self::loadForLanguage(gameDir, pack, dimensions, "en_US")
    }

    pub fn loadCachedForLanguage(
        gameDir: &Path,
        pack: &mut dyn IShaderPack,
        dimensions: &[i32],
        language: &str,
    ) -> io::Result<Self> {
        if let Some(options) = Self::tryLoadCachedForLanguage(gameDir, pack, language)? {
            return Ok(options);
        }
        Self::loadForLanguage(gameDir, pack, dimensions, language)
    }

    /// Returns the current selected-pack option model without scanning shader
    /// sources. This mirrors OptiFine's process-wide `Shaders.shaderPackOptions`:
    /// opening `GuiShaderOptions` after the pack was compiled must not parse all
    /// `.vsh`, `.fsh`, and include files a second time.
    pub fn tryLoadCachedForLanguage(
        gameDir: &Path,
        pack: &mut dyn IShaderPack,
        language: &str,
    ) -> io::Result<Option<Self>> {
        let packName = pack.getName().to_owned();
        let cacheKey = optionCacheKey(gameDir, &packName, &[]);
        let cached = OPTION_MODEL_CACHE
            .get_or_init(|| Mutex::new(HashMap::new()))
            .lock()
            .ok()
            .and_then(|cache| cache.get(&cacheKey).cloned());
        let Some(mut options) = cached else {
            return Ok(None);
        };

        reloadSavedValues(&mut options);
        let requestedLanguage = normalize_language_code(language);
        if options.translationLanguage != requestedLanguage {
            options.translations = load_translations(pack, &requestedLanguage)?;
            options.translationLanguage = requestedLanguage;
            cacheOptionModel(&options);
        }
        log::info!(
            "Reused cached OptiFine shader option model for {}",
            packName,
        );
        Ok(Some(options))
    }

    pub fn loadForLanguage(
        gameDir: &Path,
        pack: &mut dyn IShaderPack,
        dimensions: &[i32],
        language: &str,
    ) -> io::Result<Self> {
        let packName = pack.getName().to_owned();
        let mut merged = BTreeMap::<String, ShaderOption>::new();
        let mut sourceCache = ShaderOptionSourceCache::default();
        let mut directories = vec!["/shaders".to_owned()];
        for dimension in dimensions {
            let directory = format!("/shaders/world{dimension}");
            if pack.hasDirectory(&directory) {
                directories.push(directory);
            }
        }

        for directory in directories {
            for program in PROGRAM_NAMES {
                for extension in ["vsh", "fsh"] {
                    let path = format!("{directory}/{program}.{extension}");
                    let Some(source) = loadShaderOptionSource(pack, &path, &mut sourceCache)?
                    else {
                        continue;
                    };
                    collect_options_from_source(source.as_ref(), &path, &mut merged);
                }
            }
        }

        let sourceStats = sourceCache.stats();
        log::info!(
            "OptiFine shader option source scan for {}: resource_reads={}, expanded={}, cache_hits={}, resident_entries={}",
            packName,
            sourceStats.resourceReads,
            sourceStats.expansions,
            sourceStats.cacheHits,
            sourceStats.residentEntries,
        );

        let configPath = gameDir.join("shaderpacks").join(format!("{packName}.txt"));
        let cacheKey = optionCacheKey(gameDir, &packName, dimensions);
        let saved = fs::read(&configPath)
            .ok()
            .map(|bytes| parse_java_properties(&bytes))
            .unwrap_or_default();
        let mut options = merged.into_values().collect::<Vec<_>>();
        // ShaderPackParser sorts names case-insensitively before exposing them
        // to the GUI. Keep a deterministic case-sensitive tie breaker.
        options.sort_by(|left, right| {
            left.name
                .to_ascii_lowercase()
                .cmp(&right.name.to_ascii_lowercase())
                .then_with(|| left.name.cmp(&right.name))
        });
        for option in &mut options {
            if let Some(value) = saved.get(&option.name) {
                if !option.setValue(value) {
                    log::warn!(
                        "[Shaders] Invalid saved option value: {}={}, using {}",
                        option.name,
                        value,
                        option.valueDefault,
                    );
                    option.resetValue();
                }
            }
        }

        let properties = pack
            .getResourceAsStream("/shaders/shaders.properties")?
            .map(|bytes| parse_java_properties(&bytes))
            .unwrap_or_default();
        let sliders = parse_sliders(&properties, &options);
        let profiles = parse_profiles(&properties, &mut options);
        let screens = parse_screens(&properties, &mut options, !profiles.is_empty());
        let screenColumns = parse_screen_columns(&properties, &screens);
        let shadowTranslucent = properties
            .get("shadowTranslucent")
            .map(|value| !value.trim().eq_ignore_ascii_case("false"))
            .unwrap_or(true);
        let oldHandLight = parse_optional_bool(properties.get("oldHandLight"));
        let oldLighting = parse_optional_bool(properties.get("oldLighting"));
        let translationLanguage = normalize_language_code(language);
        let translations = load_translations(pack, &translationLanguage)?;

        let result = Self {
            packName,
            options,
            sliders,
            screens,
            screenColumns,
            profiles,
            shadowTranslucent,
            oldHandLight,
            oldLighting,
            translations,
            translationLanguage,
            configPath,
            cacheKey,
        };
        cacheOptionModel(&result);
        Ok(result)
    }

    pub fn translate<'a>(&'a self, key: &str, fallback: &'a str) -> &'a str {
        self.translations
            .get(key)
            .map(String::as_str)
            .unwrap_or(fallback)
    }

    pub fn optionNameText<'a>(&'a self, option: &'a ShaderOption) -> &'a str {
        self.translate(&format!("option.{}", option.name), &option.name)
    }

    pub fn optionValueText(&self, option: &ShaderOption) -> String {
        let key = format!("value.{}.{}", option.name, option.value);
        let translated = self.translate(&key, &option.value);
        let prefix = self.translate(&format!("prefix.{}", option.name), "");
        let suffix = self.translate(&format!("suffix.{}", option.name), "");
        if option.kind == ShaderOptionKind::Switch || option.kind == ShaderOptionKind::ConstSwitch {
            if translated == option.value {
                return if option.value.eq_ignore_ascii_case("true") {
                    "ON".to_owned()
                } else {
                    "OFF".to_owned()
                };
            }
        }
        format!("{prefix}{translated}{suffix}")
    }

    pub fn optionDescriptionText(&self, option: &ShaderOption) -> String {
        let fallback = option.description.trim().trim_start_matches("//").trim();
        self.translate(&format!("option.{}.comment", option.name), fallback)
            .to_owned()
    }

    pub fn screenText<'a>(&'a self, name: &'a str) -> &'a str {
        self.translate(&format!("screen.{name}"), name)
    }

    pub fn screenDescription(&self, name: &str) -> Option<String> {
        let key = format!("screen.{name}.comment");
        self.translations.get(&key).cloned()
    }

    /// Returns the exact `GuiShaderOptions` token sequence for one screen.
    /// `*` expands only options not already named on that screen; nested
    /// `[screen]`, `<profile>`, and `<empty>` entries are preserved for the GUI.
    pub fn screenTokens(&self, screenName: Option<&str>) -> Vec<String> {
        let key = screenName
            .map(|name| format!("screen.{name}"))
            .unwrap_or_else(|| "screen".to_owned());
        let Some(raw) = self.screens.get(&key) else {
            let mut fallback = Vec::new();
            if !self.profiles.is_empty() {
                fallback.push("<profile>".to_owned());
            }
            fallback.extend(
                self.options
                    .iter()
                    .filter(|option| option.visible)
                    .map(|option| option.name.clone()),
            );
            return fallback;
        };

        // `Shaders.getShaderOptionsRest` excludes every option explicitly
        // referenced by any configured GUI screen, not only this page.
        let screenOptionNames = self
            .screens
            .values()
            .flat_map(|tokens| tokens.iter())
            .filter_map(|token| {
                if token.starts_with('<') || token.starts_with('[') || token == "*" {
                    None
                } else {
                    Some(token.as_str())
                }
            })
            .collect::<HashSet<_>>();
        let mut output = Vec::new();
        for token in raw {
            if token == "*" || token == "<rest>" {
                output.extend(
                    self.options
                        .iter()
                        .filter(|option| {
                            option.visible && !screenOptionNames.contains(option.name.as_str())
                        })
                        .map(|option| option.name.clone()),
                );
            } else if token == "<profile>" {
                if !self.profiles.is_empty() {
                    output.push(token.clone());
                }
            } else if token == "<empty>" {
                output.push(token.clone());
            } else if let Some(name) = token
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
            {
                if self.screens.contains_key(&format!("screen.{name}")) {
                    output.push(token.clone());
                }
            } else if self
                .options
                .iter()
                .any(|option| option.visible && option.name == *token)
            {
                output.push(token.clone());
            }
        }
        output
    }

    pub fn screenColumnCount(&self, screenName: Option<&str>, fallback: usize) -> usize {
        let key = screenName
            .map(|name| format!("screen.{name}.columns"))
            .unwrap_or_else(|| "screen.columns".to_owned());
        self.screenColumns
            .get(&key)
            .copied()
            .unwrap_or(fallback)
            .clamp(1, 16)
    }

    pub fn visibleOptionIndices(&self, screenName: Option<&str>) -> Vec<usize> {
        let key = screenName
            .map(|name| format!("screen.{name}"))
            .unwrap_or_else(|| "screen".to_owned());
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        let mut visitedScreens = HashSet::new();
        self.collectScreenOptions(&key, &mut result, &mut seen, &mut visitedScreens);
        if result.is_empty() {
            for (index, option) in self.options.iter().enumerate() {
                if option.visible && seen.insert(index) {
                    result.push(index);
                }
            }
        }
        result
    }

    fn collectScreenOptions(
        &self,
        key: &str,
        result: &mut Vec<usize>,
        seen: &mut HashSet<usize>,
        visitedScreens: &mut HashSet<String>,
    ) {
        if !visitedScreens.insert(key.to_owned()) {
            return;
        }
        let Some(tokens) = self.screens.get(key) else {
            return;
        };
        let mut includeRest = false;
        for token in tokens {
            if token == "*" || token == "<rest>" {
                includeRest = true;
                continue;
            }
            if token == "<empty>" || token == "<profile>" {
                continue;
            }
            if let Some(name) = token
                .strip_prefix('[')
                .and_then(|value| value.strip_suffix(']'))
            {
                self.collectScreenOptions(&format!("screen.{name}"), result, seen, visitedScreens);
                continue;
            }
            if let Some(index) = self
                .options
                .iter()
                .position(|option| option.name == *token && option.visible)
            {
                if seen.insert(index) {
                    result.push(index);
                }
            }
        }
        if includeRest {
            for (index, option) in self.options.iter().enumerate() {
                if option.visible && seen.insert(index) {
                    result.push(index);
                }
            }
        }
    }

    pub fn hasProfileSelector(&self, screenName: Option<&str>) -> bool {
        if self.profiles.is_empty() {
            return false;
        }
        let key = screenName
            .map(|name| format!("screen.{name}"))
            .unwrap_or_else(|| "screen".to_owned());
        self.screens
            .get(&key)
            .map(|tokens| tokens.iter().any(|token| token == "<profile>"))
            .unwrap_or(true)
    }

    pub fn activeProfileIndex(&self) -> Option<usize> {
        self.profiles.iter().position(|profile| {
            profile.optionValues.iter().all(|(name, value)| {
                self.options
                    .iter()
                    .find(|option| option.name == *name)
                    .is_some_and(|option| option.value == *value)
            })
        })
    }

    pub fn defaultProfileIndex(&self) -> Option<usize> {
        self.profiles.iter().position(|profile| {
            profile.optionValues.iter().all(|(name, value)| {
                self.options
                    .iter()
                    .find(|option| option.name == *name)
                    .is_some_and(|option| option.valueDefault == *value)
            })
        })
    }

    pub fn applyProfile(&mut self, index: usize) -> bool {
        let Some(profile) = self.profiles.get(index).cloned() else {
            return false;
        };
        let mut changed = false;
        for (name, value) in profile.optionValues {
            if let Some(option) = self.options.iter_mut().find(|option| option.name == name) {
                let before = option.value.clone();
                if option.setValue(&value) && option.value != before {
                    changed = true;
                }
            }
        }
        changed
    }

    pub fn isProgramDisabled(&self, name: &str) -> bool {
        self.activeProfileIndex()
            .and_then(|index| self.profiles.get(index))
            .is_some_and(|profile| profile.disabledPrograms.contains(name))
    }

    pub fn applyLine(&self, line: &str) -> String {
        for option in &self.options {
            if option.enabled && option.isChanged() && option.matchesLine(line) {
                return option.sourceLine();
            }
        }
        line.to_owned()
    }

    pub fn save(&self) -> io::Result<()> {
        let mut lines = Vec::new();
        for option in &self.options {
            if option.enabled && option.isChanged() {
                lines.push(format!(
                    "{}={}",
                    escape_property_value(&option.name),
                    escape_property_value(&option.value)
                ));
            }
        }
        let result = if lines.is_empty() {
            match fs::remove_file(&self.configPath) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            }
        } else {
            if let Some(parent) = self.configPath.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&self.configPath, format!("{}\n", lines.join("\n")))
        };
        if result.is_ok() {
            cacheOptionModel(self);
        }
        result
    }

    pub fn resetAll(&mut self) -> bool {
        let mut changed = false;
        for option in &mut self.options {
            if option.value != option.valueDefault {
                option.resetValue();
                changed = true;
            }
        }
        changed
    }
}

fn parse_optional_bool(value: Option<&String>) -> Option<bool> {
    match value.map(|value| value.trim().to_ascii_lowercase()) {
        Some(value) if value == "true" => Some(true),
        Some(value) if value == "false" => Some(false),
        _ => None,
    }
}

fn optionCacheKey(gameDir: &Path, packName: &str, _dimensions: &[i32]) -> String {
    // OptiFine owns one option model for the currently loaded pack. Dimensions
    // are part of that model's parse input, not a second independently visible
    // model, so the selected pack path is the correct cache identity.
    format!("{}|{}", gameDir.to_string_lossy(), packName)
}

fn cacheOptionModel(options: &ShaderPackOptions) {
    if options.cacheKey.is_empty() {
        return;
    }
    if let Ok(mut cache) = OPTION_MODEL_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        cache.insert(options.cacheKey.clone(), options.clone());
    }
}

fn reloadSavedValues(options: &mut ShaderPackOptions) {
    let saved = fs::read(&options.configPath)
        .ok()
        .map(|bytes| parse_java_properties(&bytes))
        .unwrap_or_default();
    for option in &mut options.options {
        option.resetValue();
        if let Some(value) = saved.get(&option.name) {
            if !option.setValue(value) {
                option.resetValue();
            }
        }
    }
}

#[derive(Debug)]
struct ParsedDefine {
    name: String,
    value: Option<String>,
    description: String,
    values: Vec<String>,
    commented: bool,
}

#[derive(Debug)]
struct ParsedConst {
    name: String,
    value: String,
    ty: String,
    description: String,
    values: Vec<String>,
    boolType: bool,
}

fn collect_options_from_source(
    source: &str,
    path: &str,
    output: &mut BTreeMap<String, ShaderOption>,
) {
    for line in source.lines() {
        let parsed = parse_define(line)
            .map(|parsed| {
                let kind = if parsed.value.is_some() {
                    ShaderOptionKind::Variable
                } else {
                    ShaderOptionKind::Switch
                };
                let value = parsed
                    .value
                    .clone()
                    .unwrap_or_else(|| (!parsed.commented).to_string());
                let values = if kind == ShaderOptionKind::Switch {
                    vec!["false".to_owned(), "true".to_owned()]
                } else {
                    parsed.values.clone()
                };
                (parsed.name, parsed.description, value, values, kind, None)
            })
            .or_else(|| {
                let parsed = parse_const(line)?;
                if !CONST_OPTION_NAMES.contains(&parsed.name.as_str()) {
                    return None;
                }
                let kind = if parsed.boolType {
                    ShaderOptionKind::ConstSwitch
                } else {
                    ShaderOptionKind::ConstVariable
                };
                let values = if parsed.boolType {
                    vec!["false".to_owned(), "true".to_owned()]
                } else {
                    parsed.values.clone()
                };
                Some((
                    parsed.name,
                    parsed.description,
                    parsed.value,
                    values,
                    kind,
                    Some(parsed.ty),
                ))
            });
        let Some((name, description, value, values, kind, constType)) = parsed else {
            continue;
        };
        if name.starts_with("MC_") {
            continue;
        }
        if matches!(kind, ShaderOptionKind::Switch) && !source_uses_switch(source, &name) {
            continue;
        }
        let visible = match kind {
            ShaderOptionKind::Switch => true,
            ShaderOptionKind::Variable | ShaderOptionKind::ConstVariable => values.len() > 1,
            ShaderOptionKind::ConstSwitch => false,
        };
        let path = path.strip_prefix("/shaders/").unwrap_or(path);
        if let Some(existing) = output.get_mut(&name) {
            if existing.valueDefault != value {
                existing.enabled = false;
                log::warn!(
                    "[Shaders] Ambiguous shader option {}: {} in {:?}, {} in {}",
                    name,
                    existing.valueDefault,
                    existing.paths,
                    value,
                    path,
                );
            }
            if existing.description.is_empty() && !description.is_empty() {
                existing.description = description;
            }
            if !existing.paths.iter().any(|candidate| candidate == path) {
                existing.paths.push(path.to_owned());
            }
        } else {
            output.insert(
                name.clone(),
                ShaderOption {
                    name,
                    description,
                    value: value.clone(),
                    values,
                    valueDefault: value,
                    paths: vec![path.to_owned()],
                    enabled: true,
                    visible,
                    kind,
                    constType,
                },
            );
        }
    }
}

fn parse_define(line: &str) -> Option<ParsedDefine> {
    let mut trimmed = line.trim_start();
    let commented = trimmed.starts_with("//");
    if commented {
        trimmed = trimmed[2..].trim_start();
    }
    let rest = trimmed.strip_prefix("#define")?.trim_start();
    let (code, comment) = split_comment(rest);
    let mut tokens = code.split_whitespace();
    let name = tokens.next()?.to_owned();
    if !valid_identifier(&name) {
        return None;
    }
    let value = tokens.next().map(str::to_owned);
    if tokens.next().is_some() {
        return None;
    }
    if value
        .as_deref()
        .is_some_and(|value| !valid_define_value(value))
    {
        return None;
    }
    let (mut values, description) = parse_values_comment(comment);
    if let Some(default) = value.as_ref() {
        if values.is_empty() {
            values.push(default.clone());
        } else if !values.iter().any(|candidate| candidate == default) {
            values.insert(0, default.clone());
        }
    }
    Some(ParsedDefine {
        name,
        value,
        description,
        values,
        commented,
    })
}

fn parse_const(line: &str) -> Option<ParsedConst> {
    let (code, comment) = split_comment(line.trim());
    let code = code.trim_end_matches(';').trim();
    let mut tokens = code.split_whitespace();
    if tokens.next()? != "const" {
        return None;
    }
    let ty = tokens.next()?;
    if !matches!(ty, "bool" | "int" | "float") {
        return None;
    }
    let name = tokens.next()?.to_owned();
    if !valid_identifier(&name) || tokens.next()? != "=" {
        return None;
    }
    let value = tokens.next()?.trim_end_matches(';').to_owned();
    if tokens.next().is_some() {
        return None;
    }
    if ty == "bool" && !matches!(value.as_str(), "true" | "false") {
        return None;
    }
    if ty != "bool" && !valid_numeric_constant(&value) {
        return None;
    }
    let (mut values, description) = parse_values_comment(comment);
    if ty == "bool" {
        values = vec!["false".to_owned(), "true".to_owned()];
    } else if values.is_empty() {
        values.push(value.clone());
    } else if !values.iter().any(|candidate| candidate == &value) {
        values.insert(0, value.clone());
    }
    Some(ParsedConst {
        name,
        value,
        ty: ty.to_owned(),
        description,
        values,
        boolType: ty == "bool",
    })
}

fn split_comment(value: &str) -> (&str, &str) {
    if let Some(index) = value.find("//") {
        (&value[..index], &value[index + 2..])
    } else {
        (value, "")
    }
}

fn parse_values_comment(comment: &str) -> (Vec<String>, String) {
    let comment = comment.trim();
    let Some(start) = comment.find('[') else {
        return (Vec::new(), comment.to_owned());
    };
    let Some(relativeEnd) = comment[start + 1..].find(']') else {
        return (Vec::new(), comment.to_owned());
    };
    let end = start + 1 + relativeEnd;
    let values = comment[start + 1..end]
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let mut description = String::new();
    description.push_str(comment[..start].trim());
    if !description.is_empty() && !comment[end + 1..].trim().is_empty() {
        description.push(' ');
    }
    description.push_str(comment[end + 1..].trim());
    (values, description)
}

fn source_uses_switch(source: &str, name: &str) -> bool {
    // Exact `ShaderOptionSwitch.PATTERN_IFDEF`: only a complete #ifdef or
    // #ifndef line makes a switch discoverable in OptiFine 1.12.2.
    source.lines().any(|line| {
        let line = line.trim();
        line.strip_prefix("#ifdef")
            .or_else(|| line.strip_prefix("#ifndef"))
            .is_some_and(|rest| rest.trim() == name)
    })
}

fn valid_define_value(value: &str) -> bool {
    valid_numeric_constant(value)
        || (!value.is_empty()
            && value
                .bytes()
                .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric()))
}

fn valid_numeric_constant(value: &str) -> bool {
    let value = value
        .strip_suffix('f')
        .or_else(|| value.strip_suffix('F'))
        .unwrap_or(value);
    if value.is_empty() {
        return false;
    }
    let value = value.strip_prefix('-').unwrap_or(value);
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'.')
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn parse_sliders(
    properties: &HashMap<String, String>,
    options: &[ShaderOption],
) -> HashSet<String> {
    let valid = options
        .iter()
        .map(|option| option.name.as_str())
        .collect::<HashSet<_>>();
    properties
        .get("sliders")
        .into_iter()
        .flat_map(|value| value.split_whitespace())
        .filter(|name| valid.contains(*name))
        .map(str::to_owned)
        .collect()
}

fn parse_screens(
    properties: &HashMap<String, String>,
    options: &mut [ShaderOption],
    hasProfiles: bool,
) -> HashMap<String, Vec<String>> {
    fn parse_one(
        key: &str,
        properties: &HashMap<String, String>,
        options: &mut [ShaderOption],
        hasProfiles: bool,
        output: &mut HashMap<String, Vec<String>>,
        visiting: &mut HashSet<String>,
    ) -> bool {
        if output.contains_key(key) {
            return true;
        }
        if !visiting.insert(key.to_owned()) {
            log::warn!("[Shaders] Recursive shader option screen: {key}");
            return false;
        }
        let Some(value) = properties.get(key) else {
            visiting.remove(key);
            return false;
        };
        let mut tokens = Vec::new();
        let mut used = HashSet::new();
        for raw in value.split_whitespace() {
            if raw == "<empty>" {
                tokens.push(raw.to_owned());
                continue;
            }
            if !used.insert(raw.to_owned()) {
                log::warn!("[Shaders] Duplicate option: {raw}, key: {key}");
                continue;
            }
            if raw == "<profile>" {
                if hasProfiles {
                    tokens.push(raw.to_owned());
                } else {
                    log::warn!("[Shaders] Option profile can not be used, no profiles defined: {raw}, key: {key}");
                }
                continue;
            }
            if raw == "*" {
                tokens.push(raw.to_owned());
                continue;
            }
            if let Some(name) = raw
                .strip_prefix('[')
                .and_then(|token| token.strip_suffix(']'))
            {
                if valid_identifier(name)
                    && parse_one(
                        &format!("screen.{name}"),
                        properties,
                        options,
                        hasProfiles,
                        output,
                        visiting,
                    )
                {
                    tokens.push(raw.to_owned());
                } else {
                    log::warn!("[Shaders] Invalid screen: {raw}, key: {key}");
                }
                continue;
            }
            if let Some(option) = options.iter_mut().find(|option| option.name == raw) {
                option.visible = true;
                tokens.push(raw.to_owned());
            } else {
                // `ShaderPackParser.parseGuiScreen` inserts a null option for an
                // invalid ordinary name, preserving the configured slot.
                log::warn!("[Shaders] Invalid option: {raw}, key: {key}");
                tokens.push("<empty>".to_owned());
            }
        }
        visiting.remove(key);
        output.insert(key.to_owned(), tokens);
        true
    }

    let mut output = HashMap::new();
    let mut visiting = HashSet::new();
    parse_one(
        "screen",
        properties,
        options,
        hasProfiles,
        &mut output,
        &mut visiting,
    );
    output
}

fn parse_screen_columns(
    properties: &HashMap<String, String>,
    screens: &HashMap<String, Vec<String>>,
) -> HashMap<String, usize> {
    screens
        .keys()
        .filter_map(|key| {
            let property = format!("{key}.columns");
            let columns = properties
                .get(&property)?
                .trim()
                .parse::<usize>()
                .ok()?
                .max(1);
            Some((property, columns))
        })
        .collect()
}

fn parse_profiles(
    properties: &HashMap<String, String>,
    options: &mut [ShaderOption],
) -> Vec<ShaderProfile> {
    let raw = properties
        .iter()
        .filter_map(|(key, value)| {
            key.strip_prefix("profile.")
                .map(|name| (name.to_owned(), value.clone()))
        })
        .collect::<BTreeMap<_, _>>();
    let mut output = BTreeMap::<String, ShaderProfile>::new();
    for name in raw.keys() {
        let mut visiting = HashSet::new();
        if let Some(profile) =
            parse_profile_recursive(name, &raw, options, &mut output, &mut visiting)
        {
            output.insert(name.clone(), profile);
        }
    }
    output.into_values().collect()
}

fn parse_profile_recursive(
    name: &str,
    raw: &BTreeMap<String, String>,
    options: &mut [ShaderOption],
    parsed: &mut BTreeMap<String, ShaderProfile>,
    visiting: &mut HashSet<String>,
) -> Option<ShaderProfile> {
    if let Some(profile) = parsed.get(name) {
        return Some(profile.clone());
    }
    if !visiting.insert(name.to_owned()) {
        log::warn!("[Shaders] Recursive shader profile: {name}");
        return None;
    }
    let definition = raw.get(name)?;
    let mut profile = ShaderProfile {
        name: name.to_owned(),
        optionValues: BTreeMap::new(),
        disabledPrograms: BTreeSet::new(),
    };
    for token in definition.split_whitespace() {
        if let Some(parent) = token.strip_prefix("profile.") {
            if let Some(parentProfile) =
                parse_profile_recursive(parent, raw, options, parsed, visiting)
            {
                profile.optionValues.extend(parentProfile.optionValues);
                profile
                    .disabledPrograms
                    .extend(parentProfile.disabledPrograms);
            }
            continue;
        }
        if let Some(program) = token.strip_prefix("!program.") {
            profile.disabledPrograms.insert(program.to_owned());
            continue;
        }
        let (rawName, rawValue) =
            if let Some(index) = token.find(|character| character == ':' || character == '=') {
                (&token[..index], Some(&token[index + 1..]))
            } else {
                token
                    .strip_prefix('!')
                    .map(|optionName| (optionName, Some("false")))
                    .unwrap_or((token, Some("true")))
            };
        let Some(option) = options.iter_mut().find(|option| option.name == rawName) else {
            log::warn!("[Shaders] Invalid option: {token}");
            continue;
        };
        let value = rawValue.unwrap_or("true");
        if option.values.iter().any(|candidate| candidate == value) {
            option.visible = true;
            profile
                .optionValues
                .insert(rawName.to_owned(), value.to_owned());
        } else {
            log::warn!("[Shaders] Invalid value: {token}");
        }
    }
    visiting.remove(name);
    parsed.insert(name.to_owned(), profile.clone());
    Some(profile)
}

fn normalize_language_code(language: &str) -> String {
    let mut parts = language.split(['_', '-']);
    let language = parts.next().unwrap_or("en").to_ascii_lowercase();
    let region = parts.next().unwrap_or("US").to_ascii_uppercase();
    format!("{language}_{region}")
}

fn load_translations(
    pack: &mut dyn IShaderPack,
    language: &str,
) -> io::Result<HashMap<String, String>> {
    let current = normalize_language_code(language);
    let mut names = vec!["en_US".to_owned()];
    if current != "en_US" {
        names.push(current);
    }
    let mut output = HashMap::new();
    for name in names {
        let path = format!("/shaders/lang/{name}.lang");
        if let Some(bytes) = pack.getResourceAsStream(&path)? {
            output.extend(parse_shader_lang(&bytes));
        }
    }
    Ok(output)
}

fn parse_shader_lang(bytes: &[u8]) -> HashMap<String, String> {
    // Exact `Lang.loadLocaleData` contract used by
    // `Shaders.loadShaderPackResources`: UTF-8, comments only when `#` is the
    // first character, and one split at the first '='. Shader language files
    // are not Java Properties streams.
    let text = String::from_utf8_lossy(bytes);
    let mut output = HashMap::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        output.insert(key.to_owned(), normalize_lang_format_tokens(value));
    }
    output
}

fn normalize_lang_format_tokens(value: &str) -> String {
    // OptiFine's Lang converts numeric printf placeholders to string
    // placeholders: %(index$)?[digits/period]*[df] -> %$1s.
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(value.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            let character = value[index..]
                .chars()
                .next()
                .expect("valid UTF-8 language value");
            output.push(character);
            index += character.len_utf8();
            continue;
        }
        let start = index;
        index += 1;
        let mut argument = "";
        let digitStart = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index < bytes.len() && bytes[index] == b'$' && index > digitStart {
            index += 1;
            argument = &value[digitStart..index];
        } else {
            index = digitStart;
        }
        while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
            index += 1;
        }
        if index < bytes.len() && matches!(bytes[index], b'd' | b'f') {
            output.push('%');
            output.push_str(argument);
            output.push('s');
            index += 1;
        } else {
            output.push_str(&value[start..index.max(start + 1)]);
        }
    }
    output
}

fn parse_java_properties(bytes: &[u8]) -> HashMap<String, String> {
    // `java.util.Properties.load(InputStream)` decodes ISO-8859-1 and expands
    // `\\uXXXX`. This parser is used for shaders.properties and the persisted
    // `<pack>.txt`, not for `/shaders/lang/*.lang`.
    let text: String = bytes.iter().map(|byte| char::from(*byte)).collect();
    let mut logical = Vec::new();
    let mut current = String::new();
    let mut continuing = false;
    for raw in text.lines() {
        let line = if continuing {
            raw.trim_start_matches(|character: char| matches!(character, ' ' | '\t' | '\u{000C}'))
        } else {
            raw
        };
        let trailing = line
            .chars()
            .rev()
            .take_while(|character| *character == '\\')
            .count();
        if trailing % 2 == 1 {
            current.push_str(line.strip_suffix('\\').unwrap_or(line));
            continuing = true;
        } else {
            current.push_str(line);
            logical.push(std::mem::take(&mut current));
            continuing = false;
        }
    }
    if !current.is_empty() {
        logical.push(current);
    }

    let mut output = HashMap::new();
    for line in logical {
        let line =
            line.trim_start_matches(|character: char| matches!(character, ' ' | '\t' | '\u{000C}'));
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }
        let mut escaped = false;
        let mut separator = None;
        for (index, character) in line.char_indices() {
            if !escaped && matches!(character, '=' | ':' | ' ' | '\t' | '\u{000C}') {
                separator = Some(index);
                break;
            }
            if character == '\\' {
                escaped = !escaped;
            } else {
                escaped = false;
            }
        }
        let (key, value) = match separator {
            Some(index) => {
                let mut rest = &line[index..];
                rest = rest.trim_start_matches(|character: char| {
                    matches!(character, ' ' | '\t' | '\u{000C}')
                });
                if rest.starts_with('=') || rest.starts_with(':') {
                    rest = &rest[1..];
                }
                rest = rest.trim_start_matches(|character: char| {
                    matches!(character, ' ' | '\t' | '\u{000C}')
                });
                (&line[..index], rest)
            }
            None => (line, ""),
        };
        output.insert(unescape_property(key), unescape_property(value));
    }
    output
}

fn unescape_property(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }
        match chars.next() {
            Some('n') => output.push('\n'),
            Some('r') => output.push('\r'),
            Some('t') => output.push('\t'),
            Some('f') => output.push('\u{000C}'),
            Some('u') => {
                let mut value = 0_u32;
                let mut valid = true;
                for _ in 0..4 {
                    match chars.next().and_then(|character| character.to_digit(16)) {
                        Some(digit) => value = value * 16 + digit,
                        None => {
                            valid = false;
                            break;
                        }
                    }
                }
                if valid {
                    output.push(char::from_u32(value).unwrap_or('\u{FFFD}'));
                } else {
                    output.push('\u{FFFD}');
                }
            }
            Some(other) => output.push(other),
            None => output.push('\\'),
        }
    }
    output
}

fn escape_property_value(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for (index, character) in value.chars().enumerate() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '=' | ':' | '#' | '!' => {
                output.push('\\');
                output.push(character);
            }
            ' ' if index == 0 => output.push_str("\\ "),
            _ => output.push(character),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_switch_and_variable_options() {
        let switch = parse_define("//#define BLOOM // Bloom toggle").unwrap();
        assert_eq!(switch.name, "BLOOM");
        assert!(switch.commented);
        assert!(switch.value.is_none());
        let variable =
            parse_define("#define SHADOW_QUALITY 1.0 // [0.5 1.0 2.0] Shadow quality").unwrap();
        assert_eq!(variable.values, ["0.5", "1.0", "2.0"]);
        assert_eq!(variable.description, "Shadow quality");
    }

    #[test]
    fn optifine_visibility_rules_hide_const_switches_and_single_value_variables() {
        let mut parsed = BTreeMap::new();
        let source = "#version 120\nconst bool shadowHardwareFiltering = true;\n#define QUALITY 1.0 // [1.0]\n#define BLOOM\n#ifdef BLOOM\n#endif\n";
        collect_options_from_source(source, "/shaders/composite.fsh", &mut parsed);
        assert!(!parsed["shadowHardwareFiltering"].visible);
        assert!(!parsed["QUALITY"].visible);
        assert!(parsed["BLOOM"].visible);
        assert_eq!(parsed["QUALITY"].values, ["1.0"]);
        assert_eq!(parsed["BLOOM"].paths, ["composite.fsh"]);
    }

    #[test]
    fn variable_option_inserts_its_default_when_the_comment_list_omits_it() {
        let parsed = parse_define("#define QUALITY 1.0 // [0.5 2.0]").unwrap();
        assert_eq!(parsed.values, ["1.0", "0.5", "2.0"]);
    }

    #[test]
    fn applies_only_changed_matching_source_lines() {
        let option = ShaderOption {
            name: "BLOOM".to_owned(),
            description: String::new(),
            value: "true".to_owned(),
            values: vec!["false".to_owned(), "true".to_owned()],
            valueDefault: "false".to_owned(),
            paths: vec!["test".to_owned()],
            enabled: true,
            visible: true,
            kind: ShaderOptionKind::Switch,
            constType: None,
        };
        let set = ShaderPackOptions {
            options: vec![option],
            ..ShaderPackOptions::default()
        };
        assert_eq!(
            set.applyLine("//#define BLOOM"),
            "#define BLOOM // Shader option ON"
        );
        assert_eq!(set.applyLine("#define OTHER"), "#define OTHER");
    }

    #[test]
    fn preserves_declared_const_numeric_type_when_applying_an_option() {
        let parsed = parse_const("const float shadowDistance = 128; // [64 128 256]").unwrap();
        assert_eq!(parsed.ty, "float");
        let option = ShaderOption {
            name: parsed.name,
            description: parsed.description,
            value: "256".to_owned(),
            values: parsed.values,
            valueDefault: parsed.value,
            paths: vec![],
            enabled: true,
            visible: true,
            kind: ShaderOptionKind::ConstVariable,
            constType: Some(parsed.ty),
        };
        assert!(option
            .sourceLine()
            .starts_with("const float shadowDistance = 256;"));
    }

    #[test]
    fn parses_profile_program_disables_and_values() {
        let mut options = vec![ShaderOption {
            name: "BLOOM".to_owned(),
            description: String::new(),
            value: "false".to_owned(),
            values: vec!["false".to_owned(), "true".to_owned()],
            valueDefault: "false".to_owned(),
            paths: vec![],
            enabled: true,
            visible: true,
            kind: ShaderOptionKind::Switch,
            constType: None,
        }];
        let properties = HashMap::from([(
            "profile.FAST".to_owned(),
            "BLOOM !program.composite7".to_owned(),
        )]);
        let profiles = parse_profiles(&properties, &mut options);
        assert_eq!(
            profiles[0].optionValues.get("BLOOM").map(String::as_str),
            Some("true")
        );
        assert!(profiles[0].disabledPrograms.contains("composite7"));
    }
    #[test]
    fn expands_rest_and_preserves_nested_screen_tokens() {
        let mut set = ShaderPackOptions::default();
        set.options = ["A", "B"]
            .into_iter()
            .map(|name| ShaderOption {
                name: name.to_owned(),
                description: String::new(),
                value: "true".to_owned(),
                values: vec!["false".to_owned(), "true".to_owned()],
                valueDefault: "true".to_owned(),
                paths: vec![],
                enabled: true,
                visible: true,
                kind: ShaderOptionKind::Switch,
                constType: None,
            })
            .collect();
        set.screens.insert(
            "screen".to_owned(),
            vec!["A".to_owned(), "[lighting]".to_owned(), "*".to_owned()],
        );
        set.screens
            .insert("screen.lighting".to_owned(), vec!["B".to_owned()]);
        set.screenColumns.insert("screen.columns".to_owned(), 3);
        assert_eq!(
            set.screenTokens(None),
            vec!["A".to_owned(), "[lighting]".to_owned()],
        );
        assert_eq!(set.screenColumnCount(None, 2), 3);
    }

    #[test]
    fn cached_option_model_reopens_no_shader_sources_for_the_same_language() {
        use std::{
            collections::HashMap,
            io,
            time::{SystemTime, UNIX_EPOCH},
        };

        struct CountingPack {
            name: String,
            resources: HashMap<String, Vec<u8>>,
            reads: usize,
        }
        impl IShaderPack for CountingPack {
            fn getName(&self) -> &str {
                &self.name
            }
            fn getResourceAsStream(&mut self, name: &str) -> io::Result<Option<Vec<u8>>> {
                self.reads += 1;
                Ok(self.resources.get(name).cloned())
            }
            fn hasDirectory(&mut self, _name: &str) -> bool {
                false
            }
            fn close(&mut self) {}
        }

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let game = std::env::temp_dir().join(format!("mc112-option-cache-{unique}"));
        let packName = format!("cache-{unique}.zip");
        let resources = HashMap::from([
            (
                "/shaders/gbuffers_basic.vsh".to_owned(),
                b"#version 120\n#define BLOOM\n#ifdef BLOOM\n#endif\n".to_vec(),
            ),
            (
                "/shaders/lang/en_US.lang".to_owned(),
                b"option.BLOOM=Bloom\n".to_vec(),
            ),
        ]);
        let mut first = CountingPack {
            name: packName.clone(),
            resources: resources.clone(),
            reads: 0,
        };
        let loaded = ShaderPackOptions::loadForLanguage(&game, &mut first, &[], "en_US").unwrap();
        assert_eq!(loaded.options.len(), 1);
        assert!(first.reads > 0);

        let mut second = CountingPack {
            name: packName,
            resources,
            reads: 0,
        };
        let cached = ShaderPackOptions::tryLoadCachedForLanguage(&game, &mut second, "en_US")
            .unwrap()
            .expect("cached selected-pack model");
        assert_eq!(cached.options.len(), 1);
        assert_eq!(second.reads, 0);
    }

    #[test]
    fn java_properties_support_continuations_and_unicode_escapes() {
        let parsed = parse_java_properties(
            b"screen=A \\
  B\nname=Shader\\u0020Options\n",
        );
        assert_eq!(parsed.get("screen").map(String::as_str), Some("A B"));
        assert_eq!(
            parsed.get("name").map(String::as_str),
            Some("Shader Options")
        );
    }

    #[test]
    fn shader_language_files_use_utf8_and_only_split_the_first_equals() {
        let parsed =
            parse_shader_lang("option.CLOUDS=云层=高质量\nvalue=FPS %1$d / %2.1f\n".as_bytes());
        assert_eq!(
            parsed.get("option.CLOUDS").map(String::as_str),
            Some("云层=高质量")
        );
        assert_eq!(
            parsed.get("value").map(String::as_str),
            Some("FPS %1$s / %s")
        );
    }

    #[test]
    fn screen_column_properties_are_not_treated_as_nested_screens() {
        let properties = HashMap::from([
            ("screen".to_owned(), "A [lighting]".to_owned()),
            ("screen.columns".to_owned(), "3".to_owned()),
            ("screen.lighting".to_owned(), "B".to_owned()),
            ("screen.lighting.columns".to_owned(), "4".to_owned()),
        ]);
        let mut options = vec![
            ShaderOption {
                name: "A".to_owned(),
                description: String::new(),
                value: "true".to_owned(),
                values: vec!["false".to_owned(), "true".to_owned()],
                valueDefault: "true".to_owned(),
                paths: vec![],
                enabled: true,
                visible: true,
                kind: ShaderOptionKind::Switch,
                constType: None,
            },
            ShaderOption {
                name: "B".to_owned(),
                description: String::new(),
                value: "true".to_owned(),
                values: vec!["false".to_owned(), "true".to_owned()],
                valueDefault: "true".to_owned(),
                paths: vec![],
                enabled: true,
                visible: true,
                kind: ShaderOptionKind::Switch,
                constType: None,
            },
        ];
        let screens = parse_screens(&properties, &mut options, false);
        let columns = parse_screen_columns(&properties, &screens);
        assert!(screens.contains_key("screen"));
        assert!(screens.contains_key("screen.lighting"));
        assert!(!screens.contains_key("screen.columns"));
        assert_eq!(columns.get("screen.columns"), Some(&3));
        assert_eq!(columns.get("screen.lighting.columns"), Some(&4));
    }
}
