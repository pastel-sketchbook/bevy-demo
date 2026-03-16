# Flocking -- Math Rationale

## Overview

The flocking demo implements Craig Reynolds' Boids algorithm (1987). Each boid
(bird-oid) follows three simple local rules -- separation, alignment, and
cohesion -- that together produce emergent flocking behavior without any global
coordination. The math covers neighbor averaging, inverse-distance repulsion,
velocity clamping, and heading-to-rotation conversion.

## 1. The Three Boid Rules

For each boid, the system examines all other boids within `PERCEPTION_RADIUS`:

```rust
let diff = pos - *other_pos;
let dist = diff.length();

if dist < PERCEPTION_RADIUS && dist > 0.0 {
    separation += diff / dist;
    alignment += *other_vel;
    cohesion += *other_pos;
    count += 1;
}
```

### 1a. Separation

```
separation = sum(diff / |diff|)  for each neighbor
```

`diff / |diff|` is a unit vector pointing *away* from the neighbor. Closer
neighbors contribute equally to farther ones after normalization. The `diff / dist`
in the code is equivalent to `normalize(diff)` -- it divides the displacement
vector by its own length to produce a unit direction.

After accumulation:

```rust
separation /= count_f;
```

This averages the repulsion directions. The result points away from the local
center of mass of neighbors, weighted by direction but not by distance.

Note: a common variant weights by `1/dist^2` (stronger repulsion from closer
neighbors). This implementation uses uniform weighting for simplicity.

### 1b. Alignment

```
alignment = avg(neighbor_velocities) - self_velocity
```

In code:

```rust
alignment = (alignment / count_f - vel).clamp_length_max(BOID_SPEED);
```

First, `alignment / count_f` computes the average velocity of neighbors. Then
subtracting `vel` (the boid's own velocity) gives a correction vector pointing
toward the flock's average heading. `clamp_length_max` limits the magnitude to
prevent a single alignment step from dominating.

### 1c. Cohesion

```
cohesion = avg(neighbor_positions) - self_position
```

In code:

```rust
cohesion = (cohesion / count_f - pos).clamp_length_max(BOID_SPEED);
```

The average neighbor position is the local center of mass. Subtracting `pos`
gives a vector pointing toward that center. This is the "stay together" force.

### Combined steering

```rust
velocity.0 += separation * SEPARATION_WEIGHT
    + alignment * ALIGNMENT_WEIGHT
    + cohesion * COHESION_WEIGHT;
velocity.0 = velocity.0.clamp_length_max(MAX_SPEED);
```

Each rule's contribution is weighted and summed. The final velocity is clamped to
`MAX_SPEED` to prevent unbounded acceleration. This is a weighted sum of steering
forces -- the same pattern used in most Boids implementations.

The weights `(1.5, 1.0, 1.0)` give separation slightly more influence than the
other two rules, preventing boids from collapsing into a single point.

## 2. Heading-to-Rotation Conversion

Boids are rendered as triangles pointing in their direction of motion:

```rust
if velocity.0.length_squared() > 0.0 {
    let angle = velocity.0.y.atan2(velocity.0.x) - std::f32::consts::FRAC_PI_2;
    transform.rotation = Quat::from_rotation_z(angle);
}
```

### The formula

`atan2(vy, vx)` returns the angle of the velocity vector from the +X axis in
radians, range `(-PI, PI]`. Subtracting `PI/2` rotates the reference direction
from +X to +Y.

Why? The triangle mesh is defined with its tip at `(0, 10)` -- pointing up
along +Y in local space. The rotation must map this default "up" direction to
the actual velocity direction. Since the triangle points up (along +Y = angle
`PI/2` in standard math), we subtract `PI/2` to compensate.

General formula:

```
rotation = atan2(vy, vx) - mesh_default_angle
```

### `length_squared() > 0.0`

Checking `length_squared` instead of `length` avoids an unnecessary `sqrt`. If
velocity is zero, `atan2(0, 0)` is undefined, so the guard skips the rotation
update entirely, keeping the previous heading.

## 3. Toroidal Wrapping

```rust
if pos.x > half_w { pos.x = -half_w; }
else if pos.x < -half_w { pos.x = half_w; }
```

Boids that exit one side reappear on the opposite side. This simulates an
infinite plane by wrapping coordinates modularly:

```
x' = ((x + W/2) mod W) - W/2
```

The `if/else` chain is equivalent to the modular arithmetic but avoids floating
point modulo, which handles negative values inconsistently across platforms.

## 4. Initial Velocity via `Vec2::from_angle`

```rust
let velocity = Vec2::from_angle(angle) * BOID_SPEED;
```

Bevy's `Vec2::from_angle(theta)` returns `(cos(theta), sin(theta))` -- a unit
vector at the given angle. This is equivalent to the polar-to-Cartesian
conversion in the bouncing demo but uses Bevy's built-in convenience method.

## 5. Snapshot Pattern (Collecting Before Mutating)

```rust
let boids: Vec<(Entity, Vec2, Vec2)> = query
    .iter()
    .map(|(e, t, v)| (e, t.translation.truncate(), v.0))
    .collect();
```

The positions and velocities of all boids are collected into a `Vec` before
the mutation loop. This is necessary because Rust's borrow checker prevents
iterating a query mutably while also reading other entities from the same query.
The snapshot captures the state at the beginning of the frame, ensuring all boids
compute their steering from the same consistent world state (synchronous update).

### `truncate()`

`Vec3::truncate()` drops the Z component, returning a `Vec2` of `(x, y)`. In
this 2D demo, all Z values are 0, but the transforms are still `Vec3` because
Bevy is a 3D engine at its core.

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Separation | `avg(normalize(self - neighbor))` | Avoid crowding nearby boids |
| Alignment | `avg(neighbor_vel) - self_vel` | Steer toward flock's average heading |
| Cohesion | `avg(neighbor_pos) - self_pos` | Steer toward flock's center of mass |
| Velocity clamping | `v.clamp_length_max(max)` | Prevent unbounded acceleration |
| Heading angle | `atan2(vy, vx) - PI/2` | Orient triangle mesh along velocity |
| Toroidal wrap | `if x > half { x = -half }` | Infinite plane via boundary wrapping |
