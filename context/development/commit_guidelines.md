# Commit Guidelines

- Concise, one-line messages (multi-line only when many changes)
- Group related files logically
- No emojis
- Use `git diff` to understand changes before committing
- **Never** include AI-agent signatures in commits
    - No "Co-Authored-By: Claude..."
    - No "Generated with [Claude Code]..."
    - No "Written with the help of ..."
- Never push without being asked

## CI: how your commits get checked (run these BEFORE you push)

`.github/workflows/ci.yml` runs `test` and `lint` on every push to `main` and every pull request, on Linux runners. The crate has no platform feature, so it builds without GTK or WebKit. Reproduce the gate locally first.

### 1. Format with nightly

`rustfmt.toml` enables `imports_granularity = "Crate"`, an unstable option, so stable `cargo fmt` silently skips it: your code looks formatted locally but fails CI.

```bash
cargo +nightly fmt        # not `cargo fmt`; stable can't apply the Crate imports rule
```

### 2. Clippy, warnings are errors, on both targets

```bash
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --target wasm32-unknown-unknown --all-features -- -D warnings
```

Lint levels live in `Cargo.toml` `[lints.clippy]`: the same 22 lints zwipe uses, with the panic family (`unwrap`, `expect`, `panic`, indexing, slicing) warned outside tests. `clippy.toml` allows it inside them. The wasm pass matters because the sites build the crate for the browser, where `theme_store` and the entrance timer take their other branch.

### 3. Tests

```bash
cargo test --all-features
```

Unit tests, plus the two stylesheet gates in `tests/` (`context/development/styling.md`). No environment, no network.

## Hosts pin a commit

Every host takes this crate as a git dependency and its lockfile pins the commit. A push here changes nothing downstream until a host runs `cargo update -p orangui` and commits its lockfile, which is deliberate: a host moves its pin when it is ready to test the result.
