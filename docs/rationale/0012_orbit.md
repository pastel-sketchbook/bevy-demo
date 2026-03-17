# Orbit -- Math Rationale

## Overview

The orbit demo implements a 3D camera controller that orbits around the origin.
The camera position is computed from spherical coordinates (yaw, pitch, distance)
converted to Cartesian every frame. Mouse drag adjusts yaw/pitch, scroll adjusts
distance, and several auto-rotating 3D primitives demonstrate axis-angle rotation.

## 1. Spherical-to-Cartesian Conversion

The camera position is computed from spherical coordinates each frame:

```rust
let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
let y = orbit.distance * orbit.pitch.sin();
let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();
```

### Standard spherical coordinates

In a standard spherical coordinate system with the Y-axis as "up":

```
x = r * cos(phi) * sin(theta)
y = r * sin(phi)
z = r * cos(phi) * cos(theta)
```

where:
- `r` = `distance` (radial distance from origin)
- `theta` = `yaw` (azimuthal angle around the Y-axis)
- `phi` = `pitch` (elevation angle from the XZ-plane)

### Why this convention?

This is the **elevation-angle** convention (pitch measured from the horizontal
plane), not the **polar-angle** convention (measured from the Y-axis). The
elevation convention is more intuitive for cameras: pitch = 0 looks horizontally,
positive pitch looks upward. The relationship is:

```
polar_angle = PI/2 - elevation
```

With `cos(elevation)` and `sin(elevation)` instead of `sin(polar)` and
`cos(polar)`, the formulas are equivalent but more natural for interactive
camera controls.

### Geometric intuition

At pitch = 0 (horizontal view):
- `cos(0) = 1`, so x-z movement is at full radius
- `sin(0) = 0`, so y = 0 (camera at horizon level)

At pitch = PI/4 (45 degrees up):
- `cos(PI/4) = 0.707`, so x-z radius shrinks
- `sin(PI/4) = 0.707`, so camera rises

The `cos(pitch)` factor on x and z encodes the fact that as the camera tilts
upward, its horizontal projection onto the XZ-plane shrinks.

## 2. Yaw and Pitch from Mouse Input

```rust
orbit.yaw -= mouse_motion.delta.x * ORBIT_SENSITIVITY;
orbit.pitch -= mouse_motion.delta.y * ORBIT_SENSITIVITY;
```

### Sign conventions

- **Yaw**: Subtracting `delta.x` makes dragging the mouse right rotate the
  camera to the left (the scene appears to rotate right). This matches the
  convention that positive yaw is counter-clockwise when viewed from above
  (standard right-hand rule around Y).

- **Pitch**: Subtracting `delta.y` makes dragging the mouse down tilt the camera
  upward. Screen Y increases downward, but elevation angles increase upward, so
  the negation is necessary.

### Sensitivity scaling

`ORBIT_SENSITIVITY = 0.005` converts pixel deltas (which can be hundreds of
pixels per frame) to radians. At 1920px wide, a full-width drag gives:

```
1920 * 0.005 = 9.6 radians (~1.5 full rotations)
```

This is intentionally generous so that short drags produce meaningful rotation.

### Accumulated vs per-frame

The code uses `AccumulatedMouseMotion`, which sums all mouse events between
frames. This ensures no input is lost regardless of frame rate -- a fast flick
that generates 10 OS events per frame still accumulates correctly.

## 3. Pitch Clamping

```rust
orbit.pitch = orbit.pitch.clamp(-1.4, 1.4);
```

### Why clamp?

At pitch = ±PI/2 (±1.5708), the camera is directly above or below the origin.
The `looking_at` function uses the Y-axis as the up-vector:

```rust
*transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
```

When the camera's forward direction is parallel to the up-vector (pitch = ±PI/2),
the cross product used internally to build the view matrix degenerates (produces
a zero vector), causing the camera to flip or produce undefined behavior. This
is the **gimbal lock** problem.

### Why 1.4 specifically?

`1.4 radians ≈ 80.2 degrees`, which is close to but safely below `PI/2 ≈ 1.5708`.
The gap of ~0.17 radians (~10 degrees) provides a comfortable buffer before the
singularity. Many orbit camera implementations use values between 1.2 and 1.5.

### Floor clamp on Y

```rust
transform.translation = Vec3::new(x, y.max(0.5), z);
```

This prevents the camera from going below `y = 0.5`, ensuring it never dips
below the ground plane (at `y = -1.0`). Even at maximum negative pitch (-1.4),
`distance * sin(-1.4)` could be quite negative at large zoom distances. The
`max(0.5)` overrides the spherical math to enforce a physical floor.

## 4. Scroll Zoom

```rust
orbit.distance -= mouse_scroll.delta.y * ZOOM_SENSITIVITY;
orbit.distance = orbit.distance.clamp(MIN_DISTANCE, MAX_DISTANCE);
```

### Linear zoom

The zoom is linear: each scroll tick changes distance by a fixed amount
(`ZOOM_SENSITIVITY = 0.5` units). This is the simplest model. An alternative
is multiplicative zoom (`distance *= 1 + scroll * factor`), which feels more
natural at extreme distances but is unnecessary for the small range here.

### Distance bounds

```
MIN_DISTANCE = 3.0    -- prevents camera from entering/passing through objects
MAX_DISTANCE = 30.0   -- keeps the scene visible
```

`clamp` enforces both bounds in a single call, which is idiomatic Rust and
avoids separate `min`/`max` calls that could be applied in the wrong order.

## 5. Axis-Angle Auto-Rotation

```rust
let angle = rotate.speed * time.delta_secs();
transform.rotate(Quat::from_axis_angle(rotate.axis, angle));
```

### Axis-angle representation

`Quat::from_axis_angle(axis, angle)` constructs a unit quaternion representing
a rotation of `angle` radians around `axis`:

```
q = cos(angle/2) + sin(angle/2) * (xi + yj + zk)
```

where `(x, y, z)` is the unit axis. Quaternions avoid gimbal lock and compose
efficiently via multiplication.

### `transform.rotate()`

This method multiplies the existing transform's rotation by the new quaternion:

```
rotation = q * rotation
```

Each frame adds an incremental rotation, producing smooth continuous spinning.
The rotation accumulates multiplicatively (not additively), which is correct for
rotations in 3D (rotations don't commute, and the group structure is
multiplicative).

### Custom rotation axes

Several shapes use non-trivial axes:

```rust
Vec3::new(1.0, 0.5, 0.0).normalize()   // tilted axis
Vec3::new(0.5, 1.0, 0.3).normalize()   // diagonal wobble
Vec3::new(0.3, 1.0, 0.7).normalize()   // another diagonal
```

The `.normalize()` call is essential: `Quat::from_axis_angle` requires a unit
vector. An unnormalized axis would produce an unnormalized quaternion, leading
to scale drift and visual distortion over time.

## 6. Rust Idiom: `let Ok(...) = ... else { return }`

```rust
let Ok((mut orbit, mut transform)) = query.single_mut() else {
    return;
};
```

This is the **let-else** pattern (stabilized in Rust 1.65). It destructures
the `Ok` variant or immediately exits the function. This is cleaner than
`if let Ok(..) = .. { ... }` when the success path is the entire function body,
as it avoids a level of indentation.

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Spherical-to-Cartesian | `x = r*cos(phi)*sin(theta)`, `y = r*sin(phi)`, `z = r*cos(phi)*cos(theta)` | Camera position from yaw/pitch/distance |
| Yaw (azimuth) | `yaw -= delta_x * sensitivity` | Horizontal orbit from mouse drag |
| Pitch (elevation) | `pitch -= delta_y * sensitivity` | Vertical orbit from mouse drag |
| Pitch clamp | `clamp(-1.4, 1.4)` | Prevent gimbal lock at poles |
| Linear zoom | `distance -= scroll * sensitivity` | Scroll wheel zoom |
| Axis-angle rotation | `q = cos(a/2) + sin(a/2)*(axis)` | Smooth auto-rotation via quaternion |
