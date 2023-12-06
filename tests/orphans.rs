#[macro_use]
mod util;

mod help {
    test_cmd!(
        args: "orphans \
                --help",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod colors {
    mod plain {
        test_cmd!(
            args: "orphans",
            success: false,
            color_mode: ColorMode::Plain,
            project: orphans
        );
    }

    mod ansi {
        test_cmd!(
            args: "orphans",
            success: false,
            color_mode: ColorMode::Ansi,
            project: orphans
        );
    }

    mod truecolor {
        test_cmd!(
            args: "orphans",
            success: false,
            color_mode: ColorMode::TrueColor,
            project: orphans
        );
    }
}

mod no_orphans {
    test_cmd!(
        args: "orphans",
        success: true,
        color_mode: ColorMode::Plain,
        project: no_orphans
    );
}

mod orphans {
    test_cmd!(
        args: "orphans",
        success: false,
        color_mode: ColorMode::Plain,
        project: orphans
    );
}
