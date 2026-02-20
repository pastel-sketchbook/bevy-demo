---
name: bevy-examples
description: Catalog of official Bevy engine examples organized by category, with descriptions and source file paths. Use when looking for reference implementations of specific Bevy features.
---

## What I do

Provide a searchable catalog of all official Bevy examples. Each entry includes the example name, source file path, and a brief description of what it demonstrates.

## When to use me

- When looking for a reference implementation of a specific Bevy feature
- When you need to find an example that demonstrates a particular pattern (ECS, rendering, input, etc.)
- When recommending example code to study for a given concept
- When setting up a new Bevy binary and want to follow established patterns

## How to run examples

```sh
cargo run --example <example_name>
# For Wayland/X11:
cargo run --features wayland --example <example_name>
# Stress tests should use release mode:
cargo run --release --example <example_name>
```

Source: https://github.com/bevyengine/bevy/tree/latest/examples

## Example Categories

### 2D Rendering
| Example | File | Description |
|---------|------|-------------|
| 2D Bloom | `examples/2d/bloom_2d.rs` | Bloom post-processing in 2D |
| 2D Shapes | `examples/2d/2d_shapes.rs` | Simple 2D primitive shapes (circles, polygons) |
| 2D Viewport To World | `examples/2d/2d_viewport_to_world.rs` | `Camera::viewport_to_world_2d` with dynamic viewport |
| CPU Drawing | `examples/2d/cpu_draw.rs` | Manually read/write texture pixels |
| Manual Mesh 2D | `examples/2d/mesh2d_manual.rs` | Custom mesh with mid-level renderer APIs |
| Mesh 2D | `examples/2d/mesh2d.rs` | Renders a 2D mesh |
| Move Sprite | `examples/2d/move_sprite.rs` | Transform changes on a sprite |
| Sprite | `examples/2d/sprite.rs` | Basic sprite rendering |
| Sprite Animation | `examples/2d/sprite_animation.rs` | Sprite animation in response to events |
| Sprite Sheet | `examples/2d/sprite_sheet.rs` | Animated sprite sheet |
| Sprite Slice | `examples/2d/sprite_slice.rs` | 9-patch sprite slicing |
| Text 2D | `examples/2d/text2d.rs` | Text generation in 2D |
| Texture Atlas | `examples/2d/texture_atlas.rs` | Generate texture atlas from individual sprites |
| Tilemap Chunk | `examples/2d/tilemap_chunk.rs` | Tilemap chunk rendering |
| Transparency 2D | `examples/2d/transparency_2d.rs` | 2D transparency |

### 3D Rendering
| Example | File | Description |
|---------|------|-------------|
| 3D Scene | `examples/3d/3d_scene.rs` | Simple 3D scene with shapes and lighting |
| 3D Shapes | `examples/3d/3d_shapes.rs` | Built-in 3D shapes |
| 3D Bloom | `examples/3d/bloom_3d.rs` | Bloom with HDR and emissive materials |
| Anti-aliasing | `examples/3d/anti_aliasing.rs` | Compares AA techniques |
| Atmosphere | `examples/3d/atmosphere.rs` | PBR atmospheric scattering |
| Deferred Rendering | `examples/3d/deferred_rendering.rs` | Forward and deferred pipelines |
| Depth of Field | `examples/3d/depth_of_field.rs` | DoF demonstration |
| Fog | `examples/3d/fog.rs` | Distance fog effect |
| Generate Custom Mesh | `examples/3d/generate_custom_mesh.rs` | Custom mesh with custom texture |
| Lighting | `examples/3d/lighting.rs` | Various lighting options |
| Lines | `examples/3d/lines.rs` | Custom material for 3D lines |
| Mesh Ray Cast | `examples/3d/mesh_ray_cast.rs` | Ray casting with `MeshRayCast` |
| Mirror | `examples/3d/mirror.rs` | Mirror with second camera |
| Motion Blur | `examples/3d/motion_blur.rs` | Per-pixel motion blur |
| PBR | `examples/3d/pbr.rs` | Physically Based Rendering properties |
| Render to Texture | `examples/3d/render_to_texture.rs` | Render to texture for mirrors/UI |
| Shadow Biases | `examples/3d/shadow_biases.rs` | Shadow bias effects |
| Skybox | `examples/3d/skybox.rs` | Cubemap skybox |
| Split Screen | `examples/3d/split_screen.rs` | Two cameras, one window |
| Wireframe | `examples/3d/wireframe.rs` | Wireframe rendering |

### Animation
| Example | File | Description |
|---------|------|-------------|
| Animated Mesh | `examples/animation/animated_mesh.rs` | Skinned glTF animation (fox) |
| Animated Transform | `examples/animation/animated_transform.rs` | Code-defined Transform animation |
| Animation Graph | `examples/animation/animation_graph.rs` | Blending animations with graph |
| Animation Masks | `examples/animation/animation_masks.rs` | Animation masks |
| Color Animation | `examples/animation/color_animation.rs` | Color animation with splines |
| Custom Skinned Mesh | `examples/animation/custom_skinned_mesh.rs` | Mesh/joints defined in code |
| Easing Functions | `examples/animation/easing_functions.rs` | Built-in easing functions |
| Morph Targets | `examples/animation/morph_targets.rs` | glTF morph targets |

### Application
| Example | File | Description |
|---------|------|-------------|
| Custom Loop | `examples/app/custom_loop.rs` | Custom runner for manual updates |
| Drag and Drop | `examples/app/drag_and_drop.rs` | File drag and drop handling |
| Empty | `examples/app/empty.rs` | Empty application |
| Headless | `examples/app/headless.rs` | No default plugins |
| Headless Renderer | `examples/app/headless_renderer.rs` | No window, renders to image file |
| Plugin | `examples/app/plugin.rs` | Custom plugin creation |
| Plugin Group | `examples/app/plugin_group.rs` | Custom plugin group |

### Assets
| Example | File | Description |
|---------|------|-------------|
| Asset Loading | `examples/asset/asset_loading.rs` | Various asset loading methods |
| Custom Asset | `examples/asset/custom_asset.rs` | Custom asset loader |
| Custom Asset IO | `examples/asset/custom_asset_reader.rs` | Custom AssetReader |
| Embedded Asset | `examples/asset/embedded_asset.rs` | Embed asset in binary |
| Hot Reloading | `examples/asset/hot_asset_reloading.rs` | Auto-reload on disk changes |

### Audio
| Example | File | Description |
|---------|------|-------------|
| Audio | `examples/audio/audio.rs` | Load and play audio |
| Audio Control | `examples/audio/audio_control.rs` | Audio playback control |
| Spatial Audio 2D | `examples/audio/spatial_audio_2d.rs` | 2D spatial audio |
| Spatial Audio 3D | `examples/audio/spatial_audio_3d.rs` | 3D spatial audio |

### Camera
| Example | File | Description |
|---------|------|-------------|
| 2D Top-Down Camera | `examples/camera/2d_top_down_camera.rs` | Smooth-follow 2D camera |
| Camera Orbit | `examples/camera/camera_orbit.rs` | Orbit with pitch/yaw/roll |
| First Person View | `examples/camera/first_person_view_model.rs` | FPS camera with view model |
| Free Camera | `examples/camera/free_camera_controller.rs` | FreeCamera controller for 3D |
| Pan Camera | `examples/camera/pan_camera_controller.rs` | Pan-style 2D camera |
| Projection Zoom | `examples/camera/projection_zoom.rs` | Ortho and perspective zoom |
| Screen Shake | `examples/camera/2d_screen_shake.rs` | 2D screen shake effect |

### ECS (Entity Component System)
| Example | File | Description |
|---------|------|-------------|
| Change Detection | `examples/ecs/change_detection.rs` | Component/resource change detection |
| Component Hooks | `examples/ecs/component_hooks.rs` | Lifecycle event hooks |
| Custom Query Params | `examples/ecs/custom_query_param.rs` | Compound query types |
| Custom Schedule | `examples/ecs/custom_schedule.rs` | Custom schedules |
| Dynamic ECS | `examples/ecs/dynamic.rs` | Dynamic component creation |
| ECS Guide | `examples/ecs/ecs_guide.rs` | Full ECS guide |
| Entity Disabling | `examples/ecs/entity_disabling.rs` | Hide entities without deleting |
| Error Handling | `examples/ecs/error_handling.rs` | ECS error handling patterns |
| Fixed Timestep | `examples/ecs/fixed_timestep.rs` | Fixed update rate systems |
| Generic System | `examples/ecs/generic_system.rs` | Reusable generic systems |
| Hierarchy | `examples/ecs/hierarchy.rs` | Parent-child relationships |
| Observers | `examples/ecs/observers.rs` | Event-reacting observers |
| Observer Propagation | `examples/ecs/observer_propagation.rs` | Event propagation |
| One Shot Systems | `examples/ecs/one_shot_systems.rs` | Run systems without scheduling |
| Parallel Query | `examples/ecs/parallel_query.rs` | ParallelIterator queries |
| Relationships | `examples/ecs/relationships.rs` | Custom entity relationships |
| Run Conditions | `examples/ecs/run_conditions.rs` | Conditional system execution |
| System Param | `examples/ecs/system_param.rs` | Custom SystemParam |
| System Piping | `examples/ecs/system_piping.rs` | Pipe system outputs |

### Games
| Example | File | Description |
|---------|------|-------------|
| Alien Cake Addict | `examples/games/alien_cake_addict.rs` | 3D game example |
| Breakout | `examples/games/breakout.rs` | Classic Breakout |
| Desk Toy | `examples/games/desk_toy.rs` | Transparent window desk toy |
| Game Menu | `examples/games/game_menu.rs` | Simple game menu |
| Loading Screen | `examples/games/loading_screen.rs` | Asset loading screen |

### Input
| Example | File | Description |
|---------|------|-------------|
| Keyboard Input | `examples/input/keyboard_input.rs` | Key press/release handling |
| Mouse Input | `examples/input/mouse_input.rs` | Mouse button handling |
| Mouse Grab | `examples/input/mouse_grab.rs` | Cursor lock to window |
| Gamepad Input | `examples/input/gamepad_input.rs` | Gamepad handling |
| Touch Input | `examples/input/touch_input.rs` | Touch press/release |
| Text Input | `examples/input/text_input.rs` | Text input with IME |

### Movement
| Example | File | Description |
|---------|------|-------------|
| Physics in Fixed Timestep | `examples/movement/physics_in_fixed_timestep.rs` | Industry-standard input/physics/render loop |
| Smooth Follow | `examples/movement/smooth_follow.rs` | Smooth entity following with interpolation |

### Shaders
| Example | File | Description |
|---------|------|-------------|
| Animated Shader | `examples/shader/animate_shader.rs` | Dynamic data (time) in shaders |
| Compute Game of Life | `examples/shader/compute_shader_game_of_life.rs` | Compute shader |
| Extended Material | `examples/shader/extended_material.rs` | Build on standard material |
| Material | `examples/shader/shader_material.rs` | Custom shader material |
| Material 2D | `examples/shader/shader_material_2d.rs` | 2D shader material |
| Material GLSL | `examples/shader/shader_material_glsl.rs` | GLSL shading language |
| Shader Defs | `examples/shader/shader_defs.rs` | Selectively toggle shader parts |
| Storage Buffer | `examples/shader/storage_buffer.rs` | Bind storage buffer |

### State
| Example | File | Description |
|---------|------|-------------|
| States | `examples/state/states.rs` | Menu to InGame transitions |
| Computed States | `examples/state/computed_states.rs` | Advanced computed states |
| Sub States | `examples/state/sub_states.rs` | Hierarchical state handling |

### UI (User Interface)
| Example | File | Description |
|---------|------|-------------|
| Button | `examples/ui/button.rs` | Creating and updating buttons |
| CSS Grid | `examples/ui/grid.rs` | CSS Grid layout |
| Flex Layout | `examples/ui/flex_layout.rs` | AlignItems/JustifyContent layout |
| Scroll | `examples/ui/scroll.rs` | Scrolling containers |
| Text | `examples/ui/text.rs` | Text creation and updates |
| UI Material | `examples/ui/ui_material.rs` | Custom UI materials |
| UI Scaling | `examples/ui/ui_scaling.rs` | UI scale |
| Z-Index | `examples/ui/z_index.rs` | UI depth control |
| Borders | `examples/ui/borders.rs` | Node borders |
| Box Shadow | `examples/ui/box_shadow.rs` | Node shadows |
| Gradients | `examples/ui/gradients.rs` | Gradient rendering |
| Render UI to Texture | `examples/ui/render_ui_to_texture.rs` | UI as part of 3D world |

### Window
| Example | File | Description |
|---------|------|-------------|
| Clear Color | `examples/window/clear_color.rs` | Solid color window |
| Multiple Windows | `examples/window/multiple_windows.rs` | Multi-window rendering |
| Transparent Window | `examples/window/transparent_window.rs` | Transparent/borderless window |
| Window Settings | `examples/window/window_settings.rs` | Custom window settings |
| Screenshot | `examples/window/screenshot.rs` | Save screenshots to disk |

### glTF
| Example | File | Description |
|---------|------|-------------|
| Load glTF | `examples/gltf/load_gltf.rs` | Load and render glTF scene |
| glTF Skinned Mesh | `examples/gltf/gltf_skinned_mesh.rs` | Skinned mesh from glTF |
| Edit glTF Material | `examples/gltf/edit_material_on_gltf.rs` | Change materials post-spawn |

### Testing
| Example | File | Description |
|---------|------|-------------|
| How to Test Apps | `tests/how_to_test_apps.rs` | Simple integration testing |
| How to Test Systems | `tests/how_to_test_systems.rs` | Test systems with commands/queries |
