use winit::event_loop::{ControlFlow, EventLoop};

mod game_object;
mod renderer;

use game_object::GameObject;
use renderer::App;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(game_loop);

    event_loop.run_app(&mut app).unwrap();
}

fn game_loop(app: &mut App) {
    let game_objects = vec![GameObject {
        position: [0.0, 0.0, 1.0],
        size: 0.4,
        _padding: [0.0; 4],
        color: [1.0, 1.0, 1.0, 1.0],
    }];
    app.set_game_objects(game_objects);
}
