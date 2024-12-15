use cgmath::prelude::*;
use wgpu::{include_wgsl, util::DeviceExt, BufferUsages, Instance, ShaderStages};
use winit::{event::WindowEvent, window::Window};

use crate::{camera::{Camera, CameraController, CameraUniform}, instance::{self, InstanceRaw}, model::{DrawModel, Model}, resources, texture, vertex::{ModelVertex, Vertex, INDICES, VERTICES}};

pub struct State<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub clear_color: wgpu::Color,

    pub render_pipeline: wgpu::RenderPipeline,

    pub obj_model: Model,

    pub instances: Vec<instance::Instance>,
    pub instance_buffer: wgpu::Buffer,

    pub depth_texture: texture::Texture,

    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,

    pub window: &'a Window
}

const NUM_INSTANCES_PER_ROW: u32 = 10;

impl<'a> State<'a> {
    
    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            }
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),

                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },

                label: None,
                memory_hints: Default::default(),
            }, 
            None
        ).await.unwrap();
        
        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities.formats.iter()
            .find(|format| format.is_srgb())
            .copied()
            .unwrap_or(surface_capabilities.formats[0]);


        let texture_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture { 
                            sample_type: wgpu::TextureSampleType::Float { filterable: true }, 
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false
                        },
                        count: None
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None
                    },
                ],
                label: Some("texture_bind_group_layout")
            }
        );


        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let obj_model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout).await.unwrap();
        
        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let position = cgmath::Vector3::new(x as f32, 0.0, z as f32);
                let rotation = if position.is_zero() {
                    // https://sotrh.github.io/learn-wgpu/beginner/tutorial7-instancing/#the-instance-buffer 
                    // this is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can affect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                instance::Instance {
                    position,
                    rotation,
                }
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(instance::Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let depth_texture: texture::Texture = texture::Texture::create_depth_texture(&device, &config, Some("depth_texture"));

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0
        };

        let camera_controller = CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                        count: None,
                    }
                ]
            }
        );

        let camera_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &camera_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_buffer.as_entire_binding(),
                    }
                ]
            }
        );

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &texture_bind_group_layout,
                &camera_bind_group_layout,
            ],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    ModelVertex::desc(),
                    InstanceRaw::desc(),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default()
            },
            fragment: Some(wgpu::FragmentState { 
                module: &shader, 
                entry_point: "fs_main", 
                compilation_options: wgpu::PipelineCompilationOptions::default(), 
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None,
            cache: None,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            render_pipeline,

            obj_model,

            instances,
            instance_buffer,

            depth_texture,

            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,

            clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 },
            size
        }

    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
        
            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, Some("depth_texture"));
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        // match event {
        //     WindowEvent::CursorMoved { device_id: _, position } => {
        //         self.clear_color = wgpu::Color {
        //             r: position.x / self.config.width as f64,
        //             g: position.y / self.config.height as f64,
        //             b: 0.0,
        //             a: 1.0
        //         }
        //     },
        //     _ => return false
        // }
        self.camera_controller.handle_event(event)
    }

    pub fn update(&mut self) {
       self.camera_controller.update_camera(&mut self.camera);
       self.camera_uniform.update_view_proj(&self.camera);
       self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(
                    wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_texture.view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                ),
                occlusion_query_set: None,
                timestamp_writes: None
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw_model_instanced(&self.obj_model, 0..self.instances.len() as u32, &self.camera_bind_group);
            
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
        

    }


}

 