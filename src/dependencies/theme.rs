// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Color(pub u8, pub u8, pub u8);

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Color(r, g, b) = self;
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

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
    pub black: Color,
    #[allow(dead_code)]
    pub gray: Color,
    #[allow(dead_code)]
    pub white: Color,
}

pub(crate) fn color_palette() -> ColorPalette {
    ColorPalette {
        purple: Color(186, 111, 167),
        red: Color(219, 83, 103),
        orange: Color(254, 148, 84),
        yellow: Color(248, 192, 76),
        green: Color(129, 193, 105),
        cyan: Color(105, 190, 210),
        blue: Color(83, 151, 200),
        black: Color(0, 0, 0),
        gray: Color(127, 127, 127),
        white: Color(255, 255, 255),
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ItemVisibilityStyles {
    pub pub_crate: NodeStyle,
    pub pub_module: NodeStyle,
    pub pub_private: NodeStyle,
    pub pub_global: NodeStyle,
    pub pub_super: NodeStyle,
}

#[derive(Clone, Debug)]
pub(crate) struct NodeStyle {
    pub fill_color: Color,
}

impl NodeStyle {
    fn new(fill_color: Color) -> Self {
        Self { fill_color }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeStyles {
    #[allow(dead_code)]
    pub krate: NodeStyle,
    pub visibility: ItemVisibilityStyles,
    #[allow(dead_code)]
    pub test: NodeStyle,
}

pub(crate) fn node_styles() -> NodeStyles {
    let color_palette = color_palette();
    NodeStyles {
        krate: NodeStyle::new(color_palette.blue),
        visibility: ItemVisibilityStyles {
            pub_crate: NodeStyle::new(color_palette.yellow),
            pub_module: NodeStyle::new(color_palette.orange),
            pub_private: NodeStyle::new(color_palette.red),
            pub_global: NodeStyle::new(color_palette.green),
            pub_super: NodeStyle::new(color_palette.orange),
        },
        test: NodeStyle::new(color_palette.cyan),
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Stroke {
    Solid,
    Dashed,
}

impl fmt::Display for Stroke {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Solid => "solid",
            Self::Dashed => "dashed",
        };
        write!(f, "{name}")
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EdgeStyle {
    pub color: Color,
    pub stroke: Stroke,
}

impl EdgeStyle {
    fn new(color: Color, stroke: Stroke) -> Self {
        Self { color, stroke }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EdgeStyles {
    pub owns: EdgeStyle,
    pub uses: EdgeStyle,
}

pub(crate) fn edge_styles() -> EdgeStyles {
    let color_palette = color_palette();
    EdgeStyles {
        owns: EdgeStyle::new(color_palette.black, Stroke::Solid),
        uses: EdgeStyle::new(color_palette.gray, Stroke::Dashed),
    }
}
