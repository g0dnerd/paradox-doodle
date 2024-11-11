struct UniverseOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec3<f32>,
};

struct Data {
    // from camera to screen
    proj: mat4x4<f32>,
    // from screen to camera
    proj_inv: mat4x4<f32>,
    // from world to camera
    view: mat4x4<f32>,
    // camera position
    cam_pos: vec4<f32>,
    // transformation model
    model: mat4x4<f32>,
};

@group(0)
@binding(0)
var<uniform> r_data: Data;

@vertex
fn vs_universe(@builtin(vertex_index) vertex_index: u32) -> UniverseOutput {
    // Generate a full-screen triangle
    let pos = vec2<f32>(
      f32((vertex_index << 1) & 2),
      f32(vertex_index & 2)
    ) * 2.0 - 1.0;

    var result: UniverseOutput;
    result.position = vec4<f32>(pos, 0.9999, 1.0);

    var skybox_view = r_data.view;
    skybox_view[3] = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    result.uv = (skybox_view * vec4<f32>(pos, 1.0, 0.0)).xyz;

    // Apply lensing effect near the black hole
    let dist = length(result.uv);
    let lens_factor = 1.0 / (dist * 0.8 + 0.5);
    result.uv = result.uv * lens_factor;

    return result;
}

struct EntityOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) view: vec3<f32>,
};

@vertex
fn vs_entity(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> EntityOutput {
    var result: EntityOutput;

    let world_pos = r_data.model * vec4<f32>(pos, 1.0);
    result.world_position = world_pos;
    result.position = r_data.proj * r_data.view * world_pos;
    result.normal = normalize((r_data.model * vec4<f32>(normal, 0.0)).xyz);
    result.view = (r_data.view * world_pos).xyz;
    return result;
}

@group(0)
@binding(1)
var r_texture: texture_cube<f32>;

@group(0)
@binding(2)
var r_sampler: sampler;

@fragment
fn fs_universe(vertex: UniverseOutput) -> @location(0) vec4<f32> {
    return textureSample(r_texture, r_sampler, vertex.uv);
}

@fragment
fn fs_entity(vertex: EntityOutput) -> @location(0) vec4<f32> {
    let world_pos = vertex.world_position.xyz;
    let view_dir = normalize(-vertex.view);
    let normal = normalize(vertex.normal);

    // Define black hole parameters
    let black_hole_radius = 5.0;
    let schwarzschild_radius = 2.0 * black_hole_radius;
    let disk_inner_radius = 6.0;
    let disk_outer_radius = 12.0;
    let disk_thickness = 0.5;

    // Define disk orientation (assuming it's aligned with the xz plane)
    let disk_normal = vec3<f32>(1.0, 0.0, 0.0);

    // Calculate distance from black hole center
    let distance_from_center = length(world_pos);

    // Gravitational lensing
    let lensing_strength = 15.0;
    let deflection = lensing_strength * schwarzschild_radius / distance_from_center;
    let lensed_dir = normalize(view_dir + deflection * normalize(world_pos - view_dir * dot(world_pos, view_dir)));

    // Check if we're rendering the black hole
    if distance_from_center < black_hole_radius {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0); // Black hole is completely black
    }

    // Calculate disk intersection
    let disk_t = dot(world_pos, disk_normal) / dot(lensed_dir, disk_normal);
    let disk_intersection = world_pos - lensed_dir * disk_t;
    let disk_distance = length(disk_intersection);

    // Check if we're rendering the accretion disk
    if abs(dot(normalize(disk_intersection), disk_normal)) < disk_thickness &&
       disk_distance > disk_inner_radius && disk_distance < disk_outer_radius {
        // Calculate disk color with Doppler shift
        let orbital_velocity = sqrt(1.0 / disk_distance) * 0.5; // Simplified orbital velocity
        let doppler_factor = 1.0 / (1.0 - orbital_velocity * dot(normalize(disk_intersection), lensed_dir));
        let base_color = mix(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(1.0, 1.0, 0.0), (disk_distance - disk_inner_radius) / (disk_outer_radius - disk_inner_radius));
        let shifted_color = base_color * doppler_factor;
        return vec4<f32>(shifted_color, 1.0);
    }

    // Ambient lighting
    let ambient_color = vec3<f32>(0.1, 0.1, 0.15);
    let ambient_strength = 0.2;
    let ambient = ambient_strength * ambient_color;

    // Diffuse shading
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0)); // Directional light
    let light_color = vec3<f32>(1.0, 1.0, 0.9);
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = diff * light_color;

    // Specular reflection
    let specular_strength = 0.8;
    let shininess = 64.0;
    let reflect_dir = reflect(-light_dir, normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), shininess);
    let specular = specular_strength * spec * light_color;

    // Reflection
    let fresnel_factor = pow(1.0 - max(dot(view_dir, normal), 0.0), 3.0);
    let fresnel_strength = 0.5;
    let reflected = reflect(normalize(vertex.view), normal);
    let reflected_color = textureSample(r_texture, r_sampler, reflected).rgb;
    let lensed_reflection = textureSample(r_texture, r_sampler, lensed_dir).rgb;
    let reflection = mix(reflected_color, lensed_reflection, fresnel_factor * fresnel_strength);

    let lighting_color = ambient + diffuse + specular;
    let surface_color = lighting_color * 0.3 + reflection * 0.7;

    return vec4<f32>(surface_color, 1.0);
}
