# Racing game in Bevy

This game is using Bevy + Rust to run.
All phyiscs in the game are coded by myself. The game engines phyiscs are not being used.  

Play here: [https://bigb132.github.io/racing_game](https://bigb132.github.io/racing_game) (might lag on older devices)

The game is not finished yet, but you can drive arround on the map and test it out.

# Phyiscs

This part of the file will explain how the physics work.

## Constants

The values below are constants, but may change slightly depending on the car you want to simulate.  
All names are uppercase due to rust naming conventions.

### Chassis, Mass and Inertia
|Name|Value|Description|
|---|---|---|
|M|1730 kg|The mass of the car.|
|G|9.81 m/s²|The Gravity of earth.|
|H_COM|0.3 m|The height of the center of mass of the car.|
|I_XX|2200.0|Pitch inertia| //TODO
|I_YY|2400.0|Yaw inertia| //TODO
|I_ZZ|550.0|Roll inertia| //TODO

### Aerodynamics
|Name|Value|Description|
|---|---|---|
|P|1.225 kg/m³|The air density.|
|C_W|0.3|The aerodynamic drag coefficent.(aerodynamic efficency)|
|A|2.07 m²|The frontal surface area (for airflow).|
|C_L|0.3|The aerodynamic downforce coefficent. More velocity will result in a higher aerodynamic downward force.|

### Suspension and Anti-Roll Bars
|Name|Value|Description|
|---|---|---|
|K_FRONT|35000.0 N/m|The stiffness of the front suspension spring.|
|K_REAR|30000.0 N/m|The stiffness of the rear suspension spring.|
|C_BUMP|2800.0 N * m/s²|The suspension damper compression coefficent. Used when the spring compresses.|
|C_REBOUND|4200.0 N * m/s²|The suspension damper rebound coefficent. Used when the spring extends.|
|L_REST|0.45 m|The length of the suspension struts.|
|K_ARB_FRONT|5000.0 N/m|The front anti-roll bar stiffness.|
|K_ARB_REAR|3000.0 N/m|The rear anti-roll bar stiffness.|

### Wheels and Tires
|Name|Value|Description|
|---|---|---|
|R|0.37 m|The radius of the tire.|
|I_W|1.2 kg * m²|The rotational moment of inertia.|
|C_RR|0.012|The rolling resistance coefficent.|
|m2|1.0|The peak tire friction coefficent.|
|B|10.0|The pacejka stiffness factor.|
|C|1.45|The pacejka shape factor.|
|E|-0.6|The pacejka curvarure factor.|
|V_MIN|0.8 m/s|Low speed threshold to prevent crashes when calculating slip angles and ratios at low speed.|

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

Before we start with calculating the actual physics, we have to calulate some references:

|Name|Formula|Description|Unknown Values|
|---|---|---|---|
|u_up|q_car * Vec3(0.0, 1.0, 0.0)|This will result in a new Vector3, but only keep the y value.|**q_car**: The rotation of the car|
|d_down|q_car * Vec3(0.0, -1.0, 0.0)|This will result in a new Vector3, but only keep the negative y value.|**q_car**: The rotation of the car|
|p_com|p_car + q_car * Vec3(0, H_COM, 0)|This calculates the position of the center of mass in the world coordinate system.|**p_car**: Poition of the car in the world coordinate system. **q_car**: Rotation of the car.|
|p_anchor|p_car + q_car * p_rel_anchors[i]|This will calculate the position of the anchors in the world coordinate system per wheel.|**p_rel_anchors**: Position of the anchors relative to the cars position.|

### Raycast Suspension

Everything here is per wheel (i):  
A ray gets cast down from p_anchor[i] along d_down. If it intersects with the ground, we can calulate l_hit:  

l_hit = min(distance_of_ray, L_REST - R) - R  
Now we can use that to calculate two values:
|Name|Formula|Description|
|---|---|---|
|p_wheel|p_anchor + d_down * l_hit|The position of the weehl in the world coordinate system.|
|x|max(0, L_REST - l_hit)|The suspension compression distance|

### Springs

|Name|Formula|Description|
|---|---|---|
|v_anchor|v_car + w_car * (p_anchor - p_com)|The velocity of the suspension anchor.|
|v_spring|v_anchor * d_down|The suspension compression speed.|

If v_spring is bigger than 0, the suspension is being compressed and if it's smaller, the suspension extends. Depending on the 


# Building 

You can run it yourself using
```
cargo run
```

You can also build it yourself using

```
cargo build
```

Both will take care of installing all the deps.