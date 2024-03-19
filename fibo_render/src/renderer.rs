use std::borrow::BorrowMut;

use image::{Luma, Pixel};
use sfml::graphics::{
    Color, Image, RcSprite, RcTexture, RenderTarget, RenderWindow, Shape, Transformable,
};

use crate::draw_utils::draw_line;
use crate::fibo_fast::init_serie;
use crate::gmp_utils::{
    utils_mpz_add_mpz, utils_mpz_compare_i64, utils_mpz_compare_mpz, utils_mpz_divexact_u64,
    utils_mpz_sub_u64, utils_mpz_to_i64, utils_mpz_to_string,
};
use crate::window_manager::manage_events;
use crate::{constants::SHOW_IMAGE_TIMES, fibo_fast, gmp_utils::utils_mpz_from_u64, progressbar};
use gmp_mpfr_sys::gmp::{mpz_mul_ui, mpz_t};

pub struct Renderer {
    pub current_sprite: RcSprite,
    pub current_texture: RcTexture,
    pub current_headless_buffer:
        Option<image::ImageBuffer<Luma<u8>, Vec<<Luma<u8> as Pixel>::Subpixel>>>,
    pub pixel_size: f32,
    pub start_index_mpz: mpz_t,
    pub start_p: u64,
    pub fibo: fibo_fast::FiboFastManager,
    pub mode: u8,
    pub show_lines: bool,
    pub line_count: u32,
    pub mouse_x: i32,
    pub mouse_y: i32,
}

impl Renderer {
    pub fn new(pixel_size: f32, start_index_mpz: mpz_t, start_p: u64, mode: u8) -> Renderer {
        Renderer {
            current_sprite: RcSprite::new(),
            current_texture: RcTexture::new().unwrap(),
            current_headless_buffer: None,
            pixel_size,
            start_index_mpz,
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
        // Check if the start index can be converted to u64
        // Initialize the mpz at the right side of the generation
        let mut mpz_end: mpz_t =
            utils_mpz_from_u64((window.size().x as f32 * 1.0 / self.pixel_size).ceil() as u64);
        utils_mpz_add_mpz(mpz_end.borrow_mut(), self.start_index_mpz.borrow_mut());

        // Line in positive
        if utils_mpz_compare_i64(self.start_index_mpz.borrow_mut(), i64::MAX) < 0
            && utils_mpz_compare_i64(mpz_end.borrow_mut(), 0) > 0
        {
            let temp = utils_mpz_to_i64(self.start_index_mpz.borrow_mut());
            for i in 1..n {
                let starty = 1.0 / i as f32 * temp as f32 * self.pixel_size
                    - self.start_p as f32 * self.pixel_size;
                let endy = 1.0 / i as f32
                    * (temp as f32 * self.pixel_size + window.size().x as f32)
                    - self.start_p as f32 * self.pixel_size;
                draw_line(0., starty, window.size().x as f32, endy, window);
            }
        }

        // Line in negative
        if utils_mpz_compare_i64(mpz_end.borrow_mut(), i64::MIN) > 0
            && utils_mpz_compare_i64(self.start_index_mpz.borrow_mut(), 0) < 0
        {
            // Compute again mpz_end in floating point
            let t1 = utils_mpz_to_i64(self.start_index_mpz.borrow_mut());
            let temp = window.size().x as f32 * 1.0 / self.pixel_size + t1 as f32;
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

    pub fn fill_buffer_headless(
        &mut self,
        buffer: &mut image::ImageBuffer<Luma<u8>, Vec<<Luma<u8> as Pixel>::Subpixel>>,
        x: f32,
        y: f32,
        size: f32,
        color: f32,
    ) {
        for i in 0..size as u32 {
            for j in 0..size as u32 {
                buffer.put_pixel(
                    (x * size + i as f32) as u32,
                    (y * size + j as f32) as u32,
                    Luma([((255.0 * color).ceil()) as u8]),
                );
            }
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

    pub fn fill_buffer_wrapper(
        &mut self,
        buffer: &mut Image,
        buffer_headless: &mut image::ImageBuffer<Luma<u8>, Vec<<Luma<u8> as Pixel>::Subpixel>>,
        x: f32,
        y: f32,
        size: f32,
        color: f32,
        headless: bool,
    ) {
        if headless {
            self.fill_buffer_headless(buffer_headless, x, y, size, color);
        } else {
            self.fill_buffer(buffer, x, y, size, color);
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
        headless: bool,
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
            self.pixel_size,
            utils_mpz_to_string(&mut self.start_index_mpz),
            self.start_p,
            delta_n,
            delta_p
        );

        // Initialize the mpz at the right side of the generation
        let mut mpz_start =
            utils_mpz_from_u64(image_width as u64 * (1.0 / self.pixel_size).ceil() as u64 - 1);
        utils_mpz_add_mpz(mpz_start.borrow_mut(), self.start_index_mpz.borrow_mut());

        // Initialize buffer
        let mut buffer = Image::new(image_width, image_height);
        let mut headless_buffer = image::GrayImage::new(image_width, image_height);

        init_serie(
            ((image_height as f32 / upixel_size).floor() * (1.0 / self.pixel_size).ceil()) as u64
                + self.start_p
                + 1
                - 1,
        );

        let mut progressbar = progressbar::Progressbar::new();

        // Loop over the image size divided by the pixel size
        for y in 0_u32..(image_height as f32 / upixel_size).floor() as u32 {
            // Update progress bar and show the image sometimes
            progressbar.update(y.pow(2) as f32 / (image_height / upixel_size as u32).pow(2) as f32);
            progressbar.show();
            if !headless {
                let event = manage_events(window.as_mut().unwrap(), self);
                if event == 1 {
                    progressbar.clear();
                    return true;
                } else if event == 2 {
                    progressbar.clear();
                    return false;
                }
                if (image_height / SHOW_IMAGE_TIMES) != 0
                    && y % (image_height / SHOW_IMAGE_TIMES) == 0
                {
                    self.generate_texture(&buffer, image_width, image_height);
                    window.as_mut().unwrap().draw(&self.current_sprite);
                    self.draw_position(window.as_mut().unwrap());
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
                        self.fill_buffer_wrapper(
                            &mut buffer,
                            &mut headless_buffer,
                            x as f32,
                            y as f32,
                            upixel_size as f32,
                            sum as f32 / (1.0 / self.pixel_size).ceil() as f32,
                            headless,
                        );
                    }
                }
                // Take only once cell
                1 => {
                    for x in 0..(image_width as f32 / upixel_size).floor() as u32 {
                        if sequence[((x as f32) * (1.0 / self.pixel_size).ceil()) as usize] {
                            self.fill_buffer_wrapper(
                                &mut buffer,
                                &mut headless_buffer,
                                x as f32,
                                y as f32,
                                upixel_size as f32,
                                1.0,
                                headless,
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
                        self.fill_buffer_wrapper(
                            &mut buffer,
                            &mut headless_buffer,
                            x as f32,
                            y as f32,
                            upixel_size as f32,
                            sum as f32 / (1.0 / (self.pixel_size * self.pixel_size)).ceil() as f32,
                            headless,
                        );
                    }
                }
                _ => {}
            }
            progressbar.clear();
        }
        if headless {
            self.current_headless_buffer = Some(headless_buffer);
        } else {
            self.generate_texture(&buffer, image_width, image_height);
        }

        println!(
            "End generating texture in {:.2} seconds",
            texture_generation_time.elapsed().as_secs_f32()
        );
        return false;
    }

    pub fn change_mode(&mut self) {
        self.mode = (self.mode + 1) % 3;
    }

    // Move the start index to the right, to center the screen on the next power of 2
    pub fn move_right_next_power_of_2(&mut self, window: &mut RenderWindow) {
        // Get the position at the center of the screen
        let mut center = utils_mpz_from_u64(
            (window.size().x as f32 * (1.0 / self.pixel_size)).floor() as u64 / 2,
        );
        utils_mpz_add_mpz(center.borrow_mut(), self.start_index_mpz.borrow_mut());
        // Get the next power of 2
        let mut next_power_of_2 = utils_mpz_from_u64(1);
        while utils_mpz_compare_mpz(next_power_of_2.borrow_mut(), center.borrow_mut()) <= 0 {
            unsafe {
                mpz_mul_ui(
                    next_power_of_2.borrow_mut(),
                    next_power_of_2.borrow_mut(),
                    2,
                );
            }
        }
        // COmpute the new start index (next power of 2 - half of the screen)
        utils_mpz_sub_u64(
            next_power_of_2.borrow_mut(),
            (window.size().x as f32 * (1.0 / self.pixel_size)).floor() as u64 / 2,
        );
        self.start_index_mpz = next_power_of_2;
    }

    // Move the start index to the left, center on the previous power of two
    pub fn move_left_previous_power_of_two(&mut self, window: &mut RenderWindow) {
        // Get the position at the center of the screen
        let mut center = utils_mpz_from_u64(
            (window.size().x as f32 * (1.0 / self.pixel_size)).floor() as u64 / 2,
        );
        utils_mpz_add_mpz(center.borrow_mut(), self.start_index_mpz.borrow_mut());
        // Get the next power of 2
        let mut next_power_of_2 = utils_mpz_from_u64(1);
        while utils_mpz_compare_mpz(next_power_of_2.borrow_mut(), center.borrow_mut()) < 0 {
            unsafe {
                mpz_mul_ui(
                    next_power_of_2.borrow_mut(),
                    next_power_of_2.borrow_mut(),
                    2,
                );
            }
        }
        // Divide by two to get the previous power of two
        utils_mpz_divexact_u64(next_power_of_2.borrow_mut(), 2);

        // COmpute the new start index (next power of 2 - half of the screen)
        utils_mpz_sub_u64(
            next_power_of_2.borrow_mut(),
            (window.size().x as f32 * (1.0 / self.pixel_size)).floor() as u64 / 2,
        );
        self.start_index_mpz = next_power_of_2;
    }

    // Move the start p index to the bottom, center on the next power of two
    pub fn move_bottom_next_power_of_two(&mut self, window: &mut RenderWindow) {
        // Get the position at the center of the screen
        let center = (window.size().y as f32 * (1.0 / self.pixel_size)).floor() as u64 / 2;
        // Get the next power of 2
        let mut next_power_of_2 = 1;
        while next_power_of_2 <= center + self.start_p{
            next_power_of_2 *= 2;
        }
        // Compute the new start index (next power of 2 - half of the screen)
        let temp = next_power_of_2 as i64 - center as i64;
        if temp > 0 {
            self.start_p = temp as u64;
        } else {
            self.start_p = 0;
        }
    }

    // Move the start p index to the top, center on the previous power of two
    pub fn move_top_previous_power_of_two(&mut self, window: &mut RenderWindow) {
        // Get the position at the center of the screen
        let center = (window.size().y as f32 * (1.0 / self.pixel_size)).floor() as u64 / 2;
        // Get the next power of 2
        let mut next_power_of_2 = 1;
        while next_power_of_2 < center + self.start_p{
            next_power_of_2 *= 2;
        }
        // Divide by two to get the previous power of two
        next_power_of_2 /= 2;
        // Compute the new start index (next power of 2 - half of the screen)
        let temp = next_power_of_2 as i64 - center as i64;
        if temp > 0 {
            self.start_p = temp as u64;
        } else {
            self.start_p = 0;
        }
    }

    pub fn save_image(&self, filename: &str) {
        // Save current texture
        println!("Start image conversion...");
        match self.current_texture.copy_to_image() {
            Some(image) => {
                println!("Saving image...");
                if image.save_to_file(filename) {
                    println!("Image saved successfully as {}", filename);
                } else {
                    println!("Error while saving the image");
                }
            }
            None => {
                println!("Error while saving the image");
            }
        };
    }

    pub fn save_image_headless(&self, filename: &str) {
        // Save current texture
        println!("Start image conversion...");
        match self.current_headless_buffer {
            Some(ref buffer) => {
                println!("Saving image...");
                let file = match std::fs::File::create(filename) {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Error while saving the image: {}", e);
                        return;
                    }
                };
                let encoder = image::codecs::png::PngEncoder::new_with_quality(
                    file,
                    image::codecs::png::CompressionType::Best,
                    image::codecs::png::FilterType::NoFilter,
                );
                match buffer.write_with_encoder(encoder) {
                    Ok(_) => {
                        println!("Image saved successfully as {}", filename);
                    }
                    Err(e) => {
                        println!("Error while saving the image: {}", e);
                    }
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
        let mut temp_mpz =
            utils_mpz_from_u64((self.mouse_x as f32 * (1.0 / self.pixel_size)).floor() as u64);
        utils_mpz_add_mpz(temp_mpz.borrow_mut(), self.start_index_mpz.borrow_mut());
        let mut text = sfml::graphics::Text::new(
            &format!(
                "n: {}, p: {}",
                utils_mpz_to_string(temp_mpz.borrow_mut()),
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
