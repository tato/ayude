
use glium::{Display, glutin::{window::WindowBuilder, event_loop::{ControlFlow, EventLoop}, ContextBuilder, event::{WindowEvent, Event}}};
use std::time::{Instant, Duration};
use render::initialize_render_state;

mod render;

struct GameState {
    t: f32,
}

fn update(delta: Duration, game: &mut GameState) {
    game.t += delta.as_secs_f32();
    if game.t > 0.5 {
        game.t = -0.5;
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let wb = WindowBuilder::new();
    let cb = ContextBuilder::new()
        .with_depth_buffer(24)
        .with_vsync(true);
    let display = Display::new(wb, cb, &event_loop).unwrap();

    let render_state = initialize_render_state(&display);

    let mut game = GameState {
        t: -0.5,
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
            Event::MainEventsCleared => {
                let delta = previous_frame_time.elapsed();
                previous_frame_time = Instant::now();
                update(delta, &mut game);

                display.gl_window().window().request_redraw();
            },
            Event::RedrawRequested(..) => {
                render::render(&display, &render_state);
            },
            _ => return,
        }

        
    });
}
