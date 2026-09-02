# Contributing to GitNapse

Thanks for your interest in contributing.

## Workflow

1. Create a branch from `main`.
2. Implement your change.
3. Run local validation.
4. Open a Pull Request targeting `main`.

`main` is protected. Direct pushes are not allowed.

## Local Validation

The CI runs four mandatory checks. Run **all of them locally** before pushing or opening a PR:

```bash
./scripts/ci.sh
```

This executes:

- `cargo fmt --all -- --check` — style: must pass without diff
- `cargo clippy --all-targets --all-features -- -D warnings` — lints: zero warnings allowed
- `cargo test --all-targets --all-features` — tests: all must pass
- `cargo audit` — dependency vulnerability scan

To auto-fix formatting issues:

```bash
cargo fmt
```

If your changes affect behavior, update documentation in `README.md` and `docs/`.

## Pull Request Guidelines

- Keep PRs focused and scoped.
- Describe motivation, implementation details, and test evidence.
- Link related issues if available.
- Resolve review comments before merge.

## Commit Guidance

Use clear commit messages, for example:

- `feat: add authenticated @me repository listing`
- `fix: handle oauth runtime initialization`
- `docs: add release collaboration section`

## Security

Do not open public issues for sensitive vulnerabilities. Use the process in [SECURITY.md](./SECURITY.md).
