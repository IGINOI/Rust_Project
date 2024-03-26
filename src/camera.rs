use bevy::prelude::*;
use bevy_third_person_camera::*;

pub struct CameraPlugin;
#[derive(Component)]
pub struct MainCamera;

impl Plugin for CameraPlugin{
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, spawn_camera_3d);
    }
}

//spawn the 3d camera with the Third Person Camera component that follows the player during the simulation
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
            //With Tab you can unlock the cursor from the simulation and move it normally
            cursor_lock_key: KeyCode::Tab,
            ..default()
        },
        MainCamera,
        Name::new("3d cameras")
    );
    commands.spawn(camera);
}