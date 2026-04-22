use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct SpectrogramWidget<'a> {
    pub history: &'a Vec<Vec<f32>>,
    pub color: Color,
}

impl<'a> Widget for SpectrogramWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 || self.history.is_empty() {
            return;
        }

        for (y, row) in self.history.iter().enumerate() {
            if y >= area.height as usize { break; }
            
            let current_y = area.top() + y as u16;
            let num_bins = area.width as usize;
            let bin_size = row.len() / num_bins;

            for x in 0..area.width {
                let start = x as usize * bin_size;
                let end = (x as usize + 1) * bin_size;
                let val = if start < row.len() {
                    row[start..end.min(row.len())].iter().sum::<f32>() / (end - start) as f32
                } else {
                    0.0
                };

                let symbol = get_intensity_char(val);
                buf.get_mut(area.left() + x, current_y)
                    .set_symbol(symbol)
                    .set_style(Style::default().fg(self.color));
            }
        }
    }
}

fn get_intensity_char(val: f32) -> &'static str {
    if val > 0.8 { "█" }
    else if val > 0.6 { "▓" }
    else if val > 0.4 { "▒" }
    else if val > 0.2 { "░" }
    else if val > 0.05 { "·" }
    else { " " }
}
