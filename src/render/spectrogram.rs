use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub struct SpectrogramWidget<'a> {
    pub history: &'a Vec<Vec<f32>>,
    pub levels: [Color; 5],
}

impl<'a> Widget for SpectrogramWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if area.height == 0 || area.width == 0 || self.history.is_empty() {
            return;
        }

        for (y, row) in self.history.iter().enumerate() {
            if y >= area.height as usize {
                break;
            }

            let current_y = area.top() + y as u16;
            let num_bins = area.width as usize;

            for x in 0..area.width {
                let val = average_column(row, x as usize, num_bins);

                let symbol = get_intensity_char(val);
                buf.get_mut(area.left() + x, current_y)
                    .set_symbol(symbol)
                    .set_style(Style::default().fg(level_color(&self.levels, val)));
            }
        }
    }
}

fn get_intensity_char(val: f32) -> &'static str {
    if val > 0.8 {
        "█"
    } else if val > 0.6 {
        "▓"
    } else if val > 0.4 {
        "▒"
    } else if val > 0.2 {
        "░"
    } else if val > 0.05 {
        "·"
    } else {
        " "
    }
}

fn level_color(levels: &[Color; 5], value: f32) -> Color {
    let index = (value.clamp(0.0, 1.0) * (levels.len() - 1) as f32).round() as usize;
    levels[index]
}

fn column_range(len: usize, column: usize, columns: usize) -> Option<std::ops::Range<usize>> {
    if len == 0 || columns == 0 || column >= columns {
        return None;
    }

    let start = column * len / columns;
    let mut end = (column + 1) * len / columns;
    if end <= start {
        end = start + 1;
    }

    Some(start.min(len)..end.min(len))
}

fn average_column(data: &[f32], column: usize, columns: usize) -> f32 {
    let Some(range) = column_range(data.len(), column, columns) else {
        return 0.0;
    };

    data[range.clone()].iter().sum::<f32>() / range.len() as f32
}

#[cfg(test)]
mod tests {
    use super::{average_column, column_range, level_color};
    use ratatui::style::Color;

    #[test]
    fn column_range_handles_wide_terminals() {
        for column in 0..100 {
            let range = column_range(4, column, 100).unwrap();
            assert!(!range.is_empty());
            assert!(range.end <= 4);
        }
    }

    #[test]
    fn average_column_repeats_bins_when_terminal_is_wider_than_fft_row() {
        let row = [0.2, 0.8];

        assert_eq!(average_column(&row, 0, 60), 0.2);
        assert_eq!(average_column(&row, 59, 60), 0.8);
    }

    #[test]
    fn level_color_clamps_to_palette() {
        let levels = [
            Color::Black,
            Color::Blue,
            Color::Cyan,
            Color::Yellow,
            Color::White,
        ];

        assert_eq!(level_color(&levels, -1.0), Color::Black);
        assert_eq!(level_color(&levels, 2.0), Color::White);
    }
}
