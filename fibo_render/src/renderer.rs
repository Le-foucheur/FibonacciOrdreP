use sfml::graphics::{
    Color, Image, RcSprite, RcTexture, RenderTarget, RenderWindow, Shape, Transformable,
    VertexBufferUsage,
};

use crate::fibo_fast::init_serie;
use crate::window_manager::manage_events;
use crate::{constants::SHOW_IMAGE_TIMES, fibo_fast, gmp_utils::mpz_int_from_u64, progressbar};

pub struct Renderer {
    pub current_sprite: RcSprite,
    pub current_texture: RcTexture,
    pub pixel_size: f32,
    pub start_index: u64,
    pub start_p: u64,
    pub fibo: fibo_fast::FiboFastManager,
    pub mode: u8,
    pub show_lines: bool,
    pub line_count: u32,
    pub mouse_x: i32,
    pub mouse_y: i32,
}

impl Renderer {
    pub fn new(pixel_size: f32, start_index: u64, start_p: u64, mode: u8) -> Renderer {
        Renderer {
            current_sprite: RcSprite::new(),
            current_texture: RcTexture::new().unwrap(),
            pixel_size,
            start_index,
            start_p,
            fibo: fibo_fast::FiboFastManager::new(),
            mode,
            show_lines: true,
            line_count: 10,
            mouse_x: 0,
            mouse_y: 0,
        }
    }

    pub fn generate_line(&mut self, window: &mut RenderWindow, n: u32) {
        // generate lines -x, -/2x, ... -1/nx
        for i in 1..n {
            let starty = 1.0 / i as f32 * self.start_index as f32 * self.pixel_size
                - self.start_p as f32 * self.pixel_size;
            let endy = 1.0 / i as f32
                * (self.start_index as f32 * self.pixel_size + window.size().x as f32)
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
                        sfml::system::Vector2::new(window.size().x as f32, endy),
                        sfml::graphics::Color::RED,
                    ),
                ],
                0,
            );

            window.draw(&line);
        }
    }

    pub fn fill_buffer(&mut self, buffer: &mut Image, x: f32, y: f32, size: f32, color: f32) {
        for i in 0..size as u32 {
            for j in 0..size as u32 {
                unsafe {
                    buffer.set_pixel(
                        (x * size + i as f32) as u32,
                        (y * size + j as f32) as u32,
                        Color::rgb(
                            (255.0 * color).ceil() as u8,
                            (255.0 * color).ceil() as u8,
                            (255.0 * color).ceil() as u8,
                        ),
                    );
                }
            }
        }
    }

    fn generate_texture(&mut self, buffer: &Image, image_width: u32, image_height: u32) {
        if !self.current_texture.create(image_width, image_height) {
            panic!("Error creating texture");
        }
        unsafe { self.current_texture.update_from_image(buffer, 0, 0) };
        self.current_sprite = RcSprite::with_texture(&self.current_texture);
    }

    pub fn generate_sequences_texture(
        &mut self,
        image_width: u32,
        image_height: u32,
        mut window: Option<&mut RenderWindow>,
    ) -> bool {
        let texture_generation_time = std::time::Instant::now();
        // Round pixel size for easier computation
        let upixel_size = self.pixel_size.ceil() as f32;

        let delta_n = (image_width as f32 / self.pixel_size).floor() as u64;
        let delta_p = (((image_height as f32 / upixel_size).floor() - 1.0)
            * (1.0 / self.pixel_size).ceil()) as u64
            + 1;
        println!(
            "Start generating texture with pixel size: {}, n: {}, p: {}, delta_n: {}, delta_p: {}",
            self.pixel_size, self.start_index, self.start_p, delta_n, delta_p
        );

        // Initialize the mpz at the right side of the generation
        let mpz_start = mpz_int_from_u64(
            self.start_index + image_width as u64 * (1.0 / self.pixel_size).ceil() as u64 - 1,
        );

        // Initialize buffer
        let mut buffer = Image::new(image_width, image_height);
        
        init_serie(((image_height as f32 / upixel_size).floor() * (1.0 / self.pixel_size).ceil()) as u64 + self.start_p + 1 - 1, mpz_start);

        let mut progressbar = progressbar::Progressbar::new();


        // Loop over the image size divided by the pixel size
        for y in 0_u32..(image_height as f32 / upixel_size).floor() as u32 {
            // Update progress bar and show the image sometimes
            progressbar.update(y.pow(2) as f32 / (image_height / upixel_size as u32).pow(2) as f32);
            progressbar.show();
            if window.is_some() {
                if manage_events(window.as_mut().unwrap(), self) == 1 {
                    progressbar.clear();
                    return true;
                    // generate_sequences(window.as_mut().unwrap(), self);
                    // return;
                }
                if (image_height / SHOW_IMAGE_TIMES) != 0
                    && y % (image_height / SHOW_IMAGE_TIMES) == 0
                {
                    self.generate_texture(&buffer, image_width, image_height);
                    window.as_mut().unwrap().draw(&self.current_sprite);
                    window.as_mut().unwrap().display();
                }
            }

            let current_p = ((y as f32) * (1.0 / self.pixel_size).ceil()) as u64 + self.start_p + 1;
            let sequence = self.fibo.generate(
                current_p,
                ((image_width as f32) * (1.0 / self.pixel_size).ceil()) as u64,
                mpz_start,
            );
            match self.mode {
                // Average over n
                0 => {
                    for x in 0..(image_width as f32 / upixel_size).floor() as u32 {
                        let mut sum = 0;
                        for i in 0..(1.0 / self.pixel_size).ceil() as usize {
                            sum += sequence
                                [((x as f32) * (1.0 / self.pixel_size).ceil()) as usize + i]
                                as u32;
                        }
                        self.fill_buffer(
                            &mut buffer,
                            x as f32,
                            y as f32,
                            upixel_size as f32,
                            sum as f32 / (1.0 / self.pixel_size).ceil() as f32,
                        );
                    }
                }
                // Take only once cell
                1 => {
                    for x in 0..(image_width as f32 / upixel_size).floor() as u32 {
                        if sequence[((x as f32) * (1.0 / self.pixel_size).ceil()) as usize] {
                            self.fill_buffer(
                                &mut buffer,
                                x as f32,
                                y as f32,
                                upixel_size as f32,
                                1.0,
                            );
                        }
                    }
                }
                // Average over n and p
                2 => {
                    let mut sequences = vec![];
                    for j in 0..(1.0 / self.pixel_size).ceil() as usize {
                        sequences.push(self.fibo.generate(
                            ((y as f32) * (1.0 / self.pixel_size).ceil()) as u64
                                + self.start_p
                                + 1
                                + j as u64,
                            ((image_width as f32) * (1.0 / self.pixel_size).ceil()) as u64,
                            mpz_start,
                        ));
                    }
                    for x in 0..(image_width as f32 / upixel_size).floor() as u32 {
                        let mut sum = 0;
                        for i in 0..(1.0 / self.pixel_size).ceil() as usize {
                            for j in 0..(1.0 / self.pixel_size).ceil() as usize {
                                sum += sequences[i]
                                    [((x as f32) * (1.0 / self.pixel_size).ceil()) as usize + j]
                                    as u32;
                            }
                        }
                        self.fill_buffer(
                            &mut buffer,
                            x as f32,
                            y as f32,
                            upixel_size as f32,
                            sum as f32 / (1.0 / (self.pixel_size * self.pixel_size)).ceil() as f32,
                        );
                    }
                }
                _ => {}
            }
            progressbar.clear();
        }
        self.generate_texture(&buffer, image_width, image_height);

        println!(
            "End generating texture in {:.2} seconds",
            texture_generation_time.elapsed().as_secs_f32()
        );
        return false;
    }

    pub fn change_mode(&mut self) {
        self.mode = (self.mode + 1) % 3;
    }

    pub fn save_image(&self) {
        // Save current texture
        println!("Start image conversion...");
        match self.current_texture.copy_to_image() {
            Some(image) => {
                println!("Saving image...");
                if image.save_to_file("fibo_sequence.png") {
                    println!("Image saved successfully as fibo_sequence.png");
                } else {
                    println!("Error while saving the image");
                }
            }
            None => {
                println!("Error while saving the image");
            }
        };
    }

    // Draw text in the right left corner of the window to show the n and p under the mouse
    pub fn draw_position(&mut self, window: &mut RenderWindow) {
        // Draw text
        let font = sfml::graphics::Font::from_file("assets/monospacebold.ttf").unwrap();
        let mut text = sfml::graphics::Text::new(
            &format!(
                "n: {}, p: {}",
                self.start_index + (self.mouse_x as f32 * (1.0 / self.pixel_size)).floor() as u64,
                self.start_p + (self.mouse_y as f32 * (1.0 / self.pixel_size)).floor() as u64
            ),
            &font,
            15,
        );
        text.set_fill_color(sfml::graphics::Color::WHITE);
        text.set_position(sfml::system::Vector2::new(
            window.size().x as f32 - text.local_bounds().width - 5.0,
            window.size().y as f32 - text.local_bounds().height - 5.0,
        ));
        // Draw a black rectangle to make the text more readable
        let mut rectangle = sfml::graphics::RectangleShape::new();
        rectangle.set_size(sfml::system::Vector2::new(
            text.local_bounds().width + 7.0,
            text.local_bounds().height + 4.0,
        ));
        rectangle.set_fill_color(sfml::graphics::Color::BLACK);
        rectangle.set_position(sfml::system::Vector2::new(
            window.size().x as f32 - text.local_bounds().width - 7.0,
            window.size().y as f32 - text.local_bounds().height - 4.0,
        ));
        window.draw(&rectangle);
        window.draw(&text);
    }
}
