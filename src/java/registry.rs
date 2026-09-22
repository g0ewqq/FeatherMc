use std::collections::HashMap;

use super::player::PlayerSession;
use crate::network::ConnectionId;

#[derive(Debug, Default)]
pub struct PlayerRegistry {
    players: HashMap<ConnectionId, PlayerSession>,
}

impl PlayerRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, player: PlayerSession) {
        self.players.insert(player.connection_id, player);
    }

    pub fn remove(&mut self, id: ConnectionId) -> Option<PlayerSession> {
        self.players.remove(&id)
    }

    #[must_use]
    pub fn get(&self, id: ConnectionId) -> Option<&PlayerSession> {
        self.players.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &PlayerSession> {
        self.players.values()
    }

    pub fn get_mut(&mut self, id: ConnectionId) -> Option<&mut PlayerSession> {
        self.players.get_mut(&id)
    }

    #[must_use]
    pub fn find_by_uuid(&self, uuid: u128) -> Option<&PlayerSession> {
        self.players.values().find(|p| p.uuid == Some(uuid))
    }

    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&PlayerSession> {
        self.players
            .values()
            .find(|p| p.name.eq_ignore_ascii_case(name))
    }

    #[must_use]
    pub fn find_duplicate(
        &self,
        except: ConnectionId,
        name: &str,
        uuid: Option<u128>,
    ) -> Option<ConnectionId> {
        self.players
            .values()
            .find(|p| {
                p.connection_id != except
                    && (uuid.is_some_and(|id| p.uuid == Some(id))
                        || p.name.eq_ignore_ascii_case(name))
            })
            .map(|p| p.connection_id)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.players.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> PlayerRegistry {
        let mut registry = PlayerRegistry::new();
        registry.insert(PlayerSession::new(
            1,
            "Steve".to_owned(),
            Some(100),
            776,
            1,
            101,
        ));
        registry.insert(PlayerSession::new(2, "Alex".to_owned(), None, 776, 2, 102));
        registry
    }

    #[test]
    fn finds_players_by_id_uuid_and_name() {
        let registry = registry();
        assert_eq!(registry.get(1).unwrap().name, "Steve");
        assert!(registry.get(99).is_none());
        assert_eq!(registry.find_by_uuid(100).unwrap().name, "Steve");
        assert!(registry.find_by_uuid(999).is_none());
        assert_eq!(registry.find_by_name("alex").unwrap().connection_id, 2);
        assert!(registry.find_by_name("Nobody").is_none());
    }

    #[test]
    fn counts_and_removes_players() {
        let mut registry = registry();
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
        assert_eq!(registry.remove(1).unwrap().name, "Steve");
        assert!(registry.remove(1).is_none());
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn detects_duplicates_by_uuid_or_name() {
        let registry = registry();
        assert_eq!(registry.find_duplicate(9, "Someone", Some(100)), Some(1));
        assert_eq!(registry.find_duplicate(9, "steve", None), Some(1));
        assert_eq!(registry.find_duplicate(9, "Herobrine", None), None);
        assert_eq!(registry.find_duplicate(1, "Steve", Some(100)), None);
    }
}
