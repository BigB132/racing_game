use bevy::{prelude::*};

mod physics;

use physics::PhysicsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugin)
        .add_systems(Startup, setup_scene)
        // .add_systems(Update, move_player)
        .run();
}


#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Car {
    pub v_car: f32,
    pub p_car: f32,
    pub q_car: f32,
}

#[derive(Component)]
pub struct Wheel {
    pub car: Car,
    pub side: u8,
    pub front: u8,
    pub x: f32,
    pub f_z: f32,
    pub s: f32,
    pub a_atan: f32,
    pub f_x: f32,
    pub f_y: f32,
    pub f_roll: Vec3,
    pub v_x: f32,
    pub v_y: f32,
    pub f_i: Vec3,
    pub r_i: Vec3,
    pub w: f32,
    pub f: Vec3,
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
        Player,
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