use std::sync::Arc;

use winit::{application::ApplicationHandler, event_loop::EventLoop, window::Window};

struct State {
    window: Arc<Window>,
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        Ok(Self { window })
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        todo!()
    }

    pub fn render(&mut self) {
        self.window.request_redraw();
    }
}

struct App {
    state: Option<State>,
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
}

impl App {
    pub fn new(event_loop: &EventLoop<State>) -> Self {
        let proxy = Some(event_loop.create_proxy());

        Self { state: None, proxy }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut window_attributes = Window::default_attributes();

        let window = wgpu::web_sys::window().unwrap_throw();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        todo!()
    }
}

fn main() {
    println!("Hello");
}
