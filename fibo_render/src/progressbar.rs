use std::io::{self, Write};

use crate::constants::PROGRESS_BAR_SIZE;


pub struct Progressbar {
    progress: f32, // Value between 0 and 1
}

impl Progressbar {
    pub fn new() -> Progressbar {
        print!("Progress: [{}] 0%", " ".repeat(PROGRESS_BAR_SIZE as usize));
        io::stdout().flush().unwrap();
        Progressbar { progress: 0.0 }
    }

    pub fn update(&mut self, progress: f32) {
        self.progress = progress;
    }
    pub fn show(&mut self) {
        print!(
            "\rProgress: [{}{}] {:.2}%",
            "#".repeat((self.progress * PROGRESS_BAR_SIZE as f32) as usize),
            " ".repeat(
                (PROGRESS_BAR_SIZE as f32 - self.progress * PROGRESS_BAR_SIZE as f32) as usize
            ),
            self.progress * 100.
        );
        io::stdout().flush().unwrap();
    }

    pub fn clear(&self) {
        print!("\r\x1b[K");
        io::stdout().flush().unwrap();
    }
}