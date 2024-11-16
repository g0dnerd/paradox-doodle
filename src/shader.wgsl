struct UniverseOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec3<f32>,
};

struct Data {
  proj: mat4x4<f32>,
  proj_inv: mat4x4<f32>,
  view: mat4x4<f32>,
  cam_pos: vec4<f32>,
  black_hole_pos: vec3<f32>,
  black_hole_radius: f32,
  black_hole_mass: f32,
};

@group(0)
@binding(0)
var<uniform> r_data: Data;

@group(0)
@binding(1)
var r_texture: texture_cube<f32>;

@group(0)
@binding(2)
var r_sampler: sampler;

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

    return result;
}

@fragment
fn fs_universe(vertex: UniverseOutput) -> @location(0) vec4<f32> {
    return textureSample(r_texture, r_sampler, vertex.uv);
}

struct BlackHoleOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) world_pos: vec3<f32>,
};

@vertex
fn vs_black_hole(@builtin(vertex_index) vertex_index: u32) -> BlackHoleOutput {
    var result: BlackHoleOutput;

    let pos = vec2<f32>(
        f32((vertex_index << 1) & 2),
        f32(vertex_index & 2)
    ) * 2.0 - 1.0;

    let scale = 10.0; // Adjust this value to change the size of the quad
    let world_pos = r_data.black_hole_pos + vec3<f32>(pos * r_data.black_hole_radius * scale, 0.0);

    result.position = r_data.proj * r_data.view * vec4<f32>(world_pos, 1.0);
    result.world_pos = world_pos;

    return result;
}

@fragment
fn fs_black_hole(vertex: BlackHoleOutput) -> @location(0) vec4<f32> {
    // Calculate the direction of the current pixel from the camera
    let ray_dir = normalize(vertex.world_pos - r_data.cam_pos.xyz);

    // Calculate the vector from the camera to the black hole
    let to_center = r_data.black_hole_pos - vertex.world_pos;
    let distance = length(to_center);

    // Check if the ray intersects the black hole
    if distance < r_data.black_hole_radius * 2.0 {
        // If inside the event horizon, return black
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }

    // Simple gravitational lensing effect
    let deflection_strength = r_data.black_hole_mass / (distance * distance);
    let deflected_dir = normalize(ray_dir + normalize(to_center) * deflection_strength);

    // Add a subtle glow around the black hole
    let glow_factor = smoothstep(r_data.black_hole_radius, r_data.black_hole_radius * 2.0, distance);
    let glow_color = vec3<f32>(0.5, 0.0, 0.5); // Purple glow

    let sampled_color = textureSample(r_texture, r_sampler, deflected_dir).rgb;
    let final_color = mix(glow_color, sampled_color, glow_factor);

    return vec4<f32>(final_color, 1.0);
}
