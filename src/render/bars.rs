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

        for i in 0..num_bars {
            let val = average_column(self.data, i, num_bars);
            let peak_val = max_column(self.peaks, i, num_bars);

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

fn max_column(data: &[f32], column: usize, columns: usize) -> f32 {
    let Some(range) = column_range(data.len(), column, columns) else {
        return 0.0;
    };

    data[range].iter().copied().reduce(f32::max).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::{average_column, column_range, max_column};

    #[test]
    fn column_range_never_returns_empty_ranges_for_wide_terminals() {
        for column in 0..120 {
            let range = column_range(8, column, 120).unwrap();
            assert!(!range.is_empty());
            assert!(range.end <= 8);
        }
    }

    #[test]
    fn average_column_handles_more_columns_than_data_points() {
        let data = [0.25, 0.5];

        assert_eq!(average_column(&data, 0, 80), 0.25);
        assert_eq!(average_column(&data, 79, 80), 0.5);
    }

    #[test]
    fn max_column_uses_the_proportional_bucket() {
        let data = [0.1, 0.7, 0.2, 0.4];

        assert_eq!(max_column(&data, 0, 2), 0.7);
        assert_eq!(max_column(&data, 1, 2), 0.4);
    }
}
