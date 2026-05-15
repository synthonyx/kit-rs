# Release process

The kit is published as a Cargo workspace. Each member crate is uploaded to
crates.io in **dependency order**, lowest first:

1. `synthonyx-kit-core`
2. `synthonyx-kit-primitives`
3. `synthonyx-kit-tracing`
4. `synthonyx-kit-storage`
5. `synthonyx-kit-audit`
6. `synthonyx-kit-compliance`
7. `synthonyx-kit-password`
8. `synthonyx-kit-macros`
9. `synthonyx-kit`

## Cutting a release

1. Bump `version` in the workspace root `Cargo.toml` under
   `[workspace.package]`. Member crates inherit it.
2. Update `CHANGELOG.md` (when present).
3. Open a release PR; CI must be green.
4. After merge to `main`:

   ```sh
   git tag v0.X.Y
   git push origin v0.X.Y
   for crate in synthonyx-kit-core \
                synthonyx-kit-primitives \
                synthonyx-kit-tracing \
                synthonyx-kit-storage \
                synthonyx-kit-audit \
                synthonyx-kit-compliance \
                synthonyx-kit-password \
                synthonyx-kit-macros \
                synthonyx-kit; do
     cargo publish -p "$crate"
   done
   ```

`Cargo.lock` is committed in the repository for reproducible CI builds.
`cargo publish` strips it from the uploaded crate, so downstream consumers
get a fresh resolution against the lockfile's compatible versions.
