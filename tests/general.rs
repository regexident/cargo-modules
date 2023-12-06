#[macro_use]
mod util;

mod help {
    test_cmd!(
        args: "--help",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}
