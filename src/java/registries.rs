use super::error::ProtoError;
use super::nbt::NbtTag;
use super::packets::encode_packet;
use super::proto::Writer;

pub struct RegistryEntry {
    pub id: &'static str,
    /// Entry NBT. `None` omits it (`has_data = false`) so the client loads
    /// vanilla data from its own known packs (`minecraft:core`) — saves having
    /// to hand-craft exact NBT for every vanilla entry.
    pub data: Option<NbtTag>,
}

fn vanilla(id: &'static str) -> RegistryEntry {
    RegistryEntry { id, data: None }
}

fn with_nbt(id: &'static str, data: NbtTag) -> RegistryEntry {
    RegistryEntry {
        id,
        data: Some(data),
    }
}

pub struct ProtocolRegistry {
    pub id: &'static str,
    pub entries: Vec<RegistryEntry>,
}

impl ProtocolRegistry {
    #[must_use]
    pub fn entry_index(&self, id: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.id == id)
    }
}

/// Minimal overworld dimension NBT that dodges every tag lookup.
///
/// Vanilla's overworld points at `#minecraft:infiniburn_overworld`, the
/// `#minecraft:in_overworld` timelines and the `minecraft:overworld` clock —
/// all of which need tag/registry data this server never sends. 26.2 takes a
/// plain block id for `infiniburn`, and `timelines`/`default_clock` are
/// optional, so we leave them out entirely.
fn minimal_overworld_dimension() -> NbtTag {
    NbtTag::compound(vec![
        ("ambient_light".to_owned(), NbtTag::Float(0.0)),
        ("attributes".to_owned(), NbtTag::compound(vec![])),
        ("coordinate_scale".to_owned(), NbtTag::Double(1.0)),
        ("has_ceiling".to_owned(), NbtTag::Byte(0)),
        ("has_ender_dragon_fight".to_owned(), NbtTag::Byte(0)),
        ("has_fixed_time".to_owned(), NbtTag::Byte(0)),
        ("has_skylight".to_owned(), NbtTag::Byte(1)),
        ("height".to_owned(), NbtTag::Int(384)),
        (
            "infiniburn".to_owned(),
            NbtTag::String("minecraft:netherrack".to_owned()),
        ),
        ("logical_height".to_owned(), NbtTag::Int(384)),
        ("min_y".to_owned(), NbtTag::Int(-64)),
        ("monster_spawn_block_light_limit".to_owned(), NbtTag::Int(0)),
        (
            "monster_spawn_light_level".to_owned(),
            NbtTag::compound(vec![
                (
                    "type".to_owned(),
                    NbtTag::String("minecraft:uniform".to_owned()),
                ),
                ("max_inclusive".to_owned(), NbtTag::Int(7)),
                ("min_inclusive".to_owned(), NbtTag::Int(0)),
            ]),
        ),
    ])
}

#[allow(dead_code)]
fn overworld_dimension() -> NbtTag {
    NbtTag::compound(vec![
        ("ambient_light".to_owned(), NbtTag::Float(0.0)),
        (
            "attributes".to_owned(),
            NbtTag::compound(vec![
                (
                    "minecraft:audio/ambient_sounds".to_owned(),
                    NbtTag::compound(vec![(
                        "mood".to_owned(),
                        NbtTag::compound(vec![
                            ("block_search_extent".to_owned(), NbtTag::Int(8)),
                            ("offset".to_owned(), NbtTag::Double(2.0)),
                            (
                                "sound".to_owned(),
                                NbtTag::String("minecraft:ambient.cave".to_owned()),
                            ),
                            ("tick_delay".to_owned(), NbtTag::Int(6000)),
                        ]),
                    )]),
                ),
                (
                    "minecraft:audio/background_music".to_owned(),
                    NbtTag::compound(vec![
                        (
                            "creative".to_owned(),
                            NbtTag::compound(vec![
                                ("max_delay".to_owned(), NbtTag::Int(24000)),
                                ("min_delay".to_owned(), NbtTag::Int(12000)),
                                (
                                    "sound".to_owned(),
                                    NbtTag::String("minecraft:music.creative".to_owned()),
                                ),
                            ]),
                        ),
                        (
                            "default".to_owned(),
                            NbtTag::compound(vec![
                                ("max_delay".to_owned(), NbtTag::Int(24000)),
                                ("min_delay".to_owned(), NbtTag::Int(12000)),
                                (
                                    "sound".to_owned(),
                                    NbtTag::String("minecraft:music.game".to_owned()),
                                ),
                            ]),
                        ),
                    ]),
                ),
                (
                    "minecraft:gameplay/bed_rule".to_owned(),
                    NbtTag::compound(vec![
                        (
                            "can_set_spawn".to_owned(),
                            NbtTag::String("always".to_owned()),
                        ),
                        (
                            "can_sleep".to_owned(),
                            NbtTag::String("when_dark".to_owned()),
                        ),
                        (
                            "error_message".to_owned(),
                            NbtTag::compound(vec![(
                                "translate".to_owned(),
                                NbtTag::String("block.minecraft.bed.no_sleep".to_owned()),
                            )]),
                        ),
                    ]),
                ),
                (
                    "minecraft:gameplay/nether_portal_spawns_piglin".to_owned(),
                    NbtTag::Byte(1),
                ),
                (
                    "minecraft:gameplay/respawn_anchor_works".to_owned(),
                    NbtTag::Byte(0),
                ),
                (
                    "minecraft:visual/ambient_light_color".to_owned(),
                    NbtTag::String("#0a0a0a".to_owned()),
                ),
                (
                    "minecraft:visual/cloud_color".to_owned(),
                    NbtTag::String("#ccffffff".to_owned()),
                ),
                (
                    "minecraft:visual/cloud_height".to_owned(),
                    NbtTag::Float(192.33),
                ),
                (
                    "minecraft:visual/fog_color".to_owned(),
                    NbtTag::String("#c0d8ff".to_owned()),
                ),
                (
                    "minecraft:visual/sky_color".to_owned(),
                    NbtTag::String("#78a7ff".to_owned()),
                ),
            ]),
        ),
        ("coordinate_scale".to_owned(), NbtTag::Double(1.0)),
        (
            "default_clock".to_owned(),
            NbtTag::String("minecraft:overworld".to_owned()),
        ),
        ("has_ceiling".to_owned(), NbtTag::Byte(0)),
        ("has_ender_dragon_fight".to_owned(), NbtTag::Byte(0)),
        ("has_skylight".to_owned(), NbtTag::Byte(1)),
        ("height".to_owned(), NbtTag::Int(384)),
        (
            "infiniburn".to_owned(),
            NbtTag::String("#minecraft:infiniburn_overworld".to_owned()),
        ),
        ("logical_height".to_owned(), NbtTag::Int(384)),
        ("min_y".to_owned(), NbtTag::Int(-64)),
        ("monster_spawn_block_light_limit".to_owned(), NbtTag::Int(0)),
        (
            "monster_spawn_light_level".to_owned(),
            NbtTag::compound(vec![
                (
                    "type".to_owned(),
                    NbtTag::String("minecraft:uniform".to_owned()),
                ),
                ("max_inclusive".to_owned(), NbtTag::Int(7)),
                ("min_inclusive".to_owned(), NbtTag::Int(0)),
            ]),
        ),
        (
            "timelines".to_owned(),
            NbtTag::String("#minecraft:in_overworld".to_owned()),
        ),
    ])
}

fn plains_biome() -> NbtTag {
    NbtTag::compound(vec![
        ("temperature".to_owned(), NbtTag::Float(0.8)),
        ("downfall".to_owned(), NbtTag::Float(0.4)),
        ("has_precipitation".to_owned(), NbtTag::Byte(1)),
        (
            "temperature_modifier".to_owned(),
            NbtTag::String("none".to_owned()),
        ),
        (
            "effects".to_owned(),
            NbtTag::compound(vec![
                ("fog_color".to_owned(), NbtTag::Int(12638463)),
                ("water_color".to_owned(), NbtTag::Int(4159204)),
                ("water_fog_color".to_owned(), NbtTag::Int(329011)),
                ("sky_color".to_owned(), NbtTag::Int(7907327)),
            ]),
        ),
    ])
}

fn chat_style(translation_key: &str) -> NbtTag {
    NbtTag::compound(vec![
        (
            "parameters".to_owned(),
            NbtTag::List(vec![
                NbtTag::String("sender".to_owned()),
                NbtTag::String("content".to_owned()),
            ]),
        ),
        (
            "translation_key".to_owned(),
            NbtTag::String(translation_key.to_owned()),
        ),
    ])
}

fn damage_entry(
    message_id: &str,
    exhaustion: f32,
    effects: Option<&str>,
    death_message_type: Option<&str>,
) -> NbtTag {
    let mut fields = vec![
        (
            "message_id".to_owned(),
            NbtTag::String(message_id.to_owned()),
        ),
        (
            "scaling".to_owned(),
            NbtTag::String("when_caused_by_living_non_player".to_owned()),
        ),
        ("exhaustion".to_owned(), NbtTag::Float(exhaustion)),
    ];
    if let Some(effects) = effects {
        fields.push(("effects".to_owned(), NbtTag::String(effects.to_owned())));
    }
    if let Some(death_message_type) = death_message_type {
        fields.push((
            "death_message_type".to_owned(),
            NbtTag::String(death_message_type.to_owned()),
        ));
    }
    NbtTag::compound(fields)
}

#[must_use]
pub fn default_registries() -> Vec<ProtocolRegistry> {
    // We tell the client minecraft:core is known (and it confirms the same),
    // so we can omit all NBT — the client loads it from its own data pack.
    // We only need to list the entry IDs to establish the numeric ID mapping.
    // This is the 1.21.10+ "minimum without NBT" approach: required damage
    // types, plains biome, one entry per non-empty registry. No tags needed.
    vec![
        ProtocolRegistry {
            id: "minecraft:banner_pattern",
            entries: vec![vanilla("minecraft:base")],
        },
        ProtocolRegistry {
            id: "minecraft:chat_type",
            entries: vec![vanilla("minecraft:chat"), vanilla("minecraft:say_command")],
        },
        ProtocolRegistry {
            id: "minecraft:damage_type",
            entries: vec![
                vanilla("minecraft:cactus"),
                vanilla("minecraft:campfire"),
                vanilla("minecraft:cramming"),
                vanilla("minecraft:dragon_breath"),
                vanilla("minecraft:drown"),
                vanilla("minecraft:dry_out"),
                vanilla("minecraft:ender_pearl"),
                vanilla("minecraft:fall"),
                vanilla("minecraft:fly_into_wall"),
                vanilla("minecraft:freeze"),
                vanilla("minecraft:generic"),
                vanilla("minecraft:generic_kill"),
                vanilla("minecraft:hot_floor"),
                vanilla("minecraft:in_fire"),
                vanilla("minecraft:in_wall"),
                vanilla("minecraft:lava"),
                vanilla("minecraft:lightning_bolt"),
                vanilla("minecraft:magic"),
                vanilla("minecraft:on_fire"),
                vanilla("minecraft:out_of_world"),
                vanilla("minecraft:outside_border"),
                vanilla("minecraft:spear"),
                vanilla("minecraft:stalagmite"),
                vanilla("minecraft:starve"),
                vanilla("minecraft:sulfur_cube_hot"),
                vanilla("minecraft:sweet_berry_bush"),
                vanilla("minecraft:wither"),
            ],
        },
        ProtocolRegistry {
            id: "minecraft:dialog",
            entries: vec![],
        },
        ProtocolRegistry {
            id: "minecraft:dimension_type",
            entries: vec![with_nbt(
                "minecraft:overworld",
                minimal_overworld_dimension(),
            )],
        },
        ProtocolRegistry {
            id: "minecraft:enchantment",
            entries: vec![],
        },
        ProtocolRegistry {
            id: "minecraft:instrument",
            entries: vec![vanilla("minecraft:ponder_goat_horn")],
        },
        ProtocolRegistry {
            id: "minecraft:jukebox_song",
            entries: vec![
                vanilla("minecraft:11"),
                vanilla("minecraft:13"),
                vanilla("minecraft:5"),
                vanilla("minecraft:blocks"),
                vanilla("minecraft:bounce"),
                vanilla("minecraft:cat"),
                vanilla("minecraft:chirp"),
                vanilla("minecraft:creator"),
                vanilla("minecraft:creator_music_box"),
                vanilla("minecraft:far"),
                vanilla("minecraft:lava_chicken"),
                vanilla("minecraft:mall"),
                vanilla("minecraft:mellohi"),
                vanilla("minecraft:otherside"),
                vanilla("minecraft:pigstep"),
                vanilla("minecraft:precipice"),
                vanilla("minecraft:relic"),
                vanilla("minecraft:stal"),
                vanilla("minecraft:strad"),
                vanilla("minecraft:tears"),
                vanilla("minecraft:wait"),
                vanilla("minecraft:ward"),
            ],
        },
        ProtocolRegistry {
            id: "minecraft:painting_variant",
            entries: vec![vanilla("minecraft:kebab")],
        },
        ProtocolRegistry {
            id: "minecraft:sulfur_cube_archetype",
            entries: vec![
                vanilla("minecraft:regular"),
                vanilla("minecraft:bouncy"),
                vanilla("minecraft:slow_bouncy"),
                vanilla("minecraft:slow_flat"),
                vanilla("minecraft:fast_flat"),
                vanilla("minecraft:light"),
                vanilla("minecraft:fast_sliding"),
                vanilla("minecraft:slow_sliding"),
                vanilla("minecraft:high_resistance"),
                vanilla("minecraft:sticky"),
                vanilla("minecraft:explosive"),
                vanilla("minecraft:hot"),
            ],
        },
        ProtocolRegistry {
            id: "minecraft:test_environment",
            entries: vec![],
        },
        ProtocolRegistry {
            id: "minecraft:test_instance",
            entries: vec![],
        },
        ProtocolRegistry {
            id: "minecraft:timeline",
            entries: vec![],
        },
        ProtocolRegistry {
            id: "minecraft:trim_material",
            entries: vec![
                vanilla("minecraft:quartz"),
                vanilla("minecraft:iron"),
                vanilla("minecraft:netherite"),
                vanilla("minecraft:redstone"),
                vanilla("minecraft:copper"),
                vanilla("minecraft:gold"),
                vanilla("minecraft:emerald"),
                vanilla("minecraft:diamond"),
                vanilla("minecraft:lapis"),
                vanilla("minecraft:amethyst"),
                vanilla("minecraft:resin"),
            ],
        },
        ProtocolRegistry {
            id: "minecraft:trim_pattern",
            entries: vec![vanilla("minecraft:sentry")],
        },
        ProtocolRegistry {
            id: "minecraft:world_clock",
            entries: vec![],
        },
        ProtocolRegistry {
            id: "minecraft:worldgen/biome",
            entries: vec![vanilla("minecraft:plains")],
        },
        ProtocolRegistry {
            id: "minecraft:cat_variant",
            entries: vec![vanilla("minecraft:tabby")],
        },
        ProtocolRegistry {
            id: "minecraft:cat_sound_variant",
            entries: vec![vanilla("minecraft:classic"), vanilla("minecraft:royal")],
        },
        ProtocolRegistry {
            id: "minecraft:chicken_variant",
            entries: vec![
                vanilla("minecraft:temperate"),
                vanilla("minecraft:cold"),
                vanilla("minecraft:warm"),
            ],
        },
        ProtocolRegistry {
            id: "minecraft:chicken_sound_variant",
            entries: vec![vanilla("minecraft:classic"), vanilla("minecraft:picky")],
        },
        ProtocolRegistry {
            id: "minecraft:cow_variant",
            entries: vec![vanilla("minecraft:temperate")],
        },
        ProtocolRegistry {
            id: "minecraft:cow_sound_variant",
            entries: vec![vanilla("minecraft:classic"), vanilla("minecraft:moody")],
        },
        ProtocolRegistry {
            id: "minecraft:frog_variant",
            entries: vec![vanilla("minecraft:temperate")],
        },
        ProtocolRegistry {
            id: "minecraft:pig_variant",
            entries: vec![vanilla("minecraft:temperate")],
        },
        ProtocolRegistry {
            id: "minecraft:pig_sound_variant",
            entries: vec![
                vanilla("minecraft:classic"),
                vanilla("minecraft:big"),
                vanilla("minecraft:mini"),
            ],
        },
        ProtocolRegistry {
            id: "minecraft:wolf_variant",
            entries: vec![
                vanilla("minecraft:pale"),
                vanilla("minecraft:spotted"),
                vanilla("minecraft:snowy"),
                vanilla("minecraft:black"),
                vanilla("minecraft:ashen"),
                vanilla("minecraft:rusty"),
                vanilla("minecraft:woods"),
                vanilla("minecraft:chestnut"),
                vanilla("minecraft:striped"),
            ],
        },
        ProtocolRegistry {
            id: "minecraft:wolf_sound_variant",
            entries: vec![
                vanilla("minecraft:angry"),
                vanilla("minecraft:big"),
                vanilla("minecraft:classic"),
                vanilla("minecraft:cute"),
                vanilla("minecraft:grumpy"),
                vanilla("minecraft:puglin"),
                vanilla("minecraft:sad"),
            ],
        },
        ProtocolRegistry {
            id: "minecraft:zombie_nautilus_variant",
            entries: vec![vanilla("minecraft:temperate"), vanilla("minecraft:warm")],
        },
    ]
}

pub fn encode_registry_data(registry: &ProtocolRegistry) -> Result<Vec<u8>, ProtoError> {
    let mut body = Writer::new();
    body.write_string(registry.id);
    body.write_varint(registry.entries.len() as i32);
    for entry in &registry.entries {
        body.write_string(entry.id);
        match &entry.data {
            Some(nbt) => {
                body.write_bool(true);
                let mut nbt_buf = Writer::new();
                nbt.encode_unnamed(&mut nbt_buf)?;
                body.write_bytes(&nbt_buf.into_bytes());
            }
            None => {
                body.write_bool(false);
            }
        }
    }
    Ok(encode_packet(0x07, &body.into_bytes()))
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
    fn ships_required_registries() {
        let registries = default_registries();
        let ids: Vec<&str> = registries.iter().map(|r| r.id).collect();
        assert_eq!(
            ids,
            vec![
                "minecraft:banner_pattern",
                "minecraft:chat_type",
                "minecraft:damage_type",
                "minecraft:dialog",
                "minecraft:dimension_type",
                "minecraft:enchantment",
                "minecraft:instrument",
                "minecraft:jukebox_song",
                "minecraft:painting_variant",
                "minecraft:sulfur_cube_archetype",
                "minecraft:test_environment",
                "minecraft:test_instance",
                "minecraft:timeline",
                "minecraft:trim_material",
                "minecraft:trim_pattern",
                "minecraft:world_clock",
                "minecraft:worldgen/biome",
                "minecraft:cat_variant",
                "minecraft:cat_sound_variant",
                "minecraft:chicken_variant",
                "minecraft:chicken_sound_variant",
                "minecraft:cow_variant",
                "minecraft:cow_sound_variant",
                "minecraft:frog_variant",
                "minecraft:pig_variant",
                "minecraft:pig_sound_variant",
                "minecraft:wolf_variant",
                "minecraft:wolf_sound_variant",
                "minecraft:zombie_nautilus_variant",
            ]
        );
        let dim = registries
            .iter()
            .find(|r| r.id == "minecraft:dimension_type")
            .unwrap();
        assert_eq!(dim.entries[0].id, "minecraft:overworld");
        let biome = registries
            .iter()
            .find(|r| r.id == "minecraft:worldgen/biome")
            .unwrap();
        assert_eq!(biome.entries[0].id, "minecraft:plains");
        let damage = registries
            .iter()
            .find(|r| r.id == "minecraft:damage_type")
            .unwrap();
        assert!(damage.entries.len() >= 26);
        // Every non-optional registry must be non-empty.
        for r in &registries {
            match r.id {
                "minecraft:dialog"
                | "minecraft:enchantment"
                | "minecraft:test_environment"
                | "minecraft:test_instance"
                | "minecraft:timeline"
                | "minecraft:world_clock" => {}
                _ => assert!(!r.entries.is_empty(), "{} must be non-empty", r.id),
            }
        }
    }

    #[test]
    fn registry_packets_roundtrip() {
        for registry in default_registries() {
            let bytes = encode_registry_data(&registry).unwrap();
            let (id, payload) = decode_frame(&bytes);
            assert_eq!(id, 0x07);
            let mut reader = Reader::new(&payload);
            assert_eq!(reader.read_string().unwrap(), registry.id);
            let count = reader.read_varint().unwrap() as usize;
            assert_eq!(count, registry.entries.len());
            for entry in &registry.entries {
                assert_eq!(reader.read_string().unwrap(), entry.id);
                let has_data = reader.read_bool().unwrap();
                assert_eq!(has_data, entry.data.is_some());
                if has_data {
                    assert_eq!(
                        NbtTag::decode_unnamed(&mut reader).unwrap(),
                        *entry.data.as_ref().unwrap()
                    );
                }
            }
            assert_eq!(reader.remaining(), 0);
        }
    }

    #[test]
    fn overworld_dimension_has_expected_shape() {
        let registries = default_registries();
        let dimension = registries
            .iter()
            .find(|r| r.id == "minecraft:dimension_type")
            .unwrap();
        assert_eq!(dimension.entries[0].id, "minecraft:overworld");
        // Custom minimal NBT: must avoid every tag/registry lookup the
        // client log flagged (infiniburn tag, timelines tag, world clock).
        let NbtTag::Compound(fields) = dimension.entries[0]
            .data
            .as_ref()
            .expect("dimension must carry custom NBT")
        else {
            panic!("dimension entry must be a compound");
        };
        assert_eq!(fields.get("min_y"), Some(&NbtTag::Int(-64)));
        assert_eq!(fields.get("height"), Some(&NbtTag::Int(384)));
        assert_eq!(fields.get("has_skylight"), Some(&NbtTag::Byte(1)));
        assert_eq!(
            fields.get("infiniburn"),
            Some(&NbtTag::String("minecraft:netherrack".to_owned()))
        );
        assert!(fields.get("timelines").is_none());
        assert!(fields.get("default_clock").is_none());
    }
}
