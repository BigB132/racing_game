use bevy::prelude::*;


mod physics;

use physics::PhysicsPlugin;

use crate::physics::R;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Update, controls)
        .run();
}

#[derive(Component)]
pub struct SceneEntity;

#[derive(Component)]
pub struct Player;

#[derive(Component, Default)]
    pub struct Car {
    pub v_car: Vec3,
    pub q_car: Quat,
    pub w_car: Vec3,
    pub r_car: Vec3,
}

#[derive(Component, Default)]
pub struct Wheel {
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
    pub l_hit: f32,
    pub p_contact: Vec3,
    pub d_norm: Vec3,
    pub t_drive: f32, // if player wants to acc set to 300
    pub t_break: f32, // if player wants to break set 800
    pub steer_angle: f32, // rear wheels have 0 and front wheels around 40
}

#[derive(Component)]
pub struct Ground;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground Plane
    commands.spawn((
        SceneEntity,
        Ground,
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.25, 0.45, 0.25))),
        Transform::default(),
    ));

    // Car
    commands.spawn((
        SceneEntity,
        Car::default(),
        Player,
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // FL
    commands.spawn((
        SceneEntity,
        Wheel {
            side: 0,
            front: 0,
            ..default()
        },
        Mesh3d(meshes.add(Cylinder::new(R, 0.4))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(1.0, 0.5, 1.0).with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
    ));

    // FR
    commands.spawn((
        SceneEntity,
        Wheel {
            side: 1,
            front: 0,
            ..default()
        },
        Mesh3d(meshes.add(Cylinder::new(R, 0.4))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(-1.0, 0.5, 1.0).with_rotation(Quat::from_rotation_z(-90.0_f32.to_radians())),
    ));

    // RL
    commands.spawn((
        SceneEntity,
        Wheel {
            side: 0,
            front: 1,
            ..default()
        },
        Mesh3d(meshes.add(Cylinder::new(R, 0.4))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(1.0, 0.5, -1.0).with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
    ));

    // RR
    commands.spawn((
        SceneEntity,
        Wheel {
            side: 1,
            front: 1,
            ..default()
        },
        Mesh3d(meshes.add(Cylinder::new(R, 0.4))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
        Transform::from_xyz(-1.0, 0.5, -1.0).with_rotation(Quat::from_rotation_z(-90.0_f32.to_radians())),
    ));

    // Camera
    commands.spawn((
        SceneEntity,
        Camera3d::default(),
        Transform::from_xyz(10.0, 5.0, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
    ));

    // Sun
    commands.spawn((
        SceneEntity,
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn controls(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheels: Query<&mut Wheel>,
    scene_entities: Query<Entity, With<SceneEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,) {
    if keys.pressed(KeyCode::KeyW) {
        for mut wheel in &mut wheels {
            wheel.t_drive = 300.0;
        }
    } else {
        for mut wheel in &mut wheels {
            wheel.t_drive = 0.0;
        }
    }
    if keys.pressed(KeyCode::KeyS) {
        for mut wheel in &mut wheels {
            wheel.t_break = 800.0;
        }
    } else {
        for mut wheel in &mut wheels {
            wheel.t_break = 0.0;
        }
    }
    if keys.pressed(KeyCode::KeyA) {
        for mut wheel in &mut wheels {
            if wheel.front == 0 {
                wheel.steer_angle = 40.0_f32.to_radians();
            }
        }
    } else if keys.pressed(KeyCode::KeyD) {
        for mut wheel in &mut wheels {
            if wheel.front == 0 {
                wheel.steer_angle = -40.0_f32.to_radians();
            }
        }
    } else {
        for mut wheel in &mut wheels {
            wheel.steer_angle = 0.0_f32.to_radians();
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        if keys.just_pressed(KeyCode::KeyR) {
            for entity in &scene_entities {
                commands.entity(entity).despawn();
            }
            setup_scene(commands, meshes, materials);
        }
    }
}