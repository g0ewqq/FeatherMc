use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::entity::Entity;
use crate::inventory::ItemStack;
use crate::persist::{decode_sections, encode_sections, WorldStore, SECTION_BLOCKS};

pub const DEFAULT_WORLD_NAME: &str = "world";
pub const DEFAULT_DIMENSION_ID: &str = "minecraft:overworld";

pub const WORLD_MIN_Y: i32 = -64;
pub const WORLD_HEIGHT: i32 = 384;
pub const SECTION_SIZE: i32 = 16;
pub const SECTION_COUNT: usize = (WORLD_HEIGHT / SECTION_SIZE) as usize;
pub const CHUNK_WIDTH: i32 = 16;

pub const SPAWN_X: f64 = 0.0;
pub const SPAWN_Y: f64 = 100.0;
pub const SPAWN_Z: f64 = 0.0;
pub const SPAWN_YAW: f32 = 0.0;
pub const SPAWN_PITCH: f32 = 0.0;

pub const SURFACE_Y: i32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Block {
    Air = 0,
    Stone = 1,
    Dirt = 2,
    GrassBlock = 3,
    Cobblestone = 4,
    OakPlanks = 5,
    Sand = 6,
    Gravel = 7,
    OakLog = 8,
    Glass = 9,
    WhiteWool = 10,
    Bricks = 11,
    Obsidian = 12,
    Glowstone = 13,
    Granite = 14,
    PolishedGranite = 15,
    Diorite = 16,
    PolishedDiorite = 17,
    Andesite = 18,
    PolishedAndesite = 19,
    CoarseDirt = 20,
    SprucePlanks = 21,
    BirchPlanks = 22,
    JunglePlanks = 23,
    AcaciaPlanks = 24,
    CherryPlanks = 25,
    DarkOakPlanks = 26,
    PaleOakPlanks = 27,
    MangrovePlanks = 28,
    BambooPlanks = 29,
    BambooMosaic = 30,
    RedSand = 31,
    GoldOre = 32,
    DeepslateGoldOre = 33,
    IronOre = 34,
    DeepslateIronOre = 35,
    CoalOre = 36,
    DeepslateCoalOre = 37,
    NetherGoldOre = 38,
    Sponge = 39,
    WetSponge = 40,
    LapisOre = 41,
    DeepslateLapisOre = 42,
    LapisBlock = 43,
    Sandstone = 44,
    ChiseledSandstone = 45,
    CutSandstone = 46,
    OrangeWool = 47,
    MagentaWool = 48,
    LightBlueWool = 49,
    YellowWool = 50,
    LimeWool = 51,
    PinkWool = 52,
    GrayWool = 53,
    LightGrayWool = 54,
    CyanWool = 55,
    PurpleWool = 56,
    BlueWool = 57,
    BrownWool = 58,
    GreenWool = 59,
    RedWool = 60,
    BlackWool = 61,
    GoldBlock = 62,
    IronBlock = 63,
    Bookshelf = 64,
    MossyCobblestone = 65,
    DiamondOre = 66,
    DeepslateDiamondOre = 67,
    DiamondBlock = 68,
    CraftingTable = 69,
    Clay = 70,
    Netherrack = 71,
    SoulSand = 72,
    SoulSoil = 73,
    WhiteStainedGlass = 74,
    OrangeStainedGlass = 75,
    MagentaStainedGlass = 76,
    LightBlueStainedGlass = 77,
    YellowStainedGlass = 78,
    LimeStainedGlass = 79,
    PinkStainedGlass = 80,
    GrayStainedGlass = 81,
    LightGrayStainedGlass = 82,
    CyanStainedGlass = 83,
    PurpleStainedGlass = 84,
    BlueStainedGlass = 85,
    BrownStainedGlass = 86,
    GreenStainedGlass = 87,
    RedStainedGlass = 88,
    BlackStainedGlass = 89,
    StoneBricks = 90,
    MossyStoneBricks = 91,
    CrackedStoneBricks = 92,
    ChiseledStoneBricks = 93,
    PackedMud = 94,
    MudBricks = 95,
    Pumpkin = 96,
    Melon = 97,
    ResinBlock = 98,
    ResinBricks = 99,
    ChiseledResinBricks = 100,
    NetherBricks = 101,
    EnchantingTable = 102,
    EndStone = 103,
    EmeraldOre = 104,
    DeepslateEmeraldOre = 105,
    EmeraldBlock = 106,
    RedstoneBlock = 107,
    NetherQuartzOre = 108,
    QuartzBlock = 109,
    ChiseledQuartzBlock = 110,
    WhiteTerracotta = 111,
    OrangeTerracotta = 112,
    MagentaTerracotta = 113,
    LightBlueTerracotta = 114,
    YellowTerracotta = 115,
    LimeTerracotta = 116,
    PinkTerracotta = 117,
    GrayTerracotta = 118,
    LightGrayTerracotta = 119,
    CyanTerracotta = 120,
    PurpleTerracotta = 121,
    BlueTerracotta = 122,
    BrownTerracotta = 123,
    GreenTerracotta = 124,
    RedTerracotta = 125,
    BlackTerracotta = 126,
    Prismarine = 127,
    PrismarineBricks = 128,
    DarkPrismarine = 129,
    SeaLantern = 130,
    Terracotta = 131,
    CoalBlock = 132,
    PackedIce = 133,
    RedSandstone = 134,
    ChiseledRedSandstone = 135,
    CutRedSandstone = 136,
    SmoothStone = 137,
    SmoothSandstone = 138,
    SmoothQuartz = 139,
    SmoothRedSandstone = 140,
    PurpurBlock = 141,
    EndStoneBricks = 142,
    NetherWartBlock = 143,
    RedNetherBricks = 144,
    WhiteConcrete = 145,
    OrangeConcrete = 146,
    MagentaConcrete = 147,
    LightBlueConcrete = 148,
    YellowConcrete = 149,
    LimeConcrete = 150,
    PinkConcrete = 151,
    GrayConcrete = 152,
    LightGrayConcrete = 153,
    CyanConcrete = 154,
    PurpleConcrete = 155,
    BlueConcrete = 156,
    BrownConcrete = 157,
    GreenConcrete = 158,
    RedConcrete = 159,
    BlackConcrete = 160,
    DriedKelpBlock = 161,
    DeadTubeCoralBlock = 162,
    DeadBrainCoralBlock = 163,
    DeadBubbleCoralBlock = 164,
    DeadFireCoralBlock = 165,
    DeadHornCoralBlock = 166,
    BlueIce = 167,
    CartographyTable = 168,
    FletchingTable = 169,
    SmithingTable = 170,
    WarpedNylium = 171,
    WarpedWartBlock = 172,
    CrimsonNylium = 173,
    Shroomlight = 174,
    CrimsonPlanks = 175,
    WarpedPlanks = 176,
    HoneycombBlock = 177,
    NetheriteBlock = 178,
    AncientDebris = 179,
    CryingObsidian = 180,
    Lodestone = 181,
    Blackstone = 182,
    PolishedBlackstone = 183,
    PolishedBlackstoneBricks = 184,
    CrackedPolishedBlackstoneBricks = 185,
    ChiseledPolishedBlackstone = 186,
    GildedBlackstone = 187,
    ChiseledNetherBricks = 188,
    CrackedNetherBricks = 189,
    QuartzBricks = 190,
    AmethystBlock = 191,
    Tuff = 192,
    PolishedTuff = 193,
    ChiseledTuff = 194,
    TuffBricks = 195,
    ChiseledTuffBricks = 196,
    Calcite = 197,
    TintedGlass = 198,
    CopperBlock = 199,
    ExposedCopper = 200,
    WeatheredCopper = 201,
    OxidizedCopper = 202,
    CopperOre = 203,
    DeepslateCopperOre = 204,
    OxidizedCutCopper = 205,
    WeatheredCutCopper = 206,
    ExposedCutCopper = 207,
    CutCopper = 208,
    OxidizedChiseledCopper = 209,
    WeatheredChiseledCopper = 210,
    ExposedChiseledCopper = 211,
    ChiseledCopper = 212,
    WaxedOxidizedChiseledCopper = 213,
    WaxedWeatheredChiseledCopper = 214,
    WaxedExposedChiseledCopper = 215,
    WaxedChiseledCopper = 216,
    WaxedCopperBlock = 217,
    WaxedWeatheredCopper = 218,
    WaxedExposedCopper = 219,
    WaxedOxidizedCopper = 220,
    WaxedOxidizedCutCopper = 221,
    WaxedWeatheredCutCopper = 222,
    WaxedExposedCutCopper = 223,
    WaxedCutCopper = 224,
    DripstoneBlock = 225,
    MossBlock = 226,
    RootedDirt = 227,
    Mud = 228,
    CobbledDeepslate = 229,
    PolishedDeepslate = 230,
    DeepslateTiles = 231,
    DeepslateBricks = 232,
    ChiseledDeepslate = 233,
    CrackedDeepslateBricks = 234,
    CrackedDeepslateTiles = 235,
    SmoothBasalt = 236,
    RawIronBlock = 237,
    RawCopperBlock = 238,
    RawGoldBlock = 239,
    PaleMossBlock = 240,
}

impl Block {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Block::Air => "minecraft:air",
            Block::Stone => "minecraft:stone",
            Block::Dirt => "minecraft:dirt",
            Block::GrassBlock => "minecraft:grass_block",
            Block::Cobblestone => "minecraft:cobblestone",
            Block::OakPlanks => "minecraft:oak_planks",
            Block::Sand => "minecraft:sand",
            Block::Gravel => "minecraft:gravel",
            Block::OakLog => "minecraft:oak_log",
            Block::Glass => "minecraft:glass",
            Block::WhiteWool => "minecraft:white_wool",
            Block::Bricks => "minecraft:bricks",
            Block::Obsidian => "minecraft:obsidian",
            Block::Glowstone => "minecraft:glowstone",
            Block::Granite => "minecraft:granite",
            Block::PolishedGranite => "minecraft:polished_granite",
            Block::Diorite => "minecraft:diorite",
            Block::PolishedDiorite => "minecraft:polished_diorite",
            Block::Andesite => "minecraft:andesite",
            Block::PolishedAndesite => "minecraft:polished_andesite",
            Block::CoarseDirt => "minecraft:coarse_dirt",
            Block::SprucePlanks => "minecraft:spruce_planks",
            Block::BirchPlanks => "minecraft:birch_planks",
            Block::JunglePlanks => "minecraft:jungle_planks",
            Block::AcaciaPlanks => "minecraft:acacia_planks",
            Block::CherryPlanks => "minecraft:cherry_planks",
            Block::DarkOakPlanks => "minecraft:dark_oak_planks",
            Block::PaleOakPlanks => "minecraft:pale_oak_planks",
            Block::MangrovePlanks => "minecraft:mangrove_planks",
            Block::BambooPlanks => "minecraft:bamboo_planks",
            Block::BambooMosaic => "minecraft:bamboo_mosaic",
            Block::RedSand => "minecraft:red_sand",
            Block::GoldOre => "minecraft:gold_ore",
            Block::DeepslateGoldOre => "minecraft:deepslate_gold_ore",
            Block::IronOre => "minecraft:iron_ore",
            Block::DeepslateIronOre => "minecraft:deepslate_iron_ore",
            Block::CoalOre => "minecraft:coal_ore",
            Block::DeepslateCoalOre => "minecraft:deepslate_coal_ore",
            Block::NetherGoldOre => "minecraft:nether_gold_ore",
            Block::Sponge => "minecraft:sponge",
            Block::WetSponge => "minecraft:wet_sponge",
            Block::LapisOre => "minecraft:lapis_ore",
            Block::DeepslateLapisOre => "minecraft:deepslate_lapis_ore",
            Block::LapisBlock => "minecraft:lapis_block",
            Block::Sandstone => "minecraft:sandstone",
            Block::ChiseledSandstone => "minecraft:chiseled_sandstone",
            Block::CutSandstone => "minecraft:cut_sandstone",
            Block::OrangeWool => "minecraft:orange_wool",
            Block::MagentaWool => "minecraft:magenta_wool",
            Block::LightBlueWool => "minecraft:light_blue_wool",
            Block::YellowWool => "minecraft:yellow_wool",
            Block::LimeWool => "minecraft:lime_wool",
            Block::PinkWool => "minecraft:pink_wool",
            Block::GrayWool => "minecraft:gray_wool",
            Block::LightGrayWool => "minecraft:light_gray_wool",
            Block::CyanWool => "minecraft:cyan_wool",
            Block::PurpleWool => "minecraft:purple_wool",
            Block::BlueWool => "minecraft:blue_wool",
            Block::BrownWool => "minecraft:brown_wool",
            Block::GreenWool => "minecraft:green_wool",
            Block::RedWool => "minecraft:red_wool",
            Block::BlackWool => "minecraft:black_wool",
            Block::GoldBlock => "minecraft:gold_block",
            Block::IronBlock => "minecraft:iron_block",
            Block::Bookshelf => "minecraft:bookshelf",
            Block::MossyCobblestone => "minecraft:mossy_cobblestone",
            Block::DiamondOre => "minecraft:diamond_ore",
            Block::DeepslateDiamondOre => "minecraft:deepslate_diamond_ore",
            Block::DiamondBlock => "minecraft:diamond_block",
            Block::CraftingTable => "minecraft:crafting_table",
            Block::Clay => "minecraft:clay",
            Block::Netherrack => "minecraft:netherrack",
            Block::SoulSand => "minecraft:soul_sand",
            Block::SoulSoil => "minecraft:soul_soil",
            Block::WhiteStainedGlass => "minecraft:white_stained_glass",
            Block::OrangeStainedGlass => "minecraft:orange_stained_glass",
            Block::MagentaStainedGlass => "minecraft:magenta_stained_glass",
            Block::LightBlueStainedGlass => "minecraft:light_blue_stained_glass",
            Block::YellowStainedGlass => "minecraft:yellow_stained_glass",
            Block::LimeStainedGlass => "minecraft:lime_stained_glass",
            Block::PinkStainedGlass => "minecraft:pink_stained_glass",
            Block::GrayStainedGlass => "minecraft:gray_stained_glass",
            Block::LightGrayStainedGlass => "minecraft:light_gray_stained_glass",
            Block::CyanStainedGlass => "minecraft:cyan_stained_glass",
            Block::PurpleStainedGlass => "minecraft:purple_stained_glass",
            Block::BlueStainedGlass => "minecraft:blue_stained_glass",
            Block::BrownStainedGlass => "minecraft:brown_stained_glass",
            Block::GreenStainedGlass => "minecraft:green_stained_glass",
            Block::RedStainedGlass => "minecraft:red_stained_glass",
            Block::BlackStainedGlass => "minecraft:black_stained_glass",
            Block::StoneBricks => "minecraft:stone_bricks",
            Block::MossyStoneBricks => "minecraft:mossy_stone_bricks",
            Block::CrackedStoneBricks => "minecraft:cracked_stone_bricks",
            Block::ChiseledStoneBricks => "minecraft:chiseled_stone_bricks",
            Block::PackedMud => "minecraft:packed_mud",
            Block::MudBricks => "minecraft:mud_bricks",
            Block::Pumpkin => "minecraft:pumpkin",
            Block::Melon => "minecraft:melon",
            Block::ResinBlock => "minecraft:resin_block",
            Block::ResinBricks => "minecraft:resin_bricks",
            Block::ChiseledResinBricks => "minecraft:chiseled_resin_bricks",
            Block::NetherBricks => "minecraft:nether_bricks",
            Block::EnchantingTable => "minecraft:enchanting_table",
            Block::EndStone => "minecraft:end_stone",
            Block::EmeraldOre => "minecraft:emerald_ore",
            Block::DeepslateEmeraldOre => "minecraft:deepslate_emerald_ore",
            Block::EmeraldBlock => "minecraft:emerald_block",
            Block::RedstoneBlock => "minecraft:redstone_block",
            Block::NetherQuartzOre => "minecraft:nether_quartz_ore",
            Block::QuartzBlock => "minecraft:quartz_block",
            Block::ChiseledQuartzBlock => "minecraft:chiseled_quartz_block",
            Block::WhiteTerracotta => "minecraft:white_terracotta",
            Block::OrangeTerracotta => "minecraft:orange_terracotta",
            Block::MagentaTerracotta => "minecraft:magenta_terracotta",
            Block::LightBlueTerracotta => "minecraft:light_blue_terracotta",
            Block::YellowTerracotta => "minecraft:yellow_terracotta",
            Block::LimeTerracotta => "minecraft:lime_terracotta",
            Block::PinkTerracotta => "minecraft:pink_terracotta",
            Block::GrayTerracotta => "minecraft:gray_terracotta",
            Block::LightGrayTerracotta => "minecraft:light_gray_terracotta",
            Block::CyanTerracotta => "minecraft:cyan_terracotta",
            Block::PurpleTerracotta => "minecraft:purple_terracotta",
            Block::BlueTerracotta => "minecraft:blue_terracotta",
            Block::BrownTerracotta => "minecraft:brown_terracotta",
            Block::GreenTerracotta => "minecraft:green_terracotta",
            Block::RedTerracotta => "minecraft:red_terracotta",
            Block::BlackTerracotta => "minecraft:black_terracotta",
            Block::Prismarine => "minecraft:prismarine",
            Block::PrismarineBricks => "minecraft:prismarine_bricks",
            Block::DarkPrismarine => "minecraft:dark_prismarine",
            Block::SeaLantern => "minecraft:sea_lantern",
            Block::Terracotta => "minecraft:terracotta",
            Block::CoalBlock => "minecraft:coal_block",
            Block::PackedIce => "minecraft:packed_ice",
            Block::RedSandstone => "minecraft:red_sandstone",
            Block::ChiseledRedSandstone => "minecraft:chiseled_red_sandstone",
            Block::CutRedSandstone => "minecraft:cut_red_sandstone",
            Block::SmoothStone => "minecraft:smooth_stone",
            Block::SmoothSandstone => "minecraft:smooth_sandstone",
            Block::SmoothQuartz => "minecraft:smooth_quartz",
            Block::SmoothRedSandstone => "minecraft:smooth_red_sandstone",
            Block::PurpurBlock => "minecraft:purpur_block",
            Block::EndStoneBricks => "minecraft:end_stone_bricks",
            Block::NetherWartBlock => "minecraft:nether_wart_block",
            Block::RedNetherBricks => "minecraft:red_nether_bricks",
            Block::WhiteConcrete => "minecraft:white_concrete",
            Block::OrangeConcrete => "minecraft:orange_concrete",
            Block::MagentaConcrete => "minecraft:magenta_concrete",
            Block::LightBlueConcrete => "minecraft:light_blue_concrete",
            Block::YellowConcrete => "minecraft:yellow_concrete",
            Block::LimeConcrete => "minecraft:lime_concrete",
            Block::PinkConcrete => "minecraft:pink_concrete",
            Block::GrayConcrete => "minecraft:gray_concrete",
            Block::LightGrayConcrete => "minecraft:light_gray_concrete",
            Block::CyanConcrete => "minecraft:cyan_concrete",
            Block::PurpleConcrete => "minecraft:purple_concrete",
            Block::BlueConcrete => "minecraft:blue_concrete",
            Block::BrownConcrete => "minecraft:brown_concrete",
            Block::GreenConcrete => "minecraft:green_concrete",
            Block::RedConcrete => "minecraft:red_concrete",
            Block::BlackConcrete => "minecraft:black_concrete",
            Block::DriedKelpBlock => "minecraft:dried_kelp_block",
            Block::DeadTubeCoralBlock => "minecraft:dead_tube_coral_block",
            Block::DeadBrainCoralBlock => "minecraft:dead_brain_coral_block",
            Block::DeadBubbleCoralBlock => "minecraft:dead_bubble_coral_block",
            Block::DeadFireCoralBlock => "minecraft:dead_fire_coral_block",
            Block::DeadHornCoralBlock => "minecraft:dead_horn_coral_block",
            Block::BlueIce => "minecraft:blue_ice",
            Block::CartographyTable => "minecraft:cartography_table",
            Block::FletchingTable => "minecraft:fletching_table",
            Block::SmithingTable => "minecraft:smithing_table",
            Block::WarpedNylium => "minecraft:warped_nylium",
            Block::WarpedWartBlock => "minecraft:warped_wart_block",
            Block::CrimsonNylium => "minecraft:crimson_nylium",
            Block::Shroomlight => "minecraft:shroomlight",
            Block::CrimsonPlanks => "minecraft:crimson_planks",
            Block::WarpedPlanks => "minecraft:warped_planks",
            Block::HoneycombBlock => "minecraft:honeycomb_block",
            Block::NetheriteBlock => "minecraft:netherite_block",
            Block::AncientDebris => "minecraft:ancient_debris",
            Block::CryingObsidian => "minecraft:crying_obsidian",
            Block::Lodestone => "minecraft:lodestone",
            Block::Blackstone => "minecraft:blackstone",
            Block::PolishedBlackstone => "minecraft:polished_blackstone",
            Block::PolishedBlackstoneBricks => "minecraft:polished_blackstone_bricks",
            Block::CrackedPolishedBlackstoneBricks => "minecraft:cracked_polished_blackstone_bricks",
            Block::ChiseledPolishedBlackstone => "minecraft:chiseled_polished_blackstone",
            Block::GildedBlackstone => "minecraft:gilded_blackstone",
            Block::ChiseledNetherBricks => "minecraft:chiseled_nether_bricks",
            Block::CrackedNetherBricks => "minecraft:cracked_nether_bricks",
            Block::QuartzBricks => "minecraft:quartz_bricks",
            Block::AmethystBlock => "minecraft:amethyst_block",
            Block::Tuff => "minecraft:tuff",
            Block::PolishedTuff => "minecraft:polished_tuff",
            Block::ChiseledTuff => "minecraft:chiseled_tuff",
            Block::TuffBricks => "minecraft:tuff_bricks",
            Block::ChiseledTuffBricks => "minecraft:chiseled_tuff_bricks",
            Block::Calcite => "minecraft:calcite",
            Block::TintedGlass => "minecraft:tinted_glass",
            Block::CopperBlock => "minecraft:copper_block",
            Block::ExposedCopper => "minecraft:exposed_copper",
            Block::WeatheredCopper => "minecraft:weathered_copper",
            Block::OxidizedCopper => "minecraft:oxidized_copper",
            Block::CopperOre => "minecraft:copper_ore",
            Block::DeepslateCopperOre => "minecraft:deepslate_copper_ore",
            Block::OxidizedCutCopper => "minecraft:oxidized_cut_copper",
            Block::WeatheredCutCopper => "minecraft:weathered_cut_copper",
            Block::ExposedCutCopper => "minecraft:exposed_cut_copper",
            Block::CutCopper => "minecraft:cut_copper",
            Block::OxidizedChiseledCopper => "minecraft:oxidized_chiseled_copper",
            Block::WeatheredChiseledCopper => "minecraft:weathered_chiseled_copper",
            Block::ExposedChiseledCopper => "minecraft:exposed_chiseled_copper",
            Block::ChiseledCopper => "minecraft:chiseled_copper",
            Block::WaxedOxidizedChiseledCopper => "minecraft:waxed_oxidized_chiseled_copper",
            Block::WaxedWeatheredChiseledCopper => "minecraft:waxed_weathered_chiseled_copper",
            Block::WaxedExposedChiseledCopper => "minecraft:waxed_exposed_chiseled_copper",
            Block::WaxedChiseledCopper => "minecraft:waxed_chiseled_copper",
            Block::WaxedCopperBlock => "minecraft:waxed_copper_block",
            Block::WaxedWeatheredCopper => "minecraft:waxed_weathered_copper",
            Block::WaxedExposedCopper => "minecraft:waxed_exposed_copper",
            Block::WaxedOxidizedCopper => "minecraft:waxed_oxidized_copper",
            Block::WaxedOxidizedCutCopper => "minecraft:waxed_oxidized_cut_copper",
            Block::WaxedWeatheredCutCopper => "minecraft:waxed_weathered_cut_copper",
            Block::WaxedExposedCutCopper => "minecraft:waxed_exposed_cut_copper",
            Block::WaxedCutCopper => "minecraft:waxed_cut_copper",
            Block::DripstoneBlock => "minecraft:dripstone_block",
            Block::MossBlock => "minecraft:moss_block",
            Block::RootedDirt => "minecraft:rooted_dirt",
            Block::Mud => "minecraft:mud",
            Block::CobbledDeepslate => "minecraft:cobbled_deepslate",
            Block::PolishedDeepslate => "minecraft:polished_deepslate",
            Block::DeepslateTiles => "minecraft:deepslate_tiles",
            Block::DeepslateBricks => "minecraft:deepslate_bricks",
            Block::ChiseledDeepslate => "minecraft:chiseled_deepslate",
            Block::CrackedDeepslateBricks => "minecraft:cracked_deepslate_bricks",
            Block::CrackedDeepslateTiles => "minecraft:cracked_deepslate_tiles",
            Block::SmoothBasalt => "minecraft:smooth_basalt",
            Block::RawIronBlock => "minecraft:raw_iron_block",
            Block::RawCopperBlock => "minecraft:raw_copper_block",
            Block::RawGoldBlock => "minecraft:raw_gold_block",
            Block::PaleMossBlock => "minecraft:pale_moss_block",
        }
    }

    #[must_use]
    pub fn from_id(id: u16) -> Option<Self> {
        match id {
            0 => Some(Block::Air),
            1 => Some(Block::Stone),
            2 => Some(Block::Dirt),
            3 => Some(Block::GrassBlock),
            4 => Some(Block::Cobblestone),
            5 => Some(Block::OakPlanks),
            6 => Some(Block::Sand),
            7 => Some(Block::Gravel),
            8 => Some(Block::OakLog),
            9 => Some(Block::Glass),
            10 => Some(Block::WhiteWool),
            11 => Some(Block::Bricks),
            12 => Some(Block::Obsidian),
            13 => Some(Block::Glowstone),
            14 => Some(Block::Granite),
            15 => Some(Block::PolishedGranite),
            16 => Some(Block::Diorite),
            17 => Some(Block::PolishedDiorite),
            18 => Some(Block::Andesite),
            19 => Some(Block::PolishedAndesite),
            20 => Some(Block::CoarseDirt),
            21 => Some(Block::SprucePlanks),
            22 => Some(Block::BirchPlanks),
            23 => Some(Block::JunglePlanks),
            24 => Some(Block::AcaciaPlanks),
            25 => Some(Block::CherryPlanks),
            26 => Some(Block::DarkOakPlanks),
            27 => Some(Block::PaleOakPlanks),
            28 => Some(Block::MangrovePlanks),
            29 => Some(Block::BambooPlanks),
            30 => Some(Block::BambooMosaic),
            31 => Some(Block::RedSand),
            32 => Some(Block::GoldOre),
            33 => Some(Block::DeepslateGoldOre),
            34 => Some(Block::IronOre),
            35 => Some(Block::DeepslateIronOre),
            36 => Some(Block::CoalOre),
            37 => Some(Block::DeepslateCoalOre),
            38 => Some(Block::NetherGoldOre),
            39 => Some(Block::Sponge),
            40 => Some(Block::WetSponge),
            41 => Some(Block::LapisOre),
            42 => Some(Block::DeepslateLapisOre),
            43 => Some(Block::LapisBlock),
            44 => Some(Block::Sandstone),
            45 => Some(Block::ChiseledSandstone),
            46 => Some(Block::CutSandstone),
            47 => Some(Block::OrangeWool),
            48 => Some(Block::MagentaWool),
            49 => Some(Block::LightBlueWool),
            50 => Some(Block::YellowWool),
            51 => Some(Block::LimeWool),
            52 => Some(Block::PinkWool),
            53 => Some(Block::GrayWool),
            54 => Some(Block::LightGrayWool),
            55 => Some(Block::CyanWool),
            56 => Some(Block::PurpleWool),
            57 => Some(Block::BlueWool),
            58 => Some(Block::BrownWool),
            59 => Some(Block::GreenWool),
            60 => Some(Block::RedWool),
            61 => Some(Block::BlackWool),
            62 => Some(Block::GoldBlock),
            63 => Some(Block::IronBlock),
            64 => Some(Block::Bookshelf),
            65 => Some(Block::MossyCobblestone),
            66 => Some(Block::DiamondOre),
            67 => Some(Block::DeepslateDiamondOre),
            68 => Some(Block::DiamondBlock),
            69 => Some(Block::CraftingTable),
            70 => Some(Block::Clay),
            71 => Some(Block::Netherrack),
            72 => Some(Block::SoulSand),
            73 => Some(Block::SoulSoil),
            74 => Some(Block::WhiteStainedGlass),
            75 => Some(Block::OrangeStainedGlass),
            76 => Some(Block::MagentaStainedGlass),
            77 => Some(Block::LightBlueStainedGlass),
            78 => Some(Block::YellowStainedGlass),
            79 => Some(Block::LimeStainedGlass),
            80 => Some(Block::PinkStainedGlass),
            81 => Some(Block::GrayStainedGlass),
            82 => Some(Block::LightGrayStainedGlass),
            83 => Some(Block::CyanStainedGlass),
            84 => Some(Block::PurpleStainedGlass),
            85 => Some(Block::BlueStainedGlass),
            86 => Some(Block::BrownStainedGlass),
            87 => Some(Block::GreenStainedGlass),
            88 => Some(Block::RedStainedGlass),
            89 => Some(Block::BlackStainedGlass),
            90 => Some(Block::StoneBricks),
            91 => Some(Block::MossyStoneBricks),
            92 => Some(Block::CrackedStoneBricks),
            93 => Some(Block::ChiseledStoneBricks),
            94 => Some(Block::PackedMud),
            95 => Some(Block::MudBricks),
            96 => Some(Block::Pumpkin),
            97 => Some(Block::Melon),
            98 => Some(Block::ResinBlock),
            99 => Some(Block::ResinBricks),
            100 => Some(Block::ChiseledResinBricks),
            101 => Some(Block::NetherBricks),
            102 => Some(Block::EnchantingTable),
            103 => Some(Block::EndStone),
            104 => Some(Block::EmeraldOre),
            105 => Some(Block::DeepslateEmeraldOre),
            106 => Some(Block::EmeraldBlock),
            107 => Some(Block::RedstoneBlock),
            108 => Some(Block::NetherQuartzOre),
            109 => Some(Block::QuartzBlock),
            110 => Some(Block::ChiseledQuartzBlock),
            111 => Some(Block::WhiteTerracotta),
            112 => Some(Block::OrangeTerracotta),
            113 => Some(Block::MagentaTerracotta),
            114 => Some(Block::LightBlueTerracotta),
            115 => Some(Block::YellowTerracotta),
            116 => Some(Block::LimeTerracotta),
            117 => Some(Block::PinkTerracotta),
            118 => Some(Block::GrayTerracotta),
            119 => Some(Block::LightGrayTerracotta),
            120 => Some(Block::CyanTerracotta),
            121 => Some(Block::PurpleTerracotta),
            122 => Some(Block::BlueTerracotta),
            123 => Some(Block::BrownTerracotta),
            124 => Some(Block::GreenTerracotta),
            125 => Some(Block::RedTerracotta),
            126 => Some(Block::BlackTerracotta),
            127 => Some(Block::Prismarine),
            128 => Some(Block::PrismarineBricks),
            129 => Some(Block::DarkPrismarine),
            130 => Some(Block::SeaLantern),
            131 => Some(Block::Terracotta),
            132 => Some(Block::CoalBlock),
            133 => Some(Block::PackedIce),
            134 => Some(Block::RedSandstone),
            135 => Some(Block::ChiseledRedSandstone),
            136 => Some(Block::CutRedSandstone),
            137 => Some(Block::SmoothStone),
            138 => Some(Block::SmoothSandstone),
            139 => Some(Block::SmoothQuartz),
            140 => Some(Block::SmoothRedSandstone),
            141 => Some(Block::PurpurBlock),
            142 => Some(Block::EndStoneBricks),
            143 => Some(Block::NetherWartBlock),
            144 => Some(Block::RedNetherBricks),
            145 => Some(Block::WhiteConcrete),
            146 => Some(Block::OrangeConcrete),
            147 => Some(Block::MagentaConcrete),
            148 => Some(Block::LightBlueConcrete),
            149 => Some(Block::YellowConcrete),
            150 => Some(Block::LimeConcrete),
            151 => Some(Block::PinkConcrete),
            152 => Some(Block::GrayConcrete),
            153 => Some(Block::LightGrayConcrete),
            154 => Some(Block::CyanConcrete),
            155 => Some(Block::PurpleConcrete),
            156 => Some(Block::BlueConcrete),
            157 => Some(Block::BrownConcrete),
            158 => Some(Block::GreenConcrete),
            159 => Some(Block::RedConcrete),
            160 => Some(Block::BlackConcrete),
            161 => Some(Block::DriedKelpBlock),
            162 => Some(Block::DeadTubeCoralBlock),
            163 => Some(Block::DeadBrainCoralBlock),
            164 => Some(Block::DeadBubbleCoralBlock),
            165 => Some(Block::DeadFireCoralBlock),
            166 => Some(Block::DeadHornCoralBlock),
            167 => Some(Block::BlueIce),
            168 => Some(Block::CartographyTable),
            169 => Some(Block::FletchingTable),
            170 => Some(Block::SmithingTable),
            171 => Some(Block::WarpedNylium),
            172 => Some(Block::WarpedWartBlock),
            173 => Some(Block::CrimsonNylium),
            174 => Some(Block::Shroomlight),
            175 => Some(Block::CrimsonPlanks),
            176 => Some(Block::WarpedPlanks),
            177 => Some(Block::HoneycombBlock),
            178 => Some(Block::NetheriteBlock),
            179 => Some(Block::AncientDebris),
            180 => Some(Block::CryingObsidian),
            181 => Some(Block::Lodestone),
            182 => Some(Block::Blackstone),
            183 => Some(Block::PolishedBlackstone),
            184 => Some(Block::PolishedBlackstoneBricks),
            185 => Some(Block::CrackedPolishedBlackstoneBricks),
            186 => Some(Block::ChiseledPolishedBlackstone),
            187 => Some(Block::GildedBlackstone),
            188 => Some(Block::ChiseledNetherBricks),
            189 => Some(Block::CrackedNetherBricks),
            190 => Some(Block::QuartzBricks),
            191 => Some(Block::AmethystBlock),
            192 => Some(Block::Tuff),
            193 => Some(Block::PolishedTuff),
            194 => Some(Block::ChiseledTuff),
            195 => Some(Block::TuffBricks),
            196 => Some(Block::ChiseledTuffBricks),
            197 => Some(Block::Calcite),
            198 => Some(Block::TintedGlass),
            199 => Some(Block::CopperBlock),
            200 => Some(Block::ExposedCopper),
            201 => Some(Block::WeatheredCopper),
            202 => Some(Block::OxidizedCopper),
            203 => Some(Block::CopperOre),
            204 => Some(Block::DeepslateCopperOre),
            205 => Some(Block::OxidizedCutCopper),
            206 => Some(Block::WeatheredCutCopper),
            207 => Some(Block::ExposedCutCopper),
            208 => Some(Block::CutCopper),
            209 => Some(Block::OxidizedChiseledCopper),
            210 => Some(Block::WeatheredChiseledCopper),
            211 => Some(Block::ExposedChiseledCopper),
            212 => Some(Block::ChiseledCopper),
            213 => Some(Block::WaxedOxidizedChiseledCopper),
            214 => Some(Block::WaxedWeatheredChiseledCopper),
            215 => Some(Block::WaxedExposedChiseledCopper),
            216 => Some(Block::WaxedChiseledCopper),
            217 => Some(Block::WaxedCopperBlock),
            218 => Some(Block::WaxedWeatheredCopper),
            219 => Some(Block::WaxedExposedCopper),
            220 => Some(Block::WaxedOxidizedCopper),
            221 => Some(Block::WaxedOxidizedCutCopper),
            222 => Some(Block::WaxedWeatheredCutCopper),
            223 => Some(Block::WaxedExposedCutCopper),
            224 => Some(Block::WaxedCutCopper),
            225 => Some(Block::DripstoneBlock),
            226 => Some(Block::MossBlock),
            227 => Some(Block::RootedDirt),
            228 => Some(Block::Mud),
            229 => Some(Block::CobbledDeepslate),
            230 => Some(Block::PolishedDeepslate),
            231 => Some(Block::DeepslateTiles),
            232 => Some(Block::DeepslateBricks),
            233 => Some(Block::ChiseledDeepslate),
            234 => Some(Block::CrackedDeepslateBricks),
            235 => Some(Block::CrackedDeepslateTiles),
            236 => Some(Block::SmoothBasalt),
            237 => Some(Block::RawIronBlock),
            238 => Some(Block::RawCopperBlock),
            239 => Some(Block::RawGoldBlock),
            240 => Some(Block::PaleMossBlock),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_solid(self) -> bool {
        !matches!(self, Block::Air)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[must_use]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub fn chunk_pos(self) -> ChunkPos {
        ChunkPos::from_world(self.x, self.z)
    }

    #[must_use]
    pub fn local_x(self) -> u8 {
        self.x.rem_euclid(CHUNK_WIDTH) as u8
    }

    #[must_use]
    pub fn local_z(self) -> u8 {
        self.z.rem_euclid(CHUNK_WIDTH) as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    #[must_use]
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    #[must_use]
    pub fn from_world(world_x: i32, world_z: i32) -> Self {
        Self {
            x: world_x.div_euclid(CHUNK_WIDTH),
            z: world_z.div_euclid(CHUNK_WIDTH),
        }
    }

    #[must_use]
    pub fn min_block(self) -> (i32, i32) {
        (self.x * CHUNK_WIDTH, self.z * CHUNK_WIDTH)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spawn {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub dimension: &'static str,
}

impl Spawn {
    #[must_use]
    pub fn default_spawn() -> Self {
        Self {
            x: SPAWN_X,
            y: SPAWN_Y,
            z: SPAWN_Z,
            yaw: SPAWN_YAW,
            pitch: SPAWN_PITCH,
            dimension: DEFAULT_DIMENSION_ID,
        }
    }
}

pub struct ChunkSection {
    section_y: i32,
    blocks: Box<[u16; 4096]>,
    changed: Vec<(u8, u8, u8)>,
}

impl ChunkSection {
    #[must_use]
    pub fn new(section_y: i32) -> Self {
        Self {
            section_y,
            blocks: Box::new([Block::Air as u16; 4096]),
            changed: Vec::new(),
        }
    }

    #[must_use]
    pub fn section_y(&self) -> i32 {
        self.section_y
    }

    fn index(x: u8, y: u8, z: u8) -> usize {
        (y as usize * 16 + z as usize) * 16 + x as usize
    }

    #[must_use]
    pub fn get_local(&self, x: u8, y: u8, z: u8) -> Block {
        debug_assert!(x < 16 && y < 16 && z < 16);
        Block::from_id(self.blocks[Self::index(x, y, z)]).unwrap_or(Block::Air)
    }

    pub fn set_local(&mut self, x: u8, y: u8, z: u8, block: Block) -> Block {
        debug_assert!(x < 16 && y < 16 && z < 16);
        let slot = &mut self.blocks[Self::index(x, y, z)];
        let prev = Block::from_id(*slot).unwrap_or(Block::Air);
        if prev != block {
            *slot = block as u16;
            if !self.changed.contains(&(x, y, z)) {
                self.changed.push((x, y, z));
            }
        }
        prev
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        !self.changed.is_empty()
    }

    pub fn take_changed(&mut self) -> Vec<(u8, u8, u8)> {
        std::mem::take(&mut self.changed)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|&id| id == Block::Air as u16)
    }

    #[must_use]
    pub fn states(&self) -> &[u16; 4096] {
        &self.blocks
    }
}

pub struct Chunk {
    pos: ChunkPos,
    sections: Vec<ChunkSection>,
    dirty: bool,
}

impl Chunk {
    #[must_use]
    pub fn new(x: i32, z: i32) -> Self {
        let mut chunk = Self::new_bare(x, z);
        chunk.generate_flat();
        for section in &mut chunk.sections {
            section.take_changed();
        }
        chunk
    }

    /// Sections without terrain: decode target and unit-test helper.
    /// Starts clean, like freshly generated terrain.
    #[must_use]
    pub fn new_bare(x: i32, z: i32) -> Self {
        Self {
            pos: ChunkPos::new(x, z),
            sections: (0..SECTION_COUNT as i32).map(ChunkSection::new).collect(),
            dirty: false,
        }
    }

    /// True since the last save (or since an edit created it). Only dirty
    /// chunks ever hit the disk.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Serialized section states for [`WorldStore`].
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let sections: Vec<&[u16; SECTION_BLOCKS]> =
            self.sections.iter().map(|s| s.states()).collect();
        encode_sections(&sections)
    }

    /// Inverse of [`Chunk::encode`]; rejects corrupt input and wrong sizes.
    /// Comes back dirty so a freshly restored chunk gets saved again.
    #[must_use]
    pub fn decode(x: i32, z: i32, bytes: &[u8]) -> Option<Self> {
        let states = decode_sections(bytes, SECTION_COUNT)?;
        let mut chunk = Self::new_bare(x, z);
        for (section, filled) in chunk.sections.iter_mut().zip(states.iter()) {
            section.blocks.copy_from_slice(filled);
            section.take_changed();
        }
        chunk.dirty = true;
        Some(chunk)
    }

    pub fn section_mut(&mut self, index: usize) -> Option<&mut ChunkSection> {
        self.sections.get_mut(index)
    }

    #[must_use]
    pub fn pos(&self) -> ChunkPos {
        self.pos
    }

    fn section_for_y(&self, y: i32) -> Option<usize> {
        let rel = y - WORLD_MIN_Y;
        if !(0..WORLD_HEIGHT).contains(&rel) {
            return None;
        }
        Some((rel / SECTION_SIZE) as usize)
    }

    fn generate_flat(&mut self) {
        for y in WORLD_MIN_Y..=SURFACE_Y {
            let block = if y == SURFACE_Y {
                Block::GrassBlock
            } else if y >= SURFACE_Y - 3 {
                Block::Dirt
            } else {
                Block::Stone
            } as u16;
            let section = (y - WORLD_MIN_Y) as usize / SECTION_SIZE as usize;
            let local_y = (y - WORLD_MIN_Y) as u8 % SECTION_SIZE as u8;
            for lx in 0..16u8 {
                for lz in 0..16u8 {
                    let index = ChunkSection::index(lx, local_y, lz);
                    self.sections[section].blocks[index] = block;
                }
            }
        }
    }

    #[must_use]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        if ChunkPos::from_world(x, z) != self.pos {
            return None;
        }
        let section = self.section_for_y(y)?;
        Some(self.sections[section].get_local(
            x.rem_euclid(CHUNK_WIDTH) as u8,
            (y - WORLD_MIN_Y) as u8 % SECTION_SIZE as u8,
            z.rem_euclid(CHUNK_WIDTH) as u8,
        ))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) -> Option<Block> {
        if ChunkPos::from_world(x, z) != self.pos {
            return None;
        }
        let section = self.section_for_y(y)?;
        let prev = self.sections[section].set_local(
            x.rem_euclid(CHUNK_WIDTH) as u8,
            (y - WORLD_MIN_Y) as u8 % SECTION_SIZE as u8,
            z.rem_euclid(CHUNK_WIDTH) as u8,
            block,
        );
        if prev != block {
            self.dirty = true;
        }
        Some(prev)
    }

    #[must_use]
    pub fn section(&self, index: usize) -> Option<&ChunkSection> {
        self.sections.get(index)
    }

    #[must_use]
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }
}

#[derive(Default)]
pub struct ChunkManager {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl ChunkManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, pos: ChunkPos) -> &mut Chunk {
        self.chunks
            .entry(pos)
            .or_insert_with(|| Chunk::new(pos.x, pos.z))
    }

    #[must_use]
    pub fn get(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    pub fn get_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    pub fn unload(&mut self, pos: ChunkPos) -> bool {
        self.chunks.remove(&pos).is_some()
    }

    #[must_use]
    pub fn is_loaded(&self, pos: ChunkPos) -> bool {
        self.chunks.contains_key(&pos)
    }

    #[must_use]
    pub fn loaded_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn loaded_positions(&self) -> Vec<ChunkPos> {
        self.chunks.keys().copied().collect()
    }

    pub fn insert(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }
}

pub struct DroppedItem {
    pub entity: Entity,
    pub stack: ItemStack,
    pub age: u64,
}

pub const DROP_DESPAWN_AGE: u64 = 6000;
pub const DROP_PICKUP_DELAY: u64 = 10;

pub struct World {
    name: String,
    dimension: String,
    min_y: i32,
    height: i32,
    spawn: Spawn,
    chunks: ChunkManager,
    drops: Vec<DroppedItem>,
    store: Option<WorldStore>,
}

impl World {
    #[must_use]
    pub fn new(name: String, dimension: String) -> Self {
        Self {
            name,
            dimension,
            min_y: WORLD_MIN_Y,
            height: WORLD_HEIGHT,
            spawn: Spawn::default_spawn(),
            chunks: ChunkManager::new(),
            drops: Vec::new(),
            store: None,
        }
    }

    /// Attaches on-disk storage, creating the directory. Without a store
    /// the world is memory-only (previous behavior, still used by tests).
    pub fn set_store(&mut self, base: &Path) {
        let dir = base.join(&self.name);
        if std::fs::create_dir_all(&dir).is_ok() {
            self.store = Some(WorldStore::new(dir));
        }
    }

    pub fn spawn_drop(&mut self, entity: Entity, stack: ItemStack) {
        self.drops.push(DroppedItem {
            entity,
            stack,
            age: 0,
        });
    }

    #[must_use]
    pub fn drops(&self) -> &[DroppedItem] {
        &self.drops
    }

    pub fn drops_mut(&mut self) -> &mut Vec<DroppedItem> {
        &mut self.drops
    }

    pub fn remove_drop(&mut self, entity_id: i32) -> bool {
        match self
            .drops
            .iter()
            .position(|drop| drop.entity.id == entity_id)
        {
            Some(index) => {
                self.drops.swap_remove(index);
                true
            }
            None => false,
        }
    }

    #[must_use]
    pub fn default_world() -> Self {
        Self::new(
            DEFAULT_WORLD_NAME.to_owned(),
            DEFAULT_DIMENSION_ID.to_owned(),
        )
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn dimension(&self) -> &str {
        &self.dimension
    }

    #[must_use]
    pub fn min_y(&self) -> i32 {
        self.min_y
    }

    #[must_use]
    pub fn height(&self) -> i32 {
        self.height
    }

    #[must_use]
    pub fn spawn(&self) -> Spawn {
        self.spawn
    }

    pub fn load_chunk(&mut self, pos: ChunkPos) -> &mut Chunk {
        if self.chunks.is_loaded(pos) {
            return self.chunks.load(pos);
        }
        // Memory miss: try disk before generating flat terrain. Corrupt
        // files are ignored (fall back to generation, never crash).
        let restored = self.store.as_ref().and_then(|store| {
            let bytes = store.load_chunk(pos.x, pos.z)?;
            Chunk::decode(pos.x, pos.z, &bytes)
        });
        match restored {
            Some(chunk) => {
                self.chunks.insert(pos, chunk);
                self.chunks.load(pos)
            }
            None => self.chunks.load(pos),
        }
    }

    /// Drops every loaded chunk outside `keep`, writing dirty ones first.
    /// Without a store, dirty chunks are kept (evicting would lose edits).
    /// Returns (evicted, saved).
    pub fn evict_outside(&mut self, keep: &HashSet<ChunkPos>) -> (usize, usize) {
        let outside: Vec<ChunkPos> = self
            .chunks
            .loaded_positions()
            .into_iter()
            .filter(|pos| !keep.contains(pos))
            .collect();
        let mut evicted = 0;
        let mut saved = 0;
        for pos in outside {
            let dirty = self.chunks.get(pos).is_some_and(Chunk::is_dirty);
            if dirty {
                let Some(store) = self.store.clone() else {
                    continue;
                };
                let Some(chunk) = self.chunks.get(pos) else {
                    continue;
                };
                if store.save_chunk(pos.x, pos.z, &chunk.encode()).is_err() {
                    continue;
                }
                saved += 1;
            }
            if self.chunks.unload(pos) {
                evicted += 1;
            }
        }
        (evicted, saved)
    }

    /// Flushes dirty chunks to disk. Returns how many were written.
    pub fn save(&mut self) -> usize {
        let Some(store) = self.store.clone() else {
            return 0;
        };
        let mut saved = 0;
        for pos in self.chunks.loaded_positions() {
            let dirty = self.chunks.get(pos).is_some_and(Chunk::is_dirty);
            if !dirty {
                continue;
            }
            let Some(chunk) = self.chunks.get(pos) else {
                continue;
            };
            let bytes = chunk.encode();
            if store.save_chunk(pos.x, pos.z, &bytes).is_ok() {
                saved += 1;
                if let Some(chunk) = self.chunks.get_mut(pos) {
                    chunk.mark_clean();
                }
            }
        }
        saved
    }

    #[must_use]
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(pos)
    }

    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(pos)
    }

    pub fn unload_chunk(&mut self, pos: ChunkPos) -> bool {
        self.chunks.unload(pos)
    }

    #[must_use]
    pub fn is_chunk_loaded(&self, pos: ChunkPos) -> bool {
        self.chunks.is_loaded(pos)
    }

    #[must_use]
    pub fn loaded_chunk_count(&self) -> usize {
        self.chunks.loaded_count()
    }

    #[must_use]
    pub fn ground_top(&self, x: f64, z: f64, from_y: f64) -> Option<f64> {
        if from_y < self.min_y as f64 {
            return None;
        }
        let (bx, bz) = (x.floor() as i32, z.floor() as i32);
        let feet_solid = |y: i32| -> bool {
            matches!(self.get_block(bx, y, bz), Some(block) if block.is_solid())
        };
        let mut y = from_y.floor() as i32;
        if feet_solid(y) || feet_solid(y + 1) {
            let ceiling = self.min_y + self.height + 1;
            while y <= ceiling {
                if !feet_solid(y) && !feet_solid(y + 1) {
                    return Some(y as f64);
                }
                y += 1;
            }
            return None;
        }
        while y >= self.min_y {
            if feet_solid(y) {
                return Some(y as f64 + 1.0);
            }
            y -= 1;
        }
        None
    }

    #[must_use]
    pub fn get_block(&self, x: i32, y: i32, z: i32) -> Option<Block> {
        if !(self.min_y..self.min_y + self.height).contains(&y) {
            return None;
        }
        self.chunks
            .get(ChunkPos::from_world(x, z))
            .and_then(|chunk| chunk.get_block(x, y, z))
    }

    pub fn set_block(&mut self, x: i32, y: i32, z: i32, block: Block) -> Option<Block> {
        if !(self.min_y..self.min_y + self.height).contains(&y) {
            return None;
        }
        let pos = ChunkPos::from_world(x, z);
        self.chunks.load(pos);
        self.chunks
            .get_mut(pos)
            .and_then(|chunk| chunk.set_block(x, y, z, block))
    }
}

#[derive(Default)]
pub struct WorldManager {
    worlds: HashMap<String, World>,
}

impl WorldManager {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn create_default() -> Self {
        let mut manager = Self::new();
        manager.register(World::default_world());
        manager
    }

    pub fn register(&mut self, world: World) -> bool {
        let name = world.name().to_owned();
        if self.worlds.contains_key(&name) {
            return false;
        }
        self.worlds.insert(name, world);
        true
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&World> {
        self.worlds.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut World> {
        self.worlds.get_mut(name)
    }

    pub fn unload(&mut self, name: &str) -> bool {
        self.worlds.remove(name).is_some()
    }

    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.worlds.contains_key(name)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.worlds.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.worlds.is_empty()
    }

    pub fn names(&self) -> Vec<String> {
        self.worlds.keys().cloned().collect()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::ValuesMut<'_, String, World> {
        self.worlds.values_mut()
    }

    /// Attaches on-disk storage under `base` (one subdirectory per world).
    /// Worlds without modified chunks write nothing.
    pub fn attach_store(&mut self, base: &Path) {
        for world in self.worlds.values_mut() {
            world.set_store(base);
        }
    }

    /// Saves every world. Returns the total chunks written.
    pub fn save_all(&mut self) -> usize {
        self.worlds.values_mut().map(World::save).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_store_writes_dirty_chunks_to_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let mut manager = WorldManager::create_default();
        manager.attach_store(tmp.path());

        for world in manager.iter_mut() {
            world.load_chunk(ChunkPos::from_world(0, 0));
            world.set_block(0, 64, 0, Block::Stone);
        }

        let saved = manager.save_all();
        assert!(saved > 0, "expected at least one chunk to be written");

        let mut found_bin = false;
        let mut pending = vec![tmp.path().to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(&dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().and_then(|ext| ext.to_str()) == Some("bin") {
                    found_bin = true;
                }
            }
        }
        assert!(found_bin, "no chunk file appeared on disk");
    }

    #[test]
    fn chunk_conversion_at_boundaries() {
        for (world, chunk, local) in [
            (0, 0, 0),
            (15, 0, 15),
            (16, 1, 0),
            (17, 1, 1),
            (-1, -1, 15),
            (-16, -1, 0),
            (-17, -2, 15),
        ] {
            assert_eq!(ChunkPos::from_world(world, 0).x, chunk, "x={world}");
            assert_eq!(world.rem_euclid(16) as u8, local, "x={world}");
        }
        for (world, chunk, local) in [(0, 0, 0), (-1, -1, 15), (31, 1, 15), (-33, -3, 15)] {
            assert_eq!(ChunkPos::from_world(0, world).z, chunk, "z={world}");
            assert_eq!(world.rem_euclid(16) as u8, local, "z={world}");
        }
        assert_eq!(
            BlockPos::new(-1, 64, -17).chunk_pos(),
            ChunkPos::new(-1, -2)
        );
        assert_eq!(BlockPos::new(-1, 64, 0).local_x(), 15);
        assert_eq!(BlockPos::new(0, 64, -1).local_z(), 15);
        assert_eq!(ChunkPos::new(2, -3).min_block(), (32, -48));
    }

    #[test]
    fn sections_store_and_report_empty() {
        let mut section = ChunkSection::new(0);
        assert!(section.is_empty());
        assert_eq!(section.section_y(), 0);
        assert_eq!(section.get_local(0, 0, 0), Block::Air);
        assert_eq!(section.set_local(1, 2, 3, Block::Stone), Block::Air);
        assert_eq!(section.get_local(1, 2, 3), Block::Stone);
        assert!(!section.is_empty());
        assert_eq!(section.set_local(1, 2, 3, Block::Air), Block::Stone);
        assert!(section.is_empty());
    }

    #[test]
    fn chunk_blocks_across_sections_and_boundaries() {
        let mut chunk = Chunk::new(0, 0);
        assert_eq!(chunk.pos(), ChunkPos::new(0, 0));
        assert_eq!(chunk.section_count(), SECTION_COUNT);
        assert_eq!(chunk.get_block(0, SURFACE_Y, 0), Some(Block::GrassBlock));
        assert_eq!(chunk.get_block(0, SURFACE_Y - 1, 0), Some(Block::Dirt));
        assert_eq!(chunk.get_block(0, WORLD_MIN_Y, 0), Some(Block::Stone));
        assert_eq!(chunk.get_block(0, SURFACE_Y + 1, 0), Some(Block::Air));
        assert_eq!(chunk.get_block(0, WORLD_MIN_Y - 1, 0), None);
        assert_eq!(chunk.get_block(0, WORLD_MIN_Y + WORLD_HEIGHT, 0), None);
        assert_eq!(chunk.get_block(16, 64, 0), None);

        let prev = chunk.set_block(15, 100, 15, Block::Stone);
        assert_eq!(prev, Some(Block::Air));
        assert_eq!(chunk.get_block(15, 100, 15), Some(Block::Stone));
        let prev = chunk.set_block(15, 100, 15, Block::Dirt);
        assert_eq!(prev, Some(Block::Stone));

        let mut neg = Chunk::new(-2, -1);
        assert_eq!(neg.get_block(-17, SURFACE_Y, -1), Some(Block::GrassBlock));
        assert_eq!(neg.set_block(-32, 70, -16, Block::Dirt), Some(Block::Air));
        assert_eq!(neg.get_block(-32, 70, -16), Some(Block::Dirt));
    }

    #[test]
    fn section_changes_track_and_drain() {
        let mut chunk = Chunk::new(0, 0);
        for index in 0..chunk.section_count() {
            assert!(!chunk.section(index).unwrap().is_dirty());
        }
        assert_eq!(chunk.set_block(0, 100, 0, Block::Stone), Some(Block::Air));
        let section = (100 - WORLD_MIN_Y) as usize / SECTION_SIZE as usize;
        assert!(chunk.section(section).unwrap().is_dirty());
        let taken = chunk.section_mut(section).unwrap().take_changed();
        assert_eq!(taken, vec![(0, (100 - WORLD_MIN_Y) as u8 % 16, 0)]);
        assert!(!chunk.section(section).unwrap().is_dirty());
        assert_eq!(chunk.set_block(0, 100, 0, Block::Stone), Some(Block::Stone));
        assert!(!chunk.section(section).unwrap().is_dirty());
    }

    #[test]
    fn chunk_manager_load_get_unload() {
        let mut manager = ChunkManager::new();
        let pos = ChunkPos::new(3, -2);
        assert!(!manager.is_loaded(pos));
        assert!(manager.get(pos).is_none());
        manager.load(pos);
        assert!(manager.is_loaded(pos));
        assert_eq!(manager.loaded_count(), 1);
        manager.load(pos);
        assert_eq!(manager.loaded_count(), 1);
        assert!(manager.get_mut(pos).is_some());
        assert!(manager.unload(pos));
        assert!(!manager.is_loaded(pos));
        assert!(!manager.unload(pos));
        manager.load(ChunkPos::new(0, 0));
        manager.load(ChunkPos::new(1, 1));
        let mut positions = manager.loaded_positions();
        positions.sort_by_key(|p| (p.x, p.z));
        assert_eq!(positions, vec![ChunkPos::new(0, 0), ChunkPos::new(1, 1)]);
    }

    #[test]
    fn world_blocks_require_loaded_chunks() {
        let mut world = World::default_world();
        assert_eq!(world.name(), "world");
        assert_eq!(world.dimension(), "minecraft:overworld");
        assert_eq!(world.min_y(), WORLD_MIN_Y);
        assert_eq!(world.height(), WORLD_HEIGHT);
        let spawn = world.spawn();
        assert_eq!((spawn.x, spawn.y, spawn.z), (0.0, 100.0, 0.0));

        assert_eq!(world.get_block(0, SURFACE_Y, 0), None);
        world.load_chunk(ChunkPos::new(0, 0));
        assert!(world.is_chunk_loaded(ChunkPos::new(0, 0)));
        assert_eq!(world.loaded_chunk_count(), 1);
        assert_eq!(world.get_block(0, SURFACE_Y, 0), Some(Block::GrassBlock));
        assert_eq!(world.set_block(0, 90, 0, Block::Stone), Some(Block::Air));
        assert_eq!(world.get_block(0, 90, 0), Some(Block::Stone));
        assert_eq!(world.set_block(0, WORLD_MIN_Y - 1, 0, Block::Stone), None);
        assert!(world.unload_chunk(ChunkPos::new(0, 0)));
        assert_eq!(world.get_block(0, SURFACE_Y, 0), None);
    }

    #[test]
    fn world_manager_registers_and_unloads() {
        let mut manager = WorldManager::create_default();
        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());
        assert!(manager.contains("world"));
        assert_eq!(manager.names(), vec!["world".to_owned()]);
        assert!(manager.get("world").is_some());
        assert!(manager.get_mut("world").is_some());

        assert!(!manager.register(World::default_world()));
        assert_eq!(manager.len(), 1);
        assert!(manager.register(World::new(
            "nether".to_owned(),
            "minecraft:the_nether".to_owned()
        )));
        assert_eq!(manager.len(), 2);
        assert!(manager.get("missing").is_none());
        assert!(manager.unload("nether"));
        assert!(!manager.contains("nether"));
        assert!(!manager.unload("nether"));
        assert!(manager.unload("world"));
        assert!(manager.is_empty());
    }

    #[test]
    fn ground_top_finds_flat_surface() {
        let mut world = World::default_world();
        assert_eq!(world.ground_top(0.0, 0.0, 100.0), None);
        world.load_chunk(ChunkPos::new(0, 0));
        world.load_chunk(ChunkPos::new(-1, 0));
        assert_eq!(world.ground_top(0.0, 0.0, 100.0), Some(65.0));
        assert_eq!(world.ground_top(-0.5, 15.9, 65.0), Some(65.0));
        assert_eq!(world.ground_top(0.0, 0.0, 60.0), Some(65.0));
        assert_eq!(world.ground_top(0.0, 0.0, -100.0), None);
        assert!(Block::Stone.is_solid());
        assert!(Block::GrassBlock.is_solid());
        assert!(!Block::Air.is_solid());
    }

    #[test]
    fn evict_outside_saves_dirty_and_drops_clean() {
        let tmp = tempfile::tempdir().unwrap();
        let mut world = World::default_world();
        world.set_store(tmp.path());
        let a = ChunkPos::new(0, 0);
        let b = ChunkPos::new(1, 0);
        let c = ChunkPos::new(2, 0);
        world.load_chunk(a);
        world.load_chunk(b);
        world.load_chunk(c);
        world.set_block(0, 65, 0, Block::Stone).unwrap();

        let mut keep = HashSet::new();
        keep.insert(b);
        let (evicted, saved) = world.evict_outside(&keep);
        assert_eq!((evicted, saved), (2, 1));
        assert!(world.is_chunk_loaded(b));
        assert!(!world.is_chunk_loaded(a));
        assert!(!world.is_chunk_loaded(c));

        // The edit survives a full reload from disk.
        let mut fresh = World::default_world();
        fresh.set_store(tmp.path());
        assert_eq!(fresh.get_block(0, 65, 0), None);
        fresh.load_chunk(a);
        assert_eq!(fresh.get_block(0, 65, 0), Some(Block::Stone));
        assert_eq!(fresh.get_block(0, 64, 0), Some(Block::GrassBlock));
    }

    #[test]
    fn evict_outside_keeps_dirty_chunks_without_a_store() {
        let mut world = World::default_world();
        world.load_chunk(ChunkPos::new(0, 0));
        world.set_block(0, 65, 0, Block::Stone).unwrap();
        let (evicted, saved) = world.evict_outside(&HashSet::new());
        assert_eq!((evicted, saved), (0, 0));
        assert_eq!(world.get_block(0, 65, 0), Some(Block::Stone));
    }

    #[test]
    fn drops_spawn_age_and_remove() {
        use crate::entity::Entity;

        let mut world = World::default_world();
        assert!(world.drops().is_empty());
        world.spawn_drop(
            Entity::spawn(11, None, "world".to_owned(), 1.0, 65.0, 2.0),
            ItemStack::new(crate::inventory::Item::Dirt, 3).unwrap(),
        );
        assert_eq!(world.drops().len(), 1);
        assert_eq!(world.drops()[0].stack.count, 3);
        assert_eq!(world.drops()[0].age, 0);
        assert!(!world.remove_drop(99));
        assert!(world.remove_drop(11));
        assert!(world.drops().is_empty());
    }

    #[test]
    fn block_names_and_ids() {
        assert_eq!(Block::Air.name(), "minecraft:air");
        assert_eq!(Block::Stone.name(), "minecraft:stone");
        assert_eq!(Block::Dirt.name(), "minecraft:dirt");
        assert_eq!(Block::GrassBlock.name(), "minecraft:grass_block");
        assert_eq!(Block::Cobblestone.name(), "minecraft:cobblestone");
        assert_eq!(Block::OakPlanks.name(), "minecraft:oak_planks");
        assert_eq!(Block::Sand.name(), "minecraft:sand");
        assert_eq!(Block::Gravel.name(), "minecraft:gravel");
        assert_eq!(Block::OakLog.name(), "minecraft:oak_log");
        assert_eq!(Block::Glass.name(), "minecraft:glass");
        assert_eq!(Block::WhiteWool.name(), "minecraft:white_wool");
        assert_eq!(Block::Bricks.name(), "minecraft:bricks");
        assert_eq!(Block::Obsidian.name(), "minecraft:obsidian");
        assert_eq!(Block::Glowstone.name(), "minecraft:glowstone");
        assert_eq!(Block::from_id(0), Some(Block::Air));
        assert_eq!(Block::from_id(3), Some(Block::GrassBlock));
        assert_eq!(Block::from_id(13), Some(Block::Glowstone));
        assert_eq!(Block::from_id(240), Some(Block::PaleMossBlock));
        assert_eq!(Block::from_id(241), None);
        assert_eq!(Block::from_id(9999), None);
        assert!(Block::Glass.is_solid());
        assert!(!Block::Air.is_solid());
    }
}
