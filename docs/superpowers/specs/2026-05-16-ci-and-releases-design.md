# CI and Releases — Design

**Date:** 2026-05-16
**Topic:** Restructure termcopy's CI and release workflows to match `tumbler`'s shape.

## Goal

Replace termcopy's Nix-based CI with a cargo-based CI matching tumbler, and add a tag-driven release workflow that builds and publishes binaries for four platforms.

## Motivation

- Tumbler's CI is faster (no Nix install/cache), simpler (fewer moving parts), and has no Cachix token dependency.
- Termcopy currently has no release automation; cutting a release is manual.
- Aligning the two projects on a single shape reduces cognitive overhead when maintaining both.

## Scope

In scope:
- Rewrite `.github/workflows/ci.yml`.
- Add `.github/workflows/release.yml`.
- Light cleanup of `justfile` recipes that conflict with the new workflow.

Out of scope:
- macOS code-signing / notarization.
- Windows code-signing.
- Publishing to crates.io.
- Pushing Docker images to a registry.
- Adding `cargo audit` to CI (stays in `justfile` for local use).

## CI workflow

File: `.github/workflows/ci.yml` (rewrite).

**Triggers:**
- `push` on all branches with `tags-ignore: ['**']`
- `pull_request`

**Single job, `ubuntu-latest`:**

1. `actions/checkout@v4`
2. Install Rust toolchain via `dtolnay/rust-toolchain@3c5f7ea28cd621ae0bf5283f0e981fb97b8a7af9` with `toolchain: '1.93.1'` and `components: clippy, rustfmt`.
3. `Swatinem/rust-cache@v2`
4. `cargo fmt --check`
5. `cargo clippy -- -D warnings`
6. `cargo test`

**Removed from CI:**
- Nix install (`cachix/install-nix-action`)
- Cachix cache setup (`cachix/cachix-action`)
- `nix develop --command just ci`
- Separate `nix build` job

**Trade-off accepted:** CI no longer exercises the Nix build path. Nix builds are still verifiable locally via `nix build` and via `just build-release-all`. If Nix-build regressions become a problem, a separate (optional) workflow can be added later.

**Toolchain version pinning:** the action specifies `1.93.1` explicitly to match `rust-toolchain.toml`. Both must be kept in sync when bumping Rust.

## Release workflow

File: `.github/workflows/release.yml` (new).

**Trigger:** `push` on tags matching `v*`.

**Permissions:** `contents: write`.

### Job 1: `build` (matrix)

Four targets:

| OS              | Target                          | Builder |
|-----------------|---------------------------------|---------|
| `ubuntu-latest` | `x86_64-unknown-linux-musl`     | `cross` |
| `ubuntu-latest` | `aarch64-unknown-linux-musl`    | `cross` |
| `macos-latest`  | `aarch64-apple-darwin`          | `cargo` |
| `windows-latest`| `x86_64-pc-windows-msvc`        | `cargo` |

**Per-target steps:**

1. `actions/checkout@v4`
2. Install Rust toolchain via `dtolnay/rust-toolchain@3c5f7ea28cd621ae0bf5283f0e981fb97b8a7af9` with `toolchain: '1.93.1'` and `targets: <matrix.target>`.
3. Install `cross` via `taiki-e/install-action@v2` when `matrix.use_cross` is true.
4. Build: `cross build --release --target <target>` or `cargo build --release --target <target>`.
5. Package:
   - Unix: `tar czf termcopy-<ref_name>-<target>.tar.gz termcopy` from `target/<target>/release`.
   - Windows: `Compress-Archive` `termcopy.exe` into `termcopy-<ref_name>-<target>.zip`.
6. Upload via `actions/upload-artifact@v4` with `if-no-files-found: error`.

### Job 2: `release`

Depends on `build`. `ubuntu-latest`.

1. `actions/download-artifact@v4` with `merge-multiple: true` into `artifacts/`.
2. `softprops/action-gh-release@v2` with `files: artifacts/*` and `generate_release_notes: true`.

### Notes on release artifacts

- Release tarballs contain only the `termcopy` binary. The `tc` symlink alias (created by `default.nix` `postInstall`) is **not** included. Users installing from release artifacts get `termcopy` only; the alias remains a Nix-build feature.
- Binary names: `termcopy-vX.Y.Z-<target>.tar.gz` (or `.zip` for Windows).

## Local file changes

### `justfile`

**Keep:** all local-dev recipes (`build`, `test`, `fmt`, `lint`, `dev`, `info`, etc.). The `ci` recipe stays — useful for running the equivalent of CI locally even though Actions no longer invokes it.

**Simplify the `release` recipe** from:

```
release version: fmt lint test build-release-all
    @echo "Release {{version}} prepared. Run 'just tag {{version}}' to create and push the tag."
```

to:

```
release version: fmt lint test
    @echo "Pre-release checks passed for {{version}}. Run 'just tag {{version}}' to cut the release."
```

Rationale: `build-release-all` (nix-based) is no longer the source of release binaries; CI is. The local `release` recipe should be a pre-tag sanity check, not a binary builder.

**Keep `build-release-all`** as-is — it's a useful "do my Nix builds still work" sanity check, independent of the release pipeline.

### Files unchanged

- `flake.nix`, `default.nix`, `rust-toolchain.toml`, `Cargo.toml`, `src/`, `tests/`, `CHANGELOG.md`, `README.md`.

## Release process (post-change)

1. Bump `version` in `Cargo.toml`.
2. Update `CHANGELOG.md`.
3. Optionally run `just release <version>` locally for a pre-flight check.
4. Commit and push to `main`.
5. `just tag <version>` — creates the `v<version>` annotated tag and pushes it.
6. The `release.yml` workflow builds all four targets and publishes the GitHub release automatically.

## Risks and mitigations

- **Windows build untested.** Source is pure stdio + base64, no platform-specific code, so it should build cleanly. First tagged release will be the verification. Mitigation: if the Windows job fails on the first tag, delete the tag, fix, retag — release flow is idempotent on the artifacts side.
- **Loss of Nix CI coverage.** Nix builds may silently break on `main`. Acceptable trade-off given simplicity gains; local `nix build` is the safety net.
- **Cachix token becomes unused.** Repo secret `CACHIX_AUTH_TOKEN` is no longer referenced. Safe to leave in place or remove separately; doesn't block this work.

## Acceptance criteria

- `ci.yml` runs on PRs and on feature-branch pushes; passes `fmt`, `clippy -D warnings`, `cargo test` against Rust 1.93.1.
- Pushing a `v*` tag triggers `release.yml`, produces four artifacts, and creates a GitHub release with auto-generated notes.
- `just dev` and `just ci` still work locally as before.
- No reference to `cachix/install-nix-action`, `cachix/cachix-action`, or `nix develop` remains in `.github/workflows/`.
