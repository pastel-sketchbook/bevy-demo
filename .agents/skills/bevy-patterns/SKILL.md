---
name: bevy-patterns
description: Bevy 0.18 ECS patterns, conventions, and shared library reference specific to this project's 16 demo binaries. Use when writing or modifying Bevy code in this repo.
---

## What I do

Provide the Bevy 0.18 code patterns and project conventions used across all 16 demo binaries in this repository.

## When to use me

- When writing a new binary in `src/bin/`
- When modifying an existing demo
- When you need the correct Bevy 0.18 API for components, resources, systems, queries, or rendering
- When unsure about this project's shared library (`src/lib.rs`) exports

## Shared Library (`src/lib.rs`)

Every binary starts with `use bevy_demo::*;` which provides:

### Re-exports
```rust
pub use bevy::app::AppExit;
pub use bevy::math::ShapeSample;
pub use bevy::prelude::*;
pub use bevy::window::{PrimaryWindow, WindowPlugin, WindowPosition, WindowResolution};
pub use rand::rngs::SmallRng;
pub use rand::{Rng, SeedableRng};
```

### Constants
- `WINDOW_WIDTH: f32 = 1606.0`
- `WINDOW_HEIGHT: f32 = 1036.0`

### Functions
- `background_color(r, g, b, alpha) -> Color` — feature-aware (transparent vs opaque)
- `default_window() -> Window` — borderless 1606x1036, centered, optional transparency

### Systems
- `handle_quit` — exits on Q key
- `offset_window` — moves window to (160, 88), gated by `#[cfg(feature = "window-offset")]`

### Shared Types
- `RandomSource(pub SmallRng)` — `#[derive(Resource)]`
- `Velocity(pub Vec2)` — `#[derive(Component)]`

## Binary Bootstrap Template

```rust
use bevy_demo::*;

const BACKGROUND_COLOR: Color = background_color(0.05, 0.05, 0.08, 0.85);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(default_window()),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            system_a,
            system_b,
            handle_quit,
            #[cfg(feature = "window-offset")]
            offset_window,
        ))
        .run();
}
```

## ECS Patterns

### Components
```rust
#[derive(Component)]
struct Firefly;                    // marker — used with With<Firefly>

#[derive(Component)]
struct FireflySpeed(f32);          // newtype wrapper

#[derive(Component)]
struct Blink {                     // data struct
    speed: f32,
    timer: Timer,
    max_alpha: f32,
}
```

### Resources
```rust
#[derive(Resource)]
pub struct RandomSource(pub SmallRng);

// Insert during app build:
.insert_resource(RandomSource(SmallRng::seed_from_u64(42)))

// Or from a system:
commands.insert_resource(GameScore(0));
```

### Spawning Entities
```rust
commands.spawn((
    Mesh3d(meshes.add(Sphere::new(0.3))),
    MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.68, 0.72),
        ..default()
    })),
    Transform::from_translation(position),
    Firefly,
    FireflySpeed(speed),
));
```

## System Parameters

| Parameter | Purpose |
|---|---|
| `Res<T>` | Read-only resource |
| `ResMut<T>` | Mutable resource |
| `Query<Q, F>` | Entity iteration with filter |
| `Commands` | Deferred spawn/despawn/insert |
| `Local<T>` | Per-system persistent state |
| `Single<Q, F>` | Exactly one entity (panics if 0 or 2+) |
| `MessageWriter<T>` | Write events (e.g., `AppExit`) |
| `Gizmos` | Immediate-mode drawing |

## Query Patterns

```rust
// Basic iteration
fn move_boids(mut query: Query<(&mut Transform, &Velocity), With<Boid>>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0.extend(0.0) * dt;
    }
}

// Conflict resolution — Without<> makes queries disjoint
fn move_paddles(
    mut left: Query<&mut Transform, (With<LeftPaddle>, Without<RightPaddle>)>,
    mut right: Query<&mut Transform, (With<RightPaddle>, Without<LeftPaddle>)>,
) { ... }

// Single entity
let Ok((mut orbit, mut transform)) = query.single_mut() else { return; };
```

## Scheduling

```rust
// Sequential execution
.add_systems(Update, (flocking, apply_velocity, wrap_position).chain())

// Conditional execution
.add_systems(Update, game_systems.run_if(in_state(GameState::Playing)))

// Fixed timestep
.insert_resource(Time::<Fixed>::from_hz(120.0))
.add_systems(FixedUpdate, (verlet_integrate, apply_constraints).chain())

// State lifecycle
.add_systems(OnEnter(GameState::Playing), setup_playing)
.add_systems(OnExit(GameState::Menu), cleanup::<StateEntity>)
```

## Rendering Patterns

### 2D
```rust
commands.spawn(Camera2d);

// Mesh-based
commands.spawn((
    Mesh2d(meshes.add(Circle::new(10.0))),
    MeshMaterial2d(materials.add(Color::srgb(1.0, 0.0, 0.0))),
    Transform::from_xyz(0.0, 0.0, 0.0),
));

// Sprite-based
commands.spawn((
    Sprite { color: Color::srgb(1.0, 1.0, 1.0), custom_size: Some(Vec2::splat(64.0)), ..default() },
    Transform::from_translation(Vec3::ZERO),
));
```

### 3D
```rust
commands.spawn((Camera3d::default(), Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y)));

commands.spawn((
    Mesh3d(meshes.add(Sphere::new(0.3))),
    MeshMaterial3d(materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.68, 0.72),
        emissive: LinearRgba::new(0.95, 0.55, 0.65, 1.0) * 5.0,
        metallic: 0.6,
        perceptual_roughness: 0.35,
        ..default()
    })),
    Transform::from_translation(position),
));

commands.spawn((
    PointLight { intensity: 15_000_000.0, shadows_enabled: true, ..default() },
    Transform::from_xyz(4.0, 8.0, 4.0),
));
```

### Gizmos (immediate mode — one frame only)
```rust
fn draw(mut gizmos: Gizmos) {
    gizmos.line_2d(start, end, color);
    gizmos.circle_2d(center, radius, color);
}
```

### HDR + Bloom
```rust
use bevy::{post_process::bloom::Bloom, render::view::Hdr};

commands.spawn((Camera3d::default(), Hdr, Bloom { intensity: 0.3, ..default() }, Transform::from_xyz(...)));
// Emissive values > 1.0 trigger visible bloom
```

### Custom GPU Shader (Material2d)
```rust
use bevy::sprite::{Material2d, Material2dPlugin, AlphaMode2d};
use bevy::render::render_resource::{AsBindGroup, ShaderType};

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct MyMaterial {
    #[uniform(0)]
    params: MyParams,
}

impl Material2d for MyMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/my_shader.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d { AlphaMode2d::Blend }
}

// Register: .add_plugins(Material2dPlugin::<MyMaterial>::default())
```

## Input

```rust
// Keyboard
keyboard.pressed(KeyCode::KeyQ)       // held
keyboard.just_pressed(KeyCode::Space)  // first frame only

// Mouse buttons
buttons.pressed(MouseButton::Left)

// Mouse motion (accumulated per frame)
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
mouse_motion.delta.x  // pixels moved

// Screen to world
camera.viewport_to_world_2d(camera_transform, cursor_pos)
```

## Time and Timers

```rust
let dt = time.delta_secs();            // frame time (f32)
let elapsed = time.elapsed_secs();     // total time (f32)

let timer = Timer::from_seconds(0.15, TimerMode::Repeating);
timer.tick(time.delta());
if timer.just_finished() { /* fires once per period */ }
```

## Color Types

| Type | Use |
|---|---|
| `Color::srgb(r, g, b)` | Display colors |
| `Color::srgba(r, g, b, a)` | With transparency |
| `Color::hsl(h, s, l)` | Hue-based (pastel generation) |
| `LinearRgba` | Emissive values, light math |

```rust
// Emissive scaling (HDR)
let emissive = LinearRgba::new(0.95, 0.55, 0.65, 1.0) * 5.0;

// Pattern matching
if let Color::Srgba(Srgba { red, green, blue, .. }) = material.base_color {
    material.base_color = Color::srgba(red, green, blue, new_alpha);
}
```

## Math

```rust
Vec2::from_angle(theta)               // (cos, sin)
velocity.0.extend(0.0)                // Vec2 -> Vec3
translation.truncate()                // Vec3 -> Vec2
direction.normalize()
direction.clamp_length_max(MAX)

// Safe direction
match Dir3::new(target - current) {
    Ok(dir) => { /* valid */ }
    Err(_) => { /* too close to zero */ }
}

// Random points in shape
let region = Cuboid::from_size(Vec3::splat(WORLD_SIZE));
let point = region.sample_interior(&mut rng);

// Smooth interpolation
following.translation.smooth_nudge(&target.translation, decay_rate, dt);
```

## States (Game State Machine)

```rust
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    GameOver,
}

.init_state::<GameState>()

// Transition
fn input(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Paused);
}

// Generic cleanup
fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() { commands.entity(entity).despawn(); }
}
.add_systems(OnExit(GameState::Menu), cleanup::<StateEntity>)
```

## UI (Flexbox)

```rust
commands.spawn((
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        row_gap: Val::Px(30.0),
        ..default()
    },
)).with_children(|parent| {
    parent.spawn((Text::new("PLAY"), TextFont { font_size: 48.0, ..default() }));
});

// Button interaction
fn button_system(query: Query<(&Interaction, &MenuButton), Changed<Interaction>>) {
    for (interaction, button) in query.iter() {
        if *interaction == Interaction::Pressed { /* handle */ }
    }
}
```

## Feature Flags

```rust
// Conditional system registration
#[cfg(feature = "window-offset")]
offset_window,

// Conditional struct fields
#[cfg(feature = "transparent")]
transparent: true,
```

Features: `window-offset` (dev positioning), `transparent` (see-through window).

## Deterministic Randomness

```rust
const RANDOM_SEED: u64 = 42;
.insert_resource(RandomSource(SmallRng::seed_from_u64(RANDOM_SEED)))

fn spawn(mut rng: ResMut<RandomSource>) {
    let angle = rng.0.random_range(0.0..TAU);
}
```

## Adding a New Binary

1. Create `src/bin/name.rs` with `use bevy_demo::*;`
2. Add `[[bin]]` entry to `Cargo.toml`
3. Add `run:name` task to `Taskfile.yml`
4. Add entry to README.md Examples table and Project Structure
5. Add entry to AGENTS.md Project Structure
6. Create `docs/rationale/NNNN_name.md` math rationale document
