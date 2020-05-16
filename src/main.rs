#![feature(clamp)]
use glium::{Display, glutin::{window::WindowBuilder, event_loop::{ControlFlow, EventLoop}, ContextBuilder, event::{WindowEvent, Event, DeviceEvent, VirtualKeyCode, ElementState}}};
use std::{f32::consts::PI, time::{Instant, Duration}};
use render::initialize_render_state;
use cgmath::Vector3;
use cgmath::prelude::InnerSpace;

mod render;

pub struct GameState {
    camera_position: Vector3<f32>,
    camera_yaw: f32,
    camera_pitch: f32,

    movement: [f32; 2], // stores WASD input
}

fn update(delta: Duration, game: &mut GameState) {
    let mut forward_direction: Vector3<f32> = [
        game.camera_yaw.cos() * game.camera_pitch.cos(),
        game.camera_yaw.sin() * game.camera_pitch.cos(),
        game.camera_pitch.sin(),
    ].into();
    forward_direction = forward_direction.normalize();
    let right_direction: Vector3<f32> = forward_direction.cross([0.0, 0.0, 1.0].into()).normalize();

    let speed = 100.0;
    game.camera_position += forward_direction * game.movement[1] * speed * delta.as_secs_f32();
    game.camera_position -= right_direction   * game.movement[0] * speed * delta.as_secs_f32();
}

fn main() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new()
        .with_depth_buffer(24)
        .with_vsync(true);
    let display = Display::new(wb, cb, &event_loop).unwrap();
    display.gl_window().window().set_cursor_grab(true).unwrap();
    display.gl_window().window().set_cursor_visible(false);

    let render_state = initialize_render_state(&display);

    let mut game = GameState {
        camera_position: [2.0, -1.0, 1.0].into(),
        camera_yaw: 0.463,
        camera_pitch: 0.42,

        movement: [0.0, 0.0],
    };

    let mut previous_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                },
                _ => return,
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion{ delta } => {
                    game.camera_yaw += delta.0 as f32 * 0.006;
                    if game.camera_yaw >=  2.0*PI { game.camera_yaw -= 2.0*PI; }
                    // if game.camera_yaw <= -2.0*PI { game.camera_yaw += 2.0*PI; }

                    let freedom_y = 0.8;
                    game.camera_pitch += -delta.1 as f32 * 0.006;
                    game.camera_pitch = game.camera_pitch.clamp(-PI/2.0*freedom_y, PI/2.0*freedom_y);
                },
                DeviceEvent::Key(input) => match input.virtual_keycode {
                    Some(VirtualKeyCode::W) => game.movement[1] = if input.state == ElementState::Pressed {  1.0 } else { 0.0f32.min(game.movement[1]) },
                    Some(VirtualKeyCode::A) => game.movement[0] = if input.state == ElementState::Pressed { -1.0 } else { 0.0f32.max(game.movement[0]) },
                    Some(VirtualKeyCode::S) => game.movement[1] = if input.state == ElementState::Pressed { -1.0 } else { 0.0f32.max(game.movement[1]) },
                    Some(VirtualKeyCode::D) => game.movement[0] = if input.state == ElementState::Pressed {  1.0 } else { 0.0f32.min(game.movement[0]) },
                    _ => return,
                },
                _ => return,
            }
            Event::MainEventsCleared => {
                let delta = previous_frame_time.elapsed();
                previous_frame_time = Instant::now();
                update(delta, &mut game);

                display.gl_window().window().request_redraw();
            },
            Event::RedrawRequested(..) => {
                render::render(&display, &render_state, &game);
            },
            _ => return,
        }
    });
}


