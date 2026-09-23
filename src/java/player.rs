use crate::entity::{Entity, EntityLifecycle};
use crate::inventory::{Inventory, ItemStack};
use crate::java::metadata::{FLAG_SNEAKING, FLAG_SPRINTING, POSE_CROUCHING, POSE_STANDING};
use crate::network::ConnectionId;

pub const DEFAULT_DIMENSION: &str = crate::world::DEFAULT_DIMENSION_ID;
pub const SPAWN_X: f64 = crate::world::SPAWN_X;
pub const SPAWN_Y: f64 = crate::world::SPAWN_Y;
pub const SPAWN_Z: f64 = crate::world::SPAWN_Z;
pub const SPAWN_YAW: f32 = crate::world::SPAWN_YAW;
pub const SPAWN_PITCH: f32 = crate::world::SPAWN_PITCH;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

#[derive(Debug, Clone)]
pub struct PlayerSession {
    pub connection_id: ConnectionId,
    pub name: String,
    pub uuid: Option<u128>,
    pub protocol_version: i32,
    pub entity_id: i32,
    pub session_id: u128,
    pub gamemode: GameMode,
    pub world: String,
    pub dimension: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub on_ground: bool,
    pub sneaking: bool,
    pub sprinting: bool,
    pub inventory: Inventory,
    pub cursor: ItemStack,
}

impl PlayerSession {
    #[must_use]
    pub fn new(
        connection_id: ConnectionId,
        name: String,
        uuid: Option<u128>,
        protocol_version: i32,
        entity_id: i32,
        session_id: u128,
    ) -> Self {
        Self {
            connection_id,
            name,
            uuid,
            protocol_version,
            entity_id,
            session_id,
            gamemode: GameMode::Survival,
            world: crate::world::DEFAULT_WORLD_NAME.to_owned(),
            dimension: DEFAULT_DIMENSION.to_owned(),
            x: SPAWN_X,
            y: SPAWN_Y,
            z: SPAWN_Z,
            yaw: SPAWN_YAW,
            pitch: SPAWN_PITCH,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            on_ground: false,
            sneaking: false,
            sprinting: false,
            inventory: Inventory::starting(),
            cursor: ItemStack::empty(),
        }
    }

    #[must_use]
    pub fn held_item(&self) -> ItemStack {
        self.inventory.held()
    }

    #[must_use]
    pub fn display_uuid(&self) -> u128 {
        self.uuid.unwrap_or(self.session_id)
    }

    #[must_use]
    pub fn entity_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.sneaking {
            flags |= FLAG_SNEAKING;
        }
        if self.sprinting {
            flags |= FLAG_SPRINTING;
        }
        flags
    }

    #[must_use]
    pub fn current_pose(&self) -> i32 {
        if self.sneaking {
            POSE_CROUCHING
        } else {
            POSE_STANDING
        }
    }

    #[must_use]
    pub fn as_entity(&self) -> Entity {
        Entity {
            id: self.entity_id,
            uuid: self.uuid,
            world: self.world.clone(),
            x: self.x,
            y: self.y,
            z: self.z,
            yaw: self.yaw,
            pitch: self.pitch,
            vx: self.vx,
            vy: self.vy,
            vz: self.vz,
            lifecycle: EntityLifecycle::Active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_session_identity() {
        let session = PlayerSession::new(7, "Steve".to_owned(), Some(1234), 776, 3, 9999);
        assert_eq!(session.connection_id, 7);
        assert_eq!(session.name, "Steve");
        assert_eq!(session.uuid, Some(1234));
        assert_eq!(session.protocol_version, 776);
        assert_eq!(session.entity_id, 3);
        assert_eq!(session.session_id, 9999);
    }

    #[test]
    fn starts_at_spawn_as_survival() {
        let session = PlayerSession::new(1, "Alex".to_owned(), None, 776, 1, 5555);
        assert_eq!(session.gamemode, GameMode::Survival);
        assert_eq!(session.dimension, "minecraft:overworld");
        assert_eq!((session.x, session.y, session.z), (0.0, 100.0, 0.0));
        assert_eq!((session.yaw, session.pitch), (0.0, 0.0));
        assert_eq!((session.vx, session.vy, session.vz), (0.0, 0.0, 0.0));
        assert!(!session.on_ground);
    }

    #[test]
    fn flags_pose_and_display_uuid() {
        let mut session = PlayerSession::new(1, "Alex".to_owned(), None, 776, 5, 1111);
        assert_eq!(session.entity_flags(), 0);
        assert_eq!(session.current_pose(), 0);
        assert_eq!(session.display_uuid(), 1111);
        session.sneaking = true;
        assert_eq!(session.entity_flags(), 0x02);
        assert_eq!(session.current_pose(), 5);
        session.sprinting = true;
        assert_eq!(session.entity_flags(), 0x0A);
        let named = PlayerSession::new(2, "Steve".to_owned(), Some(7777), 776, 6, 2222);
        assert_eq!(named.display_uuid(), 7777);
    }

    #[test]
    fn entity_snapshot_mirrors_player_state() {
        let mut session = PlayerSession::new(7, "Steve".to_owned(), Some(1234), 776, 3, 9999);
        session.x = 1.5;
        session.vy = -1.0;
        session.on_ground = true;
        let entity = session.as_entity();
        assert_eq!(entity.id, 3);
        assert_eq!(entity.uuid, Some(1234));
        assert_eq!(entity.world, "world");
        assert_eq!((entity.x, entity.vy), (1.5, -1.0));
        assert!(entity.is_active());
        assert_eq!(entity.chunk_pos(), crate::world::ChunkPos::new(0, 0));
    }
}
