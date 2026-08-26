use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::net::optifine::shader::IShaderPack::IShaderPack;

#[derive(Debug, Clone)]
pub struct ShaderPackFolder {
    pub packFile: PathBuf,
}

impl ShaderPackFolder {
    pub fn new(_name: impl AsRef<str>, file: impl Into<PathBuf>) -> Self {
        Self {
            packFile: file.into(),
        }
    }

    pub fn packFile(&self) -> &Path {
        &self.packFile
    }
}

impl IShaderPack for ShaderPackFolder {
    fn getName(&self) -> &str {
        self.packFile
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
    }

    fn getResourceAsStream(&mut self, resName: &str) -> io::Result<Option<Vec<u8>>> {
        // MCP `StrUtils.removePrefixSuffix(resName, "/", "/")` removes at
        // most one matching prefix and suffix, rather than trimming all.
        let relative = remove_prefix_suffix(resName, "/", "/");
        match fs::read(self.packFile.join(relative)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(_) => Ok(None),
        }
    }

    fn hasDirectory(&mut self, name: &str) -> bool {
        let relative = name.strip_prefix('/').unwrap_or(name);
        self.packFile.join(relative).is_dir()
    }

    fn close(&mut self) {}
}

fn remove_prefix_suffix<'a>(value: &'a str, prefix: &str, suffix: &str) -> &'a str {
    let value = value.strip_prefix(prefix).unwrap_or(value);
    value.strip_suffix(suffix).unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn mirrors_optifine_folder_path_normalization() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("mc112-shader-folder-{unique}"));
        fs::create_dir_all(root.join("shaders/world0")).unwrap();
        fs::write(root.join("shaders/gbuffers_basic.vsh"), b"folder-pack").unwrap();
        let mut pack = ShaderPackFolder::new("ignored", &root);
        assert_eq!(pack.getName(), root.file_name().unwrap().to_string_lossy());
        assert_eq!(
            pack.getResourceAsStream("/shaders/gbuffers_basic.vsh/")
                .unwrap(),
            Some(b"folder-pack".to_vec())
        );
        assert!(pack.hasDirectory("/shaders"));
        assert!(pack.hasDirectory("/shaders/world0"));
        assert!(!pack.hasDirectory("/missing"));
        let _ = fs::remove_dir_all(root);
    }
}
