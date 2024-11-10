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
    return result;
}

struct EntityOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(3) view: vec3<f32>,
};

@vertex
fn vs_entity(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
) -> EntityOutput {
    var result: EntityOutput;
    result.normal = normalize(normal);
    let world_pos = vec4<f32>(pos, 1.0);

    result.view = (r_data.view * world_pos).xyz;
    result.position = r_data.proj * r_data.view * world_pos;
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
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0)); // Directional light
    let view_dir = normalize(-vertex.view);
    let normal = normalize(vertex.normal);

    // Ambient
    let ambient_strength = 0.1;
    let ambient = ambient_strength * vec3<f32>(1.0, 1.0, 1.0);

    // Diffuse
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = diff * vec3<f32>(1.0, 1.0, 1.0);

    // Specular
    let specular_strength = 0.5;
    let reflect_dir = reflect(-light_dir, normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular = specular_strength * spec * vec3<f32>(1.0, 1.0, 1.0);

    // Reflection
    let reflected = reflect(normalize(vertex.view), normal);

    let reflected_color = textureSample(r_texture, r_sampler, reflected).rgb;
    let result = (ambient + diffuse + specular) * reflected_color;
    return vec4<f32>(result, 1.0);
}
