use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct RainDrop {
    pub x: u16,
    pub y: f32,
    pub speed: f32,
    pub length: usize,
    pub chars: Vec<char>,
}

pub struct RainWidget<'a> {
    pub drops: &'a [RainDrop],
    pub peak_color: Color,
    pub levels: [Color; 5],
}

impl<'a> Widget for RainWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        for drop in self.drops {
            if drop.x >= area.width {
                continue;
            }

            for i in 0..drop.length {
                let y = drop.y as i32 - i as i32;
                if y >= 0 && y < area.height as i32 {
                    let char_idx = (i + drop.y as usize) % drop.chars.len();
                    let c = drop.chars[char_idx];

                    let alpha = 1.0 - (i as f32 / drop.length as f32);
                    let final_color = if i == 0 {
                        self.peak_color
                    } else {
                        level_color(&self.levels, alpha)
                    };

                    buf.get_mut(area.left() + drop.x, area.top() + y as u16)
                        .set_symbol(&c.to_string())
                        .set_style(Style::default().fg(final_color));
                }
            }
        }
    }
}

fn level_color(levels: &[Color; 5], value: f32) -> Color {
    let index = (value.clamp(0.0, 1.0) * (levels.len() - 1) as f32).round() as usize;
    levels[index]
}
