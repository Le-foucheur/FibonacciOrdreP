use sfml::{
    graphics::{RenderTarget, RenderWindow},
    window::Event,
};

use crate::renderer::Renderer;

pub struct WindowManager {
    window: RenderWindow,
    show_lines: bool,
    line_count: u32,
    renderer: Renderer,
}

impl WindowManager {
    pub fn new() -> WindowManager {
        let mut window = RenderWindow::new(
            (300, 10),
            "Fibonacci sequence modulo 2",
            sfml::window::Style::DEFAULT,
            &Default::default(),
        );
        window.set_vertical_sync_enabled(true);
        let renderer = Renderer::new(1.0 / 32.0 * 64.0, 000_000_000, 0);
        WindowManager {
            window,
            show_lines: true,
            line_count: 10,
            renderer,
        }
    }

    pub fn save_image(&self) {
        // Save current texture
        println!("Start image conversion...");
        match self.renderer.current_texture.copy_to_image() {
            Some(image) => {
                println!("Saving image...");
                if image.save_to_file("fibo_sequence.png") {
                    println!("Image saved successfully");
                } else {
                    println!("Error while saving the image");
                }
            }
            None => {
                println!("Error while saving the image");
            }
        };
    }

    fn generate_sequences(&mut self) {
        self.renderer.generate_sequences_texture(
            self.window.size().x,
            self.window.size().y,
            &mut self.window,
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
                            self.generate_sequences();
                            self.save_image();
                            self.generate_sequences();
                        }
                        sfml::window::Key::Down => {
                            self.renderer.start_p += 100;
                            self.generate_sequences();
                        }
                        sfml::window::Key::Up => {
                            self.renderer.start_p = if self.renderer.start_p > 100 {
                                self.renderer.start_p - 100
                            } else {
                                0
                            };
                            self.generate_sequences();
                        }
                        sfml::window::Key::Right => {
                            self.renderer.start_index += 10;
                            self.generate_sequences();
                        }
                        sfml::window::Key::Left => {
                            self.renderer.start_index = if self.renderer.start_index > 10 {
                                self.renderer.start_index - 10
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
