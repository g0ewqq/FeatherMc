use crate::world::Block;

pub const INVENTORY_SIZE: usize = 46;
pub const HOTBAR_START: usize = 36;
pub const HOTBAR_LEN: usize = 9;
pub const MAX_HOTBAR_INDEX: u8 = 8;
pub const MAX_STACK: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Air,
    Stone,
    Dirt,
    GrassBlock,
    Cobblestone,
    OakPlanks,
    Sand,
    Gravel,
    OakLog,
    Glass,
    WhiteWool,
    Bricks,
    Obsidian,
    Glowstone,
    Granite,
    PolishedGranite,
    Diorite,
    PolishedDiorite,
    Andesite,
    PolishedAndesite,
    CoarseDirt,
    SprucePlanks,
    BirchPlanks,
    JunglePlanks,
    AcaciaPlanks,
    CherryPlanks,
    DarkOakPlanks,
    PaleOakPlanks,
    MangrovePlanks,
    BambooPlanks,
    BambooMosaic,
    RedSand,
    GoldOre,
    DeepslateGoldOre,
    IronOre,
    DeepslateIronOre,
    CoalOre,
    DeepslateCoalOre,
    NetherGoldOre,
    Sponge,
    WetSponge,
    LapisOre,
    DeepslateLapisOre,
    LapisBlock,
    Sandstone,
    ChiseledSandstone,
    CutSandstone,
    OrangeWool,
    MagentaWool,
    LightBlueWool,
    YellowWool,
    LimeWool,
    PinkWool,
    GrayWool,
    LightGrayWool,
    CyanWool,
    PurpleWool,
    BlueWool,
    BrownWool,
    GreenWool,
    RedWool,
    BlackWool,
    GoldBlock,
    IronBlock,
    Bookshelf,
    MossyCobblestone,
    DiamondOre,
    DeepslateDiamondOre,
    DiamondBlock,
    CraftingTable,
    Clay,
    Netherrack,
    SoulSand,
    SoulSoil,
    WhiteStainedGlass,
    OrangeStainedGlass,
    MagentaStainedGlass,
    LightBlueStainedGlass,
    YellowStainedGlass,
    LimeStainedGlass,
    PinkStainedGlass,
    GrayStainedGlass,
    LightGrayStainedGlass,
    CyanStainedGlass,
    PurpleStainedGlass,
    BlueStainedGlass,
    BrownStainedGlass,
    GreenStainedGlass,
    RedStainedGlass,
    BlackStainedGlass,
    StoneBricks,
    MossyStoneBricks,
    CrackedStoneBricks,
    ChiseledStoneBricks,
    PackedMud,
    MudBricks,
    Pumpkin,
    Melon,
    ResinBlock,
    ResinBricks,
    ChiseledResinBricks,
    NetherBricks,
    EnchantingTable,
    EndStone,
    EmeraldOre,
    DeepslateEmeraldOre,
    EmeraldBlock,
    RedstoneBlock,
    NetherQuartzOre,
    QuartzBlock,
    ChiseledQuartzBlock,
    WhiteTerracotta,
    OrangeTerracotta,
    MagentaTerracotta,
    LightBlueTerracotta,
    YellowTerracotta,
    LimeTerracotta,
    PinkTerracotta,
    GrayTerracotta,
    LightGrayTerracotta,
    CyanTerracotta,
    PurpleTerracotta,
    BlueTerracotta,
    BrownTerracotta,
    GreenTerracotta,
    RedTerracotta,
    BlackTerracotta,
    Prismarine,
    PrismarineBricks,
    DarkPrismarine,
    SeaLantern,
    Terracotta,
    CoalBlock,
    PackedIce,
    RedSandstone,
    ChiseledRedSandstone,
    CutRedSandstone,
    SmoothStone,
    SmoothSandstone,
    SmoothQuartz,
    SmoothRedSandstone,
    PurpurBlock,
    EndStoneBricks,
    NetherWartBlock,
    RedNetherBricks,
    WhiteConcrete,
    OrangeConcrete,
    MagentaConcrete,
    LightBlueConcrete,
    YellowConcrete,
    LimeConcrete,
    PinkConcrete,
    GrayConcrete,
    LightGrayConcrete,
    CyanConcrete,
    PurpleConcrete,
    BlueConcrete,
    BrownConcrete,
    GreenConcrete,
    RedConcrete,
    BlackConcrete,
    DriedKelpBlock,
    DeadTubeCoralBlock,
    DeadBrainCoralBlock,
    DeadBubbleCoralBlock,
    DeadFireCoralBlock,
    DeadHornCoralBlock,
    BlueIce,
    CartographyTable,
    FletchingTable,
    SmithingTable,
    WarpedNylium,
    WarpedWartBlock,
    CrimsonNylium,
    Shroomlight,
    CrimsonPlanks,
    WarpedPlanks,
    HoneycombBlock,
    NetheriteBlock,
    AncientDebris,
    CryingObsidian,
    Lodestone,
    Blackstone,
    PolishedBlackstone,
    PolishedBlackstoneBricks,
    CrackedPolishedBlackstoneBricks,
    ChiseledPolishedBlackstone,
    GildedBlackstone,
    ChiseledNetherBricks,
    CrackedNetherBricks,
    QuartzBricks,
    AmethystBlock,
    Tuff,
    PolishedTuff,
    ChiseledTuff,
    TuffBricks,
    ChiseledTuffBricks,
    Calcite,
    TintedGlass,
    CopperBlock,
    ExposedCopper,
    WeatheredCopper,
    OxidizedCopper,
    CopperOre,
    DeepslateCopperOre,
    OxidizedCutCopper,
    WeatheredCutCopper,
    ExposedCutCopper,
    CutCopper,
    OxidizedChiseledCopper,
    WeatheredChiseledCopper,
    ExposedChiseledCopper,
    ChiseledCopper,
    WaxedOxidizedChiseledCopper,
    WaxedWeatheredChiseledCopper,
    WaxedExposedChiseledCopper,
    WaxedChiseledCopper,
    WaxedCopperBlock,
    WaxedWeatheredCopper,
    WaxedExposedCopper,
    WaxedOxidizedCopper,
    WaxedOxidizedCutCopper,
    WaxedWeatheredCutCopper,
    WaxedExposedCutCopper,
    WaxedCutCopper,
    DripstoneBlock,
    MossBlock,
    RootedDirt,
    Mud,
    CobbledDeepslate,
    PolishedDeepslate,
    DeepslateTiles,
    DeepslateBricks,
    ChiseledDeepslate,
    CrackedDeepslateBricks,
    CrackedDeepslateTiles,
    SmoothBasalt,
    RawIronBlock,
    RawCopperBlock,
    RawGoldBlock,
    PaleMossBlock,
}

impl Item {
    #[must_use]
    pub fn id(self) -> i32 {
        match self {
            Item::Air => 0,
            Item::Stone => 1,
            Item::GrassBlock => 27,
            Item::Dirt => 28,
            Item::Cobblestone => 35,
            Item::OakPlanks => 36,
            Item::Sand => 59,
            Item::Gravel => 63,
            Item::OakLog => 134,
            Item::Glass => 195,
            Item::WhiteWool => 213,
            Item::Bricks => 305,
            Item::Obsidian => 322,
            Item::Glowstone => 368,
            Item::Granite => 2,
            Item::PolishedGranite => 3,
            Item::Diorite => 4,
            Item::PolishedDiorite => 5,
            Item::Andesite => 6,
            Item::PolishedAndesite => 7,
            Item::CoarseDirt => 29,
            Item::SprucePlanks => 37,
            Item::BirchPlanks => 38,
            Item::JunglePlanks => 39,
            Item::AcaciaPlanks => 40,
            Item::CherryPlanks => 41,
            Item::DarkOakPlanks => 42,
            Item::PaleOakPlanks => 43,
            Item::MangrovePlanks => 44,
            Item::BambooPlanks => 45,
            Item::BambooMosaic => 48,
            Item::RedSand => 62,
            Item::GoldOre => 70,
            Item::DeepslateGoldOre => 71,
            Item::IronOre => 66,
            Item::DeepslateIronOre => 67,
            Item::CoalOre => 64,
            Item::DeepslateCoalOre => 65,
            Item::NetherGoldOre => 80,
            Item::Sponge => 193,
            Item::WetSponge => 194,
            Item::LapisOre => 76,
            Item::DeepslateLapisOre => 77,
            Item::LapisBlock => 197,
            Item::Sandstone => 198,
            Item::ChiseledSandstone => 199,
            Item::CutSandstone => 200,
            Item::OrangeWool => 214,
            Item::MagentaWool => 215,
            Item::LightBlueWool => 216,
            Item::YellowWool => 217,
            Item::LimeWool => 218,
            Item::PinkWool => 219,
            Item::GrayWool => 220,
            Item::LightGrayWool => 221,
            Item::CyanWool => 222,
            Item::PurpleWool => 223,
            Item::BlueWool => 224,
            Item::BrownWool => 225,
            Item::GreenWool => 226,
            Item::RedWool => 227,
            Item::BlackWool => 228,
            Item::GoldBlock => 92,
            Item::IronBlock => 90,
            Item::Bookshelf => 318,
            Item::MossyCobblestone => 321,
            Item::DiamondOre => 78,
            Item::DeepslateDiamondOre => 79,
            Item::DiamondBlock => 93,
            Item::CraftingTable => 333,
            Item::Clay => 343,
            Item::Netherrack => 360,
            Item::SoulSand => 361,
            Item::SoulSoil => 362,
            Item::WhiteStainedGlass => 531,
            Item::OrangeStainedGlass => 532,
            Item::MagentaStainedGlass => 533,
            Item::LightBlueStainedGlass => 534,
            Item::YellowStainedGlass => 535,
            Item::LimeStainedGlass => 536,
            Item::PinkStainedGlass => 537,
            Item::GrayStainedGlass => 538,
            Item::LightGrayStainedGlass => 539,
            Item::CyanStainedGlass => 540,
            Item::PurpleStainedGlass => 541,
            Item::BlueStainedGlass => 542,
            Item::BrownStainedGlass => 543,
            Item::GreenStainedGlass => 544,
            Item::RedStainedGlass => 545,
            Item::BlackStainedGlass => 546,
            Item::StoneBricks => 376,
            Item::MossyStoneBricks => 377,
            Item::CrackedStoneBricks => 378,
            Item::ChiseledStoneBricks => 379,
            Item::PackedMud => 380,
            Item::MudBricks => 381,
            Item::Pumpkin => 357,
            Item::Melon => 410,
            Item::ResinBlock => 414,
            Item::ResinBricks => 415,
            Item::ChiseledResinBricks => 419,
            Item::NetherBricks => 425,
            Item::EnchantingTable => 434,
            Item::EndStone => 436,
            Item::EmeraldOre => 74,
            Item::DeepslateEmeraldOre => 75,
            Item::EmeraldBlock => 441,
            Item::RedstoneBlock => 720,
            Item::NetherQuartzOre => 81,
            Item::QuartzBlock => 483,
            Item::ChiseledQuartzBlock => 482,
            Item::WhiteTerracotta => 487,
            Item::OrangeTerracotta => 488,
            Item::MagentaTerracotta => 489,
            Item::LightBlueTerracotta => 490,
            Item::YellowTerracotta => 491,
            Item::LimeTerracotta => 492,
            Item::PinkTerracotta => 493,
            Item::GrayTerracotta => 494,
            Item::LightGrayTerracotta => 495,
            Item::CyanTerracotta => 496,
            Item::PurpleTerracotta => 497,
            Item::BlueTerracotta => 498,
            Item::BrownTerracotta => 499,
            Item::GreenTerracotta => 500,
            Item::RedTerracotta => 501,
            Item::BlackTerracotta => 502,
            Item::Prismarine => 563,
            Item::PrismarineBricks => 564,
            Item::DarkPrismarine => 565,
            Item::SeaLantern => 569,
            Item::Terracotta => 522,
            Item::CoalBlock => 83,
            Item::PackedIce => 523,
            Item::RedSandstone => 570,
            Item::ChiseledRedSandstone => 571,
            Item::CutRedSandstone => 572,
            Item::SmoothStone => 304,
            Item::SmoothSandstone => 303,
            Item::SmoothQuartz => 301,
            Item::SmoothRedSandstone => 302,
            Item::PurpurBlock => 327,
            Item::EndStoneBricks => 437,
            Item::NetherWartBlock => 577,
            Item::RedNetherBricks => 579,
            Item::WhiteConcrete => 615,
            Item::OrangeConcrete => 616,
            Item::MagentaConcrete => 617,
            Item::LightBlueConcrete => 618,
            Item::YellowConcrete => 619,
            Item::LimeConcrete => 620,
            Item::PinkConcrete => 621,
            Item::GrayConcrete => 622,
            Item::LightGrayConcrete => 623,
            Item::CyanConcrete => 624,
            Item::PurpleConcrete => 625,
            Item::BlueConcrete => 626,
            Item::BrownConcrete => 627,
            Item::GreenConcrete => 628,
            Item::RedConcrete => 629,
            Item::BlackConcrete => 630,
            Item::DriedKelpBlock => 1028,
            Item::DeadTubeCoralBlock => 650,
            Item::DeadBrainCoralBlock => 651,
            Item::DeadBubbleCoralBlock => 652,
            Item::DeadFireCoralBlock => 653,
            Item::DeadHornCoralBlock => 654,
            Item::BlueIce => 680,
            Item::CartographyTable => 1358,
            Item::FletchingTable => 1359,
            Item::SmithingTable => 1361,
            Item::WarpedNylium => 34,
            Item::WarpedWartBlock => 578,
            Item::CrimsonNylium => 33,
            Item::Shroomlight => 1378,
            Item::CrimsonPlanks => 46,
            Item::WarpedPlanks => 47,
            Item::HoneycombBlock => 1383,
            Item::NetheriteBlock => 94,
            Item::AncientDebris => 82,
            Item::CryingObsidian => 1385,
            Item::Lodestone => 1384,
            Item::Blackstone => 1386,
            Item::PolishedBlackstone => 1390,
            Item::PolishedBlackstoneBricks => 1394,
            Item::CrackedPolishedBlackstoneBricks => 1397,
            Item::ChiseledPolishedBlackstone => 1393,
            Item::GildedBlackstone => 1389,
            Item::ChiseledNetherBricks => 427,
            Item::CrackedNetherBricks => 426,
            Item::QuartzBricks => 484,
            Item::AmethystBlock => 88,
            Item::Tuff => 12,
            Item::PolishedTuff => 17,
            Item::ChiseledTuff => 16,
            Item::TuffBricks => 21,
            Item::ChiseledTuffBricks => 25,
            Item::Calcite => 11,
            Item::TintedGlass => 196,
            Item::CopperBlock => 91,
            Item::ExposedCopper => 95,
            Item::WeatheredCopper => 96,
            Item::OxidizedCopper => 97,
            Item::CopperOre => 68,
            Item::DeepslateCopperOre => 69,
            Item::OxidizedCutCopper => 105,
            Item::WeatheredCutCopper => 104,
            Item::ExposedCutCopper => 103,
            Item::CutCopper => 102,
            Item::OxidizedChiseledCopper => 101,
            Item::WeatheredChiseledCopper => 100,
            Item::ExposedChiseledCopper => 99,
            Item::ChiseledCopper => 98,
            Item::WaxedOxidizedChiseledCopper => 121,
            Item::WaxedWeatheredChiseledCopper => 120,
            Item::WaxedExposedChiseledCopper => 119,
            Item::WaxedChiseledCopper => 118,
            Item::WaxedCopperBlock => 114,
            Item::WaxedWeatheredCopper => 116,
            Item::WaxedExposedCopper => 115,
            Item::WaxedOxidizedCopper => 117,
            Item::WaxedOxidizedCutCopper => 125,
            Item::WaxedWeatheredCutCopper => 124,
            Item::WaxedExposedCutCopper => 123,
            Item::WaxedCutCopper => 122,
            Item::DripstoneBlock => 26,
            Item::MossBlock => 263,
            Item::RootedDirt => 31,
            Item::Mud => 32,
            Item::CobbledDeepslate => 9,
            Item::PolishedDeepslate => 10,
            Item::DeepslateTiles => 384,
            Item::DeepslateBricks => 382,
            Item::ChiseledDeepslate => 386,
            Item::CrackedDeepslateBricks => 383,
            Item::CrackedDeepslateTiles => 385,
            Item::SmoothBasalt => 365,
            Item::RawIronBlock => 84,
            Item::RawCopperBlock => 85,
            Item::RawGoldBlock => 86,
            Item::PaleMossBlock => 266,
        }
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Item::Air => "minecraft:air",
            Item::Stone => "minecraft:stone",
            Item::Dirt => "minecraft:dirt",
            Item::GrassBlock => "minecraft:grass_block",
            Item::Cobblestone => "minecraft:cobblestone",
            Item::OakPlanks => "minecraft:oak_planks",
            Item::Sand => "minecraft:sand",
            Item::Gravel => "minecraft:gravel",
            Item::OakLog => "minecraft:oak_log",
            Item::Glass => "minecraft:glass",
            Item::WhiteWool => "minecraft:white_wool",
            Item::Bricks => "minecraft:bricks",
            Item::Obsidian => "minecraft:obsidian",
            Item::Glowstone => "minecraft:glowstone",
            Item::Granite => "minecraft:granite",
            Item::PolishedGranite => "minecraft:polished_granite",
            Item::Diorite => "minecraft:diorite",
            Item::PolishedDiorite => "minecraft:polished_diorite",
            Item::Andesite => "minecraft:andesite",
            Item::PolishedAndesite => "minecraft:polished_andesite",
            Item::CoarseDirt => "minecraft:coarse_dirt",
            Item::SprucePlanks => "minecraft:spruce_planks",
            Item::BirchPlanks => "minecraft:birch_planks",
            Item::JunglePlanks => "minecraft:jungle_planks",
            Item::AcaciaPlanks => "minecraft:acacia_planks",
            Item::CherryPlanks => "minecraft:cherry_planks",
            Item::DarkOakPlanks => "minecraft:dark_oak_planks",
            Item::PaleOakPlanks => "minecraft:pale_oak_planks",
            Item::MangrovePlanks => "minecraft:mangrove_planks",
            Item::BambooPlanks => "minecraft:bamboo_planks",
            Item::BambooMosaic => "minecraft:bamboo_mosaic",
            Item::RedSand => "minecraft:red_sand",
            Item::GoldOre => "minecraft:gold_ore",
            Item::DeepslateGoldOre => "minecraft:deepslate_gold_ore",
            Item::IronOre => "minecraft:iron_ore",
            Item::DeepslateIronOre => "minecraft:deepslate_iron_ore",
            Item::CoalOre => "minecraft:coal_ore",
            Item::DeepslateCoalOre => "minecraft:deepslate_coal_ore",
            Item::NetherGoldOre => "minecraft:nether_gold_ore",
            Item::Sponge => "minecraft:sponge",
            Item::WetSponge => "minecraft:wet_sponge",
            Item::LapisOre => "minecraft:lapis_ore",
            Item::DeepslateLapisOre => "minecraft:deepslate_lapis_ore",
            Item::LapisBlock => "minecraft:lapis_block",
            Item::Sandstone => "minecraft:sandstone",
            Item::ChiseledSandstone => "minecraft:chiseled_sandstone",
            Item::CutSandstone => "minecraft:cut_sandstone",
            Item::OrangeWool => "minecraft:orange_wool",
            Item::MagentaWool => "minecraft:magenta_wool",
            Item::LightBlueWool => "minecraft:light_blue_wool",
            Item::YellowWool => "minecraft:yellow_wool",
            Item::LimeWool => "minecraft:lime_wool",
            Item::PinkWool => "minecraft:pink_wool",
            Item::GrayWool => "minecraft:gray_wool",
            Item::LightGrayWool => "minecraft:light_gray_wool",
            Item::CyanWool => "minecraft:cyan_wool",
            Item::PurpleWool => "minecraft:purple_wool",
            Item::BlueWool => "minecraft:blue_wool",
            Item::BrownWool => "minecraft:brown_wool",
            Item::GreenWool => "minecraft:green_wool",
            Item::RedWool => "minecraft:red_wool",
            Item::BlackWool => "minecraft:black_wool",
            Item::GoldBlock => "minecraft:gold_block",
            Item::IronBlock => "minecraft:iron_block",
            Item::Bookshelf => "minecraft:bookshelf",
            Item::MossyCobblestone => "minecraft:mossy_cobblestone",
            Item::DiamondOre => "minecraft:diamond_ore",
            Item::DeepslateDiamondOre => "minecraft:deepslate_diamond_ore",
            Item::DiamondBlock => "minecraft:diamond_block",
            Item::CraftingTable => "minecraft:crafting_table",
            Item::Clay => "minecraft:clay",
            Item::Netherrack => "minecraft:netherrack",
            Item::SoulSand => "minecraft:soul_sand",
            Item::SoulSoil => "minecraft:soul_soil",
            Item::WhiteStainedGlass => "minecraft:white_stained_glass",
            Item::OrangeStainedGlass => "minecraft:orange_stained_glass",
            Item::MagentaStainedGlass => "minecraft:magenta_stained_glass",
            Item::LightBlueStainedGlass => "minecraft:light_blue_stained_glass",
            Item::YellowStainedGlass => "minecraft:yellow_stained_glass",
            Item::LimeStainedGlass => "minecraft:lime_stained_glass",
            Item::PinkStainedGlass => "minecraft:pink_stained_glass",
            Item::GrayStainedGlass => "minecraft:gray_stained_glass",
            Item::LightGrayStainedGlass => "minecraft:light_gray_stained_glass",
            Item::CyanStainedGlass => "minecraft:cyan_stained_glass",
            Item::PurpleStainedGlass => "minecraft:purple_stained_glass",
            Item::BlueStainedGlass => "minecraft:blue_stained_glass",
            Item::BrownStainedGlass => "minecraft:brown_stained_glass",
            Item::GreenStainedGlass => "minecraft:green_stained_glass",
            Item::RedStainedGlass => "minecraft:red_stained_glass",
            Item::BlackStainedGlass => "minecraft:black_stained_glass",
            Item::StoneBricks => "minecraft:stone_bricks",
            Item::MossyStoneBricks => "minecraft:mossy_stone_bricks",
            Item::CrackedStoneBricks => "minecraft:cracked_stone_bricks",
            Item::ChiseledStoneBricks => "minecraft:chiseled_stone_bricks",
            Item::PackedMud => "minecraft:packed_mud",
            Item::MudBricks => "minecraft:mud_bricks",
            Item::Pumpkin => "minecraft:pumpkin",
            Item::Melon => "minecraft:melon",
            Item::ResinBlock => "minecraft:resin_block",
            Item::ResinBricks => "minecraft:resin_bricks",
            Item::ChiseledResinBricks => "minecraft:chiseled_resin_bricks",
            Item::NetherBricks => "minecraft:nether_bricks",
            Item::EnchantingTable => "minecraft:enchanting_table",
            Item::EndStone => "minecraft:end_stone",
            Item::EmeraldOre => "minecraft:emerald_ore",
            Item::DeepslateEmeraldOre => "minecraft:deepslate_emerald_ore",
            Item::EmeraldBlock => "minecraft:emerald_block",
            Item::RedstoneBlock => "minecraft:redstone_block",
            Item::NetherQuartzOre => "minecraft:nether_quartz_ore",
            Item::QuartzBlock => "minecraft:quartz_block",
            Item::ChiseledQuartzBlock => "minecraft:chiseled_quartz_block",
            Item::WhiteTerracotta => "minecraft:white_terracotta",
            Item::OrangeTerracotta => "minecraft:orange_terracotta",
            Item::MagentaTerracotta => "minecraft:magenta_terracotta",
            Item::LightBlueTerracotta => "minecraft:light_blue_terracotta",
            Item::YellowTerracotta => "minecraft:yellow_terracotta",
            Item::LimeTerracotta => "minecraft:lime_terracotta",
            Item::PinkTerracotta => "minecraft:pink_terracotta",
            Item::GrayTerracotta => "minecraft:gray_terracotta",
            Item::LightGrayTerracotta => "minecraft:light_gray_terracotta",
            Item::CyanTerracotta => "minecraft:cyan_terracotta",
            Item::PurpleTerracotta => "minecraft:purple_terracotta",
            Item::BlueTerracotta => "minecraft:blue_terracotta",
            Item::BrownTerracotta => "minecraft:brown_terracotta",
            Item::GreenTerracotta => "minecraft:green_terracotta",
            Item::RedTerracotta => "minecraft:red_terracotta",
            Item::BlackTerracotta => "minecraft:black_terracotta",
            Item::Prismarine => "minecraft:prismarine",
            Item::PrismarineBricks => "minecraft:prismarine_bricks",
            Item::DarkPrismarine => "minecraft:dark_prismarine",
            Item::SeaLantern => "minecraft:sea_lantern",
            Item::Terracotta => "minecraft:terracotta",
            Item::CoalBlock => "minecraft:coal_block",
            Item::PackedIce => "minecraft:packed_ice",
            Item::RedSandstone => "minecraft:red_sandstone",
            Item::ChiseledRedSandstone => "minecraft:chiseled_red_sandstone",
            Item::CutRedSandstone => "minecraft:cut_red_sandstone",
            Item::SmoothStone => "minecraft:smooth_stone",
            Item::SmoothSandstone => "minecraft:smooth_sandstone",
            Item::SmoothQuartz => "minecraft:smooth_quartz",
            Item::SmoothRedSandstone => "minecraft:smooth_red_sandstone",
            Item::PurpurBlock => "minecraft:purpur_block",
            Item::EndStoneBricks => "minecraft:end_stone_bricks",
            Item::NetherWartBlock => "minecraft:nether_wart_block",
            Item::RedNetherBricks => "minecraft:red_nether_bricks",
            Item::WhiteConcrete => "minecraft:white_concrete",
            Item::OrangeConcrete => "minecraft:orange_concrete",
            Item::MagentaConcrete => "minecraft:magenta_concrete",
            Item::LightBlueConcrete => "minecraft:light_blue_concrete",
            Item::YellowConcrete => "minecraft:yellow_concrete",
            Item::LimeConcrete => "minecraft:lime_concrete",
            Item::PinkConcrete => "minecraft:pink_concrete",
            Item::GrayConcrete => "minecraft:gray_concrete",
            Item::LightGrayConcrete => "minecraft:light_gray_concrete",
            Item::CyanConcrete => "minecraft:cyan_concrete",
            Item::PurpleConcrete => "minecraft:purple_concrete",
            Item::BlueConcrete => "minecraft:blue_concrete",
            Item::BrownConcrete => "minecraft:brown_concrete",
            Item::GreenConcrete => "minecraft:green_concrete",
            Item::RedConcrete => "minecraft:red_concrete",
            Item::BlackConcrete => "minecraft:black_concrete",
            Item::DriedKelpBlock => "minecraft:dried_kelp_block",
            Item::DeadTubeCoralBlock => "minecraft:dead_tube_coral_block",
            Item::DeadBrainCoralBlock => "minecraft:dead_brain_coral_block",
            Item::DeadBubbleCoralBlock => "minecraft:dead_bubble_coral_block",
            Item::DeadFireCoralBlock => "minecraft:dead_fire_coral_block",
            Item::DeadHornCoralBlock => "minecraft:dead_horn_coral_block",
            Item::BlueIce => "minecraft:blue_ice",
            Item::CartographyTable => "minecraft:cartography_table",
            Item::FletchingTable => "minecraft:fletching_table",
            Item::SmithingTable => "minecraft:smithing_table",
            Item::WarpedNylium => "minecraft:warped_nylium",
            Item::WarpedWartBlock => "minecraft:warped_wart_block",
            Item::CrimsonNylium => "minecraft:crimson_nylium",
            Item::Shroomlight => "minecraft:shroomlight",
            Item::CrimsonPlanks => "minecraft:crimson_planks",
            Item::WarpedPlanks => "minecraft:warped_planks",
            Item::HoneycombBlock => "minecraft:honeycomb_block",
            Item::NetheriteBlock => "minecraft:netherite_block",
            Item::AncientDebris => "minecraft:ancient_debris",
            Item::CryingObsidian => "minecraft:crying_obsidian",
            Item::Lodestone => "minecraft:lodestone",
            Item::Blackstone => "minecraft:blackstone",
            Item::PolishedBlackstone => "minecraft:polished_blackstone",
            Item::PolishedBlackstoneBricks => "minecraft:polished_blackstone_bricks",
            Item::CrackedPolishedBlackstoneBricks => "minecraft:cracked_polished_blackstone_bricks",
            Item::ChiseledPolishedBlackstone => "minecraft:chiseled_polished_blackstone",
            Item::GildedBlackstone => "minecraft:gilded_blackstone",
            Item::ChiseledNetherBricks => "minecraft:chiseled_nether_bricks",
            Item::CrackedNetherBricks => "minecraft:cracked_nether_bricks",
            Item::QuartzBricks => "minecraft:quartz_bricks",
            Item::AmethystBlock => "minecraft:amethyst_block",
            Item::Tuff => "minecraft:tuff",
            Item::PolishedTuff => "minecraft:polished_tuff",
            Item::ChiseledTuff => "minecraft:chiseled_tuff",
            Item::TuffBricks => "minecraft:tuff_bricks",
            Item::ChiseledTuffBricks => "minecraft:chiseled_tuff_bricks",
            Item::Calcite => "minecraft:calcite",
            Item::TintedGlass => "minecraft:tinted_glass",
            Item::CopperBlock => "minecraft:copper_block",
            Item::ExposedCopper => "minecraft:exposed_copper",
            Item::WeatheredCopper => "minecraft:weathered_copper",
            Item::OxidizedCopper => "minecraft:oxidized_copper",
            Item::CopperOre => "minecraft:copper_ore",
            Item::DeepslateCopperOre => "minecraft:deepslate_copper_ore",
            Item::OxidizedCutCopper => "minecraft:oxidized_cut_copper",
            Item::WeatheredCutCopper => "minecraft:weathered_cut_copper",
            Item::ExposedCutCopper => "minecraft:exposed_cut_copper",
            Item::CutCopper => "minecraft:cut_copper",
            Item::OxidizedChiseledCopper => "minecraft:oxidized_chiseled_copper",
            Item::WeatheredChiseledCopper => "minecraft:weathered_chiseled_copper",
            Item::ExposedChiseledCopper => "minecraft:exposed_chiseled_copper",
            Item::ChiseledCopper => "minecraft:chiseled_copper",
            Item::WaxedOxidizedChiseledCopper => "minecraft:waxed_oxidized_chiseled_copper",
            Item::WaxedWeatheredChiseledCopper => "minecraft:waxed_weathered_chiseled_copper",
            Item::WaxedExposedChiseledCopper => "minecraft:waxed_exposed_chiseled_copper",
            Item::WaxedChiseledCopper => "minecraft:waxed_chiseled_copper",
            Item::WaxedCopperBlock => "minecraft:waxed_copper_block",
            Item::WaxedWeatheredCopper => "minecraft:waxed_weathered_copper",
            Item::WaxedExposedCopper => "minecraft:waxed_exposed_copper",
            Item::WaxedOxidizedCopper => "minecraft:waxed_oxidized_copper",
            Item::WaxedOxidizedCutCopper => "minecraft:waxed_oxidized_cut_copper",
            Item::WaxedWeatheredCutCopper => "minecraft:waxed_weathered_cut_copper",
            Item::WaxedExposedCutCopper => "minecraft:waxed_exposed_cut_copper",
            Item::WaxedCutCopper => "minecraft:waxed_cut_copper",
            Item::DripstoneBlock => "minecraft:dripstone_block",
            Item::MossBlock => "minecraft:moss_block",
            Item::RootedDirt => "minecraft:rooted_dirt",
            Item::Mud => "minecraft:mud",
            Item::CobbledDeepslate => "minecraft:cobbled_deepslate",
            Item::PolishedDeepslate => "minecraft:polished_deepslate",
            Item::DeepslateTiles => "minecraft:deepslate_tiles",
            Item::DeepslateBricks => "minecraft:deepslate_bricks",
            Item::ChiseledDeepslate => "minecraft:chiseled_deepslate",
            Item::CrackedDeepslateBricks => "minecraft:cracked_deepslate_bricks",
            Item::CrackedDeepslateTiles => "minecraft:cracked_deepslate_tiles",
            Item::SmoothBasalt => "minecraft:smooth_basalt",
            Item::RawIronBlock => "minecraft:raw_iron_block",
            Item::RawCopperBlock => "minecraft:raw_copper_block",
            Item::RawGoldBlock => "minecraft:raw_gold_block",
            Item::PaleMossBlock => "minecraft:pale_moss_block",
        }
    }

    #[must_use]
    pub fn from_id(id: i32) -> Option<Self> {
        match id {
            0 => Some(Item::Air),
            1 => Some(Item::Stone),
            27 => Some(Item::GrassBlock),
            28 => Some(Item::Dirt),
            35 => Some(Item::Cobblestone),
            36 => Some(Item::OakPlanks),
            59 => Some(Item::Sand),
            63 => Some(Item::Gravel),
            134 => Some(Item::OakLog),
            195 => Some(Item::Glass),
            213 => Some(Item::WhiteWool),
            305 => Some(Item::Bricks),
            322 => Some(Item::Obsidian),
            368 => Some(Item::Glowstone),
            2 => Some(Item::Granite),
            3 => Some(Item::PolishedGranite),
            4 => Some(Item::Diorite),
            5 => Some(Item::PolishedDiorite),
            6 => Some(Item::Andesite),
            7 => Some(Item::PolishedAndesite),
            29 => Some(Item::CoarseDirt),
            37 => Some(Item::SprucePlanks),
            38 => Some(Item::BirchPlanks),
            39 => Some(Item::JunglePlanks),
            40 => Some(Item::AcaciaPlanks),
            41 => Some(Item::CherryPlanks),
            42 => Some(Item::DarkOakPlanks),
            43 => Some(Item::PaleOakPlanks),
            44 => Some(Item::MangrovePlanks),
            45 => Some(Item::BambooPlanks),
            48 => Some(Item::BambooMosaic),
            62 => Some(Item::RedSand),
            70 => Some(Item::GoldOre),
            71 => Some(Item::DeepslateGoldOre),
            66 => Some(Item::IronOre),
            67 => Some(Item::DeepslateIronOre),
            64 => Some(Item::CoalOre),
            65 => Some(Item::DeepslateCoalOre),
            80 => Some(Item::NetherGoldOre),
            193 => Some(Item::Sponge),
            194 => Some(Item::WetSponge),
            76 => Some(Item::LapisOre),
            77 => Some(Item::DeepslateLapisOre),
            197 => Some(Item::LapisBlock),
            198 => Some(Item::Sandstone),
            199 => Some(Item::ChiseledSandstone),
            200 => Some(Item::CutSandstone),
            214 => Some(Item::OrangeWool),
            215 => Some(Item::MagentaWool),
            216 => Some(Item::LightBlueWool),
            217 => Some(Item::YellowWool),
            218 => Some(Item::LimeWool),
            219 => Some(Item::PinkWool),
            220 => Some(Item::GrayWool),
            221 => Some(Item::LightGrayWool),
            222 => Some(Item::CyanWool),
            223 => Some(Item::PurpleWool),
            224 => Some(Item::BlueWool),
            225 => Some(Item::BrownWool),
            226 => Some(Item::GreenWool),
            227 => Some(Item::RedWool),
            228 => Some(Item::BlackWool),
            92 => Some(Item::GoldBlock),
            90 => Some(Item::IronBlock),
            318 => Some(Item::Bookshelf),
            321 => Some(Item::MossyCobblestone),
            78 => Some(Item::DiamondOre),
            79 => Some(Item::DeepslateDiamondOre),
            93 => Some(Item::DiamondBlock),
            333 => Some(Item::CraftingTable),
            343 => Some(Item::Clay),
            360 => Some(Item::Netherrack),
            361 => Some(Item::SoulSand),
            362 => Some(Item::SoulSoil),
            531 => Some(Item::WhiteStainedGlass),
            532 => Some(Item::OrangeStainedGlass),
            533 => Some(Item::MagentaStainedGlass),
            534 => Some(Item::LightBlueStainedGlass),
            535 => Some(Item::YellowStainedGlass),
            536 => Some(Item::LimeStainedGlass),
            537 => Some(Item::PinkStainedGlass),
            538 => Some(Item::GrayStainedGlass),
            539 => Some(Item::LightGrayStainedGlass),
            540 => Some(Item::CyanStainedGlass),
            541 => Some(Item::PurpleStainedGlass),
            542 => Some(Item::BlueStainedGlass),
            543 => Some(Item::BrownStainedGlass),
            544 => Some(Item::GreenStainedGlass),
            545 => Some(Item::RedStainedGlass),
            546 => Some(Item::BlackStainedGlass),
            376 => Some(Item::StoneBricks),
            377 => Some(Item::MossyStoneBricks),
            378 => Some(Item::CrackedStoneBricks),
            379 => Some(Item::ChiseledStoneBricks),
            380 => Some(Item::PackedMud),
            381 => Some(Item::MudBricks),
            357 => Some(Item::Pumpkin),
            410 => Some(Item::Melon),
            414 => Some(Item::ResinBlock),
            415 => Some(Item::ResinBricks),
            419 => Some(Item::ChiseledResinBricks),
            425 => Some(Item::NetherBricks),
            434 => Some(Item::EnchantingTable),
            436 => Some(Item::EndStone),
            74 => Some(Item::EmeraldOre),
            75 => Some(Item::DeepslateEmeraldOre),
            441 => Some(Item::EmeraldBlock),
            720 => Some(Item::RedstoneBlock),
            81 => Some(Item::NetherQuartzOre),
            483 => Some(Item::QuartzBlock),
            482 => Some(Item::ChiseledQuartzBlock),
            487 => Some(Item::WhiteTerracotta),
            488 => Some(Item::OrangeTerracotta),
            489 => Some(Item::MagentaTerracotta),
            490 => Some(Item::LightBlueTerracotta),
            491 => Some(Item::YellowTerracotta),
            492 => Some(Item::LimeTerracotta),
            493 => Some(Item::PinkTerracotta),
            494 => Some(Item::GrayTerracotta),
            495 => Some(Item::LightGrayTerracotta),
            496 => Some(Item::CyanTerracotta),
            497 => Some(Item::PurpleTerracotta),
            498 => Some(Item::BlueTerracotta),
            499 => Some(Item::BrownTerracotta),
            500 => Some(Item::GreenTerracotta),
            501 => Some(Item::RedTerracotta),
            502 => Some(Item::BlackTerracotta),
            563 => Some(Item::Prismarine),
            564 => Some(Item::PrismarineBricks),
            565 => Some(Item::DarkPrismarine),
            569 => Some(Item::SeaLantern),
            522 => Some(Item::Terracotta),
            83 => Some(Item::CoalBlock),
            523 => Some(Item::PackedIce),
            570 => Some(Item::RedSandstone),
            571 => Some(Item::ChiseledRedSandstone),
            572 => Some(Item::CutRedSandstone),
            304 => Some(Item::SmoothStone),
            303 => Some(Item::SmoothSandstone),
            301 => Some(Item::SmoothQuartz),
            302 => Some(Item::SmoothRedSandstone),
            327 => Some(Item::PurpurBlock),
            437 => Some(Item::EndStoneBricks),
            577 => Some(Item::NetherWartBlock),
            579 => Some(Item::RedNetherBricks),
            615 => Some(Item::WhiteConcrete),
            616 => Some(Item::OrangeConcrete),
            617 => Some(Item::MagentaConcrete),
            618 => Some(Item::LightBlueConcrete),
            619 => Some(Item::YellowConcrete),
            620 => Some(Item::LimeConcrete),
            621 => Some(Item::PinkConcrete),
            622 => Some(Item::GrayConcrete),
            623 => Some(Item::LightGrayConcrete),
            624 => Some(Item::CyanConcrete),
            625 => Some(Item::PurpleConcrete),
            626 => Some(Item::BlueConcrete),
            627 => Some(Item::BrownConcrete),
            628 => Some(Item::GreenConcrete),
            629 => Some(Item::RedConcrete),
            630 => Some(Item::BlackConcrete),
            1028 => Some(Item::DriedKelpBlock),
            650 => Some(Item::DeadTubeCoralBlock),
            651 => Some(Item::DeadBrainCoralBlock),
            652 => Some(Item::DeadBubbleCoralBlock),
            653 => Some(Item::DeadFireCoralBlock),
            654 => Some(Item::DeadHornCoralBlock),
            680 => Some(Item::BlueIce),
            1358 => Some(Item::CartographyTable),
            1359 => Some(Item::FletchingTable),
            1361 => Some(Item::SmithingTable),
            34 => Some(Item::WarpedNylium),
            578 => Some(Item::WarpedWartBlock),
            33 => Some(Item::CrimsonNylium),
            1378 => Some(Item::Shroomlight),
            46 => Some(Item::CrimsonPlanks),
            47 => Some(Item::WarpedPlanks),
            1383 => Some(Item::HoneycombBlock),
            94 => Some(Item::NetheriteBlock),
            82 => Some(Item::AncientDebris),
            1385 => Some(Item::CryingObsidian),
            1384 => Some(Item::Lodestone),
            1386 => Some(Item::Blackstone),
            1390 => Some(Item::PolishedBlackstone),
            1394 => Some(Item::PolishedBlackstoneBricks),
            1397 => Some(Item::CrackedPolishedBlackstoneBricks),
            1393 => Some(Item::ChiseledPolishedBlackstone),
            1389 => Some(Item::GildedBlackstone),
            427 => Some(Item::ChiseledNetherBricks),
            426 => Some(Item::CrackedNetherBricks),
            484 => Some(Item::QuartzBricks),
            88 => Some(Item::AmethystBlock),
            12 => Some(Item::Tuff),
            17 => Some(Item::PolishedTuff),
            16 => Some(Item::ChiseledTuff),
            21 => Some(Item::TuffBricks),
            25 => Some(Item::ChiseledTuffBricks),
            11 => Some(Item::Calcite),
            196 => Some(Item::TintedGlass),
            91 => Some(Item::CopperBlock),
            95 => Some(Item::ExposedCopper),
            96 => Some(Item::WeatheredCopper),
            97 => Some(Item::OxidizedCopper),
            68 => Some(Item::CopperOre),
            69 => Some(Item::DeepslateCopperOre),
            105 => Some(Item::OxidizedCutCopper),
            104 => Some(Item::WeatheredCutCopper),
            103 => Some(Item::ExposedCutCopper),
            102 => Some(Item::CutCopper),
            101 => Some(Item::OxidizedChiseledCopper),
            100 => Some(Item::WeatheredChiseledCopper),
            99 => Some(Item::ExposedChiseledCopper),
            98 => Some(Item::ChiseledCopper),
            121 => Some(Item::WaxedOxidizedChiseledCopper),
            120 => Some(Item::WaxedWeatheredChiseledCopper),
            119 => Some(Item::WaxedExposedChiseledCopper),
            118 => Some(Item::WaxedChiseledCopper),
            114 => Some(Item::WaxedCopperBlock),
            116 => Some(Item::WaxedWeatheredCopper),
            115 => Some(Item::WaxedExposedCopper),
            117 => Some(Item::WaxedOxidizedCopper),
            125 => Some(Item::WaxedOxidizedCutCopper),
            124 => Some(Item::WaxedWeatheredCutCopper),
            123 => Some(Item::WaxedExposedCutCopper),
            122 => Some(Item::WaxedCutCopper),
            26 => Some(Item::DripstoneBlock),
            263 => Some(Item::MossBlock),
            31 => Some(Item::RootedDirt),
            32 => Some(Item::Mud),
            9 => Some(Item::CobbledDeepslate),
            10 => Some(Item::PolishedDeepslate),
            384 => Some(Item::DeepslateTiles),
            382 => Some(Item::DeepslateBricks),
            386 => Some(Item::ChiseledDeepslate),
            383 => Some(Item::CrackedDeepslateBricks),
            385 => Some(Item::CrackedDeepslateTiles),
            365 => Some(Item::SmoothBasalt),
            84 => Some(Item::RawIronBlock),
            85 => Some(Item::RawCopperBlock),
            86 => Some(Item::RawGoldBlock),
            266 => Some(Item::PaleMossBlock),
            _ => None,
        }
    }

    #[must_use]
    pub fn block(self) -> Option<Block> {
        match self {
            Item::Air => None,
            Item::Stone => Some(Block::Stone),
            Item::Dirt => Some(Block::Dirt),
            Item::GrassBlock => Some(Block::GrassBlock),
            Item::Cobblestone => Some(Block::Cobblestone),
            Item::OakPlanks => Some(Block::OakPlanks),
            Item::Sand => Some(Block::Sand),
            Item::Gravel => Some(Block::Gravel),
            Item::OakLog => Some(Block::OakLog),
            Item::Glass => Some(Block::Glass),
            Item::WhiteWool => Some(Block::WhiteWool),
            Item::Bricks => Some(Block::Bricks),
            Item::Obsidian => Some(Block::Obsidian),
            Item::Glowstone => Some(Block::Glowstone),
            Item::Granite => Some(Block::Granite),
            Item::PolishedGranite => Some(Block::PolishedGranite),
            Item::Diorite => Some(Block::Diorite),
            Item::PolishedDiorite => Some(Block::PolishedDiorite),
            Item::Andesite => Some(Block::Andesite),
            Item::PolishedAndesite => Some(Block::PolishedAndesite),
            Item::CoarseDirt => Some(Block::CoarseDirt),
            Item::SprucePlanks => Some(Block::SprucePlanks),
            Item::BirchPlanks => Some(Block::BirchPlanks),
            Item::JunglePlanks => Some(Block::JunglePlanks),
            Item::AcaciaPlanks => Some(Block::AcaciaPlanks),
            Item::CherryPlanks => Some(Block::CherryPlanks),
            Item::DarkOakPlanks => Some(Block::DarkOakPlanks),
            Item::PaleOakPlanks => Some(Block::PaleOakPlanks),
            Item::MangrovePlanks => Some(Block::MangrovePlanks),
            Item::BambooPlanks => Some(Block::BambooPlanks),
            Item::BambooMosaic => Some(Block::BambooMosaic),
            Item::RedSand => Some(Block::RedSand),
            Item::GoldOre => Some(Block::GoldOre),
            Item::DeepslateGoldOre => Some(Block::DeepslateGoldOre),
            Item::IronOre => Some(Block::IronOre),
            Item::DeepslateIronOre => Some(Block::DeepslateIronOre),
            Item::CoalOre => Some(Block::CoalOre),
            Item::DeepslateCoalOre => Some(Block::DeepslateCoalOre),
            Item::NetherGoldOre => Some(Block::NetherGoldOre),
            Item::Sponge => Some(Block::Sponge),
            Item::WetSponge => Some(Block::WetSponge),
            Item::LapisOre => Some(Block::LapisOre),
            Item::DeepslateLapisOre => Some(Block::DeepslateLapisOre),
            Item::LapisBlock => Some(Block::LapisBlock),
            Item::Sandstone => Some(Block::Sandstone),
            Item::ChiseledSandstone => Some(Block::ChiseledSandstone),
            Item::CutSandstone => Some(Block::CutSandstone),
            Item::OrangeWool => Some(Block::OrangeWool),
            Item::MagentaWool => Some(Block::MagentaWool),
            Item::LightBlueWool => Some(Block::LightBlueWool),
            Item::YellowWool => Some(Block::YellowWool),
            Item::LimeWool => Some(Block::LimeWool),
            Item::PinkWool => Some(Block::PinkWool),
            Item::GrayWool => Some(Block::GrayWool),
            Item::LightGrayWool => Some(Block::LightGrayWool),
            Item::CyanWool => Some(Block::CyanWool),
            Item::PurpleWool => Some(Block::PurpleWool),
            Item::BlueWool => Some(Block::BlueWool),
            Item::BrownWool => Some(Block::BrownWool),
            Item::GreenWool => Some(Block::GreenWool),
            Item::RedWool => Some(Block::RedWool),
            Item::BlackWool => Some(Block::BlackWool),
            Item::GoldBlock => Some(Block::GoldBlock),
            Item::IronBlock => Some(Block::IronBlock),
            Item::Bookshelf => Some(Block::Bookshelf),
            Item::MossyCobblestone => Some(Block::MossyCobblestone),
            Item::DiamondOre => Some(Block::DiamondOre),
            Item::DeepslateDiamondOre => Some(Block::DeepslateDiamondOre),
            Item::DiamondBlock => Some(Block::DiamondBlock),
            Item::CraftingTable => Some(Block::CraftingTable),
            Item::Clay => Some(Block::Clay),
            Item::Netherrack => Some(Block::Netherrack),
            Item::SoulSand => Some(Block::SoulSand),
            Item::SoulSoil => Some(Block::SoulSoil),
            Item::WhiteStainedGlass => Some(Block::WhiteStainedGlass),
            Item::OrangeStainedGlass => Some(Block::OrangeStainedGlass),
            Item::MagentaStainedGlass => Some(Block::MagentaStainedGlass),
            Item::LightBlueStainedGlass => Some(Block::LightBlueStainedGlass),
            Item::YellowStainedGlass => Some(Block::YellowStainedGlass),
            Item::LimeStainedGlass => Some(Block::LimeStainedGlass),
            Item::PinkStainedGlass => Some(Block::PinkStainedGlass),
            Item::GrayStainedGlass => Some(Block::GrayStainedGlass),
            Item::LightGrayStainedGlass => Some(Block::LightGrayStainedGlass),
            Item::CyanStainedGlass => Some(Block::CyanStainedGlass),
            Item::PurpleStainedGlass => Some(Block::PurpleStainedGlass),
            Item::BlueStainedGlass => Some(Block::BlueStainedGlass),
            Item::BrownStainedGlass => Some(Block::BrownStainedGlass),
            Item::GreenStainedGlass => Some(Block::GreenStainedGlass),
            Item::RedStainedGlass => Some(Block::RedStainedGlass),
            Item::BlackStainedGlass => Some(Block::BlackStainedGlass),
            Item::StoneBricks => Some(Block::StoneBricks),
            Item::MossyStoneBricks => Some(Block::MossyStoneBricks),
            Item::CrackedStoneBricks => Some(Block::CrackedStoneBricks),
            Item::ChiseledStoneBricks => Some(Block::ChiseledStoneBricks),
            Item::PackedMud => Some(Block::PackedMud),
            Item::MudBricks => Some(Block::MudBricks),
            Item::Pumpkin => Some(Block::Pumpkin),
            Item::Melon => Some(Block::Melon),
            Item::ResinBlock => Some(Block::ResinBlock),
            Item::ResinBricks => Some(Block::ResinBricks),
            Item::ChiseledResinBricks => Some(Block::ChiseledResinBricks),
            Item::NetherBricks => Some(Block::NetherBricks),
            Item::EnchantingTable => Some(Block::EnchantingTable),
            Item::EndStone => Some(Block::EndStone),
            Item::EmeraldOre => Some(Block::EmeraldOre),
            Item::DeepslateEmeraldOre => Some(Block::DeepslateEmeraldOre),
            Item::EmeraldBlock => Some(Block::EmeraldBlock),
            Item::RedstoneBlock => Some(Block::RedstoneBlock),
            Item::NetherQuartzOre => Some(Block::NetherQuartzOre),
            Item::QuartzBlock => Some(Block::QuartzBlock),
            Item::ChiseledQuartzBlock => Some(Block::ChiseledQuartzBlock),
            Item::WhiteTerracotta => Some(Block::WhiteTerracotta),
            Item::OrangeTerracotta => Some(Block::OrangeTerracotta),
            Item::MagentaTerracotta => Some(Block::MagentaTerracotta),
            Item::LightBlueTerracotta => Some(Block::LightBlueTerracotta),
            Item::YellowTerracotta => Some(Block::YellowTerracotta),
            Item::LimeTerracotta => Some(Block::LimeTerracotta),
            Item::PinkTerracotta => Some(Block::PinkTerracotta),
            Item::GrayTerracotta => Some(Block::GrayTerracotta),
            Item::LightGrayTerracotta => Some(Block::LightGrayTerracotta),
            Item::CyanTerracotta => Some(Block::CyanTerracotta),
            Item::PurpleTerracotta => Some(Block::PurpleTerracotta),
            Item::BlueTerracotta => Some(Block::BlueTerracotta),
            Item::BrownTerracotta => Some(Block::BrownTerracotta),
            Item::GreenTerracotta => Some(Block::GreenTerracotta),
            Item::RedTerracotta => Some(Block::RedTerracotta),
            Item::BlackTerracotta => Some(Block::BlackTerracotta),
            Item::Prismarine => Some(Block::Prismarine),
            Item::PrismarineBricks => Some(Block::PrismarineBricks),
            Item::DarkPrismarine => Some(Block::DarkPrismarine),
            Item::SeaLantern => Some(Block::SeaLantern),
            Item::Terracotta => Some(Block::Terracotta),
            Item::CoalBlock => Some(Block::CoalBlock),
            Item::PackedIce => Some(Block::PackedIce),
            Item::RedSandstone => Some(Block::RedSandstone),
            Item::ChiseledRedSandstone => Some(Block::ChiseledRedSandstone),
            Item::CutRedSandstone => Some(Block::CutRedSandstone),
            Item::SmoothStone => Some(Block::SmoothStone),
            Item::SmoothSandstone => Some(Block::SmoothSandstone),
            Item::SmoothQuartz => Some(Block::SmoothQuartz),
            Item::SmoothRedSandstone => Some(Block::SmoothRedSandstone),
            Item::PurpurBlock => Some(Block::PurpurBlock),
            Item::EndStoneBricks => Some(Block::EndStoneBricks),
            Item::NetherWartBlock => Some(Block::NetherWartBlock),
            Item::RedNetherBricks => Some(Block::RedNetherBricks),
            Item::WhiteConcrete => Some(Block::WhiteConcrete),
            Item::OrangeConcrete => Some(Block::OrangeConcrete),
            Item::MagentaConcrete => Some(Block::MagentaConcrete),
            Item::LightBlueConcrete => Some(Block::LightBlueConcrete),
            Item::YellowConcrete => Some(Block::YellowConcrete),
            Item::LimeConcrete => Some(Block::LimeConcrete),
            Item::PinkConcrete => Some(Block::PinkConcrete),
            Item::GrayConcrete => Some(Block::GrayConcrete),
            Item::LightGrayConcrete => Some(Block::LightGrayConcrete),
            Item::CyanConcrete => Some(Block::CyanConcrete),
            Item::PurpleConcrete => Some(Block::PurpleConcrete),
            Item::BlueConcrete => Some(Block::BlueConcrete),
            Item::BrownConcrete => Some(Block::BrownConcrete),
            Item::GreenConcrete => Some(Block::GreenConcrete),
            Item::RedConcrete => Some(Block::RedConcrete),
            Item::BlackConcrete => Some(Block::BlackConcrete),
            Item::DriedKelpBlock => Some(Block::DriedKelpBlock),
            Item::DeadTubeCoralBlock => Some(Block::DeadTubeCoralBlock),
            Item::DeadBrainCoralBlock => Some(Block::DeadBrainCoralBlock),
            Item::DeadBubbleCoralBlock => Some(Block::DeadBubbleCoralBlock),
            Item::DeadFireCoralBlock => Some(Block::DeadFireCoralBlock),
            Item::DeadHornCoralBlock => Some(Block::DeadHornCoralBlock),
            Item::BlueIce => Some(Block::BlueIce),
            Item::CartographyTable => Some(Block::CartographyTable),
            Item::FletchingTable => Some(Block::FletchingTable),
            Item::SmithingTable => Some(Block::SmithingTable),
            Item::WarpedNylium => Some(Block::WarpedNylium),
            Item::WarpedWartBlock => Some(Block::WarpedWartBlock),
            Item::CrimsonNylium => Some(Block::CrimsonNylium),
            Item::Shroomlight => Some(Block::Shroomlight),
            Item::CrimsonPlanks => Some(Block::CrimsonPlanks),
            Item::WarpedPlanks => Some(Block::WarpedPlanks),
            Item::HoneycombBlock => Some(Block::HoneycombBlock),
            Item::NetheriteBlock => Some(Block::NetheriteBlock),
            Item::AncientDebris => Some(Block::AncientDebris),
            Item::CryingObsidian => Some(Block::CryingObsidian),
            Item::Lodestone => Some(Block::Lodestone),
            Item::Blackstone => Some(Block::Blackstone),
            Item::PolishedBlackstone => Some(Block::PolishedBlackstone),
            Item::PolishedBlackstoneBricks => Some(Block::PolishedBlackstoneBricks),
            Item::CrackedPolishedBlackstoneBricks => Some(Block::CrackedPolishedBlackstoneBricks),
            Item::ChiseledPolishedBlackstone => Some(Block::ChiseledPolishedBlackstone),
            Item::GildedBlackstone => Some(Block::GildedBlackstone),
            Item::ChiseledNetherBricks => Some(Block::ChiseledNetherBricks),
            Item::CrackedNetherBricks => Some(Block::CrackedNetherBricks),
            Item::QuartzBricks => Some(Block::QuartzBricks),
            Item::AmethystBlock => Some(Block::AmethystBlock),
            Item::Tuff => Some(Block::Tuff),
            Item::PolishedTuff => Some(Block::PolishedTuff),
            Item::ChiseledTuff => Some(Block::ChiseledTuff),
            Item::TuffBricks => Some(Block::TuffBricks),
            Item::ChiseledTuffBricks => Some(Block::ChiseledTuffBricks),
            Item::Calcite => Some(Block::Calcite),
            Item::TintedGlass => Some(Block::TintedGlass),
            Item::CopperBlock => Some(Block::CopperBlock),
            Item::ExposedCopper => Some(Block::ExposedCopper),
            Item::WeatheredCopper => Some(Block::WeatheredCopper),
            Item::OxidizedCopper => Some(Block::OxidizedCopper),
            Item::CopperOre => Some(Block::CopperOre),
            Item::DeepslateCopperOre => Some(Block::DeepslateCopperOre),
            Item::OxidizedCutCopper => Some(Block::OxidizedCutCopper),
            Item::WeatheredCutCopper => Some(Block::WeatheredCutCopper),
            Item::ExposedCutCopper => Some(Block::ExposedCutCopper),
            Item::CutCopper => Some(Block::CutCopper),
            Item::OxidizedChiseledCopper => Some(Block::OxidizedChiseledCopper),
            Item::WeatheredChiseledCopper => Some(Block::WeatheredChiseledCopper),
            Item::ExposedChiseledCopper => Some(Block::ExposedChiseledCopper),
            Item::ChiseledCopper => Some(Block::ChiseledCopper),
            Item::WaxedOxidizedChiseledCopper => Some(Block::WaxedOxidizedChiseledCopper),
            Item::WaxedWeatheredChiseledCopper => Some(Block::WaxedWeatheredChiseledCopper),
            Item::WaxedExposedChiseledCopper => Some(Block::WaxedExposedChiseledCopper),
            Item::WaxedChiseledCopper => Some(Block::WaxedChiseledCopper),
            Item::WaxedCopperBlock => Some(Block::WaxedCopperBlock),
            Item::WaxedWeatheredCopper => Some(Block::WaxedWeatheredCopper),
            Item::WaxedExposedCopper => Some(Block::WaxedExposedCopper),
            Item::WaxedOxidizedCopper => Some(Block::WaxedOxidizedCopper),
            Item::WaxedOxidizedCutCopper => Some(Block::WaxedOxidizedCutCopper),
            Item::WaxedWeatheredCutCopper => Some(Block::WaxedWeatheredCutCopper),
            Item::WaxedExposedCutCopper => Some(Block::WaxedExposedCutCopper),
            Item::WaxedCutCopper => Some(Block::WaxedCutCopper),
            Item::DripstoneBlock => Some(Block::DripstoneBlock),
            Item::MossBlock => Some(Block::MossBlock),
            Item::RootedDirt => Some(Block::RootedDirt),
            Item::Mud => Some(Block::Mud),
            Item::CobbledDeepslate => Some(Block::CobbledDeepslate),
            Item::PolishedDeepslate => Some(Block::PolishedDeepslate),
            Item::DeepslateTiles => Some(Block::DeepslateTiles),
            Item::DeepslateBricks => Some(Block::DeepslateBricks),
            Item::ChiseledDeepslate => Some(Block::ChiseledDeepslate),
            Item::CrackedDeepslateBricks => Some(Block::CrackedDeepslateBricks),
            Item::CrackedDeepslateTiles => Some(Block::CrackedDeepslateTiles),
            Item::SmoothBasalt => Some(Block::SmoothBasalt),
            Item::RawIronBlock => Some(Block::RawIronBlock),
            Item::RawCopperBlock => Some(Block::RawCopperBlock),
            Item::RawGoldBlock => Some(Block::RawGoldBlock),
            Item::PaleMossBlock => Some(Block::PaleMossBlock),
        }
    }

    #[must_use]
    pub fn max_stack(self) -> u32 {
        let _ = self;
        MAX_STACK
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ItemStack {
    pub item: Item,
    pub count: u32,
}

impl ItemStack {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            item: Item::Air,
            count: 0,
        }
    }

    #[must_use]
    pub fn new(item: Item, count: u32) -> Option<Self> {
        if item == Item::Air || count == 0 || count > item.max_stack() {
            return None;
        }
        Some(Self { item, count })
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self.item == Item::Air || self.count == 0
    }
}

#[derive(Debug, Clone)]
pub struct Inventory {
    slots: [ItemStack; INVENTORY_SIZE],
    selected: u8,
}

impl Inventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            slots: [ItemStack::empty(); INVENTORY_SIZE],
            selected: 0,
        }
    }

    #[must_use]
    pub fn starting() -> Self {
        let mut inventory = Self::new();
        inventory.slots[HOTBAR_START] = ItemStack::new(Item::Stone, 64).expect("valid stack");
        inventory.slots[HOTBAR_START + 1] = ItemStack::new(Item::Dirt, 64).expect("valid stack");
        inventory.slots[HOTBAR_START + 2] =
            ItemStack::new(Item::GrassBlock, 64).expect("valid stack");
        inventory
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<ItemStack> {
        self.slots.get(index).copied()
    }

    pub fn set(&mut self, index: usize, stack: ItemStack) -> bool {
        match self.slots.get_mut(index) {
            Some(slot) => {
                *slot = stack;
                true
            }
            None => false,
        }
    }

    pub fn clear(&mut self, index: usize) -> bool {
        self.set(index, ItemStack::empty())
    }

    pub fn add(&mut self, mut stack: ItemStack) -> u32 {
        if stack.is_empty() {
            return 0;
        }
        for slot in self.slots.iter_mut() {
            if stack.count == 0 {
                break;
            }
            if slot.item == stack.item && slot.count < slot.item.max_stack() {
                let room = slot.item.max_stack() - slot.count;
                let take = room.min(stack.count);
                slot.count += take;
                stack.count -= take;
            }
        }
        for slot in self.slots.iter_mut() {
            if stack.count == 0 {
                break;
            }
            if slot.is_empty() {
                let take = stack.item.max_stack().min(stack.count);
                *slot = ItemStack {
                    item: stack.item,
                    count: take,
                };
                stack.count -= take;
            }
        }
        stack.count
    }

    pub fn take(&mut self, index: usize, count: u32) -> Option<ItemStack> {
        let slot = self.slots.get_mut(index)?;
        if slot.is_empty() || count == 0 {
            return None;
        }
        let take = count.min(slot.count);
        let taken = ItemStack {
            item: slot.item,
            count: take,
        };
        slot.count -= take;
        if slot.count == 0 {
            *slot = ItemStack::empty();
        }
        Some(taken)
    }

    pub fn consume_one(&mut self, index: usize) -> bool {
        self.take(index, 1).is_some()
    }

    #[must_use]
    pub fn selected(&self) -> u8 {
        self.selected
    }

    pub fn select(&mut self, index: u8) -> bool {
        if index > MAX_HOTBAR_INDEX {
            return false;
        }
        self.selected = index;
        true
    }

    #[must_use]
    pub fn selected_slot_index(&self) -> usize {
        HOTBAR_START + self.selected as usize
    }

    #[must_use]
    pub fn held(&self) -> ItemStack {
        self.slots[self.selected_slot_index()]
    }

    #[must_use]
    pub fn hotbar(&self, index: u8) -> Option<ItemStack> {
        if index > MAX_HOTBAR_INDEX {
            return None;
        }
        Some(self.slots[HOTBAR_START + index as usize])
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_ids_match_vanilla_registry() {
        assert_eq!(Item::Air.id(), 0);
        assert_eq!(Item::Stone.id(), 1);
        assert_eq!(Item::GrassBlock.id(), 27);
        assert_eq!(Item::Dirt.id(), 28);
        assert_eq!(Item::Cobblestone.id(), 35);
        assert_eq!(Item::OakPlanks.id(), 36);
        assert_eq!(Item::Sand.id(), 59);
        assert_eq!(Item::Gravel.id(), 63);
        assert_eq!(Item::OakLog.id(), 134);
        assert_eq!(Item::Glass.id(), 195);
        assert_eq!(Item::WhiteWool.id(), 213);
        assert_eq!(Item::Bricks.id(), 305);
        assert_eq!(Item::Obsidian.id(), 322);
        assert_eq!(Item::Glowstone.id(), 368);
        assert_eq!(Item::from_id(1), Some(Item::Stone));
        assert_eq!(Item::from_id(368), Some(Item::Glowstone));
        assert_eq!(Item::from_id(78), Some(Item::DiamondOre));
        assert_eq!(Item::from_id(333), Some(Item::CraftingTable));
        assert_eq!(Item::from_id(99), Some(Item::ExposedChiseledCopper));
        assert_eq!(Item::from_id(99999), None);
    }

    #[test]
    fn items_map_to_matching_blocks() {
        assert_eq!(Item::Stone.block(), Some(Block::Stone));
        assert_eq!(Item::Dirt.block(), Some(Block::Dirt));
        assert_eq!(Item::GrassBlock.block(), Some(Block::GrassBlock));
        assert_eq!(Item::Cobblestone.block(), Some(Block::Cobblestone));
        assert_eq!(Item::OakPlanks.block(), Some(Block::OakPlanks));
        assert_eq!(Item::Sand.block(), Some(Block::Sand));
        assert_eq!(Item::Gravel.block(), Some(Block::Gravel));
        assert_eq!(Item::OakLog.block(), Some(Block::OakLog));
        assert_eq!(Item::Glass.block(), Some(Block::Glass));
        assert_eq!(Item::WhiteWool.block(), Some(Block::WhiteWool));
        assert_eq!(Item::Bricks.block(), Some(Block::Bricks));
        assert_eq!(Item::Obsidian.block(), Some(Block::Obsidian));
        assert_eq!(Item::Glowstone.block(), Some(Block::Glowstone));
        assert_eq!(Item::Air.block(), None);
        assert_eq!(Item::Stone.name(), Block::Stone.name());
        assert_eq!(Item::Glass.name(), Block::Glass.name());
    }

    #[test]
    fn stacks_reject_invalid_counts() {
        assert!(ItemStack::new(Item::Stone, 0).is_none());
        assert!(ItemStack::new(Item::Stone, 65).is_none());
        assert!(ItemStack::new(Item::Air, 5).is_none());
        let stack = ItemStack::new(Item::Dirt, 64).unwrap();
        assert!(!stack.is_empty());
        assert!(ItemStack::empty().is_empty());
    }

    #[test]
    fn inventory_reads_writes_and_clears() {
        let mut inventory = Inventory::new();
        assert_eq!(inventory.get(0), Some(ItemStack::empty()));
        assert_eq!(inventory.get(99), None);
        assert!(inventory.set(9, ItemStack::new(Item::Stone, 5).unwrap()));
        assert_eq!(
            inventory.get(9),
            Some(ItemStack::new(Item::Stone, 5).unwrap())
        );
        assert!(!inventory.set(99, ItemStack::empty()));
        assert!(inventory.clear(9));
        assert_eq!(inventory.get(9), Some(ItemStack::empty()));
        assert!(!inventory.clear(99));
    }

    #[test]
    fn inventory_adds_merges_and_reports_leftover() {
        let mut inventory = Inventory::new();
        inventory.set(0, ItemStack::new(Item::Stone, 60).unwrap());
        assert_eq!(inventory.add(ItemStack::new(Item::Stone, 10).unwrap()), 0);
        assert_eq!(inventory.get(0).unwrap().count, 64);
        assert_eq!(inventory.get(1).unwrap().count, 6);

        let mut full = Inventory::new();
        for index in 0..INVENTORY_SIZE {
            full.set(index, ItemStack::new(Item::Dirt, 64).unwrap());
        }
        assert_eq!(full.add(ItemStack::new(Item::Dirt, 5).unwrap()), 5);
        assert_eq!(full.add(ItemStack::empty()), 0);
    }

    #[test]
    fn inventory_take_consumes_and_empties() {
        let mut inventory = Inventory::new();
        inventory.set(3, ItemStack::new(Item::Dirt, 5).unwrap());
        assert_eq!(
            inventory.take(3, 2),
            Some(ItemStack::new(Item::Dirt, 2).unwrap())
        );
        assert_eq!(inventory.get(3).unwrap().count, 3);
        assert!(inventory.consume_one(3));
        assert_eq!(inventory.take(99, 1), None);
        assert_eq!(inventory.take(4, 1), None);
        assert_eq!(inventory.take(3, 0), None);
    }

    #[test]
    fn hotbar_selection_drives_held_item() {
        let mut inventory = Inventory::starting();
        assert_eq!(inventory.selected(), 0);
        assert_eq!(inventory.hotbar(0).unwrap().item, Item::Stone);
        assert_eq!(inventory.hotbar(8), Some(ItemStack::empty()));
        assert_eq!(inventory.hotbar(9), None);
        assert_eq!(inventory.held().item, Item::Stone);
        assert!(inventory.select(2));
        assert_eq!(inventory.held().item, Item::GrassBlock);
        assert!(!inventory.select(9));
        assert!(!inventory.select(u8::MAX));
        assert_eq!(inventory.selected(), 2);
    }
}
