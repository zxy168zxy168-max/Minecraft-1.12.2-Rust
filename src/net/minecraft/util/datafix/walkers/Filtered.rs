use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::datafix::IDataFixer::IDataFixer;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// Rust equivalent of MCP 1.12.2 `walkers.Filtered`.
/// Java derives `key` from a Class through EntityList/TileEntity registries;
/// Rust receives that already-authoritative registry key because concrete type
/// reflection is intentionally not used for not-yet-ported server subclasses.
#[derive(Debug, Clone)]
pub struct Filtered {
    key: ResourceLocation,
}
impl Filtered {
    pub fn new(registryName: &str) -> Self {
        Self {
            key: ResourceLocation::parse(registryName),
        }
    }
    pub fn matches(&self, compound: &NBTTagCompound) -> bool {
        ResourceLocation::parse(compound.getString("id")) == self.key
    }
    pub fn key(&self) -> &ResourceLocation {
        &self.key
    }
    pub fn processIf<F>(
        &self,
        fixer: &dyn IDataFixer,
        compound: NBTTagCompound,
        versionIn: i32,
        process: F,
    ) -> NBTTagCompound
    where
        F: FnOnce(&dyn IDataFixer, NBTTagCompound, i32) -> NBTTagCompound,
    {
        if self.matches(&compound) {
            process(fixer, compound, versionIn)
        } else {
            compound
        }
    }
}
