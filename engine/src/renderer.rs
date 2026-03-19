use crate::{gpu_structs, wgpu_api::*};
use std::{f32::consts::PI, sync::Arc};
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

pub struct Renderer {
    window: Option<Arc<Window>>,
    wgpu_api: Option<WgpuApi>,
    main_loop_callback: Option<Box<dyn FnMut(&mut Renderer)>>,
    camera: gpu_structs::Camera,
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
    move_speed: f32,
    cursor_grabbed: bool,
}

impl Renderer {
    pub fn new(main_loop_callback: impl FnMut(&mut Renderer) + 'static) -> Self {
        Self {
            window: None,
            wgpu_api: None,
            main_loop_callback: Some(Box::new(main_loop_callback)),
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.001,
            camera: gpu_structs::Camera {
                pos: [0.0, 0.0, 0.0],
                rot: [0.0, 0.0, 1.0],
                fov: PI / 2.0,
                ..Default::default()
            },
            move_speed: 0.1,
            cursor_grabbed: false,
        }
    }

    pub fn toggle_cursor_grab(&mut self) {
        if let Some(window) = &self.window {
            self.cursor_grabbed = !self.cursor_grabbed;

            let grab_mode = if self.cursor_grabbed {
                winit::window::CursorGrabMode::Locked
            } else {
                winit::window::CursorGrabMode::None
            };

            if let Err(e) = window.set_cursor_grab(grab_mode) {
                eprintln!("Error grabbing the cursor {:?}", e);
            }

            window.set_cursor_visible(!self.cursor_grabbed);
        }
    }

    //pub fn set_game_objects(&mut self, game_objects: Vec<gpu_structs::RenderObject>) {
    pub fn set_syntax_tree(&mut self, syntax_tree: crate::game_object::FlatRenderTree) {
        let leaf_storage = gpu_structs::LeafObjectStorage {
            length: syntax_tree.leafs.len() as u32,
            _pad: [0; 3],
        };

        let node_storage = gpu_structs::SyntaxNodeStorage {
            length: syntax_tree.nodes.len() as u32,
            num_root: syntax_tree.first_layer_length as i32,
            // _pad: [0; 2],
        };

        let mut leaf_data = Vec::new();
        leaf_data.extend_from_slice(bytemuck::cast_slice(&[leaf_storage]));
        leaf_data.extend_from_slice(bytemuck::cast_slice(&syntax_tree.leafs));

        let mut node_data = Vec::new();
        node_data.extend_from_slice(bytemuck::cast_slice(&[node_storage]));
        node_data.extend_from_slice(bytemuck::cast_slice(&syntax_tree.nodes));

        if let Some(state) = &mut self.wgpu_api {
            state
                .queue
                .write_buffer(&state.gpu_buffers.game_object, 0, &leaf_data);

            state
                .queue
                .write_buffer(&state.gpu_buffers.syntax_tree, 0, &node_data);
        }
    }

    fn walk(&mut self, forward: f32, sideways: f32, vertical: f32) {
        use std::f32::consts::FRAC_PI_2;
        self.camera.pos[0] += forward * self.camera.rot[1].sin() * self.move_speed;
        self.camera.pos[2] += forward * self.camera.rot[1].cos() * self.move_speed;
        self.camera.pos[0] += sideways * (self.camera.rot[1] + FRAC_PI_2).sin() * self.move_speed;
        self.camera.pos[2] += sideways * (self.camera.rot[1] + FRAC_PI_2).cos() * self.move_speed;
        self.camera.pos[1] += vertical * self.move_speed;
    }
}

impl ApplicationHandler for Renderer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("rust-3d"))
                .unwrap(),
        );
        self.wgpu_api = Some(WgpuApi::new(window.clone()));
        self.window = Some(window);
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _device_id: DeviceId, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                if !self.cursor_grabbed {
                    return;
                }
                let (delta_x, delta_y) = (delta.0 as f32, delta.1 as f32);

                self.yaw += delta_x * self.sensitivity;
                self.pitch -= delta_y * self.sensitivity;

                self.camera.rot[0] = self.pitch;
                self.camera.rot[1] = self.yaw;
            }
            _ => {}
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => {
                            if self.cursor_grabbed {
                                self.toggle_cursor_grab();
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyW) => self.walk(1.0, 0.0, 0.0),
                        PhysicalKey::Code(KeyCode::KeyS) => self.walk(-1.0, 0.0, 0.0),
                        PhysicalKey::Code(KeyCode::KeyA) => self.walk(0.0, -1.0, 0.0),
                        PhysicalKey::Code(KeyCode::KeyD) => self.walk(0.0, 1.0, 0.0),
                        PhysicalKey::Code(KeyCode::Space) => self.walk(0.0, 0.0, -1.0),
                        PhysicalKey::Code(KeyCode::KeyC) => self.walk(0.0, 0.0, 1.0),
                        _ => (),
                    }

                    if !matches!(event.physical_key, PhysicalKey::Code(KeyCode::Escape)) {
                        if let Some(state) = &mut self.wgpu_api {
                            state.queue.write_buffer(
                                &state.gpu_buffers.camera,
                                0,
                                bytemuck::cast_slice(&[self.camera]),
                            );
                        }
                    }
                }
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                if !self.cursor_grabbed && button == winit::event::MouseButton::Left {
                    self.toggle_cursor_grab();
                }
            }

            WindowEvent::Resized(size) => {
                if let Some(state) = &mut self.wgpu_api {
                    state.resize(size);
                    state.queue.write_buffer(
                        &state.gpu_buffers.screen_size,
                        0,
                        bytemuck::cast_slice(&[gpu_structs::ScreenSize {
                            size: [size.width as f32, size.height as f32],
                            _pad: [0.0; 2],
                        }]),
                    );
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(mut callback) = self.main_loop_callback.take() {
                    (callback)(self);
                    self.main_loop_callback = Some(callback);
                }
                if let Some(state) = &mut self.wgpu_api {
                    state.render();
                }
                if let Some(window) = &self.window {
                    window.request_redraw();
                }

                if let Some(state) = &mut self.wgpu_api {
                    state.queue.write_buffer(
                        &state.gpu_buffers.camera,
                        0,
                        bytemuck::cast_slice(&[self.camera]),
                    );
                }
            }

            _ => {}
        }
    }
}
