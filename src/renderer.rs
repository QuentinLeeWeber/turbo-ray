use crate::{
    game_object::{self, Camera, GameObject},
    wgpu_api::*,
};
use std::f32::consts::PI;
use std::{collections::HashSet, sync::Arc};
use wgpu::wgc::device::queue;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

pub struct Renderer {
    window: Option<Arc<Window>>,
    wgpu_api: Option<WgpuApi>,
    main_loop_callback: Option<Box<dyn FnMut(&mut Renderer)>>,
    camera: Camera,
    yaw: f32,
    pitch: f32,
    sensitivity: f32,
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
            camera: Camera {
                pos: [0.0, 0.0, 0.0],
                rot: [0.0, 0.0, 1.0],
                fov: PI / 2.0,
                ..Default::default()
            },
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

    pub fn set_game_objects(&mut self, game_objects: Vec<game_object::GameObject>) {
        let storage = game_object::GameObjectStorage {
            length: game_objects.len() as u32,
            _pad: [0; 3],
        };

        let mut data = Vec::new();
        data.extend_from_slice(bytemuck::cast_slice(&[storage]));
        data.extend_from_slice(bytemuck::cast_slice(&game_objects));

        if let Some(state) = &mut self.wgpu_api {
            state
                .queue
                .write_buffer(&state.gpu_buffers.game_object, 0, &data);
        }
    }

    pub fn set_camera(&mut self, mut camera: game_object::Camera) {
        normalize(&mut camera.rot);
        if let Some(state) = &mut self.wgpu_api {
            state.queue.write_buffer(
                &state.gpu_buffers.camera,
                0,
                bytemuck::cast_slice(&[camera]),
            );
        }
    }
}

fn normalize(rot: &mut [f32]) {
    let len = rot.iter().map(|c| c * c).sum::<f32>().sqrt();
    for c in rot {
        *c /= len;
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

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let (delta_x, delta_y) = (delta.0 as f32, delta.1 as f32);

                self.yaw += delta_x * self.sensitivity;
                self.pitch -= delta_y * self.sensitivity;

                self.camera.rot[0] = self.pitch;
                self.camera.rot[1] = self.yaw;

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

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed && !event.repeat {
                    match event.physical_key {
                        PhysicalKey::Code(KeyCode::Escape) => {
                            if self.cursor_grabbed {
                                self.toggle_cursor_grab();
                            }
                        }
                        PhysicalKey::Code(KeyCode::KeyW) => self.camera.pos[2] += 0.05,
                        PhysicalKey::Code(KeyCode::KeyS) => self.camera.pos[2] -= 0.05,
                        PhysicalKey::Code(KeyCode::KeyA) => self.camera.pos[0] -= 0.05,
                        PhysicalKey::Code(KeyCode::KeyD) => self.camera.pos[0] += 0.05,
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
                    if let Some(window) = &self.window {
                        self.toggle_cursor_grab();
                    }
                }
            }

            WindowEvent::Resized(size) => {
                if let Some(state) = &mut self.wgpu_api {
                    state.resize(size);
                    state.queue.write_buffer(
                        &state.gpu_buffers.screen_size,
                        0,
                        bytemuck::cast_slice(&[game_object::ScreenSize {
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
                let camera = self.camera;
                self.set_camera(camera);
                println!("Camera Position: {:?}", camera.pos);
                println!("Camera Rotation : {:?}", camera.rot);
            }

            _ => {}
        }
    }
}
