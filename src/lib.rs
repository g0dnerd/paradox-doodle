use bytemuck::{Pod, Zeroable};

use clap::Parser;
use wgpu::util::DeviceExt;

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

const DEFAULT_RADIUS: f32 = 15.0;
const DEFAULT_STACKS: u32 = 40;
const DEFAULT_SLICES: u32 = 40;
pub fn generate_sphere(
    radius: Option<f32>,
    stacks: Option<u32>,
    slices: Option<u32>,
    // sphere_position: glam::Vec3,
) -> (Vec<Vertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let r = if let Some(rad) = radius {
        rad
    } else {
        DEFAULT_RADIUS
    };
    let stx = if let Some(s) = stacks {
        s
    } else {
        DEFAULT_STACKS
    };
    let slc = if let Some(s) = slices {
        s
    } else {
        DEFAULT_SLICES
    };

    // Generate vertices and normals
    for stack in 0..=stx {
        let stack_angle = std::f32::consts::PI / stx as f32 * stack as f32;
        let stack_y = r * stack_angle.cos();
        let stack_radius = r * stack_angle.sin();

        for slice in 0..=slc {
            let slice_angle = 2.0 * std::f32::consts::PI / slc as f32 * slice as f32;
            let x = stack_radius * slice_angle.cos();
            let z = stack_radius * slice_angle.sin();

            let pos = [x, stack_y, z];
            let normal = [x / r, stack_y / r, z / r];
            vertices.push(Vertex { pos, normal });

            // Generate indices for triangle faces
            if stack != stx && slice != slc {
                // Define the vertices at the corners of each quadrilateral
                let current = stack * (slc + 1) + slice;
                let next = current + slc + 1;

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

pub fn create_sphere_entity(
    device: &wgpu::Device,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
) -> Result<Entity, anyhow::Error> {
    // Create vertex buffer from the sphere vertices
    let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Sphere Vertex Buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    // Create index buffer from the sphere indices
    let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Sphere Index Buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let sphere_entity = Entity {
        vertex_buf,
        index_buf,
        vertex_count: indices.len() as u32,
    };

    Ok(sphere_entity)
}

pub struct Config {
    pub sphere_radius: f32,
    pub sphere_stacks: u32,
    pub sphere_slices: u32,
    pub image_size: u32,
    pub camera_distance: f32,
}

#[derive(Parser, Debug)]
#[command(version = "0.1")]
#[command(about = "renders a black hole in a skybox")]
#[command(long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    sphere_radius: Option<f32>,

    #[arg(short, long)]
    sphere_stacks: Option<u32>,

    #[arg(short, long)]
    sphere_slices: Option<u32>,

    #[arg(short, long)]
    image_size: u32,

    #[arg(short, long)]
    camera_distance: Option<f32>,
}
