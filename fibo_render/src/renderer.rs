use std::borrow::BorrowMut;

#[cfg(feature = "graphic")]
use crate::constants::SHOW_IMAGE_TIMES;
#[cfg(feature = "graphic")]
use crate::draw_utils::draw_line;
#[cfg(feature = "graphic")]
use crate::window_manager::manage_events;
use num::BigInt;
#[cfg(feature = "graphic")]
use sfml::graphics::{
    Color, Image, RcSprite, RcTexture, RenderTarget, RenderWindow, Shape, Transformable,
};

use crate::{fibo_fast, progressbar};

use itertools::Itertools;

pub struct Renderer {
    #[cfg(feature = "graphic")]
    pub current_sprite: RcSprite,
    #[cfg(feature = "graphic")]
    pub current_texture: RcTexture,
    pub pixel_size: f32,
    pub start_index: BigInt,
    pub start_p: usize,
    pub fibo: fibo_fast::FiboFastManager,
    pub mode: u8,
    pub show_lines: bool,
    pub line_count: u32,
    pub mouse_x: i32,
    pub mouse_y: i32,
}

impl Renderer {
    pub fn new(pixel_size: f32, start_index: BigInt, start_p: usize, mode: u8) -> Renderer {
        Renderer {
            #[cfg(feature = "graphic")]
            current_sprite: RcSprite::new(),
            #[cfg(feature = "graphic")]
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

    #[cfg(feature = "graphic")]
    pub fn generate_line(&mut self, window: &mut RenderWindow, n: u32) {
        // generate lines -x, -/2x, ... -1/nx
        // Check if the start index can be converted to u64
        // Initialize the mpz at the right side of the generation

        // Line in positive
        let start_i64_try: Result<i64, _> = self.start_index.clone().try_into();
        if let Ok(start_i64) = start_i64_try {
            if start_i64 >= 0 {
                for i in 1..n {
                    let starty = 1.0 / i as f32 * start_i64 as f32 * self.pixel_size
                        - self.start_p as f32 * self.pixel_size;
                    let endy = 1.0 / i as f32
                        * (start_i64 as f32 * self.pixel_size + window.size().x as f32)
                        - self.start_p as f32 * self.pixel_size;
                    draw_line(0., starty, window.size().x as f32, endy, window);
                }
            }
            // Line in negative
            else {
                // Compute again mpz_end in floating point
                let temp = window.size().x as f32 * 1.0 / self.pixel_size + start_i64 as f32;
                for i in 1..n {
                    let starty = -1.0 / i as f32 * temp as f32 * self.pixel_size
                        - self.start_p as f32 * self.pixel_size;
                    let endy = -1.0 / i as f32
                        * (temp as f32 * self.pixel_size - window.size().x as f32)
                        - self.start_p as f32 * self.pixel_size;
                    draw_line(0., endy, window.size().x as f32, starty, window);
                }
            }
        }
    }

    pub fn fill_buffer_headless(
        &mut self,
        buffer: &mut Vec<u8>,
        image_width: usize,
        x: usize,
        y: usize,
        size: usize,
        color: f32,
    ) {
        for i in 0..size {
            for j in 0..size {
                buffer[(y * size + j) * image_width + (x * size + i)] =
                    ((255.0 * color).ceil()) as u8;
            }
        }
    }

    #[cfg(feature = "graphic")]
    pub fn fill_buffer(&mut self, buffer: &mut Image, x: usize, y: usize, size: usize, color: f32) {
        for i in 0..size {
            for j in 0..size {
                let _ = buffer.set_pixel(
                    (x * size + i) as u32,
                    (y * size + j) as u32,
                    Color::rgb(
                        (255.0 * color).ceil() as u8,
                        (255.0 * color).ceil() as u8,
                        (255.0 * color).ceil() as u8,
                    ),
                );
            }
        }
    }

    #[cfg(feature = "graphic")]
    fn generate_texture(&mut self, buffer: &Image, image_width: u32, image_height: u32) {
        if self
            .current_texture
            .create(image_width, image_height)
            .is_err()
        {
            panic!("Error creating texture");
        }
        unsafe { self.current_texture.update_from_image(buffer, 0, 0) };
        self.current_sprite = RcSprite::with_texture(&self.current_texture);
    }

    #[cfg(feature = "graphic")]
    pub fn generate_sequences_texture(
        &mut self,
        image_width: u32,
        image_height: u32,
        window: &mut RenderWindow,
    ) -> bool {
        let texture_generation_time = std::time::Instant::now();
        // Round pixel size for easier computation
        let upixel_size = self.pixel_size.ceil() as f32;

        let delta_n = (image_width as f32 / self.pixel_size).floor() as u64;
        let delta_p = (((image_height as f32 / upixel_size).floor() - 1.0)
            * (1.0 / self.pixel_size).ceil()) as u64
            + 1;
        println!(
            "Start generating texture in mode {} with pixel size: {}, n: {}, p: {}, delta_n: {}, delta_p: {}",
            self.mode, self.pixel_size, self.start_index, self.start_p, delta_n, delta_p
        );

        // Initialize the mpz at the right side of the generation
        let right_index = self.start_index.clone()
            + (image_width as u64 * (1.0 / self.pixel_size).ceil() as u64 - 1);

        // Initialize buffer
        let mut buffer = Image::new_solid(image_width, image_height, Color::BLACK).unwrap();

        let mut progressbar = progressbar::Progressbar::new();

        let elt_per_pix = (1.0 / self.pixel_size).ceil() as usize;
        let elt_per_pix_square = (elt_per_pix * elt_per_pix) as f32;
        let pix_per_elt = self.pixel_size.ceil() as usize;

        // Loop over the image size divided by the pixel size
        for y in 0_usize..(image_height as usize / pix_per_elt) {
            // Update progress bar and show the image sometimes
            progressbar.update(y as f32 / (image_height as usize / pix_per_elt) as f32);
            progressbar.show();
            let event = manage_events(window, self);
            if event == 1 {
                progressbar.clear();
                return true;
            } else if event == 2 {
                progressbar.clear();
                return false;
            }
            if (image_height as usize / SHOW_IMAGE_TIMES) != 0
                && y % (image_height as usize / SHOW_IMAGE_TIMES) == 0
            {
                self.generate_texture(&buffer, image_width, image_height);
                window.draw(&self.current_sprite);
                self.draw_position(window);
                window.display();
            }

            let current_p = y * elt_per_pix + self.start_p + 1;
            match self.mode {
                // Average over n*n
                0 => {
                    let sequence = Itertools::chunks(
                        self.fibo.generate(
                            current_p ,
                            image_width as usize * elt_per_pix,
                            right_index.clone(),
                        ),
                        elt_per_pix,
                    )
                    .into_iter()
                    .map(|chunk| chunk.map(|x| x as usize).sum::<usize>())
                    .collect::<Vec<_>>();
                    for x in 0..image_width as usize {
                        let sum = sequence[x];
                        self.fill_buffer(
                            &mut buffer,
                            image_width as usize - x - 1,
                            y,
                            pix_per_elt,
                            sum as f32 / elt_per_pix as f32,
                        );
                    }
                }
                2 => {
                    let sequences = (0..elt_per_pix)
                        .map(|i| {
                            Itertools::chunks(
                                self.fibo.generate(
                                    current_p + i,
                                    image_width as usize * elt_per_pix,
                                    right_index.clone(),
                                ),
                                elt_per_pix,
                            )
                            .into_iter()
                            .map(|chunk| chunk.map(|x| x as usize).sum::<usize>())
                            .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    for x in 0..image_width as usize {
                        let mut sum: usize = 0;
                        for c in sequences.iter() {
                            sum += c[x];
                        }
                        self.fill_buffer(
                            &mut buffer,
                            image_width as usize - x - 1,
                            y,
                            pix_per_elt,
                            sum as f32 / elt_per_pix_square,
                        );
                    }
                }
                // Take only once cell
                1 => {
                    let sequence = self
                        .fibo
                        .generate(
                            current_p,
                            image_width as usize * elt_per_pix,
                            right_index.clone(),
                        )
                        .step_by(elt_per_pix)
                        .collect::<Vec<_>>();
                    for (x, val) in (0..(image_width as usize / pix_per_elt)).zip(sequence) {
                        if val {
                            self.fill_buffer(
                                &mut buffer,
                                image_width as usize - x - 1,
                                y,
                                pix_per_elt,
                                1.0,
                            );
                        }
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

    pub fn generate_sequences_headless(&mut self, image_width: u32, image_height: u32) -> Vec<u8> {
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
        let right_index = self.start_index.clone()
            + (image_width as u64 * (1.0 / self.pixel_size).ceil() as u64 - 1);

        // Initialize buffer
        let mut headless_buffer: Vec<u8> = vec![0; (image_width * image_height) as usize];

        let mut progressbar = progressbar::Progressbar::new();

        let elt_per_pix = (1.0 / self.pixel_size).ceil() as usize;
        let elt_per_pix_square = (elt_per_pix * elt_per_pix) as f32;
        let pix_per_elt = self.pixel_size.ceil() as usize;

        // Loop over the image size divided by the pixel size
        for y in 0_usize..(image_height as usize / pix_per_elt) {
            // Update progress bar and show the image sometimes
            progressbar.update(y as f32 / (image_height as usize / pix_per_elt) as f32);
            progressbar.show();

            let current_p = y * elt_per_pix + self.start_p + 1;
            match self.mode {
                // Average over n
                0 => {
                    let sequence = Itertools::chunks(
                        self.fibo.generate(
                            current_p,
                            image_width as usize * elt_per_pix,
                            right_index.clone(),
                        ),
                        elt_per_pix,
                    )
                    .into_iter()
                    .map(|chunk| chunk.map(|x| x as usize).sum::<usize>())
                    .collect::<Vec<_>>();
                    for x in 0..image_width as usize {
                        let sum: usize = sequence[x];
                        self.fill_buffer_headless(
                            &mut headless_buffer,
                            image_width as usize,
                            image_width as usize - x - 1,
                            y,
                            pix_per_elt,
                            sum as f32 / elt_per_pix as f32,
                        );
                    }
                }
                2 => {
                    let sequences = (0..elt_per_pix)
                        .map(|i| {
                            Itertools::chunks(
                                self.fibo.generate(
                                    current_p + i,
                                    image_width as usize * elt_per_pix,
                                    right_index.clone(),
                                ),
                                elt_per_pix,
                            )
                            .into_iter()
                            .map(|chunk| chunk.map(|x| x as usize).sum::<usize>())
                            .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    for x in 0..image_width as usize {
                        let mut sum: usize = 0;
                        for c in sequences.iter() {
                            sum += c[x];
                        }
                        self.fill_buffer_headless(
                            &mut headless_buffer,
                            image_width as usize,
                            image_width as usize - x - 1,
                            y,
                            pix_per_elt,
                            sum as f32 / elt_per_pix_square,
                        );
                    }
                }
                // Take only once cell
                1 => {
                    let sequence = self
                        .fibo
                        .generate(
                            current_p,
                            image_width as usize / pix_per_elt,
                            right_index.clone(),
                        )
                        .collect::<Vec<_>>();
                    for (x, val) in (0..(image_width as usize / pix_per_elt)).zip(sequence) {
                        if val {
                            self.fill_buffer_headless(
                                &mut headless_buffer,
                                image_width as usize,
                                image_width as usize - x - 1,
                                y,
                                pix_per_elt,
                                1.0,
                            );
                        }
                    }
                }
                _ => {}
            }
            progressbar.clear();
        }

        println!(
            "End generating texture in {:.2} seconds",
            texture_generation_time.elapsed().as_secs_f32()
        );
        return headless_buffer;
    }

    #[cfg(feature = "graphic")]
    pub fn change_mode(&mut self) {
        self.mode = (self.mode + 1) % 3;
    }

    #[cfg(feature = "graphic")]
    // Move the start index to the right, to center the screen on the next power of 2
    pub fn move_right_next_power_of_2(&mut self, window: &mut RenderWindow) {
        use num::FromPrimitive;
        let decalage = (window.size().x as f32 * (1.0 / self.pixel_size)).floor() as u64 / 2;
        self.start_index += decalage;
        // Get the next power of 2
        let mut next_power_of_2 = BigInt::from_u8(2).unwrap();
        while next_power_of_2 <= self.start_index {
            next_power_of_2 *= 2;
        }
        self.start_index = next_power_of_2 - decalage;
    }

    #[cfg(feature = "graphic")]
    // Move the start index to the left, center on the previous power of two
    pub fn move_left_previous_power_of_two(&mut self, window: &mut RenderWindow) {
        use num::FromPrimitive;

        let decalage = (window.size().x as f32 * (1.0 / self.pixel_size)).floor() as u64 / 2;
        // Get the position at the center of the screen
        self.start_index += decalage;
        // Get the next power of 2
        let mut next_power_of_2 = BigInt::from_u8(2).unwrap();
        while next_power_of_2 < self.start_index {
            next_power_of_2 *= 2;
        }

        self.start_index = next_power_of_2 / 2 - decalage;
    }

    #[cfg(feature = "graphic")]
    // Move the start p index to the bottom, center on the next power of two
    pub fn move_bottom_next_power_of_two(&mut self, window: &mut RenderWindow) {
        // Get the position at the center of the screen
        let center = (window.size().y as f32 * (1.0 / self.pixel_size)).floor() as usize / 2;
        // Get the next power of 2
        let mut next_power_of_2 = 2;
        while next_power_of_2 <= center + self.start_p {
            next_power_of_2 *= 2;
        }
        // Compute the new start index (next power of 2 - half of the screen)
        let temp = next_power_of_2 as i64 - center as i64;
        if temp > 0 {
            self.start_p = temp as usize;
        } else {
            self.start_p = 0;
        }
    }

    #[cfg(feature = "graphic")]
    // Move the start p index to the top, center on the previous power of two
    pub fn move_top_previous_power_of_two(&mut self, window: &mut RenderWindow) {
        // Get the position at the center of the screen
        let center = (window.size().y as f32 * (1.0 / self.pixel_size)).floor() as usize / 2;
        // Get the next power of 2
        let mut next_power_of_2 = 2;
        while next_power_of_2 < center + self.start_p {
            next_power_of_2 *= 2;
        }
        // Divide by two to get the previous power of two
        next_power_of_2 /= 2;
        // Compute the new start index (next power of 2 - half of the screen)
        let temp = next_power_of_2 as i64 - center as i64;
        if temp > 0 {
            self.start_p = temp as usize;
        } else {
            self.start_p = 0;
        }
    }

    #[cfg(feature = "graphic")]
    pub fn save_image(&self, filename: &str) {
        // Save current texture
        println!("Start image conversion...");
        match self.current_texture.copy_to_image() {
            Ok(image) => {
                println!("Saving image...");
                if (*image).save_to_file(filename).is_ok() {
                    println!("Image saved successfully as {}", filename);
                } else {
                    println!("Error while saving the image");
                }
            }
            Err(_) => {
                println!("Error while saving the image");
            }
        };
    }

    pub fn save_image_headless(
        &self,
        filename: &str,
        buffer: &mut Vec<u8>,
        image_width: u32,
        image_height: u32,
    ) {
        println!("Start image conversion...");
        let file = match std::fs::File::create(filename) {
            Ok(f) => f,
            Err(e) => {
                println!("Error while saving the image: {}", e);
                return;
            }
        };
        let mut encoder = png::Encoder::new(file, image_width, image_height);
        encoder.set_compression(png::Compression::Best);
        encoder.set_filter(png::FilterType::NoFilter);
        encoder.set_color(png::ColorType::Grayscale);

        let mut writer = encoder.write_header().unwrap();
        println!("Saving image...");
        // match buffer.write_with_encoder(encoder) {
        match writer.write_image_data(buffer) {
            Ok(_) => {
                println!("Image saved successfully as {}", filename);
            }
            Err(e) => {
                println!("Error while saving the image: {}", e);
            }
        }
    }

    #[cfg(feature = "graphic")]
    // Draw text in the right left corner of the window to show the n and p under the mouse
    pub fn draw_position(&mut self, window: &mut RenderWindow) {
        // Draw text
        let font = sfml::graphics::Font::from_file("assets/monospacebold.ttf").unwrap();
        let temp_mpz = self.start_index.clone()
            + ((self.mouse_x as f32 * (1.0 / self.pixel_size)).floor() as u64);
        let mut text = sfml::graphics::Text::new(
            &format!(
                "n: {}, p: {}",
                temp_mpz,
                self.start_p + (self.mouse_y as f32 * (1.0 / self.pixel_size)).floor() as usize
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
