use bevy::{prelude::*, window::PrimaryWindow};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(PreUpdate, track_mouse_position);
    }
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component, Default, Debug)]
pub struct MousePositionTracker {
    pub mouse_world_position: Option<Vec2>,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
        MousePositionTracker::default(),
    ));
}

fn track_mouse_position(
    mut q_camera: Query<(&Camera, &GlobalTransform, &mut MousePositionTracker)>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    for (camera, camera_transform, mut mouse_position_tracker) in &mut q_camera {
        mouse_position_tracker.mouse_world_position = q_windows
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor));
    }
}
