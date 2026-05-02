use crate::app::PulseRing;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct PulseWidget<'a> {
    pub rings: &'a [PulseRing],
    pub levels: [Color; 5],
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
            if r <= 0 {
                continue;
            }

            for dy in -r..=r {
                for dx in -r..=r {
                    // Using Manhattan distance or Euclidean distance
                    let dist = ((dx * dx) as f32 + (dy * dy * 4) as f32).sqrt(); // 4x multiplier for aspect ratio
                    if (dist - r as f32).abs() < 1.5 {
                        // Increased thickness from 1.0 to 1.5
                        let x = center_x as i16 + dx;
                        let y = center_y as i16 + dy;

                        if x >= area.left() as i16
                            && x < area.right() as i16
                            && y >= area.top() as i16
                            && y < area.bottom() as i16
                        {
                            buf.get_mut(x as u16, y as u16)
                                .set_symbol(ring_symbol(ring.intensity))
                                .set_style(
                                    Style::default().fg(level_color(&self.levels, ring.intensity)),
                                );
                        }
                    }
                }
            }
        }
    }
}

fn ring_symbol(intensity: f32) -> &'static str {
    if intensity > 0.7 {
        "●"
    } else if intensity > 0.35 {
        "○"
    } else {
        "∘"
    }
}

fn level_color(levels: &[Color; 5], value: f32) -> Color {
    let index = (value.clamp(0.0, 1.0) * (levels.len() - 1) as f32).round() as usize;
    levels[index]
}
