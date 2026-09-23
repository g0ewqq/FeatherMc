use std::collections::{HashMap, HashSet};

use tracing::{debug, info};

use super::chunks::{
    block_to_state, encode_batch_finished, encode_batch_start, encode_chunk_data,
    encode_set_center, encode_unload_chunk, CHUNK_RADIUS,
};
use super::entities::{
    encode_destroy_entities, encode_entity_metadata, encode_entity_position,
    encode_entity_position_rotation, encode_entity_rotation, encode_head_rotation,
    encode_player_info_add, encode_player_info_remove, encode_spawn_player, encode_teleport_entity,
};
use super::error::ProtoError;
use super::metadata::player_metadata;
use super::packets::{
    decode_handshake, decode_login_start, encode_block_changed_ack, encode_block_update,
    encode_container_content, encode_default_spawn, encode_finish_configuration, encode_game_event,
    encode_held_slot, encode_keep_alive, encode_known_packs, encode_login_disconnect,
    encode_login_success, encode_play_login, encode_pong, encode_status_response,
    encode_sync_position, encode_update_tags, DUPLICATE_LOGIN_REASON,
};
use super::player::PlayerSession;
use super::proto::{decode_varint_prefix, Reader};
use super::registries::{default_registries, encode_registry_data};
use super::registry::PlayerRegistry;
use super::state::ProtocolState;
use crate::config::Config;
use crate::inventory::{Inventory, ItemStack};
use crate::network::{Connection, ConnectionId, NetworkManager};
use crate::world::{Block, ChunkPos, World, WorldManager, DEFAULT_WORLD_NAME};

const MAX_PACKET_LEN: i32 = 2097151;
const KEEPALIVE_INTERVAL_TICKS: u64 = 200;
const MAX_MOVE_PER_PACKET: f64 = 12.0;
const GRAVITY_PER_TICK: f64 = 0.08;
const AIR_DRAG: f64 = 0.98;
const TERMINAL_VELOCITY: f64 = -3.92;
const CORRECTION_THRESHOLD: f64 = 0.5;
const PLAYER_HALF_WIDTH: f64 = 0.3;
const BLOCK_REACH: f64 = 7.0;

const FACE_OFFSETS: [(i32, i32, i32); 6] = [
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
    (-1, 0, 0),
    (1, 0, 0),
];

pub struct ServerStatus {
    pub motd: String,
    pub max_players: u32,
}

impl ServerStatus {
    #[must_use]
    pub fn from_config(config: &Config) -> Self {
        Self {
            motd: config.server.motd.clone(),
            max_players: config.server.max_players,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TrackedPose {
    entity_id: i32,
    x: f64,
    y: f64,
    z: f64,
    yaw: f32,
    pitch: f32,
    flags: u8,
    pose: i32,
}

impl TrackedPose {
    fn snapshot(player: &PlayerSession) -> Self {
        Self {
            entity_id: player.entity_id,
            x: player.x,
            y: player.y,
            z: player.z,
            yaw: player.yaw,
            pitch: player.pitch,
            flags: player.entity_flags(),
            pose: player.current_pose(),
        }
    }
}

fn player_chunk(player: &PlayerSession) -> ChunkPos {
    ChunkPos::from_world(player.x.floor() as i32, player.z.floor() as i32)
}

fn movement_delta(value: f64) -> Option<i16> {
    if value.abs() >= 8.0 {
        return None;
    }
    Some((value * 4096.0).clamp(i16::MIN as f64, i16::MAX as f64) as i16)
}

fn send_entity_movement(conn: &mut Connection, entity: &PlayerSession, last: &TrackedPose) {
    let dx = entity.x - last.x;
    let dy = entity.y - last.y;
    let dz = entity.z - last.z;
    let moved = dx != 0.0 || dy != 0.0 || dz != 0.0;
    let rotated = entity.yaw != last.yaw || entity.pitch != last.pitch;
    match (movement_delta(dx), movement_delta(dy), movement_delta(dz)) {
        (Some(dx), Some(dy), Some(dz)) if moved && rotated => {
            send_response(
                conn,
                &encode_entity_position_rotation(
                    entity.entity_id,
                    dx,
                    dy,
                    dz,
                    entity.yaw,
                    entity.pitch,
                    entity.on_ground,
                ),
            );
            send_response(conn, &encode_head_rotation(entity.entity_id, entity.yaw));
        }
        (Some(dx), Some(dy), Some(dz)) if moved => {
            send_response(
                conn,
                &encode_entity_position(entity.entity_id, dx, dy, dz, entity.on_ground),
            );
        }
        _ if moved || rotated => {
            if moved {
                send_response(
                    conn,
                    &encode_teleport_entity(
                        entity.entity_id,
                        entity.x,
                        entity.y,
                        entity.z,
                        entity.yaw,
                        entity.pitch,
                        entity.on_ground,
                    ),
                );
            } else {
                send_response(
                    conn,
                    &encode_entity_rotation(
                        entity.entity_id,
                        entity.yaw,
                        entity.pitch,
                        entity.on_ground,
                    ),
                );
            }
            send_response(conn, &encode_head_rotation(entity.entity_id, entity.yaw));
        }
        _ => {}
    }
}

pub struct JavaSession {
    id: ConnectionId,
    state: ProtocolState,
    protocol_version: i32,
    login_name: Option<String>,
    config_stage: ConfigStage,
    teleport_id: i32,
    confirmed_teleport: i32,
    staging: Vec<u8>,
    chunk_center: Option<ChunkPos>,
    sent_chunks: HashSet<ChunkPos>,
    sim_x: f64,
    sim_z: f64,
    tracked: HashMap<ConnectionId, TrackedPose>,
    edits: Vec<PendingEdit>,
    clicks: Vec<PendingClick>,
    inv_dirty: bool,
    held_dirty: bool,
    inv_state: i32,
}

#[derive(Debug, Clone, Copy)]
enum PendingEdit {
    Break {
        x: i32,
        y: i32,
        z: i32,
        sequence: i32,
    },
    Place {
        x: i32,
        y: i32,
        z: i32,
        face: i32,
        sequence: i32,
    },
}

#[derive(Debug, Clone, Copy)]
struct PendingClick {
    window: i32,
    state: i32,
    slot: i32,
    button: u8,
    mode: i32,
}

fn read_hashed_slot(reader: &mut Reader) -> Result<(), ProtoError> {
    if !reader.read_bool()? {
        return Ok(());
    }
    reader.read_varint()?;
    reader.read_varint()?;
    let added = reader.read_varint()?;
    if added < 0 {
        return Err(ProtoError::NegativeLength(added));
    }
    for _ in 0..added {
        reader.read_varint()?;
        reader.read_i32()?;
    }
    let removed = reader.read_varint()?;
    if removed < 0 {
        return Err(ProtoError::NegativeLength(removed));
    }
    for _ in 0..removed {
        reader.read_varint()?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigStage {
    Fresh,
    KnownPacksSent,
    FinishSent,
    Complete,
}

impl JavaSession {
    fn new(id: ConnectionId) -> Self {
        Self {
            id,
            state: ProtocolState::Handshaking,
            protocol_version: -1,
            login_name: None,
            config_stage: ConfigStage::Fresh,
            teleport_id: 1,
            confirmed_teleport: 0,
            staging: Vec::new(),
            chunk_center: None,
            sent_chunks: HashSet::new(),
            sim_x: crate::world::SPAWN_X,
            sim_z: crate::world::SPAWN_Z,
            tracked: HashMap::new(),
            edits: Vec::new(),
            clicks: Vec::new(),
            inv_dirty: false,
            held_dirty: false,
            inv_state: 0,
        }
    }

    #[must_use]
    pub fn protocol_version(&self) -> i32 {
        self.protocol_version
    }

    #[must_use]
    pub fn login_name(&self) -> Option<&str> {
        self.login_name.as_deref()
    }
}

pub struct JavaHandler {
    status: ServerStatus,
    sessions: HashMap<ConnectionId, JavaSession>,
    players: PlayerRegistry,
    entity_id_counter: i32,
    departed: Vec<u128>,
}

impl JavaHandler {
    #[must_use]
    pub fn new(status: ServerStatus) -> Self {
        Self {
            status,
            sessions: HashMap::new(),
            players: PlayerRegistry::new(),
            entity_id_counter: 1,
            departed: Vec::new(),
        }
    }

    #[must_use]
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    #[must_use]
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    pub fn pump(&mut self, network: &mut NetworkManager, tick: u64) {
        let ids: Vec<ConnectionId> = network.connections().map(Connection::id).collect();
        for id in &ids {
            if !self.sessions.contains_key(id) {
                self.sessions.insert(*id, JavaSession::new(*id));
            }
        }

        let ids: Vec<ConnectionId> = self.sessions.keys().copied().collect();
        for id in ids {
            self.pump_one(id, network, tick);
        }
    }

    fn pump_one(&mut self, id: ConnectionId, network: &mut NetworkManager, tick: u64) {
        loop {
            match self.pump_frame(id, network) {
                None => break,
                Some(PacketAction::None) => {}
                Some(PacketAction::CompleteLogin { name, uuid }) => {
                    self.complete_login(id, network, name, uuid);
                }
                Some(PacketAction::EnterPlay) => {
                    self.enter_play(id, network);
                }
                Some(PacketAction::PlayerAction(action)) => {
                    if let Some(player) = self.players.get_mut(id) {
                        match action {
                            0 => player.sneaking = true,
                            1 => player.sneaking = false,
                            3 => player.sprinting = true,
                            4 => player.sprinting = false,
                            _ => {}
                        }
                    }
                }
                Some(PacketAction::Edit(edit)) => {
                    if let Some(session) = self.sessions.get_mut(&id) {
                        session.edits.push(edit);
                    }
                }
                Some(PacketAction::SelectSlot(index)) => {
                    if let Some(player) = self.players.get_mut(id) {
                        player.inventory.select(index);
                    }
                }
                Some(PacketAction::HeldEcho) => {
                    if let Some(session) = self.sessions.get_mut(&id) {
                        session.held_dirty = true;
                    }
                }
                Some(PacketAction::Click(click)) => {
                    if let Some(session) = self.sessions.get_mut(&id) {
                        session.clicks.push(click);
                    }
                }
                Some(PacketAction::MovePlayer {
                    x,
                    y,
                    z,
                    yaw,
                    pitch,
                    has_position,
                    has_rotation,
                }) => {
                    if let Some(player) = self.players.get_mut(id) {
                        if has_position && x.is_finite() && y.is_finite() && z.is_finite() {
                            let dx = x - player.x;
                            let dy = y - player.y;
                            let dz = z - player.z;
                            if dx.hypot(dz) <= MAX_MOVE_PER_PACKET
                                && dy.abs() <= MAX_MOVE_PER_PACKET
                            {
                                player.x = x.clamp(-30_000_000.0, 30_000_000.0);
                                player.y = y.clamp(-64.0, 320.0);
                                player.z = z.clamp(-30_000_000.0, 30_000_000.0);
                            } else {
                                debug!("connection {id} rejected absurd move");
                            }
                        }
                        if has_rotation {
                            player.yaw = finite_or_keep_angle(yaw, player.yaw);
                            player.pitch = finite_or_keep_angle(pitch, player.pitch);
                        }
                    }
                }
            }
            if network
                .connection(id)
                .map(Connection::is_closed)
                .unwrap_or(true)
            {
                self.remove_session(id);
                return;
            }
        }
        let (Some(session), Some(conn)) = (self.sessions.get_mut(&id), network.connection_mut(id))
        else {
            self.remove_session(id);
            return;
        };
        if session.state == ProtocolState::Configuration
            && session.config_stage == ConfigStage::Fresh
        {
            send_response(conn, &encode_known_packs());
            session.config_stage = ConfigStage::KnownPacksSent;
        }
        if session.state == ProtocolState::Play && tick % KEEPALIVE_INTERVAL_TICKS == 0 {
            send_response(conn, &encode_keep_alive(tick as i64));
        }
    }

    fn pump_frame(
        &mut self,
        id: ConnectionId,
        network: &mut NetworkManager,
    ) -> Option<PacketAction> {
        let (Some(session), Some(conn)) = (self.sessions.get_mut(&id), network.connection_mut(id))
        else {
            self.remove_session(id);
            return None;
        };
        session.staging.extend(conn.drain_available());
        let frame = match take_frame(&mut session.staging) {
            Ok(frame) => frame,
            Err(e) => {
                debug!("protocol error on connection {id}: {e}");
                conn.close();
                return None;
            }
        };
        let (pid, payload) = frame?;
        debug!(
            "conn {id} [{:?}] recv packet 0x{pid:02X} ({} bytes)",
            session.state,
            payload.len()
        );
        match handle_packet(&self.status, session, pid, &payload, conn) {
            Ok(action) => Some(action),
            Err(e) => {
                debug!("protocol error on connection {id}: {e}");
                conn.close();
                None
            }
        }
    }

    fn complete_login(
        &mut self,
        id: ConnectionId,
        network: &mut NetworkManager,
        name: String,
        uuid: Option<u128>,
    ) {
        if let Some(old_id) = self.players.find_duplicate(id, &name, uuid) {
            if let Some(old_conn) = network.connection_mut(old_id) {
                send_response(old_conn, &encode_login_disconnect(DUPLICATE_LOGIN_REASON));
                old_conn.close();
            }
            if let Some(old) = self.players.remove(old_id) {
                info!("Player disconnected: {} (duplicate login)", old.name);
            }
            self.sessions.remove(&old_id);
        }
        let version = self
            .sessions
            .get(&id)
            .map(JavaSession::protocol_version)
            .unwrap_or(-1);
        let entity_id = self.entity_id_counter;
        self.entity_id_counter += 1;
        let Ok(session_id) = new_session_id() else {
            debug!("connection {id}: OS randomness unavailable, disconnecting");
            let Some(conn) = network.connection_mut(id) else {
                return;
            };
            conn.close();
            return;
        };
        let player = PlayerSession::new(id, name.clone(), uuid, version, entity_id, session_id);
        let response = encode_login_success(uuid.unwrap_or_default(), &player.name, session_id);
        let Some(conn) = network.connection_mut(id) else {
            return;
        };
        if conn.send(&response).is_err() {
            conn.close();
            return;
        }
        self.players.insert(player);
        let announcement = {
            let joined = self.players.get(id).expect("player just inserted");
            encode_player_info_add(joined.display_uuid(), &joined.name, joined.gamemode as i32)
        };
        let viewers: Vec<ConnectionId> = self
            .sessions
            .iter()
            .filter(|(other, session)| **other != id && session.state == ProtocolState::Play)
            .map(|(other, _)| *other)
            .collect();
        for other in viewers {
            if let Some(conn) = network.connection_mut(other) {
                send_response(conn, &announcement);
            }
        }
        info!("Player login completed: {name}");
    }

    pub fn pump_chunks(&mut self, network: &mut NetworkManager, worlds: &mut WorldManager) {
        let pending: Vec<(ConnectionId, ChunkPos)> = self
            .sessions
            .iter()
            .filter(|(_, session)| session.state == ProtocolState::Play)
            .filter_map(|(id, session)| {
                let player = self.players.get(*id)?;
                let center = ChunkPos::from_world(player.x.floor() as i32, player.z.floor() as i32);
                if session.chunk_center == Some(center) {
                    return None;
                }
                Some((*id, center))
            })
            .collect();
        for (id, center) in pending {
            let first = self
                .sessions
                .get(&id)
                .map(|session| session.chunk_center.is_none())
                .unwrap_or(false);
            let Some(world) = worlds.get_mut(DEFAULT_WORLD_NAME) else {
                continue;
            };
            let Some(conn) = network.connection_mut(id) else {
                self.remove_session(id);
                continue;
            };
            if first {
                send_response(conn, &encode_game_event(13, 0.0));
            }
            send_response(conn, &encode_set_center(center.x, center.z));
            send_response(conn, &encode_batch_start());
            let mut desired = HashSet::new();
            for dx in -CHUNK_RADIUS..=CHUNK_RADIUS {
                for dz in -CHUNK_RADIUS..=CHUNK_RADIUS {
                    desired.insert(ChunkPos::new(center.x + dx, center.z + dz));
                }
            }
            let mut sent = 0i32;
            if let Some(session) = self.sessions.get(&id) {
                let mut removed: Vec<ChunkPos> =
                    session.sent_chunks.difference(&desired).copied().collect();
                removed.sort_by_key(|pos| (pos.x, pos.z));
                for pos in removed {
                    send_response(conn, &encode_unload_chunk(pos.x, pos.z));
                }
                let mut added: Vec<ChunkPos> =
                    desired.difference(&session.sent_chunks).copied().collect();
                added.sort_by_key(|pos| (pos.x, pos.z));
                for pos in added {
                    let chunk = world.load_chunk(pos);
                    send_response(conn, &encode_chunk_data(chunk));
                    sent += 1;
                }
            }
            send_response(conn, &encode_batch_finished(sent));
            if conn.is_closed() {
                self.remove_session(id);
                continue;
            }
            if let Some(session) = self.sessions.get_mut(&id) {
                session.chunk_center = Some(center);
                session.sent_chunks = desired;
            }
        }
    }

    pub fn tick_physics(&mut self, network: &mut NetworkManager, worlds: &WorldManager) {
        let ids: Vec<ConnectionId> = self
            .sessions
            .iter()
            .filter(|(_, session)| session.state == ProtocolState::Play)
            .filter_map(|(id, _)| self.players.get(*id).map(|_| *id))
            .collect();
        for id in ids {
            let corrected = self.simulate_player(id, worlds);
            if !corrected {
                continue;
            }
            let (Some(session), Some(conn), Some(player)) = (
                self.sessions.get_mut(&id),
                network.connection_mut(id),
                self.players.get(id),
            ) else {
                continue;
            };
            let teleport_id = session.teleport_id;
            send_response(
                conn,
                &encode_sync_position(
                    player.x,
                    player.y,
                    player.z,
                    player.yaw,
                    player.pitch,
                    teleport_id,
                ),
            );
            if conn.is_closed() {
                self.remove_session(id);
                continue;
            }
            session.teleport_id += 1;
        }
    }

    fn simulate_player(&mut self, id: ConnectionId, worlds: &WorldManager) -> bool {
        let world_name = match self.players.get(id) {
            Some(player) => player.world.clone(),
            None => return false,
        };
        let Some(world) = worlds.get(&world_name) else {
            return false;
        };
        let (Some(session), Some(player)) = (self.sessions.get_mut(&id), self.players.get_mut(id))
        else {
            return false;
        };
        let before = (player.x, player.y, player.z);
        let mut correct = false;
        if collides(world, player.x, player.y, player.z) {
            player.x = session.sim_x;
            player.z = session.sim_z;
            correct = true;
        }
        player.vx = player.x - session.sim_x;
        player.vz = player.z - session.sim_z;
        match world.ground_top(player.x, player.z, player.y) {
            Some(top) if player.y <= top + 0.001 && player.vy <= 0.0 => {
                if before.1 < top - CORRECTION_THRESHOLD {
                    correct = true;
                }
                player.y = top;
                player.vy = 0.0;
                player.on_ground = true;
            }
            _ => {
                player.vy = (player.vy - GRAVITY_PER_TICK) * AIR_DRAG;
                if player.vy < TERMINAL_VELOCITY {
                    player.vy = TERMINAL_VELOCITY;
                }
                player.y += player.vy;
                player.on_ground = false;
                if player.vy < 0.0 {
                    if let Some(top) = world.ground_top(player.x, player.z, player.y) {
                        if player.y <= top {
                            player.y = top;
                            player.vy = 0.0;
                            player.on_ground = true;
                        }
                    }
                }
            }
        }
        session.sim_x = player.x;
        session.sim_z = player.z;
        correct
    }

    pub fn pump_visibility(&mut self, network: &mut NetworkManager) {
        for uuid in std::mem::take(&mut self.departed) {
            let packet = encode_player_info_remove(&[uuid]);
            let viewers: Vec<ConnectionId> = self
                .sessions
                .iter()
                .filter(|(_, session)| session.state == ProtocolState::Play)
                .map(|(id, _)| *id)
                .collect();
            for viewer in viewers {
                if let Some(conn) = network.connection_mut(viewer) {
                    send_response(conn, &packet);
                }
            }
        }
        let viewers: Vec<ConnectionId> = self
            .sessions
            .iter()
            .filter(|(_, session)| session.state == ProtocolState::Play)
            .map(|(id, _)| *id)
            .collect();
        for viewer in viewers {
            self.pump_viewer(viewer, network);
        }
    }

    pub fn pump_blocks(&mut self, network: &mut NetworkManager, worlds: &mut WorldManager) {
        let editors: Vec<ConnectionId> = self
            .sessions
            .iter()
            .filter(|(_, session)| {
                session.state == ProtocolState::Play && !session.edits.is_empty()
            })
            .map(|(id, _)| *id)
            .collect();
        for id in editors {
            let edits = match self.sessions.get_mut(&id) {
                Some(session) => std::mem::take(&mut session.edits),
                None => continue,
            };
            for edit in edits {
                self.apply_edit(id, edit, network, worlds);
            }
        }
    }

    fn apply_edit(
        &mut self,
        id: ConnectionId,
        edit: PendingEdit,
        network: &mut NetworkManager,
        worlds: &mut WorldManager,
    ) {
        let player = match self.players.get(id) {
            Some(player) => player.clone(),
            None => return,
        };
        let held = player.held_item();
        let (target, block, sequence) = match edit {
            PendingEdit::Break { x, y, z, sequence } => ((x, y, z), Block::Air, sequence),
            PendingEdit::Place {
                x,
                y,
                z,
                face,
                sequence,
            } => match held.item.block() {
                Some(block) if !held.is_empty() => {
                    let (ox, oy, oz) = FACE_OFFSETS[face as usize];
                    ((x + ox, y + oy, z + oz), block, sequence)
                }
                _ => return,
            },
        };
        let eyes = (player.x, player.y + 1.62, player.z);
        let center = (
            target.0 as f64 + 0.5,
            target.1 as f64 + 0.5,
            target.2 as f64 + 0.5,
        );
        let dist = (eyes.0 - center.0)
            .hypot(eyes.1 - center.1)
            .hypot(eyes.2 - center.2);
        let Some(world) = worlds.get_mut(&player.world) else {
            return;
        };
        let current = world.get_block(target.0, target.1, target.2);
        let loaded = world.is_chunk_loaded(ChunkPos::from_world(target.0, target.2));
        if dist > BLOCK_REACH {
            debug!("connection {id} edit out of reach (dist {dist:.1}): {edit:?}");
            return;
        }
        if !loaded {
            debug!("connection {id} edit in unloaded chunk: {edit:?}");
            return;
        }
        let allowed = match (edit, current) {
            (PendingEdit::Break { .. }, Some(existing)) => existing.is_solid(),
            (PendingEdit::Place { .. }, Some(Block::Air)) => {
                let (fx, fy, fz) = (
                    player.x.floor() as i32,
                    player.y.floor() as i32,
                    player.z.floor() as i32,
                );
                (target.0, target.1, target.2) != (fx, fy, fz)
                    && (target.0, target.1, target.2) != (fx, fy + 1, fz)
            }
            _ => false,
        };
        if !allowed {
            if let (Some(current), Some(conn)) = (current, network.connection_mut(id)) {
                send_response(
                    conn,
                    &encode_block_update(target.0, target.1, target.2, block_to_state(current)),
                );
            }
            return;
        }
        let changed = world
            .set_block(target.0, target.1, target.2, block)
            .is_some();
        if !changed {
            return;
        }
        info!(
            "Player {} set ({}, {}, {}) to {}",
            player.name,
            target.0,
            target.1,
            target.2,
            block.name()
        );
        if matches!(edit, PendingEdit::Place { .. }) {
            if let Some(player) = self.players.get_mut(id) {
                let held_index = player.inventory.selected_slot_index();
                player.inventory.consume_one(held_index);
            }
            if let Some(session) = self.sessions.get_mut(&id) {
                session.inv_dirty = true;
            }
        }
        let chunk = ChunkPos::from_world(target.0, target.2);
        if let Some(stored) = world.get_chunk_mut(chunk) {
            for index in 0..stored.section_count() {
                if let Some(section) = stored.section_mut(index) {
                    section.take_changed();
                }
            }
        }
        if let Some(conn) = network.connection_mut(id) {
            send_response(conn, &encode_block_changed_ack(sequence));
        }
        let packet = encode_block_update(target.0, target.1, target.2, block_to_state(block));
        let viewers: Vec<ConnectionId> = self
            .sessions
            .iter()
            .filter(|(_, session)| {
                session.state == ProtocolState::Play && session.sent_chunks.contains(&chunk)
            })
            .map(|(other, _)| *other)
            .collect();
        for viewer in viewers {
            if let Some(conn) = network.connection_mut(viewer) {
                send_response(conn, &packet);
            }
        }
    }

    pub fn pump_inventory(&mut self, network: &mut NetworkManager) {
        let clickers: Vec<(ConnectionId, Vec<PendingClick>)> = self
            .sessions
            .iter_mut()
            .filter(|(_, session)| session.state == ProtocolState::Play)
            .filter(|(_, session)| !session.clicks.is_empty())
            .map(|(id, session)| (*id, std::mem::take(&mut session.clicks)))
            .collect();
        for (id, clicks) in clickers {
            for click in clicks {
                self.apply_click(id, click);
            }
        }
        let ids: Vec<ConnectionId> = self
            .sessions
            .iter()
            .filter(|(_, session)| session.state == ProtocolState::Play)
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            let (Some(session), Some(conn), Some(player)) = (
                self.sessions.get_mut(&id),
                network.connection_mut(id),
                self.players.get(id),
            ) else {
                continue;
            };
            if session.inv_dirty {
                session.inv_state += 1;
                let mut slots = [ItemStack::empty(); crate::inventory::INVENTORY_SIZE];
                for (index, slot) in slots.iter_mut().enumerate() {
                    *slot = player.inventory.get(index).unwrap_or(ItemStack::empty());
                }
                send_response(
                    conn,
                    &encode_container_content(&slots, player.cursor, session.inv_state),
                );
                session.inv_dirty = false;
            }
            if session.held_dirty {
                send_response(conn, &encode_held_slot(player.inventory.selected()));
                session.held_dirty = false;
            }
            if conn.is_closed() {
                self.remove_session(id);
            }
        }
    }

    fn apply_click(&mut self, id: ConnectionId, click: PendingClick) {
        let (Some(session), Some(player)) = (self.sessions.get_mut(&id), self.players.get_mut(id))
        else {
            return;
        };
        if click.window != 0 {
            return;
        }
        if click.state != session.inv_state {
            session.inv_dirty = true;
            return;
        }
        let applied = match click.mode {
            0 => click_pickup(
                &mut player.inventory,
                &mut player.cursor,
                click.slot,
                click.button,
            ),
            1 => click_shift(&mut player.inventory, click.slot),
            2 => click_swap(&mut player.inventory, click.slot, click.button),
            _ => false,
        };
        if applied {
            session.inv_dirty = true;
        }
    }

    fn pump_viewer(&mut self, viewer: ConnectionId, network: &mut NetworkManager) {
        let entities: Vec<PlayerSession> = self
            .players
            .iter()
            .filter(|player| player.connection_id != viewer)
            .cloned()
            .collect();
        let tracked: Vec<(ConnectionId, TrackedPose)> = match self.sessions.get(&viewer) {
            Some(session) => session
                .tracked
                .iter()
                .map(|(owner, pose)| (*owner, *pose))
                .collect(),
            None => return,
        };
        let sent_chunks = match self.sessions.get(&viewer) {
            Some(session) => session.sent_chunks.clone(),
            None => return,
        };
        let mut despawned: Vec<ConnectionId> = tracked
            .iter()
            .filter(|(owner, _)| match self.players.get(*owner) {
                None => true,
                Some(player) => !sent_chunks.contains(&player_chunk(player)),
            })
            .map(|(owner, _)| *owner)
            .collect();
        despawned.sort_unstable();
        if !despawned.is_empty() {
            let destroy: Vec<i32> = despawned
                .iter()
                .filter_map(|owner| tracked.iter().find(|(id, _)| id == owner))
                .map(|(_, pose)| pose.entity_id)
                .collect();
            let Some(conn) = network.connection_mut(viewer) else {
                return;
            };
            for entity_id in &destroy {
                send_response(conn, &encode_destroy_entities(&[*entity_id]));
            }
            if conn.is_closed() {
                return;
            }
            if let Some(session) = self.sessions.get_mut(&viewer) {
                for owner in &despawned {
                    session.tracked.remove(owner);
                }
            }
        }
        for entity in &entities {
            let chunk = player_chunk(entity);
            if !sent_chunks.contains(&chunk) {
                continue;
            }
            let last = tracked
                .iter()
                .find(|(owner, _)| *owner == entity.connection_id)
                .map(|(_, pose)| *pose);
            let Some(conn) = network.connection_mut(viewer) else {
                return;
            };
            match last {
                None => {
                    send_response(
                        conn,
                        &encode_player_info_add(
                            entity.display_uuid(),
                            &entity.name,
                            entity.gamemode as i32,
                        ),
                    );
                    send_response(
                        conn,
                        &encode_spawn_player(
                            entity.entity_id,
                            entity.display_uuid(),
                            entity.x,
                            entity.y,
                            entity.z,
                            entity.yaw,
                            entity.pitch,
                        ),
                    );
                    send_response(
                        conn,
                        &encode_entity_metadata(
                            entity.entity_id,
                            &player_metadata(entity.entity_flags(), entity.current_pose()),
                        ),
                    );
                    if conn.is_closed() {
                        return;
                    }
                    if let Some(session) = self.sessions.get_mut(&viewer) {
                        session
                            .tracked
                            .insert(entity.connection_id, TrackedPose::snapshot(entity));
                    }
                }
                Some(previous) => {
                    let flags = entity.entity_flags();
                    let pose = entity.current_pose();
                    if (previous.flags, previous.pose) != (flags, pose) {
                        send_response(
                            conn,
                            &encode_entity_metadata(
                                entity.entity_id,
                                &player_metadata(flags, pose),
                            ),
                        );
                    }
                    send_entity_movement(conn, entity, &previous);
                    if conn.is_closed() {
                        return;
                    }
                    if let Some(session) = self.sessions.get_mut(&viewer) {
                        session
                            .tracked
                            .insert(entity.connection_id, TrackedPose::snapshot(entity));
                    }
                }
            }
        }
    }

    fn remove_session(&mut self, id: ConnectionId) {
        self.sessions.remove(&id);
        if let Some(player) = self.players.remove(id) {
            self.departed.push(player.display_uuid());
            info!("Player disconnected: {}", player.name);
        }
    }

    fn enter_play(&mut self, id: ConnectionId, network: &mut NetworkManager) {
        let (Some(session), Some(conn)) = (self.sessions.get_mut(&id), network.connection_mut(id))
        else {
            self.remove_session(id);
            return;
        };
        let Some(player) = self.players.get(id).cloned() else {
            return;
        };
        let dimension_type_id = default_registries()
            .iter()
            .find(|registry| registry.id == "minecraft:dimension_type")
            .and_then(|registry| registry.entry_index("minecraft:overworld"))
            .unwrap_or(0) as i32;
        send_response(
            conn,
            &encode_play_login(&player, self.status.max_players, dimension_type_id),
        );
        send_response(
            conn,
            &encode_default_spawn(
                &player.dimension,
                player.x as i32,
                player.y as i32,
                player.z as i32,
                player.yaw,
                player.pitch,
            ),
        );
        let teleport_id = session.teleport_id;
        send_response(
            conn,
            &encode_sync_position(
                player.x,
                player.y,
                player.z,
                player.yaw,
                player.pitch,
                teleport_id,
            ),
        );
        if conn.is_closed() {
            return;
        }
        session.teleport_id += 1;
        session.state = ProtocolState::Play;
        session.inv_dirty = true;
        info!("Player joined: {}", player.name);
    }
}

fn take_frame(staging: &mut Vec<u8>) -> Result<Option<(i32, Vec<u8>)>, ProtoError> {
    let (len, header) = match decode_varint_prefix(staging)? {
        Some(frame) => frame,
        None => return Ok(None),
    };
    if !(0..=MAX_PACKET_LEN).contains(&len) {
        return Err(ProtoError::PacketTooLong(len));
    }
    let len = len as usize;
    if staging.len() < header + len {
        return Ok(None);
    }
    let frame = &staging[header..header + len];
    let (id, id_len) = match decode_varint_prefix(frame)? {
        Some(packet) => packet,
        None => return Err(ProtoError::Truncated),
    };
    let payload = frame[id_len..].to_vec();
    staging.drain(..header + len);
    Ok(Some((id, payload)))
}

enum PacketAction {
    None,
    CompleteLogin {
        name: String,
        uuid: Option<u128>,
    },
    EnterPlay,
    PlayerAction(i32),
    Edit(PendingEdit),
    SelectSlot(u8),
    HeldEcho,
    Click(PendingClick),
    MovePlayer {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        has_position: bool,
        has_rotation: bool,
    },
}

fn finite_or_keep_angle(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-180.0, 180.0)
    } else {
        fallback
    }
}

fn handle_packet(
    status: &ServerStatus,
    session: &mut JavaSession,
    id: i32,
    payload: &[u8],
    conn: &mut Connection,
) -> Result<PacketAction, ProtoError> {
    match (session.state, id) {
        (ProtocolState::Handshaking, 0x00) => {
            let handshake = decode_handshake(payload)?;
            session.protocol_version = handshake.protocol_version;
            match handshake.next_state {
                1 => session.state = ProtocolState::Status,
                2 | 3 => session.state = ProtocolState::Login,
                next => return Err(ProtoError::InvalidNextState(next)),
            }
            debug!(
                "connection {} handshake: protocol {} -> {:?}",
                session.id, handshake.protocol_version, session.state
            );
        }
        (ProtocolState::Status, 0x00) => {
            let response = encode_status_response(
                crate::java::VERSION_NAME,
                crate::java::PROTOCOL_VERSION,
                &status.motd,
                status.max_players,
            );
            send_response(conn, &response);
        }
        (ProtocolState::Status, 0x01) => {
            let mut reader = Reader::new(payload);
            let payload = reader.read_i64()?;
            send_response(conn, &encode_pong(payload));
        }
        (ProtocolState::Login, 0x00) => {
            let login = decode_login_start(payload)?;
            session.login_name = Some(login.name.clone());
            info!("Player login started: {}", login.name);
            return Ok(PacketAction::CompleteLogin {
                name: login.name,
                uuid: login.uuid,
            });
        }
        (ProtocolState::Login, 0x03) => {
            session.state = ProtocolState::Configuration;
            debug!("connection {} entered configuration", session.id);
        }
        (ProtocolState::Configuration, 0x00 | 0x02) => {}
        (ProtocolState::Configuration, 0x07) => {
            if session.config_stage != ConfigStage::KnownPacksSent {
                return Ok(PacketAction::None);
            }
            for registry in default_registries() {
                match encode_registry_data(&registry) {
                    Ok(packet) => send_response(conn, &packet),
                    Err(e) => {
                        debug!("failed to encode registry {}: {e}", registry.id);
                        conn.close();
                        return Ok(PacketAction::None);
                    }
                }
            }
            send_response(conn, &encode_update_tags());
            send_response(conn, &encode_finish_configuration());
            session.config_stage = ConfigStage::FinishSent;
        }
        (ProtocolState::Configuration, 0x03) => {
            if session.config_stage != ConfigStage::FinishSent {
                return Ok(PacketAction::None);
            }
            session.config_stage = ConfigStage::Complete;
            match session.login_name.clone() {
                Some(name) => info!("Player configuration completed: {name}"),
                None => debug!("connection {} configuration completed", session.id),
            }
            return Ok(PacketAction::EnterPlay);
        }
        (ProtocolState::Play, 0x00) => {
            let mut reader = Reader::new(payload);
            session.confirmed_teleport = reader.read_varint()?;
        }
        (ProtocolState::Play, 0x1E) => {
            let mut reader = Reader::new(payload);
            let (x, y, z) = (reader.read_f64()?, reader.read_f64()?, reader.read_f64()?);
            reader.read_u8()?;
            return Ok(PacketAction::MovePlayer {
                x,
                y,
                z,
                yaw: 0.0,
                pitch: 0.0,
                has_position: true,
                has_rotation: false,
            });
        }
        (ProtocolState::Play, 0x1F) => {
            let mut reader = Reader::new(payload);
            let (x, y, z) = (reader.read_f64()?, reader.read_f64()?, reader.read_f64()?);
            let (yaw, pitch) = (reader.read_f32()?, reader.read_f32()?);
            reader.read_u8()?;
            return Ok(PacketAction::MovePlayer {
                x,
                y,
                z,
                yaw,
                pitch,
                has_position: true,
                has_rotation: true,
            });
        }
        (ProtocolState::Play, 0x20) => {
            let mut reader = Reader::new(payload);
            let (yaw, pitch) = (reader.read_f32()?, reader.read_f32()?);
            reader.read_u8()?;
            return Ok(PacketAction::MovePlayer {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                yaw,
                pitch,
                has_position: false,
                has_rotation: true,
            });
        }
        (ProtocolState::Play, 0x21) => {}
        (ProtocolState::Play, 0x2A) => {
            let mut reader = Reader::new(payload);
            if let (Ok(_entity), Ok(action), Ok(_jump)) = (
                reader.read_varint(),
                reader.read_varint(),
                reader.read_varint(),
            ) {
                if reader.remaining() == 0 && (0..=8).contains(&action) {
                    return Ok(PacketAction::PlayerAction(action));
                }
            }
            debug!(
                "connection {} play packet 0x2A with unexpected shape tolerated",
                session.id
            );
        }
        (ProtocolState::Play, 0x35) => {
            let mut reader = Reader::new(payload);
            if let Ok(slot) = reader.read_i16() {
                if reader.remaining() == 0 && (0..=8).contains(&slot) {
                    return Ok(PacketAction::SelectSlot(slot as u8));
                }
            }
            debug!(
                "connection {} held slot out of range, echoing current",
                session.id
            );
            return Ok(PacketAction::HeldEcho);
        }
        (ProtocolState::Play, 0x29) => {
            let mut reader = Reader::new(payload);
            if let (Ok(status), Ok(pos), Ok(face), Ok(sequence)) = (
                reader.read_varint(),
                reader.read_position(),
                reader.read_varint(),
                reader.read_varint(),
            ) {
                if status == 2 && (0..=5).contains(&face) {
                    return Ok(PacketAction::Edit(PendingEdit::Break {
                        x: pos.0,
                        y: pos.1,
                        z: pos.2,
                        sequence,
                    }));
                }
            }
            debug!(
                "connection {} play packet 0x29 with unexpected shape tolerated",
                session.id
            );
        }
        (ProtocolState::Play, 0x42) => {
            let mut reader = Reader::new(payload);
            if let (Ok(_hand), Ok(pos), Ok(face)) = (
                reader.read_varint(),
                reader.read_position(),
                reader.read_varint(),
            ) {
                if let (Ok(_cx), Ok(_cy), Ok(_cz), Ok(_inside), Ok(_border), Ok(sequence)) = (
                    reader.read_f32(),
                    reader.read_f32(),
                    reader.read_f32(),
                    reader.read_bool(),
                    reader.read_bool(),
                    reader.read_varint(),
                ) {
                    if (0..=5).contains(&face) {
                        return Ok(PacketAction::Edit(PendingEdit::Place {
                            x: pos.0,
                            y: pos.1,
                            z: pos.2,
                            face,
                            sequence,
                        }));
                    }
                }
            }
            debug!(
                "connection {} play packet 0x42 with unexpected shape tolerated",
                session.id
            );
        }
        (ProtocolState::Play, 0x12) => {
            let mut reader = Reader::new(payload);
            if let (Ok(window), Ok(state), Ok(slot), Ok(button), Ok(mode)) = (
                reader.read_varint(),
                reader.read_varint(),
                reader.read_i16(),
                reader.read_u8(),
                reader.read_varint(),
            ) {
                let count = reader.read_varint().unwrap_or(-1);
                let mut ok = count >= 0 && reader.remaining() > 0;
                for _ in 0..count.max(0) {
                    if reader.read_i16().is_err() || read_hashed_slot(&mut reader).is_err() {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    ok = read_hashed_slot(&mut reader).is_ok() && reader.remaining() == 0;
                }
                if ok && (0..=6).contains(&mode) {
                    return Ok(PacketAction::Click(PendingClick {
                        window,
                        state,
                        slot: i32::from(slot),
                        button,
                        mode,
                    }));
                }
            }
            debug!(
                "connection {} container click with unexpected shape tolerated",
                session.id
            );
        }
        // Unknown IDs are tolerated (debug-logged) so a vanilla client
        // sending newer/optional packets doesn't get disconnected.
        // Reaching play means config already succeeded — dropping here
        // shows up client-side as Connection reset.
        (ProtocolState::Play, _) => {
            debug!("connection {} play packet 0x{id:02X} tolerated", session.id);
        }
        _ => return Err(ProtoError::UnknownPacket(id)),
    }
    Ok(PacketAction::None)
}

fn collides(world: &World, x: f64, y: f64, z: f64) -> bool {
    let sample_y = (y + 0.1).floor() as i32;
    [
        (-PLAYER_HALF_WIDTH, -PLAYER_HALF_WIDTH),
        (PLAYER_HALF_WIDTH, -PLAYER_HALF_WIDTH),
        (-PLAYER_HALF_WIDTH, PLAYER_HALF_WIDTH),
        (PLAYER_HALF_WIDTH, PLAYER_HALF_WIDTH),
    ]
    .iter()
    .any(|(ox, oz)| {
        matches!(
            world.get_block(
                (x + ox).floor() as i32,
                sample_y,
                (z + oz).floor() as i32
            ),
            Some(block) if block.is_solid()
        )
    })
}

fn click_pickup(inventory: &mut Inventory, cursor: &mut ItemStack, slot: i32, button: u8) -> bool {
    if slot == -999 || !(0..crate::inventory::INVENTORY_SIZE as i32).contains(&slot) {
        return false;
    }
    let index = slot as usize;
    match button {
        0 => {
            if cursor.is_empty() {
                match inventory.take(index, u32::MAX) {
                    Some(taken) => {
                        *cursor = taken;
                        true
                    }
                    None => false,
                }
            } else {
                match inventory.get(index) {
                    None => false,
                    Some(current) if current.is_empty() => {
                        inventory.set(index, *cursor);
                        *cursor = ItemStack::empty();
                        true
                    }
                    Some(current) if current.item == cursor.item => {
                        let room = current.item.max_stack() - current.count;
                        let take = room.min(cursor.count);
                        if take == 0 {
                            return false;
                        }
                        inventory.set(
                            index,
                            ItemStack {
                                item: current.item,
                                count: current.count + take,
                            },
                        );
                        cursor.count -= take;
                        if cursor.count == 0 {
                            *cursor = ItemStack::empty();
                        }
                        true
                    }
                    Some(current) => {
                        inventory.set(index, *cursor);
                        *cursor = current;
                        true
                    }
                }
            }
        }
        1 => {
            if cursor.is_empty() {
                match inventory.get(index) {
                    Some(current) if !current.is_empty() => {
                        let half = current.count.div_ceil(2);
                        match inventory.take(index, half) {
                            Some(taken) => {
                                *cursor = taken;
                                true
                            }
                            None => false,
                        }
                    }
                    _ => false,
                }
            } else {
                match inventory.get(index) {
                    None => false,
                    Some(current) if current.is_empty() => {
                        inventory.set(
                            index,
                            ItemStack {
                                item: cursor.item,
                                count: 1,
                            },
                        );
                        cursor.count -= 1;
                        if cursor.count == 0 {
                            *cursor = ItemStack::empty();
                        }
                        true
                    }
                    Some(current)
                        if current.item == cursor.item
                            && current.count < current.item.max_stack() =>
                    {
                        inventory.set(
                            index,
                            ItemStack {
                                item: current.item,
                                count: current.count + 1,
                            },
                        );
                        cursor.count -= 1;
                        if cursor.count == 0 {
                            *cursor = ItemStack::empty();
                        }
                        true
                    }
                    _ => false,
                }
            }
        }
        _ => false,
    }
}

fn click_shift(inventory: &mut Inventory, slot: i32) -> bool {
    if !(0..crate::inventory::INVENTORY_SIZE as i32).contains(&slot) {
        return false;
    }
    let index = slot as usize;
    let (start, end) = if (9..36).contains(&index) {
        (crate::inventory::HOTBAR_START, 45)
    } else if (36..45).contains(&index) {
        (9, 36)
    } else {
        return false;
    };
    let moving = match inventory.get(index) {
        Some(stack) if !stack.is_empty() => stack,
        _ => return false,
    };
    inventory.clear(index);
    let mut rest = moving;
    for target in start..end {
        if rest.count == 0 {
            break;
        }
        if let Some(current) = inventory.get(target) {
            if current.item == rest.item && current.count < current.item.max_stack() {
                let room = current.item.max_stack() - current.count;
                let take = room.min(rest.count);
                inventory.set(
                    target,
                    ItemStack {
                        item: rest.item,
                        count: current.count + take,
                    },
                );
                rest.count -= take;
            }
        }
    }
    for target in start..end {
        if rest.count == 0 {
            break;
        }
        if inventory.get(target).is_some_and(|stack| stack.is_empty()) {
            let take = rest.item.max_stack().min(rest.count);
            inventory.set(
                target,
                ItemStack {
                    item: rest.item,
                    count: take,
                },
            );
            rest.count -= take;
        }
    }
    if rest.count > 0 {
        inventory.set(index, rest);
    }
    true
}

fn click_swap(inventory: &mut Inventory, slot: i32, button: u8) -> bool {
    if !(0..crate::inventory::INVENTORY_SIZE as i32).contains(&slot) {
        return false;
    }
    let other = match button {
        0..=8 => crate::inventory::HOTBAR_START + button as usize,
        40 => 45,
        _ => return false,
    };
    let index = slot as usize;
    let first = inventory.get(index).unwrap_or(ItemStack::empty());
    let second = inventory.get(other).unwrap_or(ItemStack::empty());
    inventory.set(index, second);
    inventory.set(other, first);
    true
}

fn send_response(conn: &mut Connection, response: &[u8]) {
    if !response.is_empty() {
        // Log packet ID (second byte after the varint length prefix)
        let pid = response.iter().skip(1).copied().next().unwrap_or(0xFF) & 0x7F;
        debug!(
            "conn {} send packet 0x{pid:02X} ({} bytes)",
            conn.id(),
            response.len()
        );
    }
    if conn.send(response).is_err() {
        conn.close();
    }
}

fn new_session_id() -> Result<u128, getrandom::Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)?;
    Ok(u128::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::super::proto::Writer;
    use super::*;

    fn test_status() -> ServerStatus {
        ServerStatus {
            motd: "Test Server".to_owned(),
            max_players: 20,
        }
    }

    fn test_stack() -> (NetworkManager, JavaHandler, std::net::SocketAddr) {
        let mut network = NetworkManager::new();
        network.bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let addr = network.local_addr().unwrap();
        (network, JavaHandler::new(test_status()), addr)
    }

    fn pump(stack: &mut (NetworkManager, JavaHandler, std::net::SocketAddr)) {
        stack.0.poll();
        stack.1.pump(&mut stack.0, 1);
    }

    fn wait_until(
        stack: &mut (NetworkManager, JavaHandler, std::net::SocketAddr),
        mut done: impl FnMut(&NetworkManager) -> bool,
    ) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !done(&stack.0) {
            assert!(Instant::now() < deadline, "timed out waiting for server");
            pump(stack);
            thread::sleep(Duration::from_millis(2));
        }
        pump(stack);
    }

    fn settle(stack: &mut (NetworkManager, JavaHandler, std::net::SocketAddr)) {
        for _ in 0..50 {
            pump(stack);
            thread::sleep(Duration::from_millis(2));
        }
    }

    fn join_play(
        stack: &mut (NetworkManager, JavaHandler, std::net::SocketAddr),
        client: &mut ScriptClient,
        name: &str,
    ) {
        join_play_uuid(
            stack,
            client,
            name,
            0x123E_4567_E89B_12D3_A456_4266_1417_4000,
        );
    }

    fn join_play_uuid(
        stack: &mut (NetworkManager, JavaHandler, std::net::SocketAddr),
        client: &mut ScriptClient,
        name: &str,
        uuid: u128,
    ) {
        client.send_handshake(776, 2);
        client.send_login_start_uuid(name, uuid);
        settle(stack);
        let (id, _) = client.read_packet();
        assert_eq!(id, 0x02);
        client.send_packet(0x03, &[]);
        settle(stack);
        let (id, _) = client.read_packet();
        assert_eq!(id, 0x0E);
        let mut packs = Writer::new();
        packs.write_varint(1);
        packs.write_string("minecraft");
        packs.write_string("core");
        packs.write_string("26.2");
        client.send_packet(0x07, &packs.into_bytes());
        settle(stack);
        for _ in 0..31 {
            client.read_packet();
        }
        client.send_packet(0x03, &[]);
        settle(stack);
    }

    struct ScriptClient {
        stream: TcpStream,
    }

    impl ScriptClient {
        fn connect(addr: std::net::SocketAddr) -> Self {
            let stream = TcpStream::connect(addr).unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            Self { stream }
        }

        fn send_packet(&mut self, id: i32, body: &[u8]) {
            let mut id_buf = Writer::new();
            id_buf.write_varint(id);
            let id_buf = id_buf.into_bytes();
            let mut packet = Writer::new();
            packet.write_varint(id_buf.len() as i32 + body.len() as i32);
            packet.write_bytes(&id_buf);
            packet.write_bytes(body);
            self.stream.write_all(&packet.into_bytes()).unwrap();
        }

        fn send_handshake(&mut self, protocol: i32, next: i32) {
            let mut body = Writer::new();
            body.write_varint(protocol);
            body.write_string("localhost");
            body.write_u16(25565);
            body.write_varint(next);
            self.send_packet(0x00, &body.into_bytes());
        }

        fn send_login_start(&mut self, name: &str) {
            self.send_login_start_uuid(name, 0x123E_4567_E89B_12D3_A456_4266_1417_4000);
        }

        fn send_login_start_uuid(&mut self, name: &str, uuid: u128) {
            let mut body = Writer::new();
            body.write_string(name);
            body.write_uuid(uuid);
            self.send_packet(0x00, &body.into_bytes());
        }

        fn read_packet(&mut self) -> (i32, Vec<u8>) {
            let prefixed = self.read_exact_prefixed();
            let mut reader = Reader::new(&prefixed);
            let id = reader.read_varint().unwrap();
            let payload = prefixed[reader.position()..].to_vec();
            (id, payload)
        }

        fn read_exact_prefixed(&mut self) -> Vec<u8> {
            let len = self.read_varint_from_stream() as usize;
            let mut buf = vec![0u8; len];
            self.stream.read_exact(&mut buf).unwrap();
            buf
        }

        fn read_varint_from_stream(&mut self) -> i32 {
            let mut value = 0u32;
            for i in 0..5 {
                let mut byte = [0u8; 1];
                self.stream.read_exact(&mut byte).unwrap();
                value = value.wrapping_add(((byte[0] & 0x7F) as u32).wrapping_shl(7 * i));
                if byte[0] & 0x80 == 0 {
                    return value as i32;
                }
            }
            panic!("varint too long");
        }

        fn expect_eof(&mut self) {
            let mut buf = [0u8; 1];
            assert_eq!(self.stream.read(&mut buf).unwrap(), 0);
        }
    }

    #[test]
    fn status_flow_returns_server_list_data() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_handshake(776, 1);
        client.send_packet(0x00, &[]);
        settle(&mut stack);
        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x00);
        let mut reader = Reader::new(&payload);
        let json = reader.read_string().unwrap();
        assert!(json.contains("\"name\":\"26.2\""));
        assert!(json.contains("\"protocol\":776"));
        assert!(json.contains("Test Server"));
        assert!(json.contains("\"max\":20"));

        let mut ping = Writer::new();
        ping.write_i64(987654321);
        client.send_packet(0x01, &ping.into_bytes());
        settle(&mut stack);
        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x01);
        assert_eq!(Reader::new(&payload).read_i64().unwrap(), 987654321);
    }

    #[test]
    fn login_flow_completes_and_acknowledges() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_handshake(776, 2);
        client.send_login_start("TestPlayer");
        settle(&mut stack);

        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x02);
        let mut reader = Reader::new(&payload);
        assert_eq!(
            reader.read_uuid().unwrap(),
            0x123E_4567_E89B_12D3_A456_4266_1417_4000
        );
        assert_eq!(reader.read_string().unwrap(), "TestPlayer");
        assert_eq!(reader.read_varint().unwrap(), 0); // properties count
        let session_id = reader.read_uuid().unwrap();
        assert_ne!(session_id, 0);
        assert_eq!(reader.remaining(), 0);
        assert_eq!(stack.1.player_count(), 1);

        client.send_packet(0x03, &[]);
        settle(&mut stack);
        assert!(stack
            .1
            .sessions
            .values()
            .any(|s| s.state == ProtocolState::Configuration));
        assert_eq!(stack.1.player_count(), 1);
    }

    #[test]
    fn play_entry_sends_login_spawn_and_position() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "PlayPlayer");

        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x31);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_i32().unwrap(), 1);
        assert!(!reader.read_bool().unwrap());
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.read_string().unwrap(), "minecraft:overworld");
        assert_eq!(reader.read_varint().unwrap(), 20);
        assert_eq!(reader.read_varint().unwrap(), 10);
        assert_eq!(reader.read_varint().unwrap(), 10);
        assert!(!reader.read_bool().unwrap());
        assert!(reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_string().unwrap(), "minecraft:overworld");
        assert_eq!(reader.read_i64().unwrap(), 0);
        assert_eq!(reader.read_u8().unwrap(), 0);
        assert_eq!(reader.read_i8().unwrap(), -1);
        assert!(!reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 63);
        assert!(!reader.read_bool().unwrap());
        assert!(!reader.read_bool().unwrap());
        assert_eq!(reader.remaining(), 0);

        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x61);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_string().unwrap(), "minecraft:overworld");
        assert_eq!(reader.read_position().unwrap(), (0, 100, 0));
        assert_eq!(reader.read_f32().unwrap(), 0.0); // yaw
        assert_eq!(reader.read_f32().unwrap(), 0.0); // pitch
        assert_eq!(reader.remaining(), 0);

        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x48);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.read_f64().unwrap(), 0.0);
        assert_eq!(reader.read_f64().unwrap(), 100.0);
        assert_eq!(reader.read_f64().unwrap(), 0.0);
        assert_eq!(reader.read_f64().unwrap(), 0.0); // velocity x
        assert_eq!(reader.read_f64().unwrap(), 0.0); // velocity y
        assert_eq!(reader.read_f64().unwrap(), 0.0); // velocity z
        assert_eq!(reader.read_f32().unwrap(), 0.0); // yaw
        assert_eq!(reader.read_f32().unwrap(), 0.0); // pitch
        assert_eq!(reader.read_i32().unwrap(), 0); // flags
        assert_eq!(reader.remaining(), 0);

        assert!(stack
            .1
            .sessions
            .values()
            .any(|s| s.state == ProtocolState::Play));
        assert_eq!(
            stack
                .1
                .players
                .find_by_name("PlayPlayer")
                .unwrap()
                .entity_id,
            1
        );
    }

    #[test]
    fn teleport_confirm_is_recorded() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "TeleportPlayer");
        for _ in 0..3 {
            client.read_packet();
        }

        let mut body = Writer::new();
        body.write_varint(1);
        client.send_packet(0x00, &body.into_bytes());
        settle(&mut stack);
        assert_eq!(stack.1.sessions.get(&1).unwrap().confirmed_teleport, 1);
        assert_eq!(stack.1.player_count(), 1);
    }

    #[test]
    fn routine_play_packets_are_tolerated() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "ChattyPlayer");
        for _ in 0..3 {
            client.read_packet();
        }

        client.send_packet(0x0E, b"client-information");
        client.send_packet(0x16, b"minecraft:brand");
        client.send_packet(0x09, b"hello everyone");
        let mut movement = Writer::new();
        movement.write_f64(1.0);
        movement.write_f64(100.0);
        movement.write_f64(2.0);
        movement.write_u8(1);
        client.send_packet(0x1E, &movement.into_bytes());
        client.send_packet(0x2B, b"input-bytes");
        let mut keepalive = Writer::new();
        keepalive.write_i64(555);
        client.send_packet(0x1C, &keepalive.into_bytes());
        settle(&mut stack);

        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut buf = [0u8; 1];
        let err = client.stream.read(&mut buf).unwrap_err();
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
        assert!(stack
            .1
            .sessions
            .values()
            .any(|s| s.state == ProtocolState::Play));
    }

    #[test]
    fn unknown_play_packet_disconnects() {
        // Play-phase packets are tolerated now (no world sim yet):
        // unknown IDs must NOT drop the connection (that surfaces as
        // Connection reset once config has succeeded).
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "DoomedPlayer");
        for _ in 0..3 {
            client.read_packet();
        }

        client.send_packet(0x7F, b"nonsense");
        settle(&mut stack);
        assert_eq!(stack.0.connection_count(), 1);
        assert_eq!(stack.1.player_count(), 1);
        assert_eq!(stack.1.session_count(), 1);
    }

    #[test]
    fn keepalive_sent_on_schedule_in_play() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "IdlePlayer");
        for _ in 0..3 {
            client.read_packet();
        }

        stack.0.poll();
        stack.1.pump(&mut stack.0, 200);
        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x2C);
        assert_eq!(Reader::new(&payload).read_i64().unwrap(), 200);

        stack.0.poll();
        stack.1.pump(&mut stack.0, 201);
        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut buf = [0u8; 1];
        let err = client.stream.read(&mut buf).unwrap_err();
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
    }

    #[test]
    fn full_lifecycle_to_disconnect() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "ShortLivedPlayer");
        for _ in 0..3 {
            client.read_packet();
        }
        assert!(stack
            .1
            .sessions
            .values()
            .any(|s| s.state == ProtocolState::Play));
        assert_eq!(stack.1.player_count(), 1);

        drop(client);
        wait_until(&mut stack, |n| n.connection_count() == 0);
        assert_eq!(stack.1.player_count(), 0);
        assert_eq!(stack.1.session_count(), 0);
    }

    #[test]
    fn configuration_flow_completes() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_handshake(776, 2);
        client.send_login_start("ConfigPlayer");
        settle(&mut stack);
        let (id, _) = client.read_packet();
        assert_eq!(id, 0x02);
        client.send_packet(0x03, &[]);
        settle(&mut stack);

        let (id, _) = client.read_packet();
        assert_eq!(id, 0x0E);

        let packs = {
            let mut packs = Writer::new();
            packs.write_varint(1);
            packs.write_string("minecraft");
            packs.write_string("core");
            packs.write_string("26.2");
            packs.into_bytes()
        };
        client.send_packet(0x07, &packs);
        settle(&mut stack);

        let mut ids = Vec::new();
        for _ in 0..31 {
            let (id, _) = client.read_packet();
            ids.push(id);
        }
        assert_eq!(ids.len(), 31);
        assert!(ids[..29].iter().all(|&id| id == 0x07));
        assert_eq!(ids[29], 0x0D);
        assert_eq!(ids[30], 0x03);

        client.send_packet(0x03, &[]);
        settle(&mut stack);
        assert_eq!(stack.1.player_count(), 1);

        let mut ids = Vec::new();
        for _ in 0..3 {
            let (id, _) = client.read_packet();
            ids.push(id);
        }
        assert_eq!(ids, vec![0x31, 0x61, 0x48]);

        client.send_packet(0x07, &packs);
        settle(&mut stack);
        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut buf = [0u8; 1];
        let err = client.stream.read(&mut buf).unwrap_err();
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
    }

    #[test]
    fn play_entry_streams_spawn_chunks_once() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "ChunkPlayer");
        for _ in 0..3 {
            client.read_packet();
        }
        let mut worlds = WorldManager::create_default();
        stack.1.pump_chunks(&mut stack.0, &mut worlds);
        // The multi-megabyte chunk burst exceeds socket buffers, so keep
        // flushing from another thread while the main thread reads.
        let side = 2 * CHUNK_RADIUS + 1;
        std::thread::scope(|s| {
            s.spawn(|| {
                for _ in 0..3000 {
                    stack.0.poll();
                    thread::sleep(Duration::from_millis(2));
                }
            });

            let (id, payload) = client.read_packet();
            assert_eq!(id, 0x26);
            assert_eq!(payload[0], 13);

            let (id, payload) = client.read_packet();
            assert_eq!(id, 0x5E);
            let mut reader = Reader::new(&payload);
            assert_eq!(reader.read_varint().unwrap(), 0);
            assert_eq!(reader.read_varint().unwrap(), 0);

            let (id, _) = client.read_packet();
            assert_eq!(id, 0x0C);

            for _ in 0..side * side {
                let (id, payload) = client.read_packet();
                assert_eq!(id, 0x2D);
                let mut reader = Reader::new(&payload);
                let x = reader.read_i32().unwrap();
                let z = reader.read_i32().unwrap();
                assert!(x.abs() <= CHUNK_RADIUS);
                assert!(z.abs() <= CHUNK_RADIUS);
            }

            let (id, payload) = client.read_packet();
            assert_eq!(id, 0x0B);
            assert_eq!(Reader::new(&payload).read_varint().unwrap(), side * side);
        });
        assert_eq!(
            worlds.get(DEFAULT_WORLD_NAME).unwrap().loaded_chunk_count() as i32,
            side * side
        );

        stack.1.pump_chunks(&mut stack.0, &mut worlds);
        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut buf = [0u8; 1];
        let err = client.stream.read(&mut buf).unwrap_err();
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
    }

    fn send_move(client: &mut ScriptClient, id: i32, body: &[u8]) {
        client.send_packet(id, body);
    }

    #[test]
    fn movement_packets_update_player_position() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "WalkerPlayer");
        for _ in 0..3 {
            client.read_packet();
        }

        let mut body = Writer::new();
        body.write_f64(2.5);
        body.write_f64(99.0);
        body.write_f64(-1.25);
        body.write_f32(90.0);
        body.write_f32(-5.0);
        body.write_u8(1);
        send_move(&mut client, 0x1F, &body.into_bytes());
        settle(&mut stack);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!((player.x, player.y, player.z), (2.5, 99.0, -1.25));
        assert_eq!((player.yaw, player.pitch), (90.0, -5.0));

        let mut body = Writer::new();
        body.write_f32(180.0);
        body.write_f32(0.0);
        body.write_u8(1);
        send_move(&mut client, 0x20, &body.into_bytes());
        settle(&mut stack);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!((player.x, player.y, player.z), (2.5, 99.0, -1.25));
        assert_eq!((player.yaw, player.pitch), (180.0, 0.0));

        let mut body = Writer::new();
        body.write_f64(12.5);
        body.write_f64(99.0);
        body.write_f64(-1.25);
        body.write_u8(1);
        send_move(&mut client, 0x1E, &body.into_bytes());
        settle(&mut stack);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!(player.x, 12.5);
        assert_eq!((player.yaw, player.pitch), (180.0, 0.0));
    }

    #[test]
    fn moving_to_new_chunk_streams_and_unloads() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "ExplorerPlayer");
        for _ in 0..3 {
            client.read_packet();
        }
        let mut worlds = WorldManager::create_default();
        stack.1.pump_chunks(&mut stack.0, &mut worlds);
        let side = 2 * CHUNK_RADIUS + 1;
        std::thread::scope(|s| {
            s.spawn(|| {
                for _ in 0..4000 {
                    stack.0.poll();
                    thread::sleep(Duration::from_millis(2));
                }
            });
            for _ in 0..2 + side * side + 1 {
                client.read_packet();
            }
            let (id, _) = client.read_packet();
            assert_eq!(id, 0x0B);
        });

        for x in [10.0, 20.0, 30.0, 40.0] {
            let mut body = Writer::new();
            body.write_f64(x);
            body.write_f64(100.0);
            body.write_f64(0.0);
            body.write_f32(0.0);
            body.write_f32(0.0);
            body.write_u8(1);
            send_move(&mut client, 0x1F, &body.into_bytes());
        }
        settle(&mut stack);
        assert_eq!(stack.1.players.get(1).unwrap().x, 40.0);
        stack.1.pump_chunks(&mut stack.0, &mut worlds);

        let added: i32 = 2 * side;
        std::thread::scope(|s| {
            s.spawn(|| {
                for _ in 0..3000 {
                    stack.0.poll();
                    thread::sleep(Duration::from_millis(2));
                }
            });
            let (id, payload) = client.read_packet();
            assert_eq!(id, 0x5E);
            let mut reader = Reader::new(&payload);
            assert_eq!(reader.read_varint().unwrap(), 2);
            assert_eq!(reader.read_varint().unwrap(), 0);

            let (id, _) = client.read_packet();
            assert_eq!(id, 0x0C);

            for _ in 0..added {
                let (id, _) = client.read_packet();
                assert_eq!(id, 0x25);
            }
            for _ in 0..added {
                let (id, _) = client.read_packet();
                assert_eq!(id, 0x2D);
            }
            let (id, payload) = client.read_packet();
            assert_eq!(id, 0x0B);
            assert_eq!(Reader::new(&payload).read_varint().unwrap(), added);
        });

        stack.1.pump_chunks(&mut stack.0, &mut worlds);
        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut buf = [0u8; 1];
        let err = client.stream.read(&mut buf).unwrap_err();
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
    }

    fn drain_stream(
        stack: &mut (NetworkManager, JavaHandler, std::net::SocketAddr),
        client: &mut ScriptClient,
    ) {
        let side = 2 * CHUNK_RADIUS + 1;
        std::thread::scope(|s| {
            s.spawn(|| {
                for _ in 0..4000 {
                    stack.0.poll();
                    thread::sleep(Duration::from_millis(2));
                }
            });
            for _ in 0..3 + side * side + 1 {
                client.read_packet();
            }
        });
    }

    #[test]
    fn physics_falls_lands_and_resets_velocity() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "FallingPlayer");
        for _ in 0..3 {
            client.read_packet();
        }
        let mut worlds = WorldManager::create_default();
        stack.1.pump_chunks(&mut stack.0, &mut worlds);
        drain_stream(&mut stack, &mut client);

        let mut speeds = Vec::new();
        for _ in 0..300 {
            stack.1.tick_physics(&mut stack.0, &worlds);
            let player = stack.1.players.get(1).unwrap();
            if speeds.len() < 3 {
                speeds.push(player.vy);
            }
            if player.on_ground {
                break;
            }
        }
        assert!(speeds[1] < speeds[0]);
        assert!(speeds[1] < 0.0);
        let player = stack.1.players.get(1).unwrap();
        assert!(player.on_ground);
        assert_eq!(player.y, 65.0);
        assert_eq!(player.vy, 0.0);

        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut buf = [0u8; 1];
        let err = client.stream.read(&mut buf).unwrap_err();
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
    }

    #[test]
    fn physics_corrects_clipped_player_with_teleport() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "ClippedPlayer");
        for _ in 0..3 {
            client.read_packet();
        }
        let mut worlds = WorldManager::create_default();
        stack.1.pump_chunks(&mut stack.0, &mut worlds);
        drain_stream(&mut stack, &mut client);

        stack.1.players.get_mut(1).unwrap().y = 60.0;
        stack.1.tick_physics(&mut stack.0, &worlds);
        assert_eq!(stack.1.players.get(1).unwrap().y, 65.0);

        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x48);
        let mut reader = Reader::new(&payload);
        reader.read_varint().unwrap();
        assert_eq!(reader.read_f64().unwrap(), 0.0);
        assert_eq!(reader.read_f64().unwrap(), 65.0);
        assert_eq!(reader.read_f64().unwrap(), 0.0);
    }

    #[test]
    fn physics_rejects_absurd_nan_and_accepts_negative() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut client, "PickyPlayer");
        for _ in 0..3 {
            client.read_packet();
        }

        let mut body = Writer::new();
        body.write_f64(1.0e9);
        body.write_f64(100.0);
        body.write_f64(0.0);
        body.write_u8(1);
        send_move(&mut client, 0x1E, &body.into_bytes());
        settle(&mut stack);
        assert_eq!(stack.1.players.get(1).unwrap().x, 0.0);

        let mut body = Writer::new();
        body.write_f64(f64::NAN);
        body.write_f64(100.0);
        body.write_f64(0.0);
        body.write_u8(1);
        send_move(&mut client, 0x1E, &body.into_bytes());
        settle(&mut stack);
        assert_eq!(stack.1.players.get(1).unwrap().x, 0.0);

        let mut body = Writer::new();
        body.write_f64(-5.0);
        body.write_f64(99.0);
        body.write_f64(-8.0);
        body.write_u8(1);
        send_move(&mut client, 0x1E, &body.into_bytes());
        settle(&mut stack);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!((player.x, player.z), (-5.0, -8.0));
        assert_eq!(player.as_entity().chunk_pos(), ChunkPos::new(-1, -1));
    }

    #[test]
    fn two_players_stay_independent() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut first = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        join_play(&mut stack, &mut first, "FirstPlayer");
        for _ in 0..3 {
            first.read_packet();
        }
        let mut second = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 2);
        join_play_uuid(
            &mut stack,
            &mut second,
            "SecondPlayer",
            0x123E_4567_E89B_12D3_A456_4266_1417_4001,
        );
        for _ in 0..3 {
            second.read_packet();
        }

        assert_eq!(stack.1.player_count(), 2);
        let first_id = stack
            .1
            .players
            .find_by_name("FirstPlayer")
            .unwrap()
            .entity_id;
        let second_id = stack
            .1
            .players
            .find_by_name("SecondPlayer")
            .unwrap()
            .entity_id;
        assert_ne!(first_id, second_id);

        let mut body = Writer::new();
        body.write_f64(2.5);
        body.write_f64(99.5);
        body.write_f64(-1.5);
        body.write_u8(1);
        send_move(&mut first, 0x1E, &body.into_bytes());
        settle(&mut stack);
        let moved = stack.1.players.find_by_name("FirstPlayer").unwrap();
        assert_eq!(moved.x, 2.5);
        let other = stack.1.players.find_by_name("SecondPlayer").unwrap();
        assert_eq!((other.x, other.y, other.z), (0.0, 100.0, 0.0));

        drop(first);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        assert_eq!(stack.1.player_count(), 1);
        assert!(stack.1.players.find_by_name("FirstPlayer").is_none());
        assert!(stack.1.players.find_by_name("SecondPlayer").is_some());
    }

    fn join_pair(
        stack: &mut (NetworkManager, JavaHandler, std::net::SocketAddr),
    ) -> (ScriptClient, ScriptClient) {
        let addr = stack.2;
        let mut first = ScriptClient::connect(addr);
        wait_until(stack, |n| n.connection_count() == 1);
        join_play(stack, &mut first, "FirstPlayer");
        for _ in 0..3 {
            first.read_packet();
        }
        let mut second = ScriptClient::connect(addr);
        wait_until(stack, |n| n.connection_count() == 2);
        join_play_uuid(
            stack,
            &mut second,
            "SecondPlayer",
            0x123E_4567_E89B_12D3_A456_4266_1417_4001,
        );
        for _ in 0..3 {
            second.read_packet();
        }
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x46);
        (first, second)
    }

    fn seed_visible(
        stack: &mut (NetworkManager, JavaHandler, std::net::SocketAddr),
        id: ConnectionId,
    ) {
        stack
            .1
            .sessions
            .get_mut(&id)
            .unwrap()
            .sent_chunks
            .insert(ChunkPos::new(0, 0));
    }

    #[test]
    fn visibility_spawns_both_ways_once() {
        let mut stack = test_stack();
        let (mut first, mut second) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        seed_visible(&mut stack, 2);
        stack.1.pump_visibility(&mut stack.0);

        for client in [&mut first, &mut second] {
            let (id, _) = client.read_packet();
            assert_eq!(id, 0x46);
            let (id, _) = client.read_packet();
            assert_eq!(id, 0x01);
            let (id, _) = client.read_packet();
            assert_eq!(id, 0x63);
        }
        assert_eq!(stack.1.sessions.get(&1).unwrap().tracked.len(), 1);
        assert_eq!(stack.1.sessions.get(&2).unwrap().tracked.len(), 1);

        stack.1.pump_visibility(&mut stack.0);
        for client in [&mut first, &mut second] {
            client
                .stream
                .set_read_timeout(Some(Duration::from_millis(200)))
                .unwrap();
            let mut buf = [0u8; 1];
            let err = client.stream.read(&mut buf).unwrap_err();
            assert!(matches!(
                err.kind(),
                std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
            ));
        }
    }

    #[test]
    fn visibility_syncs_moves_rotations_and_teleports() {
        let mut stack = test_stack();
        let (mut first, mut second) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        seed_visible(&mut stack, 2);
        stack
            .1
            .sessions
            .get_mut(&1)
            .unwrap()
            .sent_chunks
            .insert(ChunkPos::new(1, 0));
        stack.1.pump_visibility(&mut stack.0);
        for client in [&mut first, &mut second] {
            for _ in 0..3 {
                client.read_packet();
            }
        }

        stack.1.players.get_mut(2).unwrap().x = 1.0;
        stack.1.pump_visibility(&mut stack.0);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x35);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 2);
        assert_eq!(reader.read_i16().unwrap(), 4096);
        assert_eq!(reader.read_i16().unwrap(), 0);
        assert_eq!(reader.read_i16().unwrap(), 0);

        stack.1.players.get_mut(2).unwrap().yaw = 90.0;
        stack.1.pump_visibility(&mut stack.0);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x38);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x53);
        assert_eq!(Reader::new(&payload).read_varint().unwrap(), 2);

        stack.1.players.get_mut(2).unwrap().x = 20.0;
        stack.1.pump_visibility(&mut stack.0);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x7D);
        assert_eq!(Reader::new(&payload).read_varint().unwrap(), 2);
        assert_eq!(Reader::new(&payload[1..9]).read_f64().unwrap(), 20.0);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x53);
        assert_eq!(Reader::new(&payload).read_varint().unwrap(), 2);
    }

    #[test]
    fn visibility_despawns_out_of_range_and_respawns() {
        let mut stack = test_stack();
        let (mut first, mut second) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        seed_visible(&mut stack, 2);
        stack.1.pump_visibility(&mut stack.0);
        for client in [&mut first, &mut second] {
            for _ in 0..3 {
                client.read_packet();
            }
        }

        stack.1.players.get_mut(2).unwrap().x = 5000.0;
        stack.1.pump_visibility(&mut stack.0);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x4D);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.read_varint().unwrap(), 2);
        assert!(stack.1.sessions.get(&1).unwrap().tracked.is_empty());

        stack.1.players.get_mut(2).unwrap().x = 0.0;
        stack.1.pump_visibility(&mut stack.0);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x46);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x01);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x63);
        assert_eq!(stack.1.sessions.get(&1).unwrap().tracked.len(), 1);
    }

    #[test]
    fn sneak_updates_metadata_and_pose() {
        let mut stack = test_stack();
        let (mut first, mut second) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        seed_visible(&mut stack, 2);
        stack.1.pump_visibility(&mut stack.0);
        for client in [&mut first, &mut second] {
            for _ in 0..3 {
                client.read_packet();
            }
        }

        let mut body = Writer::new();
        body.write_varint(2);
        body.write_varint(0);
        body.write_varint(0);
        second.send_packet(0x2A, &body.into_bytes());
        settle(&mut stack);
        assert!(stack.1.players.get(2).unwrap().sneaking);
        stack.1.pump_visibility(&mut stack.0);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x63);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 2);
        let entries = crate::java::metadata::decode_metadata(&mut reader).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(
            entries[0],
            crate::java::metadata::MetadataEntry {
                index: 0,
                kind: crate::java::metadata::MetadataKind::Byte(0x02),
            }
        );

        let mut body = Writer::new();
        body.write_varint(2);
        body.write_varint(1);
        body.write_varint(0);
        second.send_packet(0x2A, &body.into_bytes());
        settle(&mut stack);
        assert!(!stack.1.players.get(2).unwrap().sneaking);
        stack.1.pump_visibility(&mut stack.0);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x63);
    }

    #[test]
    fn malformed_action_packet_is_tolerated() {
        let mut stack = test_stack();
        let (_first, mut second) = join_pair(&mut stack);

        second.send_packet(0x2A, &[0x02]);
        let mut body = Writer::new();
        body.write_varint(2);
        body.write_varint(99);
        body.write_varint(0);
        second.send_packet(0x2A, &body.into_bytes());
        settle(&mut stack);
        assert_eq!(stack.0.connection_count(), 2);
        assert_eq!(stack.1.player_count(), 2);
        assert!(!stack.1.players.get(2).unwrap().sneaking);
    }

    #[test]
    fn disconnect_cleans_visibility_and_reconnects_fresh() {
        let mut stack = test_stack();
        let (mut first, mut second) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        seed_visible(&mut stack, 2);
        stack.1.pump_visibility(&mut stack.0);
        for client in [&mut first, &mut second] {
            for _ in 0..3 {
                client.read_packet();
            }
        }
        let old_uuid = stack.1.players.get(2).unwrap().display_uuid();
        let old_entity = stack.1.players.get(2).unwrap().entity_id;

        drop(second);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        stack.1.pump_visibility(&mut stack.0);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x45);
        assert_eq!(Reader::new(&payload).read_varint().unwrap(), 1);
        assert_eq!(Reader::new(&payload[1..17]).read_uuid().unwrap(), old_uuid);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x4D);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.read_varint().unwrap(), old_entity);
        assert!(stack.1.sessions.get(&1).unwrap().tracked.is_empty());

        let addr = stack.2;
        let mut third = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 2);
        join_play_uuid(
            &mut stack,
            &mut third,
            "SecondPlayer",
            0x123E_4567_E89B_12D3_A456_4266_1417_4002,
        );
        for _ in 0..3 {
            third.read_packet();
        }
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x46);
        seed_visible(&mut stack, 3);
        stack.1.pump_visibility(&mut stack.0);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x46);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x01);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x63);
        let new_entity = stack
            .1
            .players
            .find_by_name("SecondPlayer")
            .unwrap()
            .entity_id;
        assert_ne!(new_entity, old_entity);
        assert_eq!(stack.1.sessions.get(&1).unwrap().tracked.len(), 1);
        for _ in 0..3 {
            third.read_packet();
        }
    }

    fn send_dig(
        client: &mut ScriptClient,
        status: i32,
        x: i32,
        y: i32,
        z: i32,
        face: i32,
        seq: i32,
    ) {
        let mut body = Writer::new();
        body.write_varint(status);
        body.write_position(x, y, z);
        body.write_varint(face);
        body.write_varint(seq);
        client.send_packet(0x29, &body.into_bytes());
    }

    fn send_place(client: &mut ScriptClient, x: i32, y: i32, z: i32, face: i32, seq: i32) {
        let mut body = Writer::new();
        body.write_varint(0);
        body.write_position(x, y, z);
        body.write_varint(face);
        body.write_f32(0.5);
        body.write_f32(0.5);
        body.write_f32(0.5);
        body.write_bool(false);
        body.write_bool(false);
        body.write_varint(seq);
        client.send_packet(0x42, &body.into_bytes());
    }

    fn expect_silence(client: &mut ScriptClient) {
        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut buf = [0u8; 1];
        let err = client.stream.read(&mut buf).unwrap_err();
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
        client
            .stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
    }

    fn lowered_worlds(
        stack: &mut (NetworkManager, JavaHandler, std::net::SocketAddr),
    ) -> WorldManager {
        stack.1.players.get_mut(1).unwrap().y = 68.0;
        let mut worlds = WorldManager::create_default();
        worlds
            .get_mut(crate::world::DEFAULT_WORLD_NAME)
            .unwrap()
            .load_chunk(ChunkPos::new(0, 0));
        worlds
    }

    #[test]
    fn breaking_sets_air_acks_and_broadcasts() {
        let mut stack = test_stack();
        let (mut first, mut second) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        seed_visible(&mut stack, 2);
        let mut worlds = lowered_worlds(&mut stack);

        send_dig(&mut first, 2, 0, 64, 0, 1, 7);
        settle(&mut stack);
        stack.1.pump_blocks(&mut stack.0, &mut worlds);
        assert_eq!(
            worlds
                .get(crate::world::DEFAULT_WORLD_NAME)
                .unwrap()
                .get_block(0, 64, 0),
            Some(crate::world::Block::Air)
        );

        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x04);
        assert_eq!(Reader::new(&payload).read_varint().unwrap(), 7);
        for client in [&mut first, &mut second] {
            let (id, payload) = client.read_packet();
            assert_eq!(id, 0x08);
            let mut reader = Reader::new(&payload);
            assert_eq!(reader.read_position().unwrap(), (0, 64, 0));
            assert_eq!(reader.read_varint().unwrap(), 0);
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn breaking_ignores_air_unloaded_and_far_blocks() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        let mut worlds = lowered_worlds(&mut stack);

        send_dig(&mut first, 2, 0, 70, 0, 1, 1);
        settle(&mut stack);
        stack.1.pump_blocks(&mut stack.0, &mut worlds);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x08);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_position().unwrap(), (0, 70, 0));
        assert_eq!(reader.read_varint().unwrap(), 0);

        send_dig(&mut first, 2, 500, 64, 500, 1, 2);
        settle(&mut stack);
        stack.1.pump_blocks(&mut stack.0, &mut worlds);
        expect_silence(&mut first);

        stack.1.players.get_mut(1).unwrap().x = 100.0;
        send_dig(&mut first, 2, 0, 64, 0, 1, 3);
        settle(&mut stack);
        stack.1.pump_blocks(&mut stack.0, &mut worlds);
        expect_silence(&mut first);
        assert_eq!(
            worlds
                .get(crate::world::DEFAULT_WORLD_NAME)
                .unwrap()
                .get_block(0, 64, 0),
            Some(crate::world::Block::GrassBlock)
        );
    }

    #[test]
    fn placing_needs_air_face_and_reach() {
        let mut stack = test_stack();
        let (mut first, mut second) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        seed_visible(&mut stack, 2);
        let mut worlds = lowered_worlds(&mut stack);

        send_place(&mut first, 0, 64, 0, 1, 11);
        settle(&mut stack);
        stack.1.pump_blocks(&mut stack.0, &mut worlds);
        assert_eq!(
            worlds
                .get(crate::world::DEFAULT_WORLD_NAME)
                .unwrap()
                .get_block(0, 65, 0),
            Some(crate::world::Block::Stone)
        );
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x04);
        assert_eq!(Reader::new(&payload).read_varint().unwrap(), 11);
        for client in [&mut first, &mut second] {
            let (id, payload) = client.read_packet();
            assert_eq!(id, 0x08);
            let mut reader = Reader::new(&payload);
            assert_eq!(reader.read_position().unwrap(), (0, 65, 0));
            assert_eq!(reader.read_varint().unwrap(), 1);
        }

        send_place(&mut first, 0, 63, 0, 1, 12);
        settle(&mut stack);
        stack.1.pump_blocks(&mut stack.0, &mut worlds);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x08);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_position().unwrap(), (0, 64, 0));
        assert_eq!(reader.read_varint().unwrap(), 8);

        send_place(&mut first, 0, 67, 0, 1, 13);
        settle(&mut stack);
        stack.1.pump_blocks(&mut stack.0, &mut worlds);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x08);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_position().unwrap(), (0, 68, 0));
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(
            worlds
                .get(crate::world::DEFAULT_WORLD_NAME)
                .unwrap()
                .get_block(0, 68, 0),
            Some(crate::world::Block::Air)
        );

        stack.1.pump_inventory(&mut stack.0);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x12);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.read_varint().unwrap(), 46);
        for _ in 0..36 {
            assert_eq!(reader.read_varint().unwrap(), 0);
        }
        assert_eq!(reader.read_varint().unwrap(), 63);
    }

    #[test]
    fn broadcast_skips_viewers_without_the_chunk() {
        let mut stack = test_stack();
        let (mut first, mut second) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        let mut worlds = lowered_worlds(&mut stack);

        send_dig(&mut first, 2, 0, 64, 0, 1, 21);
        settle(&mut stack);
        stack.1.pump_blocks(&mut stack.0, &mut worlds);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x04);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x08);
        expect_silence(&mut second);
    }

    fn send_held(client: &mut ScriptClient, slot: i16) {
        let mut body = Writer::new();
        body.write_i16(slot);
        client.send_packet(0x35, &body.into_bytes());
    }

    #[test]
    fn held_slot_selects_and_rejects_safely() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);

        send_held(&mut first, 2);
        settle(&mut stack);
        assert_eq!(stack.1.players.get(1).unwrap().inventory.selected(), 2);
        assert_eq!(
            stack.1.players.get(1).unwrap().held_item().item,
            crate::inventory::Item::GrassBlock
        );
        stack.1.pump_inventory(&mut stack.0);
        let (id, _) = first.read_packet();
        assert_eq!(id, 0x12);
        expect_silence(&mut first);

        send_held(&mut first, 9);
        settle(&mut stack);
        assert_eq!(stack.1.players.get(1).unwrap().inventory.selected(), 2);
        stack.1.pump_inventory(&mut stack.0);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x69);
        assert_eq!(Reader::new(&payload).read_varint().unwrap(), 2);
    }

    #[test]
    fn initial_inventory_syncs_on_play_entry() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x12);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.read_varint().unwrap(), 1);
        assert_eq!(reader.read_varint().unwrap(), 46);
        let mut counts = Vec::new();
        for _ in 0..46 {
            let count = reader.read_varint().unwrap();
            if count > 0 {
                counts.push((count, reader.read_varint().unwrap()));
                assert_eq!(reader.read_varint().unwrap(), 0);
                assert_eq!(reader.read_varint().unwrap(), 0);
            }
        }
        assert_eq!(
            counts,
            vec![
                (64, crate::inventory::Item::Stone.id()),
                (64, crate::inventory::Item::Dirt.id()),
                (64, crate::inventory::Item::GrassBlock.id())
            ]
        );
        assert_eq!(reader.read_varint().unwrap(), 0);
        assert_eq!(reader.remaining(), 0);

        stack.1.pump_inventory(&mut stack.0);
        expect_silence(&mut first);
    }

    #[test]
    fn empty_hand_cannot_place() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        let mut worlds = lowered_worlds(&mut stack);

        send_held(&mut first, 5);
        settle(&mut stack);
        assert!(stack.1.players.get(1).unwrap().held_item().is_empty());
        send_place(&mut first, 0, 64, 0, 1, 31);
        settle(&mut stack);
        stack.1.pump_blocks(&mut stack.0, &mut worlds);
        expect_silence(&mut first);
        assert_eq!(
            worlds
                .get(crate::world::DEFAULT_WORLD_NAME)
                .unwrap()
                .get_block(0, 65, 0),
            Some(crate::world::Block::Air)
        );
        assert_eq!(
            stack.1.players.get(1).unwrap().held_item(),
            crate::inventory::ItemStack::empty()
        );
    }

    fn write_hashed(body: &mut Writer, present: bool, id: i32, count: i32) {
        body.write_bool(present);
        if present {
            body.write_varint(id);
            body.write_varint(count);
            body.write_varint(0);
            body.write_varint(0);
        }
    }

    fn send_click(
        client: &mut ScriptClient,
        window: i32,
        state: i32,
        slot: i16,
        button: u8,
        mode: i32,
    ) {
        let mut body = Writer::new();
        body.write_varint(window);
        body.write_varint(state);
        body.write_i16(slot);
        body.write_u8(button);
        body.write_varint(mode);
        body.write_varint(0);
        write_hashed(&mut body, false, 0, 0);
        client.send_packet(0x12, &body.into_bytes());
    }

    fn read_slot(reader: &mut Reader) -> (i32, i32) {
        let count = reader.read_varint().unwrap();
        if count > 0 {
            let id = reader.read_varint().unwrap();
            reader.read_varint().unwrap();
            reader.read_varint().unwrap();
            (count, id)
        } else {
            (0, -1)
        }
    }

    fn read_content(client: &mut ScriptClient) -> (i32, Vec<(i32, i32)>, (i32, i32)) {
        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x12);
        let mut reader = Reader::new(&payload);
        assert_eq!(reader.read_varint().unwrap(), 0);
        let state = reader.read_varint().unwrap();
        assert_eq!(reader.read_varint().unwrap(), 46);
        let mut slots = Vec::new();
        for _ in 0..46 {
            slots.push(read_slot(&mut reader));
        }
        let cursor = read_slot(&mut reader);
        assert_eq!(reader.remaining(), 0);
        (state, slots, cursor)
    }

    #[test]
    fn click_pickup_and_place_with_cursor() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        let (state, _, _) = read_content(&mut first);
        assert_eq!(state, 1);

        send_click(&mut first, 0, 1, 36, 0, 0);
        settle(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!(
            player.cursor,
            crate::inventory::ItemStack::new(crate::inventory::Item::Stone, 64).unwrap()
        );
        let (state, slots, cursor) = read_content(&mut first);
        assert_eq!(state, 2);
        assert_eq!(slots[36], (0, -1));
        assert_eq!(cursor.0, 64);

        send_click(&mut first, 0, 2, 9, 0, 0);
        settle(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        let player = stack.1.players.get(1).unwrap();
        assert!(player.cursor.is_empty());
        let (state, slots, _) = read_content(&mut first);
        assert_eq!(state, 3);
        assert_eq!(slots[9].0, 64);

        stack.1.pump_inventory(&mut stack.0);
        expect_silence(&mut first);
    }

    #[test]
    fn click_right_splits_and_places_single() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        read_content(&mut first);

        send_click(&mut first, 0, 1, 36, 1, 0);
        settle(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!(player.cursor.count, 32);
        assert_eq!(player.inventory.get(36).unwrap().count, 32);
        read_content(&mut first);

        send_click(&mut first, 0, 2, 9, 1, 0);
        settle(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!(player.inventory.get(9).unwrap().count, 1);
        assert_eq!(player.cursor.count, 31);
    }

    #[test]
    fn click_shift_and_swap_move_stacks() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        stack.1.players.get_mut(1).unwrap().inventory.set(
            9,
            crate::inventory::ItemStack::new(crate::inventory::Item::Dirt, 10).unwrap(),
        );
        stack.1.pump_inventory(&mut stack.0);
        read_content(&mut first);

        send_click(&mut first, 0, 1, 9, 0, 1);
        settle(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!(player.inventory.get(9).unwrap().count, 0);
        assert_eq!(player.inventory.get(39).unwrap().count, 10);
        read_content(&mut first);

        send_click(&mut first, 0, 2, 9, 0, 2);
        settle(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!(player.inventory.get(9).unwrap().count, 64);
        assert!(player.inventory.get(36).unwrap().is_empty());
    }

    #[test]
    fn throw_drag_and_clone_clicks_are_ignored() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        read_content(&mut first);

        send_click(&mut first, 0, 1, 36, 0, 4);
        send_click(&mut first, 0, 1, 36, 0, 5);
        send_click(&mut first, 0, 1, 36, 0, 6);
        send_click(&mut first, 0, 1, 36, 0, 3);
        settle(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        expect_silence(&mut first);
        let player = stack.1.players.get(1).unwrap();
        assert_eq!(player.inventory.get(36).unwrap().count, 64);
        assert!(player.cursor.is_empty());
    }

    #[test]
    fn stale_state_click_resyncs_without_applying() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        read_content(&mut first);

        send_click(&mut first, 0, 0, 36, 0, 0);
        settle(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        let (state, slots, _) = read_content(&mut first);
        assert_eq!(state, 2);
        assert_eq!(slots[36].0, 64);
        assert!(stack.1.players.get(1).unwrap().cursor.is_empty());
    }

    #[test]
    fn wrong_window_click_is_ignored() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        read_content(&mut first);

        send_click(&mut first, 3, 1, 36, 0, 0);
        settle(&mut stack);
        stack.1.pump_inventory(&mut stack.0);
        expect_silence(&mut first);
        assert!(stack.1.players.get(1).unwrap().cursor.is_empty());
    }

    #[test]
    fn malformed_click_is_tolerated() {
        let mut stack = test_stack();
        let (mut first, _) = join_pair(&mut stack);
        first.send_packet(0x12, &[0x00]);
        let mut body = Writer::new();
        body.write_varint(0);
        body.write_varint(0);
        body.write_i16(36);
        body.write_u8(0);
        body.write_varint(9);
        body.write_varint(0);
        write_hashed(&mut body, false, 0, 0);
        first.send_packet(0x12, &body.into_bytes());
        settle(&mut stack);
        assert_eq!(stack.0.connection_count(), 1);
        assert_eq!(stack.1.player_count(), 1);
    }

    #[test]
    fn malformed_dig_and_place_are_tolerated() {
        let mut stack = test_stack();
        let (mut first, mut second) = join_pair(&mut stack);
        seed_visible(&mut stack, 1);
        seed_visible(&mut stack, 2);

        first.send_packet(0x29, &[0x02, 0x00]);
        let mut body = Writer::new();
        body.write_varint(0);
        body.write_position(0, 64, 0);
        body.write_varint(9);
        body.write_f32(0.5);
        body.write_f32(0.5);
        body.write_f32(0.5);
        body.write_bool(false);
        body.write_bool(false);
        body.write_varint(5);
        second.send_packet(0x42, &body.into_bytes());
        settle(&mut stack);
        assert_eq!(stack.0.connection_count(), 2);
        assert_eq!(stack.1.player_count(), 2);
    }

    #[test]
    fn configuration_packets_are_tolerated() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_handshake(776, 2);
        client.send_login_start("TestPlayer");
        settle(&mut stack);
        let _ = client.read_packet();
        client.send_packet(0x03, &[]);
        settle(&mut stack);

        let (id, _) = client.read_packet();
        assert_eq!(id, 0x0E);
        client.send_packet(0x00, b"client-information");
        client.send_packet(0x02, b"minecraft:brand");
        settle(&mut stack);

        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(200)))
            .unwrap();
        let mut buf = [0u8; 1];
        let err = client.stream.read(&mut buf).unwrap_err();
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
        client
            .stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        assert_eq!(stack.1.player_count(), 1);
        assert!(stack
            .1
            .sessions
            .values()
            .any(|s| s.state == ProtocolState::Configuration));
    }

    #[test]
    fn registries_wait_for_known_packs_response() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_handshake(776, 2);
        client.send_login_start("PatientPlayer");
        settle(&mut stack);
        let _ = client.read_packet();
        client.send_packet(0x03, &[]);
        settle(&mut stack);

        let (id, _) = client.read_packet();
        assert_eq!(id, 0x0E);
        client
            .stream
            .set_read_timeout(Some(Duration::from_millis(300)))
            .unwrap();
        let mut buf = [0u8; 1];
        let err = client.stream.read(&mut buf).unwrap_err();
        assert!(matches!(
            err.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ));
        client
            .stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
    }

    #[test]
    fn disconnect_during_configuration_cleans_up() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_handshake(776, 2);
        client.send_login_start("LeavingPlayer");
        settle(&mut stack);
        let _ = client.read_packet();
        client.send_packet(0x03, &[]);
        settle(&mut stack);
        assert_eq!(stack.1.player_count(), 1);

        drop(client);
        wait_until(&mut stack, |n| n.connection_count() == 0);
        assert_eq!(stack.1.player_count(), 0);
        assert_eq!(stack.1.session_count(), 0);
    }

    #[test]
    fn unknown_configuration_packet_disconnects() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_handshake(776, 2);
        client.send_login_start("LostPlayer");
        settle(&mut stack);
        let _ = client.read_packet();
        client.send_packet(0x03, &[]);
        settle(&mut stack);
        let (id, _) = client.read_packet();
        assert_eq!(id, 0x0E);

        client.send_packet(0x7F, b"nonsense");
        settle(&mut stack);
        client.expect_eof();
        wait_until(&mut stack, |n| n.connection_count() == 0);
        assert_eq!(stack.1.player_count(), 0);
    }

    fn login_success_name(body: &[u8]) -> String {
        let mut reader = Reader::new(body);
        reader.read_uuid().unwrap(); // player uuid
        reader.read_string().unwrap() // player name
                                      // varint(0) properties + session uuid follow but we don't need them
    }

    #[test]
    fn duplicate_login_replaces_old_session() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut first = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);
        first.send_handshake(776, 2);
        first.send_login_start("Steve");
        settle(&mut stack);
        let (id, first_body) = first.read_packet();
        assert_eq!(id, 0x02);
        assert_eq!(login_success_name(&first_body), "Steve");
        assert_eq!(stack.1.player_count(), 1);

        let mut second = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 2);
        second.send_handshake(776, 2);
        second.send_login_start("steve");
        settle(&mut stack);
        let (id, second_body) = second.read_packet();
        assert_eq!(id, 0x02);
        // Both packets carry the same name (case-insensitive duplicate) but are
        // distinct login success responses issued to different connections.
        assert_eq!(login_success_name(&second_body), "steve");

        let (id, payload) = first.read_packet();
        assert_eq!(id, 0x00);
        let reason = Reader::new(&payload).read_string().unwrap();
        assert!(reason.contains(DUPLICATE_LOGIN_REASON));
        first.expect_eof();

        wait_until(&mut stack, |n| n.connection_count() == 1);
        assert_eq!(stack.1.player_count(), 1);
        assert_eq!(
            stack.1.players.find_by_name("STEVE").unwrap().connection_id,
            2
        );
    }

    #[test]
    fn malformed_login_start_disconnects() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_handshake(776, 2);
        let mut bad = Writer::new();
        bad.write_varint(10);
        bad.write_bytes(b"abc");
        client.send_packet(0x00, &bad.into_bytes());
        settle(&mut stack);
        client.expect_eof();
        wait_until(&mut stack, |n| n.connection_count() == 0);
        assert_eq!(stack.1.player_count(), 0);
    }

    #[test]
    fn login_packet_before_handshake_disconnects() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_login_start("Steve");
        settle(&mut stack);
        client.expect_eof();
        wait_until(&mut stack, |n| n.connection_count() == 0);
    }

    #[test]
    fn handshake_reaches_requested_state() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut status_client = ScriptClient::connect(addr);
        let mut login_client = ScriptClient::connect(addr);
        let mut transfer_client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 3);

        status_client.send_handshake(776, 1);
        login_client.send_handshake(777, 2);
        transfer_client.send_handshake(775, 3);
        settle(&mut stack);

        let states: Vec<ProtocolState> = stack.1.sessions.values().map(|s| s.state).collect();
        assert!(states.contains(&ProtocolState::Status));
        assert_eq!(
            states
                .iter()
                .filter(|s| **s == ProtocolState::Login)
                .count(),
            2
        );
        let versions: Vec<i32> = stack
            .1
            .sessions
            .values()
            .map(|s| s.protocol_version)
            .collect();
        assert!(versions.contains(&776));
        assert!(versions.contains(&777));
        assert!(versions.contains(&775));
    }

    #[test]
    fn invalid_next_state_disconnects() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_handshake(776, 99);
        settle(&mut stack);
        client.expect_eof();
        wait_until(&mut stack, |n| n.connection_count() == 0);
    }

    #[test]
    fn unexpected_packet_disconnects() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        client.send_packet(0x7F, b"nonsense");
        settle(&mut stack);
        client.expect_eof();
        wait_until(&mut stack, |n| n.connection_count() == 0);
    }

    #[test]
    fn split_and_coalesced_packets_both_work() {
        let mut stack = test_stack();
        let addr = stack.2;
        let mut client = ScriptClient::connect(addr);
        wait_until(&mut stack, |n| n.connection_count() == 1);

        let mut handshake = Writer::new();
        handshake.write_varint(776);
        handshake.write_string("localhost");
        handshake.write_u16(25565);
        handshake.write_varint(1);
        let handshake = handshake.into_bytes();
        let mut framed = Writer::new();
        framed.write_varint(handshake.len() as i32 + 1);
        framed.write_varint(0x00);
        framed.write_bytes(&handshake);
        let framed = framed.into_bytes();

        client.stream.write_all(&framed[..3]).unwrap();
        thread::sleep(Duration::from_millis(50));
        pump(&mut stack);
        assert_eq!(stack.1.session_count(), 1);
        client.stream.write_all(&framed[3..]).unwrap();

        let mut both = Writer::new();
        both.write_varint(1);
        both.write_varint(0x00);
        let mut ping = Writer::new();
        ping.write_varint(9);
        ping.write_varint(0x01);
        ping.write_i64(42);
        both.write_bytes(&ping.into_bytes());
        client.stream.write_all(&both.into_bytes()).unwrap();
        settle(&mut stack);

        let (id, _) = client.read_packet();
        assert_eq!(id, 0x00);
        let (id, payload) = client.read_packet();
        assert_eq!(id, 0x01);
        assert_eq!(Reader::new(&payload).read_i64().unwrap(), 42);
    }

    #[test]
    fn frames_handle_partial_and_invalid_lengths() {
        let mut staging = vec![0x05];
        assert!(take_frame(&mut staging).unwrap().is_none());

        let mut staging = vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
        assert_eq!(take_frame(&mut staging), Err(ProtoError::PacketTooLong(-1)));

        let mut staging = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        assert_eq!(take_frame(&mut staging), Err(ProtoError::VarIntTooLong));
    }
}
