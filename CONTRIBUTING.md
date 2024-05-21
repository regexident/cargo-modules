# Contributing

We love contributions from everyone.

By participating in this project, you agree to abide by the Rust [code of conduct](https://www.rust-lang.org/conduct.html).

We expect everyone to follow the code of conduct
anywhere in our project codebases,
issue trackers, chatrooms, and mailing lists.

## Contributing Code

1. Fork the repo (preferably on a feature-branch).

    If you are using **Windows system**, please clone the repo with `git clone -c core.symlinks=true <repo url>`. 'Cause here are some symlink-like files under the `tests/projects` directory, these work well on Linux system, but not on Windows system. So you need to transform them as real symlinks through setting `core.symlinks=true` when cloning.

2. Make your changes.

3. Make sure your changes make the tests pass:

    `$ cargo test`

4. Make sure your changes make the lints pass:

    `$ cargo clippy` (to install clippy; `$ rustup component add clippy-preview`)

5. Make sure your changes follow the project's code style. (hint: `$ cargo fmt`)

6. Push to your fork.

7. Submit a pull request.

Others will give constructive feedback.
This is a time for discussion and improvements,
and making the necessary changes will be required before we can
merge the contribution.
