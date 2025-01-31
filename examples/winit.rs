use std::{ptr, time::Instant};

use imgui::{FontConfig, FontSource};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use winapi::shared::{d3d9::*, d3d9caps::*, d3d9types::*, windef::HWND};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

const WINDOW_WIDTH: f64 = 760.0;
const WINDOW_HEIGHT: f64 = 760.0;

unsafe fn set_up_dx_context(hwnd: HWND) -> (LPDIRECT3D9, LPDIRECT3DDEVICE9) {
    let d9 = Direct3DCreate9(D3D_SDK_VERSION);
    assert!(!d9.is_null(), "Direct3DCreate9 failed");
    let mut present_params = D3DPRESENT_PARAMETERS {
        BackBufferCount: 1,
        MultiSampleType: D3DMULTISAMPLE_NONE,
        MultiSampleQuality: 0,
        SwapEffect: D3DSWAPEFFECT_DISCARD,
        hDeviceWindow: hwnd,
        Flags: 0,
        FullScreen_RefreshRateInHz: D3DPRESENT_RATE_DEFAULT,
        PresentationInterval: D3DPRESENT_INTERVAL_DEFAULT,
        BackBufferFormat: D3DFMT_R5G6B5,
        EnableAutoDepthStencil: 0,
        Windowed: 1,
        BackBufferWidth: WINDOW_WIDTH as _,
        BackBufferHeight: WINDOW_HEIGHT as _,
        ..core::mem::zeroed()
    };
    let mut device = ptr::null_mut();
    let r = (&*d9).CreateDevice(
        D3DADAPTER_DEFAULT,
        D3DDEVTYPE_HAL,
        hwnd,
        D3DCREATE_SOFTWARE_VERTEXPROCESSING,
        &mut present_params,
        &mut device,
    );
    assert!(!(r < 0), "CreateDevice failed");
    (d9, device)
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("imgui_dx9_renderer winit example")
        .with_resizable(false)
        .with_inner_size(LogicalSize { width: WINDOW_WIDTH, height: WINDOW_HEIGHT })
        .build(&event_loop)
        .unwrap();
    let hwnd = if let RawWindowHandle::Windows(handle) = window.raw_window_handle() {
        handle.hwnd
    } else {
        unreachable!()
    };
    let (d9, device) = unsafe { set_up_dx_context(hwnd as _) };
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);
    let mut platform = WinitPlatform::init(&mut imgui);
    platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Rounded);

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (13.0 * hidpi_factor) as f32;
    imgui.fonts().add_font(&[FontSource::DefaultFontData {
        config: Some(FontConfig { size_pixels: font_size, ..FontConfig::default() }),
    }]);
    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let mut renderer = unsafe {
        imgui_dx9_renderer::Renderer::new(&mut imgui, wio::com::ComPtr::from_raw(device)).unwrap()
    };

    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(_) => {
            let now = Instant::now();
            imgui.io_mut().update_delta_time(now - last_frame);
            last_frame = now;
        },
        Event::MainEventsCleared => {
            let io = imgui.io_mut();
            platform.prepare_frame(io, &window).expect("Failed to start frame");
            window.request_redraw();
        },
        Event::RedrawRequested(_) => {
            unsafe {
                (&*device).Clear(0, ptr::null_mut(), D3DCLEAR_TARGET, 0xFFAA_AAAA, 1.0, 0);
                (&*device).BeginScene();
            }

            let ui = imgui.frame();
            imgui::Window::new("Hello world")
                .size([300.0, 100.0], imgui::Condition::FirstUseEver)
                .build(&ui, || {
                    ui.text("Hello world!");
                    ui.text("This...is...imgui-rs!");
                    ui.separator();
                    let mouse_pos = ui.io().mouse_pos;
                    ui.text(&format!("Mouse Position: ({:.1},{:.1})", mouse_pos[0], mouse_pos[1]));
                });
            ui.show_demo_window(&mut true);
            platform.prepare_render(&ui, &window);
            renderer.render(ui.render()).unwrap();
            unsafe {
                (&*device).EndScene();
                (&*device).Present(
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                );
            }
        },
        Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
            *control_flow = winit::event_loop::ControlFlow::Exit
        },
        Event::LoopDestroyed => unsafe {
            (&*d9).Release();
        },
        event => {
            platform.handle_event(imgui.io_mut(), &window, &event);
        },
    });
}
