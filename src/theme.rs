use clap::ValueEnum;
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Theme {
    Neon,
    Fire,
    Ice,
    Rainbow,
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub primary: Color,
    pub peak: Color,
    pub accent: Color,
    pub levels: [Color; 5],
    pub help_bg: Color,
    pub help_fg: Color,
}

impl Palette {
    pub fn level(&self, value: f32) -> Color {
        let value = value.clamp(0.0, 1.0);
        let index = (value * (self.levels.len() - 1) as f32).round() as usize;
        self.levels[index]
    }
}

impl Theme {
    pub fn next(self) -> Self {
        match self {
            Theme::Neon => Theme::Fire,
            Theme::Fire => Theme::Ice,
            Theme::Ice => Theme::Rainbow,
            Theme::Rainbow => Theme::Neon,
        }
    }

    pub fn palette(self, no_color: bool) -> Palette {
        if no_color {
            return Palette {
                primary: Color::White,
                peak: Color::White,
                accent: Color::White,
                levels: [
                    Color::DarkGray,
                    Color::Gray,
                    Color::White,
                    Color::White,
                    Color::White,
                ],
                help_bg: Color::DarkGray,
                help_fg: Color::White,
            };
        }

        match self {
            Theme::Neon => Palette {
                primary: Color::Green,
                peak: Color::White,
                accent: Color::Cyan,
                levels: [
                    Color::DarkGray,
                    Color::Green,
                    Color::LightGreen,
                    Color::Cyan,
                    Color::White,
                ],
                help_bg: Color::DarkGray,
                help_fg: Color::White,
            },
            Theme::Fire => Palette {
                primary: Color::Red,
                peak: Color::Yellow,
                accent: Color::LightRed,
                levels: [
                    Color::DarkGray,
                    Color::Red,
                    Color::LightRed,
                    Color::Yellow,
                    Color::White,
                ],
                help_bg: Color::Black,
                help_fg: Color::White,
            },
            Theme::Ice => Palette {
                primary: Color::LightCyan,
                peak: Color::White,
                accent: Color::Blue,
                levels: [
                    Color::DarkGray,
                    Color::Blue,
                    Color::Cyan,
                    Color::LightCyan,
                    Color::White,
                ],
                help_bg: Color::Black,
                help_fg: Color::White,
            },
            Theme::Rainbow => Palette {
                primary: Color::Magenta,
                peak: Color::Yellow,
                accent: Color::LightGreen,
                levels: [
                    Color::Blue,
                    Color::Magenta,
                    Color::LightRed,
                    Color::Yellow,
                    Color::LightGreen,
                ],
                help_bg: Color::DarkGray,
                help_fg: Color::White,
            },
        }
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let theme = match self {
            Theme::Neon => "neon",
            Theme::Fire => "fire",
            Theme::Ice => "ice",
            Theme::Rainbow => "rainbow",
        };
        f.write_str(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::Theme;
    use ratatui::style::Color;

    #[test]
    fn next_cycles_through_themes() {
        assert_eq!(Theme::Neon.next(), Theme::Fire);
        assert_eq!(Theme::Fire.next(), Theme::Ice);
        assert_eq!(Theme::Ice.next(), Theme::Rainbow);
        assert_eq!(Theme::Rainbow.next(), Theme::Neon);
    }

    #[test]
    fn palette_level_clamps_to_available_colors() {
        let palette = Theme::Fire.palette(false);

        assert_eq!(palette.level(-1.0), Color::DarkGray);
        assert_eq!(palette.level(2.0), Color::White);
    }
}
