# Racing game in Bevy

This game is using Bevy + Rust to run.
The physics are fully self made and build from ground up. The game engines phyiscs are not being used.  
AI was involved by explaining the physics to me, but didn't code a single line.  

Play here: [https://bigb132.github.io/racing_game](https://bigb132.github.io/racing_game) (might lag, which is why I recommend using the windows build to run)

# Phyiscs

This part of the file will explain how the physics work.

## Constants

The values below are constants, but may change slightly depending on the car you want to simulate.  
All names are uppercase due to rust naming conventions.

### Chassis, Mass & Inertia
|Name|Value|Description|
|---|---|---|
|M|1715 kg|The mass of the car.|
|G|9.81 m/s²|The Gravity of earth.|
|H_COM|0.22 m|The height of the center of mass of the car.|
|I_XX|2400.0|Pitch moment of inertia|
|I_YY|2750.0|Yaw moment of inertia|
|I_ZZ|620.0|Roll moment of inertia|

### Aerodynamics
|Name|Value|Description|
|---|---|---|
|P|1.225 kg/m³|The air density.|
|C_W|0.31|The aerodynamic drag coefficent.(aerodynamic efficency)|
|A|2.07 m²|The frontal surface area (for airflow).|
|C_L|0.42|The aerodynamic downforce coefficent. More velocity will result in a higher aerodynamic downward force.|

### Suspension & Anti-Roll Bars
|Name|Value|Description|
|---|---|---|
|K_FRONT|65000.0 N/m|The stiffness of the front suspension spring.|
|K_REAR|90000.0 N/m|The stiffness of the rear suspension spring.|
|C_BUMP|4800.0 N * m/s²|The suspension damper compression coefficent. Used when the spring compresses.|
|C_REBOUND|6800.0 N * m/s²|The suspension damper rebound coefficent. Used when the spring extends.|
|L_REST|0.3 m|The length of the suspension struts.|
|K_ARB_FRONT|18000.0 N/m|The front anti-roll bar stiffness.|
|K_ARB_REAR|22000.0 N/m|The rear anti-roll bar stiffness.|

### Wheels & Tires
|Name|Value|Description|
|---|---|---|
|R|0.37 m|The radius of the tire.|
|I_W|1.2 kg * m²|The rotational moment of inertia.|
|C_RR|0.012|The rolling resistance coefficent.|
|MU_TIRE|1.35|The peak tire friction coefficent.|
|B_LONG|10.0|The pacejka stiffness factor.|
|C_LONG|1.45|The pacejka shape factor.|
|B_LAT|15.5|Lateral Pacejka Stiffness|
|C_LAT|1.4|Lateral Pacejka shape of the grip curve|
|E|-0.4|The pacejka curvarure factor.|
|V_MIN|0.5 m/s|The low speed threshold to prevent game crashes when calculating slip angles and ratios at low speed.|

### Collision 
|Name|Value|Description|
|---|---|---|
|K_CRASH|250000.0 N/m|The penalty spring stiffness, which is used to push the car out of objects.|
|C_CRASH|15000.0 N * s/m|The damping coefficent for collision penetration.|
|MU_WALL|0.4|The coulomb friction coefficent between the car and the objects.|
|HALF_EXTENDS|Vec3(0.95, 0.65, 2.25)|The half-dimensions of the cars bounding boxes. (width/height/length)|
|COLLIDER_POINTS|10 Vectors|An array of 10 vectors on the chasis used to detect collisions.|
|CAMERA_MODE|3 Camera Poitions|The offsets of the camera relative to the car. There is a chase camera, a side camera, and low side camera.|

## Calculations

### References

Before we can start with calculating the actual physics, we have to calculate some references:

|Name|Formula|Description|Unknown Values|
|---|---|---|---|
|u_up|q_car * Vec3(0.0, 1.0, 0.0)|This will result in a new Vector3, but only keep the y value.|**q_car**: The rotation of the car|
|d_down|q_car * Vec3(0.0, -1.0, 0.0)|This will result in a new Vector3, but only keep the negative y value.|**q_car**: The rotation of the car|
|p_com|p_car + q_car * Vec3(0, H_COM, 0)|This calculates the position of the center of mass in the world coordinate system.|**p_car**: Position of the car in the world coordinate system. **q_car**: Rotation of the car.|

### Raycast Suspension

Everything here is per wheel (i):  
A ray gets cast down from p_anchor[i] along d_down. If it intersects with the ground, we get distance_of_ray  
We can then use that to calculate some important values:
|Name|Formula|Description|Unknown Values|
|---|---|---|---|
|l_hit|min(distance_of_ray, L_REST - R) - R|||
|d_norm||The 3D surface normal vector of the ground mesh at the hit point.|
|p_contact||The position in the world coordinate system, where the ray hit the ground.|
|p_anchor|p_car + q_car * p_rel_anchors[i]|This will calculate the position of the anchors in the world coordinate system per wheel.|**p_rel_anchors**: Position of the anchors relative to the cars position.|
|p_wheel|p_anchor + d_down * l_hit|The position of the wheel in the world coordinate system.||
|x|max(0, L_REST - l_hit)|The suspension compression distance||

### Springs

The following calculations are still per wheel.
|Name|Formula|Description|Unknown Values|
|---|---|---|---|
|v_anchor|v_car + w_car $\times$ (p_anchor - p_com)|The velocity of the suspension anchor.|**v_car**: The velocity vector of the car. **w_car**: The angular velocity vector.|
|v_spring|v_anchor $\cdot$ d_down|The suspension compression speed (basically how fast the car bumps up/down).||

If v_spring is bigger than 0, the suspension is being compressed and if it's smaller, the suspension extends. Depending on that we assign a value to **c**:  
**c = if v_spring >= 0: C_BUMP, if v_spring < 0: C_REBOUND**  
With c we can now calculate f_raw, the raw spring damper force:  
**f_raw = <K_FRONT|K_REAR> * x + c * v_spring**  
If the wheel is at the front you have to use K_FRONT, otherwise you should use K_REAR. If is x <= 0, f_raw is 0 aswell.

### Kinematics & Tires
|Name|Formula|Description|Unknown Values|
|---|---|---|---|
|f_car|q_car * Vec3(0,0,1)|The car body's forward vector.|**q_car**: The rotation of the car.|
|r_car|q_car * Vec3(1,0,0)|The car body's right vector.|**q_car**: The rotation of the car.|
|f_steer|f_car * steer_angle.cos() + r_car * steer_angle.sin()|Used to rotate the wheel relative to the car body.|**steer_angle**: How the steering angle is being calculated is explained in the controls part (below).|
|r_steer|r_car * steer_angle.cos() - f_car * wheel_angle.sin()|Used to rotate the wheel relative to the car body.|**steer_angle**: How the steering angle is being calculated is explained in the controls part (below).|
|f_i|normalize(f_steer - (f_steer * d_norm) * d_norm)|Using the vectors from above, we can project the vectors onto the ground plane.||
|r_i|normalize(_steer - (r_steer * d_norm) * d_norm)|Using the vectors from above, we can project the vectors onto the ground plane.||
|v_contact|v_car + w_car * (p_contact - p_com)|The ground contacts velocity.||´
|v_x|v_contact * f_i|The longitudinal speed of the wheel.||
|v_y|v_contact * r_i|The lateral speed of the wheel.||
|s|(w * R - v_x) / max(V.min, v_x.abs())|The longitudinal slip ratio.|**w**: The angular velocity vector.|
|a_atan|atan2(v_y, max(v_x.abs(), V_MIN))|Lateral slip angle.||


### Anti-Roll Bar   

Using the per wheel values we just got, we can calculate multiple values for the entire car. These need some of the wheels values. The wheel variables are declared like this: <front|rear>_<left|right>_<var_name>
|Name|Formula|Description|Unknown Values|
|---|---|---|---|
|f_arb_front|K_ARB_FRONT * (front_left_x - front_right_x)|The Anti-Roll bar resist force at the front.||
|f_arb_rear|K_ARB_REAR * (rear_left_x - rear_right_x)|The Anti-Roll bar resist force at the back.||
|front_left_f_z|max(front_left_f_raw + f_arb_front, 0)|Total vertical normal force on the front left tire.|
|front_right_f_z|max(front_right_f_raw - f_arb_front, 0)|Total vertical normal force on the front right tire.|
|rear_left_f_z|max(rear_left_f_raw + f_arb_rear, 0)|Total vertical normal force on the rear left tire.|
|rear_right_f_z|max(rear_right_f_raw - f_arb_rear, 0)|Total vertical normal force on the rear right tire.|

If the x value of a car is smaller than 0, the according f_z value will be zero aswell.

### Tires
These are being calculated per wheel again.
|Name|Formula|Descripton|Unknown Values|
|---|---|---|---|
|tire_grip_mult|if wheel_at_front: 1.0, else: 1.08|Using that multiplier we can get even more realistic brakes, as the rear wheel has 8% more peak friction.||
|d|MU_TIRE * tire_grip_mult * f_z|The peak available tire force.||
|current_slip|(w * R - v_x) / max(V_MIN, v_x.abs())|The current slip.|**w**:| // TODO: Explain W
|max_slip_allowed|0.12|This should be a constant. I dont know why it's down here lol.||

if current_slip is bigger than max_slip_allowed, we reduce the acceleration like this:  
**t_drive * = (max_slip_allowed / current_slip).clamp(0.1, 1)**  
Depending on current_slip, this can reduce the acceleration to 1/10.  
If the current slip is smaller than -0.14, we multiply t_break with 0.25.

|Name|Formula|Description|Unknown Values|
|---|---|---|---|
|w_free|w + ((t_drive - f_x * R) / I_W) * delta_time|The angular velocity of the wheel (unfinished).|**t_drive**: Driving torque, **delta_time**: The time that passed since the last iteration.|
|w|sgn(w_free) * max(w_free.abs() - (effective_brake.abs() / I_W) * delta_time, 0.0)|The final angular velocity of the wheel.|**effective_break**: Basically just t_break, but if current_slip is smaller than -0.14, it is t_break * 0.25, **delta_time**: The time that passed since the last iteration.|

|Name|Formula|Description|Unknown Values|
|---|---|---|---|
|bx|B_LONG * s|The longitudinal slip.||
|f_x0|d * (C_LONG * (bx - E * (bx - bx.atan())).atan()).sin()|The longitudinal force.||
|by|B_LAT * a_atan|The lateral slip.||
|f_y0|-d * (C_LAT * (by - E * (by - by.atan())).atan()).sin()|The lateral force.||
|f_total|((f_x0 * 0.85)² + f_y0²).sqrt()|The combined tire force.||
|scale|min((d / max(f_total, 0.0001)), 1)|The friction scaling factor.||
|f_x|f_x0 * scale|The final longitudinal force of the wheel.||
|f_y|f_y0 * scale|The final lateral force of the wheel.||
|f_roll|-C_RR * f_z * (v_x / V_MIN).clam p(-1, 1) * f_i|The rolling resistance.||
|spin_angle|(spin_angle + w * delta_time).rem_euclid(TAU)|The x rotation of the wheel (for the spinning animation).|**TAU**: Equal to 2π|
|f|f_z * d_norm + f_x * f_i + f_y * r_i + f_roll|The total force of the wheel.||

### Collision
In this part the collision forces are being calculated.  
The car has 10 collider points. Which are all getting checked like this:

|Name|Formula|Description|Unknown Values|
|---|---|---|---|
|world_offset|q_car * pt|Rotates the collider point into the cars direction.|**pt**: The collision point.|
|max_dist|world_offset.length()|The distance from p_com to the collider point.||g
|penetration|max_dist - hit_data.distance|If a ray is shorter than the distance to the collider point, it's inside the cars body.|**hit_data.distance**: The length of a ray.|
|n||The normal vector of the wall surface.||
|p_hit||The position of the hit point in the world coordinate system.||
|r_arm|p_hit - p_com|The lever arm vector.|**p_com**: Center of mass of the car.|
|v_point|v_car + w_car $\times$ r_arm|The exact velocity of the car's body at the collision point.||
|v_norm|v_point $\cdot$ n|The speed of the collision point along the wall normal.||
|damping|if v_norm < 0: -C_CRASH * v_norm, else: 0|The collision shock absorption.||
|f_normal_mag|max(K_CRASH * penetration + damping, 0)|The magnitude of the penalty spring damper force.||
|f_normal|n * f_normal_mag|The normal repulsion force to the wall.||
|v_tangent|v_point - v_norm * n|The impact velocity to slide along a wall.||
|f_friction|-v_tangent.normalize() * (f_normal_mag * MU_WALL)|The coulomb kinetic friction force.||
|f_crash|f_normal + f_friction|The total collision force.||


### Bringing everything together
This will finally calculate the actual position and rotation of the car and the wheels.
|Name|Formula|Description
|---|---|---|
|f_drag|-0.5 * P * C_W * A * v_car.length() * v_car|The aerodynamic drag.|
|f_down|-0.5 * P * C_L * A * v_car.length_squared() * u_up|The aerodynamic downforce.|
|f_gravity|Vec3(0, -M * G, 0)|The gravity.|

**sum_f = f_wheels + f_drag + f_down + f_gravity**  
In this case f_wheels is the sum of all the wheels f values.

|Name|Formula|Description|
|---|---|---|
|a_acc|sum_f / M||
|v_car|v_car + a_acc * delta_time|The velocity of the car.|
|p_car|p_car + v_car * delta_time|The position of the car in the world coordinate system.|

And finally the position and rotation the wheels (per wheel):

|Name|Formula|Description|
|---|---|---|
|new_p_anchor|car_translation + q_car * p_rel_anchor|**car_translation**: The position of the car in the world coordinate systen. **p_rel_anchor**: The anchor of the wheel relative to the car.|
|p_wheel|new_p_anchor + d_down * l_hit|The position of the wheel in the world coordinate system.||
|base_rot|if side == left: 90, else: -90|We need this, as the wheels on the right side look into the opposite direction than the wheel on the left side.||
|wheel_rot|q_car * y_rot(steer_angle) * x_rot(spin_angle) * base_rot * z_rot(-90)|The final wheel rotation.|**x_rot, y_rot, z_rot**: Helper functions to create an directional vector.|

# Building 

You can run it yourself using
```
cargo run
```

You can also build it yourself using

```
cargo build --release
```

Both will take care of installing all the deps.
