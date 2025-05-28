use sfml::graphics::{RenderTarget, VertexBufferUsage};

pub fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, window: &mut sfml::graphics::RenderWindow) {
    let mut line = sfml::graphics::VertexBuffer::new(
        sfml::graphics::PrimitiveType::LINES,
        2,
        VertexBufferUsage::STATIC,
    )
    .unwrap();
    line.update(
        &[
            sfml::graphics::Vertex::with_pos_color(
                sfml::system::Vector2::new(x1, y1),
                sfml::graphics::Color::RED,
            ),
            sfml::graphics::Vertex::with_pos_color(
                sfml::system::Vector2::new(x2, y2),
                sfml::graphics::Color::RED,
            ),
        ],
        0,
    );

    window.draw(&*line);
}
