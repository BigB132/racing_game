This file includes all the formulas required to calculate the physics of a car.
AI did help me understanding the math behind it, but didn't write any of the text in this document nor any code.

# References

NAME | DEFAULT  

## General
M           1450kg  
g           9.81m/s^2  
h_com       0.48m  
I_xx        2200kg * m²  
I_yy        2400kg * m²  
I_zz        550kg * m²  
p           1.225kg / m³  
c_w         0.31  
A           2.2m²  
c_l         0.05  

## Per wheel
k_front     35000N/m  
k_rear      30000N/m  
c_bump      2800N*s/m  
c_rebound   4200N*s/m  
L_rest      0.45m  
k_arb_front 5000N/m  
k_arb_rear  3000N/m  

## Other
R           0.32m  
m_wheel     20kg  
I_w         1.2kg * m²  
c_rr        0.012  
m           1  
B           10  
C           1.45  
E           -0.6  
v_min       0.8  
  
# Calculating the physics of the car in general  
  
F_drag = -0.5 * p * c_w * A * length(v_car) * v_car  
F_down = -0.5 * p * c_l * A * length(v_car)² * u_up  
F_gravity = vec3(0, -M * g, 0)  
  
# Calculating the physics per wheel  
  
## Suspension  
x = max(0, L_rest - L_hit)  
v_anchor = v_car + cross(w_car, p_anchor - p_CoM)  
v_spring = dot(v_anchor, d_down)  
F_raw = k * x + c * v_spring  
  
F_arb_front = k_arb_front * (x_front_left - x_front_right)  
F_arb_rear  = k_arb_rear  * (x_rear_left  - x_rear_right)  
  
F_z_front_left  = max(0, F_raw_front_left  + F_arb_front) # 0 wenn x_front_left == 0  
F_z_front_right = max(0, F_raw_front_right - F_arb_front) # 0 wenn x_front_right == 0  
F_z_rear_left   = max(0, F_raw_rear_left   + F_arb_rear)  # 0 wenn x_rear_left == 0  
F_z_rear_right  = max(0, F_raw_rear_right  - F_arb_rear)  # 0 wenn x_rear_right == 0  
  
## Kinematics  
v_contact = v_car + cross(w_car, p_contact - p_CoM)  
v_x = dot(v_contact, f_i)  
v_y = dot(v_contact, r_i)  
  
## Tires  
s = (w * R - v_x) / max(abs(v_x), v_min)  
a = arctan2(v_y, max(abs(v_x), v_min))  
  
D = m * F_z  
F_pacejka(o, F_z) = D * sin(C * arctan(B * o - E * (B * o - arctan(B * o))))  
F_x0 = F_pacejka(s, F_z)  
F_y0 = -F_pacejka(a, F_z)  
  
F_total = sqrt(F_x0² + F_y0²)  
F_max = m * F_z  
  
F_x = F_x0 * min(1, (F_max / max(F_total, 0.0001)))  
F_y = F_y0 * min(1, (F_max / max(F_total, 0.0001)))   
    
F_roll = -c_rr * F_z * clamp(v_x / v_min, -1, 1) * f_i  
  
## Acceleration  
w_free = w + ((T_drive - F_x * R) / I_w) * delta_time  
w = sgn(w_free) * max(0, abs(w_free) - (T_brake / I_w) * delta_time)  
    
## Adding everything together  
F_wheel = F_z * d_norm + F_x * f_i + F_y * r_i + F_roll
  
sum_F = F_gravity + F_drag + F_down + sum_of_i(F_wheel_i)    
a_acc = sum_F / M  
v_car = v_car + a_acc * delta_time  
p_car = p_car + v_car * delta_time  
  
sum_r = sum_of_i(cross(p_contact_i - p_CoM, F_wheel_i))  
 
a_ang = (R_car * I⁻¹ * R_carᵀ) * sum_r  
w_car = w_car + a_ang * delta_time  
q_car = normalize(q_car + 0.5 * quat(w_car, 0) * q_car * delta_time)  