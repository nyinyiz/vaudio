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
    pub color: Color,
    pub intensity: f32,
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
                    
                    let _alpha = 1.0 - (i as f32 / drop.length as f32);
                    let final_color = if i == 0 {
                        Color::White // Lead character
                    } else {
                        self.color // Tail
                    };

                    buf.get_mut(area.left() + drop.x, area.top() + y as u16)
                        .set_symbol(&c.to_string())
                        .set_style(Style::default().fg(final_color));
                }
            }
        }
    }
}
