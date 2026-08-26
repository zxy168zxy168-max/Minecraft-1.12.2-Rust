use std::{fs, io, path::PathBuf};

use crate::net::optifine::shader::IShaderPack::IShaderPack;
use crate::net::optifine::shader::Shaders::packNameDefault;

/// OptiFine's class-path backed `(internal)` shader pack.
///
/// The Java implementation resolves `/shaders/...` from the OptiFine JAR.
/// The Rust launcher may supply the extracted class-path root. Keeping that
/// root explicit avoids silently redirecting the internal pack to Minecraft's
/// unrelated `assets/minecraft/shaders` post-processing programs.
#[derive(Debug, Default, Clone)]
pub struct ShaderPackDefault {
    classPathRoot: Option<PathBuf>,
}

impl ShaderPackDefault {
    pub fn new(classPathRoot: Option<PathBuf>) -> Self {
        Self { classPathRoot }
    }
}

impl IShaderPack for ShaderPackDefault {
    fn getName(&self) -> &str {
        packNameDefault
    }

    fn getResourceAsStream(&mut self, resName: &str) -> io::Result<Option<Vec<u8>>> {
        let Some(root) = self.classPathRoot.as_ref() else {
            return Ok(None);
        };
        let relative = resName.trim_start_matches('/');
        match fs::read(root.join(relative)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(_) => Ok(None),
        }
    }

    fn hasDirectory(&mut self, _name: &str) -> bool {
        false
    }
    fn close(&mut self) {}
}
