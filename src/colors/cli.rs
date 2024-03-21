// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub use yansi::Color;

#[derive(Clone, Debug)]
pub(crate) struct ColorPalette {
    #[allow(dead_code)]
    pub purple: Color,
    pub red: Color,
    pub orange: Color,
    pub yellow: Color,
    pub green: Color,
    pub cyan: Color,
    pub blue: Color,
    #[allow(dead_code)]
    pub black: Color,
    #[allow(dead_code)]
    pub gray: Color,
    #[allow(dead_code)]
    pub white: Color,
}

enum ColorDepth {
    Fixed,
    Rgb,
}

fn color_depth() -> Option<ColorDepth> {
    if std::env::var("NO_COLOR").is_ok() {
        return None;
    }

    match std::env::var("COLORTERM").as_deref() {
        Ok("truecolor") | Ok("24bit") => return Some(ColorDepth::Rgb),
        _ => {}
    };

    Some(ColorDepth::Fixed)
}

pub(crate) fn color_palette() -> ColorPalette {
    match color_depth() {
        Some(ColorDepth::Fixed) => ColorPalette {
            purple: Color::Fixed(133),
            red: Color::Fixed(167),
            orange: Color::Fixed(209),
            yellow: Color::Fixed(215),
            green: Color::Fixed(107),
            cyan: Color::Fixed(74),
            blue: Color::Fixed(68),
            black: Color::Fixed(0),
            gray: Color::Fixed(8),
            white: Color::Fixed(15),
        },
        Some(ColorDepth::Rgb) => ColorPalette {
            purple: Color::Rgb(196, 104, 223),
            red: Color::Rgb(219, 77, 89),
            orange: Color::Rgb(255, 178, 102),
            yellow: Color::Rgb(247, 210, 99),
            green: Color::Rgb(129, 193, 105),
            cyan: Color::Rgb(105, 190, 210),
            blue: Color::Rgb(84, 142, 200),
            black: Color::Rgb(0, 0, 0),
            gray: Color::Rgb(127, 127, 127),
            white: Color::Rgb(255, 255, 255),
        },
        None => ColorPalette {
            purple: Color::Primary,
            red: Color::Primary,
            orange: Color::Primary,
            yellow: Color::Primary,
            green: Color::Primary,
            cyan: Color::Primary,
            blue: Color::Primary,
            black: Color::Primary,
            gray: Color::Primary,
            white: Color::Primary,
        },
    }
}
