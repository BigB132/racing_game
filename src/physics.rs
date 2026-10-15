use bevy::{math::Quat, prelude::*};
use crate::{Car, Wheel, Ground};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, (
                tick_physics,
                // log_data.run_if(on_timer(Duration::from_secs(1)))
        ))
            .insert_resource(Time::<Fixed>::from_hz(250.0));
            
    }
}


// constants 
pub const M: f32 = 1730.0;
pub const G: f32 = 9.81;
pub const H_COM: f32 = 0.3;
pub const I_XX: f32 = 2200.0;
pub const I_YY: f32 = 2400.0;
pub const I_ZZ: f32 = 550.0;
pub const P: f32 = 1.225;
pub const C_W: f32 = 0.30;
pub const A: f32 = 2.07;
pub const C_L: f32 = 0.3;

pub const K_FRONT: f32 = 35000.0;
pub const K_REAR: f32 = 30000.0;
pub const C_BUMP: f32 = 2800.0;
pub const C_REBOUND: f32 = 4200.0;
pub const L_REST: f32 = 0.45;
pub const K_ARB_FRONT: f32 = 5000.0;
pub const K_ARB_REAR: f32 = 3000.0;

pub const R: f32 = 0.37;
pub const I_W: f32 = 1.2;
pub const C_RR: f32 = 0.012;
pub const M2: f32 = 1.0;
pub const B: f32 = 10.0;
pub const C: f32 = 1.45;
pub const E: f32 = -0.6;
pub const V_MIN: f32 = 0.8;

pub const K_CRASH: f32 = 250000.0;
pub const C_CRASH: f32 = 15000.0;
pub const MU_WALL: f32 = 0.4;


pub const HALF_EXTENTS: Vec3 = Vec3::new(0.95, 0.65, 2.25);
pub const COLLIDER_POINTS: [Vec3; 10] = [
    Vec3::new(-HALF_EXTENTS.x, -0.1, HALF_EXTENTS.z),
    Vec3::new(HALF_EXTENTS.x, -0.1, HALF_EXTENTS.z),
    Vec3::new(-HALF_EXTENTS.x, -0.1, -HALF_EXTENTS.z),
    Vec3::new(HALF_EXTENTS.x, -0.1, -HALF_EXTENTS.z),

    Vec3::new(0.0, -0.1, HALF_EXTENTS.z),
    Vec3::new(0.0, -0.1, -HALF_EXTENTS.z),

    Vec3::new(HALF_EXTENTS.x, 0.0, 0.0),
    Vec3::new(-HALF_EXTENTS.x, 0.0, 0.0),

    Vec3::new(0.6, HALF_EXTENTS.y, -0.5),
    Vec3::new(-0.6, HALF_EXTENTS.y, 0.5)
];

pub const CAMERA_MODE: [Vec3; 3] = [
    Vec3::new(0.0, 4.0, -10.0),
    Vec3::new(4.0, 2.0, 0.0),
    Vec3::new(4.0, 0.1, 0.0),
];


fn tick_physics (
    mut wheels: Query<(&mut Wheel, &mut Transform), Without<Car>>,
    mut car: Single<&mut Car>,
    mut car_transform: Single<&mut Transform, With<Car>>,
    time: Res<Time>,
    mut ray_cast: MeshRayCast,
    ground: Query<Entity, With<Ground>>,
    mut cam_transform: Single<&mut Transform, (With<Camera>, Without<Car>, Without<Wheel>)>,
) {

    let mut front_left_x: f32 = 0.0;
    let mut front_right_x: f32 = 0.0;
    let mut rear_left_x: f32 = 0.0;
    let mut rear_right_x: f32 = 0.0;

    let mut front_left_f_raw: f32 = 0.0;
    let mut front_right_f_raw: f32 = 0.0;
    let mut rear_left_f_raw: f32 = 0.0;
    let mut rear_right_f_raw: f32 = 0.0;


    let p_rel_anchors: [Vec3; 4] = [
        Vec3::new(-0.75, 0.65,  1.25),
        Vec3::new( 0.75, 0.65,  1.25),
        Vec3::new(-0.78, 0.67, -1.25),
        Vec3::new( 0.78, 0.67, -1.25),
    ];
    

    let u_up: Vec3 = car.q_car * Vec3::new(0.0, 1.0, 0.0);
    let d_down: Vec3 = car.q_car * Vec3::new(0.0,-1.0, 0.0);
    let p_com: Vec3 = car_transform.translation + car.q_car * Vec3::new(0.0, H_COM, 0.0);


    for (mut wheel, mut wheel_transform) in &mut wheels {
        // suspension
        let p_anchor: Vec3;
        if wheel.front == 0 {
            if wheel.side == 0 {
                p_anchor = car_transform.translation + car.q_car * p_rel_anchors[0]; 
            } else {
                p_anchor = car_transform.translation + car.q_car * p_rel_anchors[1];
            }
        } else {
            if wheel.side == 0 {
                p_anchor = car_transform.translation + car.q_car * p_rel_anchors[2]
            } else {
                p_anchor = car_transform.translation + car.q_car * p_rel_anchors[3]
            }
        }
        
        // ray cast
        let ray_dir:Dir3 = Dir3::new(d_down).unwrap_or(Dir3::NEG_Y);
        let ray = Ray3d::new(p_anchor, ray_dir);
        let filter = |entity: Entity| ground.contains(entity);
        let hits = ray_cast.cast_ray(ray, &MeshRayCastSettings::default().with_filter(&filter));
        
        if let Some((_entity, hit_data)) = hits.first() {
            // println!("Hit entity {:?} at point {}", entity, hit_data.point);
            wheel.l_hit = hit_data.distance.min(L_REST + R) - R;
            wheel.d_norm = hit_data.normal;
            wheel.p_contact = hit_data.point;
        } else {
            // println!("Didn't hit any entity");
            wheel.l_hit = L_REST;
            wheel.d_norm = Vec3::new(0.0, 0.0, 0.0);
            wheel.p_contact = Vec3::new(0.0, 0.0, 0.0);
        }

        // update wheel pos and rot
        wheel_transform.translation = p_anchor + d_down * wheel.l_hit;
        
        wheel.x = (L_REST - wheel.l_hit).max(0.0);

        let v_anchor: Vec3 = car.v_car + car.w_car.cross(p_anchor - p_com);
        let v_spring: f32 = v_anchor.dot(d_down);
        
        let c = if v_spring >= 0.0 { C_BUMP } else { C_REBOUND };
        if wheel.front == 0 {
           let f_raw = if wheel.x > 0.0 { K_FRONT * wheel.x + c * v_spring } else { 0.0 };
            if wheel.side == 0 {
                front_left_x = wheel.x;
                front_left_f_raw = f_raw;
            } else {
                front_right_x = wheel.x;
                front_right_f_raw = f_raw;
            }
        } else {
            let f_raw = if wheel.x > 0.0 { K_REAR * wheel.x + c * v_spring } else { 0.0 };
            if wheel.side == 0 {
                rear_left_x = wheel.x;
                rear_left_f_raw = f_raw;
            } else {
                rear_right_x = wheel.x;
                rear_right_f_raw = f_raw;
            }
        }

        // Kinematics
        let f_car = car.q_car * Vec3::new(0.0, 0.0, 1.0);
        let r_car = car.q_car * Vec3::new(1.0, 0.0, 0.0);

        let f_steer = f_car * wheel.steer_angle.cos() + r_car * wheel.steer_angle.sin();
        let r_steer = r_car * wheel.steer_angle.cos() - f_car * wheel.steer_angle.sin();
        
        wheel.f_i = (f_steer - f_steer.dot(wheel.d_norm) * wheel.d_norm).normalize_or_zero();
        wheel.r_i = (r_steer - r_steer.dot(wheel.d_norm) * wheel.d_norm).normalize_or_zero();

        let v_contact = car.v_car + car.w_car.cross(wheel.p_contact - p_com);
        wheel.v_x = v_contact.dot(wheel.f_i);
        wheel.v_y = v_contact.dot(wheel.r_i);

        // Tires
        wheel.s = (wheel.w * R - wheel.v_x) / V_MIN.max(wheel.v_x.abs());
        wheel.a_atan = wheel.v_y.atan2(wheel.v_x.abs().max(V_MIN));
    }

    // Calculate everything that needs all data combined
    let f_arb_front: f32 = K_ARB_FRONT * (front_left_x - front_right_x);
    let f_arb_rear: f32 = K_ARB_REAR * (rear_left_x - rear_right_x);

    let front_left_f_z  = if front_left_x > 0.0  { (front_left_f_raw + f_arb_front).max(0.0) } else { 0.0 };
    let front_right_f_z = if front_right_x > 0.0 { (front_right_f_raw - f_arb_front).max(0.0) } else { 0.0 };
    let rear_left_f_z   = if rear_left_x > 0.0   { (rear_left_f_raw + f_arb_rear).max(0.0) }   else { 0.0 };
    let rear_right_f_z  = if rear_right_x > 0.0  { (rear_right_f_raw - f_arb_rear).max(0.0) }  else { 0.0 };

    let mut sum_f: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    let mut sum_r: Vec3 = Vec3::new(0.0, 0.0, 0.0);

    for (mut wheel, mut wheel_transform) in &mut wheels {
        if wheel.front == 0 {
            if wheel.side == 0 {
                wheel.f_z = front_left_f_z;
            } else {
                wheel.f_z = front_right_f_z;
            }
        } else {
            if wheel.side == 0 {
                wheel.f_z = rear_left_f_z;
            } else {
                wheel.f_z = rear_right_f_z;
            }
        }

        let d: f32 = M2 * wheel.f_z;
        let f_x0 = d * (C * (B * wheel.s - E * (B * wheel.s - (B * wheel.s).atan())).atan()).sin();
        let f_y0 = -d * (C * (B * wheel.a_atan - E * (B * wheel.a_atan - (B * wheel.a_atan).atan())).atan()).sin();

        let f_total = (f_x0 * f_x0 + f_y0 * f_y0).sqrt();
        
        wheel.f_x = f_x0 * (d / f_total.max(0.0001)).min(1.0);
        wheel.f_y = f_y0 * (d / f_total.max(0.0001)).min(1.0);

        wheel.f_roll = -C_RR * wheel.f_z * (wheel.v_x / V_MIN).clamp(-1.0, 1.0) * wheel.f_i;

        let w_free: f32 = wheel.w + ((wheel.t_drive - wheel.f_x * R) / I_W) * time.delta_secs();
        wheel.w = w_free.signum() * (w_free.abs() - (wheel.t_break.abs() / I_W) * time.delta_secs()).max(0.0);

        wheel.spin_angle = wheel.spin_angle + wheel.w * time.delta_secs() % std::f32::consts::TAU;

        let base_rot: Quat = if wheel.side == 0 { Quat::from_rotation_x(90.0_f32.to_radians()) } else { Quat::from_rotation_x(-90.0_f32.to_radians()) };

        wheel_transform.rotation = car.q_car * Quat::from_rotation_y(wheel.steer_angle) * Quat::from_rotation_x(wheel.spin_angle) * base_rot * Quat::from_rotation_z(-90.0_f32.to_radians());
        
        wheel.f = wheel.f_z * wheel.d_norm + wheel.f_x * wheel.f_i + wheel.f_y * wheel.r_i + wheel.f_roll;
        
        sum_f += wheel.f;
        sum_r += (wheel.p_contact - p_com).cross(wheel.f);
    }

    // detects walls and stuff
    for pt in COLLIDER_POINTS {
        let world_offset = car.q_car * pt;
        let max_dist = world_offset.length();

        if let Ok(ray_dir) = Dir3::new(world_offset) {
            let ray = Ray3d::new(p_com, ray_dir);
            let filter = |entity: Entity| ground.contains(entity);
            let hits = ray_cast.cast_ray(ray, &MeshRayCastSettings::default().with_filter(&filter));

            if let Some((_entity, hit_data)) = hits.first() {
                if hit_data.distance < max_dist {
                    let penetration = max_dist - hit_data.distance;
                    let n = hit_data.normal.normalize_or_zero();
                    let p_hit = hit_data.point;

                    let r_arm = p_hit - p_com;
                    let v_point = car.v_car + car.w_car.cross(r_arm);

                    let v_norm = v_point.dot(n);

                    let damping = if v_norm < 0.0 { -C_CRASH * v_norm } else { 0.0 };
                    let f_normal_mag = (K_CRASH * penetration + damping).max(0.0);
                    let f_normal = n * f_normal_mag;

                    let v_tangent = v_point - v_norm * n;
                    let f_friction = if v_tangent.length() > 0.01 { -v_tangent.normalize() * (f_normal_mag * MU_WALL) } else { Vec3::ZERO };

                    let f_crash = f_normal + f_friction;

                    sum_f += f_crash;
                    sum_r += r_arm.cross(f_crash);

                    if penetration > 0.05 {
                        car_transform.translation += n * (penetration - 0.05) * 0.5;
                    }
                }
            }
        }
    }

    let f_drag: Vec3 = -0.5 * P * C_W * A * car.v_car.length() * car.v_car;
    let f_down: Vec3 = -0.5 * P * C_L * A * car.v_car.length_squared() * u_up;
    let f_gravity: Vec3 = Vec3::new(0.0, -M * G, 0.0);

    sum_f += f_gravity + f_drag + f_down;
    let a_acc = sum_f / M;

    // Car velocity
    car.v_car += a_acc * time.delta_secs();
    // Car position
    let v_car = car.v_car;
    car_transform.translation += v_car * time.delta_secs();


    let r_car_mat = Mat3::from_quat(car.q_car);
    let i_inv = Mat3::from_diagonal(Vec3::new(1.0 / I_XX, 1.0 / I_YY, 1.0 / I_ZZ));
    let a_ang: Vec3 = (r_car_mat * i_inv * r_car_mat.transpose()) * sum_r;
    car.w_car += a_ang * time.delta_secs();
    let w_quat = Quat::from_xyzw(car.w_car.x, car.w_car.y, car.w_car.z, 0.0);
    car.q_car = (car.q_car + (w_quat * car.q_car) * (0.5 * time.delta_secs())).normalize();
    car_transform.rotation = car.q_car;

    cam_transform.translation = car_transform.translation + CAMERA_MODE[car.cam_mode];

    let rotated_offset = car.q_car * CAMERA_MODE[car.cam_mode];
    cam_transform.translation = car_transform.translation + rotated_offset;
    cam_transform.look_at(car_transform.translation, u_up);

}