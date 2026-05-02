use clap::ValueEnum;
use ratatui::style::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Theme {
    Neon,
    Fire,
    Ice,
    Rainbow,
}

pub struct Palette {
    pub primary: Color,
    pub peak: Color,
    pub accent: Color,
    pub help_bg: Color,
    pub help_fg: Color,
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
                help_bg: Color::DarkGray,
                help_fg: Color::White,
            };
        }

        match self {
            Theme::Neon => Palette {
                primary: Color::Green,
                peak: Color::White,
                accent: Color::Cyan,
                help_bg: Color::DarkGray,
                help_fg: Color::White,
            },
            Theme::Fire => Palette {
                primary: Color::Red,
                peak: Color::Yellow,
                accent: Color::LightRed,
                help_bg: Color::Black,
                help_fg: Color::White,
            },
            Theme::Ice => Palette {
                primary: Color::LightCyan,
                peak: Color::White,
                accent: Color::Blue,
                help_bg: Color::Black,
                help_fg: Color::White,
            },
            Theme::Rainbow => Palette {
                primary: Color::Magenta,
                peak: Color::Yellow,
                accent: Color::LightGreen,
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

    #[test]
    fn next_cycles_through_themes() {
        assert_eq!(Theme::Neon.next(), Theme::Fire);
        assert_eq!(Theme::Fire.next(), Theme::Ice);
        assert_eq!(Theme::Ice.next(), Theme::Rainbow);
        assert_eq!(Theme::Rainbow.next(), Theme::Neon);
    }
}
