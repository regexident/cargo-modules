#[macro_use]
mod util;

mod colors {
    mod plain {
        test_cmd!(
            args: "structure",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod default {
    mod pass {
        test_cmds!(
            args: "structure",
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
            args: "structure",
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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
            args: "structure \
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

mod cfg_test {
    test_cmd!(
        args: "dependencies \
                --cfg-test",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod selection {
    mod no_fns {
        test_cmd!(
            args: "structure \
                    --no-fns",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod no_traits {
        test_cmd!(
            args: "structure \
                    --no-traits",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod no_types {
        test_cmd!(
            args: "structure \
                    --no-types",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod focus_on {
    mod simple_path {
        test_cmd!(
            args: "structure \
                    --focus-on \"smoke::visibility::dummy\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod glob_path {
        test_cmd!(
            args: "structure \
                    --focus-on \"smoke::visibility::*\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod self_path {
        test_cmd!(
            args: "structure \
                    --focus-on \"smoke::visibility::dummy::{self}\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod use_tree {
        test_cmd!(
            args: "structure \
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
            args: "structure \
                    --no-types \
                    --no-traits \
                    --no-fns \
                    --max-depth 0",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_1 {
        test_cmd!(
            args: "structure \
                    --no-types \
                    --no-traits \
                    --no-fns \
                    --max-depth 1",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_2 {
        test_cmd!(
            args: "structure \
                    --no-types \
                    --no-traits \
                    --no-fns \
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
            args: "structure \
                    --sort-by \"name\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod visibility {
        test_cmd!(
            args: "structure \
                    --sort-by \"visibility\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod kind {
        test_cmd!(
            args: "structure \
                    --sort-by \"kind\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod sort_reversed {
    test_cmd!(
        args: "structure \
                --sort-reversed",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod github {
    mod issue_80 {
        mod tests {
            test_cmd!(
                args: "structure \
                        --cfg-test \
                        --no-traits \
                        --no-fns",
                success: true,
                color_mode: ColorMode::Plain,
                project: github_issue_80
            );
        }

        mod without_tests {
            test_cmd!(
                args: "structure \
                        --no-traits \
                        --no-fns",
                success: true,
                color_mode: ColorMode::Plain,
                project: github_issue_80
            );
        }
    }

    mod issue_222 {
        test_cmd!(
            args: "structure \
                    --no-types \
                    --no-traits \
                    --no-fns",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_222
        );
    }
}
