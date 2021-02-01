#[macro_use]
mod util;

mod smoke {
    test_plain_cmd!(
        args: "generate graph",
        output: stdout,
        name: plain,
        project: smoke
    );
}

mod default {
    mod pass {
        test_plain_cmds!(
            args: "generate graph",
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
            args: "generate graph",
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
            args: "generate graph \
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
            args: "generate graph \
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
            args: "generate graph \
                    --bin package_bin_target",
            output: stdout,
            projects: [
                package_bin_target,
                virtual_workspace_single_package_bin_target,
                workspace_single_package_bin_target,
            ]
        );

        test_plain_cmds!(
            args: "generate graph \
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
            args: "generate graph \
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
            args: "generate graph \
                    --package package_lib_target",
            output: stdout,
            projects: [
                package_lib_target,
                virtual_workspace_single_package_lib_target,
                workspace_single_package_lib_target,
            ]
        );

        test_plain_cmds!(
            args: "generate graph \
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
            args: "generate graph \
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
            args: "generate graph \
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
            args: "generate graph \
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
            args: "generate graph \
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
            args: "generate graph \
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
            args: "generate graph \
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
            args: "generate graph \
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
        args: "generate graph \
                --with-orphans",
        output: stdout,
        project: smoke
    );
}

mod with_tests {
    test_plain_cmds!(
        args: "generate graph \
                --with-tests",
        output: stdout,
        project: smoke
    );
}

mod with_types {
    test_plain_cmds!(
        args: "generate graph \
                --with-types",
        output: stdout,
        project: smoke
    );
}

mod with_uses {
    test_plain_cmds!(
        args: "generate graph \
                --with-uses",
        output: stdout,
        project: smoke
    );
}

mod with_externs {
    test_plain_cmds!(
        args: "generate graph \
                --with-externs",
        output: stderr,
        project: smoke
    );
}

mod with_uses_with_externs {
    test_plain_cmds!(
        args: "generate graph \
                --with-uses \
                --with-externs",
        output: stdout,
        project: smoke
    );
}
