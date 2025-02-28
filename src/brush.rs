use cgmath::Point3;
use wgpu::{include_wgsl, util::DeviceExt, ShaderStages};
use crate::{camera::{Camera, CameraController, CameraUniform}, instance::{self, InstanceRaw}, model::{DrawModel, Model}, resources, texture, vertex::{ModelVertex, Vertex, INDICES, VERTICES}};

pub struct Brush {
    pub position: cgmath::Point3<f32>,
    pub radius: f32,

    pub render_pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,

    pub buffer: wgpu::Buffer,

    pub uniform: BrushUniform
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BrushUniform {
    pub position: [f32; 3],
    pub radius: f32
}

impl Brush {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Self {
        let brush_uniform = BrushUniform::new();
        
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[brush_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Brush Bind Group Layout"),
            entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                        count: None,
                    }
                ]
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Brush Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding()
                }
            ]
        });



        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Brush Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });


        let brush_shader: wgpu::ShaderModule = device.create_shader_module(include_wgsl!("brush_shader.wgsl"));


        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Brush Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &brush_shader,
                entry_point: "vs_main",
                buffers: &[ ],
                compilation_options: wgpu::PipelineCompilationOptions::default()
            },
            fragment: Some(wgpu::FragmentState { 
                module: &brush_shader, 
                entry_point: "fs_main", 
                compilation_options: wgpu::PipelineCompilationOptions::default(), 
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None,
            cache: None,
        });

        Self {
            render_pipeline,
            position: Point3::new(0.0, 0.0, 0.0),
            radius: 5.0,

            bind_group_layout,
            bind_group,

            buffer,

            uniform: brush_uniform,
        }
    }

    pub fn update_position(&mut self, new_position: Point3<f32>) {
        self.position = new_position;
        self.uniform.update_position(new_position);
    }

    pub fn update_radius(&mut self, new_radius: f32) {
        if new_radius <= 0.0 {
            return;
        }
        self.radius = new_radius;
        self.uniform.update_radius(new_radius);
    }

}

impl BrushUniform {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            radius: 5.0
        }
    }

    pub fn update_position(&mut self, new_position: Point3<f32>) {
        self.position = [new_position.x, new_position.y, new_position.z];
    }

    pub fn update_radius(&mut self, new_radius: f32) {
        self.radius = new_radius;
    }
}