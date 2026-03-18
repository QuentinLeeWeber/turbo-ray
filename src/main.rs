use std::f32::consts::PI;
use winit::event_loop::{ControlFlow, EventLoop};

mod game_object;
mod gpu_structs;
mod renderer;
mod wgpu_api;

use game_object::scene::Scene;
use gpu_structs::Camera;
use renderer::Renderer;

use crate::game_object::big_dummy::BigDummy;
use crate::game_object::dummy::Dummy;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = Renderer::new(update());

    event_loop.run_app(&mut app).unwrap();
}

fn update() -> impl FnMut(&mut Renderer) {
    let mut scene = Scene::new();

    scene.add(BigDummy::new());

    move |app| {
        app.set_camera(Camera {
            pos: [0.0, 0.0, 0.0],
            rot: [0.0, 0.0, 1.0],
            fov: PI / 2.0,
            ..Default::default()
        });

        scene.update();
        app.set_syntax_tree(scene.build_render_tree());
    }
}
