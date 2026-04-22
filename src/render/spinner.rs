use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct SpinnerWidget {
    pub angle: f32,
    pub rms: f32,
    pub color: Color,
}

impl Widget for SpinnerWidget {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        let center_x = area.left() + area.width / 2;
        let center_y = area.top() + area.height / 2;
        let base_radius = (area.height as f32 / 3.0).min(area.width as f32 / 4.0);
        let radius = base_radius + self.rms * base_radius;

        // Draw multiple arms
        let num_arms = 4;
        for i in 0..num_arms {
            let arm_angle = self.angle + (i as f32 * std::f32::consts::TAU / num_arms as f32);
            
            // Draw a line from center to radius
            for r_step in 0..(radius as i16) {
                let dx = (arm_angle.cos() * r_step as f32 * 2.0) as i16; // 2x for aspect ratio
                let dy = (arm_angle.sin() * r_step as f32) as i16;

                let x = center_x as i16 + dx;
                let y = center_y as i16 + dy;

                if x >= area.left() as i16 && x < area.right() as i16 
                   && y >= area.top() as i16 && y < area.bottom() as i16 {
                    buf.get_mut(x as u16, y as u16)
                        .set_symbol("✸")
                        .set_style(Style::default().fg(self.color));
                }
            }
        }
    }
}
