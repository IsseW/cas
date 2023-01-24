use bevy::{ecs::event::Events, prelude::*, window::CursorGrabMode};
use bevy::{ecs::event::ManualEventReader, input::mouse::MouseMotion};

/// Keeps track of mouse motion events, pitch, and yaw
#[derive(Default, Resource)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

#[derive(Resource)]
/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

/// Used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_grab_mode(match window.cursor_grab_mode() {
        CursorGrabMode::None => CursorGrabMode::Confined,
        CursorGrabMode::Confined | CursorGrabMode::Locked => CursorGrabMode::None,
    });
    window.set_cursor_visibility(!window.cursor_visible());
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    toggle_grab_cursor(windows.get_primary_mut().unwrap());
}

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(mut commands: Commands) {
    let transform = Transform::from_xyz(-2.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y);
    let rotation = transform.rotation.to_euler(EulerRot::XYZ);
    commands.insert_resource(InputState {
        pitch: rotation.0,
        yaw: rotation.2,
        ..Default::default()
    });
    commands
        .spawn(Camera3dBundle {
            transform,
            projection: Projection::Perspective(PerspectiveProjection {
                near: 0.0001,
                far: 15.0,
                ..Default::default()
            }),
            ..Default::default()
        })
        .insert(FlyCam);
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    for (_camera, mut transform) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);
        let mut boost = 1.0;

        for key in keys.get_pressed() {
            if matches!(window.cursor_grab_mode(), CursorGrabMode::Confined) {
                match key {
                    KeyCode::W => velocity += forward,
                    KeyCode::S => velocity -= forward,
                    KeyCode::A => velocity -= right,
                    KeyCode::D => velocity += right,
                    KeyCode::Space => velocity += Vec3::Y,
                    KeyCode::LShift => velocity -= Vec3::Y,
                    KeyCode::LControl => boost *= 5.0,
                    _ => (),
                }
            }
        }

        velocity = velocity.normalize_or_zero();

        const SPEED: f32 = 2.0;
        transform.translation += velocity * time.delta_seconds() * settings.speed * SPEED * boost;
    }
}

fn reset_position(
    input: Res<Input<KeyCode>>,
    mut input_state: ResMut<InputState>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if input.pressed(KeyCode::B) {
        for (_camera, mut transform) in query.iter_mut() {
            *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);
            let rotation = transform.rotation.to_euler(EulerRot::XYZ);
            input_state.pitch = rotation.0;
            input_state.yaw = rotation.2;
        }
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    mut windows: ResMut<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    let window = windows.get_primary_mut().unwrap();
    if matches!(window.cursor_grab_mode(), CursorGrabMode::Confined) {
        for (_camera, mut transform) in query.iter_mut() {
            let mut new_yaw = state.yaw;
            let mut new_pitch = state.pitch;

            for ev in state.reader_motion.iter(&motion) {
                // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                let window_scale = window.height().min(window.width());
                const SENSITIVITY: f32 = 0.2;
                new_pitch -= (settings.sensitivity * SENSITIVITY * ev.delta.y * window_scale).to_radians();
                new_yaw -= (settings.sensitivity * SENSITIVITY * ev.delta.x * window_scale).to_radians();
            }
            new_pitch = new_pitch.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            transform.rotation =
                Quat::from_axis_angle(Vec3::Y, new_yaw) * Quat::from_axis_angle(Vec3::X, new_pitch);
            state.pitch = new_pitch;
            state.yaw = new_yaw;
        }
        window.set_cursor_position(Vec2::new(window.width() / 2.0, window.height() / 2.0));
    }
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab_cursor(window);
    }
}

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            .add_startup_system(setup_player)
            .add_startup_system(initial_grab_cursor)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(reset_position)
            .add_system(cursor_grab);
    }
}

/// Same as `PlayerPlugin` but does not spawn a camera
pub struct NoCameraPlayerPlugin;
impl Plugin for NoCameraPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MovementSettings>()
            .add_startup_system(initial_grab_cursor)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(reset_position)
            .add_system(cursor_grab);
    }
}
