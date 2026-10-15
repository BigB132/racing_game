use bevy::{prelude::*, scene::ResolvedSceneRoot};


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

#[derive(Component)]
pub struct Camera;

#[derive(Component, Default)]
pub struct Car {
    pub v_car: Vec3,
    pub q_car: Quat,
    pub w_car: Vec3,
    pub r_car: Vec3,
    pub cam_mode: usize,
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
    pub spin_angle: f32,
}

#[derive(Component)]
pub struct Ground;

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut asset_server: Res<AssetServer>,
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
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("car/car.glb"))),
        Transform::from_xyz(0.0, 5.0, 0.0),
    ));

    let wheels: [[f32; 6]; 4] = [
        // side, front, x, y, z, rot
        [0.0, 0.0, 0.8, 2.0, 0.9, 90.0],
        [1.0, 0.0, -0.8, 2.0, 0.9, -90.0],
        [0.0, 1.0, 0.8, 2.0, -0.9, 90.0],
        [1.0, 1.0, -0., 2.0, -0.9, -90.0],
    ];

    for wheel in wheels {
        commands.spawn((
            SceneEntity,
            Wheel {
                side: wheel[0] as u8,
                front: wheel[1] as u8,
                ..default()
            },
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("car/wheel.glb"))),
            Transform::from_xyz(wheel[2], wheel[3], wheel[4]).with_rotation(Quat::from_rotation_y(wheel[5].to_radians())),
        ));
    }

    // // FL
    // commands.spawn((
    //     SceneEntity,
    //     Wheel {
    //         side: 0,
    //         front: 0,
    //         ..default()
    //     },
    //     Mesh3d(meshes.add(Cylinder::new(R, 0.4))),
    //     MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
    //     Transform::from_xyz(1.0, 0.5, 1.0).with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
    // ));

    // // FR
    // commands.spawn((
    //     SceneEntity,
    //     Wheel {
    //         side: 1,
    //         front: 0,
    //         ..default()
    //     },
    //     Mesh3d(meshes.add(Cylinder::new(R, 0.4))),
    //     MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
    //     Transform::from_xyz(-1.0, 0.5, 1.0).with_rotation(Quat::from_rotation_z(-90.0_f32.to_radians())),
    // ));

    // // RL
    // commands.spawn((
    //     SceneEntity,
    //     Wheel {
    //         side: 0,
    //         front: 1,
    //         ..default()
    //     },
    //     Mesh3d(meshes.add(Cylinder::new(R, 0.4))),
    //     MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
    //     Transform::from_xyz(1.0, 0.5, -1.0).with_rotation(Quat::from_rotation_z(90.0_f32.to_radians())),
    // ));

    // // RR
    // commands.spawn((
    //     SceneEntity,
    //     Wheel {
    //         side: 1,
    //         front: 1,
    //         ..default()
    //     },
    //     Mesh3d(meshes.add(Cylinder::new(R, 0.4))),
    //     MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.1))),
    //     Transform::from_xyz(-1.0, 0.5, -1.0).with_rotation(Quat::from_rotation_z(-90.0_f32.to_radians())),
    // ));

    // Camera
    commands.spawn((
        Camera,
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
    mut car: Single<&mut Car>,
    scene_entities: Query<Entity, With<SceneEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    if keys.pressed(KeyCode::KeyW) {
        for mut wheel in &mut wheels {
            wheel.t_drive = 800.0;
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
        for entity in &scene_entities {
            commands.entity(entity).despawn();
        }
        setup_scene(commands, meshes, materials, asset_server);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        car.cam_mode = (car.cam_mode + 1) % 3
    }
}