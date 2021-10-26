// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::env;

pub use yansi::{Color, Style};

#[derive(Clone, Debug)]
pub(crate) struct ColorPalette {
    pub purple: Color,
    pub red: Color,
    pub orange: Color,
    pub yellow: Color,
    pub green: Color,
    pub cyan: Color,
    pub blue: Color,
    pub black: Color,
    pub gray: Color,
    pub white: Color,
}

enum ColorDepth {
    Fixed,
    Rgb,
}

fn color_depth() -> Option<ColorDepth> {
    if env::var("NO_COLOR").is_ok() {
        return None;
    }

    match env::var("COLORTERM").as_deref() {
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
            purple: Color::RGB(186, 111, 167),
            red: Color::RGB(219, 83, 103),
            orange: Color::RGB(254, 148, 84),
            yellow: Color::RGB(248, 192, 76),
            green: Color::RGB(129, 193, 105),
            cyan: Color::RGB(105, 190, 210),
            blue: Color::RGB(83, 151, 200),
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

#[derive(Clone, Debug)]
pub(crate) struct VisibilityStyles {
    pub pub_crate: Style,
    pub pub_module: Style,
    pub pub_private: Style,
    pub pub_global: Style,
    pub pub_super: Style,
}

#[derive(Clone, Debug)]
pub(crate) struct Styles {
    pub kind: Style,
    pub name: Style,
    pub visibility: VisibilityStyles,
    pub attr: Style,
    pub orphan: Style,
}

pub(crate) fn styles() -> Styles {
    let color_palette = color_palette();
    Styles {
        kind: Style::new(color_palette.blue),
        name: Style::default(),
        visibility: VisibilityStyles {
            pub_crate: Style::new(color_palette.yellow),
            pub_module: Style::new(color_palette.orange),
            pub_private: Style::new(color_palette.red),
            pub_global: Style::new(color_palette.green),
            pub_super: Style::new(color_palette.orange),
        },
        attr: Style::new(color_palette.cyan),
        orphan: Style::new(color_palette.purple),
    }
}
