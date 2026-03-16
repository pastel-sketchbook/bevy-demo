# Clock -- Math Rationale

## Overview

The clock demo renders an analog clock face using Bevy's Gizmos (immediate-mode
line drawing). The math covers coordinate system conversion between clock
conventions and standard trigonometry, smooth cascading hand movement, and
thick-line approximation via parallel offsets.

## 1. Clock Angle to Direction Vector

Standard math conventions place angle 0 pointing right (+X), increasing
counter-clockwise. Clocks place 12 o'clock at the top (angle 0 pointing up),
increasing clockwise. The conversion bridges these two systems.

### The formula

```rust
fn clock_direction(angle: f32) -> Vec2 {
    let math_angle = FRAC_PI_2 - angle;
    Vec2::new(math_angle.cos(), math_angle.sin())
}
```

Derivation:
- Clock 0 (12 o'clock) = up = standard `PI/2`
- Clock increases clockwise = standard angle *decreases*
- Therefore: `math_angle = PI/2 - clock_angle`

Verification:
| Clock position | clock_angle | math_angle | cos, sin | Direction |
|---|---|---|---|---|
| 12 o'clock | 0 | PI/2 | (0, 1) | Up |
| 3 o'clock | PI/2 | 0 | (1, 0) | Right |
| 6 o'clock | PI | -PI/2 | (0, -1) | Down |
| 9 o'clock | 3PI/2 | -PI | (-1, 0) | Left |

### Rust idiom

`FRAC_PI_2` is imported from `std::f32::consts`. Rust provides fractional-PI
constants (`FRAC_PI_2`, `FRAC_PI_3`, `FRAC_PI_4`, etc.) to avoid repeated
division and improve readability.

## 2. Smooth Cascading Hand Angles

The clock avoids discrete jumps by cascading fractional components upward:

```rust
let smooth_minutes = minutes + seconds / 60.0;
let smooth_hours = hours + smooth_minutes / 60.0;
```

### The formula

```
smooth_minutes = m + s/60
smooth_hours   = h + smooth_minutes/60 = h + m/60 + s/3600
```

The second hand angle uses raw seconds (it ticks continuously because
`subsec_nanosecond` provides sub-second precision):

```rust
let seconds = now.second() as f64 + now.subsec_nanosecond() as f64 / 1_000_000_000.0;
```

Each hand's angle is then a fraction of its full cycle converted to radians:

```rust
let second_angle = (seconds / 60.0) as f32 * TAU;
let minute_angle = (smooth_minutes / 60.0) as f32 * TAU;
let hour_angle   = (smooth_hours / 12.0) as f32 * TAU;
```

The general form is:

```
angle = (value / period) * TAU
```

where `period` is 60 for seconds/minutes and 12 for hours. This maps the value
linearly onto the full circle.

### Why `f64` intermediates?

The raw time values are computed in `f64` to avoid precision loss in the
nanosecond-to-seconds conversion (`1e9` exceeds `f32`'s ~7-digit mantissa). The
cast to `f32` happens only after division normalizes the value to `[0, 1)`.

## 3. Thick Line Drawing via Parallel Offsets

Gizmos only draws 1-pixel lines. Thick lines are approximated by drawing
multiple parallel copies offset along the perpendicular direction:

```rust
fn draw_thick_line(gizmos: &mut Gizmos, start: Vec2, end: Vec2, width: f32, color: Color) {
    let dir = (end - start).normalize_or(Vec2::Y);
    let perp = Vec2::new(-dir.y, dir.x);
    let steps = (width / 1.5).ceil() as i32;
    let half = width / 2.0;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let offset = perp * (t * width - half);
        gizmos.line_2d(start + offset, end + offset, color);
    }
}
```

### Perpendicular vector

Given a 2D direction `(dx, dy)`, the perpendicular is `(-dy, dx)`. This is a
90-degree counter-clockwise rotation, derived from the rotation matrix:

```
R(90) = | 0  -1 |    applied to (dx, dy) gives (-dy, dx)
        | 1   0 |
```

### Offset distribution

The parameter `t` sweeps from `0` to `1` across `steps+1` samples. The offset
formula `t * width - half` maps `t=0` to `-half` and `t=1` to `+half`, centering
the band of lines around the original start-end segment.

The step size `width / 1.5` ensures lines overlap slightly (each line covers ~1
pixel, spaced ~1.5 pixels apart), producing a visually solid band.

### `normalize_or(Vec2::Y)`

This is a Bevy method that returns a normalized vector, falling back to
`Vec2::Y` if the input is zero-length. This prevents a division-by-zero when
`start == end`.

## 4. Clock Face Ring

The clock face border is a thick ring drawn as concentric circles:

```rust
let steps = (FACE_RING_WIDTH / 1.0).ceil() as i32;
for i in 0..=steps {
    let t = i as f32 / steps as f32;
    let r = CLOCK_RADIUS - t * FACE_RING_WIDTH;
    gizmos.circle_2d(center, r, FACE_COLOR);
}
```

This sweeps the radius from `CLOCK_RADIUS` down to `CLOCK_RADIUS - FACE_RING_WIDTH`,
drawing one circle per pixel of width. The same interpolation pattern `t in [0, 1]`
appears here as in the thick-line function.

## 5. Tick Mark Placement

60 tick marks are evenly distributed around the circle:

```rust
let angle = (i as f32 / 60.0) * TAU;
let dir = clock_direction(angle);
let outer = center + dir * CLOCK_RADIUS;
let inner = center + dir * (CLOCK_RADIUS - length);
```

Each tick is a radial segment from `inner` to `outer`. Major ticks (at multiples
of 5, i.e., the hour positions) use greater length and width. The modulo check
`i % 5 == 0` cleanly separates the two cases.

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Clock-to-math angle | `PI/2 - clock_angle` | Convert clockwise-from-12 to CCW-from-right |
| Smooth cascade | `m + s/60`, `h + m'/60` | Eliminate discrete jumps in hand positions |
| Fraction to radians | `(value / period) * TAU` | Map time unit to angular position |
| 2D perpendicular | `(-dy, dx)` | Offset direction for thick lines |
| Interpolated offset | `t * width - half` | Center a band of parallel lines |
