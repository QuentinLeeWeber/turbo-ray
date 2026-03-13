use pollster::FutureExt;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::*;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

mod game_object;

struct State {
    surface: Surface<'static>,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pipeline: RenderPipeline,
    game_objects: Vec<game_object::GameObject>,
    game_object_buffer: wgpu::Buffer,
    screen_size_buffer: wgpu::Buffer,
    game_object_group: wgpu::BindGroup,
    screen_size_group: wgpu::BindGroup,
}

impl State {
    fn new(window: Arc<Window>) -> Self {
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

        let shader_src = include_str!("../shader/shader.wgsl");
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

        let game_objects = vec![game_object::GameObject {
            position: [0.0, 0.0, 0.0],
            size: 0.0,
            color: [0.0, 0.0, 0.0, 0.0],
        }];

        let game_object_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Game Objects Storage Buffer"),
            contents: bytemuck::cast_slice(&game_objects),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let screen_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Frame Size Unitform Buffer"),
            contents: bytemuck::cast_slice(&vec![game_object::ScreenSize {
                size: [0.0; 2],
                _pad: [0.0; 2],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let game_object_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game Objects Uniform"),
            layout: &game_object_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: game_object_buffer.as_entire_binding(),
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
            bind_group_layouts: &[&game_object_group_layout, &screen_size_group_layout],
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
            game_objects,
            game_object_buffer,
            screen_size_buffer,
            game_object_group,
            screen_size_group,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) {
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
            self.queue.write_buffer(
                &self.game_object_buffer,
                0,
                bytemuck::cast_slice(&self.game_objects),
            );

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

            pass.set_bind_group(0, &self.game_object_group, &[]);
            pass.set_bind_group(1, &self.screen_size_group, &[]);

            pass.set_pipeline(&self.pipeline);
            pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}

struct App {
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("rust-3d"))
                .unwrap(),
        );
        self.state = Some(State::new(window.clone()));
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(state) = &mut self.state {
                    state.resize(size);

                    state.queue.write_buffer(
                        &state.screen_size_buffer,
                        0,
                        bytemuck::cast_slice(&vec![game_object::ScreenSize {
                            size: [size.width as f32, size.height as f32],
                            _pad: [0.0; 2],
                        }]),
                    );
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.render();
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        state: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
