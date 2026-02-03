# Bevy Demo Project

This repository contains practical examples of the Bevy game engine in Rust.

**Goal:** Provide hands-on, well-documented examples demonstrating Bevy engine concepts including ECS (Entity Component System), rendering, input handling, and game logic patterns.

## Tech Stack

- **Rust** (Edition 2024)
- **Bevy 0.18.0** - Data-driven game engine
- **rand** - Deterministic randomness (uses SmallRng)

## Project Structure

```
src/bin/          # Standalone example binaries
  firefly.rs      # 3D firefly simulation with blinking, movement, keyboard input
  followings.rs   # Smooth entity interpolation/following demo
assets/           # Game assets (textures, models, etc.)
```

## Commands

```bash
# Build
cargo build --release
task build              # Format + build release with features

# Run examples
cargo run --bin firefly
cargo run --bin followings
task run                # Runs firefly (default)

# Run with features
cargo run --bin firefly --features transparent
cargo run --bin firefly --features "transparent,window-offset"

# Check/lint
cargo check
cargo fmt
cargo clippy
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `window-offset` | Offset window position for local development (160, 88) |
| `transparent` | Enable semi-transparent window background to see desktop wallpaper |

Features are enabled in Taskfile by default:
```bash
task build  # Builds with window-offset,transparent
```

To build without features:
```bash
cargo build --release
```

## Development Principles

- **Learn by Example**: Each binary demonstrates specific Bevy concepts
- **Self-Contained**: Examples should run independently with minimal setup
- **Well-Commented**: Complex patterns should include inline explanations
- **Idiomatic Bevy**: Follow Bevy's ECS patterns (Components, Resources, Systems)

## Bevy Patterns Used

### Components
Define data attached to entities:
```rust
#[derive(Component)]
struct Firefly;

#[derive(Component)]
struct FireflySpeed(f32);
```

### Resources
Global state accessible by systems:
```rust
#[derive(Resource)]
struct RandomSource(ChaCha8Rng);
```

### Systems
Functions that operate on entities/resources:
```rust
fn move_firefly(
    mut query: Query<(&mut Transform, &FireflySpeed), With<Firefly>>,
    time: Res<Time>,
) { ... }
```

### System Ordering
Chain dependent systems:
```rust
.add_systems(Update, (move_target, move_follower).chain())
```

## Adding New Examples

1. Create a new file in `src/bin/`
2. Add the binary entry to `Cargo.toml`:
   ```toml
   [[bin]]
   name = "example_name"
   src = "src/bin/example_name.rs"
   ```
3. Include a module-level doc comment explaining the example's purpose
4. Run and verify with `cargo run --bin example_name`

## Commit Conventions

- `feat:` new example or feature
- `fix:` bug fix
- `refactor:` code improvement without behavior change
- `chore:` tooling/config/documentation
- `struct:` structural changes only (no behavioral impact)

## Git Push Policy

**Do not push without explicit human approval.**
