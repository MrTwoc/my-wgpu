use std::sync::{Arc, Mutex};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalPosition, PhysicalSize},
    event::{
        DeviceEvent, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent,
    },
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct WgpuApp {
    // 窗口相关
    #[allow(unused)]
    window: Arc<Window>,
    // surface: 展示平面
    surface: wgpu::Surface<'static>,
    // device: GPU设备
    device: wgpu::Device,
    // queue：GPU队列
    queue: wgpu::Queue,
    // config：展示平面的配置
    config: wgpu::SurfaceConfiguration,
    // size：物理尺寸
    size: winit::dpi::PhysicalSize<u32>,
    // size_changed: 尺寸是否改变
    size_changed: bool,
    // 第二章挑战内容
    // clear_color: 清除颜色
    clear_color: wgpu::Color,
}
impl WgpuApp {
    /*
       new()
       创建一个新的 WgpuApp 实例
       必须参数：
       - window: 窗口实例。
       instance: GPU实例，
       surface: 展示平面，用于创建渲染目标。
       adapter: GPU适配器，用于选择和配置 GPU 设备。
       device: GPU设备，用于执行渲染操作。
       queue: GPU队列，用于提交命令到 GPU。

    */
    async fn new(window: Arc<Window>) -> Self {
        // instance: GPU实例
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            // 后端: 可以是OpenGL, Vulkan, Metal, DX12, or Browsers WebGPU
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        // surface: 展示平面
        let surface = instance.create_surface(window.clone()).unwrap();
        // adapter: GPU适配器
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // power_preference: 电源偏好
                // 可以是HighPerformance, LowPower, or Default
                power_preference: wgpu::PowerPreference::default(),
                // 兼容的展示平面
                compatible_surface: Some(&surface),
                // 是否强制使用回退适配器
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // device: GPU设备、queue: GPU队列
        // 为什么 device 和 queue 要一起声明，因为request_device方法返回的是一个元组，包含了 device 和 queue
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                // 所需的功能
                required_features: wgpu::Features::empty(),
                // 所需的限制
                required_limits: wgpu::Limits::defaults(),
                // 实验性功能: wgpu 27 新增参数
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                // 设备标签
                label: None,
                // 内存提示：作用是提示 GPU 内存分配器如何分配内存
                memory_hints: wgpu::MemoryHints::Performance,
                // 跟踪: 开启跟踪会在 GPU 上记录所有操作，用于调试
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();
        // caps: 展示平面的能力，比如支持的格式、alpha 模式等
        let caps = surface.get_capabilities(&adapter);
        // 处理窗口尺寸，max(1) 宽高最少1像素
        let mut size = window.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);
        let config = wgpu::SurfaceConfiguration {
            // 展示平面的使用方式
            // RENDER_ATTACHMENT: 表示这个表面将用作渲染目标，可以进行绘制操作
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // format：指定了 SurfaceTexture 在 GPU 内存上如何被存储
            format: caps.formats[0],
            // 宽高不能为0，否则会崩溃
            width: size.width,
            height: size.height,
            // present_mode: 展示模式
            // FIFO: 表示展示模式为先进先出，即按照绘制顺序展示图像
            // FIFO：指定了显示设备的刷新率做为渲染的帧速率，这本质上就是垂直同步
            present_mode: wgpu::PresentMode::Fifo,
            // 透明度模式，使用第一个支持的模式
            alpha_mode: caps.alpha_modes[0],
            // 视图格式：空向量，因为我们没有使用多视图渲染
            view_formats: vec![],
            // 期望的最大帧延迟：2帧，
            // 表示 GPU 可以延迟展示 2 帧图像，以提高渲染性能
            desired_maximum_frame_latency: 2,
        };
        // 配置展示平面
        surface.configure(&device, &config);

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            size_changed: false,
            clear_color,
        }
    }
    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        if new_size == self.size {
            return;
        }
        self.size = new_size;
        self.size_changed = true;
    }
    // 调整展示平面大小
    fn resize_surface_if_needed(&mut self) {
        if self.size_changed {
            self.config.width = self.size.width;
            self.config.height = self.size.height;
            // configure参数：device: GPU设备, config: 展示平面配置
            self.surface.configure(&self.device, &self.config);
            self.size_changed = false;
        }
    }

    fn update(&mut self) {}

    // 渲染函数
    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                // label 作用：用于调试，方便在 GPU 上查看命令编码器
                label: Some("Render Encoder"),
            });
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
        }
        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

    // 各种事件处理函数
    // 键盘事件, event: &KeyEvent 是键盘事件的引用
    fn keyboard_input(&mut self, _event: &KeyEvent) -> bool {
        false
    }
    // 鼠标点击事件, state: ElementState 是鼠标按钮的状态, button: MouseButton 是鼠标按钮
    fn mouse_click(&mut self, _state: ElementState, _button: MouseButton) -> bool {
        match _button {
            MouseButton::Left => {
                if _state == ElementState::Pressed {
                    self.clear_color = wgpu::Color {
                        r: 0.2,
                        g: 0.3,
                        b: 0.4,
                        a: 1.0,
                    };
                }
            }
            MouseButton::Right => {
                if _state == ElementState::Pressed {
                    self.clear_color = wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    };
                }
            }
            _ => {}
        }
        false
    }
    // 鼠标滚轮事件, delta: MouseScrollDelta 是鼠标滚轮的滚动量, phase: TouchPhase 是触摸阶段
    fn mouse_wheel(&mut self, _delta: MouseScrollDelta, _phase: TouchPhase) -> bool {
        false
    }
    // 鼠标移动事件, position: 鼠标的物理位置
    fn cursor_move(&mut self, _position: PhysicalPosition<f64>) -> bool {
        false
    }
    // 设备输入事件，event:设备事件
    fn device_input(&mut self, _event: &DeviceEvent) -> bool {
        false
    }
}

#[derive(Default)]
struct WgpuAppHandler {
    app: Arc<Mutex<Option<WgpuApp>>>,
    #[allow(dead_code)]
    missed_resize: Arc<Mutex<Option<PhysicalSize<u32>>>>,
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

        let window_attributes = Window::default_attributes().with_title("第二章");
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
        let mut guard = match self.app.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        let app = match guard.as_mut() {
            Some(app) => app,
            None => return,
        };
        match event {
            // 关闭窗口事件
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            // 窗口大小改变事件
            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {
                } else {
                    app.set_window_resized(physical_size);
                }
            }
            // 键盘输入事件
            WindowEvent::KeyboardInput { .. } => {}
            // 鼠标点击事件
            WindowEvent::MouseInput { state, button, .. } => {
                app.mouse_click(state, button);
            }
            // 重绘事件
            WindowEvent::RedrawRequested => {
                // pre_present_notify 作用：在渲染前调用，用于通知窗口系统渲染即将开始
                app.window.pre_present_notify();
                // match 作用：处理渲染函数返回的结果
                match app.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost) => {
                        eprintln!("Lost surface");
                    }
                    Err(_) => {}
                }
                // request_redraw 作用：请求重绘窗口，触发重绘事件
                app.window.request_redraw();
            }
            _ => (),
        }
    }
}

fn main() {
    let events_loop = EventLoop::new().unwrap();
    let mut app = WgpuAppHandler::default();
    let _ = events_loop.run_app(&mut app);
}
