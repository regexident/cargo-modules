pub use yansi::Style;

#[derive(Copy, Clone, Debug)]
pub(crate) struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<Rgb> for yansi::Color {
    fn from(rgba: Rgb) -> Self {
        yansi::Color::RGB(rgba.r, rgba.g, rgba.b)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ColorPalette {
    pub purple: Rgb,
    pub red: Rgb,
    pub orange: Rgb,
    pub yellow: Rgb,
    pub green: Rgb,
    pub cyan: Rgb,
    pub blue: Rgb,
}

pub(crate) fn color_palette() -> ColorPalette {
    ColorPalette {
        purple: Rgb {
            r: 186,
            g: 111,
            b: 167,
        },
        red: Rgb {
            r: 219,
            g: 83,
            b: 103,
        },
        orange: Rgb {
            r: 254,
            g: 148,
            b: 84,
        },
        yellow: Rgb {
            r: 248,
            g: 192,
            b: 76,
        },
        green: Rgb {
            r: 129,
            g: 193,
            b: 105,
        },
        cyan: Rgb {
            r: 105,
            g: 190,
            b: 210,
        },
        blue: Rgb {
            r: 83,
            g: 151,
            b: 200,
        },
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VisibilityColors {
    pub pub_crate: Rgb,
    pub pub_module: Rgb,
    pub pub_private: Rgb,
    pub pub_global: Rgb,
    pub pub_super: Rgb,
}

#[derive(Clone, Debug)]
pub(crate) struct Colors {
    pub kind: Rgb,

    pub visibility: VisibilityColors,
    pub attr: Rgb,
    pub orphan: Rgb,
}

pub(crate) fn colors() -> Colors {
    let color_palette = color_palette();

    Colors {
        kind: color_palette.blue,
        visibility: VisibilityColors {
            pub_crate: color_palette.yellow,
            pub_module: color_palette.orange,
            pub_private: color_palette.red,
            pub_global: color_palette.green,
            pub_super: color_palette.orange,
        },
        attr: color_palette.cyan,
        orphan: color_palette.purple,
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
    let colors = colors();
    Styles {
        kind: Style::new(colors.kind.into()),
        name: Style::default(),
        visibility: VisibilityStyles {
            pub_crate: Style::new(colors.visibility.pub_crate.into()),
            pub_module: Style::new(colors.visibility.pub_module.into()),
            pub_private: Style::new(colors.visibility.pub_private.into()),
            pub_global: Style::new(colors.visibility.pub_global.into()),
            pub_super: Style::new(colors.visibility.pub_super.into()),
        },
        attr: Style::new(colors.attr.into()),
        orphan: Style::new(colors.orphan.into()),
    }
}
