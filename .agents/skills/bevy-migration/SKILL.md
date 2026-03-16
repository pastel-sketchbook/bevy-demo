---
name: bevy-migration
description: Guide for migrating Bevy projects between versions, with detailed API changes, renamed types, and code examples for each breaking change. Covers 0.4 through 0.18.
---

## What I do

Provide migration guidance when upgrading Bevy projects between versions. I know the breaking changes, renamed APIs, moved types, and new patterns introduced in each Bevy release.

## When to use me

- When upgrading a Bevy project from one version to another
- When encountering compilation errors after a Bevy version bump
- When you need to understand what changed between Bevy releases
- When deprecated APIs need replacement with their modern equivalents

## Migration Guide Reference

Full guides are available at: https://bevy.org/learn/migration-guides/introduction/

Available migration paths:
- 0.17 to 0.18 (latest)
- 0.16 to 0.17
- 0.15 to 0.16
- 0.14 to 0.15
- 0.13 to 0.14
- 0.12 to 0.13
- And earlier versions back to 0.4 to 0.5

## Key Changes in 0.17 to 0.18 (Current)

This project uses **Bevy 0.18.0**. The most important breaking changes from 0.17:

### Entity API Rework
- Major rework to entity IDs, entity pointers, and entity commands
- `clear_children` / `clear_related` / `remove_children` renamed to `detach_*` variants:
  - `clear_children` -> `detach_all_children`
  - `remove_children` -> `detach_children`
  - `remove_child` -> `detach_child`
  - `clear_related` -> `detach_all_related`

### RenderTarget is now a component
```rust
// 0.17
commands.spawn((
    Camera3d::default(),
    Camera { target: RenderTarget::Image(handle.into()), ..default() },
));

// 0.18
commands.spawn((
    Camera3d::default(),
    RenderTarget::Image(handle.into()),
));
```

### Feature renames
- `animation` -> `gltf_animation`
- `bevy_sprite_picking_backend` -> `sprite_picking`
- `bevy_ui_picking_backend` -> `ui_picking`
- `bevy_mesh_picking_backend` -> `mesh_picking`
- `documentation` (bevy_reflect) -> `reflect_documentation`

### Reflect attribute syntax
Only parentheses are now supported: `#[reflect(Clone)]` (not braces or brackets)

### Material changes
- `MaterialPlugin` fields `prepass_enabled`/`shadows_enabled` replaced by `Material` methods `enable_prepass()`/`enable_shadows()`
- Per-RenderPhase draw functions replace generic `MaterialDrawFunction`

### BorderRadius moved into Node
`BorderRadius` is no longer a separate component; it's now a field on `Node`.

### LineHeight is a separate component
`LineHeight` extracted from text styling into its own component.

### Mesh try_* functions
`Mesh` functions now have `try_*` variants that return `Result` instead of panicking for `RENDER_WORLD`-only meshes.

### Bundle changes
`Bundle::component_ids` and `Bundle::get_component_ids` now return iterators instead of using callbacks.

### AssetSource changes
```rust
// 0.17
AssetSource::build().with_reader(move || /* ... */);
// 0.18
AssetSourceBuilder::new(move || /* reader logic */);
```

### ron re-export removed
`ron` is no longer re-exported from `bevy_scene` or `bevy_asset`. Add `ron` as a direct dependency.

### System Combinators
Failed system conditions in combinators (`and`, `or`, `xor`, etc.) now evaluate to `false` instead of propagating errors.

### SimpleExecutor removed
Use `SingleThreadedExecutor` or `MultiThreadedExecutor` instead.

### Schedule cleanup
- `ScheduleGraph::topsort_graph` moved to `DiGraph::toposort`
- Various `ScheduleBuildError` variants restructured

### Tick-related refactors
- `TickCells` renamed to `ComponentTickCells`
- `Tick`, `ComponentTicks`, `ComponentTickCells`, `CheckChangeTicks` moved from `component` to `change_detection` module

### AmbientLight split
`AmbientLight` split into a component and a resource.

### Atmosphere changes
Most fields on `Atmosphere` replaced by `Handle<ScatteringMedium>`:
```rust
// 0.18
commands.spawn((
    Camera3d,
    Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
));
```

### Gizmos rename
`Gizmos::cuboid` renamed to `Gizmos::cube`

### Winit user events
`WinitPlugin` and `EventLoopProxyWrapper` are no longer generic. Use `WinitUserEvent` type instead.

### Resource lifetime restriction
`#[derive(Resource)]` now fails with non-static lifetimes.

### ArchetypeQueryData trait
Code requiring `ExactSizeIterator` on queries should use `ArchetypeQueryData` bound instead of `QueryData`.

## How to use

When migrating, I recommend:
1. Update `Cargo.toml` to the target Bevy version
2. Run `cargo check` to identify compilation errors
3. Search this guide for each error's type/function name
4. Apply the suggested code changes
5. For the full detailed guide with all code examples, visit:
   https://bevy.org/learn/migration-guides/0-17-to-0-18/
