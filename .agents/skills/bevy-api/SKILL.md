---
name: bevy-api
description: Reference for Bevy 0.18 crate structure, modules, key types, cargo features, and feature profiles. Use when you need to understand Bevy's architecture or configure cargo features.
---

## What I do

Provide a reference for the Bevy 0.18 crate architecture, module organization, key types, and the full cargo feature system including profiles and collections.

## When to use me

- When you need to know which Bevy module contains a specific type or feature
- When configuring `Cargo.toml` feature flags for Bevy
- When you want to minimize compile times by selecting only needed features
- When understanding the Bevy crate/subcrate structure
- When looking up what `bevy::prelude::*` provides

## Crate Overview

Bevy 0.18.0 is a container crate that re-exports subcrates. Each module corresponds to a `bevy_*` crate on crates.io (e.g., `bevy::app` -> `bevy_app`).

API docs: https://docs.rs/bevy/0.18.0/bevy/

## Hello World

```rust
use bevy::prelude::*;

fn main() {
    App::new()
        .add_systems(Update, hello_world_system)
        .run();
}

fn hello_world_system() {
    println!("hello world");
}
```

## Plugin Groups

- **`DefaultPlugins`** - Full default Bevy application plugins
- **`MinimalPlugins`** - Minimal plugin set for headless/custom apps

## Module Reference

### Core Framework
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::app` | `bevy_app` | Application layer, `App`, `Plugin`, schedules |
| `bevy::ecs` | `bevy_ecs` | Entity Component System core |
| `bevy::reflect` | `bevy_reflect` | Runtime reflection |
| `bevy::tasks` | `bevy_tasks` | Async task pools |
| `bevy::utils` | `bevy_utils` | General utilities |
| `bevy::platform` | `bevy_platform` | Platform compatibility |
| `bevy::log` | `bevy_log` | Logging with `tracing` integration |
| `bevy::diagnostic` | `bevy_diagnostic` | Diagnostics (FPS, etc.) |

### Math & Transform
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::math` | `bevy_math` | Math types (Vec3, Quat, etc.) |
| `bevy::transform` | `bevy_transform` | Transform and GlobalTransform |
| `bevy::color` | `bevy_color` | Color types and operations |

### Rendering
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::render` | `bevy_render` | Core rendering |
| `bevy::core_pipeline` | `bevy_core_pipeline` | Camera and basic render pipeline |
| `bevy::pbr` | `bevy_pbr` | Physically Based Rendering |
| `bevy::light` | `bevy_light` | Light types (point, directional, spot) |
| `bevy::camera` | `bevy_camera` | Camera and visibility types |
| `bevy::mesh` | `bevy_mesh` | Mesh format and primitives |
| `bevy::shader` | `bevy_shader` | Shader asset handles |
| `bevy::image` | `bevy_image` | Image loading and access |
| `bevy::anti_alias` | `bevy_anti_alias` | Anti-aliasing solutions |
| `bevy::post_process` | `bevy_post_process` | Post-processing (DoF, bloom, etc.) |
| `bevy::gizmos` | `bevy_gizmos` | Immediate mode debug drawing |

### 2D
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::sprite` | `bevy_sprite` | 2D sprite functionality |
| `bevy::sprite_render` | `bevy_sprite_render` | 2D sprite rendering |

### 3D
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::gltf` | `bevy_gltf` | glTF 2.0 loading |
| `bevy::solari` | `bevy_solari` | Raytraced lighting (experimental) |

### UI
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::ui` | `bevy_ui` | ECS-driven UI framework |
| `bevy::ui_render` | `bevy_ui_render` | UI rendering |
| `bevy::ui_widgets` | `bevy_ui_widgets` | Standard headless widgets |
| `bevy::feathers` | `bevy_feathers` | Styled widget collection |
| `bevy::text` | `bevy_text` | Text positioning and rendering |

### Input & Interaction
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::input` | `bevy_input` | Input handling |
| `bevy::input_focus` | `bevy_input_focus` | UI focus system |
| `bevy::picking` | `bevy_picking` | Pointer-entity interaction |
| `bevy::gilrs` | `bevy_gilrs` | Gamepad support |

### Windowing
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::window` | `bevy_window` | Platform-agnostic windowing |
| `bevy::winit` | `bevy_winit` | winit window/input backend |
| `bevy::a11y` | `bevy_a11y` | Accessibility primitives |

### Assets & Scenes
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::asset` | `bevy_asset` | Asset loading and management |
| `bevy::scene` | `bevy_scene` | Scene definition and serialization |

### Animation & State
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::animation` | `bevy_animation` | Animation system |
| `bevy::state` | `bevy_state` | Global state machines |

### Audio & Time
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::audio` | `bevy_audio` | Audio playback |
| `bevy::time` | `bevy_time` | Time management |

### Other
| Module | Crate | Description |
|--------|-------|-------------|
| `bevy::camera_controller` | `bevy_camera_controller` | First-party camera controllers |
| `bevy::dev_tools` | `bevy_dev_tools` | Developer utilities |
| `bevy::remote` | `bevy_remote` | Bevy Remote Protocol |

## Cargo Feature Profiles

High-level feature groups for `default-features = false`:

| Profile | Description |
|---------|-------------|
| `default` | Full experience: 2d + 3d + ui |
| `2d` | Core framework + 2D + UI + scenes + audio + picking |
| `3d` | Core framework + 3D + UI + scenes + audio + picking |
| `ui` | Core framework + UI + scenes + audio + picking |

```toml
# Example: 2D-only project
bevy = { version = "0.18", default-features = false, features = ["2d"] }
```

## Feature Collections

Mid-level groups for custom profiles:

| Collection | Description |
|------------|-------------|
| `dev` | Hot-reloading, debug tools (don't ship!) |
| `audio` | Audio features |
| `scene` | Scene composition |
| `picking` | Picking functionality |
| `default_app` | Core baseline for headless apps |
| `default_platform` | OS support, windowing, input backends |
| `common_api` | Scene definition (no renderer) |
| `2d_api` | 2D features without render backend |
| `3d_api` | 3D features without render backend |
| `ui_api` | UI features without render backend |
| `default_no_std` | Defaults for no_std |

## Commonly Used Individual Features

| Feature | Description |
|---------|-------------|
| `dynamic_linking` | Faster iterative compile times |
| `file_watcher` | Asset hot-reloading from filesystem |
| `embedded_watcher` | Hot-reloading for embedded assets |
| `asset_processor` | Built-in asset processor |
| `serialize` | Serde serialization support |
| `bevy_dev_tools` | Developer tools collection |
| `bevy_ci_testing` | Automated CI testing |
| `bevy_debug_stepping` | Step-through system debugging |
| `bevy_remote` | Bevy Remote Protocol |
| `trace` | Tracing support |
| `trace_chrome` | Chrome Tracing format output |
| `trace_tracy` | Tracy profiler integration |
| `multi_threaded` | Multithreaded parallelism (on by default) |
| `hotpatching` | Hot-patching of systems |
| `webgl2` | WebGL2 support (Wasm) |
| `webgpu` | WebGPU support (Wasm, overrides webgl2) |

## Image Format Features

`png`, `jpeg`, `bmp`, `hdr`, `dds`, `ktx2`, `exr`, `basis-universal`, `tga`, `tiff`, `webp`, `gif`, `ico`, `pnm`, `ff`, `qoi`

## Audio Format Features

`vorbis`, `flac`, `mp3`, `wav`, `symphonia-all`, `symphonia-aac`, `symphonia-flac`, `symphonia-isomp4`, `symphonia-vorbis`, `symphonia-wav`

## Platform Features

| Feature | Description |
|---------|-------------|
| `x11` | X11 display server |
| `wayland` | Wayland display server |
| `web` | Browser APIs (wasm32) |
| `android-game-activity` | Android GameActivity (default) |
| `android-native-activity` | Android NativeActivity (legacy) |
