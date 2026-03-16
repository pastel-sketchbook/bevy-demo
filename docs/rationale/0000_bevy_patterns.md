# Bevy Patterns -- Code Reference

## Overview

This document catalogs every Bevy engine pattern used across the 16 demo
binaries in this repository. It serves as a reference for the ECS architecture,
rendering pipeline, input handling, scheduling, and Rust idioms specific to
Bevy 0.18. Each section identifies the pattern, shows where it appears, and
explains why it works the way it does.

The shared library `src/lib.rs` re-exports common types so every binary starts
with a single `use bevy_demo::*;` import.

---

## 1. Application Bootstrap

Every binary follows the same skeleton:

```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(default_window()),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(Update, (system_a, system_b, system_c))
        .run();
}
```

### `App::new()`

Creates an empty Bevy application. `App` is the top-level orchestrator that
owns the ECS `World`, the schedule runner, and plugin state.

### `DefaultPlugins`

A plugin group that adds everything needed for a typical app: windowing, asset
loading, rendering, input, audio, time, diagnostics, etc. Without it, there is
no window, no renderer, and no input.

### `.set(WindowPlugin { ... })`

Overrides one plugin within the group. Here it replaces the default window
configuration with `default_window()` from `src/lib.rs`, which sets:
- `decorations: false` (borderless)
- `transparent: true` (when the `transparent` feature is on)
- `resolution: 1606x1036`
- `position: Centered`

### `.run()`

Starts the event loop. This call never returns on most platforms (it enters
the OS event loop). All systems, resources, and entities must be registered
before this point.

**Used in**: every binary.

---

## 2. ECS Fundamentals

### Components

Data attached to entities. Defined with `#[derive(Component)]`:

```rust
#[derive(Component)]
struct Firefly;

#[derive(Component)]
struct FireflySpeed(f32);

#[derive(Component)]
struct Blink {
    speed: f32,
    timer: Timer,
    max_alpha: f32,
}
```

Components can be:
- **Marker types** (unit structs like `Firefly`, `Ball`, `Player`) -- used to
  tag entities for query filtering with `With<T>`
- **Newtype wrappers** (single-field tuples like `FireflySpeed(f32)`) -- attach
  one piece of data
- **Data structs** (like `Blink`, `OrbitCamera`, `BezierPath`) -- group related
  fields

**Used in**: every binary.

### Resources

Global singletons accessible by any system. Defined with `#[derive(Resource)]`:

```rust
#[derive(Resource)]
pub struct RandomSource(pub SmallRng);

#[derive(Resource)]
struct GameScore(u32);

#[derive(Resource)]
struct Rope {
    positions: Vec<Vec2>,
    old_positions: Vec<Vec2>,
    pinned: Vec<bool>,
}
```

Resources are inserted into the world via:

```rust
.insert_resource(ClearColor(BACKGROUND_COLOR))        // overwrites if exists
.insert_resource(RandomSource(SmallRng::seed_from_u64(SEED)))
commands.insert_resource(GameTimer(...));               // from within a system
```

**Used in**: every binary. Common resources include `RandomSource`, `ClearColor`,
`Time`, `Assets<T>`.

### Entities

Entities are just integer IDs. They have no data of their own -- components
give them meaning. Created via `commands.spawn(...)`:

```rust
commands.spawn((
    Mesh3d(firefly_mesh),
    MeshMaterial3d(firefly_material),
    Transform::from_translation(random_position),
    Firefly,
    FireflySpeed(speed_random),
));
```

The tuple `(A, B, C, ...)` is a **bundle**: all listed components are attached
to the new entity atomically.

**Used in**: every binary.

---

## 3. Systems

Systems are plain functions whose parameters are ECS accessors. Bevy
automatically injects the right data at runtime.

### Parameter types

| Parameter | Meaning | Example |
|---|---|---|
| `Res<T>` | Immutable reference to resource T | `time: Res<Time>` |
| `ResMut<T>` | Mutable reference to resource T | `mut rng: ResMut<RandomSource>` |
| `Query<Q, F>` | Iterate entities matching Q with filter F | `Query<&mut Transform, With<Boid>>` |
| `Commands` | Deferred entity/resource mutations | `mut commands: Commands` |
| `Local<T>` | Per-system persistent state | `mut done: Local<bool>` |
| `Single<Q, F>` | Exactly one matching entity (panics if 0 or 2+) | `Single<&mut Transform, With<TargetSphere>>` |
| `MessageWriter<T>` | Write events/messages | `mut app_exit: MessageWriter<AppExit>` |
| `Gizmos` | Immediate-mode debug drawing | `mut gizmos: Gizmos` |

### `Res<T>` vs `ResMut<T>`

`Res<T>` provides shared (read-only) access. `ResMut<T>` provides exclusive
(read-write) access. Bevy uses these types to determine which systems can run
in parallel: two systems that both use `Res<Time>` can run concurrently, but
a system using `ResMut<Time>` blocks all others that access `Time`.

### `Local<T>`

Per-system state that persists across frames but is invisible to other systems:

```rust
pub fn offset_window(mut windows: Query<&mut Window>, mut done: Local<bool>) {
    if *done { return; }
    // ... move window ...
    *done = true;
}
```

`Local<bool>` defaults to `false`. This pattern creates a "run once" system
without needing a separate resource.

**Used in**: `offset_window` (lib.rs).

### `Single<Q, F>`

A convenience wrapper that expects exactly one matching entity:

```rust
fn move_follower(
    mut following: Single<&mut Transform, With<FollowingSphere>>,
    target: Single<&Transform, (With<TargetSphere>, Without<FollowingSphere>)>,
    ...
)
```

If zero or multiple entities match, the system panics. Use `query.single()`
or `query.single_mut()` (which return `Result`) when the count might vary.

**Used in**: followings.rs.

---

## 4. System Scheduling

### Schedule labels

| Label | When it runs |
|---|---|
| `Startup` | Once, before the first frame |
| `Update` | Every frame |
| `FixedUpdate` | At a fixed timestep (decoupled from frame rate) |
| `OnEnter(State)` | Once when entering a state |
| `OnExit(State)` | Once when leaving a state |

### `.chain()`

Forces sequential execution within a system tuple:

```rust
.add_systems(Update, (flocking, apply_velocity, wrap_position, handle_quit).chain())
```

Without `.chain()`, Bevy may run these systems in any order or in parallel.
With `.chain()`, they execute in the listed order. This is critical when one
system's output is another's input (e.g., velocity must be computed before
position is updated).

**Used in**: flocking.rs, followings.rs, rope.rs.

### `.run_if()`

Conditionally runs systems based on a predicate:

```rust
.add_systems(Update,
    (menu_button_system, menu_button_colors).run_if(in_state(GameState::Menu)),
)
```

`in_state(GameState::Menu)` is a built-in run condition that returns `true`
only when the current state is `Menu`. Systems with `.run_if()` still exist
in the schedule but are skipped when the condition is false.

**Used in**: menu.rs.

### `FixedUpdate` with custom Hz

```rust
.insert_resource(Time::<Fixed>::from_hz(120.0))
.add_systems(FixedUpdate, (verlet_integrate, apply_constraints).chain())
```

`Time::<Fixed>` configures the fixed timestep. Bevy accumulates real time and
runs `FixedUpdate` zero or more times per frame to maintain the target rate.
Systems in `FixedUpdate` always see the same `dt`, making physics deterministic.

**Used in**: rope.rs (120 Hz), life.rs (10 Hz).

---

## 5. States

Bevy's state machine for managing game phases:

```rust
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    GameOver,
}
```

### Registration

```rust
.init_state::<GameState>()
```

This creates the state resource initialized to the `#[default]` variant.

### Transitions

```rust
fn playing_input(mut next_state: ResMut<NextState<GameState>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}
```

`NextState<T>` is a resource. Setting it schedules a transition that occurs
between frames. `OnExit(old)` runs, then `OnEnter(new)` runs.

### Generic cleanup

```rust
fn cleanup<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

.add_systems(OnExit(GameState::Menu), cleanup::<StateEntity>)
```

A single generic function handles cleanup for all states. Entities tagged with
`StateEntity` are despawned when leaving any state. This avoids writing separate
cleanup functions for each state.

**Used in**: menu.rs.

---

## 6. Rendering

### 2D Rendering

```rust
commands.spawn(Camera2d);   // required for 2D rendering

// Mesh-based 2D shape
commands.spawn((
    Mesh2d(meshes.add(Circle::new(10.0))),
    MeshMaterial2d(materials.add(Color::srgb(1.0, 1.0, 0.0))),
    Transform::from_xyz(0.0, 0.0, 0.0),
));

// Sprite-based 2D
commands.spawn((
    Sprite {
        color: ANIMATION_COLORS[0],
        custom_size: Some(Vec2::splat(64.0)),
        ..default()
    },
    Transform::from_translation(Vec3::ZERO),
));
```

- `Mesh2d` + `MeshMaterial2d<ColorMaterial>`: procedural shapes (circles,
  rectangles, triangles) with color materials
- `Sprite`: image-based or solid-color rectangles with built-in size/color

**Used in**: bouncing.rs, flocking.rs, particles.rs, pong.rs (Mesh2d); sprites.rs (Sprite); life.rs (Sprite with CPU texture).

### 3D Rendering

```rust
commands.spawn((Camera3d::default(), Transform::from_xyz(...).looking_at(Vec3::ZERO, Vec3::Y)));

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

- `Camera3d`: perspective camera
- `Mesh3d` + `MeshMaterial3d<StandardMaterial>`: PBR meshes with physically-
  based materials
- `PointLight`: omnidirectional light source
- `StandardMaterial` fields: `base_color`, `emissive`, `metallic`,
  `perceptual_roughness`, `reflectance`, `alpha_mode`

**Used in**: firefly.rs, followings.rs, cubic.rs, orbit.rs, bloom.rs, mandala.rs.

### Gizmos (Immediate Mode)

```rust
fn draw_clock(mut gizmos: Gizmos) {
    gizmos.line_2d(start, end, color);
    gizmos.circle_2d(center, radius, color);
}
```

Gizmos are drawn for one frame only -- no entity or mesh management. Ideal for:
- Debug visualization
- Dynamic geometry that changes every frame (clock hands, rope segments)
- Prototyping without asset setup

**Used in**: clock.rs, rope.rs, mandala.rs.

### Post-Processing (HDR + Bloom)

```rust
use bevy::{post_process::bloom::Bloom, render::view::Hdr};

commands.spawn((
    Camera3d::default(),
    Hdr,
    Bloom { intensity: 0.3, ..default() },
    Transform::from_xyz(...),
));
```

`Hdr` marks the camera for high-dynamic-range rendering. `Bloom` applies a
post-processing bloom effect that makes bright (emissive) objects glow.
Emissive values > 1.0 in `StandardMaterial` are what trigger visible bloom.

**Used in**: bloom.rs.

---

## 7. Assets

The asset system manages meshes, materials, textures, and other GPU resources.

### Creating assets

```rust
fn setup(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_handle: Handle<Mesh> = meshes.add(Circle::new(10.0));
    let mat_handle: Handle<ColorMaterial> = materials.add(Color::srgb(1.0, 0.0, 0.0));
}
```

`meshes.add(M)` takes anything that implements `Into<Mesh>` (primitives like
`Circle`, `Sphere`, `Cuboid`, `Rectangle`, etc.), stores it in the asset
collection, and returns a `Handle<Mesh>`. Handles are lightweight reference-
counted pointers.

### Mutating materials at runtime

Several demos modify material properties per frame (alpha fade, emissive pulse,
color change):

```rust
fn pulse_emissive(
    query: Query<(&EmissiveShape, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (shape, material_handle) in query.iter() {
        let Some(material) = materials.get_mut(&material_handle.0) else {
            continue;
        };
        material.emissive = shape.base_color * intensity;
    }
}
```

The pattern is:
1. Query entities to get their `MeshMaterial3d<T>` (which holds a `Handle<T>`)
2. Use `materials.get_mut(&handle)` to get mutable access to the actual material
3. Modify fields directly

**Used in**: bloom.rs (emissive), firefly.rs (alpha), particles.rs (alpha).

### CPU Textures

```rust
let mut image = Image::new_fill(
    Extent3d { width: 320, height: 206, depth_or_array_layers: 1 },
    TextureDimension::D2,
    &CELL_DEAD_COLOR,
    TextureFormat::Rgba8UnormSrgb,
    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
);
image.sampler = ImageSampler::nearest();
let handle = images.add(image);
```

`RenderAssetUsages::MAIN_WORLD | RENDER_WORLD` keeps the pixel data accessible
on both CPU (for writing) and GPU (for rendering). The `render_grid` system
writes pixel data directly:

```rust
let data = image.data.as_mut().expect("Image has no CPU data");
data[idx] = color[0];   // R
data[idx+1] = color[1]; // G
data[idx+2] = color[2]; // B
data[idx+3] = color[3]; // A
```

**Used in**: life.rs.

---

## 8. Input

### Keyboard

```rust
fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {     // held down
        app_exit.write(AppExit::Success);
    }
}

if keyboard.just_pressed(KeyCode::Space) {   // first frame only
    score.0 += 1;
}
```

- `pressed()`: true every frame while held
- `just_pressed()`: true only on the frame the key first goes down
- `just_released()`: true only on the frame the key goes up

### Mouse Buttons

```rust
let painting = buttons.pressed(MouseButton::Left);
let erasing = buttons.pressed(MouseButton::Right);
```

Same API as keyboard: `Res<ButtonInput<MouseButton>>`.

### Mouse Motion and Scroll

```rust
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};

fn orbit_camera(
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
) {
    orbit.yaw -= mouse_motion.delta.x * ORBIT_SENSITIVITY;
    orbit.distance -= mouse_scroll.delta.y * ZOOM_SENSITIVITY;
}
```

`AccumulatedMouseMotion` sums all mouse movement events since last frame.
This is more reliable than per-event processing because it handles high-polling-
rate mice and low frame rates correctly.

**Used in**: orbit.rs.

### Viewport-to-World Conversion

```rust
let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
    return;
};
```

Converts screen-space pixel coordinates (origin top-left, Y-down) to world-
space coordinates (origin center, Y-up for 2D). Essential for mouse interaction
with game objects.

**Used in**: rope.rs, life.rs.

---

## 9. Time

### Frame time

```rust
let dt = time.delta_secs();           // seconds since last frame (f32)
let elapsed = time.elapsed_secs();    // total seconds since app start (f32)
```

`Res<Time>` is automatically provided by Bevy. In `Update`, it gives wall-clock
frame time. In `FixedUpdate`, it gives the fixed timestep.

### Fixed time

```rust
.insert_resource(Time::<Fixed>::from_hz(120.0))
```

Configures the fixed timestep. Within `FixedUpdate` systems, `time.delta_secs()`
always returns `1.0 / 120.0`.

### Timers

```rust
let timer = Timer::from_seconds(0.15, TimerMode::Repeating);
timer.tick(time.delta());

if timer.just_finished() {
    // fires once per period
}

// How many times the timer fired this tick (for catch-up):
for _ in 0..timer.times_finished_this_tick() { ... }
```

- `TimerMode::Repeating`: resets automatically, fires repeatedly
- `TimerMode::Once`: fires once, then stays finished
- `just_finished()`: true only on the tick that crossed the threshold
- `times_finished_this_tick()`: for large `dt`, a repeating timer might fire
  multiple times

**Used in**: sprites.rs, particles.rs, firefly.rs, menu.rs.

---

## 10. Queries

### Basic query

```rust
fn move_sprite(mut query: Query<&mut Transform, With<Player>>) {
    for mut transform in query.iter_mut() {
        transform.translation += direction * speed * dt;
    }
}
```

`Query<&mut Transform, With<Player>>` iterates all entities that have both a
`Transform` component (mutable access) and a `Player` component (used as filter
only, not accessed).

### Multi-component queries

```rust
Query<(&mut Transform, &Velocity, &mut Lifetime, &MeshMaterial2d<ColorMaterial>)>
```

Tuples access multiple components from the same entity. All components in the
tuple must be present on the entity for it to match.

### Filters

| Filter | Meaning |
|---|---|
| `With<T>` | Entity must have component T (not accessed) |
| `Without<T>` | Entity must NOT have component T |
| `Changed<T>` | Only entities where T was mutated this frame |

### Conflict resolution with `Without<>`

```rust
fn move_paddles(
    mut left_paddle: Query<&mut Transform, (With<LeftPaddle>, Without<RightPaddle>, Without<Ball>)>,
    mut right_paddle: Query<&mut Transform, (With<RightPaddle>, Without<LeftPaddle>, Without<Ball>)>,
) { ... }
```

Two `Query<&mut Transform>` parameters would conflict (both want mutable access
to `Transform`). `Without<>` guarantees the queries access disjoint sets of
entities, resolving the conflict.

**Used in**: pong.rs.

### `query.single()` and `query.single_mut()`

```rust
let Ok((mut orbit, mut transform)) = query.single_mut() else {
    return;
};
```

Returns the single matching entity, or `Err` if zero or multiple match. Used
when exactly one entity is expected (camera, ball, etc.).

---

## 11. Commands (Deferred Mutations)

### Spawning entities

```rust
commands.spawn((ComponentA, ComponentB, ComponentC));
```

`spawn` returns an `EntityCommands` builder. Commands are deferred: they execute
between system runs, not inline.

### Despawning entities

```rust
commands.entity(entity).despawn();
```

Removes the entity and all its components from the world.

### Hierarchies with `.with_children()`

```rust
commands
    .spawn((Node { ... }, StateEntity))
    .with_children(|parent| {
        parent.spawn((Text::new("PLAY"), TextFont { ... }));
    });
```

Creates parent-child relationships. Child transforms are relative to the parent.
Despawning a parent does NOT automatically despawn children (use
`despawn_recursive()` for that, or tag children for manual cleanup).

**Used in**: menu.rs (UI hierarchies).

### Inserting resources from systems

```rust
commands.insert_resource(GameScore(0));
commands.insert_resource(GameTimer(Timer::from_seconds(10.0, TimerMode::Once)));
```

Useful for initializing resources from within `OnEnter` systems.

---

## 12. UI

Bevy's UI uses a flexbox layout model:

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
    StateEntity,
));
```

### Key UI components

| Component | Purpose |
|---|---|
| `Node` | Layout container (like a `<div>`) |
| `Text` | Text display |
| `TextFont` | Font size and family |
| `TextColor` | Text color |
| `Button` | Clickable element with `Interaction` |
| `BackgroundColor` | Background fill |

### Button interaction

```rust
fn menu_button_system(
    query: Query<(&Interaction, &MenuButton), Changed<Interaction>>,
) {
    for (interaction, button) in query.iter() {
        if *interaction == Interaction::Pressed {
            match button {
                MenuButton::Play => next_state.set(GameState::Playing),
                MenuButton::Quit => app_exit.write(AppExit::Success),
            }
        }
    }
}
```

`Changed<Interaction>` filters to only entities whose `Interaction` component
changed this frame. `Interaction` is automatically updated by Bevy's UI system
to `None`, `Hovered`, or `Pressed`.

**Used in**: menu.rs.

---

## 13. Color Types

Bevy 0.18 has multiple color representations:

| Type | Space | When to use |
|---|---|---|
| `Color::srgb(r, g, b)` | sRGB | Display colors (gamma-corrected) |
| `Color::srgba(r, g, b, a)` | sRGB + alpha | Transparent display colors |
| `Color::hsl(h, s, l)` | HSL | Hue-based generation (pastel palettes) |
| `Color::hsla(h, s, l, a)` | HSL + alpha | Hue-based with transparency |
| `LinearRgba` | Linear RGB | Emissive values, math on light |
| `Srgba` | sRGB struct | Pattern-matching color channels |
| `Hsla` | HSL struct | Pattern-matching HSL channels |

### Pattern matching on color variants

```rust
if let Color::Srgba(Srgba { red, green, blue, .. }) = material.base_color {
    material.base_color = Color::srgba(red, green, blue, new_alpha);
}
```

Colors are enums with variant-specific structs. Destructuring extracts channels
without conversion.

### LinearRgba for emissive scaling

```rust
let emissive = LinearRgba::new(0.95, 0.55, 0.65, 1.0) * 5.0;
```

Emissive values are in linear space. Multiplying by a scalar > 1.0 creates
HDR intensity that the bloom post-process picks up. `LinearRgba` supports
`Mul<f32>` for uniform scaling.

**Used in**: bloom.rs.

---

## 14. Transforms and Math Types

### Transform

```rust
Transform::from_xyz(x, y, z)
Transform::from_translation(Vec3::new(x, y, z))
Transform::from_xyz(-2.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y)
```

`Transform` holds translation (position), rotation (quaternion), and scale.
`.looking_at(target, up)` orients the entity to face `target` with `up` as the
reference up direction.

### Quaternion rotation

```rust
transform.rotate(Quat::from_axis_angle(axis, angle));
transform.rotation = Quat::from_rotation_z(angle);
```

- `from_axis_angle`: rotate around arbitrary axis
- `from_rotation_z`: rotate around Z-axis (useful for 2D heading)
- `transform.rotate()`: multiply existing rotation by new one (accumulative)

### Vec2 / Vec3

```rust
Vec2::new(x, y)
Vec3::new(x, y, z)
Vec3::splat(10.0)           // (10, 10, 10)
Vec3::ZERO                  // (0, 0, 0)
Vec3::Y                     // (0, 1, 0)
velocity.0.extend(0.0)      // Vec2 -> Vec3 (adds z=0)
translation.truncate()      // Vec3 -> Vec2 (drops z)
direction.normalize()        // unit vector
direction.length()           // magnitude
direction.length_squared()   // magnitude^2 (avoids sqrt)
direction.clamp_length_max(MAX) // cap magnitude
Vec2::from_angle(theta)      // (cos(theta), sin(theta))
```

### Dir3

```rust
match Dir3::new(target - current) {
    Ok(dir) => { /* valid direction */ }
    Err(_) => { /* vectors are too close, direction undefined */ }
}
```

`Dir3` is a validated unit vector. `Dir3::new()` returns `Err` if the input is
too close to zero, preventing NaN from normalization.

**Used in**: followings.rs.

### ShapeSample

```rust
let legal_region = Cuboid::from_size(Vec3::splat(WORLD_SIZE));
let random_point = legal_region.sample_interior(&mut rng);
```

`ShapeSample` trait (from `bevy::math`) provides `sample_interior()` to generate
random points uniformly distributed inside a shape. Works with `Cuboid`,
`Sphere`, `Circle`, etc.

**Used in**: firefly.rs, followings.rs.

### `smooth_nudge`

```rust
following.translation.smooth_nudge(&target.translation, decay_rate, delta_time);
```

From `bevy::math::NormedVectorSpace`. Applies exponential decay interpolation:
the follower moves toward the target, covering a fraction of the remaining
distance each frame. The `decay_rate` controls how fast the gap closes.

**Used in**: followings.rs.

---

## 15. Window and Display

### ClearColor

```rust
.insert_resource(ClearColor(BACKGROUND_COLOR))
```

Sets the background color for the entire window. Every binary defines its own
color constant via `background_color()`.

### Transparent windows

```rust
pub const fn background_color(r: f32, g: f32, b: f32, alpha: f32) -> Color {
    #[cfg(feature = "transparent")]
    { Color::srgba(r, g, b, alpha) }
    #[cfg(not(feature = "transparent"))]
    { Color::srgb(r, g, b) }
}
```

When the `transparent` feature is enabled, the window background is semi-
transparent, letting the desktop wallpaper show through. The window itself
must also be configured with `transparent: true` and
`CompositeAlphaMode::PostMultiplied`.

### Window offset (development helper)

```rust
#[cfg(feature = "window-offset")]
pub fn offset_window(mut windows: Query<&mut Window>, mut done: Local<bool>) {
    if *done { return; }
    window.position = WindowPosition::At(IVec2::new(160, 88));
    *done = true;
}
```

Moves the window to a fixed screen position for consistent development. The
`Local<bool>` guard ensures it runs only once. Conditionally compiled via
`#[cfg(feature = "window-offset")]`.

---

## 16. Events and App Exit

### MessageWriter

```rust
fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}
```

`MessageWriter<AppExit>` sends an `AppExit` event that triggers application
shutdown. This replaced the older `EventWriter<AppExit>` pattern in Bevy 0.18.

**Used in**: every binary (via shared `handle_quit` or local quit handlers).

---

## 17. Feature Flags

### Cargo features

```toml
[features]
window-offset = []
transparent = []
```

### Conditional compilation

```rust
#[cfg(feature = "window-offset")]
offset_window,

#[cfg(feature = "transparent")]
transparent: true,
```

Features affect:
- Whether `offset_window` is added to the system schedule
- Whether the window is transparent
- Whether `background_color()` uses alpha

Systems gated with `#[cfg(feature = "...")]` are completely removed from the
binary when the feature is off -- zero runtime cost.

---

## 18. Shared Library (`src/lib.rs`)

The library crate re-exports common items so binaries use a single import:

```rust
use bevy_demo::*;
```

### Re-exports

```rust
pub use bevy::app::AppExit;
pub use bevy::math::ShapeSample;
pub use bevy::prelude::*;
pub use bevy::window::{PrimaryWindow, WindowPlugin, WindowPosition, WindowResolution};
pub use rand::rngs::SmallRng;
pub use rand::{Rng, SeedableRng};
```

### Shared items

| Item | Type | Purpose |
|---|---|---|
| `WINDOW_WIDTH` / `WINDOW_HEIGHT` | `const f32` | Standard window dimensions |
| `background_color()` | `const fn` | Feature-aware background color |
| `default_window()` | `fn` | Standard borderless window config |
| `handle_quit()` | system | Q-key quit handler |
| `offset_window()` | system | Dev window positioning |
| `RandomSource` | `Resource` | Seeded RNG wrapper |
| `Velocity` | `Component` | 2D velocity (`Vec2`) |

This eliminates ~30 lines of boilerplate per binary.

---

## 19. Deterministic Randomness

```rust
.insert_resource(RandomSource(SmallRng::seed_from_u64(RANDOM_SEED)))
```

Every demo that uses randomness seeds its RNG from a constant. This makes
behavior reproducible across runs -- the same seed produces the same sequence
of random numbers. `SmallRng` is a non-cryptographic, fast PRNG suitable for
games.

To access it in systems:

```rust
fn spawn(mut rng: ResMut<RandomSource>) {
    let angle = rng.0.random_range(0.0..TAU);
}
```

**Used in**: firefly.rs, flocking.rs, particles.rs, pong.rs, life.rs, gravity.rs, followings.rs.

---

## 20. Common Patterns Summary

| Pattern | Where | Why |
|---|---|---|
| Marker components + `With<T>` filter | Every binary | Tag entities for query discrimination |
| Newtype wrappers for components | Most binaries | Type safety, avoids "stringly typed" data |
| `let Ok(...) = query.single_mut() else { return }` | orbit.rs, pong.rs, life.rs | Safe single-entity access with early exit |
| `let-chain` (`if let A && let B`) | particles.rs, firefly.rs | Flat chaining of fallible patterns |
| `..default()` struct update syntax | Every binary | Fill remaining fields with defaults |
| `.clamp(min, max)` | Most binaries | Bound values in one readable call |
| `iter_mut()` over queries | Every binary | Iterate entities with mutable access |
| Handle cloning for shared meshes | flocking.rs, pong.rs | Multiple entities share one mesh asset |
| `#[allow(clippy::...)]` | pong.rs, menu.rs | Suppress warnings for complex query signatures |
| `matches!((alive, n), (true, 2)\|(true, 3)\|(false, 3))` | life.rs | Compact multi-condition boolean logic |
