#![feature(clamp)]
use glam::Vec3;
use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};
use glfw::Context;

mod error;
use error::AyudeError;

mod graphics;
#[allow(non_snake_case)]
mod gltf;
mod render;

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

fn main() {
    let mut glfw = glfw::init(Some(glfw::FAIL_ON_ERRORS.unwrap())).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
    glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(false));
    glfw.window_hint(glfw::WindowHint::OpenGlDebugContext(true));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));
    //glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::OpenGlEs));

    let (mut window, mut events) = glfw.create_window(
        800, 600,
        "a.yude",
        glfw::WindowMode::Windowed
    ).unwrap();

    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_cursor_mode(glfw::CursorMode::Disabled);
    window.make_current();

    gl::load_with(|s| window.get_proc_address(s));
    let mut render_state = render::RenderState::new();

    let mut game = GameState {
        camera_position: [0.0, 0.0, 0.0].into(),
        camera_yaw: 0.0,
        camera_pitch: 0.0,

        movement: [0.0, 0.0],
    };

    let mut previous_frame_time = Instant::now();

    'running: while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Close => {
                    break 'running;
                },
                glfw::WindowEvent::CursorPos(x, y) => {
                    let delta = (x, y);
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
                },
                glfw::WindowEvent::Key(key, _scancode, action, _modifiers) => match key {
                    glfw::Key::W => {
                        game.movement[1] = if action == glfw::Action::Press {
                            1.0
                        } else {
                            0.0f32.min(game.movement[1])
                        }
                    }
                    glfw::Key::A => {
                        game.movement[0] = if action == glfw::Action::Press {
                            -1.0
                        } else {
                            0.0f32.max(game.movement[0])
                        }
                    }
                    glfw::Key::S => {
                        game.movement[1] = if action == glfw::Action::Press {
                            -1.0
                        } else {
                            0.0f32.max(game.movement[1])
                        }
                    }
                    glfw::Key::D => {
                        game.movement[0] = if action == glfw::Action::Press {
                            1.0
                        } else {
                            0.0f32.min(game.movement[0])
                        }
                    }
                    _ => { },
                },
                _ => return,
            }
        }
        let delta = previous_frame_time.elapsed();
        previous_frame_time = Instant::now();
        update(delta, &mut game);

        render_state.render(&game, window.get_framebuffer_size());
        window.swap_buffers();
    }
}
