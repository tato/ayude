use ayude::{
    camera::Camera,
    graphics::{self, GraphicsContext, Material, Mesh},
    import_gltf,
    transform::{Transform, GLOBAL_UP},
    Scene,
};
use glam::{Mat4, Vec2, Vec3};
use rusttype::{Font, Scale};
use std::{
    borrow::Cow,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct World {
    camera: Camera,

    movement: [f32; 2], // stores WASD input

    the_scene: Scene,
    the_sphere: Scene,

    ricardo: graphics::Texture,

    rendering_skin: bool,

    graphics: GraphicsContext,
}

impl World {
    fn new(renderer: GraphicsContext) -> Self {
        let gltf_file_name = "samples/knight/knight.gltf";
        // let gltf_file_name = "samples/principito_y_el_aviador/scene.gltf";
        let the_entity = import_gltf::import_default_scene(gltf_file_name, &renderer).unwrap();

        let the_sphere =
            import_gltf::import_default_scene("samples/sphere.gltf", &renderer).unwrap();

        // let ricardo = {
        //     let file = std::fs::read("samples/ricardo.jpg").unwrap();
        //     let image = image::load_from_memory(&file).unwrap();
        //     let image = image.into_rgba();
        //     graphics::Texture::builder(
        //         image.as_bytes(),
        //         image.width() as u16,
        //         image.height() as u16,
        //         graphics::texture::TextureFormat::RGBA,
        //     )
        //     .build()
        // };

        let camera = Camera::new(Vec3::from([0.0, 0.0, 37.0]), std::f32::consts::PI, 0.0);

        let test_font_texture = {
            let data = std::fs::read("data/Cousine.ttf").expect("font file should exist");
            let font = Font::try_from_vec(data).expect("font should load");

            let height: f32 = 12.4;
            let pixel_height = height.ceil() as usize;

            let scale = Scale {
                x: height * 2.0,
                y: height,
            };

            let v_metrics = font.v_metrics(scale);
            let offset = rusttype::point(0.0, v_metrics.ascent);

            let glyphs: Vec<_> = font.layout("RIGHT NOW.", scale, offset).collect();

            let width = glyphs
                .iter()
                .rev()
                .map(|g| g.position().x as f32 + g.unpositioned().h_metrics().advance_width)
                .next()
                .unwrap_or(0.0)
                .ceil() as usize;

            let mut pixel_data = vec![0u8; width * pixel_height * 4];
            for g in glyphs {
                if let Some(bb) = g.pixel_bounding_box() {
                    g.draw(|x, y, v| {
                        let gray = (v * 255.5) as u8;
                        let x = x as i32 + bb.min.x;
                        let y = y as i32 + bb.min.y;
                        if x >= 0 && x < width as i32 && y >= 0 && y < pixel_height as i32 {
                            let i = (y as usize * width + x as usize) * 4;
                            pixel_data[i] = gray;
                            pixel_data[i + 1] = gray;
                            pixel_data[i + 2] = gray;
                            pixel_data[i + 3] = 255;
                        }
                    });
                }
            }

            renderer.create_texture(
                &pixel_data,
                width as u32,
                pixel_height as u32,
                wgpu::TextureFormat::Rgba8Unorm,
            )
        };

        let world = World {
            camera,

            movement: [0.0, 0.0],

            the_scene: the_entity,
            the_sphere,

            ricardo: test_font_texture,

            rendering_skin: false,

            graphics: renderer,
        };

        world
    }

    fn update(&mut self, delta: Duration) {
        let mov = Vec2::from(self.movement) * delta.as_secs_f32();
        self.camera.drive(mov);
    }

    fn render(&mut self, window_dimensions: (i32, i32)) {
        let frame = self.graphics.get_current_frame();

        let perspective = glam::Mat4::perspective_rh_gl(
            std::f32::consts::PI / 3.0,
            window_dimensions.0 as f32 / window_dimensions.1 as f32,
            0.1,
            1024.0,
        );

        let view = self.camera.view();

        let mut encoder = self
            .graphics
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            if !self.rendering_skin {
                self.the_scene
                    .render(&self.graphics, perspective, view, &mut pass);
                let translation = Vec3::new(-1.0, -1.0, 0.0);
                self.graphics.render_billboard(
                    &self.ricardo,
                    &mut pass,
                    translation,
                    perspective,
                    &self.camera,
                );
            } else {
                // let scene = &self.the_scene;
                // for node in &scene.nodes {
                //     let skin = match node.skin.as_ref() {
                //         Some(skin) => skin,
                //         None => continue,
                //     };

                //     let skeleton_transform = match skin.skeleton {
                //         Some(skeleton) => Transform::from(
                //             scene.nodes[usize::from(skeleton)].transform.mat4().clone(),
                //         ),
                //         None => scene.transform.clone(),
                //     };

                //     for &joint in &skin.joints {
                //         let joint = &scene.nodes[usize::from(joint)];

                //         let mut transform = joint.transform.mat4().clone();
                //         let mut current = joint;
                //         'transform: loop {
                //             match current.parent {
                //                 Some(index) => current = &scene.nodes[usize::from(index)],
                //                 None => break 'transform,
                //             }
                //             transform = transform.mul_mat4(current.transform.mat4());
                //         }

                //         self.the_sphere.transform = Transform::from(
                //             transform.mul_mat4(skeleton_transform.mat4())
                //                 * Mat4::from_scale(Vec3::new(0.25, 0.25, 0.25)),
                //         );

                //         self.renderer
                //             .render_scene(&self.the_sphere, &frame, &perspective, &view);
                //     }
                // }
            };
        }
        self.graphics.queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("a.yude")
        .with_inner_size(LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)
        .expect("Failed to open window.");

    window.set_cursor_grab(true).unwrap();
    window.set_cursor_visible(false);

    let window: Arc<Window> = window.into();

    set_panic_hook(window.clone());

    pollster::block_on(async_main(event_loop, window));
}

async fn async_main(event_loop: EventLoop<()>, window: Arc<Window>) {
    let renderer = GraphicsContext::new(&window).await;

    let mut game = World::new(renderer);

    let mut previous_frame_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    game.graphics.resize(size.width, size.height);
                }
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => return,
            },
            Event::DeviceEvent { event, .. } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    game.camera
                        .rotate(Vec2::new(delta.0 as f32, delta.1 as f32) / 150.0);
                }
                DeviceEvent::Key(input) => match input.virtual_keycode {
                    Some(VirtualKeyCode::W) => {
                        if input.state == ElementState::Pressed {
                            game.movement[1] = 1.0;
                        } else if input.state == ElementState::Released {
                            game.movement[1] = 0.0f32.min(game.movement[1]);
                        }
                    }
                    Some(VirtualKeyCode::A) => {
                        if input.state == ElementState::Pressed {
                            game.movement[0] = -1.0;
                        } else if input.state == ElementState::Released {
                            game.movement[0] = 0.0f32.max(game.movement[0]);
                        }
                    }
                    Some(VirtualKeyCode::S) => {
                        if input.state == ElementState::Pressed {
                            game.movement[1] = -1.0;
                        } else if input.state == ElementState::Released {
                            game.movement[1] = 0.0f32.max(game.movement[1]);
                        }
                    }
                    Some(VirtualKeyCode::D) => {
                        if input.state == ElementState::Pressed {
                            game.movement[0] = 1.0;
                        } else if input.state == ElementState::Released {
                            game.movement[0] = 0.0f32.min(game.movement[0]);
                        }
                    }
                    Some(VirtualKeyCode::Tab) if input.state == ElementState::Pressed => {
                        game.rendering_skin = !game.rendering_skin;
                    }
                    _ => return,
                },
                _ => return,
            },
            Event::MainEventsCleared => {
                let delta = previous_frame_time.elapsed();
                previous_frame_time = Instant::now();
                game.update(delta);
                window.request_redraw();
            }
            Event::RedrawRequested(..) => {
                game.render(get_window_dimensions(&window));
            }
            _ => return,
        }
    });
}

fn get_window_dimensions(window: &Window) -> (i32, i32) {
    let inner_size = window.inner_size();
    (inner_size.width as i32, inner_size.height as i32)
}

fn set_panic_hook(window: Arc<Window>) {
    std::panic::set_hook(Box::new(move |panic_info| {
        window.set_cursor_grab(false).unwrap();
        window.set_cursor_visible(true);

        let mut lines = vec![];
        if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            lines.push(message.to_string());
        }
        if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            lines.push(message.to_string());
        }
        if let Some(location) = panic_info.location() {
            let loc = format!(
                "[{},{}] {}",
                location.line(),
                location.column(),
                location.file()
            );
            lines.push(loc);
        }

        msgbox::create("Error", &lines.join("\n"), msgbox::IconType::Error).unwrap_or_else(|_| {
            println!("{}", lines.join("\n"));
        })
    }));
}
