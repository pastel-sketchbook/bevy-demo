# Particles -- Math Rationale

## Overview

The particles demo spawns 2D particles from the origin at random angles with
random speeds, colors, sizes, and lifetimes. Particles move in straight lines
and fade out as their lifetime expires. The math covers radial emission via
polar-to-Cartesian conversion, lifetime-based alpha decay, and HSL color space
for pastel generation.

## 1. Radial Emission (Polar-to-Cartesian Velocity)

```rust
let angle = rng.0.random_range(0.0..std::f32::consts::TAU);
let speed = rng.0.random_range(PARTICLE_SPEED * 0.5..PARTICLE_SPEED * 1.5);
let velocity = Vec2::new(angle.cos(), angle.sin()) * speed;
```

### The math

A random direction in 2D is a point on the unit circle at angle `theta`:

```
direction = (cos(theta), sin(theta))
velocity  = direction * speed
```

This is the polar-to-Cartesian conversion. By sampling `theta` uniformly from
`[0, TAU)`, particles are emitted in all directions with equal probability.
The speed is then sampled independently from a uniform range, making this a
**uniform angular, variable speed** emission pattern.

### Why `TAU`?

`TAU = 2*PI` represents a full revolution. Bevy re-exports `std::f32::consts::TAU`
for this purpose. Using `TAU` instead of `2.0 * PI` is more readable and avoids
a multiplication.

### Velocity distribution

The speed range is `[0.5 * PARTICLE_SPEED, 1.5 * PARTICLE_SPEED]`, centered on
`PARTICLE_SPEED = 300.0`:

```
speed ∈ [150, 450] pixels/second
```

This uniform range creates a spreading cloud where faster particles form the
outer edge and slower ones linger near the center.

## 2. Euler Integration (Position Update)

```rust
transform.translation.x += velocity.0.x * dt;
transform.translation.y += velocity.0.y * dt;
```

### The math

With no forces acting on particles (no gravity, no drag), velocity is constant:

```
p(t + dt) = p(t) + v * dt
```

This is forward Euler integration applied to the trivial case of zero
acceleration. Since `v` is constant, Euler is exact here -- there is no
integration error.

### Why not `Transform::translate`?

Direct field access (`translation.x += ...`) avoids constructing an intermediate
`Vec3`. This is a micro-optimization but also makes the intent explicit: we're
modifying specific axes independently.

## 3. Lifetime and Alpha Fade-Out

```rust
lifetime.remaining -= dt;
let alpha = (lifetime.remaining / lifetime.total).clamp(0.0, 1.0);
```

### The Lifetime component

```rust
struct Lifetime {
    remaining: f32,
    total: f32,
}
```

Storing both `remaining` and `total` allows computing the normalized age ratio
without additional bookkeeping. The ratio `remaining / total` linearly decays
from 1.0 (just born) to 0.0 (about to die).

### Linear alpha decay

```
alpha = clamp(remaining / total, 0, 1)
```

This is a **linear fade-out**: alpha decreases at a constant rate. The particle
starts fully opaque and reaches full transparency exactly at death. The `clamp`
guards against floating-point overshoot (remaining going slightly negative
between the update and despawn systems).

### Why linear?

Linear fade is the simplest choice. Alternatives include:
- **Quadratic**: `alpha = (remaining / total)^2` -- fades slowly then drops fast
- **Ease-out**: `alpha = 1 - (1 - t)^2` -- fades fast then tapers
- **Exponential**: `alpha = e^(-k * age)` -- never truly reaches zero

Linear is chosen for simplicity and predictability.

## 4. HSL Color Space for Material Mutation

```rust
let hue = rng.0.random_range(0.0..360.0);
let color = Color::hsl(hue, 0.55, 0.78);
```

### Why HSL?

HSL (Hue, Saturation, Lightness) separates chromaticity from brightness.
Randomizing only the hue while fixing saturation and lightness produces colors
that are perceptually uniform in vibrancy and brightness -- all particles look
equally "pastel" regardless of hue.

Compare to randomizing RGB: `Color::srgb(rand, rand, rand)` produces wildly
varying brightness (pure red vs near-black vs near-white). HSL avoids this.

### The chosen values

- **Saturation = 0.55**: moderate saturation, muted but not gray
- **Lightness = 0.78**: bright pastels, lighter than 50% midtone

### Material mutation for alpha

```rust
if let Some(material) = materials.get_mut(&material_handle.0)
    && let Color::Hsla(Hsla { hue, saturation, lightness, .. }) = material.color
{
    material.color = Color::hsla(hue, saturation, lightness, alpha);
}
```

This destructures the existing HSL color to extract its channels, then
reconstructs it with the new alpha. The pattern preserves the original hue,
saturation, and lightness while only changing transparency.

### Rust idiom: chained `let` in `if let`

```rust
if let Some(material) = materials.get_mut(&material_handle.0)
    && let Color::Hsla(Hsla { hue, saturation, lightness, .. }) = material.color
```

This is a **let-chain** (stabilized in Rust 1.87 nightly, requires edition 2024).
It combines two fallible patterns into a single `if` condition. If either
pattern fails (material not found, or color isn't HSL), the entire block is
skipped. This replaces nested `if let` blocks with a flat, readable chain.

## 5. Spawn Rate via Timer

```rust
.insert_resource(SpawnTimer(Timer::from_seconds(
    1.0 / SPAWN_RATE,
    TimerMode::Repeating,
)))
```

### The math

`SPAWN_RATE = 50.0` particles per second. The timer period is:

```
period = 1 / rate = 1/50 = 0.02 seconds = 20ms
```

### Catching up with `times_finished_this_tick()`

```rust
timer.0.tick(time.delta());
for _ in 0..timer.0.times_finished_this_tick() {
    // spawn one particle
}
```

If the frame takes longer than the timer period (e.g., a 60ms lag spike), the
timer may fire multiple times. `times_finished_this_tick()` returns how many
periods elapsed during this tick, so the loop spawns the correct number of
particles to maintain the target rate regardless of frame time.

At 60 FPS with a 20ms timer: `16.67ms / 20ms ≈ 0.83`, so most frames fire 0 or
1 times, with occasional frames firing 1 time. The timer accumulates fractional
time across frames, maintaining the average rate precisely.

## 6. Despawn by Lifetime

```rust
fn despawn_particles(mut commands: Commands, query: Query<(Entity, &Lifetime), With<Particle>>) {
    for (entity, lifetime) in query.iter() {
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

### Why a separate system?

Decoupling despawn from update follows Bevy's ECS philosophy: each system has a
single responsibility. The update system modifies transforms and materials; the
despawn system removes dead entities. This makes both systems simpler and allows
Bevy's scheduler to potentially parallelize them with other non-conflicting
systems.

### Entity cleanup

`commands.entity(entity).despawn()` removes the entity and all its components.
Without this, dead particles would accumulate, consuming memory and wasting
iteration time in queries. The `With<Particle>` filter ensures only particles
are checked, not other entities like the camera.

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Radial emission | `v = (cos(theta), sin(theta)) * speed` | Uniform directional particle spawning |
| Euler integration | `p += v * dt` | Constant-velocity motion (exact for zero acceleration) |
| Linear alpha decay | `alpha = remaining / total` | Smooth fade-out over particle lifetime |
| HSL color | `hsl(rand_hue, 0.55, 0.78)` | Perceptually uniform pastel colors |
| Spawn period | `period = 1 / rate` | Convert spawn rate to timer interval |
| Catch-up spawning | `times_finished_this_tick()` | Frame-rate-independent spawn count |
