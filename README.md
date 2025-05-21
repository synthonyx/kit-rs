# Synthonyx Kit for Rust.

This kit provides all the tools for implementation of the Synthonyx Rust idiomatics.

**This project is under heavy development and not yet production ready. Use with caution.**

## Licensing

Copyright (c) 2024-2025, [Synthonyx Technologies Ltd](https://synthonyx.com).

This SDK is dual-licensed under the terms of the Apache License, Version 2.0 and the MIT license. Choose the one that best fits your needs.

## Using this crate

For now this crate isn't published at crates.io and it has to be used by adding a git dependency in Cargo.toml:
```toml
[dependencies]
synthonyx-kit = { git = "https://github.com/synthonyx/kit-rs", tag = "v0.1.2", features = ["serde"] }
```

### Publishing updates

Git tags are used for versioning, so you can publish a new release by running the following commands:
```bash
git tag -a v0.1.x -m "Release version 0.1.x"
git push origin v0.1.x
```

Also update this README, so that the dependency import example contains the most current version.