#[macro_use]
mod util;

mod help {
    test_cmd!(
        args: "dependencies \
                --help",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod colors {
    mod plain {
        test_cmd!(
            args: "dependencies",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod ansi {
        test_cmd!(
            args: "dependencies",
            success: true,
            color_mode: ColorMode::Ansi,
            project: smoke
        );
    }

    mod truecolor {
        test_cmd!(
            args: "dependencies",
            success: true,
            color_mode: ColorMode::TrueColor,
            project: smoke
        );
    }
}

mod default {
    mod pass {
        test_cmds!(
            args: "dependencies",
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
            args: "dependencies",
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
            args: "dependencies \
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
    mod without_tests {
        test_cmd!(
            args: "dependencies",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod with_tests {
        test_cmd!(
            args: "dependencies \
                    --cfg-test",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod selection {
    mod no_externs {
        test_cmd!(
            args: "dependencies \
                    --no-externs",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod no_fns {
        test_cmd!(
            args: "dependencies \
                    --no-fns",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod no_modules {
        test_cmd!(
            args: "dependencies \
                    --no-modules",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod no_owns {
        test_cmd!(
            args: "dependencies \
                    --no-owns",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod no_traits {
        test_cmd!(
            args: "dependencies \
                    --no-traits",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod no_types {
        test_cmd!(
            args: "dependencies \
                    --no-types",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod no_uses {
        test_cmd!(
            args: "dependencies \
                    --no-uses",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod focus_on {
    mod simple_path {
        test_cmd!(
            args: "dependencies \
                    --focus-on \"crate::visibility::dummy\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod glob_path {
        test_cmd!(
            args: "dependencies \
                    --focus-on \"crate::visibility::*\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod self_path {
        test_cmd!(
            args: "dependencies \
                    --focus-on \"crate::visibility::dummy::{self}\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod use_tree {
        test_cmd!(
            args: "dependencies \
                    --focus-on \"crate::visibility::{dummy, hierarchy}\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod nonexistent_path {
        test_cmd!(
            args: "dependencies \
                    --focus-on \"nonexistent\"",
            success: false,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod max_depth {
    mod depth_0 {
        test_cmd!(
            args: "dependencies \
                    --max-depth 0",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_1 {
        test_cmd!(
            args: "dependencies \
                    --max-depth 1",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_2 {
        test_cmd!(
            args: "dependencies \
                    --max-depth 2",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod fields {
    test_cmds!(
        args: "dependencies",
        success: true,
        color_mode: ColorMode::Plain,
        projects: [
            enum_fields,
            struct_fields,
            tuple_fields,
            union_fields,
        ]
    );
}

mod functions {
    test_cmds!(
        args: "dependencies",
        success: true,
        color_mode: ColorMode::Plain,
        projects: [
            function_inputs,
            function_outputs,
            function_body,
        ]
    );
}

mod github {
    mod issue_79 {
        test_cmd!(
            args: "dependencies \
                    --no-externs \
                    --no-fns \
                    --no-traits \
                    --no-types",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_79
        );
    }

    mod issue_80 {
        mod tests {
            test_cmd!(
                args: "dependencies \
                        --cfg-test \
                        --no-externs \
                        --no-fns \
                        --no-traits",
                success: true,
                color_mode: ColorMode::Plain,
                project: github_issue_80
            );
        }

        mod without_tests {
            test_cmd!(
                args: "dependencies \
                        --no-externs \
                        --no-fns \
                        --no-traits",
                success: true,
                color_mode: ColorMode::Plain,
                project: github_issue_80
            );
        }
    }

    mod issue_102 {
        test_cmd!(
            args: "dependencies \
                    --no-externs \
                    --no-fns \
                    --no-traits \
                    --no-types",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_102
        );
    }

    mod issue_172 {
        test_cmd!(
            args: "dependencies \
                    --no-externs \
                    --no-fns \
                    --layout dot",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_172
        );
    }

    mod issue_362 {
        test_cmd!(
            args: "dependencies \
                    --features 'opt-in'",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_362
        );
    }
}
