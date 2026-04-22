use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};
use crate::app::PulseRing;

pub struct PulseWidget<'a> {
    pub rings: &'a [PulseRing],
    pub color: Color,
}

impl<'a> Widget for PulseWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let center_x = area.left() + area.width / 2;
        let center_y = area.top() + area.height / 2;

        for ring in self.rings {
            // Draw a crude circle/diamond using radius
            let r = ring.radius as i16;
            if r <= 0 { continue; }

            for dy in -r..=r {
                for dx in -r..=r {
                    // Using Manhattan distance or Euclidean distance
                    let dist = ((dx * dx) as f32 + (dy * dy * 4) as f32).sqrt(); // 4x multiplier for aspect ratio
                    if (dist - r as f32).abs() < 1.0 {
                        let x = center_x as i16 + dx;
                        let y = center_y as i16 + dy;

                        if x >= area.left() as i16 && x < area.right() as i16 
                           && y >= area.top() as i16 && y < area.bottom() as i16 {
                            buf.get_mut(x as u16, y as u16)
                                .set_symbol("∘")
                                .set_style(Style::default().fg(self.color));
                        }
                    }
                }
            }
        }
    }
}
