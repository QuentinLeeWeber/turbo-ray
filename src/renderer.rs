use crate::{gpu_structs, wgpu_api::*};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

pub struct Renderer {
    window: Option<Arc<Window>>,
    wgpu_api: Option<WgpuApi>,
    main_loop_callback: Option<Box<dyn FnMut(&mut Renderer)>>,
}

impl Renderer {
    pub fn new(main_loop_callback: impl FnMut(&mut Renderer) + 'static) -> Self {
        Self {
            window: None,
            wgpu_api: None,
            main_loop_callback: Some(Box::new(main_loop_callback)),
        }
    }

    //pub fn set_game_objects(&mut self, game_objects: Vec<gpu_structs::RenderObject>) {
    pub fn set_syntax_tree(&mut self, syntax_tree: crate::game_object::FlatRenderTree) {
        let leaf_storage = gpu_structs::LeafObjectStorage {
            length: syntax_tree.leafs.len() as u32,
            _pad: [0; 3],
        };

        let node_storage = gpu_structs::SyntaxNodeStorage {
            length: syntax_tree.nodes.len() as i32,
            num_root: syntax_tree.first_layer_length as i32,
            _pad: [0; 2],
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

    pub fn set_camera(&mut self, mut camera: gpu_structs::Camera) {
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

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(state) = &mut self.wgpu_api {
                    state.resize(size);

                    state.queue.write_buffer(
                        &state.gpu_buffers.screen_size,
                        0,
                        bytemuck::cast_slice(&vec![gpu_structs::ScreenSize {
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
            }
            _ => {}
        }
    }
}
