use crate::net::minecraft::client::renderer::block::model::ModelResourceLocation::ModelResourceLocation;
use crate::net::minecraft::item::Item::Item;
use crate::net::minecraft::item::ItemStack::ItemStack;

/// MCP 1.12.2 `ItemModelMesher.simpleShapes` extracted by executing the
/// original RenderItem constructor after the vanilla registries were loaded.
/// The table is deliberately data-driven: no model name is inferred from an
/// item registry name or from current resource-pack contents.
pub struct ItemModelMesher;

pub const REGISTERED_ITEM_MODELS: &[(i16, i16, &str, &str)] = &[
    (0, 0, "minecraft:purpur_double_slab", "inventory"), // minecraft:air
    (1, 0, "minecraft:stone", "inventory"),              // minecraft:stone
    (1, 1, "minecraft:granite", "inventory"),            // minecraft:stone
    (1, 2, "minecraft:granite_smooth", "inventory"),     // minecraft:stone
    (1, 3, "minecraft:diorite", "inventory"),            // minecraft:stone
    (1, 4, "minecraft:diorite_smooth", "inventory"),     // minecraft:stone
    (1, 5, "minecraft:andesite", "inventory"),           // minecraft:stone
    (1, 6, "minecraft:andesite_smooth", "inventory"),    // minecraft:stone
    (2, 0, "minecraft:grass", "inventory"),              // minecraft:grass
    (3, 0, "minecraft:dirt", "inventory"),               // minecraft:dirt
    (3, 1, "minecraft:coarse_dirt", "inventory"),        // minecraft:dirt
    (3, 2, "minecraft:podzol", "inventory"),             // minecraft:dirt
    (4, 0, "minecraft:cobblestone", "inventory"),        // minecraft:cobblestone
    (5, 0, "minecraft:oak_planks", "inventory"),         // minecraft:planks
    (5, 1, "minecraft:spruce_planks", "inventory"),      // minecraft:planks
    (5, 2, "minecraft:birch_planks", "inventory"),       // minecraft:planks
    (5, 3, "minecraft:jungle_planks", "inventory"),      // minecraft:planks
    (5, 4, "minecraft:acacia_planks", "inventory"),      // minecraft:planks
    (5, 5, "minecraft:dark_oak_planks", "inventory"),    // minecraft:planks
    (6, 0, "minecraft:oak_sapling", "inventory"),        // minecraft:sapling
    (6, 1, "minecraft:spruce_sapling", "inventory"),     // minecraft:sapling
    (6, 2, "minecraft:birch_sapling", "inventory"),      // minecraft:sapling
    (6, 3, "minecraft:jungle_sapling", "inventory"),     // minecraft:sapling
    (6, 4, "minecraft:acacia_sapling", "inventory"),     // minecraft:sapling
    (6, 5, "minecraft:dark_oak_sapling", "inventory"),   // minecraft:sapling
    (7, 0, "minecraft:bedrock", "inventory"),            // minecraft:bedrock
    (12, 0, "minecraft:sand", "inventory"),              // minecraft:sand
    (12, 1, "minecraft:red_sand", "inventory"),          // minecraft:sand
    (13, 0, "minecraft:gravel", "inventory"),            // minecraft:gravel
    (14, 0, "minecraft:gold_ore", "inventory"),          // minecraft:gold_ore
    (15, 0, "minecraft:iron_ore", "inventory"),          // minecraft:iron_ore
    (16, 0, "minecraft:coal_ore", "inventory"),          // minecraft:coal_ore
    (17, 0, "minecraft:oak_log", "inventory"),           // minecraft:log
    (17, 1, "minecraft:spruce_log", "inventory"),        // minecraft:log
    (17, 2, "minecraft:birch_log", "inventory"),         // minecraft:log
    (17, 3, "minecraft:jungle_log", "inventory"),        // minecraft:log
    (18, 0, "minecraft:oak_leaves", "inventory"),        // minecraft:leaves
    (18, 1, "minecraft:spruce_leaves", "inventory"),     // minecraft:leaves
    (18, 2, "minecraft:birch_leaves", "inventory"),      // minecraft:leaves
    (18, 3, "minecraft:jungle_leaves", "inventory"),     // minecraft:leaves
    (19, 0, "minecraft:sponge", "inventory"),            // minecraft:sponge
    (19, 1, "minecraft:sponge_wet", "inventory"),        // minecraft:sponge
    (20, 0, "minecraft:glass", "inventory"),             // minecraft:glass
    (21, 0, "minecraft:lapis_ore", "inventory"),         // minecraft:lapis_ore
    (22, 0, "minecraft:lapis_block", "inventory"),       // minecraft:lapis_block
    (23, 0, "minecraft:dispenser", "inventory"),         // minecraft:dispenser
    (24, 0, "minecraft:sandstone", "inventory"),         // minecraft:sandstone
    (24, 1, "minecraft:chiseled_sandstone", "inventory"), // minecraft:sandstone
    (24, 2, "minecraft:smooth_sandstone", "inventory"),  // minecraft:sandstone
    (25, 0, "minecraft:noteblock", "inventory"),         // minecraft:noteblock
    (27, 0, "minecraft:golden_rail", "inventory"),       // minecraft:golden_rail
    (28, 0, "minecraft:detector_rail", "inventory"),     // minecraft:detector_rail
    (29, 0, "minecraft:sticky_piston", "inventory"),     // minecraft:sticky_piston
    (30, 0, "minecraft:web", "inventory"),               // minecraft:web
    (31, 0, "minecraft:dead_bush", "inventory"),         // minecraft:tallgrass
    (31, 1, "minecraft:tall_grass", "inventory"),        // minecraft:tallgrass
    (31, 2, "minecraft:fern", "inventory"),              // minecraft:tallgrass
    (32, 0, "minecraft:dead_bush", "inventory"),         // minecraft:deadbush
    (33, 0, "minecraft:piston", "inventory"),            // minecraft:piston
    (35, 0, "minecraft:white_wool", "inventory"),        // minecraft:wool
    (35, 1, "minecraft:orange_wool", "inventory"),       // minecraft:wool
    (35, 2, "minecraft:magenta_wool", "inventory"),      // minecraft:wool
    (35, 3, "minecraft:light_blue_wool", "inventory"),   // minecraft:wool
    (35, 4, "minecraft:yellow_wool", "inventory"),       // minecraft:wool
    (35, 5, "minecraft:lime_wool", "inventory"),         // minecraft:wool
    (35, 6, "minecraft:pink_wool", "inventory"),         // minecraft:wool
    (35, 7, "minecraft:gray_wool", "inventory"),         // minecraft:wool
    (35, 8, "minecraft:silver_wool", "inventory"),       // minecraft:wool
    (35, 9, "minecraft:cyan_wool", "inventory"),         // minecraft:wool
    (35, 10, "minecraft:purple_wool", "inventory"),      // minecraft:wool
    (35, 11, "minecraft:blue_wool", "inventory"),        // minecraft:wool
    (35, 12, "minecraft:brown_wool", "inventory"),       // minecraft:wool
    (35, 13, "minecraft:green_wool", "inventory"),       // minecraft:wool
    (35, 14, "minecraft:red_wool", "inventory"),         // minecraft:wool
    (35, 15, "minecraft:black_wool", "inventory"),       // minecraft:wool
    (37, 0, "minecraft:dandelion", "inventory"),         // minecraft:yellow_flower
    (38, 0, "minecraft:poppy", "inventory"),             // minecraft:red_flower
    (38, 1, "minecraft:blue_orchid", "inventory"),       // minecraft:red_flower
    (38, 2, "minecraft:allium", "inventory"),            // minecraft:red_flower
    (38, 3, "minecraft:houstonia", "inventory"),         // minecraft:red_flower
    (38, 4, "minecraft:red_tulip", "inventory"),         // minecraft:red_flower
    (38, 5, "minecraft:orange_tulip", "inventory"),      // minecraft:red_flower
    (38, 6, "minecraft:white_tulip", "inventory"),       // minecraft:red_flower
    (38, 7, "minecraft:pink_tulip", "inventory"),        // minecraft:red_flower
    (38, 8, "minecraft:oxeye_daisy", "inventory"),       // minecraft:red_flower
    (39, 0, "minecraft:brown_mushroom", "inventory"),    // minecraft:brown_mushroom
    (40, 0, "minecraft:red_mushroom", "inventory"),      // minecraft:red_mushroom
    (41, 0, "minecraft:gold_block", "inventory"),        // minecraft:gold_block
    (42, 0, "minecraft:iron_block", "inventory"),        // minecraft:iron_block
    (44, 0, "minecraft:stone_slab", "inventory"),        // minecraft:stone_slab
    (44, 1, "minecraft:sandstone_slab", "inventory"),    // minecraft:stone_slab
    (44, 2, "minecraft:old_wood_slab", "inventory"),     // minecraft:stone_slab
    (44, 3, "minecraft:cobblestone_slab", "inventory"),  // minecraft:stone_slab
    (44, 4, "minecraft:brick_slab", "inventory"),        // minecraft:stone_slab
    (44, 5, "minecraft:stone_brick_slab", "inventory"),  // minecraft:stone_slab
    (44, 6, "minecraft:nether_brick_slab", "inventory"), // minecraft:stone_slab
    (44, 7, "minecraft:quartz_slab", "inventory"),       // minecraft:stone_slab
    (45, 0, "minecraft:brick_block", "inventory"),       // minecraft:brick_block
    (46, 0, "minecraft:tnt", "inventory"),               // minecraft:tnt
    (47, 0, "minecraft:bookshelf", "inventory"),         // minecraft:bookshelf
    (48, 0, "minecraft:mossy_cobblestone", "inventory"), // minecraft:mossy_cobblestone
    (49, 0, "minecraft:obsidian", "inventory"),          // minecraft:obsidian
    (50, 0, "minecraft:torch", "inventory"),             // minecraft:torch
    (52, 0, "minecraft:mob_spawner", "inventory"),       // minecraft:mob_spawner
    (53, 0, "minecraft:oak_stairs", "inventory"),        // minecraft:oak_stairs
    (54, 0, "minecraft:chest", "inventory"),             // minecraft:chest
    (56, 0, "minecraft:diamond_ore", "inventory"),       // minecraft:diamond_ore
    (57, 0, "minecraft:diamond_block", "inventory"),     // minecraft:diamond_block
    (58, 0, "minecraft:crafting_table", "inventory"),    // minecraft:crafting_table
    (60, 0, "minecraft:farmland", "inventory"),          // minecraft:farmland
    (61, 0, "minecraft:furnace", "inventory"),           // minecraft:furnace
    (65, 0, "minecraft:ladder", "inventory"),            // minecraft:ladder
    (66, 0, "minecraft:rail", "inventory"),              // minecraft:rail
    (67, 0, "minecraft:stone_stairs", "inventory"),      // minecraft:stone_stairs
    (69, 0, "minecraft:lever", "inventory"),             // minecraft:lever
    (70, 0, "minecraft:stone_pressure_plate", "inventory"), // minecraft:stone_pressure_plate
    (72, 0, "minecraft:wooden_pressure_plate", "inventory"), // minecraft:wooden_pressure_plate
    (73, 0, "minecraft:redstone_ore", "inventory"),      // minecraft:redstone_ore
    (76, 0, "minecraft:redstone_torch", "inventory"),    // minecraft:redstone_torch
    (77, 0, "minecraft:stone_button", "inventory"),      // minecraft:stone_button
    (78, 0, "minecraft:snow_layer", "inventory"),        // minecraft:snow_layer
    (79, 0, "minecraft:ice", "inventory"),               // minecraft:ice
    (80, 0, "minecraft:snow", "inventory"),              // minecraft:snow
    (81, 0, "minecraft:cactus", "inventory"),            // minecraft:cactus
    (82, 0, "minecraft:clay", "inventory"),              // minecraft:clay
    (84, 0, "minecraft:jukebox", "inventory"),           // minecraft:jukebox
    (85, 0, "minecraft:oak_fence", "inventory"),         // minecraft:fence
    (86, 0, "minecraft:pumpkin", "inventory"),           // minecraft:pumpkin
    (87, 0, "minecraft:netherrack", "inventory"),        // minecraft:netherrack
    (88, 0, "minecraft:soul_sand", "inventory"),         // minecraft:soul_sand
    (89, 0, "minecraft:glowstone", "inventory"),         // minecraft:glowstone
    (91, 0, "minecraft:lit_pumpkin", "inventory"),       // minecraft:lit_pumpkin
    (95, 0, "minecraft:white_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 1, "minecraft:orange_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 2, "minecraft:magenta_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 3, "minecraft:light_blue_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 4, "minecraft:yellow_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 5, "minecraft:lime_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 6, "minecraft:pink_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 7, "minecraft:gray_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 8, "minecraft:silver_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 9, "minecraft:cyan_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 10, "minecraft:purple_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 11, "minecraft:blue_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 12, "minecraft:brown_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 13, "minecraft:green_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 14, "minecraft:red_stained_glass", "inventory"), // minecraft:stained_glass
    (95, 15, "minecraft:black_stained_glass", "inventory"), // minecraft:stained_glass
    (96, 0, "minecraft:trapdoor", "inventory"),          // minecraft:trapdoor
    (97, 0, "minecraft:stone_monster_egg", "inventory"), // minecraft:monster_egg
    (97, 1, "minecraft:cobblestone_monster_egg", "inventory"), // minecraft:monster_egg
    (97, 2, "minecraft:stone_brick_monster_egg", "inventory"), // minecraft:monster_egg
    (97, 3, "minecraft:mossy_brick_monster_egg", "inventory"), // minecraft:monster_egg
    (97, 4, "minecraft:cracked_brick_monster_egg", "inventory"), // minecraft:monster_egg
    (97, 5, "minecraft:chiseled_brick_monster_egg", "inventory"), // minecraft:monster_egg
    (98, 0, "minecraft:stonebrick", "inventory"),        // minecraft:stonebrick
    (98, 1, "minecraft:mossy_stonebrick", "inventory"),  // minecraft:stonebrick
    (98, 2, "minecraft:cracked_stonebrick", "inventory"), // minecraft:stonebrick
    (98, 3, "minecraft:chiseled_stonebrick", "inventory"), // minecraft:stonebrick
    (99, 0, "minecraft:brown_mushroom_block", "inventory"), // minecraft:brown_mushroom_block
    (100, 0, "minecraft:red_mushroom_block", "inventory"), // minecraft:red_mushroom_block
    (101, 0, "minecraft:iron_bars", "inventory"),        // minecraft:iron_bars
    (102, 0, "minecraft:glass_pane", "inventory"),       // minecraft:glass_pane
    (103, 0, "minecraft:melon_block", "inventory"),      // minecraft:melon_block
    (106, 0, "minecraft:vine", "inventory"),             // minecraft:vine
    (107, 0, "minecraft:oak_fence_gate", "inventory"),   // minecraft:fence_gate
    (108, 0, "minecraft:brick_stairs", "inventory"),     // minecraft:brick_stairs
    (109, 0, "minecraft:stone_brick_stairs", "inventory"), // minecraft:stone_brick_stairs
    (110, 0, "minecraft:mycelium", "inventory"),         // minecraft:mycelium
    (111, 0, "minecraft:waterlily", "inventory"),        // minecraft:waterlily
    (112, 0, "minecraft:nether_brick", "inventory"),     // minecraft:nether_brick
    (113, 0, "minecraft:nether_brick_fence", "inventory"), // minecraft:nether_brick_fence
    (114, 0, "minecraft:nether_brick_stairs", "inventory"), // minecraft:nether_brick_stairs
    (116, 0, "minecraft:enchanting_table", "inventory"), // minecraft:enchanting_table
    (120, 0, "minecraft:end_portal_frame", "inventory"), // minecraft:end_portal_frame
    (121, 0, "minecraft:end_stone", "inventory"),        // minecraft:end_stone
    (122, 0, "minecraft:dragon_egg", "inventory"),       // minecraft:dragon_egg
    (123, 0, "minecraft:redstone_lamp", "inventory"),    // minecraft:redstone_lamp
    (126, 0, "minecraft:oak_slab", "inventory"),         // minecraft:wooden_slab
    (126, 1, "minecraft:spruce_slab", "inventory"),      // minecraft:wooden_slab
    (126, 2, "minecraft:birch_slab", "inventory"),       // minecraft:wooden_slab
    (126, 3, "minecraft:jungle_slab", "inventory"),      // minecraft:wooden_slab
    (126, 4, "minecraft:acacia_slab", "inventory"),      // minecraft:wooden_slab
    (126, 5, "minecraft:dark_oak_slab", "inventory"),    // minecraft:wooden_slab
    (128, 0, "minecraft:sandstone_stairs", "inventory"), // minecraft:sandstone_stairs
    (129, 0, "minecraft:emerald_ore", "inventory"),      // minecraft:emerald_ore
    (130, 0, "minecraft:ender_chest", "inventory"),      // minecraft:ender_chest
    (131, 0, "minecraft:tripwire_hook", "inventory"),    // minecraft:tripwire_hook
    (133, 0, "minecraft:emerald_block", "inventory"),    // minecraft:emerald_block
    (134, 0, "minecraft:spruce_stairs", "inventory"),    // minecraft:spruce_stairs
    (135, 0, "minecraft:birch_stairs", "inventory"),     // minecraft:birch_stairs
    (136, 0, "minecraft:jungle_stairs", "inventory"),    // minecraft:jungle_stairs
    (137, 0, "minecraft:command_block", "inventory"),    // minecraft:command_block
    (138, 0, "minecraft:beacon", "inventory"),           // minecraft:beacon
    (139, 0, "minecraft:cobblestone_wall", "inventory"), // minecraft:cobblestone_wall
    (139, 1, "minecraft:mossy_cobblestone_wall", "inventory"), // minecraft:cobblestone_wall
    (143, 0, "minecraft:wooden_button", "inventory"),    // minecraft:wooden_button
    (145, 0, "minecraft:anvil_intact", "inventory"),     // minecraft:anvil
    (145, 1, "minecraft:anvil_slightly_damaged", "inventory"), // minecraft:anvil
    (145, 2, "minecraft:anvil_very_damaged", "inventory"), // minecraft:anvil
    (146, 0, "minecraft:trapped_chest", "inventory"),    // minecraft:trapped_chest
    (
        147,
        0,
        "minecraft:light_weighted_pressure_plate",
        "inventory",
    ), // minecraft:light_weighted_pressure_plate
    (
        148,
        0,
        "minecraft:heavy_weighted_pressure_plate",
        "inventory",
    ), // minecraft:heavy_weighted_pressure_plate
    (151, 0, "minecraft:daylight_detector", "inventory"), // minecraft:daylight_detector
    (152, 0, "minecraft:redstone_block", "inventory"),   // minecraft:redstone_block
    (153, 0, "minecraft:quartz_ore", "inventory"),       // minecraft:quartz_ore
    (154, 0, "minecraft:hopper", "inventory"),           // minecraft:hopper
    (155, 0, "minecraft:quartz_block", "inventory"),     // minecraft:quartz_block
    (155, 1, "minecraft:chiseled_quartz_block", "inventory"), // minecraft:quartz_block
    (155, 2, "minecraft:quartz_column", "inventory"),    // minecraft:quartz_block
    (156, 0, "minecraft:quartz_stairs", "inventory"),    // minecraft:quartz_stairs
    (157, 0, "minecraft:activator_rail", "inventory"),   // minecraft:activator_rail
    (158, 0, "minecraft:dropper", "inventory"),          // minecraft:dropper
    (159, 0, "minecraft:white_stained_hardened_clay", "inventory"), // minecraft:stained_hardened_clay
    (
        159,
        1,
        "minecraft:orange_stained_hardened_clay",
        "inventory",
    ), // minecraft:stained_hardened_clay
    (
        159,
        2,
        "minecraft:magenta_stained_hardened_clay",
        "inventory",
    ), // minecraft:stained_hardened_clay
    (
        159,
        3,
        "minecraft:light_blue_stained_hardened_clay",
        "inventory",
    ), // minecraft:stained_hardened_clay
    (
        159,
        4,
        "minecraft:yellow_stained_hardened_clay",
        "inventory",
    ), // minecraft:stained_hardened_clay
    (159, 5, "minecraft:lime_stained_hardened_clay", "inventory"), // minecraft:stained_hardened_clay
    (159, 6, "minecraft:pink_stained_hardened_clay", "inventory"), // minecraft:stained_hardened_clay
    (159, 7, "minecraft:gray_stained_hardened_clay", "inventory"), // minecraft:stained_hardened_clay
    (
        159,
        8,
        "minecraft:silver_stained_hardened_clay",
        "inventory",
    ), // minecraft:stained_hardened_clay
    (159, 9, "minecraft:cyan_stained_hardened_clay", "inventory"), // minecraft:stained_hardened_clay
    (
        159,
        10,
        "minecraft:purple_stained_hardened_clay",
        "inventory",
    ), // minecraft:stained_hardened_clay
    (159, 11, "minecraft:blue_stained_hardened_clay", "inventory"), // minecraft:stained_hardened_clay
    (
        159,
        12,
        "minecraft:brown_stained_hardened_clay",
        "inventory",
    ), // minecraft:stained_hardened_clay
    (
        159,
        13,
        "minecraft:green_stained_hardened_clay",
        "inventory",
    ), // minecraft:stained_hardened_clay
    (159, 14, "minecraft:red_stained_hardened_clay", "inventory"), // minecraft:stained_hardened_clay
    (
        159,
        15,
        "minecraft:black_stained_hardened_clay",
        "inventory",
    ), // minecraft:stained_hardened_clay
    (160, 0, "minecraft:white_stained_glass_pane", "inventory"),   // minecraft:stained_glass_pane
    (160, 1, "minecraft:orange_stained_glass_pane", "inventory"),  // minecraft:stained_glass_pane
    (160, 2, "minecraft:magenta_stained_glass_pane", "inventory"), // minecraft:stained_glass_pane
    (
        160,
        3,
        "minecraft:light_blue_stained_glass_pane",
        "inventory",
    ), // minecraft:stained_glass_pane
    (160, 4, "minecraft:yellow_stained_glass_pane", "inventory"),  // minecraft:stained_glass_pane
    (160, 5, "minecraft:lime_stained_glass_pane", "inventory"),    // minecraft:stained_glass_pane
    (160, 6, "minecraft:pink_stained_glass_pane", "inventory"),    // minecraft:stained_glass_pane
    (160, 7, "minecraft:gray_stained_glass_pane", "inventory"),    // minecraft:stained_glass_pane
    (160, 8, "minecraft:silver_stained_glass_pane", "inventory"),  // minecraft:stained_glass_pane
    (160, 9, "minecraft:cyan_stained_glass_pane", "inventory"),    // minecraft:stained_glass_pane
    (160, 10, "minecraft:purple_stained_glass_pane", "inventory"), // minecraft:stained_glass_pane
    (160, 11, "minecraft:blue_stained_glass_pane", "inventory"),   // minecraft:stained_glass_pane
    (160, 12, "minecraft:brown_stained_glass_pane", "inventory"),  // minecraft:stained_glass_pane
    (160, 13, "minecraft:green_stained_glass_pane", "inventory"),  // minecraft:stained_glass_pane
    (160, 14, "minecraft:red_stained_glass_pane", "inventory"),    // minecraft:stained_glass_pane
    (160, 15, "minecraft:black_stained_glass_pane", "inventory"),  // minecraft:stained_glass_pane
    (161, 0, "minecraft:acacia_leaves", "inventory"),              // minecraft:leaves2
    (161, 1, "minecraft:dark_oak_leaves", "inventory"),            // minecraft:leaves2
    (162, 0, "minecraft:acacia_log", "inventory"),                 // minecraft:log2
    (162, 1, "minecraft:dark_oak_log", "inventory"),               // minecraft:log2
    (163, 0, "minecraft:acacia_stairs", "inventory"),              // minecraft:acacia_stairs
    (164, 0, "minecraft:dark_oak_stairs", "inventory"),            // minecraft:dark_oak_stairs
    (165, 0, "minecraft:slime", "inventory"),                      // minecraft:slime
    (166, 0, "minecraft:barrier", "inventory"),                    // minecraft:barrier
    (167, 0, "minecraft:iron_trapdoor", "inventory"),              // minecraft:iron_trapdoor
    (168, 0, "minecraft:prismarine", "inventory"),                 // minecraft:prismarine
    (168, 1, "minecraft:prismarine_bricks", "inventory"),          // minecraft:prismarine
    (168, 2, "minecraft:dark_prismarine", "inventory"),            // minecraft:prismarine
    (169, 0, "minecraft:sea_lantern", "inventory"),                // minecraft:sea_lantern
    (170, 0, "minecraft:hay_block", "inventory"),                  // minecraft:hay_block
    (171, 0, "minecraft:white_carpet", "inventory"),               // minecraft:carpet
    (171, 1, "minecraft:orange_carpet", "inventory"),              // minecraft:carpet
    (171, 2, "minecraft:magenta_carpet", "inventory"),             // minecraft:carpet
    (171, 3, "minecraft:light_blue_carpet", "inventory"),          // minecraft:carpet
    (171, 4, "minecraft:yellow_carpet", "inventory"),              // minecraft:carpet
    (171, 5, "minecraft:lime_carpet", "inventory"),                // minecraft:carpet
    (171, 6, "minecraft:pink_carpet", "inventory"),                // minecraft:carpet
    (171, 7, "minecraft:gray_carpet", "inventory"),                // minecraft:carpet
    (171, 8, "minecraft:silver_carpet", "inventory"),              // minecraft:carpet
    (171, 9, "minecraft:cyan_carpet", "inventory"),                // minecraft:carpet
    (171, 10, "minecraft:purple_carpet", "inventory"),             // minecraft:carpet
    (171, 11, "minecraft:blue_carpet", "inventory"),               // minecraft:carpet
    (171, 12, "minecraft:brown_carpet", "inventory"),              // minecraft:carpet
    (171, 13, "minecraft:green_carpet", "inventory"),              // minecraft:carpet
    (171, 14, "minecraft:red_carpet", "inventory"),                // minecraft:carpet
    (171, 15, "minecraft:black_carpet", "inventory"),              // minecraft:carpet
    (172, 0, "minecraft:hardened_clay", "inventory"),              // minecraft:hardened_clay
    (173, 0, "minecraft:coal_block", "inventory"),                 // minecraft:coal_block
    (174, 0, "minecraft:packed_ice", "inventory"),                 // minecraft:packed_ice
    (175, 0, "minecraft:sunflower", "inventory"),                  // minecraft:double_plant
    (175, 1, "minecraft:syringa", "inventory"),                    // minecraft:double_plant
    (175, 2, "minecraft:double_grass", "inventory"),               // minecraft:double_plant
    (175, 3, "minecraft:double_fern", "inventory"),                // minecraft:double_plant
    (175, 4, "minecraft:double_rose", "inventory"),                // minecraft:double_plant
    (175, 5, "minecraft:paeonia", "inventory"),                    // minecraft:double_plant
    (179, 0, "minecraft:red_sandstone", "inventory"),              // minecraft:red_sandstone
    (179, 1, "minecraft:chiseled_red_sandstone", "inventory"),     // minecraft:red_sandstone
    (179, 2, "minecraft:smooth_red_sandstone", "inventory"),       // minecraft:red_sandstone
    (180, 0, "minecraft:red_sandstone_stairs", "inventory"),       // minecraft:red_sandstone_stairs
    (182, 0, "minecraft:red_sandstone_slab", "inventory"),         // minecraft:stone_slab2
    (183, 0, "minecraft:spruce_fence_gate", "inventory"),          // minecraft:spruce_fence_gate
    (184, 0, "minecraft:birch_fence_gate", "inventory"),           // minecraft:birch_fence_gate
    (185, 0, "minecraft:jungle_fence_gate", "inventory"),          // minecraft:jungle_fence_gate
    (186, 0, "minecraft:dark_oak_fence_gate", "inventory"),        // minecraft:dark_oak_fence_gate
    (187, 0, "minecraft:acacia_fence_gate", "inventory"),          // minecraft:acacia_fence_gate
    (188, 0, "minecraft:spruce_fence", "inventory"),               // minecraft:spruce_fence
    (189, 0, "minecraft:birch_fence", "inventory"),                // minecraft:birch_fence
    (190, 0, "minecraft:jungle_fence", "inventory"),               // minecraft:jungle_fence
    (191, 0, "minecraft:dark_oak_fence", "inventory"),             // minecraft:dark_oak_fence
    (192, 0, "minecraft:acacia_fence", "inventory"),               // minecraft:acacia_fence
    (198, 0, "minecraft:end_rod", "inventory"),                    // minecraft:end_rod
    (199, 0, "minecraft:chorus_plant", "inventory"),               // minecraft:chorus_plant
    (200, 0, "minecraft:chorus_flower", "inventory"),              // minecraft:chorus_flower
    (201, 0, "minecraft:purpur_block", "inventory"),               // minecraft:purpur_block
    (202, 0, "minecraft:purpur_pillar", "inventory"),              // minecraft:purpur_pillar
    (203, 0, "minecraft:purpur_stairs", "inventory"),              // minecraft:purpur_stairs
    (205, 0, "minecraft:purpur_slab", "inventory"),                // minecraft:purpur_slab
    (206, 0, "minecraft:end_bricks", "inventory"),                 // minecraft:end_bricks
    (208, 0, "minecraft:grass_path", "inventory"),                 // minecraft:grass_path
    (210, 0, "minecraft:repeating_command_block", "inventory"), // minecraft:repeating_command_block
    (211, 0, "minecraft:chain_command_block", "inventory"),     // minecraft:chain_command_block
    (213, 0, "minecraft:magma", "inventory"),                   // minecraft:magma
    (214, 0, "minecraft:nether_wart_block", "inventory"),       // minecraft:nether_wart_block
    (215, 0, "minecraft:red_nether_brick", "inventory"),        // minecraft:red_nether_brick
    (216, 0, "minecraft:bone_block", "inventory"),              // minecraft:bone_block
    (217, 0, "minecraft:structure_void", "inventory"),          // minecraft:structure_void
    (218, 0, "minecraft:observer", "inventory"),                // minecraft:observer
    (219, 0, "minecraft:white_shulker_box", "inventory"),       // minecraft:white_shulker_box
    (220, 0, "minecraft:orange_shulker_box", "inventory"),      // minecraft:orange_shulker_box
    (221, 0, "minecraft:magenta_shulker_box", "inventory"),     // minecraft:magenta_shulker_box
    (222, 0, "minecraft:light_blue_shulker_box", "inventory"),  // minecraft:light_blue_shulker_box
    (223, 0, "minecraft:yellow_shulker_box", "inventory"),      // minecraft:yellow_shulker_box
    (224, 0, "minecraft:lime_shulker_box", "inventory"),        // minecraft:lime_shulker_box
    (225, 0, "minecraft:pink_shulker_box", "inventory"),        // minecraft:pink_shulker_box
    (226, 0, "minecraft:gray_shulker_box", "inventory"),        // minecraft:gray_shulker_box
    (227, 0, "minecraft:silver_shulker_box", "inventory"),      // minecraft:silver_shulker_box
    (228, 0, "minecraft:cyan_shulker_box", "inventory"),        // minecraft:cyan_shulker_box
    (229, 0, "minecraft:purple_shulker_box", "inventory"),      // minecraft:purple_shulker_box
    (230, 0, "minecraft:blue_shulker_box", "inventory"),        // minecraft:blue_shulker_box
    (231, 0, "minecraft:brown_shulker_box", "inventory"),       // minecraft:brown_shulker_box
    (232, 0, "minecraft:green_shulker_box", "inventory"),       // minecraft:green_shulker_box
    (233, 0, "minecraft:red_shulker_box", "inventory"),         // minecraft:red_shulker_box
    (234, 0, "minecraft:black_shulker_box", "inventory"),       // minecraft:black_shulker_box
    (235, 0, "minecraft:white_glazed_terracotta", "inventory"), // minecraft:white_glazed_terracotta
    (236, 0, "minecraft:orange_glazed_terracotta", "inventory"), // minecraft:orange_glazed_terracotta
    (237, 0, "minecraft:magenta_glazed_terracotta", "inventory"), // minecraft:magenta_glazed_terracotta
    (
        238,
        0,
        "minecraft:light_blue_glazed_terracotta",
        "inventory",
    ), // minecraft:light_blue_glazed_terracotta
    (239, 0, "minecraft:yellow_glazed_terracotta", "inventory"), // minecraft:yellow_glazed_terracotta
    (240, 0, "minecraft:lime_glazed_terracotta", "inventory"),   // minecraft:lime_glazed_terracotta
    (241, 0, "minecraft:pink_glazed_terracotta", "inventory"),   // minecraft:pink_glazed_terracotta
    (242, 0, "minecraft:gray_glazed_terracotta", "inventory"),   // minecraft:gray_glazed_terracotta
    (243, 0, "minecraft:silver_glazed_terracotta", "inventory"), // minecraft:silver_glazed_terracotta
    (244, 0, "minecraft:cyan_glazed_terracotta", "inventory"),   // minecraft:cyan_glazed_terracotta
    (245, 0, "minecraft:purple_glazed_terracotta", "inventory"), // minecraft:purple_glazed_terracotta
    (246, 0, "minecraft:blue_glazed_terracotta", "inventory"),   // minecraft:blue_glazed_terracotta
    (247, 0, "minecraft:brown_glazed_terracotta", "inventory"), // minecraft:brown_glazed_terracotta
    (248, 0, "minecraft:green_glazed_terracotta", "inventory"), // minecraft:green_glazed_terracotta
    (249, 0, "minecraft:red_glazed_terracotta", "inventory"),   // minecraft:red_glazed_terracotta
    (250, 0, "minecraft:black_glazed_terracotta", "inventory"), // minecraft:black_glazed_terracotta
    (251, 0, "minecraft:white_concrete", "inventory"),          // minecraft:concrete
    (251, 1, "minecraft:orange_concrete", "inventory"),         // minecraft:concrete
    (251, 2, "minecraft:magenta_concrete", "inventory"),        // minecraft:concrete
    (251, 3, "minecraft:light_blue_concrete", "inventory"),     // minecraft:concrete
    (251, 4, "minecraft:yellow_concrete", "inventory"),         // minecraft:concrete
    (251, 5, "minecraft:lime_concrete", "inventory"),           // minecraft:concrete
    (251, 6, "minecraft:pink_concrete", "inventory"),           // minecraft:concrete
    (251, 7, "minecraft:gray_concrete", "inventory"),           // minecraft:concrete
    (251, 8, "minecraft:silver_concrete", "inventory"),         // minecraft:concrete
    (251, 9, "minecraft:cyan_concrete", "inventory"),           // minecraft:concrete
    (251, 10, "minecraft:purple_concrete", "inventory"),        // minecraft:concrete
    (251, 11, "minecraft:blue_concrete", "inventory"),          // minecraft:concrete
    (251, 12, "minecraft:brown_concrete", "inventory"),         // minecraft:concrete
    (251, 13, "minecraft:green_concrete", "inventory"),         // minecraft:concrete
    (251, 14, "minecraft:red_concrete", "inventory"),           // minecraft:concrete
    (251, 15, "minecraft:black_concrete", "inventory"),         // minecraft:concrete
    (252, 0, "minecraft:white_concrete_powder", "inventory"),   // minecraft:concrete_powder
    (252, 1, "minecraft:orange_concrete_powder", "inventory"),  // minecraft:concrete_powder
    (252, 2, "minecraft:magenta_concrete_powder", "inventory"), // minecraft:concrete_powder
    (252, 3, "minecraft:light_blue_concrete_powder", "inventory"), // minecraft:concrete_powder
    (252, 4, "minecraft:yellow_concrete_powder", "inventory"),  // minecraft:concrete_powder
    (252, 5, "minecraft:lime_concrete_powder", "inventory"),    // minecraft:concrete_powder
    (252, 6, "minecraft:pink_concrete_powder", "inventory"),    // minecraft:concrete_powder
    (252, 7, "minecraft:gray_concrete_powder", "inventory"),    // minecraft:concrete_powder
    (252, 8, "minecraft:silver_concrete_powder", "inventory"),  // minecraft:concrete_powder
    (252, 9, "minecraft:cyan_concrete_powder", "inventory"),    // minecraft:concrete_powder
    (252, 10, "minecraft:purple_concrete_powder", "inventory"), // minecraft:concrete_powder
    (252, 11, "minecraft:blue_concrete_powder", "inventory"),   // minecraft:concrete_powder
    (252, 12, "minecraft:brown_concrete_powder", "inventory"),  // minecraft:concrete_powder
    (252, 13, "minecraft:green_concrete_powder", "inventory"),  // minecraft:concrete_powder
    (252, 14, "minecraft:red_concrete_powder", "inventory"),    // minecraft:concrete_powder
    (252, 15, "minecraft:black_concrete_powder", "inventory"),  // minecraft:concrete_powder
    (255, 0, "minecraft:structure_block", "inventory"),         // minecraft:structure_block
    (255, 1, "minecraft:structure_block", "inventory"),         // minecraft:structure_block
    (255, 2, "minecraft:structure_block", "inventory"),         // minecraft:structure_block
    (255, 3, "minecraft:structure_block", "inventory"),         // minecraft:structure_block
    (256, 0, "minecraft:iron_shovel", "inventory"),             // minecraft:iron_shovel
    (257, 0, "minecraft:iron_pickaxe", "inventory"),            // minecraft:iron_pickaxe
    (258, 0, "minecraft:iron_axe", "inventory"),                // minecraft:iron_axe
    (259, 0, "minecraft:flint_and_steel", "inventory"),         // minecraft:flint_and_steel
    (260, 0, "minecraft:apple", "inventory"),                   // minecraft:apple
    (261, 0, "minecraft:bow", "inventory"),                     // minecraft:bow
    (262, 0, "minecraft:arrow", "inventory"),                   // minecraft:arrow
    (263, 0, "minecraft:coal", "inventory"),                    // minecraft:coal
    (263, 1, "minecraft:charcoal", "inventory"),                // minecraft:coal
    (264, 0, "minecraft:diamond", "inventory"),                 // minecraft:diamond
    (265, 0, "minecraft:iron_ingot", "inventory"),              // minecraft:iron_ingot
    (266, 0, "minecraft:gold_ingot", "inventory"),              // minecraft:gold_ingot
    (267, 0, "minecraft:iron_sword", "inventory"),              // minecraft:iron_sword
    (268, 0, "minecraft:wooden_sword", "inventory"),            // minecraft:wooden_sword
    (269, 0, "minecraft:wooden_shovel", "inventory"),           // minecraft:wooden_shovel
    (270, 0, "minecraft:wooden_pickaxe", "inventory"),          // minecraft:wooden_pickaxe
    (271, 0, "minecraft:wooden_axe", "inventory"),              // minecraft:wooden_axe
    (272, 0, "minecraft:stone_sword", "inventory"),             // minecraft:stone_sword
    (273, 0, "minecraft:stone_shovel", "inventory"),            // minecraft:stone_shovel
    (274, 0, "minecraft:stone_pickaxe", "inventory"),           // minecraft:stone_pickaxe
    (275, 0, "minecraft:stone_axe", "inventory"),               // minecraft:stone_axe
    (276, 0, "minecraft:diamond_sword", "inventory"),           // minecraft:diamond_sword
    (277, 0, "minecraft:diamond_shovel", "inventory"),          // minecraft:diamond_shovel
    (278, 0, "minecraft:diamond_pickaxe", "inventory"),         // minecraft:diamond_pickaxe
    (279, 0, "minecraft:diamond_axe", "inventory"),             // minecraft:diamond_axe
    (280, 0, "minecraft:stick", "inventory"),                   // minecraft:stick
    (281, 0, "minecraft:bowl", "inventory"),                    // minecraft:bowl
    (282, 0, "minecraft:mushroom_stew", "inventory"),           // minecraft:mushroom_stew
    (283, 0, "minecraft:golden_sword", "inventory"),            // minecraft:golden_sword
    (284, 0, "minecraft:golden_shovel", "inventory"),           // minecraft:golden_shovel
    (285, 0, "minecraft:golden_pickaxe", "inventory"),          // minecraft:golden_pickaxe
    (286, 0, "minecraft:golden_axe", "inventory"),              // minecraft:golden_axe
    (287, 0, "minecraft:string", "inventory"),                  // minecraft:string
    (288, 0, "minecraft:feather", "inventory"),                 // minecraft:feather
    (289, 0, "minecraft:gunpowder", "inventory"),               // minecraft:gunpowder
    (290, 0, "minecraft:wooden_hoe", "inventory"),              // minecraft:wooden_hoe
    (291, 0, "minecraft:stone_hoe", "inventory"),               // minecraft:stone_hoe
    (292, 0, "minecraft:iron_hoe", "inventory"),                // minecraft:iron_hoe
    (293, 0, "minecraft:diamond_hoe", "inventory"),             // minecraft:diamond_hoe
    (294, 0, "minecraft:golden_hoe", "inventory"),              // minecraft:golden_hoe
    (295, 0, "minecraft:wheat_seeds", "inventory"),             // minecraft:wheat_seeds
    (296, 0, "minecraft:wheat", "inventory"),                   // minecraft:wheat
    (297, 0, "minecraft:bread", "inventory"),                   // minecraft:bread
    (298, 0, "minecraft:leather_helmet", "inventory"),          // minecraft:leather_helmet
    (299, 0, "minecraft:leather_chestplate", "inventory"),      // minecraft:leather_chestplate
    (300, 0, "minecraft:leather_leggings", "inventory"),        // minecraft:leather_leggings
    (301, 0, "minecraft:leather_boots", "inventory"),           // minecraft:leather_boots
    (302, 0, "minecraft:chainmail_helmet", "inventory"),        // minecraft:chainmail_helmet
    (303, 0, "minecraft:chainmail_chestplate", "inventory"),    // minecraft:chainmail_chestplate
    (304, 0, "minecraft:chainmail_leggings", "inventory"),      // minecraft:chainmail_leggings
    (305, 0, "minecraft:chainmail_boots", "inventory"),         // minecraft:chainmail_boots
    (306, 0, "minecraft:iron_helmet", "inventory"),             // minecraft:iron_helmet
    (307, 0, "minecraft:iron_chestplate", "inventory"),         // minecraft:iron_chestplate
    (308, 0, "minecraft:iron_leggings", "inventory"),           // minecraft:iron_leggings
    (309, 0, "minecraft:iron_boots", "inventory"),              // minecraft:iron_boots
    (310, 0, "minecraft:diamond_helmet", "inventory"),          // minecraft:diamond_helmet
    (311, 0, "minecraft:diamond_chestplate", "inventory"),      // minecraft:diamond_chestplate
    (312, 0, "minecraft:diamond_leggings", "inventory"),        // minecraft:diamond_leggings
    (313, 0, "minecraft:diamond_boots", "inventory"),           // minecraft:diamond_boots
    (314, 0, "minecraft:golden_helmet", "inventory"),           // minecraft:golden_helmet
    (315, 0, "minecraft:golden_chestplate", "inventory"),       // minecraft:golden_chestplate
    (316, 0, "minecraft:golden_leggings", "inventory"),         // minecraft:golden_leggings
    (317, 0, "minecraft:golden_boots", "inventory"),            // minecraft:golden_boots
    (318, 0, "minecraft:flint", "inventory"),                   // minecraft:flint
    (319, 0, "minecraft:porkchop", "inventory"),                // minecraft:porkchop
    (320, 0, "minecraft:cooked_porkchop", "inventory"),         // minecraft:cooked_porkchop
    (321, 0, "minecraft:painting", "inventory"),                // minecraft:painting
    (322, 0, "minecraft:golden_apple", "inventory"),            // minecraft:golden_apple
    (322, 1, "minecraft:golden_apple", "inventory"),            // minecraft:golden_apple
    (323, 0, "minecraft:sign", "inventory"),                    // minecraft:sign
    (324, 0, "minecraft:oak_door", "inventory"),                // minecraft:wooden_door
    (325, 0, "minecraft:bucket", "inventory"),                  // minecraft:bucket
    (326, 0, "minecraft:water_bucket", "inventory"),            // minecraft:water_bucket
    (327, 0, "minecraft:lava_bucket", "inventory"),             // minecraft:lava_bucket
    (328, 0, "minecraft:minecart", "inventory"),                // minecraft:minecart
    (329, 0, "minecraft:saddle", "inventory"),                  // minecraft:saddle
    (330, 0, "minecraft:iron_door", "inventory"),               // minecraft:iron_door
    (331, 0, "minecraft:redstone", "inventory"),                // minecraft:redstone
    (332, 0, "minecraft:snowball", "inventory"),                // minecraft:snowball
    (333, 0, "minecraft:oak_boat", "inventory"),                // minecraft:boat
    (334, 0, "minecraft:leather", "inventory"),                 // minecraft:leather
    (335, 0, "minecraft:milk_bucket", "inventory"),             // minecraft:milk_bucket
    (336, 0, "minecraft:brick", "inventory"),                   // minecraft:brick
    (337, 0, "minecraft:clay_ball", "inventory"),               // minecraft:clay_ball
    (338, 0, "minecraft:reeds", "inventory"),                   // minecraft:reeds
    (339, 0, "minecraft:paper", "inventory"),                   // minecraft:paper
    (340, 0, "minecraft:book", "inventory"),                    // minecraft:book
    (341, 0, "minecraft:slime_ball", "inventory"),              // minecraft:slime_ball
    (342, 0, "minecraft:chest_minecart", "inventory"),          // minecraft:chest_minecart
    (343, 0, "minecraft:furnace_minecart", "inventory"),        // minecraft:furnace_minecart
    (344, 0, "minecraft:egg", "inventory"),                     // minecraft:egg
    (345, 0, "minecraft:compass", "inventory"),                 // minecraft:compass
    (346, 0, "minecraft:fishing_rod", "inventory"),             // minecraft:fishing_rod
    (347, 0, "minecraft:clock", "inventory"),                   // minecraft:clock
    (348, 0, "minecraft:glowstone_dust", "inventory"),          // minecraft:glowstone_dust
    (349, 0, "minecraft:cod", "inventory"),                     // minecraft:fish
    (349, 1, "minecraft:salmon", "inventory"),                  // minecraft:fish
    (349, 2, "minecraft:clownfish", "inventory"),               // minecraft:fish
    (349, 3, "minecraft:pufferfish", "inventory"),              // minecraft:fish
    (350, 0, "minecraft:cooked_cod", "inventory"),              // minecraft:cooked_fish
    (350, 1, "minecraft:cooked_salmon", "inventory"),           // minecraft:cooked_fish
    (351, 0, "minecraft:dye_black", "inventory"),               // minecraft:dye
    (351, 1, "minecraft:dye_red", "inventory"),                 // minecraft:dye
    (351, 2, "minecraft:dye_green", "inventory"),               // minecraft:dye
    (351, 3, "minecraft:dye_brown", "inventory"),               // minecraft:dye
    (351, 4, "minecraft:dye_blue", "inventory"),                // minecraft:dye
    (351, 5, "minecraft:dye_purple", "inventory"),              // minecraft:dye
    (351, 6, "minecraft:dye_cyan", "inventory"),                // minecraft:dye
    (351, 7, "minecraft:dye_silver", "inventory"),              // minecraft:dye
    (351, 8, "minecraft:dye_gray", "inventory"),                // minecraft:dye
    (351, 9, "minecraft:dye_pink", "inventory"),                // minecraft:dye
    (351, 10, "minecraft:dye_lime", "inventory"),               // minecraft:dye
    (351, 11, "minecraft:dye_yellow", "inventory"),             // minecraft:dye
    (351, 12, "minecraft:dye_light_blue", "inventory"),         // minecraft:dye
    (351, 13, "minecraft:dye_magenta", "inventory"),            // minecraft:dye
    (351, 14, "minecraft:dye_orange", "inventory"),             // minecraft:dye
    (351, 15, "minecraft:dye_white", "inventory"),              // minecraft:dye
    (352, 0, "minecraft:bone", "inventory"),                    // minecraft:bone
    (353, 0, "minecraft:sugar", "inventory"),                   // minecraft:sugar
    (354, 0, "minecraft:cake", "inventory"),                    // minecraft:cake
    (356, 0, "minecraft:repeater", "inventory"),                // minecraft:repeater
    (357, 0, "minecraft:cookie", "inventory"),                  // minecraft:cookie
    (359, 0, "minecraft:shears", "inventory"),                  // minecraft:shears
    (360, 0, "minecraft:melon", "inventory"),                   // minecraft:melon
    (361, 0, "minecraft:pumpkin_seeds", "inventory"),           // minecraft:pumpkin_seeds
    (362, 0, "minecraft:melon_seeds", "inventory"),             // minecraft:melon_seeds
    (363, 0, "minecraft:beef", "inventory"),                    // minecraft:beef
    (364, 0, "minecraft:cooked_beef", "inventory"),             // minecraft:cooked_beef
    (365, 0, "minecraft:chicken", "inventory"),                 // minecraft:chicken
    (366, 0, "minecraft:cooked_chicken", "inventory"),          // minecraft:cooked_chicken
    (367, 0, "minecraft:rotten_flesh", "inventory"),            // minecraft:rotten_flesh
    (368, 0, "minecraft:ender_pearl", "inventory"),             // minecraft:ender_pearl
    (369, 0, "minecraft:blaze_rod", "inventory"),               // minecraft:blaze_rod
    (370, 0, "minecraft:ghast_tear", "inventory"),              // minecraft:ghast_tear
    (371, 0, "minecraft:gold_nugget", "inventory"),             // minecraft:gold_nugget
    (372, 0, "minecraft:nether_wart", "inventory"),             // minecraft:nether_wart
    (373, 0, "minecraft:bottle_drinkable", "inventory"),        // minecraft:potion
    (374, 0, "minecraft:glass_bottle", "inventory"),            // minecraft:glass_bottle
    (375, 0, "minecraft:spider_eye", "inventory"),              // minecraft:spider_eye
    (376, 0, "minecraft:fermented_spider_eye", "inventory"),    // minecraft:fermented_spider_eye
    (377, 0, "minecraft:blaze_powder", "inventory"),            // minecraft:blaze_powder
    (378, 0, "minecraft:magma_cream", "inventory"),             // minecraft:magma_cream
    (379, 0, "minecraft:brewing_stand", "inventory"),           // minecraft:brewing_stand
    (380, 0, "minecraft:cauldron", "inventory"),                // minecraft:cauldron
    (381, 0, "minecraft:ender_eye", "inventory"),               // minecraft:ender_eye
    (382, 0, "minecraft:speckled_melon", "inventory"),          // minecraft:speckled_melon
    (384, 0, "minecraft:experience_bottle", "inventory"),       // minecraft:experience_bottle
    (385, 0, "minecraft:fire_charge", "inventory"),             // minecraft:fire_charge
    (386, 0, "minecraft:writable_book", "inventory"),           // minecraft:writable_book
    (387, 0, "minecraft:written_book", "inventory"),            // minecraft:written_book
    (388, 0, "minecraft:emerald", "inventory"),                 // minecraft:emerald
    (389, 0, "minecraft:item_frame", "inventory"),              // minecraft:item_frame
    (390, 0, "minecraft:flower_pot", "inventory"),              // minecraft:flower_pot
    (391, 0, "minecraft:carrot", "inventory"),                  // minecraft:carrot
    (392, 0, "minecraft:potato", "inventory"),                  // minecraft:potato
    (393, 0, "minecraft:baked_potato", "inventory"),            // minecraft:baked_potato
    (394, 0, "minecraft:poisonous_potato", "inventory"),        // minecraft:poisonous_potato
    (395, 0, "minecraft:map", "inventory"),                     // minecraft:map
    (396, 0, "minecraft:golden_carrot", "inventory"),           // minecraft:golden_carrot
    (397, 0, "minecraft:skull_skeleton", "inventory"),          // minecraft:skull
    (397, 1, "minecraft:skull_wither", "inventory"),            // minecraft:skull
    (397, 2, "minecraft:skull_zombie", "inventory"),            // minecraft:skull
    (397, 3, "minecraft:skull_char", "inventory"),              // minecraft:skull
    (397, 4, "minecraft:skull_creeper", "inventory"),           // minecraft:skull
    (397, 5, "minecraft:skull_dragon", "inventory"),            // minecraft:skull
    (398, 0, "minecraft:carrot_on_a_stick", "inventory"),       // minecraft:carrot_on_a_stick
    (399, 0, "minecraft:nether_star", "inventory"),             // minecraft:nether_star
    (400, 0, "minecraft:pumpkin_pie", "inventory"),             // minecraft:pumpkin_pie
    (401, 0, "minecraft:fireworks", "inventory"),               // minecraft:fireworks
    (402, 0, "minecraft:firework_charge", "inventory"),         // minecraft:firework_charge
    (404, 0, "minecraft:comparator", "inventory"),              // minecraft:comparator
    (405, 0, "minecraft:netherbrick", "inventory"),             // minecraft:netherbrick
    (406, 0, "minecraft:quartz", "inventory"),                  // minecraft:quartz
    (407, 0, "minecraft:tnt_minecart", "inventory"),            // minecraft:tnt_minecart
    (408, 0, "minecraft:hopper_minecart", "inventory"),         // minecraft:hopper_minecart
    (409, 0, "minecraft:prismarine_shard", "inventory"),        // minecraft:prismarine_shard
    (410, 0, "minecraft:prismarine_crystals", "inventory"),     // minecraft:prismarine_crystals
    (411, 0, "minecraft:rabbit", "inventory"),                  // minecraft:rabbit
    (412, 0, "minecraft:cooked_rabbit", "inventory"),           // minecraft:cooked_rabbit
    (413, 0, "minecraft:rabbit_stew", "inventory"),             // minecraft:rabbit_stew
    (414, 0, "minecraft:rabbit_foot", "inventory"),             // minecraft:rabbit_foot
    (415, 0, "minecraft:rabbit_hide", "inventory"),             // minecraft:rabbit_hide
    (416, 0, "minecraft:armor_stand", "inventory"),             // minecraft:armor_stand
    (417, 0, "minecraft:iron_horse_armor", "inventory"),        // minecraft:iron_horse_armor
    (418, 0, "minecraft:golden_horse_armor", "inventory"),      // minecraft:golden_horse_armor
    (419, 0, "minecraft:diamond_horse_armor", "inventory"),     // minecraft:diamond_horse_armor
    (420, 0, "minecraft:lead", "inventory"),                    // minecraft:lead
    (421, 0, "minecraft:name_tag", "inventory"),                // minecraft:name_tag
    (422, 0, "minecraft:command_block_minecart", "inventory"),  // minecraft:command_block_minecart
    (423, 0, "minecraft:mutton", "inventory"),                  // minecraft:mutton
    (424, 0, "minecraft:cooked_mutton", "inventory"),           // minecraft:cooked_mutton
    (426, 0, "minecraft:end_crystal", "inventory"),             // minecraft:end_crystal
    (427, 0, "minecraft:spruce_door", "inventory"),             // minecraft:spruce_door
    (428, 0, "minecraft:birch_door", "inventory"),              // minecraft:birch_door
    (429, 0, "minecraft:jungle_door", "inventory"),             // minecraft:jungle_door
    (430, 0, "minecraft:acacia_door", "inventory"),             // minecraft:acacia_door
    (431, 0, "minecraft:dark_oak_door", "inventory"),           // minecraft:dark_oak_door
    (432, 0, "minecraft:chorus_fruit", "inventory"),            // minecraft:chorus_fruit
    (433, 0, "minecraft:chorus_fruit_popped", "inventory"),     // minecraft:chorus_fruit_popped
    (434, 0, "minecraft:beetroot", "inventory"),                // minecraft:beetroot
    (435, 0, "minecraft:beetroot_seeds", "inventory"),          // minecraft:beetroot_seeds
    (436, 0, "minecraft:beetroot_soup", "inventory"),           // minecraft:beetroot_soup
    (437, 0, "minecraft:dragon_breath", "inventory"),           // minecraft:dragon_breath
    (438, 0, "minecraft:bottle_splash", "inventory"),           // minecraft:splash_potion
    (439, 0, "minecraft:spectral_arrow", "inventory"),          // minecraft:spectral_arrow
    (440, 0, "minecraft:tipped_arrow", "inventory"),            // minecraft:tipped_arrow
    (441, 0, "minecraft:bottle_lingering", "inventory"),        // minecraft:lingering_potion
    (443, 0, "minecraft:elytra", "inventory"),                  // minecraft:elytra
    (444, 0, "minecraft:spruce_boat", "inventory"),             // minecraft:spruce_boat
    (445, 0, "minecraft:birch_boat", "inventory"),              // minecraft:birch_boat
    (446, 0, "minecraft:jungle_boat", "inventory"),             // minecraft:jungle_boat
    (447, 0, "minecraft:acacia_boat", "inventory"),             // minecraft:acacia_boat
    (448, 0, "minecraft:dark_oak_boat", "inventory"),           // minecraft:dark_oak_boat
    (449, 0, "minecraft:totem", "inventory"),                   // minecraft:totem_of_undying
    (450, 0, "minecraft:shulker_shell", "inventory"),           // minecraft:shulker_shell
    (452, 0, "minecraft:iron_nugget", "inventory"),             // minecraft:iron_nugget
    (453, 0, "minecraft:knowledge_book", "inventory"),          // minecraft:knowledge_book
    (2256, 0, "minecraft:record_13", "inventory"),              // minecraft:record_13
    (2257, 0, "minecraft:record_cat", "inventory"),             // minecraft:record_cat
    (2258, 0, "minecraft:record_blocks", "inventory"),          // minecraft:record_blocks
    (2259, 0, "minecraft:record_chirp", "inventory"),           // minecraft:record_chirp
    (2260, 0, "minecraft:record_far", "inventory"),             // minecraft:record_far
    (2261, 0, "minecraft:record_mall", "inventory"),            // minecraft:record_mall
    (2262, 0, "minecraft:record_mellohi", "inventory"),         // minecraft:record_mellohi
    (2263, 0, "minecraft:record_stal", "inventory"),            // minecraft:record_stal
    (2264, 0, "minecraft:record_strad", "inventory"),           // minecraft:record_strad
    (2265, 0, "minecraft:record_ward", "inventory"),            // minecraft:record_ward
    (2266, 0, "minecraft:record_11", "inventory"),              // minecraft:record_11
    (2267, 0, "minecraft:record_wait", "inventory"),            // minecraft:record_wait
];

/// MCP RenderItem mesh-definition registrations. These six items bypass
/// simpleShapes but still resolve to stable inventory ModelResourceLocations.
pub const MESH_ITEM_MODELS: &[(i16, i16, &str, &str)] = &[
    (355, 0, "minecraft:bed", "inventory"),
    (358, 0, "minecraft:filled_map", "inventory"),
    (383, 0, "minecraft:spawn_egg", "inventory"),
    (403, 0, "minecraft:enchanted_book", "inventory"),
    (425, 0, "minecraft:banner", "inventory"),
    (442, 0, "minecraft:shield", "inventory"),
];

pub fn registeredItemModels(
) -> impl Iterator<Item = &'static (i16, i16, &'static str, &'static str)> {
    REGISTERED_ITEM_MODELS.iter().chain(MESH_ITEM_MODELS.iter())
}

impl ItemModelMesher {
    pub fn getModelKey(stack: &ItemStack) -> Option<(i16, i16)> {
        if stack.isEmpty() {
            return None;
        }
        if MESH_ITEM_MODELS.iter().any(|entry| entry.0 == stack.itemId) {
            return Some((stack.itemId, 0));
        }
        let metadata = if Item::isDamageable(stack.itemId) {
            0
        } else {
            stack.itemDamage
        };
        Self::getModelLocationById(stack.itemId, metadata).map(|_| (stack.itemId, metadata))
    }

    pub fn getModelLocation(stack: &ItemStack) -> Option<ModelResourceLocation> {
        let (itemId, metadata) = Self::getModelKey(stack)?;
        Self::getModelLocationById(itemId, metadata)
    }

    pub fn getModelLocationById(itemId: i16, metadata: i16) -> Option<ModelResourceLocation> {
        if let Some(entry) = MESH_ITEM_MODELS
            .iter()
            .find(|entry| entry.0 == itemId && entry.1 == metadata)
        {
            return Some(ModelResourceLocation::new(entry.2, entry.3));
        }
        let index = REGISTERED_ITEM_MODELS
            .binary_search_by_key(&(itemId, metadata), |entry| (entry.0, entry.1))
            .ok()?;
        let entry = REGISTERED_ITEM_MODELS[index];
        Some(ModelResourceLocation::new(entry.2, entry.3))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted_and_contains_mcp_metadata_variants() {
        assert!(REGISTERED_ITEM_MODELS
            .windows(2)
            .all(|pair| (pair[0].0, pair[0].1) < (pair[1].0, pair[1].1)));
        let granite = ItemModelMesher::getModelLocationById(1, 1).unwrap();
        assert_eq!(granite.getPath(), "granite");
        assert_eq!(granite.getVariant(), "inventory");
        let redWool = ItemModelMesher::getModelLocationById(35, 14).unwrap();
        assert_eq!(redWool.getPath(), "red_wool");
        let spawnEgg = ItemModelMesher::getModelLocationById(383, 0).unwrap();
        assert_eq!(spawnEgg.getPath(), "spawn_egg");
    }

    #[test]
    fn damageable_items_fall_back_to_metadata_zero() {
        let stack = ItemStack {
            itemId: 257,
            count: 1,
            itemDamage: 37,
            tagCompound: None,
        };
        assert_eq!(
            ItemModelMesher::getModelLocation(&stack).unwrap().getPath(),
            "iron_pickaxe"
        );
    }
}
