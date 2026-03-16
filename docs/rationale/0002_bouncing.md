# Bouncing -- Math Rationale

## Overview

The bouncing demo spawns 2D shapes that move at constant speed, reflecting off
window boundaries. The core math covers polar-to-Cartesian velocity conversion,
Euler integration for movement, and axis-aligned reflection for wall bounces.

## 1. Velocity from Polar Coordinates

Each shape's initial velocity is computed from a random angle and speed:

```rust
let angle = rng.random_range(0.0..std::f32::consts::TAU);
let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
```

### The formula

Given a direction angle `theta` and scalar speed `s`:

```
v = (s * cos(theta), s * sin(theta))
```

This converts from polar `(s, theta)` to Cartesian `(vx, vy)`. The angle is
sampled uniformly from `[0, TAU)` where `TAU = 2 * PI`, giving an unbiased
random direction. Because `cos^2(theta) + sin^2(theta) = 1`, the magnitude of
the resulting vector is exactly `s`, regardless of the angle chosen.

### Rust idiom

`std::f32::consts::TAU` is Rust's standard-library constant for `2 * PI`. Using
`TAU` directly avoids the `2.0 * PI` multiplication and communicates "full
circle" semantically.

## 2. Euler Integration (Movement)

Position is updated each frame using explicit (forward) Euler integration:

```rust
transform.translation.x += velocity.0.x * time.delta_secs();
transform.translation.y += velocity.0.y * time.delta_secs();
```

### The formula

```
p(t + dt) = p(t) + v * dt
```

This is the simplest numerical integrator. For constant velocity (no
acceleration), Euler integration is exact -- there is no accumulated error. The
`time.delta_secs()` call returns `dt` as `f32`, which Bevy computes from the
real elapsed wall-clock time between frames.

### Why not `transform.translation += velocity.extend(0.0) * dt`?

The code updates `x` and `y` separately. Both approaches are mathematically
equivalent. Component-wise updates avoid constructing a temporary `Vec3`.

## 3. Axis-Aligned Reflection

When a shape reaches a boundary, its velocity component along the collision axis
is reversed:

```rust
if transform.translation.x <= -half_width {
    transform.translation.x = -half_width;
    velocity.0.x = velocity.0.x.abs();
    bounced = true;
} else if transform.translation.x >= half_width {
    transform.translation.x = half_width;
    velocity.0.x = -velocity.0.x.abs();
    bounced = true;
}
```

### The formula

For an axis-aligned boundary, reflection is trivial:

```
Hitting left wall:   vx = |vx|       (ensure positive, moving right)
Hitting right wall:  vx = -|vx|      (ensure negative, moving left)
Hitting bottom wall: vy = |vy|       (ensure positive, moving up)
Hitting top wall:    vy = -|vy|      (ensure negative, moving down)
```

This is a special case of the general reflection formula:

```
v' = v - 2(v . n)n
```

where `n` is the wall normal. For axis-aligned walls, the normal is a basis
vector (e.g., `n = (1, 0)` for the left wall), so the dot product isolates a
single component, and the formula collapses to flipping that one component's
sign.

### Why `.abs()` instead of negation?

Using `velocity.0.x.abs()` instead of `-velocity.0.x` is a defensive pattern.
If a shape moves fast enough to penetrate multiple pixels past a wall in one
frame, a simple negation might leave the velocity pointing into the wall again
(the shape already passed through). Taking the absolute value unconditionally
ensures the velocity points *away* from the wall regardless of how deep the
penetration is.

The position is also clamped to the boundary (`transform.translation.x = -half_width`)
to prevent shapes from visibly escaping.

## 4. Boundary Inset

The collision boundary is inset by `MAX_SIZE`:

```rust
let half_width = window.width() / 2.0 - MAX_SIZE;
let half_height = window.height() / 2.0 - MAX_SIZE;
```

Since shapes are rendered at scales up to `MAX_SIZE`, the center of a shape must
stay at least `MAX_SIZE` pixels from the window edge to prevent visual clipping.
This is a conservative bound (it uses the maximum possible size, not each
shape's actual size), which slightly shrinks the playfield but simplifies the
collision check.

## 5. Shape Morphing on Bounce

On collision, the shape's mesh, size, and color are randomized:

```rust
mesh.0 = random_mesh(&shape_meshes, &mut rng.0);
```

The mesh handle is swapped via direct assignment to the `Mesh2d` component. Bevy
holds the actual mesh data in `Assets<Mesh>` -- the component only stores a
`Handle<Mesh>`, so swapping is a cheap pointer-sized operation. The old mesh
remains in the asset store (shared across entities via `clone()`).

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Polar to Cartesian | `(s*cos(theta), s*sin(theta))` | Random direction with fixed speed |
| Euler integration | `p += v * dt` | Frame-rate-independent movement |
| Axis-aligned reflection | `vx = abs(vx)` or `-abs(vx)` | Bounce off walls without tunneling |
| Boundary inset | `half - max_size` | Keep shapes fully visible |
