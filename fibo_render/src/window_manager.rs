use std::borrow::BorrowMut;

use sfml::{
    graphics::{
        Color, Image, IntRect, RcSprite, RcTexture, Rect, RectangleShape, RenderStates, RenderTarget, RenderTexture, RenderWindow, Shape, Transform, Transformable, VertexBufferUsage
    },
    window::Event,
};

use crate::gmp_utils::{mpz_int_from_u64, mpz_int_set_u64};

// use crate::fibo;
use crate::fibo_fast;

pub struct WindowManager {
    window: RenderWindow,
    current_sprite: RcSprite,
    current_texture: RcTexture,
    show_lines: bool,
    line_count: u32,
    pixel_size: f32,
    start_index: u64,
    start_p: u64,
    mode: u8,
    fibo: fibo_fast::FiboFastManager,
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
        WindowManager {
            window,
            current_sprite: RcSprite::new(),
            current_texture: RcTexture::new().unwrap(),
            show_lines: true,
            line_count: 10,
            pixel_size: 1.0 / 64.0,
            start_index: 1_000_000_000,
            start_p: 0,
            mode: 1,
            fibo: fibo_fast::FiboFastManager::new(),
        }
    }

    pub fn save_image(&self) {
        // Save current texture
        println!("Start image conversion...");
        match self.current_texture.copy_to_image() {
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

    fn generate_line(&mut self, n: u32) {
        // generate lines -x, -/2x, ... -1/nx
        for i in 1..n {
            let starty = 1.0 / i as f32 * self.start_index as f32 * self.pixel_size
                - self.start_p as f32 * self.pixel_size;
            let endy = 1.0 / i as f32
                * (self.start_index as f32 * self.pixel_size + self.window.size().x as f32)
                - self.start_p as f32 * self.pixel_size;
            let mut line = sfml::graphics::VertexBuffer::new(
                sfml::graphics::PrimitiveType::LINES,
                2,
                VertexBufferUsage::STATIC,
            );
            line.update(
                &[
                    sfml::graphics::Vertex::with_pos_color(
                        sfml::system::Vector2::new(0., starty),
                        sfml::graphics::Color::RED,
                    ),
                    sfml::graphics::Vertex::with_pos_color(
                        sfml::system::Vector2::new(self.window.size().x as f32, endy),
                        sfml::graphics::Color::RED,
                    ),
                ],
                0,
            );

            self.window.draw(&line);
        }
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

    fn fill_buffer(&mut self, buffer: &mut RenderTexture, x: u32, y: u32, size: u32) {
        let mut rect = RectangleShape::new();
        rect.set_size((size as f32, size as f32));
        rect.set_position(((x * size) as f32, (y * size) as f32));
        rect.set_fill_color(Color::WHITE);
        buffer.draw(
            &rect,
        );
    }

    fn generate_sequences_internal(&mut self, image_width: u32, image_height: u32) {
        let mut mpz_start = mpz_int_from_u64(self.start_index);

        let mut buffer = Image::new(image_width, image_height);
        let mut buffer2 = RenderTexture::new(image_width, image_height).unwrap();
        buffer2.clear(Color::BLACK);

        let upixel_size = self.pixel_size.ceil() as u32;

        // progress bar
        const SHOW_IMAGE_TIMES: u32 = 20;
        const PROGRESS_BAR_SIZE: u32 = 100;
        // print!("Progress: [{}] 0%", " ".repeat(PROGRESS_BAR_SIZE as usize));

        // Loop over the image size divided by the pixel size
        for y in 0..(image_height / upixel_size) {
            // Compute progress with a pow
            // let progress = (y as f32).powf(2.0) / (image_height as f32).powf(2.0);
            // print!(
            //     "\rProgress: [{}{}] {:.2}%",
            //     "#".repeat((progress * PROGRESS_BAR_SIZE as f32) as usize),
            //     " ".repeat(
            //         (PROGRESS_BAR_SIZE as f32 - progress * PROGRESS_BAR_SIZE as f32) as usize
            //     ),
            //     progress * 100.
            // );
            if (image_height / SHOW_IMAGE_TIMES) != 0 && y % (image_height / SHOW_IMAGE_TIMES) == 0
            {
                self.generate_texture(&buffer2, image_width, image_height);
                self.window.draw(&self.current_sprite);
                self.window.display();
            }
            let sequence = self.fibo.generate(
                ((y as f32) * (1.0 / self.pixel_size).ceil()) as u64 + self.start_p + 1,
                ((image_width as f32) * (1.0 / self.pixel_size).ceil()) as u64,
                self.start_index,
                mpz_start,
            );
            for x in 0..(image_width / upixel_size) {
                if sequence[((x as f32) * (1.0 / self.pixel_size).ceil()) as usize] {
                    self.fill_buffer(&mut buffer2, x, image_height / upixel_size - y, upixel_size);
                }
            }

            // Increment mpz_start
            mpz_int_set_u64(mpz_start.borrow_mut(), self.start_index + ((y as f32) * (1.0 / self.pixel_size).ceil()) as u64 + self.start_p + 1);
        }

        self.generate_texture(&buffer2, image_width, image_height);
    }

    fn generate_sequences(&mut self) {
        self.generate_sequences_internal(self.window.size().x, self.window.size().y);
    }

    fn generate_texture(&mut self, buffer: &RenderTexture, image_width: u32, image_height: u32) {
        if !self.current_texture.create(image_width, image_height) {
            println!("Error");
        }
        unsafe { self.current_texture.update_from_texture(buffer.texture(), 0, 0) };
        self.current_sprite = RcSprite::with_texture(&self.current_texture);
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
                            self.start_p += 100;
                            self.generate_sequences();
                        }
                        sfml::window::Key::Up => {
                            self.start_p = if self.start_p > 100 {
                                self.start_p - 100
                            } else {
                                0
                            };
                            self.generate_sequences();
                        }
                        sfml::window::Key::Right => {
                            self.start_index += 100;
                            self.generate_sequences();
                        }
                        sfml::window::Key::Left => {
                            self.start_index = if self.start_index > 100 {
                                self.start_index - 100
                            } else {
                                0
                            };
                            self.generate_sequences();
                        }
                        sfml::window::Key::Z => {
                            self.pixel_size *= 2.;
                            self.generate_sequences();
                        }
                        sfml::window::Key::S => {
                            self.pixel_size /= 2.;
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
            self.window.draw(&self.current_sprite);
            if self.show_lines {
                self.generate_line(self.line_count);
            }
            self.window.display();
        }
    }
}
