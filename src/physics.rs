use bevy::prelude::*;
use crate::Wheel;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, tick_physics);
    }
}

// constants 
pub const M: f32 = 1450.0;
pub const G: f32 = 9.81;
pub const H_COM: f32 = 0.48;
pub const I_XX: f32 = 2200.0;
pub const I_YY: f32 = 2400.0;
pub const I_ZZ: f32 = 550.0;
pub const P: f32 = 1.225;
pub const C_W: f32 = 0.31;
pub const A: f32 = 2.2;
pub const C_L: f32 = 0.05;

pub const K_FRONT: f32 = 35000.0;
pub const K_REAR: f32 = 30000.0;
pub const C_BUMP: f32 = 2800.0;
pub const C_REBOUND: f32 = 4200.0;
pub const L_REST: f32 = 0.45;
pub const K_ARB_FRONT: f32 = 5000.0;
pub const K_ARB_REAR: f32 = 3000.0;

pub const R: f32 = 0.32;
pub const M_WHEEL: f32 = 20.0;
pub const I_W: f32 = 1.2;
pub const C_RR: f32 = 0.012;
pub const m: f32 = 1.0;
pub const B: f32 = 10.0;
pub const C: f32 = 1.45;
pub const E: f32 = -0.6;
pub const V_MIN: f32 = 0.8;

fn tick_physics (
    mut wheels: Query<&mut Wheel>,
    time: Res<Time>,
) {

    let mut front_left_x: f32 = 0.0;
    let mut front_right_x: f32 = 0.0;
    let mut rear_left_x: f32 = 0.0;
    let mut rear_right_x: f32 = 0.0;

    let mut front_left_f_raw: f32 = 0.0;
    let mut front_right_f_raw: f32 = 0.0;
    let mut rear_left_f_raw: f32 = 0.0;
    let mut rear_right_f_raw: f32 = 0.0;

    // only placeholders
    let t_drive: f32 = 1.0;
    let t_break: f32 = 1.0;
    let w: f32 = 1.0;
    let mut v_car: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    let mut p_car: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    let mut r_car: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    let mut w_car: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    let mut q_car: Quat = Vec3::new(0.0, 0.0, 0.0);
    let u_up: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    let d_norm: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    let p_contact: Vec3 = Vec3::new(1.0, 1.0, 1.0);
    let p_com: Vec3 = Vec3::new(1.0, 1.0, 1.0);


    for mut wheel in &mut wheels {

        // only placeholders
        let l_hit = 0.0;
        let p_anchor: Vec3 = Vec3::new(1.0, 1.0, 1.0);
        let w: f32 = 1.0;
        let d_down: Vec3 = wheel.car.q_car * Vec3::new(0.0,-1.0, 0.0);


        // Suspension
        wheel.x = (L_REST - l_hit).max(0.0);

        let v_anchor: Vec3 = v_car + w_car.cross(p_anchor - p_com);
        let v_spring: f32 = v_anchor.dot(d_down);
        
        let f_raw: f32;
        if wheel.front == 0 {
            let c = if v_spring >= 0.0 { C_BUMP } else { C_REBOUND };
            f_raw = K_FRONT * wheel.x + c * v_spring;
            if wheel.side == 0 {
                front_left_x = wheel.x;
                front_left_f_raw = f_raw;
            } else {
                front_right_x = wheel.x;
                front_right_f_raw = f_raw;
            }
        } else {
            f_raw = K_REAR * wheel.x + C * v_spring;
            if wheel.side == 0 {
                rear_left_x = wheel.x;
                rear_left_f_raw = f_raw;
            } else {
                rear_right_x = wheel.x;
                rear_right_f_raw = f_raw;
            }
        }

        
        // Kinematics
        let f_car = wheel.car.q_car * Vec3::new(0.0, 0.0, 1.0);
        let r_car = wheel.car.q_car * Vec3::new(1.0, 0.0, 0.0);

        let f_steer = f_car * time.delta_secs().cos() + r_car * time.delta_secs().sin();
        let r_steer = r_car * time.delta_secs().cos() - f_car * time.delta_secs().sin();
        
        wheel.f_i = (f_steer - f_steer.dot(d_norm) * d_norm).normalize();
        wheel.r_i = (r_steer - r_steer.dot(d_norm) * d_norm).normalize();

        let v_contact = v_car + w_car.cross(p_contact - p_com);
        wheel.v_x = v_contact.dot(wheel.f_i);
        wheel.v_y = v_contact.dot(wheel.r_i);

        // Tires
        wheel.s = (w * R - wheel.v_x) / V_MIN.max(wheel.v_x.abs());
        wheel.a_atan = wheel.v_y.atan2(wheel.v_x.abs().max(V_MIN));
    }

    // Calculate everything that needs all data combined
    let f_arb_front: f32 = K_ARB_FRONT * (front_left_x - front_right_x);
    let f_arb_rear: f32 = K_ARB_REAR * (rear_left_x - rear_right_x);

    let front_left_f_z: f32 = (front_left_f_raw + f_arb_front).max(0.0);
    let front_right_f_z: f32 = (front_right_f_raw - f_arb_front).max(0.0);
    let rear_left_f_z: f32 = (rear_left_f_raw + f_arb_rear).max(0.0);
    let rear_right_f_z: f32 = (rear_right_f_raw - f_arb_rear).max(0.0);

    let mut sum_f: Vec3 = Vec3::new(0.0, 0.0, 0.0);
    let mut sum_r: Vec3 = Vec3::new(0.0, 0.0, 0.0);

    for mut wheel in &mut wheels {
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

        let d: f32 = m * wheel.f_z;
        let f_x0 = d * (C * (B * wheel.s - E * (B * wheel.s - (B * wheel.s).atan())).atan()).sin();
        let f_y0 = -d * (C * (B * wheel.a_atan - E * (B * wheel.a_atan - (B * wheel.a_atan).atan())).atan()).sin();

        let f_total = (f_x0 * f_x0 + f_y0 * f_y0).sqrt();
        
        wheel.f_x = f_x0 * (d / f_total.max(0.0001)).min(1.0);
        wheel.f_y = f_y0 * (d / f_total.max(0.0001)).min(1.0);

        wheel.f_roll = -C_RR * wheel.f_z * (wheel.v_x / V_MIN).clamp(-1.0, 1.0) * wheel.f_i;

        let w_free: f32 = w + ((t_drive - wheel.f_x * R) / I_W) * time.delta_secs();
        wheel.w = w_free.signum() * (w_free.abs() - (t_break / I_W) * time.delta_secs()).max(0.0);
        
        wheel.f = wheel.f_z * d_norm + wheel.f_x * wheel.f_i + wheel.f_y * wheel.r_i + wheel.f_roll;
        sum_f += wheel.f;
        sum_r += (p_contact - p_com).cross(wheel.f);
    }

    let f_drag: Vec3 = -0.5 * P * C_W * A * v_car.length() * v_car;
    let f_down: Vec3 = -0.5 * P * C_L * A * v_car.length_squared() * u_up;
    let f_gravity: Vec3 = Vec3::new(0.0, -M * G, 0.0);

    sum_f += f_gravity + f_drag + f_down;
    let a_acc = sum_f / M;
    // Car velocity
    v_car += a_acc * time.delta_secs();
    // Car position
    p_car += v_car * time.delta_secs();


    let r_car_mat = Mat3::from_quat(q_car);
    let i_inv = Mat3::from_diagonal(Vec3::new(1.0 / I_XX, 1.0 / I_YY, 1.0 / I_ZZ));
    let a_ang: Vec3 = (r_car_mat * i_inv * r_car_mat.transpose()) * sum_r;
    w_car += a_ang * time.delta_secs();
    let w_quat = Quat::from_xyzw(w_car.x, w_car.y, w_car.z, 0.0);
    q_car = (q_car + (0.5 * time.delta_secs()) * (w_quat * q_car)).normalize();

}

