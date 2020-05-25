#![feature(clamp)]
use glam::Vec3;
use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};
use winit::{event::{WindowEvent, Event, DeviceEvent, VirtualKeyCode, ElementState}, event_loop::ControlFlow};

mod opengl;
#[allow(non_snake_case)]
mod gltf;
mod render;
mod texture_repository;

pub struct GameState {
    camera_position: Vec3,
    camera_yaw: f32,
    camera_pitch: f32,

    movement: [f32; 2], // stores WASD input
}

fn update(delta: Duration, game: &mut GameState) {
    let mut forward_direction: Vec3 = [
        game.camera_yaw.cos() * game.camera_pitch.cos(),
        game.camera_yaw.sin() * game.camera_pitch.cos(),
        game.camera_pitch.sin(),
    ]
    .into();
    forward_direction = forward_direction.normalize();
    let right_direction: Vec3 = forward_direction.cross([0.0, 0.0, 1.0].into()).normalize();

    let speed = 100.0;
    game.camera_position += forward_direction * game.movement[1] * speed * delta.as_secs_f32();
    game.camera_position -= right_direction * game.movement[0] * speed * delta.as_secs_f32();
}

type GetProcAddress = fn(&'static str) -> *const std::os::raw::c_void;
fn get_get_proc_address(window: &winit::window::Window) -> Option<GetProcAddress> {
    unsafe {
        use raw_window_handle::*;
        let raw_window_handle = window.raw_window_handle();
    
        match raw_window_handle {
            RawWindowHandle::Windows(windows::WindowsHandle{ hwnd, hinstance, .. }) => {
                use winapi::um::wingdi::*;
                use winapi::um::winuser::*;
                use winapi::shared::windef::*;
        
                let hwnd = hwnd as HWND;
                let hdc = GetDC(hwnd);
        
                let pfd = PIXELFORMATDESCRIPTOR {
                    nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
                    nVersion: 1,
                    dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
                    iPixelType: PFD_TYPE_RGBA,
                    cColorBits: 32,
                    cDepthBits: 24,
                    cStencilBits: 8,
                    iLayerType: PFD_MAIN_PLANE,
                    ..std::mem::zeroed()
                };
                let pfd_id = ChoosePixelFormat(hdc, &pfd);
                debug_assert!(pfd_id != 0);
                SetPixelFormat(hdc, pfd_id, &pfd);
    
                let mut chosen_pfd: PIXELFORMATDESCRIPTOR = std::mem::zeroed();
                DescribePixelFormat(hdc, pfd_id, std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u32, &mut chosen_pfd);
        
                let dummy_gl_context = wglCreateContext(hdc);
                let dummy_gl_context_make_current_result = wglMakeCurrent(hdc, dummy_gl_context);
                debug_assert!(!dummy_gl_context.is_null());
                debug_assert!(dummy_gl_context_make_current_result != 0);
    
                fn get_proc_address(symbol: &'static str) -> *const std::os::raw::c_void {
                    unsafe {
                        let c = format!("{}{}", symbol, "\0");
                        let ptr = wglGetProcAddress(std::mem::transmute(c.as_bytes().as_ptr())) as *const std::os::raw::c_void;
                        println!("for {}, the result was {:?}", symbol, ptr);
                        ptr
                    }
                }
    
                let wglGetExtensionsStringARB: extern fn(HDC) -> *mut u8 = std::mem::transmute(get_proc_address("wglGetExtensionsStringARB"));
    
                let extensions = (wglGetExtensionsStringARB)(hdc);
                // WGL_ARB_create_context is in there
    
                let wglCreateContextAttribsARB: extern fn(HDC, HGLRC, *const i32) -> HGLRC = 
                    std::mem::transmute(get_proc_address("wglCreateContextAttribsARB"));
    
                // https://www.khronos.org/registry/OpenGL/extensions/ARB/WGL_ARB_create_context.txt
                let gl_attributes = [
                    0x2091, 3,      // WGL_CONTEXT_MAJOR_VERSION_ARB, 3,
                    0x2092, 3,      // WGL_CONTEXT_MINOR_VERSION_ARB, 3,
                    0x2094, 0x0001, // WGL_CONTEXT_FLAGS_ARB, WGL_CONTEXT_DEBUG_BIT_ARB,
                    0x9126, 0x0001, // WGL_CONTEXT_PROFILE_MASK_ARB, WGL_CONTEXT_CORE_PROFILE_BIT_ARB,
                    0,
                ];
    
                let gl_context = (wglCreateContextAttribsARB)(hdc, std::ptr::null_mut(), &gl_attributes as *const i32);
                debug_assert!(!gl_context.is_null());
                wglMakeCurrent(hdc, gl_context);
    
                let wglSwapIntervalEXT: extern fn(i32) -> bool = std::mem::transmute(get_proc_address("wglSwapIntervalEXT"));
                
                gl::load_with(get_proc_address);
                //gl::TexParameteri::load_with(get_proc_address);
    
                (wglSwapIntervalEXT)(1);
        
                wglDeleteContext(dummy_gl_context);
                Some(get_proc_address)
            },
            _ => None
        }
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .build(&event_loop)
        .unwrap();
    get_get_proc_address(&window).unwrap();

    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);

    let mut render_state = render::RenderState::new();

    let mut game = GameState {
        camera_position: [2.0, -1.0, 1.0].into(),
        camera_yaw: 0.463,
        camera_pitch: 0.42,

        movement: [0.0, 0.0],
    };
    let mut game = GameState {
        camera_position: [0.0, 0.0, 0.0].into(),
        camera_yaw: 0.0,
        camera_pitch: 0.0,

        movement: [0.0, 0.0],
    };

    let mut previous_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => return,
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    game.camera_yaw += delta.0 as f32 * 0.006;
                    if game.camera_yaw >= 2.0 * PI {
                        game.camera_yaw -= 2.0 * PI;
                    }
                    // if game.camera_yaw <= -2.0*PI { game.camera_yaw += 2.0*PI; }

                    let freedom_y = 0.8;
                    game.camera_pitch += -delta.1 as f32 * 0.006;
                    game.camera_pitch = game
                        .camera_pitch
                        .clamp(-PI / 2.0 * freedom_y, PI / 2.0 * freedom_y);
                }
                DeviceEvent::Key(input) => match input.virtual_keycode {
                    Some(VirtualKeyCode::W) => {
                        game.movement[1] = if input.state == ElementState::Pressed {
                            1.0
                        } else {
                            0.0f32.min(game.movement[1])
                        }
                    }
                    Some(VirtualKeyCode::A) => {
                        game.movement[0] = if input.state == ElementState::Pressed {
                            -1.0
                        } else {
                            0.0f32.max(game.movement[0])
                        }
                    }
                    Some(VirtualKeyCode::S) => {
                        game.movement[1] = if input.state == ElementState::Pressed {
                            -1.0
                        } else {
                            0.0f32.max(game.movement[1])
                        }
                    }
                    Some(VirtualKeyCode::D) => {
                        game.movement[0] = if input.state == ElementState::Pressed {
                            1.0
                        } else {
                            0.0f32.min(game.movement[0])
                        }
                    }
                    _ => return,
                },
                _ => return,
            },
            Event::MainEventsCleared => {
                let delta = previous_frame_time.elapsed();
                previous_frame_time = Instant::now();
                update(delta, &mut game);

                todo!("display.gl_window().window().request_redraw();");
            }
            Event::RedrawRequested(..) => {
                todo!("render::render(&display, &mut render_state, &game);");
            }
            _ => return,
        }
    });
}
