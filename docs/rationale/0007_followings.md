# Followings -- Math Rationale

## Overview

The followings demo shows one sphere smoothly chasing another. The target sphere
moves linearly toward random waypoints, while the follower uses exponential
decay interpolation (Bevy's `smooth_nudge`) to approach the target. The math
covers clamped linear movement and the theory behind exponential smoothing.

## 1. Clamped Linear Movement (Target Sphere)

```rust
match Dir3::new(target_pos.0 - target.translation) {
    Ok(dir) => {
        let abs_delta = (target_pos.0 - target.translation).norm();
        let magnitude = f32::min(abs_delta, delta_time * target_speed.0);
        target.translation += dir * magnitude;
    }
    Err(_) => {
        target_pos.0 = TargetPosition(legal_region.sample_interior(&mut rng.0));
    }
}
```

### The formula

```
d = target - current
step = min(|d|, speed * dt)
position += normalize(d) * step
```

This is identical to the firefly's movement pattern. `Dir3::new(v)` attempts to
normalize a `Vec3` into a `Dir3` (a unit-length direction type). It returns
`Err` when the input is too short to normalize (near zero), which triggers
selection of a new random waypoint.

### Rust idiom: `Dir3` as a validated type

`Dir3` is a Bevy newtype that guarantees unit length at construction time. Using
`Dir3::new` instead of `Vec3::normalize()` makes the directionality explicit in
the type system and gracefully handles the zero-vector case through `Result`
instead of silently producing `NaN`.

### `NormedVectorSpace::norm()`

The code uses `.norm()` instead of `.length()`. `NormedVectorSpace` is a Bevy
trait that provides `norm()` as a general abstraction for the Euclidean norm.
For `Vec3`, `norm()` and `length()` are identical -- both compute
`sqrt(x^2 + y^2 + z^2)`. The trait import signals that this code participates
in the broader `NormedVectorSpace` ecosystem (which also provides `smooth_nudge`).

## 2. Exponential Decay Smoothing (Follower Sphere)

```rust
following
    .translation
    .smooth_nudge(&target.translation, decay_rate, delta_time);
```

### The math behind `smooth_nudge`

`smooth_nudge` implements frame-rate-independent exponential decay interpolation:

```
value = lerp(value, target, 1 - e^(-decay_rate * dt))
```

Expanded:

```
value += (target - value) * (1 - e^(-lambda * dt))
```

where `lambda` is the decay rate.

### Why exponential decay?

The naive approach `value += (target - value) * factor * dt` is frame-rate
dependent -- at higher frame rates, the object moves faster because it
recomputes the shrinking distance more often, and each step is from a closer
starting point.

The exponential form `1 - e^(-lambda * dt)` is the exact solution to the
first-order ODE:

```
dx/dt = -lambda * (x - target)
```

This ODE says "move toward the target at a rate proportional to the remaining
distance." Its solution over a time step `dt` is:

```
x(t + dt) = target + (x(t) - target) * e^(-lambda * dt)
```

Rearranging to update form:

```
x += (target - x) * (1 - e^(-lambda * dt))
```

The key property: the result depends only on the *total time elapsed*, not on
how many frames were computed. Whether you compute 1 step of 16ms or 16 steps of
1ms, the final position converges to the same value.

### Decay rate semantics

`DecayRate(2.0)` means the follower closes ~86% of the gap per second:

```
fraction_remaining = e^(-2.0 * 1.0) = e^(-2) ≈ 0.135
fraction_closed = 1 - 0.135 ≈ 0.865
```

Higher decay rates produce snappier following. The follower asymptotically
approaches but never exactly reaches the target (Zeno-like behavior), which
produces the characteristic smooth easing motion.

### `NormedVectorSpace` trait

`smooth_nudge` is available because `Vec3` implements the `NormedVectorSpace`
trait. This trait provides:
- `norm()` -- Euclidean length
- `smooth_nudge(&target, decay_rate, dt)` -- frame-rate-independent smoothing

The trait-based design means the same smoothing algorithm works for `Vec2`,
`Vec3`, `f32`, or any custom type implementing the trait.

## 3. `Single<>` Query Pattern

```rust
fn move_follower(
    mut following: Single<&mut Transform, With<FollowingSphere>>,
    target: Single<&Transform, (With<TargetSphere>, Without<FollowingSphere>)>,
```

`Single<>` is a Bevy query parameter that expects exactly one matching entity.
It automatically unwraps, eliminating the need for `.single()` / `.single_mut()`
calls. This is a Bevy 0.15+ ergonomic improvement.

The `Without<FollowingSphere>` constraint on the target query resolves Bevy's
query conflict detection -- both queries access `Transform`, but the `Without`
filter guarantees they can never match the same entity, proving to the ECS that
the mutable and immutable borrows are disjoint.

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Clamped linear move | `min(distance, speed*dt)` | Move to waypoint without overshoot |
| Dir3 construction | `Dir3::new(v) -> Result` | Validated unit direction with zero guard |
| Exponential decay | `x += (target - x) * (1 - e^(-lambda*dt))` | Frame-rate-independent smooth following |
| Decay rate meaning | `e^(-lambda)` remaining per second | Controls following responsiveness |
