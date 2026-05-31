# AGENTS.md - google-admin-mcp

## Scope

- Applies to this repository.

## Operating intent

- Reduce repeated Google credential friction for downstream MCPs.
- Keep v1 local, auditable, and conservative: inspect first, plan next, mutate only behind explicit profile gates.
- Prefer official Google CLI/API surfaces over dashboard-only instructions whenever they exist.
- Never print, log, or return refresh tokens, access tokens, client secrets, private keys, or raw credential files.

## Architecture boundaries

- `main.rs`: bootstrap, CLI parsing, stdio transport.
- `server.rs`: MCP protocol handler and tool routing.
- `tools.rs`: MCP argument structs and tool handlers.
- `google_cli.rs`: `gcloud`/local credential adapter.
- `google_api.rs`: authenticated Google API calls.
- `config.rs`: settings and capability profile.
- `contract.rs`: Contract V1 response envelopes.
- `error.rs`: shared error model.

## Safety

- Default profile is `read_only`; mutating tools must fail closed unless `GOOGLE_ADMIN_MCP_PROFILE=operator`.
- Do not add Workspace-wide or domain-wide delegation mutations without explicit approval.
- Do not install credentials on remote hosts in v1; return a plan only.
- Redact secret-bearing paths and values from error details where possible.

## Quality bar

- Use `mcp-toolkit-rs` for shared MCP model helpers and observability where sensible.
- Add tests for redaction, profile gates, and command construction when changing auth flows.
- Keep README current when tool names, env vars, or safety posture changes.
