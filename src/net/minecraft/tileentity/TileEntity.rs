use crate::net::minecraft::nbt::NBTTagCompound::NBTTagCompound;
use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileEntityRegistryEntry {
    pub registryPath: &'static str,
    pub javaClass: &'static str,
}

/// Registry/lifecycle base of MCP 1.12.2 `TileEntity`.
///
/// Batch 124 ports the authoritative registry independently from concrete
/// runtime subclasses. This prevents the persistence layer from accepting an
/// invented ID while also avoiding fake TileEntity instances for subclasses
/// that are not yet implemented on the integrated-server side.
pub struct TileEntity;
const REGISTRY: [TileEntityRegistryEntry; 25] = [
    TileEntityRegistryEntry {
        registryPath: "furnace",
        javaClass: "TileEntityFurnace",
    },
    TileEntityRegistryEntry {
        registryPath: "chest",
        javaClass: "TileEntityChest",
    },
    TileEntityRegistryEntry {
        registryPath: "ender_chest",
        javaClass: "TileEntityEnderChest",
    },
    TileEntityRegistryEntry {
        registryPath: "jukebox",
        javaClass: "BlockJukebox.TileEntityJukebox",
    },
    TileEntityRegistryEntry {
        registryPath: "dispenser",
        javaClass: "TileEntityDispenser",
    },
    TileEntityRegistryEntry {
        registryPath: "dropper",
        javaClass: "TileEntityDropper",
    },
    TileEntityRegistryEntry {
        registryPath: "sign",
        javaClass: "TileEntitySign",
    },
    TileEntityRegistryEntry {
        registryPath: "mob_spawner",
        javaClass: "TileEntityMobSpawner",
    },
    TileEntityRegistryEntry {
        registryPath: "noteblock",
        javaClass: "TileEntityNote",
    },
    TileEntityRegistryEntry {
        registryPath: "piston",
        javaClass: "TileEntityPiston",
    },
    TileEntityRegistryEntry {
        registryPath: "brewing_stand",
        javaClass: "TileEntityBrewingStand",
    },
    TileEntityRegistryEntry {
        registryPath: "enchanting_table",
        javaClass: "TileEntityEnchantmentTable",
    },
    TileEntityRegistryEntry {
        registryPath: "end_portal",
        javaClass: "TileEntityEndPortal",
    },
    TileEntityRegistryEntry {
        registryPath: "beacon",
        javaClass: "TileEntityBeacon",
    },
    TileEntityRegistryEntry {
        registryPath: "skull",
        javaClass: "TileEntitySkull",
    },
    TileEntityRegistryEntry {
        registryPath: "daylight_detector",
        javaClass: "TileEntityDaylightDetector",
    },
    TileEntityRegistryEntry {
        registryPath: "hopper",
        javaClass: "TileEntityHopper",
    },
    TileEntityRegistryEntry {
        registryPath: "comparator",
        javaClass: "TileEntityComparator",
    },
    TileEntityRegistryEntry {
        registryPath: "flower_pot",
        javaClass: "TileEntityFlowerPot",
    },
    TileEntityRegistryEntry {
        registryPath: "banner",
        javaClass: "TileEntityBanner",
    },
    TileEntityRegistryEntry {
        registryPath: "structure_block",
        javaClass: "TileEntityStructure",
    },
    TileEntityRegistryEntry {
        registryPath: "end_gateway",
        javaClass: "TileEntityEndGateway",
    },
    TileEntityRegistryEntry {
        registryPath: "command_block",
        javaClass: "TileEntityCommandBlock",
    },
    TileEntityRegistryEntry {
        registryPath: "shulker_box",
        javaClass: "TileEntityShulkerBox",
    },
    TileEntityRegistryEntry {
        registryPath: "bed",
        javaClass: "TileEntityBed",
    },
];
impl TileEntity {
    pub fn registry() -> &'static [TileEntityRegistryEntry] {
        &REGISTRY
    }
    pub fn getEntry(name: &ResourceLocation) -> Option<&'static TileEntityRegistryEntry> {
        REGISTRY
            .iter()
            .find(|entry| ResourceLocation::parse(entry.registryPath) == *name)
    }
    pub fn isRegisteredName(name: &str) -> bool {
        Self::getEntry(&ResourceLocation::parse(name)).is_some()
    }
    pub fn isRegisteredNBT(nbt: &NBTTagCompound) -> bool {
        Self::isRegisteredName(&nbt.getString("id"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_registry_has_every_vanilla_1122_tile_entity() {
        assert_eq!(TileEntity::registry().len(), 25);
        assert!(TileEntity::isRegisteredName("minecraft:sign"));
        assert!(TileEntity::isRegisteredName("minecraft:mob_spawner"));
        assert!(TileEntity::isRegisteredName("minecraft:bed"));
        assert!(!TileEntity::isRegisteredName("minecraft:not_real"));
    }
}
