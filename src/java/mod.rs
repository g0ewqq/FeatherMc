pub mod chunks;
pub mod entities;
pub mod error;
pub mod metadata;
pub mod nbt;
pub mod packets;
pub mod player;
pub mod proto;
pub mod registries;
pub mod registry;
pub mod session;
pub mod state;

pub use chunks::{
    block_to_state, encode_batch_finished, encode_batch_start, encode_chunk_data,
    encode_set_center, encode_unload_chunk, CHUNK_RADIUS,
};
pub use entities::{
    encode_destroy_entities, encode_entity_metadata, encode_entity_position,
    encode_entity_position_rotation, encode_entity_rotation, encode_head_rotation,
    encode_player_info_add, encode_player_info_remove, encode_spawn_item, encode_spawn_player,
    encode_teleport_entity, ITEM_ENTITY_TYPE, PLAYER_ENTITY_TYPE,
};
pub use error::ProtoError;
pub use metadata::{player_metadata, MetadataEntry, MetadataKind};
pub use nbt::NbtTag;
pub use packets::{
    encode_block_changed_ack, encode_block_update, encode_container_content, encode_container_slot,
    encode_default_spawn, encode_finish_configuration, encode_game_event, encode_health,
    encode_held_slot, encode_keep_alive, encode_known_packs, encode_login_disconnect,
    encode_login_success, encode_play_login, encode_player_abilities, encode_pong, encode_respawn,
    encode_status_response, encode_sync_position, encode_system_chat, encode_update_tags,
    Handshake, LoginStart,
};
pub use player::{GameMode, PlayerSession};
pub use proto::{Reader, Writer, MAX_STRING_BYTES};
pub use registries::{default_registries, ProtocolRegistry, RegistryEntry};
pub use registry::PlayerRegistry;
pub use session::{JavaHandler, ServerStatus};
pub use state::ProtocolState;

pub const PROTOCOL_VERSION: i32 = 776;
pub const VERSION_NAME: &str = "26.2";
