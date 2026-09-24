<p align="center">
	<b><font size="+3">FeatherMC</font></b><br>
	<b>A from-scratch Minecraft: Java Edition server written in Rust</b>
</p>

<p align="center">
	<img alt="Java 26.2" src="https://img.shields.io/badge/java-26.2-green">
	<img alt="Protocol 776" src="https://img.shields.io/badge/protocol-776-blue">
	<img alt="Rust" src="https://img.shields.io/badge/made_with-Rust-orange">
</p>

## What is this?

FeatherMC is a Minecraft: Java Edition server built from scratch in Rust,
with no vanilla server code. Every packet on the wire is implemented by
hand against the real protocol and verified with real Java clients.

If you want to tinker with server internals, learn the Minecraft protocol,
or grow a small multiplayer sandbox, this is for you.

- 🧱 **Real gameplay foundation** - join, spawn, walk, fall, break and place
  blocks, see other players move, sneak and look around
- 🗺️ **Server-side world** - deterministic flat overworld with chunk storage,
  serialization and streaming straight from memory
- ⚡ **20 TPS tick loop** - network, physics, chunk streaming and entity
  tracking run on a fixed-rate schedule
- 🔍 **Protocol-first development** - every packet ID and layout is verified
  against authoritative sources, never guessed

## FeatherMC is NOT a vanilla Minecraft server.

It is poorly suited to hosting a survival server. It is missing most of the
vanilla game, such as world generation, mobs and AI, redstone, fluids,
inventory, combat, persistence and plugins.

If you just want to play **vanilla survival multiplayer**, use the
[official Minecraft: Java server software](https://www.minecraft.net/en-us/download/server)
(or Paper/Purpur) instead of FeatherMC.

## Supported Versions

| Edition | Supported Versions |
|---------|-------------------|
| Java    | 26.2 (protocol 776, offline mode) |
| Bedrock | Not supported |

## Getting Started

### Requirements

- Stable Rust (see `rust-version` in `Cargo.toml`)
- Minecraft: Java Edition 26.2 for playtesting

### Build

```bash
cargo build
```

### Run

```bash
cargo run
```

On first start the server creates `config/server.toml` with defaults and
the runtime directories if they are missing. Existing files are never
overwritten. Connect with a 26.2 client to `localhost:25565`. Type `stop`
in the console (or press Ctrl-C) to shut down.

Set `RUST_LOG=debug` for verbose logs.

### Checks

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy
cargo build
```

(`cargo build` fails while a previously built server binary is still
running because Windows locks the executable; stop it first.)

## Configuration

`config/server.toml`:

```toml
[server]
name = "FeatherMC"
motd = "A FeatherMC Server"
max_players = 20

[network]
address = "0.0.0.0"
java_port = 25565
```

## Features

- Handshake, status/ping, login, configuration (known packs, registries,
  tags) and play entry with teleport confirmation and keep-alive
- Player sessions with duplicate-login handling and clean disconnects
- Server-authoritative movement: validated input, gravity, ground collision,
  teleport corrections
- Chunk streaming around spawn with follow-up streaming and unloads
- Player visibility: profiles, spawn, metadata, movement/rotation sync,
  sneak/sprint states, despawn on disconnect or range
- Block breaking and placing with reach checks, dirty-section tracking
  and updates broadcast to nearby viewers
- Item and inventory foundation: validated stacks, 46-slot inventory with
  hotbar selection, starting items, click/drag/shift handling with
  state-ID validation, container sync, held-item-driven placement
- Creative mode: console `gamemode` switching and in-game
  `/gamemode <mode> [player]`, abilities sync with client-toggled
  flight (gravity-exempt), instant digging, placement without
  consuming items, and creative slot writes with slot -1 drops
- Full NBT reader/writer, TOML config, structured logging

## Known limitations

Single flat world, no disk persistence, placing uses the held hotbar
item and only Stone, Dirt and Grass Block exist server-side (other
items fall back to the held survival item and the client is corrected,
which looks like morphing), creative inventory is client-trusted (any
stack size up to the max and any known item can be written to any
slot), full-bright lighting, default skins, no mobs, combat, redstone,
fluids or plugins.

## Project layout

```text
src/
  main.rs        binary entry point
  lib.rs         library root
  server.rs      tick loop and lifecycle
  world.rs       worlds, chunks, sections, blocks
  entity.rs      minimal entity foundation
  network/       listener, connections, framing
  java/          protocol: packets, registries, NBT, session,
                 chunk serializer, entities, metadata
  config.rs      server configuration
  ...
```

## Contributing

FeatherMC is an early-stage learning project. Bug reports with client
disconnect logs (`debug/disconnect-*.txt` plus the exact behavior) are the
most useful contribution right now.

## Licensing information

No license has been chosen yet. All rights reserved for now.

FeatherMC is not affiliated with Mojang or Microsoft. All brands and
trademarks belong to their respective owners. It is not Mojang-approved
software.
