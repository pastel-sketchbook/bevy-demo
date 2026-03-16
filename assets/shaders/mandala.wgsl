// Mandala fractal shader — Julia set + kaleidoscopic folding.
//
// Mapping: C × R³ × Z → R⁴
//   C  = complex Julia parameter (uniform)
//   R³ = screen position (x, y) + per-layer rotation
//   Z  = layer index (encoded via hue_offset / alpha)
//   R⁴ = RGBA output
//
// Each full-screen quad receives its own MandalaParams uniform and
// composites additively via alpha blending.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct MandalaParams {
    c: vec2<f32>,          // Julia set parameter
    folds: f32,            // Kaleidoscope fold count
    time: f32,             // Animation time (seconds)
    hue_offset: f32,       // Per-layer colour rotation [0,1]
    alpha: f32,            // Per-layer opacity
    zoom: f32,             // Zoom level
    rotation: f32,         // Per-layer rotation (radians)
};

@group(2) @binding(0)
var<uniform> params: MandalaParams;

const PI: f32  = 3.141592653589793;
const TAU: f32 = 6.283185307179586;
const ASPECT: f32 = 1.55;           // WINDOW_WIDTH / WINDOW_HEIGHT
const MAX_ITER: i32 = 64;

// --- Colour helpers --------------------------------------------------------

// FFE palette — 8 accent colours from Fuzzy Find Everything light theme.
// These are deeper/more saturated than the dark-theme accents, compositing
// without washing out to white when stacked across transparent layers.
const PALETTE_SIZE: i32 = 8;
const PAL_TAUPE:   vec3<f32> = vec3(0.95, 0.75, 0.45); // warm amber
const PAL_TEAL:    vec3<f32> = vec3(0.05, 0.85, 0.70); // vivid teal
const PAL_BERRY:   vec3<f32> = vec3(0.95, 0.15, 0.50); // hot pink
const PAL_GOLD:    vec3<f32> = vec3(1.00, 0.80, 0.00); // bright gold
const PAL_OCEAN:   vec3<f32> = vec3(0.10, 0.60, 1.00); // electric blue
const PAL_CRIMSON: vec3<f32> = vec3(1.00, 0.10, 0.20); // vivid red
const PAL_BURNT:   vec3<f32> = vec3(1.00, 0.55, 0.00); // bright orange
const PAL_SLATE:   vec3<f32> = vec3(0.40, 0.20, 0.85); // deep violet

/// Look up a colour from the FFE palette by floating-point index,
/// with smooth interpolation between adjacent entries.
fn palette(t: f32) -> vec3<f32> {
    let idx = ((t % 8.0) + 8.0) % 8.0;
    let i = i32(floor(idx));
    let f = idx - floor(idx);

    let a = palette_at(i % PALETTE_SIZE);
    let b = palette_at((i + 1) % PALETTE_SIZE);
    return mix(a, b, f);
}

fn palette_at(i: i32) -> vec3<f32> {
    switch i {
        case 0  { return PAL_TAUPE;   }
        case 1  { return PAL_TEAL;    }
        case 2  { return PAL_BERRY;   }
        case 3  { return PAL_GOLD;    }
        case 4  { return PAL_OCEAN;   }
        case 5  { return PAL_CRIMSON; }
        case 6  { return PAL_BURNT;   }
        default { return PAL_SLATE;   }
    }
}

/// Map a smooth iteration count to an FFE palette colour.
/// hue_offset shifts the palette start per layer so each layer
/// leads with a different accent colour.
fn colour_from_iter(smooth_i: f32) -> vec3<f32> {
    let band = smooth_i * 0.35 + params.hue_offset * f32(PALETTE_SIZE) + params.time * 0.15;
    return palette(band);
}

// --- Kaleidoscope ----------------------------------------------------------

/// Fold a 2D point into the first sector of an N-fold kaleidoscope.
fn kaleidoscope(p: vec2<f32>, n: f32) -> vec2<f32> {
    let sector = TAU / n;
    var angle = atan2(p.y, p.x);

    // Wrap angle into [0, TAU)
    angle = ((angle % TAU) + TAU) % TAU;

    // Fold into the first sector
    angle = angle % sector;

    // Reflect around sector midpoint for mirror symmetry
    if angle > sector * 0.5 {
        angle = sector - angle;
    }

    let r = length(p);
    return vec2(cos(angle), sin(angle)) * r;
}

// --- Julia set iteration ---------------------------------------------------

/// Run Julia iteration z = z² + c and return a smooth escape count.
fn julia(z_in: vec2<f32>, c: vec2<f32>) -> f32 {
    var z = z_in;
    var i: i32 = 0;
    for (; i < MAX_ITER; i = i + 1) {
        // z = z² + c  (complex multiply)
        let zx = z.x * z.x - z.y * z.y + c.x;
        let zy = 2.0 * z.x * z.y + c.y;
        z = vec2(zx, zy);

        if dot(z, z) > 64.0 {
            break;
        }
    }

    if i >= MAX_ITER {
        return -1.0; // inside the set
    }

    // Smooth iteration count (continuous colouring)
    let log_zn = log2(dot(z, z)) * 0.5;
    let nu     = log2(log_zn);
    return f32(i) + 1.0 - nu;
}

// --- Fragment entry point --------------------------------------------------

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Normalised coordinates centred at origin, aspect-corrected
    let uv = (in.uv - 0.5) * vec2(ASPECT, 1.0);

    // Apply per-layer rotation
    let ca = cos(params.rotation);
    let sa = sin(params.rotation);
    let rotated = vec2(uv.x * ca - uv.y * sa, uv.x * sa + uv.y * ca);

    // Zoom
    let p = rotated / params.zoom;

    // Kaleidoscopic fold
    let folded = kaleidoscope(p, params.folds);

    // Animate Julia c parameter — slow orbit around the base value
    let orbit_speed = 0.12;
    let orbit_radius = 0.03;
    let c = params.c + vec2(
        cos(params.time * orbit_speed) * orbit_radius,
        sin(params.time * orbit_speed * 1.3) * orbit_radius,
    );

    // Julia set iteration
    let smooth_i = julia(folded, c);

    if smooth_i < 0.0 {
        // Inside the set — fully transparent (let layers beneath show)
        return vec4(0.0, 0.0, 0.0, 0.0);
    }

    let rgb = colour_from_iter(smooth_i) * 1.4;

    // Radial vignette for soft edges
    let dist = length(uv);
    let vignette = smoothstep(1.2, 0.2, dist);

    // Fade out pixels far from the set boundary to keep fractal detail
    // crisp. Wider band so more of the fractal is visible.
    let boundary = smoothstep(55.0, 3.0, smooth_i) * smoothstep(0.2, 1.5, smooth_i);
    let a = params.alpha * vignette * boundary;

    // Discard truly invisible pixels
    if a < 0.003 {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }

    return vec4(rgb, a);
}
