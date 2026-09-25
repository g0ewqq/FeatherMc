use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::inventory::{Item, ItemStack};

/// File layout for one saved chunk (little-endian throughout):
/// `FCHK` magic, version byte (1), section count (u16), then per section a
/// run-length encoding: run count (u16) followed by (block id u16, run
/// length u16) pairs whose lengths must sum to 4096.
pub const CHUNK_FILE_MAGIC: &[u8; 4] = b"FCHK";
pub const CHUNK_FILE_VERSION: u8 = 1;
pub const SECTION_BLOCKS: usize = 4096;

pub fn encode_sections(sections: &[&[u16; SECTION_BLOCKS]]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(CHUNK_FILE_MAGIC);
    out.push(CHUNK_FILE_VERSION);
    out.extend_from_slice(&(sections.len() as u16).to_le_bytes());
    for states in sections {
        let mut runs: Vec<(u16, usize)> = Vec::new();
        for &value in states.iter() {
            match runs.last_mut() {
                Some(last) if last.0 == value => last.1 += 1,
                _ => runs.push((value, 1)),
            }
        }
        out.extend_from_slice(&(runs.len() as u16).to_le_bytes());
        for (value, len) in runs {
            out.extend_from_slice(&value.to_le_bytes());
            out.extend_from_slice(&(len as u16).to_le_bytes());
        }
    }
    out
}

/// Inverse of [`encode_sections`]. Truncated, over-long or otherwise corrupt
/// input returns `None` — callers regenerate the chunk rather than trust it.
pub fn decode_sections(bytes: &[u8], section_count: usize) -> Option<Vec<[u16; SECTION_BLOCKS]>> {
    if bytes.get(..4)? != *CHUNK_FILE_MAGIC {
        return None;
    }
    if *bytes.get(4)? != CHUNK_FILE_VERSION {
        return None;
    }
    let mut pos = 5;
    let read_u16 = |pos: &mut usize| {
        let chunk = bytes.get(*pos..*pos + 2)?;
        *pos += 2;
        Some(u16::from_le_bytes([chunk[0], chunk[1]]))
    };
    if read_u16(&mut pos)? as usize != section_count {
        return None;
    }
    let mut sections = Vec::with_capacity(section_count);
    for _ in 0..section_count {
        let run_count = read_u16(&mut pos)? as usize;
        let mut states = [0u16; SECTION_BLOCKS];
        let mut at = 0usize;
        for _ in 0..run_count {
            let value = read_u16(&mut pos)?;
            let len = read_u16(&mut pos)? as usize;
            if len == 0 || at + len > SECTION_BLOCKS {
                return None;
            }
            states[at..at + len].fill(value);
            at += len;
        }
        if at != SECTION_BLOCKS {
            return None;
        }
        sections.push(states);
    }
    if pos != bytes.len() {
        return None;
    }
    Some(sections)
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "player".to_owned()
    } else {
        cleaned
    }
}

/// On-disk chunk files for one world: `<dir>/c.<x>.<z>.bin`.
#[derive(Debug, Clone)]
pub struct WorldStore {
    dir: PathBuf,
}

impl WorldStore {
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    #[must_use]
    pub fn chunk_file(&self, x: i32, z: i32) -> PathBuf {
        self.dir.join(format!("c.{x}.{z}.bin"))
    }

    pub fn save_chunk(&self, x: i32, z: i32, bytes: &[u8]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        std::fs::write(self.chunk_file(x, z), bytes)?;
        Ok(())
    }

    #[must_use]
    pub fn load_chunk(&self, x: i32, z: i32) -> Option<Vec<u8>> {
        std::fs::read(self.chunk_file(x, z)).ok()
    }
}

/// Serializable player snapshot. Stacks are (item id, count) pairs; unknown ids
/// and out-of-range counts get skipped on load. The uuid and entity/session ids
/// are deliberately not stored — login supplies the uuid, and every session
/// mints fresh ids.
fn full_health() -> f32 {
    20.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    pub name: String,
    pub world: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub gamemode: String,
    pub flying: bool,
    #[serde(default = "full_health")]
    pub health: f32,
    pub selected: u8,
    pub inventory: Vec<(i32, u32)>,
    pub cursor: (i32, u32),
}

impl PlayerData {
    #[must_use]
    pub fn stack_of(id: i32, count: u32) -> ItemStack {
        if count == 0 {
            return ItemStack::empty();
        }
        match Item::from_id(id) {
            Some(item) => ItemStack::new(item, count).unwrap_or_else(ItemStack::empty),
            None => ItemStack::empty(),
        }
    }
}

/// On-disk player snapshots: `<dir>/<sanitized-name>.toml`.
#[derive(Debug, Clone)]
pub struct PlayerStore {
    dir: PathBuf,
}

impl PlayerStore {
    #[must_use]
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    #[must_use]
    pub fn player_file(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{}.toml", sanitize_filename(name)))
    }

    pub fn save_player(&self, data: &PlayerData) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        let text = toml::to_string_pretty(data).map_err(std::io::Error::other)?;
        std::fs::write(self.player_file(&data.name), text)?;
        Ok(())
    }

    #[must_use]
    pub fn load_player(&self, name: &str) -> Option<PlayerData> {
        let text = std::fs::read_to_string(self.player_file(name)).ok()?;
        match toml::from_str(&text) {
            Ok(data) => Some(data),
            Err(e) => {
                debug!("ignoring corrupt player file for {name:?}: {e}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_files_roundtrip_rle() {
        let uniform = [7u16; SECTION_BLOCKS];
        let mut mixed = [0u16; SECTION_BLOCKS];
        for (i, slot) in mixed.iter_mut().enumerate() {
            *slot = (i % 5) as u16;
        }
        let sections = vec![&uniform, &mixed];
        let bytes = encode_sections(&sections);
        // A single uniform section must compress to almost nothing (one run).
        let uniform_only = encode_sections(&[&uniform]);
        assert!(uniform_only.len() < 200);
        let back = decode_sections(&bytes, 2).unwrap();
        assert_eq!(back[0], uniform);
        assert_eq!(back[1], mixed);
    }

    #[test]
    fn chunk_files_reject_corrupt_input() {
        assert!(decode_sections(&[], 1).is_none());
        assert!(decode_sections(b"NOPE", 1).is_none());
        let good = encode_sections(&[&[3u16; SECTION_BLOCKS]]);
        assert!(decode_sections(&good, 2).is_none()); // wrong section count
        assert!(decode_sections(&good[..good.len() - 1], 1).is_none()); // truncated
        let mut extra = good.clone();
        extra.push(0);
        assert!(decode_sections(&extra, 1).is_none()); // trailing garbage
        let mut bad_version = good.clone();
        bad_version[4] = 99;
        assert!(decode_sections(&bad_version, 1).is_none());
    }

    #[test]
    fn stack_of_skips_unknown_or_invalid() {
        assert!(PlayerData::stack_of(0, 0).is_empty());
        assert!(PlayerData::stack_of(1, 0).is_empty());
        assert_eq!(
            PlayerData::stack_of(99, 5).item,
            crate::inventory::Item::ExposedChiseledCopper
        );
        assert!(PlayerData::stack_of(99999, 5).is_empty());
        assert!(PlayerData::stack_of(1, 65).is_empty());
        assert_eq!(PlayerData::stack_of(1, 5).count, 5);
    }

    #[test]
    fn player_files_roundtrip_through_toml() {
        let dir = tempfile::tempdir().unwrap();
        let store = PlayerStore::new(dir.path().to_owned());
        let data = PlayerData {
            name: "Steve!".to_owned(),
            world: "world".to_owned(),
            x: 1.5,
            y: 65.0,
            z: -3.0,
            yaw: 90.0,
            pitch: 0.0,
            gamemode: "creative".to_owned(),
            flying: true,
            health: 7.5,
            selected: 2,
            inventory: vec![(1, 64), (0, 0)],
            cursor: (28, 3),
        };
        store.save_player(&data).unwrap();
        // Unsafe characters are sanitized out of the file name.
        assert!(store.player_file("Steve!").ends_with("Steve_.toml"));
        let back = store.load_player("Steve!").unwrap();
        assert_eq!(back.x, 1.5);
        assert_eq!(back.health, 7.5);
        assert_eq!(back.gamemode, "creative");
        assert_eq!(back.inventory[0], (1, 64));
        assert_eq!(back.cursor, (28, 3));
        assert!(store.load_player("Nobody").is_none());
    }
}
