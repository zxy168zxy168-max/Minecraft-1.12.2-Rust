use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::compat::JavaProperties::parse_java_properties;
use crate::net::optifine::shader::config::ShaderPackZip::ShaderPackZip;
use crate::net::optifine::shader::IShaderPack::IShaderPack;
use crate::net::optifine::shader::ShaderPackDefault::ShaderPackDefault;
use crate::net::optifine::shader::ShaderPackFolder::ShaderPackFolder;
use crate::net::optifine::shader::ShaderPackNone::ShaderPackNone;

pub const packNameNone: &str = "OFF";
pub const packNameDefault: &str = "(internal)";
pub const shaderpacksdirname: &str = "shaderpacks";
pub const optionsfilename: &str = "optionsshaders.txt";
pub const shaderPackPropertyKey: &str = "shaderPack";

pub const QUALITY_MULTIPLIERS: [f32; 5] = [0.5, 0.707_106_77, 1.0, 1.414_213_5, 2.0];
pub const QUALITY_MULTIPLIER_NAMES: [&str; 5] = ["0.5x", "0.7x", "1x", "1.5x", "2x"];
pub const HAND_DEPTH_VALUES: [f32; 3] = [0.0625, 0.125, 0.25];
pub const HAND_DEPTH_NAMES: [&str; 3] = ["0.5x", "1x", "2x"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyDefaultTrueFalse {
    Default,
    True,
    False,
}

impl PropertyDefaultTrueFalse {
    pub fn parse(value: &str) -> Self {
        match value {
            "true" => Self::True,
            "false" => Self::False,
            _ => Self::Default,
        }
    }

    pub fn nextValue(&mut self) {
        *self = match self {
            Self::Default => Self::True,
            Self::True => Self::False,
            Self::False => Self::Default,
        };
    }

    pub const fn propertyValue(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::True => "true",
            Self::False => "false",
        }
    }

    pub const fn userValue(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::True => "ON",
            Self::False => "OFF",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShaderPackKind {
    None,
    Default,
    Folder(PathBuf),
    Zip(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderPackEntry {
    pub name: String,
    pub kind: ShaderPackKind,
}

impl ShaderPackEntry {
    pub fn isExternal(&self) -> bool {
        matches!(
            self.kind,
            ShaderPackKind::Folder(_) | ShaderPackKind::Zip(_)
        )
    }
}

/// Configuration and shader-pack repository portion of OptiFine 1.12.2
/// `Shaders`. The fields and defaults mirror `EnumShaderOption` and
/// `Shaders.loadConfig/storeConfig` from OptiFine HD U C6 for Minecraft 1.12.2.
#[derive(Debug, Clone)]
pub struct Shaders {
    pub gameDir: PathBuf,
    pub shaderpacksdir: PathBuf,
    pub configFile: PathBuf,
    pub currentshadername: String,
    pub configAntialiasingLevel: i32,
    pub configNormalMap: bool,
    pub configSpecularMap: bool,
    pub configRenderResMul: f32,
    pub configShadowResMul: f32,
    pub configHandDepthMul: f32,
    pub configCloudShadow: bool,
    pub configOldHandLight: PropertyDefaultTrueFalse,
    pub configOldLighting: PropertyDefaultTrueFalse,
    pub configTweakBlockDamage: bool,
    pub configShadowClipFrustrum: bool,
    pub configTexMinFilB: i32,
    pub configTexMinFilN: i32,
    pub configTexMinFilS: i32,
    pub configTexMagFilB: i32,
    pub configTexMagFilN: i32,
    pub configTexMagFilS: i32,
}

impl Shaders {
    pub fn loadConfig(gameDir: impl Into<PathBuf>) -> Self {
        let gameDir = gameDir.into();
        let shaderpacksdir = gameDir.join(shaderpacksdirname);
        let configFile = gameDir.join(optionsfilename);

        // OptiFine uses File.mkdir() and suppresses failure here.
        if !shaderpacksdir.exists() {
            let _ = fs::create_dir(&shaderpacksdir);
        }

        let values = fs::read(&configFile)
            .ok()
            .map(|bytes| parse_java_properties(&bytes))
            .unwrap_or_default();
        let mut shaders = Self {
            gameDir,
            shaderpacksdir,
            configFile,
            currentshadername: property(&values, shaderPackPropertyKey, "").to_owned(),
            configAntialiasingLevel: parse_i32(property(&values, "antialiasingLevel", "0"), 0),
            configNormalMap: parse_bool(property(&values, "normalMapEnabled", "true"), true),
            configSpecularMap: parse_bool(property(&values, "specularMapEnabled", "true"), true),
            configRenderResMul: parse_f32(property(&values, "renderResMul", "1.0"), 1.0),
            configShadowResMul: parse_f32(property(&values, "shadowResMul", "1.0"), 1.0),
            configHandDepthMul: parse_f32(property(&values, "handDepthMul", "0.125"), 0.125),
            configCloudShadow: parse_bool(property(&values, "cloudShadow", "true"), true),
            configOldHandLight: PropertyDefaultTrueFalse::parse(property(
                &values,
                "oldHandLight",
                "default",
            )),
            configOldLighting: PropertyDefaultTrueFalse::parse(property(
                &values,
                "oldLighting",
                "default",
            )),
            configTweakBlockDamage: parse_bool(
                property(&values, "tweakBlockDamage", "false"),
                false,
            ),
            configShadowClipFrustrum: parse_bool(
                property(&values, "shadowClipFrustrum", "true"),
                true,
            ),
            configTexMinFilB: parse_i32(property(&values, "TexMinFilB", "0"), 0),
            configTexMinFilN: parse_i32(property(&values, "TexMinFilN", "0"), 0),
            configTexMinFilS: parse_i32(property(&values, "TexMinFilS", "0"), 0),
            configTexMagFilB: parse_i32(property(&values, "TexMagFilB", "0"), 0),
            configTexMagFilN: parse_i32(property(&values, "TexMagFilN", "0"), 0),
            configTexMagFilS: parse_i32(property(&values, "TexMagFilS", "0"), 0),
        };
        shaders.normalizeConfig();
        if !shaders.configFile.exists() {
            let _ = shaders.storeConfig();
        }
        shaders
    }

    fn normalizeConfig(&mut self) {
        self.configAntialiasingLevel = match self.configAntialiasingLevel {
            2 => 2,
            4 => 4,
            _ => 0,
        };
        if !self.configRenderResMul.is_finite() {
            self.configRenderResMul = 1.0;
        }
        if !self.configShadowResMul.is_finite() {
            self.configShadowResMul = 1.0;
        }
        if !self.configHandDepthMul.is_finite() {
            self.configHandDepthMul = 0.125;
        }
    }

    /// OptiFine `Shaders.listOfShaders()` ordering and filtering. Java's
    /// `File.listFiles()` does not parse pack contents; this remains metadata-only.
    pub fn listOfShaders(&self) -> Vec<ShaderPackEntry> {
        let mut result = vec![
            ShaderPackEntry {
                name: packNameNone.to_owned(),
                kind: ShaderPackKind::None,
            },
            ShaderPackEntry {
                name: packNameDefault.to_owned(),
                kind: ShaderPackKind::Default,
            },
        ];

        if !self.shaderpacksdir.exists() {
            let _ = fs::create_dir(&self.shaderpacksdir);
        }
        let Ok(entries) = fs::read_dir(&self.shaderpacksdir) else {
            return result;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            if path.is_dir() {
                let shaders = path.join("shaders");
                if shaders.exists() && shaders.is_dir() {
                    result.push(ShaderPackEntry {
                        name,
                        kind: ShaderPackKind::Folder(path),
                    });
                }
            } else if path.is_file() && name.to_ascii_lowercase().ends_with(".zip") {
                result.push(ShaderPackEntry {
                    name,
                    kind: ShaderPackKind::Zip(path),
                });
            }
        }
        result
    }

    pub fn setShaderPack(&mut self, name: impl Into<String>) -> io::Result<()> {
        self.currentshadername = name.into();
        self.storeConfig()
    }

    pub fn nextAntialiasingLevel(&mut self) {
        self.configAntialiasingLevel += 2;
        self.configAntialiasingLevel = self.configAntialiasingLevel / 2 * 2;
        if self.configAntialiasingLevel > 4 {
            self.configAntialiasingLevel = 0;
        }
        self.configAntialiasingLevel = self.configAntialiasingLevel.clamp(0, 4);
    }

    pub fn cycleRenderQuality(&mut self, previous: bool) {
        self.configRenderResMul =
            cycle_float(self.configRenderResMul, &QUALITY_MULTIPLIERS, previous);
    }

    pub fn cycleShadowQuality(&mut self, previous: bool) {
        self.configShadowResMul =
            cycle_float(self.configShadowResMul, &QUALITY_MULTIPLIERS, previous);
    }

    pub fn cycleHandDepth(&mut self, previous: bool) {
        self.configHandDepthMul =
            cycle_float(self.configHandDepthMul, &HAND_DEPTH_VALUES, previous);
    }

    /// Equivalent to the resource-container selection portion at the start of
    /// OptiFine `Shaders.loadShaderPack()`.
    pub fn loadShaderPack(&self, classPathRoot: Option<PathBuf>) -> Box<dyn IShaderPack> {
        let name = self.currentshadername.as_str();
        if name.is_empty() || name == packNameNone {
            return Box::new(ShaderPackNone);
        }
        if name == packNameDefault {
            return Box::new(ShaderPackDefault::new(classPathRoot));
        }

        let file = self.shaderpacksdir.join(name);
        if file.is_dir() {
            Box::new(ShaderPackFolder::new(name, file))
        } else if file.is_file() && name.to_ascii_lowercase().ends_with(".zip") {
            Box::new(ShaderPackZip::new(name, file))
        } else {
            Box::new(ShaderPackNone)
        }
    }

    pub fn selectedIndex(&self, packs: &[ShaderPackEntry]) -> usize {
        packs
            .iter()
            .position(|entry| entry.name == self.currentshadername)
            .unwrap_or(0)
    }

    pub fn storeConfig(&self) -> io::Result<()> {
        let properties = self.configProperties();
        let existing = fs::read_to_string(&self.configFile).unwrap_or_default();
        let mut output = String::with_capacity(existing.len().max(512) + 64);
        let mut written = HashMap::<&str, bool>::new();

        for line in existing.split_inclusive(['\n', '\r']) {
            let trimmed = line.trim_start_matches([' ', '\t', '\u{000C}']);
            let key = property_key(trimmed);
            if let Some((propertyKey, propertyValue)) = properties
                .iter()
                .find(|(propertyKey, _)| Some(*propertyKey) == key)
            {
                if !written.get(propertyKey).copied().unwrap_or(false) {
                    output.push_str(propertyKey);
                    output.push('=');
                    output.push_str(&escape_property_value(propertyValue));
                    output.push('\n');
                    written.insert(propertyKey, true);
                }
            } else if !line.is_empty() {
                output.push_str(line);
            }
        }
        for (propertyKey, propertyValue) in properties {
            if written.get(propertyKey).copied().unwrap_or(false) {
                continue;
            }
            if !output.is_empty() && !output.ends_with(['\n', '\r']) {
                output.push('\n');
            }
            output.push_str(propertyKey);
            output.push('=');
            output.push_str(&escape_property_value(&propertyValue));
            output.push('\n');
        }
        fs::write(&self.configFile, output)
    }

    fn configProperties(&self) -> Vec<(&'static str, String)> {
        vec![
            (
                "antialiasingLevel",
                self.configAntialiasingLevel.to_string(),
            ),
            ("normalMapEnabled", self.configNormalMap.to_string()),
            ("specularMapEnabled", self.configSpecularMap.to_string()),
            ("renderResMul", format_float(self.configRenderResMul)),
            ("shadowResMul", format_float(self.configShadowResMul)),
            ("handDepthMul", format_float(self.configHandDepthMul)),
            ("cloudShadow", self.configCloudShadow.to_string()),
            (
                "oldHandLight",
                self.configOldHandLight.propertyValue().to_owned(),
            ),
            (
                "oldLighting",
                self.configOldLighting.propertyValue().to_owned(),
            ),
            (shaderPackPropertyKey, self.currentshadername.clone()),
            ("tweakBlockDamage", self.configTweakBlockDamage.to_string()),
            (
                "shadowClipFrustrum",
                self.configShadowClipFrustrum.to_string(),
            ),
            ("TexMinFilB", self.configTexMinFilB.to_string()),
            ("TexMinFilN", self.configTexMinFilN.to_string()),
            ("TexMinFilS", self.configTexMinFilS.to_string()),
            ("TexMagFilB", self.configTexMagFilB.to_string()),
            ("TexMagFilN", self.configTexMagFilN.to_string()),
            ("TexMagFilS", self.configTexMagFilS.to_string()),
        ]
    }

    pub fn shaderpacksDir(&self) -> &Path {
        &self.shaderpacksdir
    }
}

pub fn valueIndex(value: f32, values: &[f32]) -> usize {
    values
        .iter()
        .position(|candidate| *candidate >= value)
        .unwrap_or(values.len().saturating_sub(1))
}

pub fn qualityName(value: f32) -> &'static str {
    QUALITY_MULTIPLIER_NAMES[valueIndex(value, &QUALITY_MULTIPLIERS)]
}

pub fn handDepthName(value: f32) -> &'static str {
    HAND_DEPTH_NAMES[valueIndex(value, &HAND_DEPTH_VALUES)]
}

fn cycle_float(value: f32, values: &[f32], previous: bool) -> f32 {
    if values.is_empty() {
        return value;
    }
    let mut index = valueIndex(value, values);
    if previous {
        index = if index == 0 {
            values.len() - 1
        } else {
            index - 1
        };
    } else {
        index = (index + 1) % values.len();
    }
    values[index]
}

fn property<'a>(values: &'a HashMap<String, String>, key: &str, default: &'a str) -> &'a str {
    values.get(key).map(String::as_str).unwrap_or(default)
}

fn parse_bool(value: &str, _default: bool) -> bool {
    // Java Boolean.parseBoolean, used by OptiFine 1.12.2 Config.parseBoolean,
    // trims the property and returns true only for case-insensitive "true".
    value.trim().eq_ignore_ascii_case("true")
}

fn parse_i32(value: &str, default: i32) -> i32 {
    value.parse().unwrap_or(default)
}

fn parse_f32(value: &str, default: f32) -> f32 {
    value
        .parse()
        .ok()
        .filter(|value: &f32| value.is_finite())
        .unwrap_or(default)
}

fn format_float(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

fn property_key(line: &str) -> Option<&str> {
    if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
        return None;
    }
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if !escaped && (matches!(ch, '=' | ':') || matches!(ch, ' ' | '\t' | '\u{000C}')) {
            return Some(&line[..index]);
        }
        if ch == '\\' {
            escaped = !escaped;
        } else {
            escaped = false;
        }
    }
    Some(line.trim_end_matches(['\n', '\r']))
}

fn escape_property_value(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    for (index, ch) in value.chars().enumerate() {
        match ch {
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{000C}' => result.push_str("\\f"),
            '=' | ':' | '#' | '!' => {
                result.push('\\');
                result.push(ch);
            }
            ' ' if index == 0 => result.push_str("\\ "),
            ch if (ch as u32) < 0x20 || (ch as u32) > 0x7E => {
                let mut units = [0u16; 2];
                for unit in ch.encode_utf16(&mut units) {
                    result.push_str(&format!("\\u{unit:04X}"));
                }
            }
            _ => result.push(ch),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::optifine::shader::IShaderPack::IShaderPack;
    use std::{
        io::Write,
        time::{SystemTime, UNIX_EPOCH},
    };
    use zip::{write::SimpleFileOptions, ZipWriter};

    fn temp_game_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("mc112-shaders-{unique}"))
    }

    #[test]
    fn lists_only_optifine_eligible_folders_and_zip_files() {
        let game = temp_game_dir();
        fs::create_dir_all(game.join("shaderpacks/FolderPack/shaders")).unwrap();
        fs::create_dir_all(game.join("shaderpacks/NotAPack")).unwrap();
        fs::write(game.join("shaderpacks/readme.txt"), b"ignored").unwrap();
        let file = fs::File::create(game.join("shaderpacks/ZipPack.ZIP")).unwrap();
        let mut writer = ZipWriter::new(file);
        writer
            .start_file("shaders/gbuffers_basic.vsh", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(b"test").unwrap();
        writer.finish().unwrap();

        let shaders = Shaders::loadConfig(&game);
        let packs = shaders.listOfShaders();
        let names = packs
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(&names[..2], &[packNameNone, packNameDefault]);
        assert!(names.contains(&"FolderPack"));
        assert!(names.contains(&"ZipPack.ZIP"));
        assert!(!names.contains(&"NotAPack"));
        assert!(!names.contains(&"readme.txt"));
        let _ = fs::remove_dir_all(game);
    }

    #[test]
    fn loads_and_persists_all_original_shader_configuration_keys() {
        let game = temp_game_dir();
        fs::create_dir_all(&game).unwrap();
        fs::write(
            game.join(optionsfilename),
            b"normalMapEnabled=false\nshaderPack=OFF\nrenderResMul=1.4142135\noldLighting=false\ncustomKey=keep\n",
        ).unwrap();
        let mut shaders = Shaders::loadConfig(&game);
        assert!(!shaders.configNormalMap);
        assert_eq!(shaders.configRenderResMul, 1.414_213_5);
        assert_eq!(shaders.configOldLighting, PropertyDefaultTrueFalse::False);
        shaders.currentshadername = "Pack With Space.zip".to_owned();
        shaders.configShadowResMul = 2.0;
        shaders.storeConfig().unwrap();
        let text = fs::read_to_string(game.join(optionsfilename)).unwrap();
        for key in [
            "antialiasingLevel",
            "normalMapEnabled",
            "specularMapEnabled",
            "renderResMul",
            "shadowResMul",
            "handDepthMul",
            "cloudShadow",
            "oldHandLight",
            "oldLighting",
            "shaderPack",
            "tweakBlockDamage",
            "shadowClipFrustrum",
            "TexMinFilB",
            "TexMinFilN",
            "TexMinFilS",
            "TexMagFilB",
            "TexMagFilN",
            "TexMagFilS",
        ] {
            assert!(text.contains(&format!("{key}=")), "missing {key}");
        }
        assert!(text.contains("customKey=keep"));
        let reloaded = Shaders::loadConfig(&game);
        assert_eq!(reloaded.currentshadername, "Pack With Space.zip");
        assert_eq!(reloaded.configShadowResMul, 2.0);
        let _ = fs::remove_dir_all(game);
    }

    #[test]
    fn persists_selection_without_destroying_other_shader_options() {
        let game = temp_game_dir();
        fs::create_dir_all(&game).unwrap();
        fs::write(
            game.join(optionsfilename),
            b"shaderPack=OFF\ncustomKey=keep\nnormalMapEnabled=false\n",
        )
        .unwrap();
        let mut shaders = Shaders::loadConfig(&game);
        shaders.currentshadername = "Folder Pack".to_owned();
        shaders.storeConfig().unwrap();
        let text = fs::read_to_string(game.join(optionsfilename)).unwrap();
        assert!(text.contains(r"shaderPack=Folder\ Pack"));
        assert!(text.contains("customKey=keep"));
        assert!(text.contains("normalMapEnabled=false"));
        let _ = fs::remove_dir_all(game);
    }

    #[test]
    fn original_quality_and_hand_depth_cycles_are_preserved() {
        let game = temp_game_dir();
        let mut shaders = Shaders::loadConfig(&game);
        shaders.configRenderResMul = 1.0;
        shaders.cycleRenderQuality(false);
        assert_eq!(shaders.configRenderResMul, 1.414_213_5);
        shaders.cycleRenderQuality(true);
        assert_eq!(shaders.configRenderResMul, 1.0);
        shaders.configHandDepthMul = 0.125;
        shaders.cycleHandDepth(false);
        assert_eq!(shaders.configHandDepthMul, 0.25);
        shaders.nextAntialiasingLevel();
        assert_eq!(shaders.configAntialiasingLevel, 2);
        shaders.nextAntialiasingLevel();
        assert_eq!(shaders.configAntialiasingLevel, 4);
        shaders.nextAntialiasingLevel();
        assert_eq!(shaders.configAntialiasingLevel, 0);
        let _ = fs::remove_dir_all(game);
    }

    #[test]
    fn opens_selected_folder_pack_and_falls_back_to_off_for_missing_pack() {
        let game = temp_game_dir();
        fs::create_dir_all(game.join("shaderpacks/FolderPack/shaders")).unwrap();
        fs::write(
            game.join("shaderpacks/FolderPack/shaders/test.fsh"),
            b"shader",
        )
        .unwrap();
        let mut shaders = Shaders::loadConfig(&game);
        shaders.setShaderPack("FolderPack").unwrap();
        let mut pack = shaders.loadShaderPack(None);
        assert_eq!(pack.getName(), "FolderPack");
        assert_eq!(
            pack.getResourceAsStream("/shaders/test.fsh").unwrap(),
            Some(b"shader".to_vec())
        );
        shaders.setShaderPack("missing.zip").unwrap();
        assert_eq!(shaders.loadShaderPack(None).getName(), packNameNone);
        let _ = fs::remove_dir_all(game);
    }
}
