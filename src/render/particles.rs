use crate::app::Particle;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct ParticlesWidget<'a> {
    pub particles: &'a [Particle],
    pub color: Color,
}

impl<'a> Widget for ParticlesWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        for p in self.particles {
            let x = p.x as u16;
            let y = p.y as u16;

            if x >= area.left() && x < area.right() && y >= area.top() && y < area.bottom() {
                let symbol = if p.life > 0.8 {
                    "*"
                } else if p.life > 0.5 {
                    "+"
                } else if p.life > 0.2 {
                    "."
                } else {
                    " "
                };

                buf.get_mut(x, y)
                    .set_symbol(symbol)
                    .set_style(Style::default().fg(self.color));
            }
        }
    }
}
