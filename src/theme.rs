use yansi::{Color, Style};

pub(crate) struct VisibilityTheme {
    pub pub_crate: Style,
    pub pub_module: Style,
    pub pub_private: Style,
    pub pub_public: Style,
    pub pub_super: Style,
}

pub(crate) struct Theme {
    pub name: Style,
    pub visibility: VisibilityTheme,
    pub cfg: Style,
    pub orphan: Style,
}

// GREEN:  Color::RGB(129, 193, 105)
// YELLOW: Color::RGB(248, 192,  76)
// ORANGE: Color::RGB(254, 148,  84)
// RED:    Color::RGB(219,  83, 103)
// PURPLE: Color::RGB(186, 111, 167)
// BLUE:   Color::RGB( 83, 151, 200)
pub(crate) fn theme() -> Theme {
    Theme {
        name: Style::default(),
        visibility: VisibilityTheme {
            pub_crate: Style::new(Color::RGB(248, 192, 76)), // YELLOW TEXT
            pub_module: Style::new(Color::RGB(254, 148, 84)), // ORANGE TEXT
            pub_private: Style::new(Color::RGB(219, 83, 103)), // RED TEXT
            pub_public: Style::new(Color::RGB(129, 193, 105)), // GREEN TEXT
            pub_super: Style::new(Color::RGB(254, 148, 84)), // ORANGE TEXT
        },
        cfg: Style::new(Color::RGB(83, 151, 200)), // BLUE TEXT
        orphan: Style::new(Color::RGB(186, 111, 167)), // orphan: Style::new(Color::RGB(0, 0, 0)).bg(Color::RGB(219, 83, 103)), // RED BACKGROUND

                                                       // Style::default().dimmed().bg(Color::RGB(219, 83, 103)), // RED BACKGROUND
    }
}
