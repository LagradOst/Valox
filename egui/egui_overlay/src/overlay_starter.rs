use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::prelude::OsStrExt;
use std::time::Instant;

use ::egui::FontDefinitions;

use egui_winit_platform::{Platform, PlatformDescriptor};

//use epi::egui::emath::Numeric;
use wgpu::{Backends, CommandBuffer, InstanceDescriptor};

use winit::event_loop::ControlFlow;

use winit::event::Event::*;

use crate::egui_wgpu_backend::{RenderPass, ScreenDescriptor};

#[cfg(feature = "log")]
fn log(str: &'static str) {
    println!("{str}");
}
#[cfg(not(feature = "log"))]
fn log(str: &'static str) {}

/// A custom event type for the winit app.
#[allow(dead_code)]
enum Event {
    RequestRedraw,
}
pub trait EguiStruct {
    fn draw(&mut self, context: &egui::Context);
}

pub trait Draw {
    //fn hide_window(&self, windowid: u64) -> Result<(), String>;
    fn run(&mut self, show_ui: &mut bool, ctx: &egui::Context);
    fn setup(&mut self, ctx: &egui::Context);
    fn save(&self);
}

#[cfg(not(windows))]
fn set_window(window: &winit::window::Window, maximized: bool, show_ui: bool) {
    if maximized {
        window.set_maximized(true);
        window.set_window_level(WindowLevel::AlwaysOnTop);
    } else {
        window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        window.set_window_level(WindowLevel::AlwaysOnTop);
    }
    window.set_cursor_hittest(show_ui).unwrap();
    window.focus_window();
}

fn set_clickthrou(window: &winit::window::Window, show_ui: bool) {
    use winapi::{
        shared::windef::HWND__,
        um::winuser::{SetWindowLongW, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT},
    };
    let id: u64 = window.id().into();
    let window: *mut HWND__ = id as _;
    unsafe {
        let flags = if show_ui {
            WS_EX_TRANSPARENT
        } else {
            WS_EX_LAYERED | WS_EX_TRANSPARENT //WS_EX_COMPOSITED | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST
        };

        SetWindowLongW(window, GWL_EXSTYLE, flags as i32);
    }
}

#[cfg(windows)]
fn set_window(window: &winit::window::Window, show_ui: bool) {
    use winapi::{
        shared::windef::{HWND__, RECT},
        um::{
            dwmapi::DwmExtendFrameIntoClientArea,
            uxtheme::MARGINS,
            wingdi::{GetDeviceCaps, DESKTOPVERTRES, VERTRES},
            winuser::{
                GetDC, GetDesktopWindow, GetWindowRect, MoveWindow, SetLayeredWindowAttributes,
                LWA_ALPHA,
            },
        },
    };

    set_clickthrou(&window, show_ui);

    let id: u64 = window.id().into();
    let window: *mut HWND__ = id as _;
    unsafe {
        let screen_width: i32;
        let screen_height: i32;
        {
            let mut desktop: RECT = RECT::default();
            // Get a handle to the desktop window
            let h_desktop = GetDesktopWindow();
            // Get the size of screen to the variable desktop
            GetWindowRect(h_desktop, &mut desktop as _);
            let monitor = GetDC(h_desktop);

            let current = GetDeviceCaps(monitor, VERTRES);
            let total = GetDeviceCaps(monitor, DESKTOPVERTRES);

            screen_width = (desktop.right - desktop.left) * total / current;
            screen_height = (desktop.bottom - desktop.top) * total / current;
        }
        MoveWindow(window, 0, 0, screen_width - 1, screen_height - 1, 1);
        /*
        SetWindowLongPtrA(window, -20, GetWindowLongA(window, -20) as LONG_PTR | 0x20);
        */
        let margin = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        DwmExtendFrameIntoClientArea(window, &margin as _);
        SetLayeredWindowAttributes(window, 0, 255, LWA_ALPHA);
        log("topmost");
    }
}

pub fn encode_wide(string: impl AsRef<OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(once(0)).collect()
}

/// A simple egui + wgpu + winit based example.
pub fn start_overlay(mut app: Box<dyn Draw>, name: String, vsync : bool) {
    log("start_overlay started");
    let event_loop = winit::event_loop::EventLoopBuilder::<Event>::with_user_event().build();
    //let window_id = winit::window::WindowId::from(0u64);

    let window = winit::window::WindowBuilder::new()
        .with_decorations(false)
        .with_resizable(false)
        //.with_transparent(true)
        //.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        //.with_transparent(true)
        // .with_transparent(true).with_maximized(true)
        .with_title(name)
        //.with_owner_window(0)
        .build(&event_loop)
        .unwrap();
    unsafe {
        winapi::um::winuser::ShowWindow(u64::from(window.id()) as _, winapi::um::winuser::SW_SHOW);
    }

    log("creating instance");
    let instance_descr = InstanceDescriptor {
        backends: Backends::VULKAN,
        dx12_shader_compiler: wgpu::Dx12Compiler::default(),
    };
    let instance = wgpu::Instance::new(instance_descr);
    let surface = unsafe { instance.create_surface(&window).expect("Valid surface") };

    // WGPU 0.11+ support force fallback (if HW implementation not supported), set it to true or false (optional).
    log("creating adapter");
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    log("requesting device");
    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ))
    .unwrap();

    let size = window.inner_size();
    let surf_capabilities = surface.get_capabilities(&adapter);
    //wgpu::TextureFormat::Bgra8UnormSrgb;
    let surface_format = surf_capabilities.formats[0]; //.get_supported_formats(&adapter)[0];

    let mut surface_config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width as u32,
        height: size.height as u32,
        present_mode: if vsync { wgpu::PresentMode::AutoVsync } else { wgpu::PresentMode::Immediate },
        alpha_mode: wgpu::CompositeAlphaMode::Opaque,
        view_formats: vec![surface_format],
    };
    log("configuring surface");
    surface.configure(&device, &surface_config);

    log("loading shaders");

    let mut platform = Platform::new(PlatformDescriptor {
        physical_width: size.width as u32,
        physical_height: size.height as u32,
        scale_factor: window.scale_factor(),
        font_definitions: FontDefinitions::default(),
        style: Default::default(),
    });

    let data: &str = include_str!("../res/shaders/egui.wgsl");
    let mut egui_rpass = RenderPass::new(&device, surface_format, 1, data, false);

    // Display the demo application that ships with egui.
    //let demo_app = egui_demo_lib::DemoWindows::default();

    let start_time = Instant::now();

    /*
        log("init window");
        let win_id: u64 = window.id().into();
    if cfg!(debug_assertions) {
        log("Debug enable, dont hide window");
    } else {
        if let Err(e) = app.hide_window(win_id) {
            eprintln!("Failed to hide window with error {}", e);
        } else {
            log("window hidden")
        }
    }*/
    let mut show_ui = false;
    let start = Instant::now();
    let mut has_showed = false;
    let ctx = platform.context();
    /*
    let mut style: egui::Style = (*platform.context().style()).clone();
    style.visuals.window_shadow.color = Color32::from_black_alpha(10);
    style.visuals.window_shadow.extrusion = 10.0;
    style.interaction.show_tooltips_only_when_still = false;
    style.visuals.window_rounding = Rounding::same(5.0);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgba_unmultiplied(32, 32, 32,250);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(10.0, Color32::WHITE);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(0.5, Color32::from_gray(50));

    style.visuals.widgets.inactive.bg_stroke = Stroke::new(0.5,Color32::from_white_alpha(20));
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0,Color32::WHITE);
    style.visuals.widgets.hovered = style.visuals.widgets.inactive;
    style.visuals.widgets.hovered.expansion = 1.0;

    /*style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(110, 170, 217);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(0.0,Color32::from_rgb(0, 0, 0)) ;


    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(110, 170, 217);
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(0.0,Color32::from_rgb(0, 0, 0)) ;

    style.visuals.widgets.active.bg_fill = Color32::from_rgb(110, 170, 217);
    style.visuals.widgets.active.fg_stroke = Stroke::new(0.0,Color32::from_rgb(0, 0, 0)) ;

    style.visuals.widgets.open.bg_fill = Color32::from_rgb(110, 170, 217);
    style.visuals.widgets.open.fg_stroke = Stroke::new(0.0,Color32::from_rgb(0, 0, 0)) ;*/
    style.visuals.selection.bg_fill = Color32::from_rgb(110, 170, 217);
    style.visuals.selection.stroke = Stroke::new(0.0,Color32::from_rgb(0, 0, 0));

    //style.visuals.widgets.active.fg_stroke = Color32::from_rgb(110, 170, 217);

    //style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_gray(50));
    //TODO SWITCH https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/toggle_switch.rs

    //style.visuals.override_text_color = Some(Color32::WHITE);
    //style.visuals.window_shadow.extrusion = 0.0f32;

    ctx.set_style(style);*/

    app.setup(&ctx);

    log("starting event loop");
    window.request_redraw();
    event_loop.run(move |event, _, control_flow| {
        // Pass the winit events to the platform integration.
        platform.handle_event(&event);

        match event {
            RedrawRequested(..) => {
                if !has_showed {
                    has_showed = true;
                    set_window(&window, show_ui);
                }

                platform.update_time(start_time.elapsed().as_secs_f64());

                let output_frame = match surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(wgpu::SurfaceError::Outdated) => {
                        // This error occurs when the app is minimized on Windows.
                        // Silently return here to prevent spamming the console with:
                        // "The underlying surface has changed, and therefore the swap chain must be updated"
                        return;
                    }
                    Err(e) => {
                        eprintln!("Dropped frame with error: {}", e);
                        return;
                    }
                };
                let output_view = output_frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // Begin to draw the UI frame.
                platform.begin_frame();

                let pre_run = show_ui;

                app.run(&mut show_ui, &platform.context());

                if pre_run != show_ui {
                    set_clickthrou(&window, show_ui);
                    // save every time we close the UI just to have it up to date in case of crash of ctrl+c
                    app.save();
                }

                // End the UI frame. We could now handle the output and draw the UI with the backend.
                let screen_descriptor = ScreenDescriptor {
                    physical_width: surface_config.width,
                    physical_height: surface_config.height,
                    scale_factor: window.scale_factor() as f32,
                    start,
                };
                let mut buffers: Vec<CommandBuffer> = Vec::new();

                let full_output = platform.end_frame(Some(&window));
                let paint_jobs = platform.context().tessellate(full_output.shapes);

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

                // Upload all resources for the GPU.
                let tdelta = full_output.textures_delta;

                egui_rpass
                    .add_textures(&device, &queue, &tdelta)
                    .expect("add texture ok");

                egui_rpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

                // Record all render passes.
                egui_rpass
                    .execute(
                        &mut encoder,
                        &output_view,
                        &paint_jobs,
                        &screen_descriptor,
                        Some(wgpu::Color::TRANSPARENT),
                    )
                    .unwrap();
                // Submit the commands.
                buffers.push(encoder.finish());

                queue.submit(buffers);

                // Redraw egui
                output_frame.present();

                egui_rpass
                    .remove_textures(tdelta)
                    .expect("remove texture ok");
            }
            MainEventsCleared | UserEvent(Event::RequestRedraw) => {
                window.request_redraw();
            }
            WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Resized(size) => {
                    // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                    // See: https://github.com/rust-windowing/winit/issues/208
                    // This solves an issue where the app would panic when minimizing on Windows.
                    if size.width > 0 && size.height > 0 {
                        surface_config.width = size.width;
                        surface_config.height = size.height;
                        surface.configure(&device, &surface_config);
                    }
                }
                winit::event::WindowEvent::CloseRequested => {
                    app.save();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => (),
        }
    });
}
