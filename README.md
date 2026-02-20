# Bevy Demo

A collection of self-contained examples for the [Bevy](https://bevyengine.org/) 0.18 game engine in Rust. Each binary demonstrates specific engine concepts — from basic 2D rendering to post-processing, state machines, and physics simulations.

## Tech Stack

- **Rust** (Edition 2024)
- **Bevy 0.18** — Data-driven game engine (ECS)
- **jiff 0.2** — Timezone-aware datetime (IANA database)
- **rand 0.9** — Deterministic randomness (`SmallRng`)

## Examples

| Demo | Description | Concepts |
|------|-------------|----------|
| `bloom` | HDR bloom with pulsing emissive pastel shapes orbiting over a silver plane | Post-processing, `Bloom`, `Hdr`, emissive `StandardMaterial` |
| `bouncing` | Shapes bouncing off window edges with velocity and shape morphing | 2D movement, boundary collision, `Mesh2d` |
| `clock` | Analog clock with hour/minute/second hands, America/Chicago timezone | Gizmos, `jiff` timezone, real wall-clock time |
| `cubic` | 3D rotating cube with "PASTEL" bitmap font faces and reflection twins | CPU texture, 3D transforms, bitmap font rendering |
| `firefly` | 3D firefly simulation with blinking, movement, and keyboard input | 3D scene, `PointLight`, keyboard input |
| `flocking` | Boids flocking with separation, alignment, and cohesion | Spatial queries, emergent behavior, `SmallRng` |
| `followings` | Smooth entity interpolation/following | Lerp, `Transform`, system chaining |
| `gravity` | Gravitational attraction with a sun and orbiting planets (Space to add) | N-body simulation, `SmallRng`, keyboard input |
| `life` | Conway's Game of Life with mouse painting and CPU texture rendering | `FixedUpdate`, mouse input, `Image` pixel writes |
| `menu` | State machine: Menu / Playing / Paused / GameOver with UI buttons | `States`, `OnEnter`/`OnExit`, UI nodes, `Button`, `Interaction` |
| `orbit` | 3D camera orbit controller with auto-rotating pastel primitives | `AccumulatedMouseMotion`, `AccumulatedMouseScroll`, spherical coords |
| `particles` | Continuous particle spawning with velocity, lifetime, and fade-out | Entity lifecycle, `Sprite`, despawn |
| `pong` | Pong with bezier ball movement, AI paddles, and scoring | 2D collision, `Text2d`, `Mesh2d` |
| `rope` | Verlet integration rope simulation with mouse-draggable anchor | `FixedUpdate` 120Hz, Gizmos, constraint solver |
| `sprites` | Sprite animation with procedural color cycling | `Sprite`, keyboard movement, color math |

## Quick Start

```bash
# Build all examples
task build                    # cargo fmt + release build with features

# Run a specific example
task run:clock
task run:bloom
task run:life
cargo run --bin orbit         # without Taskfile

# Run default (firefly)
task run
```

All examples use a 1606x1036 borderless window. Press **Q** to quit.

## Feature Flags

| Feature | Description |
|---------|-------------|
| `window-offset` | Position window at (160, 88) for multi-monitor dev setups |
| `transparent` | Semi-transparent window background (see desktop through it) |

```bash
# Build with features (Taskfile default)
task build

# Build without features
cargo build --release

# Run with specific features
cargo run --bin bloom --features "transparent,window-offset"
```

## Project Structure

```
src/bin/           # 15 standalone example binaries
  bloom.rs         # HDR bloom post-processing
  bouncing.rs      # 2D bouncing shapes
  clock.rs         # Analog clock with Gizmos + jiff timezone
  cubic.rs         # 3D rotating cube with bitmap font
  firefly.rs       # 3D firefly simulation
  flocking.rs      # Boids flocking
  followings.rs    # Smooth interpolation
  gravity.rs       # Orbital mechanics
  life.rs          # Conway's Game of Life
  menu.rs          # State machine + UI
  orbit.rs         # 3D camera orbit controller
  particles.rs     # Particle system
  pong.rs          # Pong game
  rope.rs          # Verlet rope physics
  sprites.rs       # Sprite animation
assets/            # Game assets
Cargo.toml         # 15 [[bin]] entries + dependencies
Taskfile.yml       # Build and run tasks
AGENTS.md          # AI agent conventions
```

## Design Rationale

Each example is designed around a specific learning goal:

**Rendering techniques** — The demos cover the rendering spectrum from immediate-mode Gizmos (`clock`, `rope`) to CPU-drawn textures (`life`, `cubic`), standard 3D materials with lighting (`bloom`, `orbit`, `firefly`), and 2D sprites/meshes (`bouncing`, `particles`, `sprites`, `pong`).

**Input handling** — Keyboard input appears in most demos (Q to quit, Space for actions). Mouse input is demonstrated across three interaction models: click-to-paint (`life`), drag-to-orbit (`orbit`), and drag-to-move (`rope`).

**Simulation patterns** — `FixedUpdate` is used at different rates depending on the simulation: 10Hz for Game of Life (visible discrete steps), 120Hz for rope physics (stability requires high frequency). Standard `Update` is used where frame-rate-dependent behavior is acceptable.

**State machines** — `menu` demonstrates the full `States` lifecycle with `OnEnter`/`OnExit` cleanup, multiple game states, and UI interaction — patterns that most real games need but basic demos often skip.

**Post-processing** — `bloom` shows HDR rendering with the `Hdr` marker component and `Bloom` post-process, using emissive materials with intensity values above 1.0 to create the glow effect.

**Visual style** — All demos use a consistent pastel color palette with dark backgrounds. 3D scenes use semi-transparent silver ground planes. The aesthetic is intentionally soft to keep the focus on the code patterns rather than art assets.

**Self-contained** — Every binary runs independently with zero asset dependencies (except `cubic` which embeds its bitmap font). No shared library crate — duplication across binaries is acceptable because each file should be readable in isolation.
