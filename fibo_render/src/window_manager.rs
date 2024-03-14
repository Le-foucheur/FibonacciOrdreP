use sfml::{
    graphics::{RenderTarget, RenderWindow},
    window::{Event, Key},
};

use crate::{constants::MOVE_STEP, command_line::HELP_MESSAGE, renderer::Renderer};

pub struct WindowManager {
    window: RenderWindow,
    renderer: Renderer,
}

pub fn generate_sequences(window: &mut RenderWindow, renderer: &mut Renderer) {
    renderer.generate_sequences_texture(
        window.size().x,
        window.size().y,
        Some(window),
    );
}

pub fn manage_events(window: &mut RenderWindow, renderer: &mut Renderer) -> u8 {
    let mut result = 0;
    while let Some(ev) = window.poll_event() {
        match ev {
            Event::Closed => window.close(),
            Event::Resized { width, height } => {
                window.set_view(&sfml::graphics::View::new(
                    sfml::system::Vector2::new(width as f32 / 2.0, height as f32 / 2.0),
                    sfml::system::Vector2::new(width as f32, height as f32),
                ));
                result = 1;
            }
            Event::KeyPressed { code, .. } => match code {
                sfml::window::Key::P => {
                    renderer.save_image();
                }
                sfml::window::Key::Down => {
                    renderer.start_p += MOVE_STEP;
                    result = 1;
                }
                sfml::window::Key::Up => {
                    renderer.start_p = if renderer.start_p > MOVE_STEP {
                        renderer.start_p - MOVE_STEP
                    } else {
                        0
                    };
                    result = 1;
                }
                sfml::window::Key::Right => {
                    renderer.start_index += MOVE_STEP;
                    result = 1;
                }
                sfml::window::Key::Left => {
                    renderer.start_index = if renderer.start_index > MOVE_STEP {
                        renderer.start_index - MOVE_STEP
                    } else {
                        0
                    };
                    result = 1;
                }
                sfml::window::Key::Z => {
                    renderer.pixel_size *= 2.;
                    result = 1;
                }
                sfml::window::Key::S => {
                    renderer.pixel_size /= 2.;
                    result = 1;
                }
                sfml::window::Key::L => {
                    renderer.show_lines = !renderer.show_lines;
                }
                sfml::window::Key::M => {
                    renderer.change_mode();
                    result = 1;
                }
                sfml::window::Key::Q => {
                    if Key::LControl.is_pressed() {
                        window.close();
                    }
                }
                sfml::window::Key::H => {
                    println!("{}", HELP_MESSAGE);
                }
                _ => {}
            },
            _ => {}
        }
    }
    return result;
}

impl WindowManager {
    pub fn new(renderer: Renderer) -> WindowManager {
        let mut window = RenderWindow::new(
            (300, 10),
            "Fibonacci sequence modulo 2",
            sfml::window::Style::DEFAULT,
            &Default::default(),
        );
        window.set_vertical_sync_enabled(true);
        
        WindowManager {
            window,
            renderer,
        }
    }

    pub fn run(&mut self) {
        generate_sequences(&mut self.window,&mut self.renderer);
        while self.window.is_open() {
            if manage_events(&mut self.window, &mut self.renderer) == 1 {
                generate_sequences(&mut self.window, &mut self.renderer);
            }
            self.window.draw(&self.renderer.current_sprite);
            if self.renderer.show_lines {
                self.renderer
                    .generate_line(&mut self.window, self.renderer.line_count);
            }
            self.window.display();
        }
    }
}
