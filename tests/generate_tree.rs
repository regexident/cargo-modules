#[macro_use]
mod util;

mod smoke {
    test_plain_cmd!(
        args: "generate tree",
        output: stdout,
        name: plain,
        project: smoke
    );

    test_colored_cmd!(
        args: "generate tree",
        output: stdout,
        name: colored,
        project: smoke
    );
}

mod default {
    mod pass {
        test_plain_cmds!(
            args: "generate tree",
            output: stdout,
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
        test_plain_cmds!(
            args: "generate tree",
            output: stderr,
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
        test_plain_cmds!(
            args: "generate tree \
                    --lib",
            output: stdout,
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
        test_plain_cmds!(
            args: "generate tree \
                    --lib", // does not exist
            output: stderr,
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
        test_plain_cmds!(
            args: "generate tree \
                    --bin package_bin_target",
            output: stdout,
            projects: [
                package_bin_target,
                virtual_workspace_single_package_bin_target,
                workspace_single_package_bin_target,
            ]
        );

        test_plain_cmds!(
            args: "generate tree \
                    --bin package_multi_target",
            output: stdout,
            projects: [
                package_multi_target,
                virtual_workspace_single_package_multi_target,
                workspace_single_package_multi_target,
            ]
        );
    }

    mod fail {
        test_plain_cmds!(
            args: "generate tree \
                    --bin foobar", // does not exist
            output: stderr,
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
        test_plain_cmds!(
            args: "generate tree \
                    --package package_lib_target",
            output: stdout,
            projects: [
                package_lib_target,
                virtual_workspace_single_package_lib_target,
                workspace_single_package_lib_target,
            ]
        );

        test_plain_cmds!(
            args: "generate tree \
                    --package package_bin_target",
            output: stdout,
            projects: [
                package_bin_target,
                virtual_workspace_single_package_bin_target,
                workspace_single_package_bin_target,
            ]
        );
    }

    mod fail {
        test_plain_cmds!(
            args: "generate tree \
                    --package foobar",
            output: stderr,
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
        test_plain_cmds!(
            args: "generate tree \
                    --package package_lib_target \
                    --lib",
            output: stdout,
            projects: [
                package_lib_target,
                workspace_single_package_lib_target,
                virtual_workspace_single_package_lib_target,
            ]
        );

        test_plain_cmds!(
            args: "generate tree \
                    --package package_multi_target \
                    --lib",
            output: stdout,
            projects: [
                package_multi_target,
                workspace_single_package_multi_target,
                virtual_workspace_single_package_multi_target,
            ]
        );
    }

    mod fail {
        test_plain_cmds!(
            args: "generate tree \
                    --package package_bin_target \
                    --lib", // does not exist
            output: stderr,
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
        test_plain_cmds!(
            args: "generate tree \
                    --package package_bin_target \
                    --bin package_bin_target",
            output: stdout,
            projects: [
                package_bin_target,
                workspace_single_package_bin_target,
                virtual_workspace_single_package_bin_target,
            ]
        );

        test_plain_cmds!(
            args: "generate tree \
                    --package package_multi_target \
                    --bin package_multi_target",
            output: stdout,
            projects: [
                package_multi_target,
                workspace_single_package_multi_target,
                virtual_workspace_single_package_multi_target,
            ]
        );
    }

    mod fail {
        test_plain_cmds!(
            args: "generate tree \
                    --package workspace-package \
                    --bin foobar", // does not exist
            output: stderr,
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

mod with_orphans {
    test_plain_cmds!(
        args: "generate tree \
                --with-orphans",
        output: stdout,
        project: smoke
    );
}

mod with_tests {
    test_plain_cmds!(
        args: "generate tree \
                --with-tests",
        output: stdout,
        project: smoke
    );
}

mod with_types {
    test_plain_cmds!(
        args: "generate tree \
                --with-types",
        output: stdout,
        project: smoke
    );
}
