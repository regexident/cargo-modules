#![allow(unused_macros)]

use std::{path::PathBuf, str::from_utf8};

use assert_cmd::Command;
use shellwords::split;

pub fn stdout(mut cmd: Command) -> String {
    let assert = cmd.assert().success();
    let output = assert.get_output();
    from_utf8(&output.stdout).unwrap().to_owned()
}

pub fn stderr(mut cmd: Command) -> String {
    let assert = cmd.assert().failure();
    let output = assert.get_output();
    from_utf8(&output.stderr).unwrap().to_owned()
}

pub fn cmd(dir: &str, args: &str) -> Command {
    let mut dir_path = PathBuf::new();

    dir_path.push(".");
    dir_path.push("tests");
    dir_path.push("projects");
    dir_path.push(dir);

    let args: Vec<String> = match split(args) {
        Ok(args) => args,
        Err(err) => panic!("{}", err),
    };
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

    let mut command = Command::cargo_bin("cargo-modules").unwrap();

    command.current_dir(dir_path);
    command.args(args);

    command
}

macro_rules! test_plain_cmds {
    (args: $args:expr, output: $output:ident, projects: [$(,)?]) => {};
    (args: $args:expr, output: $output:ident, projects: [$head:ident, $($tail:ident),* $(,)?]) => {
        test_plain_cmds!(args: $args, output: $output, project: $head);
        test_plain_cmds!(args: $args, output: $output, projects: [$($tail),* ,]);
    };
    (args: $args:expr, output: $output:ident, project: $project:ident) => {
        mod $project {
            test_plain_cmd!(
                args: $args,
                output: $output,
                name: plain,
                project: $project
            );
        }
    };
}

macro_rules! test_colored_cmds {
    (args: $args:expr, output: $output:ident, projects: [$(,)?]) => {};
    (args: $args:expr, output: $output:ident, projects: [$head:ident, $($tail:ident),* $(,)?]) => {
        test_plain_cmds!(args: $args, output: $output, project: $head);
        test_plain_cmds!(args: $args, output: $output, projects: [$($tail),* ,]);
    };
    (args: $args:expr, output: $output:ident, project: $project:ident) => {
        mod $project {
            test_colored_cmd!(
                args: $args,
                output: $output,
                name: colored,
                project: $project
            );
        }
    };
}

macro_rules! test_plain_cmd {
    (args: $args:expr, output: $output:ident, project: $project:ident) => {
        test_plain_cmd!(
            args: $args,
            output: $output,
            name: $project,
            project: $project
        );
    };
    (args: $args:expr, output: $output:ident, name: $name:ident, project: $project:ident) => {
        test_cmd!(
            args: $args,
            output: $output,
            name: $name,
            colored: false,
            project: $project
        );
    };
}

macro_rules! test_colored_cmd {
    (args: $args:expr, output: $output:ident, project: $project:ident) => {
        test_colored_cmd!(
            args: $args,
            output: $output,
            name: $project,
            project: $project
        );
    };
    (args: $args:expr, output: $output:ident, name: $name:ident, project: $project:ident) => {
        test_cmd!(
            args: $args,
            output: $output,
            name: $name,
            colored: true,
            project: $project
        );
    };
}

macro_rules! test_cmd {
    (args: $args:expr, output: $output:ident, name: $name:ident, colored: $colored:expr, project: $project:ident) => {
        #[test]
        fn $name() {
            let mut cmd = crate::util::cmd(stringify!($project), $args);

            if $colored {
                cmd.env("COLORTERM", "truecolor");
            } else {
                cmd.env_remove("COLORTERM");
            }

            let output = crate::util::$output(cmd);
            insta::assert_snapshot!(output);
        }
    };
}
