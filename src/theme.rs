use yansi::{Color, Style};

pub(crate) struct VisibilityTheme {
    pub krate: Style,
    pub module: Style,
    pub private: Style,
    pub public: Style,
    pub zuper: Style,
    pub orphan: Style,
}

pub(crate) struct Theme {
    pub name: Style,
    pub visibility: VisibilityTheme,
    pub cfg: Style,
}

// GREEN:  Color::RGB(129, 193, 105)
// YELLOW: Color::RGB(248, 192,  76)
// ORANGE: Color::RGB(254, 148,  84)
// RED:    Color::RGB(219,  83, 103)
// PURPLE: Color::RGB(198, 105, 170)
// BLUE:   Color::RGB( 83, 151, 200)
pub(crate) fn theme() -> Theme {
    Theme {
        name: Style::default(),
        visibility: VisibilityTheme {
            krate: Style::new(Color::RGB(248, 192, 76)), // YELLOW TEXT
            module: Style::new(Color::RGB(254, 148, 84)), // ORANGE TEXT
            private: Style::new(Color::RGB(219, 83, 103)), // RED TEXT
            public: Style::new(Color::RGB(129, 193, 105)), // GREEN TEXT
            zuper: Style::new(Color::RGB(254, 148, 84)), // ORANGE TEXT
            orphan: Style::default().bg(Color::RGB(219, 83, 103)), // RED BACKGROUND
        },
        cfg: Style::new(Color::RGB(83, 151, 200)), // BLUE TEXT
    }
}
