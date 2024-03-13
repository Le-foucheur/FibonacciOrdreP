use sfml::graphics::{
    Color, Image, RcSprite, RcTexture, RenderTarget, RenderWindow, VertexBufferUsage,
};

use crate::{fibo_fast, gmp_utils::mpz_int_from_u64, progressbar};

const SHOW_IMAGE_TIMES: u32 = 5;

pub struct Renderer {
    pub current_sprite: RcSprite,
    pub current_texture: RcTexture,
    pub pixel_size: f32,
    pub start_index: u64,
    pub start_p: u64,
    pub fibo: fibo_fast::FiboFastManager,
    pub mode: u8,
}

impl Renderer {
    pub fn new(pixel_size: f32, start_index: u64, start_p: u64) -> Renderer {
        Renderer {
            current_sprite: RcSprite::new(),
            current_texture: RcTexture::new().unwrap(),
            pixel_size,
            start_index,
            start_p,
            fibo: fibo_fast::FiboFastManager::new(),
            mode: 0,
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
        window: &mut RenderWindow,
    ) {
        // Round pixel size for easier computation
        let upixel_size = self.pixel_size.ceil() as f32;

        let mpz_start = mpz_int_from_u64(
            self.start_index + image_width as u64 * (1.0 / self.pixel_size).ceil() as u64 - 1,
        );

        // Initialize buffer
        let mut buffer = Image::new(image_width, image_height);

        // progress bar
        let mut progressbar = progressbar::Progressbar::new();

        // Loop over the image size divided by the pixel size
        for y in 0_u32..(image_height as f32 / upixel_size).floor() as u32 {
            progressbar.update(y.pow(2) as f32 / (image_height / upixel_size as u32).pow(2) as f32);
            progressbar.show();
            if (image_height / SHOW_IMAGE_TIMES) != 0 && y % (image_height / SHOW_IMAGE_TIMES) == 0
            {
                self.generate_texture(&buffer, image_width, image_height);
                window.draw(&self.current_sprite);
                window.display();
            }

            let sequence = self.fibo.generate(
                ((y as f32) * (1.0 / self.pixel_size).ceil()) as u64 + self.start_p + 1,
                ((image_width as f32) * (1.0 / self.pixel_size).ceil()) as u64,
                mpz_start,
            );
            for x in 0..(image_width as f32 / upixel_size).floor() as u32 {
                match self.mode {
                    0 => {
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
                    1 => {
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
                    _ => {}
                }
            }
            progressbar.clear();
        }
        self.generate_texture(&buffer, image_width, image_height);
    }

    pub fn change_mode(&mut self) {
        self.mode = (self.mode + 1) % 2;
    }
}
