#![allow(unused_macros)]

use std::{path::PathBuf, str::from_utf8};

use assert_cmd::Command;

// If we clicking the `Run test(s)` btn which is(are) provided by `vscode-rust-analyzer` plugin.
// That plugin will automatically set the `RUST_BACKTRACE` env as `short` and then trigger `cargo test`.
// See: https://github.com/rust-lang/rust-analyzer/blob/21ec8f523812b88418b2bfc64240c62b3dd967bd/crates/rust-analyzer/src/bin/main.rs#L105
// See: https://github.com/rust-lang/rust-analyzer/blob/21ec8f523812b88418b2bfc64240c62b3dd967bd/editors/code/src/run.ts#L73
fn correct_cmd_env(command: &mut Command) {
    // The baseline snapshots (fixtures) make core assumption that tests are running without `backtrace`.
    // So we need to make sure that currently the command is without `backtrace` as well.
    if std::env::var("RUST_BACKTRACE").is_ok_and(|v| !v.is_empty()) {
        command.env("RUST_BACKTRACE", "0");
    }
}

#[allow(dead_code)]
pub enum ColorMode {
    Plain,
    Ansi,
    TrueColor,
}

#[allow(dead_code)]
pub fn output(mut cmd: Command, expect_success: bool) -> (String, String) {
    let assert = cmd.assert();
    let assert = if expect_success {
        assert.success()
    } else {
        assert.failure()
    };
    let output = assert.get_output();
    let stdout_str = from_utf8(&output.stdout).unwrap();
    let stderr_str = from_utf8(&output.stderr).unwrap();
    let stdout = stdout_str.to_owned();
    let stderr = stderr_str.to_owned();
    (stdout, stderr)
}

#[allow(dead_code)]
pub fn cmd<'a>(dir: &str, args: impl Iterator<Item = &'a String>) -> Command {
    let mut dir_path = PathBuf::new();

    dir_path.push(".");
    dir_path.push("tests");
    dir_path.push("projects");
    dir_path.push(dir);

    let mut command = Command::cargo_bin("cargo-modules").unwrap();

    command.current_dir(dir_path);
    command.args(args);

    // Correct the env of the command.
    correct_cmd_env(&mut command);

    command
}

macro_rules! test_cmds {
    (
        args: $args:expr,
        success: $success:expr,
        color_mode: $color_mode:expr,
        projects: [$($projects:ident),+ $(,)?]
    ) => {
        test_cmds!(
            attrs: [],
            args: $args,
            success: $success,
            color_mode: $color_mode,
            projects: [ $($projects),* ]
        );
    };
    (
        attrs: [ $(#[$attrs:meta]),* $(,)? ],
        args: $args:expr,
        success: $success:expr,
        color_mode: $color_mode:expr,
        projects: [ $(,)? ]
    ) => {};
    (
        attrs: [ $(#[$attrs:meta]),* $(,)? ],
        args: $args:expr,
        success: $success:expr,
        color_mode: $color_mode:expr,
        projects: [ $head:ident $(, $tail:ident)* $(,)? ]
    ) => {
        test_cmd!(
            attrs: [ $(#[$attrs]),* ],
            args: $args,
            success: $success,
            color_mode: $color_mode,
            project: $head
        );
        test_cmds!(
            attrs: [ $(#[$attrs]),* ],
            args: $args,
            success: $success,
            color_mode: $color_mode,
            projects: [ $($tail),* ]
        );
    };
}

macro_rules! test_cmd {
    (
        args: $args:expr,
        success: $success:expr,
        color_mode: $color_mode:expr,
        project: $project:ident
    ) => {
        test_cmd!(
            attrs: [],
            args: $args,
            success: $success,
            color_mode: $color_mode,
            project: $project
        );
    };
    (
        attrs: [ $(#[$attrs:meta]),* $(,)? ],
        args: $args:expr,
        success: $success:expr,
        color_mode: $color_mode:expr,
        project: $project:ident
    ) => {
        $(#[$attrs])*
        #[test]
        fn $project() {
            use std::fmt::Write;

            use crate::util::ColorMode;

            let args: Vec<String> = ::shellwords::split($args).unwrap();
            let mut cmd = crate::util::cmd(stringify!($project), args.iter());

            #[allow(unreachable_patterns)]
            match $color_mode {
                ColorMode::Plain => {
                    cmd.env("NO_COLOR", "1");
                },
                ColorMode::Ansi => {
                    cmd.env_remove("COLORTERM");
                },
                ColorMode::TrueColor => {
                    cmd.env("COLORTERM", "truecolor");
                },
            }

            let command = format!("{}\n", args.join("\n"));

            let (stdout, stderr) = crate::util::output(cmd, $success);

            let mut output = String::new();

            writeln!(&mut output, "COMMAND:\n{command}").unwrap();
            writeln!(&mut output, "STDERR:\n{stderr}").unwrap();
            writeln!(&mut output, "STDOUT:\n{stdout}").unwrap();

            insta::assert_snapshot!(output);
        }
    };
}
