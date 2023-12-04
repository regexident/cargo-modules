#[macro_use]
mod util;

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

mod negative_args {
    use clap::Parser;

    use cargo_modules::{command::Command, options::App};

    fn args_for(command: &str, arg: &str, default: bool) -> Vec<(Vec<String>, bool)> {
        let args_prefix = vec!["modules".to_owned(), command.to_owned()];

        let pos_arg = format!("--{arg}");
        let neg_arg = format!("--no-{arg}");

        let arg_suffixes = vec![
            (vec![], default),
            (vec![pos_arg.clone()], true),
            (vec![neg_arg.clone()], false),
            (vec![pos_arg.clone(), neg_arg.clone()], false),
            (vec![neg_arg.clone(), pos_arg.clone()], true),
        ];

        arg_suffixes
            .into_iter()
            .map(|(args_suffix, expected)| {
                let mut args = args_prefix.clone();
                args.extend(args_suffix);
                (args, expected)
            })
            .collect()
    }

    mod project {
        use super::*;

        #[test]
        fn cfg_test() {
            for command in ["structure", "dependencies"] {
                for (args, expected) in args_for(command, "cfg-test", false) {
                    let app = App::parse_from(&args);

                    let Command::Dependencies(cmd) = app.command else {
                        continue;
                    };

                    assert_eq!(cmd.options.project.cfg_test, expected, "{:?}", args);
                }
            }
        }

        #[test]
        fn sysroot() {
            for command in ["structure", "dependencies"] {
                for (args, expected) in args_for(command, "sysroot", false) {
                    let app = App::parse_from(&args);

                    let Command::Dependencies(cmd) = app.command else {
                        continue;
                    };

                    assert_eq!(cmd.options.project.sysroot, expected, "{:?}", args);
                }
            }
        }
    }

    mod dependencies {
        use super::*;

        #[test]
        fn fns() {
            for command in ["structure", "dependencies"] {
                for (args, expected) in args_for(command, "fns", false) {
                    let app = App::parse_from(&args);

                    let Command::Dependencies(cmd) = app.command else {
                        continue;
                    };

                    assert_eq!(cmd.options.selection.fns, expected, "{:?}", args);
                }
            }
        }

        #[test]
        fn tests() {
            for command in ["structure", "dependencies"] {
                for (args, expected) in args_for(command, "tests", false) {
                    let app = App::parse_from(&args);

                    let Command::Dependencies(cmd) = app.command else {
                        continue;
                    };

                    assert_eq!(cmd.options.selection.tests, expected, "{:?}", args);
                }
            }
        }

        #[test]
        fn types() {
            for command in ["structure", "dependencies"] {
                for (args, expected) in args_for(command, "types", false) {
                    let app = App::parse_from(&args);

                    let Command::Dependencies(cmd) = app.command else {
                        continue;
                    };

                    assert_eq!(cmd.options.selection.types, expected, "{:?}", args);
                }
            }
        }
    }

    mod dependencies_only {
        use super::*;

        #[test]
        fn modules() {
            for (args, expected) in args_for("dependencies", "modules", true) {
                let app = App::parse_from(&args);

                let Command::Dependencies(cmd) = app.command else {
                    continue;
                };

                assert_eq!(cmd.options.selection.modules, expected, "{:?}", args);
            }
        }

        #[test]
        fn uses() {
            for (args, expected) in args_for("dependencies", "uses", false) {
                let app = App::parse_from(&args);

                let Command::Dependencies(cmd) = app.command else {
                    continue;
                };

                assert_eq!(cmd.options.selection.uses, expected, "{:?}", args);
            }
        }

        #[test]
        fn externs() {
            for (args, expected) in args_for("dependencies", "externs", false) {
                let app = App::parse_from(&args);

                let Command::Dependencies(cmd) = app.command else {
                    continue;
                };

                assert_eq!(cmd.options.selection.externs, expected, "{:?}", args);
            }
        }
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

mod tests {
    test_cmd!(
        args: "dependencies \
                --tests",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod types {
    test_cmd!(
        args: "dependencies \
                --types",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod types_with_no_modules {
    test_cmd!(
        args: "dependencies \
                --types \
                --no-modules",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod traits {
    test_cmd!(
        args: "dependencies \
                --traits",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod fns {
    test_cmd!(
        args: "dependencies \
                --fns",
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

mod uses {
    test_cmd!(
        args: "dependencies \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod externs {
    test_cmd!(
        args: "dependencies \
                --externs",
        success: false,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod uses_with_externs {
    test_cmd!(
        args: "dependencies \
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
        args: "dependencies \
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
            args: "dependencies \
                    --uses \
                    --focus-on \"smoke::visibility::dummy\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod glob_path {
        test_cmd!(
            args: "dependencies \
                    --uses \
                    --focus-on \"smoke::visibility::*\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod self_path {
        test_cmd!(
            args: "dependencies \
                    --uses \
                    --focus-on \"smoke::visibility::dummy::{self}\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod use_tree {
        test_cmd!(
            args: "dependencies \
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
            args: "dependencies \
                    --uses \
                    --max-depth 0",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_1 {
        test_cmd!(
            args: "dependencies \
                    --uses \
                    --max-depth 1",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_2 {
        test_cmd!(
            args: "dependencies \
                    --uses \
                    --max-depth 2",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
}

mod fields {
    test_cmd!(
        args: "dependencies \
                --externs \
                --fns \
                --modules \
                --sysroot \
                --traits \
                --types \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: tuple_fields
    );

    test_cmd!(
        args: "dependencies \
                --externs \
                --fns \
                --modules \
                --sysroot \
                --traits \
                --types \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: struct_fields
    );

    test_cmd!(
        args: "dependencies \
                --externs \
                --fns \
                --modules \
                --sysroot \
                --traits \
                --types \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: enum_fields
    );

    test_cmd!(
        args: "dependencies \
                --externs \
                --fns \
                --modules \
                --sysroot \
                --traits \
                --types \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: union_fields
    );
}

mod functions {
    test_cmd!(
        args: "dependencies \
                --externs \
                --fns \
                --modules \
                --sysroot \
                --traits \
                --types \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );

    test_cmd!(
        args: "dependencies \
                --externs \
                --fns \
                --modules \
                --sysroot \
                --traits \
                --types \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: function_inputs
    );

    test_cmd!(
        args: "dependencies \
                --externs \
                --fns \
                --modules \
                --sysroot \
                --traits \
                --types \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: function_outputs
    );

    test_cmd!(
        args: "dependencies \
                --externs \
                --fns \
                --modules \
                --sysroot \
                --traits \
                --types \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: function_body
    );
}

mod github_issue_79 {
    test_cmd!(
        args: "dependencies \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: github_issue_79
    );
}

mod github_issue_80 {
    mod tests {
        test_cmd!(
            args: "dependencies \
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
            args: "dependencies \
                    --uses \
                    --types",
            success: true,
            color_mode: ColorMode::Plain,
            project: github_issue_80
        );
    }
}

mod github_issue_102 {
    test_cmd!(
        args: "dependencies \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: github_issue_102
    );
}

mod github_issue_172 {
    test_cmd!(
        args: "dependencies \
                --types \
                --uses \
                --traits \
                --layout dot",
        success: true,
        color_mode: ColorMode::Plain,
        project: github_issue_172
    );
}
