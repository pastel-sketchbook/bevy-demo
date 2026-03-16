# Firefly -- Math Rationale

## Overview

The firefly demo simulates 3D insects drifting toward random target positions
within a cubic volume while blinking their glow. The math covers
direction-clamped movement (steering without overshoot) and sinusoidal alpha
oscillation for a smooth blink effect.

## 1. Direction-Clamped Movement

Each firefly moves toward a target position with overshoot prevention:

```rust
let target_direction = target_pos.0 - transform.translation;
if target_direction == Vec3::ZERO {
    target_pos.0 = legal_region.sample_interior(&mut rng.0);
    continue;
}
let move_direction = target_direction.normalize();
let abs_delta = target_direction.length();
let magnitude = f32::min(abs_delta, delta_time * target_speed.0);
transform.translation += move_direction * magnitude;
```

### The formula

```
d = target - position
direction = d / |d|
step = min(|d|, speed * dt)
position += direction * step
```

The key insight is the `min(|d|, speed * dt)`:

- When the firefly is far from the target, `speed * dt < |d|`, so it moves at
  full speed.
- When close, `|d| < speed * dt`, so it moves exactly to the target without
  overshooting.

This is a common game-dev pattern sometimes called "move toward" or "clamped
linear interpolation." Without the `min`, high speeds or large `dt` values would
cause the firefly to oscillate back and forth across the target.

### Zero-length guard

```rust
if target_direction == Vec3::ZERO {
    target_pos.0 = legal_region.sample_interior(&mut rng.0);
    continue;
}
```

When the firefly has exactly reached the target, the direction vector is zero
and `normalize()` would produce `NaN`. The guard skips movement and generates a
new random target. This is a common defensive pattern in game math.

### `Cuboid::sample_interior`

Bevy's `Cuboid::sample_interior` uniformly samples a random point inside the
cuboid. For a cube of side `WORLD_SIZE`, this produces a point in
`[-WORLD_SIZE/2, +WORLD_SIZE/2]^3`.

## 2. Sinusoidal Blink (Alpha Oscillation)

Each firefly's transparency pulses using a sine wave:

```rust
let fraction = blink.timer.elapsed_secs() * blink.speed;
let alpha_multiplier =
    (f32::sin(fraction * std::f32::consts::PI * 2.0) * 0.25 + 0.75).clamp(0.5, 1.0);
```

### The formula

```
raw = sin(t * speed * 2PI)          // range: [-1, 1]
scaled = raw * 0.25 + 0.75          // range: [0.5, 1.0]
clamped = clamp(scaled, 0.5, 1.0)   // ensures no negative alpha
```

Breaking this down:
1. `sin(...)` oscillates in `[-1, 1]`
2. Multiplying by `0.25` compresses to `[-0.25, 0.25]`
3. Adding `0.75` shifts to `[0.5, 1.0]`
4. The `clamp` is technically redundant (the range already fits) but serves as a
   safety net against floating-point edge cases.

The result: the firefly never becomes invisible (minimum alpha = 0.5) and
smoothly pulses to full brightness (alpha = 1.0). The `blink.speed` per
firefly ensures desynchronized blinking across the swarm.

### Why `f32::sin(x)` instead of `x.sin()`?

Both are equivalent. The code uses the free-function form `f32::sin(fraction * ...)`
here -- this is a style choice. Elsewhere in the codebase the method syntax
`x.sin()` is used. Both compile to the same instructions.

## 3. Material Alpha Mutation

The blink system modifies the material's alpha channel in-place:

```rust
if let Color::Srgba(Srgba { red, green, blue, .. }) = material.base_color {
    material.base_color = Color::srgba(red, green, blue, blink.max_alpha * alpha_multiplier);
}
```

### Rust idiom: pattern matching on Color

Bevy's `Color` is an enum with variants for different color spaces (`Srgba`,
`LinearRgba`, `Hsla`, etc.). The `if let` pattern destructures the `Srgba`
variant, extracting the RGB channels while discarding the old alpha with `..`.
If the color happened to be stored in a different variant, the `if let` would
silently skip -- a safe default.

The reconstructed color preserves the original RGB channels and applies the new
modulated alpha: `max_alpha * alpha_multiplier`, where `max_alpha` is the
firefly's initial random alpha (0.7-1.0) and `alpha_multiplier` oscillates in
`[0.5, 1.0]`, giving an effective alpha range of `[0.35, 1.0]`.

## 4. Handle Cloning for Material Access

```rust
let material_handle = material_component.0.clone();
if let Some(material) = materials.get_mut(&material_handle) { ... }
```

Bevy's `Handle<T>` is a reference-counted smart pointer to an asset. Cloning the
handle before passing it to `get_mut` avoids borrowing `material_component`
(which holds a shared reference to the entity's component) at the same time as
mutably borrowing the `Assets<StandardMaterial>` resource. This is a common
Bevy pattern to work around Rust's borrow checker when you need to read a
component handle and write to the asset store simultaneously.

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Clamped movement | `min(distance, speed*dt)` | Move toward target without overshoot |
| Direction normalization | `d / abs(d)` | Unit vector toward target |
| Sine blink | `sin(t*2PI) * 0.25 + 0.75` | Smooth alpha oscillation in [0.5, 1.0] |
| Alpha modulation | `max_alpha * multiplier` | Per-firefly brightness variation |
