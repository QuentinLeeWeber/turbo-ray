use super::gpu_structs::{self, Camera};
use pollster::FutureExt;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::*;
use winit::window::Window;

pub struct GpuBuffers {
    pub game_object: wgpu::Buffer,
    pub screen_size: wgpu::Buffer,
    pub camera: wgpu::Buffer,
    pub syntax_tree: wgpu::Buffer,
    game_object_group: wgpu::BindGroup,
    screen_size_group: wgpu::BindGroup,
    camera_group: wgpu::BindGroup,
    syntax_tree_group: wgpu::BindGroup,
}

pub struct WgpuApi {
    pub queue: Queue,
    pub gpu_buffers: GpuBuffers,
    surface: Surface<'static>,
    device: Device,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
}

impl WgpuApi {
    pub fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = Instance::default();
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .block_on()
            .expect("no compatible adapter found");

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default())
            .block_on()
            .expect("Device creation failed");

        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader_src = include_str!("shader.wgsl");
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("shader.wgsl"),
            source: ShaderSource::Wgsl(shader_src.into()),
        });

        let game_object_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Game Objects Storage Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let syntax_tree_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Syntax Tree Storage Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Uniform Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let screen_size_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Screen Size Uniform Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let game_object_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            mapped_at_creation: false,
            label: Some("Game Objects Storage Buffer"),
            size: 48 * 1000,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let syntax_tree_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            mapped_at_creation: false,
            label: Some("Syntax Tree Storage Buffer"),
            size: 48 * 1000,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&vec![Camera::default()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let screen_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Frame Size Unitform Buffer"),
            contents: bytemuck::cast_slice(&vec![gpu_structs::ScreenSize {
                size: [0.0; 2],
                _pad: [0.0; 2],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let game_object_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game Objects Storage"),
            layout: &game_object_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: game_object_buffer.as_entire_binding(),
            }],
        });

        let syntax_tree_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Syntax Tree Storage"),
            layout: &syntax_tree_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: syntax_tree_buffer.as_entire_binding(),
            }],
        });

        let camera_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Uniform"),
            layout: &camera_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let screen_size_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Frame Size Uniform"),
            layout: &screen_size_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_size_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &game_object_group_layout,
                &screen_size_group_layout,
                &camera_group_layout,
                &syntax_tree_group_layout,
            ],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            gpu_buffers: GpuBuffers {
                game_object: game_object_buffer,
                screen_size: screen_size_buffer,
                camera: camera_buffer,
                syntax_tree: syntax_tree_buffer,
                game_object_group,
                screen_size_group,
                camera_group,
                syntax_tree_group,
            },
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) {
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(e) => {
                eprintln!("Surface-error: {e:?}");
                return;
            }
        };

        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            pass.set_bind_group(0, &self.gpu_buffers.game_object_group, &[]);
            pass.set_bind_group(1, &self.gpu_buffers.screen_size_group, &[]);
            pass.set_bind_group(2, &self.gpu_buffers.camera_group, &[]);
            pass.set_bind_group(3, &self.gpu_buffers.syntax_tree_group, &[]);

            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
