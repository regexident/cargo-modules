// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub use yansi::{Color, Style};

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
            purple: Color::RGB(196, 104, 223),
            red: Color::RGB(219, 77, 89),
            orange: Color::RGB(255, 178, 102),
            yellow: Color::RGB(247, 210, 99),
            green: Color::RGB(129, 193, 105),
            cyan: Color::RGB(105, 190, 210),
            blue: Color::RGB(84, 142, 200),
            black: Color::RGB(0, 0, 0),
            gray: Color::RGB(127, 127, 127),
            white: Color::RGB(255, 255, 255),
        },
        None => ColorPalette {
            purple: Color::Unset,
            red: Color::Unset,
            orange: Color::Unset,
            yellow: Color::Unset,
            green: Color::Unset,
            cyan: Color::Unset,
            blue: Color::Unset,
            black: Color::Unset,
            gray: Color::Unset,
            white: Color::Unset,
        },
    }
}
