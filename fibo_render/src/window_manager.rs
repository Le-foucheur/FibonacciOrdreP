use sfml::{
    graphics::{RenderTarget, RenderWindow},
    window::{Event, Key},
};

use crate::{constants::MOVE_STEP, renderer::Renderer};

pub struct WindowManager {
    window: RenderWindow,
    show_lines: bool,
    line_count: u32,
    renderer: Renderer,
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
            show_lines: true,
            line_count: 10,
            renderer,
        }
    }

    fn generate_sequences(&mut self) {
        self.renderer.generate_sequences_texture(
            self.window.size().x,
            self.window.size().y,
            Some(&mut self.window),
        );
    }

    pub fn run(&mut self) {
        self.generate_sequences();
        while self.window.is_open() {
            while let Some(ev) = self.window.poll_event() {
                match ev {
                    Event::Closed => self.window.close(),
                    Event::Resized { width, height } => {
                        self.window.set_view(&sfml::graphics::View::new(
                            sfml::system::Vector2::new(width as f32 / 2.0, height as f32 / 2.0),
                            sfml::system::Vector2::new(width as f32, height as f32),
                        ));
                        self.generate_sequences();
                    }
                    Event::KeyPressed { code, .. } => match code {
                        sfml::window::Key::P => {
                            self.renderer.save_image();
                        }
                        sfml::window::Key::Down => {
                            self.renderer.start_p += MOVE_STEP;
                            self.generate_sequences();
                        }
                        sfml::window::Key::Up => {
                            self.renderer.start_p = if self.renderer.start_p > MOVE_STEP {
                                self.renderer.start_p - MOVE_STEP
                            } else {
                                0
                            };
                            self.generate_sequences();
                        }
                        sfml::window::Key::Right => {
                            self.renderer.start_index += MOVE_STEP;
                            self.generate_sequences();
                        }
                        sfml::window::Key::Left => {
                            self.renderer.start_index = if self.renderer.start_index > MOVE_STEP {
                                self.renderer.start_index - MOVE_STEP
                            } else {
                                0
                            };
                            self.generate_sequences();
                        }
                        sfml::window::Key::Z => {
                            self.renderer.pixel_size *= 2.;
                            self.generate_sequences();
                        }
                        sfml::window::Key::S => {
                            self.renderer.pixel_size /= 2.;
                            self.generate_sequences();
                        }
                        sfml::window::Key::L => {
                            self.show_lines = !self.show_lines;
                        }
                        sfml::window::Key::M => {
                            self.renderer.change_mode();
                            self.generate_sequences();
                        }
                        sfml::window::Key::Q => {
                            if Key::LControl.is_pressed() {
                                self.window.close();
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
            self.window.draw(&self.renderer.current_sprite);
            if self.show_lines {
                self.renderer
                    .generate_line(&mut self.window, self.line_count);
            }
            self.window.display();
        }
    }
}
