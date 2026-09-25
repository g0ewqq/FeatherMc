use super::packets::encode_packet;
use super::proto::Writer;
use crate::world::{Block, Chunk, SECTION_COUNT, SURFACE_Y, WORLD_HEIGHT, WORLD_MIN_Y};

pub const CHUNK_RADIUS: i32 = 6;

const AIR_STATE: i32 = 0;
const STONE_STATE: i32 = 1;
// Grass Block states 8 (snowy) / 9 (dry); 9 is the natural default.
const GRASS_STATE: i32 = 9;
const DIRT_STATE: i32 = 10;
const COBBLESTONE_STATE: i32 = 14;
const OAK_PLANKS_STATE: i32 = 15;
const SAND_STATE: i32 = 118;
const GRAVEL_STATE: i32 = 124;
// Oak Log axis states 136-138; 137 (axis y) is the natural default.
const OAK_LOG_STATE: i32 = 137;
const GLASS_STATE: i32 = 562;
const WHITE_WOOL_STATE: i32 = 2293;
const BRICKS_STATE: i32 = 2340;
const OBSIDIAN_STATE: i32 = 3369;
const GLOWSTONE_STATE: i32 = 7016;
const GRANITE_STATE: i32 = 2;
const POLISHED_GRANITE_STATE: i32 = 3;
const DIORITE_STATE: i32 = 4;
const POLISHED_DIORITE_STATE: i32 = 5;
const ANDESITE_STATE: i32 = 6;
const POLISHED_ANDESITE_STATE: i32 = 7;
const COARSE_DIRT_STATE: i32 = 11;
const SPRUCE_PLANKS_STATE: i32 = 16;
const BIRCH_PLANKS_STATE: i32 = 17;
const JUNGLE_PLANKS_STATE: i32 = 18;
const ACACIA_PLANKS_STATE: i32 = 19;
const CHERRY_PLANKS_STATE: i32 = 20;
const DARK_OAK_PLANKS_STATE: i32 = 21;
const PALE_OAK_PLANKS_STATE: i32 = 25;
const MANGROVE_PLANKS_STATE: i32 = 26;
const BAMBOO_PLANKS_STATE: i32 = 27;
const BAMBOO_MOSAIC_STATE: i32 = 28;
const RED_SAND_STATE: i32 = 123;
const GOLD_ORE_STATE: i32 = 129;
const DEEPSLATE_GOLD_ORE_STATE: i32 = 130;
const IRON_ORE_STATE: i32 = 131;
const DEEPSLATE_IRON_ORE_STATE: i32 = 132;
const COAL_ORE_STATE: i32 = 133;
const DEEPSLATE_COAL_ORE_STATE: i32 = 134;
const NETHER_GOLD_ORE_STATE: i32 = 135;
const SPONGE_STATE: i32 = 560;
const WET_SPONGE_STATE: i32 = 561;
const LAPIS_ORE_STATE: i32 = 563;
const DEEPSLATE_LAPIS_ORE_STATE: i32 = 564;
const LAPIS_BLOCK_STATE: i32 = 565;
const SANDSTONE_STATE: i32 = 578;
const CHISELED_SANDSTONE_STATE: i32 = 579;
const CUT_SANDSTONE_STATE: i32 = 580;
const ORANGE_WOOL_STATE: i32 = 2294;
const MAGENTA_WOOL_STATE: i32 = 2295;
const LIGHT_BLUE_WOOL_STATE: i32 = 2296;
const YELLOW_WOOL_STATE: i32 = 2297;
const LIME_WOOL_STATE: i32 = 2298;
const PINK_WOOL_STATE: i32 = 2299;
const GRAY_WOOL_STATE: i32 = 2300;
const LIGHT_GRAY_WOOL_STATE: i32 = 2301;
const CYAN_WOOL_STATE: i32 = 2302;
const PURPLE_WOOL_STATE: i32 = 2303;
const BLUE_WOOL_STATE: i32 = 2304;
const BROWN_WOOL_STATE: i32 = 2305;
const GREEN_WOOL_STATE: i32 = 2306;
const RED_WOOL_STATE: i32 = 2307;
const BLACK_WOOL_STATE: i32 = 2308;
const GOLD_BLOCK_STATE: i32 = 2338;
const IRON_BLOCK_STATE: i32 = 2339;
const BOOKSHELF_STATE: i32 = 2343;
const MOSSY_COBBLESTONE_STATE: i32 = 3368;
const DIAMOND_ORE_STATE: i32 = 5307;
const DEEPSLATE_DIAMOND_ORE_STATE: i32 = 5308;
const DIAMOND_BLOCK_STATE: i32 = 5309;
const CRAFTING_TABLE_STATE: i32 = 5310;
const CLAY_STATE: i32 = 6946;
const NETHERRACK_STATE: i32 = 6997;
const SOUL_SAND_STATE: i32 = 6998;
const SOUL_SOIL_STATE: i32 = 6999;
const WHITE_STAINED_GLASS_STATE: i32 = 7098;
const ORANGE_STAINED_GLASS_STATE: i32 = 7099;
const MAGENTA_STAINED_GLASS_STATE: i32 = 7100;
const LIGHT_BLUE_STAINED_GLASS_STATE: i32 = 7101;
const YELLOW_STAINED_GLASS_STATE: i32 = 7102;
const LIME_STAINED_GLASS_STATE: i32 = 7103;
const PINK_STAINED_GLASS_STATE: i32 = 7104;
const GRAY_STAINED_GLASS_STATE: i32 = 7105;
const LIGHT_GRAY_STAINED_GLASS_STATE: i32 = 7106;
const CYAN_STAINED_GLASS_STATE: i32 = 7107;
const PURPLE_STAINED_GLASS_STATE: i32 = 7108;
const BLUE_STAINED_GLASS_STATE: i32 = 7109;
const BROWN_STAINED_GLASS_STATE: i32 = 7110;
const GREEN_STAINED_GLASS_STATE: i32 = 7111;
const RED_STAINED_GLASS_STATE: i32 = 7112;
const BLACK_STAINED_GLASS_STATE: i32 = 7113;
const STONE_BRICKS_STATE: i32 = 7754;
const MOSSY_STONE_BRICKS_STATE: i32 = 7755;
const CRACKED_STONE_BRICKS_STATE: i32 = 7756;
const CHISELED_STONE_BRICKS_STATE: i32 = 7757;
const PACKED_MUD_STATE: i32 = 7758;
const MUD_BRICKS_STATE: i32 = 7759;
const PUMPKIN_STATE: i32 = 8332;
const MELON_STATE: i32 = 8333;
const RESIN_BLOCK_STATE: i32 = 8921;
const RESIN_BRICKS_STATE: i32 = 8922;
const CHISELED_RESIN_BRICKS_STATE: i32 = 9333;
const NETHER_BRICKS_STATE: i32 = 9334;
const ENCHANTING_TABLE_STATE: i32 = 9451;
const END_STONE_STATE: i32 = 9477;
const EMERALD_ORE_STATE: i32 = 9573;
const DEEPSLATE_EMERALD_ORE_STATE: i32 = 9574;
const EMERALD_BLOCK_STATE: i32 = 9727;
const REDSTONE_BLOCK_STATE: i32 = 11311;
const NETHER_QUARTZ_ORE_STATE: i32 = 11312;
const QUARTZ_BLOCK_STATE: i32 = 11323;
const CHISELED_QUARTZ_BLOCK_STATE: i32 = 11324;
const WHITE_TERRACOTTA_STATE: i32 = 11444;
const ORANGE_TERRACOTTA_STATE: i32 = 11445;
const MAGENTA_TERRACOTTA_STATE: i32 = 11446;
const LIGHT_BLUE_TERRACOTTA_STATE: i32 = 11447;
const YELLOW_TERRACOTTA_STATE: i32 = 11448;
const LIME_TERRACOTTA_STATE: i32 = 11449;
const PINK_TERRACOTTA_STATE: i32 = 11450;
const GRAY_TERRACOTTA_STATE: i32 = 11451;
const LIGHT_GRAY_TERRACOTTA_STATE: i32 = 11452;
const CYAN_TERRACOTTA_STATE: i32 = 11453;
const PURPLE_TERRACOTTA_STATE: i32 = 11454;
const BLUE_TERRACOTTA_STATE: i32 = 11455;
const BROWN_TERRACOTTA_STATE: i32 = 11456;
const GREEN_TERRACOTTA_STATE: i32 = 11457;
const RED_TERRACOTTA_STATE: i32 = 11458;
const BLACK_TERRACOTTA_STATE: i32 = 11459;
const PRISMARINE_STATE: i32 = 12631;
const PRISMARINE_BRICKS_STATE: i32 = 12632;
const DARK_PRISMARINE_STATE: i32 = 12633;
const SEA_LANTERN_STATE: i32 = 12892;
const TERRACOTTA_STATE: i32 = 12912;
const COAL_BLOCK_STATE: i32 = 12913;
const PACKED_ICE_STATE: i32 = 12914;
const RED_SANDSTONE_STATE: i32 = 13247;
const CHISELED_RED_SANDSTONE_STATE: i32 = 13248;
const CUT_RED_SANDSTONE_STATE: i32 = 13249;
const SMOOTH_STONE_STATE: i32 = 13480;
const SMOOTH_SANDSTONE_STATE: i32 = 13481;
const SMOOTH_QUARTZ_STATE: i32 = 13482;
const SMOOTH_RED_SANDSTONE_STATE: i32 = 13483;
const PURPUR_BLOCK_STATE: i32 = 14712;
const END_STONE_BRICKS_STATE: i32 = 14796;
const NETHER_WART_BLOCK_STATE: i32 = 14846;
const RED_NETHER_BRICKS_STATE: i32 = 14847;
const WHITE_CONCRETE_STATE: i32 = 15030;
const ORANGE_CONCRETE_STATE: i32 = 15031;
const MAGENTA_CONCRETE_STATE: i32 = 15032;
const LIGHT_BLUE_CONCRETE_STATE: i32 = 15033;
const YELLOW_CONCRETE_STATE: i32 = 15034;
const LIME_CONCRETE_STATE: i32 = 15035;
const PINK_CONCRETE_STATE: i32 = 15036;
const GRAY_CONCRETE_STATE: i32 = 15037;
const LIGHT_GRAY_CONCRETE_STATE: i32 = 15038;
const CYAN_CONCRETE_STATE: i32 = 15039;
const PURPLE_CONCRETE_STATE: i32 = 15040;
const BLUE_CONCRETE_STATE: i32 = 15041;
const BROWN_CONCRETE_STATE: i32 = 15042;
const GREEN_CONCRETE_STATE: i32 = 15043;
const RED_CONCRETE_STATE: i32 = 15044;
const BLACK_CONCRETE_STATE: i32 = 15045;
const DRIED_KELP_BLOCK_STATE: i32 = 15089;
const DEAD_TUBE_CORAL_BLOCK_STATE: i32 = 15137;
const DEAD_BRAIN_CORAL_BLOCK_STATE: i32 = 15138;
const DEAD_BUBBLE_CORAL_BLOCK_STATE: i32 = 15139;
const DEAD_FIRE_CORAL_BLOCK_STATE: i32 = 15140;
const DEAD_HORN_CORAL_BLOCK_STATE: i32 = 15141;
const BLUE_ICE_STATE: i32 = 15275;
const CARTOGRAPHY_TABLE_STATE: i32 = 20770;
const FLETCHING_TABLE_STATE: i32 = 20771;
const SMITHING_TABLE_STATE: i32 = 20800;
const WARPED_NYLIUM_STATE: i32 = 20957;
const WARPED_WART_BLOCK_STATE: i32 = 20959;
const CRIMSON_NYLIUM_STATE: i32 = 20974;
const SHROOMLIGHT_STATE: i32 = 20976;
const CRIMSON_PLANKS_STATE: i32 = 21032;
const WARPED_PLANKS_STATE: i32 = 21033;
const HONEYCOMB_BLOCK_STATE: i32 = 21817;
const NETHERITE_BLOCK_STATE: i32 = 21818;
const ANCIENT_DEBRIS_STATE: i32 = 21819;
const CRYING_OBSIDIAN_STATE: i32 = 21820;
const LODESTONE_STATE: i32 = 21830;
const BLACKSTONE_STATE: i32 = 21831;
const POLISHED_BLACKSTONE_STATE: i32 = 22242;
const POLISHED_BLACKSTONE_BRICKS_STATE: i32 = 22243;
const CRACKED_POLISHED_BLACKSTONE_BRICKS_STATE: i32 = 22244;
const CHISELED_POLISHED_BLACKSTONE_STATE: i32 = 22245;
const GILDED_BLACKSTONE_STATE: i32 = 22656;
const CHISELED_NETHER_BRICKS_STATE: i32 = 23093;
const CRACKED_NETHER_BRICKS_STATE: i32 = 23094;
const QUARTZ_BRICKS_STATE: i32 = 23095;
const AMETHYST_BLOCK_STATE: i32 = 23402;
const TUFF_STATE: i32 = 23452;
const POLISHED_TUFF_STATE: i32 = 23863;
const CHISELED_TUFF_STATE: i32 = 24274;
const TUFF_BRICKS_STATE: i32 = 24275;
const CHISELED_TUFF_BRICKS_STATE: i32 = 24686;
const CALCITE_STATE: i32 = 24687;
const TINTED_GLASS_STATE: i32 = 24688;
const COPPER_BLOCK_STATE: i32 = 25309;
const EXPOSED_COPPER_STATE: i32 = 25310;
const WEATHERED_COPPER_STATE: i32 = 25311;
const OXIDIZED_COPPER_STATE: i32 = 25312;
const COPPER_ORE_STATE: i32 = 25313;
const DEEPSLATE_COPPER_ORE_STATE: i32 = 25314;
const OXIDIZED_CUT_COPPER_STATE: i32 = 25315;
const WEATHERED_CUT_COPPER_STATE: i32 = 25316;
const EXPOSED_CUT_COPPER_STATE: i32 = 25317;
const CUT_COPPER_STATE: i32 = 25318;
const OXIDIZED_CHISELED_COPPER_STATE: i32 = 25319;
const WEATHERED_CHISELED_COPPER_STATE: i32 = 25320;
const EXPOSED_CHISELED_COPPER_STATE: i32 = 25321;
const CHISELED_COPPER_STATE: i32 = 25322;
const WAXED_OXIDIZED_CHISELED_COPPER_STATE: i32 = 25323;
const WAXED_WEATHERED_CHISELED_COPPER_STATE: i32 = 25324;
const WAXED_EXPOSED_CHISELED_COPPER_STATE: i32 = 25325;
const WAXED_CHISELED_COPPER_STATE: i32 = 25326;
const WAXED_COPPER_BLOCK_STATE: i32 = 25671;
const WAXED_WEATHERED_COPPER_STATE: i32 = 25672;
const WAXED_EXPOSED_COPPER_STATE: i32 = 25673;
const WAXED_OXIDIZED_COPPER_STATE: i32 = 25674;
const WAXED_OXIDIZED_CUT_COPPER_STATE: i32 = 25675;
const WAXED_WEATHERED_CUT_COPPER_STATE: i32 = 25676;
const WAXED_EXPOSED_CUT_COPPER_STATE: i32 = 25677;
const WAXED_CUT_COPPER_STATE: i32 = 25678;
const DRIPSTONE_BLOCK_STATE: i32 = 27755;
const MOSS_BLOCK_STATE: i32 = 27862;
const ROOTED_DIRT_STATE: i32 = 27921;
const MUD_STATE: i32 = 27922;
const COBBLED_DEEPSLATE_STATE: i32 = 27926;
const POLISHED_DEEPSLATE_STATE: i32 = 28337;
const DEEPSLATE_TILES_STATE: i32 = 28748;
const DEEPSLATE_BRICKS_STATE: i32 = 29159;
const CHISELED_DEEPSLATE_STATE: i32 = 29570;
const CRACKED_DEEPSLATE_BRICKS_STATE: i32 = 29571;
const CRACKED_DEEPSLATE_TILES_STATE: i32 = 29572;
const SMOOTH_BASALT_STATE: i32 = 29576;
const RAW_IRON_BLOCK_STATE: i32 = 29577;
const RAW_COPPER_BLOCK_STATE: i32 = 29578;
const RAW_GOLD_BLOCK_STATE: i32 = 29579;
const PALE_MOSS_BLOCK_STATE: i32 = 29703;
const PLAINS_BIOME: i32 = 0;

const LIGHT_SECTION_COUNT: usize = SECTION_COUNT + 2;
const SKY_FULL: u8 = 0xFF;

#[must_use]
pub fn block_to_state(block: Block) -> i32 {
    match block {
        Block::Air => AIR_STATE,
        Block::Stone => STONE_STATE,
        Block::Dirt => DIRT_STATE,
        Block::GrassBlock => GRASS_STATE,
        Block::Cobblestone => COBBLESTONE_STATE,
        Block::OakPlanks => OAK_PLANKS_STATE,
        Block::Sand => SAND_STATE,
        Block::Gravel => GRAVEL_STATE,
        Block::OakLog => OAK_LOG_STATE,
        Block::Glass => GLASS_STATE,
        Block::WhiteWool => WHITE_WOOL_STATE,
        Block::Bricks => BRICKS_STATE,
        Block::Obsidian => OBSIDIAN_STATE,
        Block::Glowstone => GLOWSTONE_STATE,
        Block::Granite => GRANITE_STATE,
        Block::PolishedGranite => POLISHED_GRANITE_STATE,
        Block::Diorite => DIORITE_STATE,
        Block::PolishedDiorite => POLISHED_DIORITE_STATE,
        Block::Andesite => ANDESITE_STATE,
        Block::PolishedAndesite => POLISHED_ANDESITE_STATE,
        Block::CoarseDirt => COARSE_DIRT_STATE,
        Block::SprucePlanks => SPRUCE_PLANKS_STATE,
        Block::BirchPlanks => BIRCH_PLANKS_STATE,
        Block::JunglePlanks => JUNGLE_PLANKS_STATE,
        Block::AcaciaPlanks => ACACIA_PLANKS_STATE,
        Block::CherryPlanks => CHERRY_PLANKS_STATE,
        Block::DarkOakPlanks => DARK_OAK_PLANKS_STATE,
        Block::PaleOakPlanks => PALE_OAK_PLANKS_STATE,
        Block::MangrovePlanks => MANGROVE_PLANKS_STATE,
        Block::BambooPlanks => BAMBOO_PLANKS_STATE,
        Block::BambooMosaic => BAMBOO_MOSAIC_STATE,
        Block::RedSand => RED_SAND_STATE,
        Block::GoldOre => GOLD_ORE_STATE,
        Block::DeepslateGoldOre => DEEPSLATE_GOLD_ORE_STATE,
        Block::IronOre => IRON_ORE_STATE,
        Block::DeepslateIronOre => DEEPSLATE_IRON_ORE_STATE,
        Block::CoalOre => COAL_ORE_STATE,
        Block::DeepslateCoalOre => DEEPSLATE_COAL_ORE_STATE,
        Block::NetherGoldOre => NETHER_GOLD_ORE_STATE,
        Block::Sponge => SPONGE_STATE,
        Block::WetSponge => WET_SPONGE_STATE,
        Block::LapisOre => LAPIS_ORE_STATE,
        Block::DeepslateLapisOre => DEEPSLATE_LAPIS_ORE_STATE,
        Block::LapisBlock => LAPIS_BLOCK_STATE,
        Block::Sandstone => SANDSTONE_STATE,
        Block::ChiseledSandstone => CHISELED_SANDSTONE_STATE,
        Block::CutSandstone => CUT_SANDSTONE_STATE,
        Block::OrangeWool => ORANGE_WOOL_STATE,
        Block::MagentaWool => MAGENTA_WOOL_STATE,
        Block::LightBlueWool => LIGHT_BLUE_WOOL_STATE,
        Block::YellowWool => YELLOW_WOOL_STATE,
        Block::LimeWool => LIME_WOOL_STATE,
        Block::PinkWool => PINK_WOOL_STATE,
        Block::GrayWool => GRAY_WOOL_STATE,
        Block::LightGrayWool => LIGHT_GRAY_WOOL_STATE,
        Block::CyanWool => CYAN_WOOL_STATE,
        Block::PurpleWool => PURPLE_WOOL_STATE,
        Block::BlueWool => BLUE_WOOL_STATE,
        Block::BrownWool => BROWN_WOOL_STATE,
        Block::GreenWool => GREEN_WOOL_STATE,
        Block::RedWool => RED_WOOL_STATE,
        Block::BlackWool => BLACK_WOOL_STATE,
        Block::GoldBlock => GOLD_BLOCK_STATE,
        Block::IronBlock => IRON_BLOCK_STATE,
        Block::Bookshelf => BOOKSHELF_STATE,
        Block::MossyCobblestone => MOSSY_COBBLESTONE_STATE,
        Block::DiamondOre => DIAMOND_ORE_STATE,
        Block::DeepslateDiamondOre => DEEPSLATE_DIAMOND_ORE_STATE,
        Block::DiamondBlock => DIAMOND_BLOCK_STATE,
        Block::CraftingTable => CRAFTING_TABLE_STATE,
        Block::Clay => CLAY_STATE,
        Block::Netherrack => NETHERRACK_STATE,
        Block::SoulSand => SOUL_SAND_STATE,
        Block::SoulSoil => SOUL_SOIL_STATE,
        Block::WhiteStainedGlass => WHITE_STAINED_GLASS_STATE,
        Block::OrangeStainedGlass => ORANGE_STAINED_GLASS_STATE,
        Block::MagentaStainedGlass => MAGENTA_STAINED_GLASS_STATE,
        Block::LightBlueStainedGlass => LIGHT_BLUE_STAINED_GLASS_STATE,
        Block::YellowStainedGlass => YELLOW_STAINED_GLASS_STATE,
        Block::LimeStainedGlass => LIME_STAINED_GLASS_STATE,
        Block::PinkStainedGlass => PINK_STAINED_GLASS_STATE,
        Block::GrayStainedGlass => GRAY_STAINED_GLASS_STATE,
        Block::LightGrayStainedGlass => LIGHT_GRAY_STAINED_GLASS_STATE,
        Block::CyanStainedGlass => CYAN_STAINED_GLASS_STATE,
        Block::PurpleStainedGlass => PURPLE_STAINED_GLASS_STATE,
        Block::BlueStainedGlass => BLUE_STAINED_GLASS_STATE,
        Block::BrownStainedGlass => BROWN_STAINED_GLASS_STATE,
        Block::GreenStainedGlass => GREEN_STAINED_GLASS_STATE,
        Block::RedStainedGlass => RED_STAINED_GLASS_STATE,
        Block::BlackStainedGlass => BLACK_STAINED_GLASS_STATE,
        Block::StoneBricks => STONE_BRICKS_STATE,
        Block::MossyStoneBricks => MOSSY_STONE_BRICKS_STATE,
        Block::CrackedStoneBricks => CRACKED_STONE_BRICKS_STATE,
        Block::ChiseledStoneBricks => CHISELED_STONE_BRICKS_STATE,
        Block::PackedMud => PACKED_MUD_STATE,
        Block::MudBricks => MUD_BRICKS_STATE,
        Block::Pumpkin => PUMPKIN_STATE,
        Block::Melon => MELON_STATE,
        Block::ResinBlock => RESIN_BLOCK_STATE,
        Block::ResinBricks => RESIN_BRICKS_STATE,
        Block::ChiseledResinBricks => CHISELED_RESIN_BRICKS_STATE,
        Block::NetherBricks => NETHER_BRICKS_STATE,
        Block::EnchantingTable => ENCHANTING_TABLE_STATE,
        Block::EndStone => END_STONE_STATE,
        Block::EmeraldOre => EMERALD_ORE_STATE,
        Block::DeepslateEmeraldOre => DEEPSLATE_EMERALD_ORE_STATE,
        Block::EmeraldBlock => EMERALD_BLOCK_STATE,
        Block::RedstoneBlock => REDSTONE_BLOCK_STATE,
        Block::NetherQuartzOre => NETHER_QUARTZ_ORE_STATE,
        Block::QuartzBlock => QUARTZ_BLOCK_STATE,
        Block::ChiseledQuartzBlock => CHISELED_QUARTZ_BLOCK_STATE,
        Block::WhiteTerracotta => WHITE_TERRACOTTA_STATE,
        Block::OrangeTerracotta => ORANGE_TERRACOTTA_STATE,
        Block::MagentaTerracotta => MAGENTA_TERRACOTTA_STATE,
        Block::LightBlueTerracotta => LIGHT_BLUE_TERRACOTTA_STATE,
        Block::YellowTerracotta => YELLOW_TERRACOTTA_STATE,
        Block::LimeTerracotta => LIME_TERRACOTTA_STATE,
        Block::PinkTerracotta => PINK_TERRACOTTA_STATE,
        Block::GrayTerracotta => GRAY_TERRACOTTA_STATE,
        Block::LightGrayTerracotta => LIGHT_GRAY_TERRACOTTA_STATE,
        Block::CyanTerracotta => CYAN_TERRACOTTA_STATE,
        Block::PurpleTerracotta => PURPLE_TERRACOTTA_STATE,
        Block::BlueTerracotta => BLUE_TERRACOTTA_STATE,
        Block::BrownTerracotta => BROWN_TERRACOTTA_STATE,
        Block::GreenTerracotta => GREEN_TERRACOTTA_STATE,
        Block::RedTerracotta => RED_TERRACOTTA_STATE,
        Block::BlackTerracotta => BLACK_TERRACOTTA_STATE,
        Block::Prismarine => PRISMARINE_STATE,
        Block::PrismarineBricks => PRISMARINE_BRICKS_STATE,
        Block::DarkPrismarine => DARK_PRISMARINE_STATE,
        Block::SeaLantern => SEA_LANTERN_STATE,
        Block::Terracotta => TERRACOTTA_STATE,
        Block::CoalBlock => COAL_BLOCK_STATE,
        Block::PackedIce => PACKED_ICE_STATE,
        Block::RedSandstone => RED_SANDSTONE_STATE,
        Block::ChiseledRedSandstone => CHISELED_RED_SANDSTONE_STATE,
        Block::CutRedSandstone => CUT_RED_SANDSTONE_STATE,
        Block::SmoothStone => SMOOTH_STONE_STATE,
        Block::SmoothSandstone => SMOOTH_SANDSTONE_STATE,
        Block::SmoothQuartz => SMOOTH_QUARTZ_STATE,
        Block::SmoothRedSandstone => SMOOTH_RED_SANDSTONE_STATE,
        Block::PurpurBlock => PURPUR_BLOCK_STATE,
        Block::EndStoneBricks => END_STONE_BRICKS_STATE,
        Block::NetherWartBlock => NETHER_WART_BLOCK_STATE,
        Block::RedNetherBricks => RED_NETHER_BRICKS_STATE,
        Block::WhiteConcrete => WHITE_CONCRETE_STATE,
        Block::OrangeConcrete => ORANGE_CONCRETE_STATE,
        Block::MagentaConcrete => MAGENTA_CONCRETE_STATE,
        Block::LightBlueConcrete => LIGHT_BLUE_CONCRETE_STATE,
        Block::YellowConcrete => YELLOW_CONCRETE_STATE,
        Block::LimeConcrete => LIME_CONCRETE_STATE,
        Block::PinkConcrete => PINK_CONCRETE_STATE,
        Block::GrayConcrete => GRAY_CONCRETE_STATE,
        Block::LightGrayConcrete => LIGHT_GRAY_CONCRETE_STATE,
        Block::CyanConcrete => CYAN_CONCRETE_STATE,
        Block::PurpleConcrete => PURPLE_CONCRETE_STATE,
        Block::BlueConcrete => BLUE_CONCRETE_STATE,
        Block::BrownConcrete => BROWN_CONCRETE_STATE,
        Block::GreenConcrete => GREEN_CONCRETE_STATE,
        Block::RedConcrete => RED_CONCRETE_STATE,
        Block::BlackConcrete => BLACK_CONCRETE_STATE,
        Block::DriedKelpBlock => DRIED_KELP_BLOCK_STATE,
        Block::DeadTubeCoralBlock => DEAD_TUBE_CORAL_BLOCK_STATE,
        Block::DeadBrainCoralBlock => DEAD_BRAIN_CORAL_BLOCK_STATE,
        Block::DeadBubbleCoralBlock => DEAD_BUBBLE_CORAL_BLOCK_STATE,
        Block::DeadFireCoralBlock => DEAD_FIRE_CORAL_BLOCK_STATE,
        Block::DeadHornCoralBlock => DEAD_HORN_CORAL_BLOCK_STATE,
        Block::BlueIce => BLUE_ICE_STATE,
        Block::CartographyTable => CARTOGRAPHY_TABLE_STATE,
        Block::FletchingTable => FLETCHING_TABLE_STATE,
        Block::SmithingTable => SMITHING_TABLE_STATE,
        Block::WarpedNylium => WARPED_NYLIUM_STATE,
        Block::WarpedWartBlock => WARPED_WART_BLOCK_STATE,
        Block::CrimsonNylium => CRIMSON_NYLIUM_STATE,
        Block::Shroomlight => SHROOMLIGHT_STATE,
        Block::CrimsonPlanks => CRIMSON_PLANKS_STATE,
        Block::WarpedPlanks => WARPED_PLANKS_STATE,
        Block::HoneycombBlock => HONEYCOMB_BLOCK_STATE,
        Block::NetheriteBlock => NETHERITE_BLOCK_STATE,
        Block::AncientDebris => ANCIENT_DEBRIS_STATE,
        Block::CryingObsidian => CRYING_OBSIDIAN_STATE,
        Block::Lodestone => LODESTONE_STATE,
        Block::Blackstone => BLACKSTONE_STATE,
        Block::PolishedBlackstone => POLISHED_BLACKSTONE_STATE,
        Block::PolishedBlackstoneBricks => POLISHED_BLACKSTONE_BRICKS_STATE,
        Block::CrackedPolishedBlackstoneBricks => CRACKED_POLISHED_BLACKSTONE_BRICKS_STATE,
        Block::ChiseledPolishedBlackstone => CHISELED_POLISHED_BLACKSTONE_STATE,
        Block::GildedBlackstone => GILDED_BLACKSTONE_STATE,
        Block::ChiseledNetherBricks => CHISELED_NETHER_BRICKS_STATE,
        Block::CrackedNetherBricks => CRACKED_NETHER_BRICKS_STATE,
        Block::QuartzBricks => QUARTZ_BRICKS_STATE,
        Block::AmethystBlock => AMETHYST_BLOCK_STATE,
        Block::Tuff => TUFF_STATE,
        Block::PolishedTuff => POLISHED_TUFF_STATE,
        Block::ChiseledTuff => CHISELED_TUFF_STATE,
        Block::TuffBricks => TUFF_BRICKS_STATE,
        Block::ChiseledTuffBricks => CHISELED_TUFF_BRICKS_STATE,
        Block::Calcite => CALCITE_STATE,
        Block::TintedGlass => TINTED_GLASS_STATE,
        Block::CopperBlock => COPPER_BLOCK_STATE,
        Block::ExposedCopper => EXPOSED_COPPER_STATE,
        Block::WeatheredCopper => WEATHERED_COPPER_STATE,
        Block::OxidizedCopper => OXIDIZED_COPPER_STATE,
        Block::CopperOre => COPPER_ORE_STATE,
        Block::DeepslateCopperOre => DEEPSLATE_COPPER_ORE_STATE,
        Block::OxidizedCutCopper => OXIDIZED_CUT_COPPER_STATE,
        Block::WeatheredCutCopper => WEATHERED_CUT_COPPER_STATE,
        Block::ExposedCutCopper => EXPOSED_CUT_COPPER_STATE,
        Block::CutCopper => CUT_COPPER_STATE,
        Block::OxidizedChiseledCopper => OXIDIZED_CHISELED_COPPER_STATE,
        Block::WeatheredChiseledCopper => WEATHERED_CHISELED_COPPER_STATE,
        Block::ExposedChiseledCopper => EXPOSED_CHISELED_COPPER_STATE,
        Block::ChiseledCopper => CHISELED_COPPER_STATE,
        Block::WaxedOxidizedChiseledCopper => WAXED_OXIDIZED_CHISELED_COPPER_STATE,
        Block::WaxedWeatheredChiseledCopper => WAXED_WEATHERED_CHISELED_COPPER_STATE,
        Block::WaxedExposedChiseledCopper => WAXED_EXPOSED_CHISELED_COPPER_STATE,
        Block::WaxedChiseledCopper => WAXED_CHISELED_COPPER_STATE,
        Block::WaxedCopperBlock => WAXED_COPPER_BLOCK_STATE,
        Block::WaxedWeatheredCopper => WAXED_WEATHERED_COPPER_STATE,
        Block::WaxedExposedCopper => WAXED_EXPOSED_COPPER_STATE,
        Block::WaxedOxidizedCopper => WAXED_OXIDIZED_COPPER_STATE,
        Block::WaxedOxidizedCutCopper => WAXED_OXIDIZED_CUT_COPPER_STATE,
        Block::WaxedWeatheredCutCopper => WAXED_WEATHERED_CUT_COPPER_STATE,
        Block::WaxedExposedCutCopper => WAXED_EXPOSED_CUT_COPPER_STATE,
        Block::WaxedCutCopper => WAXED_CUT_COPPER_STATE,
        Block::DripstoneBlock => DRIPSTONE_BLOCK_STATE,
        Block::MossBlock => MOSS_BLOCK_STATE,
        Block::RootedDirt => ROOTED_DIRT_STATE,
        Block::Mud => MUD_STATE,
        Block::CobbledDeepslate => COBBLED_DEEPSLATE_STATE,
        Block::PolishedDeepslate => POLISHED_DEEPSLATE_STATE,
        Block::DeepslateTiles => DEEPSLATE_TILES_STATE,
        Block::DeepslateBricks => DEEPSLATE_BRICKS_STATE,
        Block::ChiseledDeepslate => CHISELED_DEEPSLATE_STATE,
        Block::CrackedDeepslateBricks => CRACKED_DEEPSLATE_BRICKS_STATE,
        Block::CrackedDeepslateTiles => CRACKED_DEEPSLATE_TILES_STATE,
        Block::SmoothBasalt => SMOOTH_BASALT_STATE,
        Block::RawIronBlock => RAW_IRON_BLOCK_STATE,
        Block::RawCopperBlock => RAW_COPPER_BLOCK_STATE,
        Block::RawGoldBlock => RAW_GOLD_BLOCK_STATE,
        Block::PaleMossBlock => PALE_MOSS_BLOCK_STATE,
    }
}

fn pack_bits(values: &[u64], bits_per_entry: u32) -> Vec<u64> {
    if bits_per_entry == 0 {
        return Vec::new();
    }
    let entries_per_long = 64 / bits_per_entry as usize;
    let longs = values.len().div_ceil(entries_per_long);
    let mut out = vec![0u64; longs];
    for (i, &value) in values.iter().enumerate() {
        out[i / entries_per_long] |= value << ((i % entries_per_long) as u32 * bits_per_entry);
    }
    out
}

fn bits_needed(count: usize) -> u32 {
    let mut bits = 0u32;
    while (1usize << bits) < count {
        bits += 1;
    }
    bits
}

fn write_heightmaps(body: &mut Writer) {
    let bits = bits_needed((WORLD_HEIGHT + 1) as usize);
    let top = (SURFACE_Y - WORLD_MIN_Y + 1) as u64;
    let longs = pack_bits(&vec![top; 256], bits);
    body.write_varint(2);
    for kind in [1, 4] {
        body.write_varint(kind);
        body.write_varint(longs.len() as i32);
        for long in &longs {
            body.write_u64(*long);
        }
    }
}

fn write_block_container(body: &mut Writer, states: &[u16; 4096]) {
    let mut palette: Vec<i32> = Vec::new();
    let mut indices = vec![0u64; 4096];
    let mut non_air = 0i16;
    for (i, &local) in states.iter().enumerate() {
        let global = block_to_state(Block::from_id(local).unwrap_or(Block::Air));
        if global != AIR_STATE {
            non_air += 1;
        }
        let slot = match palette.iter().position(|&entry| entry == global) {
            Some(slot) => slot,
            None => {
                palette.push(global);
                palette.len() - 1
            }
        };
        indices[i] = slot as u64;
    }
    body.write_i16(non_air);
    body.write_i16(0);
    if palette.len() == 1 {
        body.write_u8(0);
        body.write_varint(palette[0]);
        return;
    }
    let bits = bits_needed(palette.len()).clamp(4, 8);
    body.write_u8(bits as u8);
    body.write_varint(palette.len() as i32);
    for entry in &palette {
        body.write_varint(*entry);
    }
    for long in pack_bits(&indices, bits) {
        body.write_u64(long);
    }
}

fn write_biome_container(body: &mut Writer) {
    body.write_u8(0);
    body.write_varint(PLAINS_BIOME);
}

fn write_light(body: &mut Writer) {
    let full_mask = if LIGHT_SECTION_COUNT >= 64 {
        u64::MAX
    } else {
        (1u64 << LIGHT_SECTION_COUNT) - 1
    };
    body.write_varint(1);
    body.write_u64(full_mask);
    body.write_varint(0);
    body.write_varint(0);
    body.write_varint(1);
    body.write_u64(full_mask);
    body.write_varint(LIGHT_SECTION_COUNT as i32);
    for _ in 0..LIGHT_SECTION_COUNT {
        body.write_varint(2048);
        body.write_bytes(&[SKY_FULL; 2048]);
    }
    body.write_varint(0);
}

#[must_use]
pub fn encode_chunk_data(chunk: &Chunk) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_i32(chunk.pos().x);
    body.write_i32(chunk.pos().z);
    write_heightmaps(&mut body);
    let mut data = Writer::new();
    for index in 0..chunk.section_count() {
        if let Some(section) = chunk.section(index) {
            write_block_container(&mut data, section.states());
            write_biome_container(&mut data);
        }
    }
    let data = data.into_bytes();
    body.write_varint(data.len() as i32);
    body.write_bytes(&data);
    body.write_varint(0);
    write_light(&mut body);
    encode_packet(0x2D, &body.into_bytes())
}

#[must_use]
pub fn encode_set_center(x: i32, z: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(x);
    body.write_varint(z);
    encode_packet(0x5E, &body.into_bytes())
}

#[must_use]
pub fn encode_batch_start() -> Vec<u8> {
    encode_packet(0x0C, &[])
}

#[must_use]
pub fn encode_unload_chunk(x: i32, z: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_i32(x);
    body.write_i32(z);
    encode_packet(0x25, &body.into_bytes())
}

#[must_use]
pub fn encode_batch_finished(batch_size: i32) -> Vec<u8> {
    let mut body = Writer::new();
    body.write_varint(batch_size);
    encode_packet(0x0B, &body.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::super::proto::{decode_varint_prefix, Reader};
    use super::*;

    fn decode_frame(bytes: &[u8]) -> (i32, Vec<u8>) {
        let (len, header) = decode_varint_prefix(bytes).unwrap().unwrap();
        let mut reader = Reader::new(&bytes[header..header + len as usize]);
        let id = reader.read_varint().unwrap();
        (id, bytes[header + reader.position()..].to_vec())
    }

    #[test]
    fn maps_blocks_to_vanilla_states() {
        assert_eq!(block_to_state(Block::Air), 0);
        assert_eq!(block_to_state(Block::Stone), 1);
        assert_eq!(block_to_state(Block::GrassBlock), 9); // dry (default)
        assert_eq!(block_to_state(Block::Dirt), 10);
        assert_eq!(block_to_state(Block::Cobblestone), 14);
        assert_eq!(block_to_state(Block::OakPlanks), 15);
        assert_eq!(block_to_state(Block::Sand), 118);
        assert_eq!(block_to_state(Block::Gravel), 124);
        assert_eq!(block_to_state(Block::OakLog), 137); // axis y
        assert_eq!(block_to_state(Block::Glass), 562);
        assert_eq!(block_to_state(Block::WhiteWool), 2293);
        assert_eq!(block_to_state(Block::Bricks), 2340);
        assert_eq!(block_to_state(Block::Obsidian), 3369);
        assert_eq!(block_to_state(Block::Glowstone), 7016);
        assert_eq!(block_to_state(Block::DiamondOre), 5307);
        assert_eq!(block_to_state(Block::CraftingTable), 5310);
        assert_eq!(block_to_state(Block::MossyCobblestone), 3368);
        assert_eq!(block_to_state(Block::PaleMossBlock), 29703);
    }

    #[test]
    fn packs_bits_lsb_first_with_padding() {
        assert!(pack_bits(&[], 4).is_empty());
        assert!(pack_bits(&[1, 2], 0).is_empty());
        let longs = pack_bits(&[1u64, 2, 3], 4);
        assert_eq!(longs.len(), 1);
        assert_eq!(longs[0], 0x321);
        let longs = pack_bits(&vec![0u64; 256], 9);
        assert_eq!(longs.len(), 37);
    }

    #[test]
    fn chunk_packet_has_expected_shape() {
        let mut chunk = Chunk::new(3, -2);
        chunk.set_block(3, 100, -2, Block::Dirt);
        let bytes = encode_chunk_data(&chunk);
        let (id, payload) = decode_frame(&bytes);
        assert_eq!(id, 0x2D);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_i32().unwrap(), 3);
        assert_eq!(reader.read_i32().unwrap(), -2);

        assert_eq!(reader.read_varint().unwrap(), 2);
        for want in [1, 4] {
            assert_eq!(reader.read_varint().unwrap(), want);
            assert_eq!(reader.read_varint().unwrap(), 37);
            for _ in 0..37 {
                reader.read_u64().unwrap();
            }
        }

        let size = reader.read_varint().unwrap() as usize;
        let data = reader.read_bytes(size).unwrap();
        let mut sections = Reader::new(data);
        for _ in 0..SECTION_COUNT {
            let non_air = sections.read_i16().unwrap();
            assert!(non_air >= 0);
            sections.read_i16().unwrap();
            let bits = sections.read_u8().unwrap();
            if bits == 0 {
                sections.read_varint().unwrap();
            } else {
                assert!((4..=8).contains(&bits));
                let len = sections.read_varint().unwrap() as usize;
                assert!((2..=16).contains(&len));
                for _ in 0..len {
                    sections.read_varint().unwrap();
                }
                let entries_per_long = 64 / bits as usize;
                let longs = 4096usize.div_ceil(entries_per_long);
                for _ in 0..longs {
                    sections.read_u64().unwrap();
                }
            }
            assert_eq!(sections.read_u8().unwrap(), 0);
            assert_eq!(sections.read_varint().unwrap(), PLAINS_BIOME);
        }
        assert_eq!(sections.remaining(), 0);

        assert_eq!(reader.read_varint().unwrap(), 0);

        assert_eq!(reader.read_varint().unwrap(), 1);
        reader.read_u64().unwrap();
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 1);
        reader.read_u64().unwrap();
        assert_eq!(reader.read_varint().unwrap(), LIGHT_SECTION_COUNT as i32);
        for _ in 0..LIGHT_SECTION_COUNT {
            assert_eq!(reader.read_varint().unwrap(), 2048);
            assert_eq!(reader.read_bytes(2048).unwrap().len(), 2048);
        }
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.remaining(), 0);
    }

    #[test]
    fn center_and_batch_packets_encode() {
        let (id, payload) = decode_frame(&encode_set_center(5, -7));
        assert_eq!(id, 0x5E);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 5);
        assert_eq!(reader.read_varint().unwrap(), -7);

        let (id, payload) = decode_frame(&encode_batch_start());
        assert_eq!(id, 0x0C);
        assert!(payload.is_empty());

        let (id, payload) = decode_frame(&encode_unload_chunk(-3, 7));
        assert_eq!(id, 0x25);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_i32().unwrap(), -3);
        assert_eq!(reader.read_i32().unwrap(), 7);
        assert_eq!(reader.remaining(), 0);

        let (id, payload) = decode_frame(&encode_batch_finished(81));
        assert_eq!(id, 0x0B);
        assert_eq!(Reader::new(&payload).read_varint().unwrap(), 81);
    }
}
