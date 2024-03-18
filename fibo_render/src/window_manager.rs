use std::{borrow::BorrowMut, io::Write};

use sfml::{
    graphics::{RenderTarget, RenderWindow},
    window::{Event, Key},
};

use crate::{command_line::HELP_MESSAGE, constants::MOVE_STEP, gmp_utils::{utils_mpz_add_u64, utils_mpz_set_string, utils_mpz_sub_u64}, renderer::Renderer};

pub struct WindowManager {
    window: RenderWindow,
    renderer: Renderer,
}

pub fn generate_sequences(window: &mut RenderWindow, renderer: &mut Renderer) {
    let mut flag = true;
    while flag {
        flag = renderer.generate_sequences_texture(window.size().x, window.size().y, false, Some(window));
    }
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
                    renderer.save_image("fibo_sequence.png");
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
                    if Key::LShift.is_pressed() {
                        utils_mpz_add_u64(renderer.start_index_mpz.borrow_mut(), 1);
                    } else {
                        utils_mpz_add_u64(renderer.start_index_mpz.borrow_mut(), MOVE_STEP);
                    }
                    result = 1;
                }
                sfml::window::Key::Left => {
                    if Key::LShift.is_pressed() {
                        utils_mpz_sub_u64(renderer.start_index_mpz.borrow_mut(), 1);
                    } else {
                        utils_mpz_sub_u64(renderer.start_index_mpz.borrow_mut(), MOVE_STEP);
                    }
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
                        return 2;
                    }
                }
                sfml::window::Key::N => {
                    // Input start index
                    print!("Enter n start index: ");
                    std::io::stdout().flush().unwrap();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    utils_mpz_set_string(input.trim().to_string(), renderer.start_index_mpz.borrow_mut());
                    result = 1;
                }
                sfml::window::Key::B => {
                    // Input p start index
                    print!("Enter p start index: ");
                    std::io::stdout().flush().unwrap();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    renderer.start_p = input.trim().parse().unwrap();
                    result = 1;
                }
                sfml::window::Key::H => {
                    println!("{}", HELP_MESSAGE);
                }
                _ => {}
            },
            Event::MouseMoved { x, y } => {
                renderer.mouse_x = x;
                renderer.mouse_y = y;
            }
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

        WindowManager { window, renderer }
    }

    pub fn run(&mut self) {
        generate_sequences(&mut self.window, &mut self.renderer);
        while self.window.is_open() {
            if manage_events(&mut self.window, &mut self.renderer) == 1 {
                generate_sequences(&mut self.window, &mut self.renderer);
            }
            self.window.draw(&self.renderer.current_sprite);
            if self.renderer.show_lines {
                self.renderer
                    .generate_line(&mut self.window, self.renderer.line_count);
            }
            self.renderer.draw_position(&mut self.window);
            self.window.display();
        }
    }
}
