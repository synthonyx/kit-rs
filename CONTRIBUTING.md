# Contributing

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

## Repository hygiene

Editor, IDE, and external tooling configuration files — including hidden
directories, dotfiles, and per-tool session, cache, or context files — must
not be committed. The canonical ignore list lives in
[`.gitignore`](.gitignore). CI rejects any tracked file matching the
ignored patterns.

Contributors are responsible for ensuring their local tooling does not write
to tracked paths. Configure any tool that maintains state in the working
directory to write to a path that is already ignored, or add a global git
ignore entry on your machine.

## Development

```sh
cargo xtest    # cargo test --workspace --all-features
cargo xclippy  # cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo xdeny    # cargo deny check
cargo fmt --all
```

The MSRV is documented in `Cargo.toml` (`rust-version`).
