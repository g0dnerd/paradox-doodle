use bytemuck::{Pod, Zeroable};
use wgpu::{util::DeviceExt, AstcBlock, AstcChannel};

use crate::camera::Camera;

const IMAGE_SIZE: u32 = 1024;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
}

struct Entity {
    vertex_count: u32,
    vertex_buf: wgpu::Buffer,
}

pub struct Scene {
    camera: Camera,
    universe_pipeline: wgpu::RenderPipeline,
    entity_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    entities: Vec<Entity>,
    depth_view: wgpu::TextureView,
    staging_belt: wgpu::util::StagingBelt,
}

impl Scene {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

    fn create_depth_texture(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
    ) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
            view_formats: &[],
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

impl crate::framework::Framework for Scene {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::TEXTURE_COMPRESSION_ASTC
            | wgpu::Features::TEXTURE_COMPRESSION_ETC2
            | wgpu::Features::TEXTURE_COMPRESSION_BC
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let entities = Vec::new();
        {
            // let source = include_bytes!("assets/models/model.obj");
            // let data = obj::ObjData::load_buf(&source[..]).unwrap();
            // let mut vertices = Vec::new();
            // for object in data.objects {
            //     for group in object.groups {
            //         vertices.clear();
            //         for poly in group.polys {
            //             for end_idx in 2..poly.0.len() {
            //                 for &idx in &[0, end_idx - 1, end_idx] {
            //                     let obj::IndexTuple(position_id, _texture_id, normal_id) =
            //                         poly.0[idx];
            //                     vertices.push(Vertex {
            //                         pos: data.position[position_id],
            //                         normal: data.normal[normal_id.unwrap()],
            //                     })
            //                 }
            //             }
            //             let vertex_buf =
            //                 device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            //                     label: Some("Vertex"),
            //                     contents: bytemuck::cast_slice(&vertices),
            //                     usage: wgpu::BufferUsages::VERTEX,
            //                 });
            //             entities.push(Entity {
            //                 vertex_count: vertices.len() as u32,
            //                 vertex_buf,
            //             });
            //         }
            //     }
            // }

            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                multisampled: false,
                                view_dimension: wgpu::TextureViewDimension::Cube,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                });

            // Create the render pipeline
            let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

            let camera = Camera {
                screen_size: (config.width, config.height),
                angle_xz: 0.2,
                angle_y: 0.2,
                dist: 20.0,
            };
            let raw_uniforms = camera.to_uniform_data();
            let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Buffer"),
                contents: bytemuck::cast_slice(&raw_uniforms),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

            // Create the render pipelines
            let universe_pipeline =
                device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Universe"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: Some("vs_universe"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: Some("fs_universe"),
                        compilation_options: Default::default(),
                        targets: &[Some(config.view_formats[0].into())],
                    }),
                    primitive: wgpu::PrimitiveState {
                        front_face: wgpu::FrontFace::Cw,
                        ..Default::default()
                    },
                    depth_stencil: Some(wgpu::DepthStencilState {
                        format: Self::DEPTH_FORMAT,
                        depth_write_enabled: false,
                        depth_compare: wgpu::CompareFunction::LessEqual,
                        stencil: wgpu::StencilState::default(),
                        bias: wgpu::DepthBiasState::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                    cache: None,
                });

            let entity_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Entity"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_entity"),
                    compilation_options: Default::default(),
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                    }],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_entity"),
                    compilation_options: Default::default(),
                    targets: &[Some(config.view_formats[0].into())],
                }),
                primitive: wgpu::PrimitiveState {
                    front_face: wgpu::FrontFace::Cw,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Self::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

            let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: None,
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            let device_features = device.features();

            let skybox_format =
                if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ASTC) {
                    log::info!("Using astc");
                    wgpu::TextureFormat::Astc {
                        block: AstcBlock::B4x4,
                        channel: AstcChannel::UnormSrgb,
                    }
                } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_ETC2) {
                    log::info!("Using etc2");
                    wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb
                // } else if device_features.contains(wgpu::Features::TEXTURE_COMPRESSION_BC) {
                //     log::info!("Using bc7");
                //     wgpu::TextureFormat::Bc7RgbaUnormSrgb
                } else {
                    log::info!("Using rgba8");
                    wgpu::TextureFormat::Rgba8UnormSrgb
                };

            let size = wgpu::Extent3d {
                width: IMAGE_SIZE,
                height: IMAGE_SIZE,
                depth_or_array_layers: 6,
            };

            let layer_size = wgpu::Extent3d {
                depth_or_array_layers: 1,
                ..size
            };
            let max_mips = layer_size.max_mips(wgpu::TextureDimension::D2);

            log::info!(
                "Copying {:?} skybox images of size {}, {}, 6 with {} mips to gpu",
                skybox_format,
                IMAGE_SIZE,
                IMAGE_SIZE,
                max_mips,
            );

            let bytes = match skybox_format {
                wgpu::TextureFormat::Astc {
                    block: AstcBlock::B4x4,
                    channel: AstcChannel::UnormSrgb,
                } => &include_bytes!("assets/images/astc.ktx2")[..],
                wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb => {
                    &include_bytes!("assets/images/etc2.ktx2")[..]
                }
                // wgpu::TextureFormat::Bc7RgbaUnormSrgb => &include_bytes!("images/bc7.ktx2")[..],
                wgpu::TextureFormat::Rgba8UnormSrgb => {
                    log::info!("Using rgba8 file");
                    &include_bytes!("assets/images/rgba8.ktx2")[..]
                }
                _ => unreachable!(),
            };

            let reader = ktx2::Reader::new(bytes).unwrap();
            let header = reader.header();

            let mut image = Vec::with_capacity(reader.data().len());
            for level in reader.levels() {
                image.extend_from_slice(level);
            }

            let texture = device.create_texture_with_data(
                queue,
                &wgpu::TextureDescriptor {
                    size,
                    mip_level_count: header.level_count,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: skybox_format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    label: None,
                    view_formats: &[],
                },
                // KTX2 stores mip levels in mip major order.
                wgpu::util::TextureDataOrder::MipMajor,
                &image,
            );

            let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                dimension: Some(wgpu::TextureViewDimension::Cube),
                ..wgpu::TextureViewDescriptor::default()
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buf.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
                label: None,
            });

            let depth_view = Self::create_depth_texture(config, device);

            Scene {
                camera,
                universe_pipeline,
                entity_pipeline,
                bind_group,
                uniform_buf,
                entities,
                depth_view,
                staging_belt: wgpu::util::StagingBelt::new(0x100),
            }
        }
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.depth_view = Self::create_depth_texture(config, device);
        self.camera.screen_size = (config.width, config.height);
    }

    #[allow(clippy::single_match)]
    fn update(&mut self, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                let norm_x = position.x as f32 / self.camera.screen_size.0 as f32 - 0.5;
                let norm_y = position.y as f32 / self.camera.screen_size.1 as f32 - 0.5;
                self.camera.angle_y = norm_x * 5.0;
                self.camera.angle_xz = norm_y;
            }
            _ => {}
        }
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // update rotation
        let raw_uniforms = self.camera.to_uniform_data();
        self.staging_belt
            .write_buffer(
                &mut encoder,
                &self.uniform_buf,
                0,
                wgpu::BufferSize::new((raw_uniforms.len() * 4) as wgpu::BufferAddress).unwrap(),
                device,
            )
            .copy_from_slice(bytemuck::cast_slice(&raw_uniforms));

        self.staging_belt.finish();

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_pipeline(&self.entity_pipeline);

            for entity in self.entities.iter() {
                rpass.set_vertex_buffer(0, entity.vertex_buf.slice(..));
                rpass.draw(0..entity.vertex_count, 0..1);
            }

            rpass.set_pipeline(&self.universe_pipeline);
            rpass.draw(0..3, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));

        self.staging_belt.recall();
    }
}
