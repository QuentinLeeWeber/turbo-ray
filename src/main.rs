use winit::event_loop::{ControlFlow, EventLoop};

mod game_object;
mod renderer;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = renderer::App::new();
    event_loop.run_app(&mut app).unwrap();
}
