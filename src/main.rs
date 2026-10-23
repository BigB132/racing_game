use bevy::prelude::*;
use avian3d::prelude::*;


mod physics;

use physics::PhysicsPlugin;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugin)
        .add_plugins((
            PhysicsPlugins::default(),
        ))
        .add_systems(Startup, setup_scene)
        .add_systems(Update, controls)
        .add_observer(spawn_car)
        .run();
}

#[derive(Event)]
pub struct MapReady;

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
    pub t_drive: f32,
    pub t_break: f32,
    pub steer_angle: f32, // rear wheels have 0 and front wheels around 40
    pub spin_angle: f32,
}

#[derive(Component)]
pub struct Drivetrain {
    pub throttle: f32,
    pub current_gear: i32,
    pub gear_ratios: [f32; 10],
    pub final_drive: f32,
    pub drivetrain_effeciency: f32,

    pub rpm: f32,
    pub idle_rpm: f32,
    pub max_rpm: f32,
    pub rev_limiter: bool,

    pub shift_timer: f32,
    pub shift_duration: f32,
    pub shift_cooldown: f32,
    pub clutch: f32,
}

impl Default for Drivetrain {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            current_gear: 1,
            gear_ratios: [-3.99, 0.0, 4.89, 3.17, 2.15, 1.56, 1.18, 0.94, 0.76, 0.61],
            final_drive: 3.7,
            drivetrain_effeciency: 0.9,
            rpm: 900.0,
            idle_rpm: 900.0,
            max_rpm: 7200.0,
            rev_limiter: false,
            shift_timer: 0.0,
            shift_duration: 0.12,
            shift_cooldown: 0.6,
            clutch: 1.0
        }
    }
}

#[derive(Component)]
pub struct Ground;

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
        
        // Camera
        commands.spawn((
            Camera,
            Camera3d::default(),
            Transform::from_xyz(10.0, 5.0, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
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

        commands.spawn((
            Ground,
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("race_track.glb"))),
            Transform::default(),
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            RigidBody::Static,
        )).observe(
            |trigger: On<ColliderConstructorHierarchyReady>,
             children_query: Query<&Children>,
             mut commands: Commands| {
                info!("Map ready");
                for child in children_query.iter_descendants(trigger.entity) {
                    commands.entity(child).insert(Ground);
                }
                commands.trigger(MapReady);
            },
        );

}

fn spawn_car (
    _: On<MapReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    // Car
    commands.spawn((
        SceneEntity,
        Car::default(),
        Drivetrain::default(),
        Player,
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("car/car.glb"))),
        Transform::from_xyz(0.0, 3.0, 0.0),
    ));
    
    let wheels: [[f32; 6]; 4] = [
        // side, front, x, y, z, rot
        [0.0, 0.0, 0.8, 2.0, 0.9, 90.0],
        [1.0, 0.0, -0.8, 2.0, 0.9, -90.0],
        [0.0, 1.0, 0.8, 2.0, -0.9, 90.0],
        [1.0, 1.0, -0.8, 2.0, -0.9, -90.0],
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
}

fn controls(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheels: Query<&mut Wheel>,
    mut car: Single<&mut Car>,
    mut drivetrain: Single<&mut Drivetrain>,
    scene_entities: Query<Entity, With<SceneEntity>>,
    time: Res<Time>,
) {
    if keys.pressed(KeyCode::KeyW) {
        drivetrain.throttle = 1.0;
    } else {
        drivetrain.throttle = 0.0;
    }
    
    // let brake_torque = if keys.pressed(KeyCode::KeyS) { 800.0 } else { 0.0 };
    // for mut wheel in &mut wheels {
    //     wheel.t_break = brake_torque;
    // }

    let braking = keys.pressed(KeyCode::KeyS);
    for mut wheel in &mut wheels {
        wheel.t_break = if braking {
            if wheel.front == 0 { 2600.0 } else { 1800.0 }
        } else {
            0.0
        };
    }

    let speed = car.v_car.length();
    let v_local = car.q_car.inverse() * car.v_car;
    let slip_angle = if speed > 2.0 {
        v_local.x.atan2(v_local.z.max(1.0)).clamp(-0.45, 0.45)
    } else {
        0.0
    };

    let max_steer = (35.0_f32.to_radians() / (1.0 + speed * 0.045)).max(8.0_f32.to_radians());

    let raw_input = if keys.pressed(KeyCode::KeyA) {
        1.0
    } else if keys.pressed(KeyCode::KeyD) {
        -1.0
    } else {
        0.0
    };

    let target_front_steer = (raw_input * max_steer + slip_angle * 0.55).clamp(-35.0_f32.to_radians(), 35.0_f32.to_radians());

    let ras_factor = if speed < 15.0 {
        -0.07
    } else {
        ((speed - 15.0) * 0.01).min(0.05)
    };

    let target_rear_steer = (target_front_steer * ras_factor).clamp(-2.8_f32.to_radians(), 1.8_f32.to_radians());

    for mut wheel in &mut wheels {
        let target = if wheel.front == 0 { target_front_steer } else { target_rear_steer };
        wheel.steer_angle = wheel.steer_angle.lerp(target, 14.0 * time.delta_secs());
    }
    
    if keys.just_pressed(KeyCode::KeyR) {
        for entity in &scene_entities {
            commands.entity(entity).despawn();
        }
        commands.trigger(MapReady);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        car.cam_mode = (car.cam_mode + 1) % 3
    }
}