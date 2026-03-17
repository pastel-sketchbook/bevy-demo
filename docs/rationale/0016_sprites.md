# Sprites -- Math Rationale

## Overview

The sprites demo is a minimal 2D example: a colored square moves with arrow
keys and cycles through a palette of six colors on a timer. The math is limited
to direction normalization for diagonal parity, boundary clamping, and
timer-based frame cycling via modular arithmetic.

## 1. Direction Normalization for Diagonal Parity

```rust
let mut direction = Vec3::ZERO;

if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
    direction.y += 1.0;
}
if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
    direction.y -= 1.0;
}
if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
    direction.x -= 1.0;
}
if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
    direction.x += 1.0;
}

if direction != Vec3::ZERO {
    direction = direction.normalize();
}
```

### The problem

Without normalization, pressing Up+Right produces `direction = (1, 1, 0)` with
magnitude `sqrt(2) ≈ 1.414`. Moving diagonally would be ~41% faster than
moving cardinally (magnitude 1.0). This is a classic bug in grid-based input.

### The fix

`direction.normalize()` scales any non-zero vector to unit length:

```
normalize((1, 1, 0)) = (1/sqrt(2), 1/sqrt(2), 0) ≈ (0.707, 0.707, 0)
```

Now `|direction| = 1.0` regardless of angle, and `direction * MOVE_SPEED * dt`
produces the same speed in all directions.

### The zero-vector guard

```rust
if direction != Vec3::ZERO {
    direction = direction.normalize();
}
```

`Vec3::ZERO.normalize()` produces `NaN` (zero divided by zero magnitude).
The guard ensures normalization only applies to non-zero vectors. When no keys
are pressed, `direction` stays as `Vec3::ZERO` and the sprite is stationary.

### Why `Vec3` for 2D?

Bevy's `Transform::translation` is a `Vec3` (x, y, z). Even in 2D, the z-axis
is used for draw order (z-fighting resolution). Using `Vec3` for the direction
avoids a `Vec2 -> Vec3` conversion when adding to `translation`. The z-component
is always zero.

### Opposing key cancellation

If both Up and Down are pressed simultaneously:

```
direction.y = +1.0 + (-1.0) = 0.0
```

The movements cancel out, resulting in no vertical movement. The same applies
for Left+Right. This is a natural consequence of additive key accumulation and
is the expected behavior for most games.

## 2. Boundary Clamping

```rust
let half_width = window.width() / 2.0 - SPRITE_SIZE / 2.0;
let half_height = window.height() / 2.0 - SPRITE_SIZE / 2.0;

transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
transform.translation.y = transform.translation.y.clamp(-half_height, half_height);
```

### The math

The window extends from `-window_width/2` to `+window_width/2` in world
coordinates (with a centered 2D camera). The sprite has a size of
`SPRITE_SIZE = 64` pixels and is anchored at its center. To prevent any part
of the sprite from going off-screen:

```
max_x = window_width/2 - sprite_width/2
min_x = -(window_width/2 - sprite_width/2)
```

For a 1920px window with 64px sprite:

```
max_x = 960 - 32 = 928
```

### Why clamp after movement?

The position is updated first, then clamped:

```rust
transform.translation += direction * MOVE_SPEED * time.delta_secs();
transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
```

This "move-then-clamp" pattern is simpler than "predict-and-limit" and produces
identical results for axis-aligned boundaries. The sprite slides along the wall
when pressing into it because the clamped axis is fixed while the unclamped axis
continues to move freely.

### `clamp` vs manual `min`/`max`

`f32::clamp(min, max)` is equivalent to `self.max(min).min(max)` but reads more
clearly and signals the intent: constrain a value to a range. It also panics if
`min > max` (debug mode), catching logic errors early.

## 3. Timer-Based Frame Cycling

```rust
#[derive(Component)]
struct AnimatedSprite {
    current_frame: usize,
    timer: Timer,
}
```

```rust
fn animate_sprite(mut query: Query<(&mut Sprite, &mut AnimatedSprite)>, time: Res<Time>) {
    for (mut sprite, mut anim) in query.iter_mut() {
        anim.timer.tick(time.delta());

        if anim.timer.just_finished() {
            anim.current_frame = (anim.current_frame + 1) % ANIMATION_COLORS.len();
            sprite.color = ANIMATION_COLORS[anim.current_frame];
        }
    }
}
```

### Modular arithmetic for cycling

```rust
anim.current_frame = (anim.current_frame + 1) % ANIMATION_COLORS.len();
```

This is **modular increment**: `(n + 1) % N` wraps from `N-1` back to `0`.
With 6 colors: `0 → 1 → 2 → 3 → 4 → 5 → 0 → 1 → ...`

The modulo operator `%` is the natural way to implement cyclic indexing. No
`if frame >= len { frame = 0 }` branching is needed.

### Timer mechanics

`Timer::from_seconds(FRAME_DURATION, TimerMode::Repeating)` creates a timer
that fires every `FRAME_DURATION = 0.15` seconds. The animation rate is:

```
frames_per_second = 1 / 0.15 ≈ 6.67 FPS
cycle_period = 6 colors * 0.15s = 0.9 seconds per full cycle
```

### `just_finished()` vs `finished()`

`just_finished()` returns `true` only on the tick that crosses the timer
threshold, not on every subsequent tick. For a `Repeating` timer, this fires
once per period. Using `finished()` would return `true` every frame after the
timer expires, causing the animation to advance every frame instead of every
0.15 seconds.

### `tick()` with `time.delta()`

```rust
anim.timer.tick(time.delta());
```

`tick()` advances the timer by the frame's duration. The `Timer` handles
accumulation internally -- if a frame takes 0.3s (twice the period), the timer
will fire once with `just_finished() = true` and carry over the extra 0.15s
into the next period.

## 4. Color Palette as Const Array

```rust
const ANIMATION_COLORS: [Color; 6] = [
    Color::srgb(1.0, 0.2, 0.2), // Red
    Color::srgb(1.0, 0.6, 0.2), // Orange
    Color::srgb(1.0, 1.0, 0.2), // Yellow
    Color::srgb(0.2, 1.0, 0.2), // Green
    Color::srgb(0.2, 0.6, 1.0), // Blue
    Color::srgb(0.8, 0.2, 1.0), // Purple
];
```

### Why `const`?

A `const` array is embedded in the binary at compile time and involves no heap
allocation or initialization code. This is possible because `Color::srgb` is a
`const fn` in Bevy. The array lives in the `.rodata` section and is accessed
with zero overhead.

### Hue progression

The colors roughly follow the visible spectrum (Red → Orange → Yellow → Green →
Blue → Purple), creating a rainbow cycling effect. Each color has one channel at
high saturation (≥ 0.6) and at least one at low (0.2), producing vivid,
distinct colors.

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Direction normalization | `v / |v|` if `v ≠ 0` | Equal speed in all directions |
| Boundary clamp | `clamp(pos, -max, max)` | Keep sprite within window |
| Modular cycling | `(frame + 1) % len` | Wrap frame index to palette size |
| Timer period | `period = 0.15s` → 6.67 FPS | Frame-rate-independent animation speed |
