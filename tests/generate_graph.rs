#[macro_use]
mod util;

mod colors {
    mod plain {
        test_cmd!(
            args: "graph",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod ansi {
        test_cmd!(
            args: "graph",
            success: true,
            color_mode: ColorMode::Ansi,
            project: smoke
        );
    }

    mod truecolor {
        test_cmd!(
            args: "graph",
            success: true,
            color_mode: ColorMode::TrueColor,
            project: smoke
        );
    }
}

mod default {
    mod pass {
        test_cmds!(
            args: "graph",
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
            args: "graph",
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
            for command in ["structure", "graph"] {
                for (args, expected) in args_for(command, "cfg-test", false) {
                    let app = App::parse_from(&args);

                    let Command::Graph(options) = app.command else {
                        continue;
                    };

                    assert_eq!(options.project.cfg_test, expected, "{:?}", args);
                }
            }
        }

        #[test]
        fn sysroot() {
            for command in ["structure", "graph"] {
                for (args, expected) in args_for(command, "sysroot", false) {
                    let app = App::parse_from(&args);

                    let Command::Graph(options) = app.command else {
                        continue;
                    };

                    assert_eq!(options.project.sysroot, expected, "{:?}", args);
                }
            }
        }
    }

    mod graph {
        use super::*;

        #[test]
        fn fns() {
            for command in ["structure", "graph"] {
                for (args, expected) in args_for(command, "fns", false) {
                    let app = App::parse_from(&args);

                    let Command::Graph(options) = app.command else {
                        continue;
                    };

                    assert_eq!(options.selection.fns, expected, "{:?}", args);
                }
            }
        }

        #[test]
        fn tests() {
            for command in ["structure", "graph"] {
                for (args, expected) in args_for(command, "tests", false) {
                    let app = App::parse_from(&args);

                    let Command::Graph(options) = app.command else {
                        continue;
                    };

                    assert_eq!(options.selection.tests, expected, "{:?}", args);
                }
            }
        }

        #[test]
        fn types() {
            for command in ["structure", "graph"] {
                for (args, expected) in args_for(command, "types", false) {
                    let app = App::parse_from(&args);

                    let Command::Graph(options) = app.command else {
                        continue;
                    };

                    assert_eq!(options.selection.types, expected, "{:?}", args);
                }
            }
        }
    }

    mod graph_only {
        use super::*;

        #[test]
        fn modules() {
            for (args, expected) in args_for("graph", "modules", true) {
                let app = App::parse_from(&args);

                let Command::Graph(options) = app.command else {
                    continue;
                };

                assert_eq!(options.modules, expected, "{:?}", args);
            }
        }

        #[test]
        fn uses() {
            for (args, expected) in args_for("graph", "uses", false) {
                let app = App::parse_from(&args);

                let Command::Graph(options) = app.command else {
                    continue;
                };

                assert_eq!(options.uses, expected, "{:?}", args);
            }
        }

        #[test]
        fn externs() {
            for (args, expected) in args_for("graph", "externs", false) {
                let app = App::parse_from(&args);

                let Command::Graph(options) = app.command else {
                    continue;
                };

                assert_eq!(options.externs, expected, "{:?}", args);
            }
        }
    }
}

mod lib {
    mod pass {
        test_cmds!(
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
            args: "graph \
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
        args: "graph \
                --tests",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod types {
    test_cmd!(
        args: "graph \
                --types",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod types_with_no_modules {
    test_cmd!(
        args: "graph \
                --types \
                --no-modules",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod traits {
    test_cmd!(
        args: "graph \
                --traits",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod fns {
    test_cmd!(
        args: "graph \
                --fns",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod no_modules {
    test_cmd!(
        args: "graph \
                --no-modules",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod uses {
    test_cmd!(
        args: "graph \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod externs {
    test_cmd!(
        args: "graph \
                --externs",
        success: false,
        color_mode: ColorMode::Plain,
        project: smoke
    );
}

mod uses_with_externs {
    test_cmd!(
        args: "graph \
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
        args: "graph \
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
            args: "graph \
                    --uses \
                    --focus-on \"smoke::visibility::dummy\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
    mod glob_path {
        test_cmd!(
            args: "graph \
                    --uses \
                    --focus-on \"smoke::visibility::*\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
    mod self_path {
        test_cmd!(
            args: "graph \
                    --uses \
                    --focus-on \"smoke::visibility::dummy::{self}\"",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }
    mod structure {
        test_cmd!(
            args: "graph \
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
            args: "graph \
                    --uses \
                    --max-depth 0",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_1 {
        test_cmd!(
            args: "graph \
                    --uses \
                    --max-depth 1",
            success: true,
            color_mode: ColorMode::Plain,
            project: smoke
        );
    }

    mod depth_2 {
        test_cmd!(
            args: "graph \
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
        args: "graph \
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
        args: "graph \
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
        args: "graph \
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
        args: "graph \
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
        args: "graph \
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
        args: "graph \
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
        args: "graph \
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
        args: "graph \
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
        args: "graph \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: github_issue_79
    );
}

mod github_issue_80 {
    mod tests {
        test_cmd!(
            args: "graph \
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
            args: "graph \
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
        args: "graph \
                --uses",
        success: true,
        color_mode: ColorMode::Plain,
        project: github_issue_102
    );
}

mod github_issue_172 {
    test_cmd!(
        args: "graph \
                --types \
                --uses \
                --traits \
                --layout dot",
        success: true,
        color_mode: ColorMode::Plain,
        project: github_issue_172
    );
}
