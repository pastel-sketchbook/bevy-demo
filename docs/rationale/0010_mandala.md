# Mandala -- Math Rationale

## Overview

The mandala demo renders a GPU-computed fractal by stacking 8 transparent
full-screen layers, each running its own Julia set iteration with kaleidoscopic
folding in a WGSL fragment shader. The math covers complex-plane iteration,
smooth escape-time colouring, kaleidoscopic angular folding, 2D rotation
matrices, heightfield-based normal estimation, and Blinn-Phong lighting applied
to a 2D fractal surface.

## 1. Julia Set Iteration (Complex Quadratic Map)

The core fractal is the Julia set of the map `z -> z^2 + c` where `c` is a
fixed complex parameter and `z_0` is the screen-space coordinate.

### The formula

```
z_{n+1} = z_n^2 + c
```

Expanding the complex square in Cartesian form:

```
(a + bi)^2 = a^2 - b^2 + 2abi
```

So each iteration computes:

```
z.x' = z.x * z.x - z.y * z.y + c.x
z.y' = 2.0 * z.x * z.y + c.y
```

### WGSL implementation

```wgsl
let zx = z.x * z.x - z.y * z.y + c.x;
let zy = 2.0 * z.x * z.y + c.y;
z = vec2(zx, zy);
```

Iteration continues until either `|z|^2 > 64.0` (escaped) or the iteration
count reaches `MAX_ITER = 64` (inside the set). The escape radius of 64 (rather
than the minimal 2) provides extra iterations for smoother colouring.

### Escape test

```wgsl
if dot(z, z) > 64.0 { break; }
```

`dot(z, z) = z.x^2 + z.y^2 = |z|^2` -- this avoids computing a square root.

## 2. Smooth Iteration Count (Continuous Colouring)

Discrete iteration counts produce banding artefacts. The smooth iteration count
eliminates them using the *normalised iteration count* formula.

### The formula

```
smooth_i = i + 1 - log2(log2(|z|^2) / 2)
         = i + 1 - log2(log2(|z_n|))
```

### WGSL implementation

```wgsl
let log_zn = log2(dot(z, z)) * 0.5;  // log2(|z|)
let nu     = log2(log_zn);            // log2(log2(|z|))
return f32(i) + 1.0 - nu;
```

The key insight: when a point escapes, its distance from the origin carries
fractional information about "how close" it was to escaping on the previous
iteration. The double logarithm maps this exponentially-growing magnitude back
to a smooth `[0, 1)` fractional correction.

## 3. Kaleidoscopic Folding

Before Julia iteration, each screen coordinate is folded into the first sector
of an N-fold kaleidoscope, producing radial symmetry.

### The formula

Given fold count `n` and sector angle `s = 2*PI / n`:

```
theta = atan2(y, x)            -- angle of the point
theta = theta mod s             -- fold into first sector
if theta > s/2:
    theta = s - theta           -- mirror reflect
```

The point is then reconstructed as `(r * cos(theta), r * sin(theta))`.

### Why mirror reflect?

Without the reflection, adjacent sectors would be rotated copies. With it,
alternating sectors are mirrored, creating the characteristic mandala symmetry
where each "petal" is a mirror image of its neighbour.

### WGSL implementation

```wgsl
fn kaleidoscope(p: vec2<f32>, n: f32) -> vec2<f32> {
    let sector = TAU / n;
    var angle = atan2(p.y, p.x);
    angle = ((angle % TAU) + TAU) % TAU;   // wrap to [0, TAU)
    angle = angle % sector;                 // fold to first sector
    if angle > sector * 0.5 {
        angle = sector - angle;             // mirror
    }
    let r = length(p);
    return vec2(cos(angle), sin(angle)) * r;
}
```

The double-modulo `((a % b) + b) % b` is the standard pattern for wrapping a
potentially negative value into `[0, b)` -- necessary because `atan2` returns
values in `(-PI, PI]`.

## 4. Per-Layer Rotation (2D Rotation Matrix)

Each layer rotates at a different speed. The rotation is a standard 2x2 matrix:

```
x' = x * cos(a) - y * sin(a)
y' = x * sin(a) + y * cos(a)
```

### WGSL implementation

```wgsl
let ca = cos(params.rotation);
let sa = sin(params.rotation);
let rotated = vec2(uv.x * ca - uv.y * sa, uv.x * sa + uv.y * ca);
```

The `rotation` parameter is computed on the CPU side as:

```rust
mat.params.rotation = t * rot_speed * 0.3;
```

where `rot_speed` varies per layer (some negative for counter-rotation). The
`0.3` factor keeps the visual rotation speed gentle.

## 5. Julia Parameter Animation

The Julia `c` parameter orbits slowly around its base value via a Lissajous
curve:

```wgsl
let c = params.c + vec2(
    cos(params.time * 0.12) * 0.03,
    sin(params.time * 0.12 * 1.3) * 0.03,
);
```

The incommensurate frequency ratio `1.0 : 1.3` between x and y ensures the
orbit never exactly closes, tracing a dense path in parameter space. The small
radius (`0.03`) keeps the fractal topologically similar to its base shape while
introducing continuous variation.

## 6. Surface Normal Estimation (Central Differences)

The fractal's smooth iteration count is treated as a heightfield. Surface
normals are estimated using central finite differences:

```wgsl
fn estimate_normal(uv: vec2<f32>, c: vec2<f32>) -> vec3<f32> {
    let eps = 0.002;
    let hL = fractal_height(uv - vec2(eps, 0.0), c);
    let hR = fractal_height(uv + vec2(eps, 0.0), c);
    let hD = fractal_height(uv - vec2(0.0, eps), c);
    let hU = fractal_height(uv + vec2(0.0, eps), c);

    let height_scale = 0.15;
    return normalize(vec3(
        (hL - hR) * height_scale,
        (hD - hU) * height_scale,
        1.0,
    ));
}
```

### The formula

The surface gradient is approximated as:

```
dh/dx ~ (h(x-e) - h(x+e)) / (2*e)
dh/dy ~ (h(y-e) - h(y+e)) / (2*e)
```

The unnormalised normal is `(-dh/dx, -dh/dy, 1)`. The `height_scale` controls
how pronounced the 3D effect appears -- smaller values flatten the surface.

This is computationally expensive (4 extra fractal evaluations per pixel) but
produces convincing pseudo-3D lighting on a flat 2D fractal.

## 7. Blinn-Phong Lighting

Three lighting components are combined for the final colour:

### Diffuse (Lambert)

```wgsl
let ndotl = max(dot(normal, light_dir), 0.0);
let diffuse = ambient + (1.0 - ambient) * ndotl;
```

The `ambient = 0.35` floor prevents fully unlit areas from going black.

### Specular (Blinn-Phong)

```wgsl
let half_dir = normalize(light_dir + view_dir);
let spec = pow(max(dot(normal, half_dir), 0.0), 32.0);
```

The half-vector formulation is cheaper than reflection-based Phong (one
`normalize` instead of a `reflect`). The exponent `32.0` produces moderately
tight highlights.

### Rim light

```wgsl
let rim = 1.0 - max(dot(normal, view_dir), 0.0);
let rim_glow = pow(rim, 3.0) * 0.3;
```

Rim lighting highlights edges where the surface normal is perpendicular to the
view direction. The `pow(_, 3.0)` concentrates the effect near the edges. This
adds a subtle glow that separates overlapping layers visually.

### Light direction

The light orbits slowly, with a per-layer phase offset:

```wgsl
let light_angle = params.time * 0.2 + params.layer_depth * PI * 0.5;
let light_dir = normalize(vec3(cos(light_angle) * 0.6, sin(light_angle) * 0.6, 1.0));
```

The z-component of `1.0` keeps the light predominantly frontal, while the xy
components orbit to create shifting highlights across layers.

## 8. Layer Compositing

Eight full-screen quads are stacked at increasing z-depths:

```rust
commands.spawn((
    Mesh2d(mesh_handle.clone()),
    MeshMaterial2d(mat_handle),
    Transform::from_xyz(0.0, 0.0, i as f32),
));
```

Each layer uses `AlphaMode2d::Blend`, so Bevy composites them back-to-front
using standard alpha blending:

```
output = src_rgb * src_a + dst_rgb * (1 - src_a)
```

The per-layer alpha values in `LAYER_PRESETS` range from `0.15` (subtle
background layers) to `0.55` (hero layers), creating depth through differential
transparency.

### Vignette and boundary fade

Two smoothstep functions control per-pixel opacity:

```wgsl
let vignette = smoothstep(1.2, 0.2, dist);        // radial fade at edges
let boundary = smoothstep(55.0, 3.0, smooth_i)     // fade far-escaped pixels
             * smoothstep(0.2, 1.5, smooth_i);     // fade near-set pixels
let a = params.alpha * vignette * boundary;
```

`smoothstep(edge0, edge1, x)` returns a Hermite-interpolated value in `[0, 1]`.
The boundary fade restricts colour to a band near the set boundary -- the most
visually interesting region of any fractal.

## 9. Colour Mapping (Palette Interpolation)

The smooth iteration count is mapped to colour via an 8-entry palette with
linear interpolation:

```wgsl
fn palette(t: f32) -> vec3<f32> {
    let idx = ((t % 8.0) + 8.0) % 8.0;
    let i = i32(floor(idx));
    let f = idx - floor(idx);
    return mix(palette_at(i % 8), palette_at((i + 1) % 8), f);
}
```

The double-modulo wrapping ensures negative inputs produce valid indices. The
`mix` (linear interpolation) between adjacent entries eliminates hard colour
boundaries.

Each layer's `hue_offset` shifts the palette starting point, so stacked layers
lead with different accent colours and do not blend into a uniform wash.

## Summary of Math

| Concept | Formula | Purpose |
|---|---|---|
| Complex square | `(a+bi)^2 = (a^2-b^2) + 2abi` | Julia set iteration kernel |
| Escape test | `dot(z,z) > R^2` | Avoid `sqrt` in magnitude check |
| Smooth iteration | `i + 1 - log2(log2(\|z\|))` | Continuous colouring without banding |
| Kaleidoscope fold | `theta mod (2*PI/n)` + mirror | N-fold radial symmetry |
| 2D rotation | `[cos -sin; sin cos] * p` | Per-layer animated rotation |
| Central differences | `(h(x-e) - h(x+e)) / 2e` | Surface normal from heightfield |
| Lambert diffuse | `max(N . L, 0)` | Basic directional lighting |
| Blinn-Phong specular | `pow(max(N . H, 0), k)` | Shiny highlights |
| Rim light | `pow(1 - N . V, 3)` | Edge glow for layer separation |
| Alpha compositing | `src*a + dst*(1-a)` | Back-to-front layer blending |
| Smoothstep vignette | `smoothstep(edge0, edge1, d)` | Radial and boundary fade |
