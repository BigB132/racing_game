use std::{f32::consts::TAU, time::Duration};

use bevy::{math::Quat, prelude::*, time::common_conditions::on_timer};
use crate::{Car, Drivetrain, Gear, Ground, RPM, Wheel};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(FixedUpdate, (
                update_drivetrain,
                tick_physics,
                log_data.run_if(on_timer(Duration::from_secs(1)))
            ).chain())
            .insert_resource(Time::<Fixed>::from_hz(250.0));
            
    }
}


// constants 
pub const M: f32 = 1715.0;
pub const G: f32 = 9.81;
pub const H_COM: f32 = 0.22;
pub const I_XX: f32 = 2400.0;
pub const I_YY: f32 = 2750.0;
pub const I_ZZ: f32 = 620.0;
pub const P: f32 = 1.225;
pub const C_W: f32 = 0.31;
pub const A: f32 = 2.07;
pub const C_L: f32 = 0.42;

pub const K_FRONT: f32 = 65000.0;
pub const K_REAR: f32 = 90000.0;
pub const C_BUMP: f32 = 4800.0;
pub const C_REBOUND: f32 = 6800.0;
pub const L_REST: f32 = 0.3;
pub const K_ARB_FRONT: f32 = 18000.0;
pub const K_ARB_REAR: f32 = 22000.0;

pub const R: f32 = 0.36;
pub const I_W: f32 = 1.6;
pub const C_RR: f32 = 0.012;
pub const MU_TIRE: f32 = 1.35;
pub const B_LONG: f32 = 13.0;
pub const C_LONG: f32 = 1.60;
pub const B_LAT: f32 = 15.5;
pub const C_LAT: f32 = 1.40;
pub const E: f32 = -0.4;
pub const V_MIN: f32 = 0.5;

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

// anchors zu niedrig / hoch ???
    let p_rel_anchors: [Vec3; 4] = [
        Vec3::new(-0.76, 0.55,  1.22),
        Vec3::new( 0.76, 0.55,  1.22),
        Vec3::new(-0.79, 0.57, -1.22),
        Vec3::new( 0.79, 0.57, -1.22),
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
            // println!("hit entity {:?} at point {}", entity, hit_data.point);
            wheel.l_hit = hit_data.distance.min(L_REST + R) - R;
            wheel.d_norm = hit_data.normal;
            wheel.p_contact = hit_data.point;
        } else {
            // println!("didn't hit any entity");
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

    for (mut wheel, _wheel_transform) in &mut wheels {
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

        let tire_grip_mult = if wheel.front == 0 { 1.0 } else { 1.08 };
        let d: f32 = MU_TIRE * tire_grip_mult * wheel.f_z;
        
        let current_slip = (wheel.w * R - wheel.v_x) / V_MIN.max(wheel.v_x.abs());
        let max_slip_allowed = 0.12;
        if current_slip > max_slip_allowed && wheel.t_drive > 0.0 {
            wheel.t_drive *= (max_slip_allowed / current_slip).clamp(0.1, 1.0);
        }

        let effective_brake = if current_slip < -0.14 {
            wheel.t_break * 0.25
        } else {
            wheel.t_break
        };

        let w_free: f32 = wheel.w + ((wheel.t_drive - wheel.f_x * R) / I_W) * time.delta_secs();
        wheel.w = w_free.signum() * (w_free.abs() - (effective_brake.abs() / I_W) * time.delta_secs()).max(0.0);

        if wheel.f_z <= 0.0 && wheel.t_drive <= 0.0 {
            wheel.w = wheel.w.lerp(wheel.v_x / R, 5.0 * time.delta_secs());
        }

        wheel.s = ((wheel.w * R - wheel.v_x) / V_MIN.max(wheel.v_x.abs())).clamp(-1.0, 1.0);
        wheel.a_atan = wheel.v_y.atan2(wheel.v_x.abs().max(V_MIN));

        let bx = B_LONG * wheel.s;
        let f_x0 = d * (C_LONG * (bx - E * (bx - bx.atan())).atan()).sin();

        let by = B_LAT * wheel.a_atan;
        let f_y0 = -d * (C_LAT * (by - E * (by - by.atan())).atan()).sin();

        let f_total = ((f_x0 * 0.85).powi(2) + f_y0.powi(2)).sqrt();
        let scale = (d / f_total.max(0.0001)).min(1.0);

        wheel.f_x = f_x0 * scale;
        wheel.f_y = f_y0 * scale;
        
        if wheel.v_x.abs() < V_MIN {
            let low_speed_damp = (-wheel.v_y * 4500.0).clamp(-d, d);
            wheel.f_y = wheel.f_y.lerp(low_speed_damp, 1.0 - (wheel.v_x.abs() / V_MIN));
        }

        wheel.f_roll = -C_RR * wheel.f_z * (wheel.v_x / V_MIN).clamp(-1.0, 1.0) * wheel.f_i;
        wheel.spin_angle = (wheel.spin_angle + wheel.w * time.delta_secs()).rem_euclid(TAU);

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

    car.v_car += a_acc * time.delta_secs();
    let v_car = car.v_car;
    car_transform.translation += v_car * time.delta_secs();

    let r_car_mat = Mat3::from_quat(car.q_car);
    let i_inv = Mat3::from_diagonal(Vec3::new(1.0 / I_XX, 1.0 / I_YY, 1.0 / I_ZZ));
    let a_ang: Vec3 = (r_car_mat * i_inv * r_car_mat.transpose()) * sum_r;
    car.w_car += a_ang * time.delta_secs();
    let w_quat = Quat::from_xyzw(car.w_car.x, car.w_car.y, car.w_car.z, 0.0);
    car.q_car = (car.q_car + (w_quat * car.q_car) * (0.5 * time.delta_secs())).normalize();
    car_transform.rotation = car.q_car;


    let new_d_down = car.q_car * Vec3::NEG_Y;
    for (wheel, mut wheel_transform) in &mut wheels {
        let anchor_idx = (wheel.front as usize) * 2 + (wheel.side as usize);
        let new_p_anchor = car_transform.translation + car.q_car * p_rel_anchors[anchor_idx];

        wheel_transform.translation = new_p_anchor + new_d_down * wheel.l_hit;

        let base_rot: Quat = if wheel.side == 0 {
            Quat::from_rotation_x(90.0_f32.to_radians())
        } else {
            Quat::from_rotation_x(-90.0_f32.to_radians())
        };

        wheel_transform.rotation = car.q_car
            * Quat::from_rotation_y(wheel.steer_angle)
            * Quat::from_rotation_x(wheel.spin_angle)
            * base_rot
            * Quat::from_rotation_z(-90.0_f32.to_radians());
    }


    cam_transform.translation = car_transform.translation + CAMERA_MODE[car.cam_mode];

    let rotated_offset = car.q_car * CAMERA_MODE[car.cam_mode];
    cam_transform.translation = car_transform.translation + rotated_offset;
    cam_transform.look_at(car_transform.translation, u_up);

}

pub fn update_drivetrain(
    time: Res<Time>,
    mut drivetrain: Single<&mut Drivetrain>,
    mut wheels: Query<&mut Wheel>,
    mut texts: Query<(&mut Text, Option<&RPM>, Option<&Gear>)>,
) {

    if drivetrain.shift_timer > 0.0 {
        drivetrain.shift_timer = (drivetrain.shift_timer - time.delta_secs()).max(0.0);
    }
    if drivetrain.shift_cooldown > 0.0 {
        drivetrain.shift_cooldown = (drivetrain.shift_cooldown - time.delta_secs()).max(0.0);
    }

    let mut rear_w_sum = 0.0;
    let mut front_w_sum = 0.0;

    for wheel in &wheels {
        if wheel.front != 0 {
            rear_w_sum += wheel.w;
        } else {
            front_w_sum += wheel.w;
        }
    }

    let avg_rear_w = rear_w_sum / 2.0;
    let avg_front_w = front_w_sum / 2.0;
    let avg_drive_w  =avg_rear_w * 0.68 + avg_front_w * 0.32;

    let gear_idx = (drivetrain.current_gear + 1) as usize;
    let gear_ratio = drivetrain.gear_ratios[gear_idx];
    let total_ratio = gear_ratio * drivetrain.final_drive;

    let raw_rpm = (avg_rear_w * total_ratio).abs() * (60.0 / std::f32::consts::TAU);

    let is_neutral = drivetrain.current_gear == 0 || drivetrain.shift_timer > 0.0;

    if is_neutral {
        let target_idle = drivetrain.idle_rpm + drivetrain.throttle * (drivetrain.max_rpm - drivetrain.idle_rpm);
        drivetrain.rpm += (target_idle - drivetrain.rpm) * 8.0 * time.delta_secs();
        drivetrain.clutch = 0.0;
    } else {
        if raw_rpm < drivetrain.idle_rpm {
            drivetrain.clutch = drivetrain.throttle.clamp(0.0, 1.0);
            let slip_rpm = drivetrain.idle_rpm + drivetrain.throttle * 2000.0;
            drivetrain.rpm = drivetrain.rpm.lerp(slip_rpm, 5.0 * time.delta_secs());
        } else {
            drivetrain.clutch = 1.0;
            drivetrain.rpm = raw_rpm;
        } 
    }

    if drivetrain.rpm >= drivetrain.max_rpm {
        drivetrain.rev_limiter = true;
        drivetrain.rpm = drivetrain.max_rpm;
    } else {
        drivetrain.rev_limiter = false;
    }

    if drivetrain.shift_timer <= 0.0 && drivetrain.current_gear > 0 {
        let upshift_rpm = drivetrain.max_rpm * 0.93;
        let downshift_rpm = 2200.0;

        if drivetrain.rpm >= upshift_rpm && drivetrain.current_gear < 8 {
            drivetrain.current_gear += 1;
            drivetrain.shift_timer = drivetrain.shift_duration;
            info!("gear up")
        } else if drivetrain.rpm < downshift_rpm && drivetrain.current_gear > 1 {
            drivetrain.current_gear -= 1;
            drivetrain.shift_timer = drivetrain.shift_duration;
            info!("gear down")
        }
    }

    for (mut text, is_rpm, is_gear) in &mut texts {
        if is_rpm.is_some() {
            text.0 = format!("RPM: {:.0}", raw_rpm);
        } else if is_gear.is_some() {
            text.0 = format!("Gear: {:.0}", drivetrain.current_gear)
        }
    }

    let t_engine_max = {
        let rpm_clamped: f32 = drivetrain.rpm.clamp(800.0, 7200.0);
        if rpm_clamped < 2500.0 {
            420.0 + (rpm_clamped - 800.0) * (380.0 / 1700.0)
        } else if rpm_clamped <= 4500.0 {
            800.0
        } else {
            800.0 - (rpm_clamped - 4500.0) * (160.0 / 2700.0)
        }
    };

    let t_engine = if drivetrain.rev_limiter || is_neutral {
        0.0
    } else if drivetrain.throttle > 0.05 {
        t_engine_max * drivetrain.throttle
    } else if avg_drive_w > 1.5 {
        -90.0 
    } else {
        0.0
    };

    let t_drive_total = t_engine * total_ratio * drivetrain.drivetrain_effeciency * drivetrain.clutch;
    
    let rear_slip_excess = (avg_rear_w - avg_front_w).max(0.0);
    let front_split = (0.32 + (rear_slip_excess * 0.015)).clamp(0.3, 0.45);
    let rear_split = 1.0 - front_split;

    for mut wheel in &mut wheels {
        if wheel.front == 0 {
            wheel.t_drive = (t_drive_total * front_split) / 2.0;
        } else {
            wheel.t_drive = (t_drive_total * rear_split) / 2.0;
        }
    }


}

pub fn log_data(
    drivetrain: Single<&Drivetrain>,
) {
    info!(drivetrain.current_gear);
}