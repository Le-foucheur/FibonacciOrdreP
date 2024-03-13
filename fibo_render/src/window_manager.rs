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
        let renderer = Renderer::new(1.0 / 64.0, 1_000_000_000, 0);
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

    // fn generate_sequences(&mut self) {
    //     let now = Instant::now();
    //     let mut generation_time = Duration::new(0, 0);

    //     let window_width = self.window.size().x;
    //     let window_height = self.window.size().y;
    //     let sequence_width = (window_width as f32 / self.pixel_size).ceil() as u32;

    //     let sequence_size = 1. / self.pixel_size;

    //     let mut mpz_start = mpz_int_from_u64(self.start_index);

    //     println!(
    //         "Start generation with min_p: {}, max_p={}, min_n={}, max_n={}",
    //         self.start_p,
    //         self.start_p + (window_height as f32 * sequence_size) as u64,
    //         self.start_index,
    //         self.start_index + (window_width as f32 * sequence_size) as u64
    //     );

    //     const SHOW_IMAGE_TIMES: u32 = 20;
    //     // Create buffer to draw the sequence
    //     let mut buffer = Image::new(window_width, window_height);

    //     // progress bar
    //     const PROGRESS_BAR_SIZE: u32 = 100;
    //     print!("Progress: [{}] 0%", " ".repeat(PROGRESS_BAR_SIZE as usize));
    //     match self.mode {
    //         1 => {
    //             // take only a cell and skip the others
    //             for y in 0..window_height {
    //                 // Compute progress with a pow
    //                 let progress = (y as f32).powf(2.0) / (window_height as f32).powf(2.0);
    //                 print!(
    //                     "\rProgress: [{}{}] {:.2}%",
    //                     "#".repeat((progress * PROGRESS_BAR_SIZE as f32) as usize),
    //                     " ".repeat(
    //                         (PROGRESS_BAR_SIZE as f32 - progress * PROGRESS_BAR_SIZE as f32)
    //                             as usize
    //                     ),
    //                     progress * 100.
    //                 );
    //                 if (window_height / SHOW_IMAGE_TIMES) != 0
    //                     && y % (window_height / SHOW_IMAGE_TIMES) == 0
    //                 {
    //                     self.gen_texture(&buffer);
    //                     self.window.draw(&self.current_sprite);
    //                     self.window.display();
    //                 }
    //                 let generation_now = Instant::now();
    //                 let sequence = self.fibo.generate(
    //                     (y as f32 * sequence_size).floor() as u64 + self.start_p + 1,
    //                     sequence_width as u64 + self.start_index,
    //                     self.start_index,
    //                     mpz_start,
    //                 );
    //                 unsafe { mpz_add_ui(mpz_start.borrow_mut(), mpz_start.borrow(), sequence_size.floor() as u64) };
    //                 generation_time += generation_now.elapsed();
    //                 for x in 0..window_width {
    //                     if sequence[(x as f32 * sequence_size).floor() as usize] {
    //                         // Draw a pixel with pixel_size
    //                         unsafe {
    //                             buffer.set_pixel(x, y, Color::WHITE);
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    //     unsafe { mpz_clear(mpz_start.borrow_mut()) };
    //     print!("\r\x1b[K");

    //     // Render the buffer
    //     self.gen_texture(&buffer);
    //     println!("Time to generate sequence: {:?}", generation_time);
    //     println!(
    //         "Time to draw sequence: {:?}",
    //         now.elapsed() - generation_time
    //     );
    // }

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
                            self.renderer.start_index += 100;
                            self.generate_sequences();
                        }
                        sfml::window::Key::Left => {
                            self.renderer.start_index = if self.renderer.start_index > 100 {
                                self.renderer.start_index - 100
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
                        sfml::window::Key::A => {
                            self.line_count += 1;
                        }
                        sfml::window::Key::Q => {
                            self.line_count = if self.line_count > 1 {
                                self.line_count - 1
                            } else {
                                1
                            };
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
