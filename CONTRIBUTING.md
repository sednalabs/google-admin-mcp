# Contributing

Thanks for improving `google-admin-mcp`. Keep changes conservative: this
server is a local inspect/plan/verify helper for Google credential workflows.

## Development Principles

- Preserve the default `read_only` profile.
- Do not return or log access tokens, refresh tokens, client secrets, private
  keys, or raw credential files.
- Do not add remote credential installation or Workspace-wide mutations without
  explicit maintainer approval.
- Keep tool responses in Contract V1 form: `ok/data/meta` or `ok/error/meta`.
- Match the existing Rust module boundaries and naming conventions.
- Keep deployment-specific notes out of tracked public docs.

## Local Checks

For behavior changes:

```bash
cargo fmt --all --check
cargo test
cargo run -- --print-tools
```

For docs-only changes:

```bash
git diff --check
```

Live Google smoke checks require real local credentials. Run them only when
intentionally validating auth state.

## Documentation

Update public docs when changing tool names, argument semantics, profiles,
credential handling, or security behavior. Redact local identifiers and secret
material from examples.
