use std::fmt;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Color(pub u8, pub u8, pub u8);

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Color(r, g, b) = self;
        write!(f, "#{:02x}{:02x}{:02x}", r, g, b)
    }
}

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
pub(crate) struct NodeVisibilityStyles {
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
    pub krate: NodeStyle,
    pub visibility: NodeVisibilityStyles,
    pub orphan: NodeStyle,
    pub test: NodeStyle,
}

pub(crate) fn node_styles() -> NodeStyles {
    let color_palette = color_palette();
    NodeStyles {
        krate: NodeStyle::new(color_palette.blue),
        visibility: NodeVisibilityStyles {
            pub_crate: NodeStyle::new(color_palette.yellow),
            pub_module: NodeStyle::new(color_palette.orange),
            pub_private: NodeStyle::new(color_palette.red),
            pub_global: NodeStyle::new(color_palette.green),
            pub_super: NodeStyle::new(color_palette.orange),
        },
        orphan: NodeStyle::new(color_palette.purple),
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
            Self::Dashed => "Dashed",
        };
        write!(f, "{}", name)
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
