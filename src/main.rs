use std::sync::{Arc, Mutex};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct WgpuApp {
    #[allow(unused)]
    window: Arc<Window>,
}
impl WgpuApp {
    async fn new(window: Arc<Window>) -> Self {
        Self { window }
    }
}

#[derive(Default)]
struct WgpuAppHandler {
    app: Arc<Mutex<Option<WgpuApp>>>,
}

impl ApplicationHandler for WgpuAppHandler {
    // 恢复事件
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // 以下文章源码，但是好像没有处理lock()可能返回的错误，所以换了一种写法
        // if self.app.as_ref().lock().is_some() {
        //     return;
        // }
        if let Ok(guard) = self.app.as_ref().lock() {
            if guard.is_some() {
                return;
            }
        }

        let window_attributes = Window::default_attributes().with_title("第一章-窗口");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let wgpu_app = pollster::block_on(WgpuApp::new(window));
        // 同上，好像没有处理lock()可能返回的错误，所以换了一种写法
        // self.app.lock().replace(wgpu_app);
        if let Ok(mut guard) = self.app.lock() {
            guard.replace(wgpu_app);
        }
    }

    // 暂停事件
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {}

    // 窗口事件
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            // 关闭窗口事件
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            // 窗口大小改变事件
            WindowEvent::Resized(_size) => {}
            // 键盘输入事件
            WindowEvent::KeyboardInput { .. } => {}
            // 重绘事件
            WindowEvent::RedrawRequested => {}
            _ => (),
        }
    }
}

fn main() {
    let events_loop = EventLoop::new().unwrap();
    let mut app = WgpuAppHandler::default();
    let _ = events_loop.run_app(&mut app);
}
