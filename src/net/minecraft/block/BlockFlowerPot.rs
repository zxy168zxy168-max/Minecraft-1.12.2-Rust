use crate::net::minecraft::tileentity::TileEntityFlowerPot::TileEntityFlowerPot;

pub const BLOCK_ID: i32 = 140;

/// Exact `BlockFlowerPot.EnumFlowerType#getName` value used by the 1.12.2
/// block-state mapper. `legacy_data` is ignored by the vanilla mapper.
pub fn contentsName(tile: Option<&TileEntityFlowerPot>) -> &'static str {
    let Some(tile) = tile else {
        return "empty";
    };
    let data = tile.itemData();
    match tile.itemId() {
        6 => match data.rem_euclid(6) {
            0 => "oak_sapling",
            1 => "spruce_sapling",
            2 => "birch_sapling",
            3 => "jungle_sapling",
            4 => "acacia_sapling",
            _ => "dark_oak_sapling",
        },
        31 => match data {
            0 => "dead_bush",
            2 => "fern",
            _ => "empty",
        },
        37 => "dandelion",
        38 => match data.rem_euclid(9) {
            0 => "rose",
            1 => "blue_orchid",
            2 => "allium",
            3 => "houstonia",
            4 => "red_tulip",
            5 => "orange_tulip",
            6 => "white_tulip",
            7 => "pink_tulip",
            _ => "oxeye_daisy",
        },
        39 => "mushroom_brown",
        40 => "mushroom_red",
        32 => "dead_bush",
        81 => "cactus",
        _ => "empty",
    }
}

pub fn modelVariant(contents: &str) -> String {
    format!("contents={contents}")
}

pub const CONTENTS: [&str; 22] = [
    "empty",
    "rose",
    "blue_orchid",
    "allium",
    "houstonia",
    "red_tulip",
    "orange_tulip",
    "white_tulip",
    "pink_tulip",
    "oxeye_daisy",
    "dandelion",
    "oak_sapling",
    "spruce_sapling",
    "birch_sapling",
    "jungle_sapling",
    "acacia_sapling",
    "dark_oak_sapling",
    "mushroom_red",
    "mushroom_brown",
    "dead_bush",
    "fern",
    "cactus",
];
