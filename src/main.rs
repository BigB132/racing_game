use bevy::{prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (
            wheel_physics,
        ))
        // .add_systems(Update, move_player)
        .run();
}

#[derive(Component)]
struct Player {
    speed: f32,
}

#[derive(Component)]
struct Car;

#[derive(Component)]
struct Wheel {
    spring_constant: f64,
    suspension_constant: f64,
    l_rest: f64,
    l_hit: f64,
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground Plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.25, 0.45, 0.25))),
        Transform::default(),
    ));

    // Player
    commands.spawn((
        Player { speed: 8.0 },
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.5, 5.0),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Sun
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn wheel_physics(
    wheels: Query<&Wheel>,
) {
    for wheel in wheels {
        // Per wheel physics
    }
}