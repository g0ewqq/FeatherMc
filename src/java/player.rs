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
    pub flying: bool,
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
            flying: false,
            inventory: Inventory::starting(),
            cursor: ItemStack::empty(),
        }
    }

    #[must_use]
    pub fn held_item(&self) -> ItemStack {
        self.inventory.held()
    }

    /// Server-side ability flags sent to the client. Creative players may
    /// fly and are flagged as instant builders; they are not made
    /// invulnerable because combat/damage is not implemented. Spectators
    /// fly and are invulnerable, matching vanilla.
    #[must_use]
    pub fn ability_flags(&self) -> u8 {
        use super::packets::{
            ABILITY_ALLOW_FLYING, ABILITY_FLYING, ABILITY_INSTANT_BUILD, ABILITY_INVULNERABLE,
        };
        match self.gamemode {
            GameMode::Creative => {
                let mut flags = ABILITY_ALLOW_FLYING | ABILITY_INSTANT_BUILD;
                if self.flying {
                    flags |= ABILITY_FLYING;
                }
                flags
            }
            GameMode::Spectator => ABILITY_INVULNERABLE | ABILITY_ALLOW_FLYING | ABILITY_FLYING,
            GameMode::Survival | GameMode::Adventure => 0,
        }
    }

    #[must_use]
    pub fn is_creative(&self) -> bool {
        self.gamemode == GameMode::Creative
    }

    /// True when the player is exempt from gravity and ground collision.
    #[must_use]
    pub fn is_flying_exempt(&self) -> bool {
        self.gamemode == GameMode::Spectator || (self.gamemode == GameMode::Creative && self.flying)
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

    /// Snapshot for [`crate::persist::PlayerStore`].
    #[must_use]
    pub fn persist(&self) -> crate::persist::PlayerData {
        crate::persist::PlayerData {
            name: self.name.clone(),
            world: self.world.clone(),
            x: self.x,
            y: self.y,
            z: self.z,
            yaw: self.yaw,
            pitch: self.pitch,
            gamemode: match self.gamemode {
                GameMode::Survival => "survival".to_owned(),
                GameMode::Creative => "creative".to_owned(),
                GameMode::Adventure => "adventure".to_owned(),
                GameMode::Spectator => "spectator".to_owned(),
            },
            flying: self.flying,
            selected: self.inventory.selected(),
            inventory: (0..crate::inventory::INVENTORY_SIZE)
                .map(|index| {
                    let stack = self.inventory.get(index).unwrap_or(ItemStack::empty());
                    (stack.item.id(), stack.count)
                })
                .collect(),
            cursor: (self.cursor.item.id(), self.cursor.count),
        }
    }

    /// Restore from a [`crate::persist::PlayerStore`] snapshot. Every field is
    /// validated — bad entries fall back to current values or empty stacks
    /// rather than panicking or landing in an invalid state.
    pub fn apply_persisted(&mut self, data: &crate::persist::PlayerData) {
        if !data.world.is_empty() {
            self.world = data.world.clone();
        }
        if data.x.is_finite() && data.z.is_finite() {
            self.x = data.x.clamp(-30_000_000.0, 30_000_000.0);
            self.z = data.z.clamp(-30_000_000.0, 30_000_000.0);
        }
        if data.y.is_finite() {
            self.y = data.y.clamp(-64.0, 320.0);
        }
        if data.yaw.is_finite() {
            self.yaw = data.yaw.clamp(-180.0, 180.0);
        }
        if data.pitch.is_finite() {
            self.pitch = data.pitch.clamp(-90.0, 90.0);
        }
        self.gamemode = match data.gamemode.as_str() {
            "creative" => GameMode::Creative,
            "adventure" => GameMode::Adventure,
            "spectator" => GameMode::Spectator,
            _ => GameMode::Survival,
        };
        self.flying = data.flying && self.is_flying_exempt_mode();
        self.inventory.select(data.selected);
        for (index, (id, count)) in data
            .inventory
            .iter()
            .take(crate::inventory::INVENTORY_SIZE)
            .enumerate()
        {
            self.inventory
                .set(index, crate::persist::PlayerData::stack_of(*id, *count));
        }
        self.cursor = crate::persist::PlayerData::stack_of(data.cursor.0, data.cursor.1);
    }

    #[must_use]
    fn is_flying_exempt_mode(&self) -> bool {
        matches!(self.gamemode, GameMode::Creative | GameMode::Spectator)
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
    fn creative_reports_abilities_and_flight_exemption() {
        let mut session = PlayerSession::new(1, "Creative".to_owned(), None, 776, 1, 111);
        session.gamemode = GameMode::Creative;
        session.flying = true;
        assert!(session.is_creative());
        assert_eq!(session.ability_flags(), 0x04 | 0x08 | 0x02);
        assert!(session.is_flying_exempt());

        session.flying = false;
        assert_eq!(session.ability_flags(), 0x04 | 0x08);
        assert!(!session.is_flying_exempt());
    }

    #[test]
    fn survival_has_no_abilities() {
        let session = PlayerSession::new(1, "Survival".to_owned(), None, 776, 1, 222);
        assert!(!session.is_creative());
        assert!(!session.flying);
        assert_eq!(session.ability_flags(), 0);
        assert!(!session.is_flying_exempt());
    }

    #[test]
    fn spectator_always_flies_and_is_invulnerable() {
        let mut session = PlayerSession::new(1, "Spec".to_owned(), None, 776, 1, 333);
        session.gamemode = GameMode::Spectator;
        assert_eq!(session.ability_flags(), 0x01 | 0x04 | 0x02);
        assert!(session.is_flying_exempt());
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
