use crate::net::minecraft::util::ResourceLocation::ResourceLocation;

/// MCP 1.12.2 `net.minecraft.block.SoundType`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundType {
    volume: f32,
    pitch: f32,
    breakSound: &'static str,
    stepSound: &'static str,
    placeSound: &'static str,
    hitSound: &'static str,
    fallSound: &'static str,
}

impl SoundType {
    pub const WOOD: Self = Self::new(1.0, 1.0, "block.wood");
    pub const GROUND: Self = Self::new(1.0, 1.0, "block.gravel");
    pub const PLANT: Self = Self::new(1.0, 1.0, "block.grass");
    pub const STONE: Self = Self::new(1.0, 1.0, "block.stone");
    pub const METAL: Self = Self::new(1.0, 1.5, "block.metal");
    pub const GLASS: Self = Self::new(1.0, 1.0, "block.glass");
    pub const CLOTH: Self = Self::new(1.0, 1.0, "block.cloth");
    pub const SAND: Self = Self::new(1.0, 1.0, "block.sand");
    pub const SNOW: Self = Self::new(1.0, 1.0, "block.snow");
    pub const LADDER: Self = Self::new(1.0, 1.0, "block.ladder");
    pub const ANVIL: Self = Self::new(0.3, 1.0, "block.anvil");
    pub const SLIME: Self = Self::new(1.0, 1.0, "block.slime");

    const fn new(volume: f32, pitch: f32, prefix: &'static str) -> Self {
        // Rust cannot concatenate const strings. Concrete event names are
        // selected by the accessors below from the exact type index.
        Self {
            volume,
            pitch,
            breakSound: prefix,
            stepSound: prefix,
            placeSound: prefix,
            hitSound: prefix,
            fallSound: prefix,
        }
    }

    pub const fn getVolume(self) -> f32 {
        self.volume
    }
    pub const fn getPitch(self) -> f32 {
        self.pitch
    }
    pub fn getBreakSound(self) -> ResourceLocation {
        ResourceLocation::parse(event_name(self.breakSound, "break"))
    }
    pub fn getStepSound(self) -> ResourceLocation {
        ResourceLocation::parse(event_name(self.stepSound, "step"))
    }
    pub fn getPlaceSound(self) -> ResourceLocation {
        ResourceLocation::parse(event_name(self.placeSound, "place"))
    }
    pub fn getHitSound(self) -> ResourceLocation {
        ResourceLocation::parse(event_name(self.hitSound, "hit"))
    }
    pub fn getFallSound(self) -> ResourceLocation {
        ResourceLocation::parse(event_name(self.fallSound, "fall"))
    }

    /// Exact default `Block.blockSoundType` assigned by `Block.registerBlocks`.
    pub fn forBlockId(blockId: i32) -> Self {
        let index = blockId.clamp(0, 255) as usize;
        SOUND_TYPE_BY_BLOCK[index]
    }
}

fn event_name(prefix: &'static str, suffix: &'static str) -> &'static str {
    match (prefix, suffix) {
        ("block.stone", "break") => "block.stone.break",
        ("block.stone", "step") => "block.stone.step",
        ("block.stone", "place") => "block.stone.place",
        ("block.stone", "hit") => "block.stone.hit",
        ("block.stone", "fall") => "block.stone.fall",
        ("block.wood", "break") => "block.wood.break",
        ("block.wood", "step") => "block.wood.step",
        ("block.wood", "place") => "block.wood.place",
        ("block.wood", "hit") => "block.wood.hit",
        ("block.wood", "fall") => "block.wood.fall",
        ("block.gravel", "break") => "block.gravel.break",
        ("block.gravel", "step") => "block.gravel.step",
        ("block.gravel", "place") => "block.gravel.place",
        ("block.gravel", "hit") => "block.gravel.hit",
        ("block.gravel", "fall") => "block.gravel.fall",
        ("block.grass", "break") => "block.grass.break",
        ("block.grass", "step") => "block.grass.step",
        ("block.grass", "place") => "block.grass.place",
        ("block.grass", "hit") => "block.grass.hit",
        ("block.grass", "fall") => "block.grass.fall",
        ("block.metal", "break") => "block.metal.break",
        ("block.metal", "step") => "block.metal.step",
        ("block.metal", "place") => "block.metal.place",
        ("block.metal", "hit") => "block.metal.hit",
        ("block.metal", "fall") => "block.metal.fall",
        ("block.glass", "break") => "block.glass.break",
        ("block.glass", "step") => "block.glass.step",
        ("block.glass", "place") => "block.glass.place",
        ("block.glass", "hit") => "block.glass.hit",
        ("block.glass", "fall") => "block.glass.fall",
        ("block.cloth", "break") => "block.cloth.break",
        ("block.cloth", "step") => "block.cloth.step",
        ("block.cloth", "place") => "block.cloth.place",
        ("block.cloth", "hit") => "block.cloth.hit",
        ("block.cloth", "fall") => "block.cloth.fall",
        ("block.sand", "break") => "block.sand.break",
        ("block.sand", "step") => "block.sand.step",
        ("block.sand", "place") => "block.sand.place",
        ("block.sand", "hit") => "block.sand.hit",
        ("block.sand", "fall") => "block.sand.fall",
        ("block.snow", "break") => "block.snow.break",
        ("block.snow", "step") => "block.snow.step",
        ("block.snow", "place") => "block.snow.place",
        ("block.snow", "hit") => "block.snow.hit",
        ("block.snow", "fall") => "block.snow.fall",
        ("block.ladder", "break") => "block.ladder.break",
        ("block.ladder", "step") => "block.ladder.step",
        ("block.ladder", "place") => "block.ladder.place",
        ("block.ladder", "hit") => "block.ladder.hit",
        ("block.ladder", "fall") => "block.ladder.fall",
        ("block.anvil", "break") => "block.anvil.break",
        ("block.anvil", "step") => "block.anvil.step",
        ("block.anvil", "place") => "block.anvil.place",
        ("block.anvil", "hit") => "block.anvil.hit",
        ("block.anvil", "fall") => "block.anvil.fall",
        ("block.slime", "break") => "block.slime.break",
        ("block.slime", "step") => "block.slime.step",
        ("block.slime", "place") => "block.slime.place",
        ("block.slime", "hit") => "block.slime.hit",
        ("block.slime", "fall") => "block.slime.fall",
        _ => "block.stone.break",
    }
}

const SOUND_TYPES: [SoundType; 12] = [
    SoundType::STONE,
    SoundType::WOOD,
    SoundType::GROUND,
    SoundType::PLANT,
    SoundType::METAL,
    SoundType::GLASS,
    SoundType::CLOTH,
    SoundType::SAND,
    SoundType::SNOW,
    SoundType::LADDER,
    SoundType::ANVIL,
    SoundType::SLIME,
];

const SOUND_TYPE_INDEX_BY_BLOCK: [u8; 256] = [
    0, 0, 3, 2, 0, 1, 3, 0, 0, 0, 0, 0, 7, 2, 0, 0, 0, 1, 3, 3, 5, 0, 0, 0, 0, 1, 1, 4, 4, 0, 0, 3,
    3, 0, 0, 6, 0, 3, 3, 3, 3, 4, 4, 0, 0, 0, 3, 1, 0, 0, 1, 6, 4, 0, 1, 0, 0, 4, 1, 3, 2, 0, 0, 1,
    1, 9, 4, 0, 1, 1, 0, 4, 1, 0, 0, 1, 1, 0, 8, 5, 8, 6, 2, 3, 0, 1, 1, 0, 7, 5, 5, 1, 6, 1, 1, 5,
    1, 0, 0, 1, 1, 4, 5, 1, 1, 1, 3, 1, 0, 0, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 5, 5, 1, 1, 1,
    0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 3, 3, 1, 0, 10, 1, 1, 1, 1, 1, 1, 4, 0, 4, 0, 0, 4, 0,
    0, 5, 3, 1, 0, 0, 11, 0, 4, 0, 5, 3, 6, 0, 0, 5, 3, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 3, 3, 0, 0, 0, 5, 0, 1, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0,
    0, 0,
];

const fn build_sound_type_table() -> [SoundType; 256] {
    let mut table = [SoundType::STONE; 256];
    let mut index = 0;
    while index < 256 {
        table[index] = SOUND_TYPES[SOUND_TYPE_INDEX_BY_BLOCK[index] as usize];
        index += 1;
    }
    table
}

const SOUND_TYPE_BY_BLOCK: [SoundType; 256] = build_sound_type_table();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vanilla_examples_match_registration() {
        assert_eq!(SoundType::forBlockId(5), SoundType::WOOD);
        assert_eq!(SoundType::forBlockId(20), SoundType::GLASS);
        assert_eq!(SoundType::forBlockId(145), SoundType::ANVIL);
        assert_eq!(SoundType::forBlockId(165), SoundType::SLIME);
        assert_eq!(SoundType::forBlockId(42).getPitch(), 1.5);
    }
}
