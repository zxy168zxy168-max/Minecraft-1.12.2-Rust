use std::io;

use crate::net::optifine::shader::IShaderPack::IShaderPack;
use crate::net::optifine::shader::Shaders::packNameNone;

#[derive(Debug, Default, Clone, Copy)]
pub struct ShaderPackNone;

impl IShaderPack for ShaderPackNone {
    fn getName(&self) -> &str {
        packNameNone
    }
    fn getResourceAsStream(&mut self, _resName: &str) -> io::Result<Option<Vec<u8>>> {
        Ok(None)
    }
    fn hasDirectory(&mut self, _name: &str) -> bool {
        false
    }
    fn close(&mut self) {}
}
