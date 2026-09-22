use crate::world::ChunkPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityLifecycle {
    Active,
    Gone,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: i32,
    pub uuid: Option<u128>,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub lifecycle: EntityLifecycle,
}

impl Entity {
    #[must_use]
    pub fn spawn(id: i32, uuid: Option<u128>, world: String, x: f64, y: f64, z: f64) -> Self {
        Self {
            id,
            uuid,
            world,
            x,
            y,
            z,
            yaw: 0.0,
            pitch: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            lifecycle: EntityLifecycle::Active,
        }
    }

    pub fn despawn(&mut self) {
        self.lifecycle = EntityLifecycle::Gone;
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.lifecycle == EntityLifecycle::Active
    }

    #[must_use]
    pub fn chunk_pos(&self) -> ChunkPos {
        ChunkPos::from_world(self.x.floor() as i32, self.z.floor() as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_spawns_active_and_despawns() {
        let mut entity = Entity::spawn(1, Some(99), "world".to_owned(), 0.0, 65.0, 0.0);
        assert!(entity.is_active());
        assert_eq!((entity.x, entity.y, entity.z), (0.0, 65.0, 0.0));
        assert_eq!((entity.vx, entity.vy, entity.vz), (0.0, 0.0, 0.0));
        entity.despawn();
        assert!(!entity.is_active());
        assert_eq!(entity.lifecycle, EntityLifecycle::Gone);
    }

    #[test]
    fn chunk_pos_follows_negative_coordinates() {
        let entity = Entity::spawn(2, None, "world".to_owned(), -1.0, 65.0, -17.0);
        assert_eq!(entity.chunk_pos(), ChunkPos::new(-1, -2));
    }
}
