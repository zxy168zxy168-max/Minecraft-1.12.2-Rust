use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct AssetRoot {
    root: PathBuf,
    coverage: AssetCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssetCoverage {
    pub visual_assets: bool,
    pub sound_registry: bool,
    pub sound_objects: bool,
    pub optifine_assets: bool,
}

#[derive(Debug, Error)]
pub enum AssetRootError {
    #[error("asset root does not exist: {0}")]
    MissingRoot(PathBuf),
    #[error("missing Minecraft asset namespace: {0}")]
    MissingMinecraftNamespace(PathBuf),
    #[error("missing required 1.12.2 visual asset: {0}")]
    MissingRequiredAsset(PathBuf),
}

impl AssetRoot {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, AssetRootError> {
        let root = root.into();
        if !root.is_dir() {
            return Err(AssetRootError::MissingRoot(root));
        }
        let namespace = root.join("minecraft");
        if !namespace.is_dir() {
            return Err(AssetRootError::MissingMinecraftNamespace(namespace));
        }
        for required in [
            "lang/en_us.lang",
            "textures/gui/title/minecraft.png",
            "textures/gui/widgets.png",
            "textures/font/ascii.png",
        ] {
            let path = namespace.join(required);
            if !path.is_file() {
                return Err(AssetRootError::MissingRequiredAsset(path));
            }
        }
        let coverage = AssetCoverage {
            visual_assets: true,
            sound_registry: namespace.join("sounds.json").is_file(),
            sound_objects: namespace.join("sounds").is_dir(),
            optifine_assets: namespace.join("optifine").is_dir()
                || namespace.join("mcpatcher").is_dir(),
        };
        Ok(Self { root, coverage })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn coverage(&self) -> AssetCoverage {
        self.coverage
    }
    pub fn namespace_root(&self, namespace: &str) -> PathBuf {
        self.root.join(namespace)
    }
}
