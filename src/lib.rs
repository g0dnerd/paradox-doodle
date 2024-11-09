use bytemuck::{Pod, Zeroable};

pub mod camera;
pub mod framework;
pub mod scene;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
}

pub struct Entity {
    pub vertex_count: u32,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
}

pub fn generate_sphere(
    radius: f32,
    stacks: u32,
    slices: u32,
    sphere_position: glam::Vec3,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices and normals
    for stack in 0..=stacks {
        let stack_angle = std::f32::consts::PI / stacks as f32 * stack as f32;
        let stack_y = radius * stack_angle.cos();
        let stack_radius = radius * stack_angle.sin();

        for slice in 0..=slices {
            let slice_angle = 2.0 * std::f32::consts::PI / slices as f32 * slice as f32;
            let x = stack_radius * slice_angle.cos();
            let z = stack_radius * slice_angle.sin();

            let pos = [
                x + sphere_position.x,
                stack_y + sphere_position.y,
                z + sphere_position.z,
            ];
            let normal = [x / radius, stack_y / radius, z / radius];
            vertices.push(Vertex { pos, normal });

            // Generate indices for triangle faces
            if stack != stacks && slice != slices {
                // Define the vertices at the corners of each quadrilateral
                let current = stack * (slices + 1) + slice;
                let next = current + slices + 1;

                // First triangle of the quad (counter-clockwise winding)
                indices.push(current);
                indices.push(next);
                indices.push(current + 1);

                // Second triangle of the quad (counter-clockwise winding)
                indices.push(next);
                indices.push(next + 1);
                indices.push(current + 1);
            }
        }
    }
    (vertices, indices)
}
