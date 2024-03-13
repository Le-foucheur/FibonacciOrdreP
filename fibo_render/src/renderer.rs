use std::borrow::BorrowMut;

use sfml::graphics::{
    Color, RcSprite, RcTexture, RectangleShape, RenderTarget, RenderTexture, RenderWindow, Shape,
    Transformable, VertexBufferUsage,
};

use crate::{fibo_fast, gmp_utils::{mpz_int_from_u64, mpz_int_set_u64}, progressbar};

pub struct Renderer {
    pub current_sprite: RcSprite,
    pub current_texture: RcTexture,
    pub pixel_size: f32,
    pub start_index: u64,
    pub start_p: u64,
    pub fibo: fibo_fast::FiboFastManager,
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

    pub fn fill_buffer(&mut self, buffer: &mut RenderTexture, x: u32, y: u32, size: u32) {
        let mut rect = RectangleShape::new();
        rect.set_size((size as f32, size as f32));
        rect.set_position(((x * size) as f32, (y * size) as f32));
        rect.set_fill_color(Color::WHITE);
        buffer.draw(&rect);
    }

    fn generate_texture(&mut self, buffer: &RenderTexture, image_width: u32, image_height: u32) {
        if !self.current_texture.create(image_width, image_height) {
            println!("Error");
        }
        unsafe {
            self.current_texture
                .update_from_texture(buffer.texture(), 0, 0)
        };
        self.current_sprite = RcSprite::with_texture(&self.current_texture);
    }

    pub fn generate_sequences_texture(
        &mut self,
        image_width: u32,
        image_height: u32,
        window: &mut RenderWindow,
    ) {
        let mut mpz_start = mpz_int_from_u64(self.start_index);

        // Initialize buffer
        let mut buffer = RenderTexture::new(image_width, image_height).unwrap();
        buffer.clear(Color::BLACK);

        // Round pixel size for easier computation
        let upixel_size = self.pixel_size.ceil() as u32;

        // progress bar
        const SHOW_IMAGE_TIMES: u32 = 20;
        let mut progressbar = progressbar::Progressbar::new();

        // Loop over the image size divided by the pixel size
        for y in 0..(image_height / upixel_size) {
            progressbar.update(y.pow(2) as f32 / (image_height / upixel_size).pow(2) as f32);
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
            for x in 0..(image_width / upixel_size) {
                if sequence[((x as f32) * (1.0 / self.pixel_size).ceil()) as usize] {
                    self.fill_buffer(&mut buffer, x, image_height / upixel_size - y, upixel_size);
                }
            }
            progressbar.clear();

            // Increment mpz_start
            mpz_int_set_u64(
                mpz_start.borrow_mut(),
                self.start_index
                    + ((y as f32) * (1.0 / self.pixel_size).ceil()) as u64
                    + self.start_p
                    + 1,
            );
        }

        self.generate_texture(&buffer, image_width, image_height);
    }
}