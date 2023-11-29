#[macro_use]
mod util;

mod colors {
    mod plain {
        test_cmd!(
            args: "generate tree",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod default {
    mod pass {
        test_cmds!(
            args: "generate tree",
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
            args: "generate tree",
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
            args: "generate tree \
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
        args: "generate tree \
                --orphans",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod tests {
    test_cmd!(
        args: "generate tree \
                --tests",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod types {
    test_cmd!(
        args: "generate tree \
                --types",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod traits {
    test_cmd!(
        args: "generate tree \
                --traits",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod fns {
    test_cmd!(
        args: "generate tree \
                --fns",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod functions {
    test_cmd!(
        args: "generate tree \
                --fns \
                --traits \
                --types",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod github_issue_80 {
    mod tests {
        test_cmd!(
            args: "generate tree \
                    --types \
                    --tests",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_80
        );
    }

    mod without_tests {
        test_cmd!(
            args: "generate tree \
                    --types",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_80
        );
    }
}

mod github_issue_222 {
    test_cmd!(
        args: "generate tree",
        success: true,
        color_mode: ColorMode::Plain,
        project: github_issue_222
    );
}

mod focus_on {
    mod simple_path {
        test_cmd!(
            args: "generate tree \
                    --focus-on \"smoke::visibility::dummy\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
    mod glob_path {
        test_cmd!(
            args: "generate tree \
                    --focus-on \"smoke::visibility::*\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
    mod self_path {
        test_cmd!(
            args: "generate tree \
                    --focus-on \"smoke::visibility::dummy::{self}\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
    mod tree {
        test_cmd!(
            args: "generate tree \
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
            args: "generate tree \
                    --max-depth 0",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_1 {
        test_cmd!(
            args: "generate tree \
                    --max-depth 1",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_2 {
        test_cmd!(
            args: "generate tree \
                    --max-depth 2",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod sort_by {
    mod name {
        test_cmd!(
            args: "generate tree \
            --types \
            --traits \
            --fns \
            --sort-by \"name\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod visibility {
        test_cmd!(
            args: "generate tree \
            --types \
            --traits \
            --fns \
            --sort-by \"visibility\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod kind {
        test_cmd!(
            args: "generate tree \
            --types \
            --traits \
            --fns \
            --sort-by \"kind\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod sort_reversed {
    test_cmd!(
        args: "generate tree \
        --types \
        --traits \
        --fns \
        --sort-reversed",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}
