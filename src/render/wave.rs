use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct WaveWidget<'a> {
    pub samples: &'a [f32],
    pub color: Color,
}

impl<'a> Widget for WaveWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 || self.samples.is_empty() {
            return;
        }

        let mid_y = area.top() + area.height / 2;
        let amplitude = area.height as f32 / 2.0;

        for x in 0..area.width {
            let idx = (x as f32 / area.width as f32 * self.samples.len() as f32) as usize;
            let sample = self.samples.get(idx).cloned().unwrap_or(0.0);

            let offset = (sample * amplitude).round() as i16;
            let y = (mid_y as i16 + offset) as u16;
            let y = y.clamp(area.top(), area.bottom() - 1);

            buf.get_mut(area.left() + x, y)
                .set_symbol("·")
                .set_style(Style::default().fg(self.color));

            // Draw a vertical line connecting to center to make it look like a solid wave
            let start = y.min(mid_y);
            let end = y.max(mid_y);
            for fill_y in start..=end {
                buf.get_mut(area.left() + x, fill_y)
                    .set_symbol("┃")
                    .set_style(Style::default().fg(self.color));
            }
        }
    }
}
