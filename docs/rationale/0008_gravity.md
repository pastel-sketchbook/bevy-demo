# Gravity -- Math Rationale

## Overview

The gravity demo simulates Newtonian gravitational attraction between a central
sun and orbiting planets. The math covers Newton's law of universal gravitation,
circular orbital velocity derivation, tangent vectors for initial velocity, and
a softening factor to prevent singularities.

## 1. Newton's Law of Universal Gravitation

The force between two bodies is:

```
F = G * m1 * m2 / r^2
```

In code:

```rust
let direction = sun_pos - planet_pos;
let distance_sq = direction.length_squared().max(100.0);
let force_magnitude = GRAVITATIONAL_CONSTANT * sun_mass.0 * planet_mass.0 / distance_sq;
let acceleration = direction.normalize() * (force_magnitude / planet_mass.0);
velocity.0 += acceleration * time.delta_secs();
```

### Decomposition

1. **Direction**: `sun_pos - planet_pos` points from planet toward sun (attractive force)
2. **Distance squared**: `direction.length_squared()` computes `r^2` directly,
   avoiding an unnecessary `sqrt` since the law uses `r^2` in the denominator
3. **Force magnitude**: scalar `F = G * M * m / r^2`
4. **Acceleration**: Newton's second law `a = F / m`, directed toward the sun
   via `direction.normalize()`
5. **Velocity update**: Euler integration `v += a * dt`

### Simplification

Since `a = F/m = G*M*m/(r^2 * m) = G*M/r^2`, the planet mass cancels out for
acceleration. The code computes `force_magnitude / planet_mass.0` which is
`G*M/r^2` (the planet mass in the numerator and denominator cancel). This is
mathematically equivalent to computing acceleration directly from the sun's
mass only, matching the real-world observation that gravitational acceleration
is independent of the falling body's mass (equivalence principle).

## 2. Circular Orbital Velocity

Planets are initialized with the exact velocity needed for a circular orbit:

```rust
let orbital_speed = (GRAVITATIONAL_CONSTANT * SUN_MASS / distance).sqrt();
```

### Derivation

For a circular orbit, gravitational force provides the centripetal acceleration:

```
G*M*m / r^2 = m * v^2 / r
```

Solving for `v`:

```
G*M / r = v^2
v = sqrt(G*M / r)
```

This is the **vis-viva equation** specialized for circular orbits (where
semi-major axis equals radius). Planets initialized with this speed will
maintain a stable circular orbit if the integration is exact. Because Euler
integration introduces energy drift, orbits will slowly spiral inward or
outward over long simulations.

## 3. Tangent Vector for Initial Velocity

The velocity must be perpendicular to the radial direction for a circular orbit:

```rust
let angle = rng.random_range(0.0..std::f32::consts::TAU);
let position = Vec2::new(angle.cos() * distance, angle.sin() * distance);
let velocity_dir = Vec2::new(-angle.sin(), angle.cos());
let velocity = velocity_dir * orbital_speed;
```

### The math

Given a position on a circle at angle `theta`:

```
position = (r*cos(theta), r*sin(theta))
```

The tangent (perpendicular to the radius, counter-clockwise) is:

```
tangent = (-sin(theta), cos(theta))
```

This is the derivative of the position with respect to `theta`:

```
d/d(theta) (cos(theta), sin(theta)) = (-sin(theta), cos(theta))
```

The tangent vector has unit length, so multiplying by `orbital_speed` gives the
correct velocity magnitude and direction for a prograde (counter-clockwise)
circular orbit.

### Why counter-clockwise?

The tangent `(-sin, cos)` points counter-clockwise. For clockwise orbits, use
`(sin, -cos)`. The choice is arbitrary for this demo.

## 4. Distance Softening

```rust
let distance_sq = direction.length_squared().max(100.0);
```

### The problem

Newton's law has a singularity at `r = 0`: as distance approaches zero,
`F -> infinity`. In a discrete simulation, two bodies can approach very close
in one frame, producing an enormous acceleration that flings them apart at
unrealistic speed.

### The solution

Clamping `distance_sq` to a minimum of `100.0` (equivalent to `r_min = 10`
pixels) prevents the force from exceeding `G*M*m/100`. This is called
**gravitational softening** or **softening length** -- a standard technique in
N-body simulations (used in astrophysics codes like Gadget and AREPO).

The softened potential is effectively:

```
F = G*M*m / max(r^2, epsilon^2)
```

with `epsilon = 10`. Physics fidelity is sacrificed at very close range in
exchange for numerical stability.

## 5. Euler Integration (Position Update)

```rust
transform.translation.x += velocity.0.x * time.delta_secs();
transform.translation.y += velocity.0.y * time.delta_secs();
```

### Symplectic Euler (semi-implicit)

The velocity is updated *before* the position update (in a chained system
ordering). This makes the integration **symplectic Euler** (also called
semi-implicit Euler), which has better energy conservation than explicit
(forward) Euler:

```
v(t+dt) = v(t) + a(t) * dt          // velocity updated first
p(t+dt) = p(t) + v(t+dt) * dt       // position uses NEW velocity
```

Symplectic Euler is time-reversible and conserves a shadow Hamiltonian, making
orbits more stable over long simulations. Forward Euler, by contrast, would use
the old velocity for the position update and tends to spiral outward.

The `.chain()` modifier on the system tuple ensures `apply_gravity` runs before
`update_positions`:

```rust
(apply_gravity, update_positions, handle_input).chain()
```

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Gravitational force | `F = G*M*m / r^2` | Attractive force between bodies |
| Circular orbit speed | `v = sqrt(G*M / r)` | Stable circular orbit initialization |
| Tangent vector | `(-sin(theta), cos(theta))` | Perpendicular to radial for orbit direction |
| Softening | `max(r^2, epsilon^2)` | Prevent singularity at close approach |
| Symplectic Euler | `v += a*dt` then `p += v*dt` | Better energy conservation than forward Euler |
