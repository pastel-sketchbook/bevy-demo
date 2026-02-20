//! Conway's Game of Life rendered as a CPU-drawn texture on a sprite.
//! Left-click to paint cells, right-click to erase. Space to pause/resume,
//! R to randomize, C to clear. Simulation runs in FixedUpdate at ~10Hz.

#[cfg(feature = "transparent")]
use bevy::window::CompositeAlphaMode;
use bevy::{
    app::AppExit,
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::{PrimaryWindow, WindowPlugin, WindowPosition, WindowResolution},
};
use rand::{Rng, SeedableRng, rngs::SmallRng};

// --- Constants ---
const WINDOW_WIDTH: f32 = 1606.0;
const WINDOW_HEIGHT: f32 = 1036.0;
const RANDOM_SEED: u64 = 98765432101234567;

const GRID_WIDTH: usize = 320;
const GRID_HEIGHT: usize = 206;
const CELL_ALIVE_COLOR: [u8; 4] = [178, 225, 198, 255]; // Pastel mint
const CELL_DEAD_COLOR: [u8; 4] = [42, 40, 52, 255]; // Pastel charcoal
const SIMULATION_HZ: f64 = 10.0;
const BRUSH_RADIUS: i32 = 2;

#[cfg(feature = "transparent")]
const BACKGROUND_COLOR: Color = Color::srgba(0.05, 0.05, 0.1, 0.3);
#[cfg(not(feature = "transparent"))]
const BACKGROUND_COLOR: Color = Color::srgb(0.05, 0.05, 0.1);

// --- Components ---

#[derive(Component)]
struct GridSprite;

// --- Resources ---

#[derive(Resource)]
struct RandomSource(SmallRng);

#[derive(Resource)]
struct LifeGrid {
    cells: Vec<bool>,
    back: Vec<bool>,
    width: usize,
    height: usize,
}

impl LifeGrid {
    fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            cells: vec![false; size],
            back: vec![false; size],
            width,
            height,
        }
    }

    fn randomize(&mut self, rng: &mut SmallRng) {
        for cell in self.cells.iter_mut() {
            *cell = rng.random_bool(0.3);
        }
    }

    fn clear(&mut self) {
        self.cells.fill(false);
    }

    fn get(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x]
        } else {
            false
        }
    }

    fn set(&mut self, x: usize, y: usize, alive: bool) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = alive;
        }
    }

    fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0u8;
        for dy in [-1i32, 0, 1] {
            for dx in [-1i32, 0, 1] {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0
                    && nx < self.width as i32
                    && ny >= 0
                    && ny < self.height as i32
                    && self.cells[ny as usize * self.width + nx as usize]
                {
                    count += 1;
                }
            }
        }
        count
    }

    fn step(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let neighbors = self.count_neighbors(x, y);
                let alive = self.cells[y * self.width + x];
                self.back[y * self.width + x] =
                    matches!((alive, neighbors), (true, 2) | (true, 3) | (false, 3));
            }
        }
        std::mem::swap(&mut self.cells, &mut self.back);
    }
}

#[derive(Resource)]
struct SimPaused(bool);

#[derive(Resource)]
struct GridImageHandle(Handle<Image>);

// --- Main ---

#[cfg(feature = "window-offset")]
fn offset_window(mut windows: Query<&mut Window>, mut done: Local<bool>) {
    if *done {
        return;
    }
    for mut window in windows.iter_mut() {
        window.position = WindowPosition::At(IVec2::new(160, 88));
        info!("Window positioned at: (160, 88)");
        *done = true;
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            decorations: false,
            #[cfg(feature = "transparent")]
            transparent: true,
            #[cfg(feature = "transparent")]
            composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
            resolution: WindowResolution::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32),
            position: WindowPosition::Centered(MonitorSelection::Primary),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(BACKGROUND_COLOR))
    .insert_resource(RandomSource(SmallRng::seed_from_u64(RANDOM_SEED)))
    .insert_resource(SimPaused(false))
    .insert_resource(Time::<Fixed>::from_hz(SIMULATION_HZ))
    .add_systems(Startup, setup)
    .add_systems(FixedUpdate, simulate)
    .add_systems(
        Update,
        (
            #[cfg(feature = "window-offset")]
            offset_window,
            handle_input,
            mouse_paint,
            render_grid,
            handle_quit,
        ),
    );
    app.run();
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut rng: ResMut<RandomSource>) {
    commands.spawn(Camera2d);

    // Create the grid texture
    let size = Extent3d {
        width: GRID_WIDTH as u32,
        height: GRID_HEIGHT as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &CELL_DEAD_COLOR,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    let image_handle = images.add(image);

    commands.spawn((
        Sprite {
            image: image_handle.clone(),
            custom_size: Some(Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT)),
            ..default()
        },
        GridSprite,
    ));

    commands.insert_resource(GridImageHandle(image_handle));

    // Initialize grid with random state
    let mut grid = LifeGrid::new(GRID_WIDTH, GRID_HEIGHT);
    grid.randomize(&mut rng.0);
    commands.insert_resource(grid);
}

fn simulate(mut grid: ResMut<LifeGrid>, paused: Res<SimPaused>) {
    if paused.0 {
        return;
    }
    grid.step();
}

fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut grid: ResMut<LifeGrid>,
    mut paused: ResMut<SimPaused>,
    mut rng: ResMut<RandomSource>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        paused.0 = !paused.0;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        grid.randomize(&mut rng.0);
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        grid.clear();
    }
}

fn mouse_paint(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut grid: ResMut<LifeGrid>,
) {
    let painting = buttons.pressed(MouseButton::Left);
    let erasing = buttons.pressed(MouseButton::Right);
    if !painting && !erasing {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera.single() else {
        return;
    };
    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    // Convert world position to grid coordinates
    let gx = ((world_pos.x + WINDOW_WIDTH / 2.0) / WINDOW_WIDTH * GRID_WIDTH as f32) as i32;
    let gy = ((WINDOW_HEIGHT / 2.0 - world_pos.y) / WINDOW_HEIGHT * GRID_HEIGHT as f32) as i32;

    // Paint with brush
    for dy in -BRUSH_RADIUS..=BRUSH_RADIUS {
        for dx in -BRUSH_RADIUS..=BRUSH_RADIUS {
            let nx = gx + dx;
            let ny = gy + dy;
            if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_HEIGHT as i32 {
                grid.set(nx as usize, ny as usize, painting);
            }
        }
    }
}

fn render_grid(
    grid: Res<LifeGrid>,
    handle: Res<GridImageHandle>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(image) = images.get_mut(&handle.0) else {
        return;
    };
    let data = image.data.as_mut().expect("Image has no CPU data");

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let idx = (y * GRID_WIDTH + x) * 4;
            let color = if grid.get(x, y) {
                CELL_ALIVE_COLOR
            } else {
                CELL_DEAD_COLOR
            };
            data[idx] = color[0];
            data[idx + 1] = color[1];
            data[idx + 2] = color[2];
            data[idx + 3] = color[3];
        }
    }
}

fn handle_quit(keyboard: Res<ButtonInput<KeyCode>>, mut app_exit: MessageWriter<AppExit>) {
    if keyboard.pressed(KeyCode::KeyQ) {
        app_exit.write(AppExit::Success);
    }
}
