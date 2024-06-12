use bevy::prelude::*;
use bevy_third_person_camera::*;
//use crate::MainCamera;

pub struct CameraPlugin;
#[derive(Component)]
pub struct MainCamera;

impl Plugin for CameraPlugin{
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_camera_3d)
            .add_systems(Startup, spawn_camera_ui);
    }
}

fn spawn_camera_3d(mut commands: Commands)
{
    let camera = (
        Camera3dBundle {
            transform: Transform::from_xyz(10.0,10.0,10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        ThirdPersonCamera {
            mouse_sensitivity: 2.0,
            zoom: Zoom::new(0.0, 200.0),
            cursor_lock_toggle_enabled: true,
            cursor_lock_active: true,
            cursor_lock_key: KeyCode::Tab,
            ..default()
        },
        Name::new("3d cameras")
    );
    commands.spawn(camera).insert(MainCamera);


}

fn spawn_camera_ui(mut commands: Commands)
{
    let ui_camera = (
        UiCameraConfig{
            ..default()
        },
        Name::new("2d camera")
    );
    commands.spawn(ui_camera);
}