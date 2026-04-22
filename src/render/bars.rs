use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct BarsWidget<'a> {
    pub data: &'a [f32],
    pub peaks: &'a [f32],
    pub color: Color,
    pub mirror: bool,
}

impl<'a> Widget for BarsWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 || self.data.is_empty() {
            return;
        }

        let num_bars = area.width as usize;
        let bin_size = self.data.len() / num_bars;

        for i in 0..num_bars {
            let start = i * bin_size;
            let end = (i + 1) * bin_size;
            let val = if start < self.data.len() {
                self.data[start..end.min(self.data.len())]
                    .iter()
                    .sum::<f32>()
                    / (end - start) as f32
            } else {
                0.0
            };

            let peak_val = if start < self.peaks.len() {
                self.peaks[start..end.min(self.peaks.len())]
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .cloned()
                    .unwrap_or(0.0)
            } else {
                0.0
            };

            let bar_height = (val * area.height as f32).round() as u16;
            let peak_height = (peak_val * area.height as f32).round() as u16;

            let x = if self.mirror {
                area.right() - 1 - i as u16
            } else {
                area.left() + i as u16
            };

            for y in 0..area.height {
                let current_y = area.bottom() - 1 - y;
                if y < bar_height {
                    let symbol = get_block_char(y, bar_height);
                    buf.get_mut(x, current_y)
                        .set_symbol(symbol)
                        .set_style(Style::default().fg(self.color));
                } else if y + 1 == peak_height && peak_height > 0 {
                    buf.get_mut(x, current_y)
                        .set_symbol("▔")
                        .set_style(Style::default().fg(Color::White));
                }
            }
        }
    }
}

fn get_block_char(_y: u16, _max: u16) -> &'static str {
    "█"
}
