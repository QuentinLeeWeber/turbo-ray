use winit::event_loop::{ControlFlow, EventLoop};

mod game_object;
mod gpu_structs;
mod renderer;
mod wgpu_api;

use renderer::Renderer;

pub use game_object::{
    GameObjectTrait, SceneCommand, object_tree::ObjectNode, object_tree::ObjectNodeRaw,
    object_tree::ObjectNodeType, object_tree::SignedDistanceFunction, scene::Scene,
};

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

/*fn update() -> impl FnMut(&mut Renderer) {
    let mut scene = Scene::new();

    scene.add(Dummy::new());

    move |app| {
        scene.update();
        app.set_syntax_tree(scene.build_render_tree());
    }
}*/
