// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub use yansi::Style;

use crate::colors::cli::color_palette;

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
    pub colon: Style,
    pub attr_chrome: Style,
    pub branch: Style,
}

pub(crate) fn styles() -> Styles {
    let color_palette = color_palette();
    Styles {
        kind: Style::default().fg(color_palette.blue),
        name: Style::default(),
        visibility: VisibilityStyles {
            pub_crate: Style::default().fg(color_palette.yellow),
            pub_module: Style::default().fg(color_palette.orange),
            pub_private: Style::default().fg(color_palette.red),
            pub_global: Style::default().fg(color_palette.green),
            pub_super: Style::default().fg(color_palette.orange),
        },
        attr: Style::default().fg(color_palette.cyan),
        colon: Style::default().dim(),
        attr_chrome: Style::default().dim(),
        branch: Style::default().dim(),
    }
}
