#![allow(unused_macros)]

use std::{path::PathBuf, str::from_utf8};

use assert_cmd::Command;
use bitflags::bitflags;
use shellwords::split;

bitflags! {
    pub struct ColorModes: u8 {
        const PLAIN = 0b00000001;
        const ANSI = 0b00000010;
        const TRUE_COLOR = 0b00000100;

        const ALL_COLORS = Self::ANSI.bits | Self::TRUE_COLOR.bits;
        const ALL = Self::PLAIN.bits | Self::ANSI.bits | Self::TRUE_COLOR.bits;
    }
}

pub fn stdout(mut cmd: Command) -> String {
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let output_str = from_utf8(&output.stdout).unwrap();
    output_str.trim_start().to_owned()
}

pub fn stderr(mut cmd: Command) -> String {
    let assert = cmd.assert().failure();
    let output = assert.get_output();
    let output_str = from_utf8(&output.stderr).unwrap();
    output_str.trim_start().to_owned()
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

macro_rules! test_cmds {
    (
        args: $args:expr,
        output: $output:ident,
        color_modes: $color_modes:expr,
        projects: [$($projects:ident),+ $(,)?]
    ) => {
        test_cmds!(
            attrs: [],
            args: $args,
            output: $output,
            color_modes: $color_modes,
            projects: [ $($projects),* ]
        );
    };
    (
        attrs: [ $(#[$attrs:meta]),* $(,)? ],
        args: $args:expr,
        output: $output:ident,
        color_modes: $color_modes:expr,
        projects: [ $(,)? ]
    ) => {};
    (
        attrs: [ $(#[$attrs:meta]),* $(,)? ],
        args: $args:expr,
        output: $output:ident,
        color_modes: $color_modes:expr,
        projects: [ $head:ident $(, $tail:ident)* $(,)? ]
    ) => {
        test_cmd!(
            attrs: [ $(#[$attrs]),* ],
            args: $args,
            output: $output,
            color_modes: $color_modes,
            project: $head
        );
        test_cmds!(
            attrs: [ $(#[$attrs]),* ],
            args: $args,
            output: $output,
            color_modes: $color_modes,
            projects: [ $($tail),* ]
        );
    };
}

macro_rules! test_cmd {
    (
        args: $args:expr,
        output: $output:ident,
        color_modes: $color_modes:expr,
        project: $project:ident
    ) => {
        test_cmd!(
            attrs: [],
            args: $args,
            output: $output,
            color_modes: $color_modes,
            project: $project
        );
    };
    (
        attrs: [ $(#[$attrs:meta]),* $(,)? ],
        args: $args:expr,
        output: $output:ident,
        color_modes: $color_modes:expr,
        project: $project:ident
    ) => {
        mod $project {
            #[allow(unused_imports)]
            use super::*;
            use crate::util::ColorModes;

            $(#[$attrs])*
            #[test]
            fn plain() {
                if !$color_modes.contains(ColorModes::PLAIN) {
                    return;
                }

                let mut cmd = crate::util::cmd(stringify!($project), $args);

                cmd.env("NO_COLOR", "1");

                let output = crate::util::$output(cmd);
                insta::assert_snapshot!(output);
            }

            $(#[$attrs])*
            #[test]
            fn ansi() {
                if !$color_modes.contains(ColorModes::ANSI) {
                    return;
                }

                let mut cmd = crate::util::cmd(stringify!($project), $args);

                cmd.env_remove("COLORTERM");

                let output = crate::util::$output(cmd);
                insta::assert_snapshot!(output);
            }

            $(#[$attrs])*
            #[test]
            fn truecolor() {
                if !$color_modes.contains(ColorModes::TRUE_COLOR) {
                    return;
                }

                let mut cmd = crate::util::cmd(stringify!($project), $args);

                cmd.env("COLORTERM", "truecolor");

                let output = crate::util::$output(cmd);
                insta::assert_snapshot!(output);
            }
        }
    };
}
