use wgpu::{util::DeviceExt, AstcBlock, AstcChannel};

use crate::{camera::Camera, create_sphere_entity, generate_sphere, Cli, Entity, Vertex};

pub struct Scene {
    camera: Camera,
    universe_pipeline: wgpu::RenderPipeline,
    entity_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    entities: Vec<Entity>,
    depth_view: wgpu::TextureView,
    staging_belt: wgpu::util::StagingBelt,
    rotation_angle: f32,
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
        args: &Cli,
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self, anyhow::Error> {
        let mut entities = Vec::new();
        {
            let r = args.sphere_radius;
            let stacks = args.sphere_stacks;
            let slices = args.sphere_slices;

            let (vertices, indices) = generate_sphere(r, stacks, slices);
            let sphere_entity = create_sphere_entity(device, vertices, indices)?;

            entities.push(sphere_entity);

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

            let camera_distance = args.camera_distance.unwrap_or(150.0);

            let camera = Camera {
                screen_size: (config.width, config.height),
                dist: camera_distance,
                angle: -2.5,
            };
            let raw_camera_data = camera.to_uniform_data();

            let rotation_matrix = glam::Mat4::from_rotation_y(0.0);
            let translation_matrix = glam::Mat4::from_translation(glam::Vec3::new(0.0, 2.0, 0.0));
            // let model_matrix = rotation_matrix * translation_matrix;
            let model_matrix = translation_matrix * rotation_matrix;

            let raw_model_matrix = model_matrix.to_cols_array();

            // Camera data: 52, Entity data: 16
            let mut raw_uniforms = Vec::with_capacity(52 + 16);

            raw_uniforms.extend_from_slice(&raw_camera_data);
            raw_uniforms.extend_from_slice(&raw_model_matrix);

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

            let image_size = args.image_size;

            let size = wgpu::Extent3d {
                width: image_size,
                height: image_size,
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
                image_size,
                image_size,
                max_mips,
            );

            // Only supporting rgba8 files for now
            if skybox_format != wgpu::TextureFormat::Rgba8UnormSrgb {
                return Err(anyhow::anyhow!(
                    "Unsupported texture type {:?} (only rgba8 is supported at the moment)",
                    skybox_format
                ));
            }
            let bytes = &include_bytes!("assets/images/skybox.ktx2");

            let reader = ktx2::Reader::new(bytes)
                .map_err(|e| anyhow::anyhow!("Failed to create KTX2 reader: {}", e))?;
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

            Ok(Scene {
                camera,
                universe_pipeline,
                entity_pipeline,
                bind_group,
                uniform_buf,
                entities,
                depth_view,
                staging_belt: wgpu::util::StagingBelt::new(0x100),
                rotation_angle: 0.0,
            })
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
                let _norm_x = position.x as f32 / self.camera.screen_size.0 as f32 - 0.5;
                let _norm_y = position.y as f32 / self.camera.screen_size.1 as f32 - 0.5;
                // self.camera.angle = norm_x * 5.0;
                // self.camera.angle_xz = (norm_y * 5.0).clamp(-1.5, 1.5);
            }
            _ => {}
        }
    }

    fn render(&mut self, view: &wgpu::TextureView, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Update rotation angle for the sphere
        self.rotation_angle += 0.0003;

        // Translation matrix to move the sphere to the origin
        let rotation_matrix = glam::Mat4::from_rotation_y(self.rotation_angle);

        let translation_matrix = glam::Mat4::from_translation(glam::Vec3::new(0.0, 2.0, 0.0));
        // let model_matrix = rotation_matrix * translation_matrix;
        let model_matrix = translation_matrix * rotation_matrix;
        let raw_model_matrix = model_matrix.to_cols_array();

        let raw_camera_data = self.camera.to_uniform_data();

        // Camera data: 52, Entity data: 16
        let mut raw_uniforms = Vec::with_capacity(52 + 16);

        raw_uniforms.extend_from_slice(&raw_camera_data);
        raw_uniforms.extend_from_slice(&raw_model_matrix);

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
                rpass.set_index_buffer(entity.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                rpass.draw_indexed(0..entity.vertex_count, 0, 0..1);
            }

            rpass.set_pipeline(&self.universe_pipeline);
            rpass.draw(0..3, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));

        self.staging_belt.recall();
    }
}
