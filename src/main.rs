use std::f32::consts::PI;
use winit::event_loop::{ControlFlow, EventLoop};

mod gpu_structs;
mod renderer;
mod wgpu_api;

use gpu_structs::*;
use renderer::Renderer;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Renderer::new(game_loop);

    event_loop.run_app(&mut app).unwrap();
}

fn game_loop(app: &mut Renderer) {
    let game_objects = vec![GameObject {
        position: [0.0, 0.0, 1.0],
        size: 0.1,
        color: [1.0, 1.0, 0.0, 1.0],
        ..Default::default()
    }];
    app.set_game_objects(game_objects);

    app.set_camera(Camera {
        pos: [0.0, 0.0, 0.0],
        rot: [0.0, 0.0, 1.0],
        fov: PI / 2.0,
        ..Default::default()
    });
}
