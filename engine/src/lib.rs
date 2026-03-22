use winit::event_loop::{ControlFlow, EventLoop};

pub mod game_object;
pub mod scene;

mod renderer;

use renderer::Renderer;
use scene::Scene;

pub struct Engine {
    renderer: Renderer,
    event_loop: EventLoop<()>,
}

impl Engine {
    pub fn new<T>(main_loop: T, scene: Scene) -> Self
    where
        T: FnMut(&mut Renderer) + 'static + Send,
    {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        Self {
            renderer: Renderer::new(main_loop, scene),
            event_loop,
        }
    }

    pub fn run(mut self) {
        self.event_loop.run_app(&mut self.renderer).unwrap();
    }
}
