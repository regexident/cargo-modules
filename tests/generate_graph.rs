#[macro_use]
mod util;

mod colors {
    mod plain {
        test_cmd!(
            args: "generate graph",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod ansi {
        test_cmd!(
            args: "generate graph",
            success: true,
            color_mode: ColorMode::Ansi,
            project: smoke
        );
    }

    mod truecolor {
        test_cmd!(
            args: "generate graph",
            success: true,
            color_mode: ColorMode::TrueColor,
            project: smoke
        );
    }
}

mod default {
    mod pass {
        test_cmds!(
            args: "generate graph",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_bin_target,
                package_lib_target,
                virtual_workspace_single_package_lib_target,
                virtual_workspace_single_package_bin_target,
                workspace_single_package_lib_target,
                workspace_single_package_bin_target,
            ]
        );
    }

    mod fail {
        test_cmds!(
            args: "generate graph",
            success: false,
            color_mode: ColorMode::Plain,
            projects: [
                package_multi_target,
                virtual_workspace_single_package_multi_target,
                workspace_single_package_multi_target,
            ]
        );
    }
}

mod lib {
    mod pass {
        test_cmds!(
            args: "generate graph \
                    --lib",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_lib_target,
                package_multi_target,
                virtual_workspace_single_package_lib_target,
                virtual_workspace_single_package_multi_target,
                workspace_single_package_lib_target,
                workspace_single_package_multi_target,
            ]
        );
    }

    mod fail {
        test_cmds!(
            args: "generate graph \
                    --lib", // does not exist
            success: false,
            color_mode: ColorMode::Plain,
            projects: [
                package_bin_target,
                virtual_workspace_single_package_bin_target,
                workspace_single_package_bin_target,
            ]
        );
    }
}

mod bin {
    mod pass {
        test_cmds!(
            args: "generate graph \
                    --bin package_bin_target",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_bin_target,
                virtual_workspace_single_package_bin_target,
                workspace_single_package_bin_target,
            ]
        );

        test_cmds!(
            args: "generate graph \
                    --bin package_multi_target",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_multi_target,
                virtual_workspace_single_package_multi_target,
                workspace_single_package_multi_target,
            ]
        );
    }

    mod fail {
        test_cmds!(
            args: "generate graph \
                    --bin foobar", // does not exist
            success: false,
            color_mode: ColorMode::Plain,
            projects: [
                package_lib_target,
                package_multi_target,
                virtual_workspace_multi_package,
                virtual_workspace_single_package_lib_target,
                virtual_workspace_single_package_multi_target,
                workspace_multi_package,
                workspace_single_package_lib_target,
                workspace_single_package_multi_target,
            ]
        );
    }
}

mod package {
    mod pass {
        test_cmds!(
            args: "generate graph \
                    --package package_lib_target",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_lib_target,
                virtual_workspace_single_package_lib_target,
                workspace_single_package_lib_target,
            ]
        );

        test_cmds!(
            args: "generate graph \
                    --package package_bin_target",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_bin_target,
                virtual_workspace_single_package_bin_target,
                workspace_single_package_bin_target,
            ]
        );
    }

    mod fail {
        test_cmds!(
            args: "generate graph \
                    --package foobar",
            success: false,
            color_mode: ColorMode::Plain,
            projects: [
                package_bin_target,
                package_lib_target,
                package_multi_target,
                virtual_workspace_multi_package,
                virtual_workspace_single_package_bin_target,
                virtual_workspace_single_package_lib_target,
                virtual_workspace_single_package_multi_target,
                workspace_multi_package,
                workspace_single_package_bin_target,
                workspace_single_package_lib_target,
                workspace_single_package_multi_target,
            ]
        );
    }
}

mod package_lib {
    mod pass {
        test_cmds!(
            args: "generate graph \
                    --package package_lib_target \
                    --lib",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_lib_target,
                workspace_single_package_lib_target,
                virtual_workspace_single_package_lib_target,
            ]
        );

        test_cmds!(
            args: "generate graph \
                    --package package_multi_target \
                    --lib",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_multi_target,
                workspace_single_package_multi_target,
                virtual_workspace_single_package_multi_target,
            ]
        );
    }

    mod fail {
        test_cmds!(
            args: "generate graph \
                    --package package_bin_target \
                    --lib", // does not exist
            success: false,
            color_mode: ColorMode::Plain,
            projects: [
                package_bin_target,
                workspace_single_package_bin_target,
                virtual_workspace_single_package_bin_target,
            ]
        );
    }
}

mod package_bin {
    mod pass {
        test_cmds!(
            args: "generate graph \
                    --package package_bin_target \
                    --bin package_bin_target",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_bin_target,
                workspace_single_package_bin_target,
                virtual_workspace_single_package_bin_target,
            ]
        );

        test_cmds!(
            args: "generate graph \
                    --package package_multi_target \
                    --bin package_multi_target",
            success: true,
            color_mode: ColorMode::Plain,
            projects: [
                package_multi_target,
                workspace_single_package_multi_target,
                virtual_workspace_single_package_multi_target,
            ]
        );
    }

    mod fail {
        test_cmds!(
            args: "generate graph \
                    --package workspace-package \
                    --bin foobar", // does not exist
            success: false,
            color_mode: ColorMode::Plain,
            projects: [
                package_bin_target,
                package_lib_target,
                package_multi_target,
                virtual_workspace_multi_package,
                virtual_workspace_single_package_bin_target,
                virtual_workspace_single_package_lib_target,
                virtual_workspace_single_package_multi_target,
                workspace_multi_package,
                workspace_single_package_bin_target,
                workspace_single_package_lib_target,
                workspace_single_package_multi_target,
            ]
        );
    }
}

mod orphans {
    test_cmd!(
        args: "generate graph \
                --orphans",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod tests {
    test_cmd!(
        args: "generate graph \
                --tests",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod types {
    test_cmd!(
        args: "generate graph \
                --types",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod traits {
    test_cmd!(
        args: "generate graph \
                --traits",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod fns {
    test_cmd!(
        args: "generate graph \
                --fns",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod uses {
    test_cmd!(
        args: "generate graph \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod externs {
    test_cmd!(
        args: "generate graph \
                --externs",
        success: false,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod uses_with_externs {
    test_cmd!(
        args: "generate graph \
                --uses \
                --externs",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod uses_with_externs_with_sysroot {
    test_cmd!(
        attrs: [
            // `sysroot` is expensive, so only run on release builds:
            #[ignore]
        ],
        args: "generate graph \
                --uses \
                --externs \
                --sysroot",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod focus_on {
    mod simple_path {
        test_cmd!(
            args: "generate graph \
                    --uses \
                    --focus-on \"smoke::visibility::dummy\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
    mod glob_path {
        test_cmd!(
            args: "generate graph \
                    --uses \
                    --focus-on \"smoke::visibility::*\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
    mod self_path {
        test_cmd!(
            args: "generate graph \
                    --uses \
                    --focus-on \"smoke::visibility::dummy::{self}\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
    mod tree {
        test_cmd!(
            args: "generate graph \
                    --uses \
                    --focus-on \"smoke::visibility::{dummy, hierarchy}\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod max_depth {
    mod depth_0 {
        test_cmd!(
            args: "generate graph \
                    --uses \
                    --max-depth 0",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_1 {
        test_cmd!(
            args: "generate graph \
                    --uses \
                    --max-depth 1",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_2 {
        test_cmd!(
            args: "generate graph \
                    --uses \
                    --max-depth 2",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod github_issue_79 {
    test_cmd!(
        args: "generate graph \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: github_issue_79
    );
}

mod github_issue_80 {
    mod tests {
        test_cmd!(
            args: "generate graph \
                    --uses \
                    --types \
                    --tests",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_80
        );
    }

    mod without_tests {
        test_cmd!(
            args: "generate graph \
                    --uses \
                    --types",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_80
        );
    }
}

mod github_issue_172 {
    test_cmd!(
        args: "generate graph \
                --types \
                --uses \
                --traits \
                --layout dot",
        success: true,
        color_mode: ColorMode::Plain,
        project: github_issue_172
    );
}
