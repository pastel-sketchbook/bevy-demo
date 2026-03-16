# Bloom -- Math Rationale

## Overview

The bloom demo renders emissive 3D shapes that orbit a central point while pulsing
their glow intensity. Three distinct mathematical models drive the visuals:
sinusoidal pulsing, parametric circular orbits, and additive harmonic bobbing.

## 1. Sinusoidal Pulse (Emissive Intensity)

Each shape oscillates its emissive brightness over time using a phase-shifted
sine wave.

### The formula

```
pulse = (sin(t * PULSE_SPEED + phase) + 1.0) / 2.0
intensity = EMISSIVE_BASE + pulse * EMISSIVE_AMPLITUDE
```

`sin` returns values in `[-1, 1]`. Adding `1.0` and dividing by `2.0` remaps
the output to `[0, 1]` -- a standard normalization trick for sine waves when you
need a non-negative oscillation factor.

The `phase` offset per shape ensures they do not all pulse in lockstep. Two
shapes with phases `0.0` and `PI` will glow in perfect opposition (one peaks
while the other troughs).

### Rust idiom

```rust
let pulse = ((t * PULSE_SPEED + shape.phase).sin() + 1.0) / 2.0;
```

Method chaining on `f32` -- `.sin()` is called directly on the expression rather
than using `f32::sin(x)`. This is idiomatic Rust: the receiver syntax reads
left-to-right matching the data flow.

The final emissive color is produced by scalar multiplication on `LinearRgba`:

```rust
material.emissive = shape.base_color * intensity;
```

Bevy's `LinearRgba` implements `Mul<f32>`, scaling each channel uniformly.
Because bloom post-processing responds to emissive values > 1.0, an intensity
of `EMISSIVE_BASE(5.0) + EMISSIVE_AMPLITUDE(8.0) = 13.0` produces the
characteristic HDR glow.

## 2. Parametric Circular Orbit

Each shape travels along a circle in the XZ plane (horizontal orbit in 3D).

### The formula

```
angle = phase + t * speed
x = radius * cos(angle)
z = radius * sin(angle)
```

This is the standard parametric form of a circle:

```
P(theta) = (r * cos(theta), r * sin(theta))
```

where `theta` increases linearly with time. The `phase` offset staggers the
starting positions so shapes are distributed around their orbital rings at t=0.

At setup time, the initial position is computed identically:

```rust
let x = orbit_radius * phase.cos();
let z = orbit_radius * phase.sin();
```

This ensures continuity -- the shape starts exactly where the orbit system will
place it on the first frame.

### Why XZ and not XY?

In Bevy's right-handed coordinate system, Y is up. Orbiting in the XZ plane
produces a horizontal circle visible from the camera positioned above
(`CAMERA_HEIGHT = 5.0`).

## 3. Additive Harmonic Bobbing

A secondary vertical oscillation is layered on top of the fixed orbit height:

```rust
transform.translation.y = orbit.height + (t * 0.8 + orbit.phase).sin() * 0.3;
```

### Superposition of motions

The Y position is `orbit.height + A * sin(omega * t + phi)` where:
- `A = 0.3` (bobbing amplitude in world units)
- `omega = 0.8` (frequency, deliberately slower than `PULSE_SPEED`)
- `phi = orbit.phase` (reuses the same phase offset)

Using a different frequency (`0.8`) from the pulse speed (`2.0`) ensures the bob
and the glow pulse are not synchronized. This creates visual richness through
*incommensurable frequencies* -- the combined motion never exactly repeats on a
short timescale.

## 4. Emissive Scaling (LinearRgba Arithmetic)

At setup, the initial emissive is:

```rust
let emissive = color * EMISSIVE_BASE;
```

`LinearRgba * f32` multiplies R, G, B channels by the scalar. This works
correctly only in linear color space (not sRGB) because physical light intensity
addition is linear. Bevy's `LinearRgba` type enforces this at the type level --
you cannot accidentally multiply sRGB values.

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Sine normalization | `(sin(x) + 1) / 2` | Map `[-1,1]` to `[0,1]` |
| Parametric circle | `(r*cos(t), r*sin(t))` | Circular orbit path |
| Phase offset | `sin(t + phi)` | Desynchronize per-entity animation |
| Superposition | `base + A*sin(w*t + phi)` | Layer independent oscillations |
| Linear color scaling | `LinearRgba * scalar` | HDR emissive intensity for bloom |
