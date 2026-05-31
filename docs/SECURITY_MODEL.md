# Security Model

`google-admin-mcp` is an auth helper, not an auth bypass. Its job is to make
local Google credential state easier to inspect and verify while keeping secret
material out of MCP responses.

## V1 Boundary

The v1 tool surface is read-only and local:

- inspect `gcloud` and ADC status;
- build a `gcloud auth application-default login` command without running it;
- validate OAuth client JSON metadata without returning client secrets;
- list visible Google Cloud projects through `gcloud`;
- smoke-test GA4 account-summary access using the current ADC access token;
- search tool metadata for deferred-loading clients.

The server does not install credentials on remote hosts in v1. Future mutating
tools must be behind an explicit profile gate.

## Secret Handling

Tools must not print, log, or return:

- access tokens or refresh tokens;
- OAuth client secrets;
- service-account private keys;
- raw ADC or credential file contents.

The GA4 smoke test uses an ADC access token in memory for the upstream Google
API request and returns only the Contract V1 response.

## Profiles

- `GOOGLE_ADMIN_MCP_PROFILE=read_only` is the default.
- `GOOGLE_ADMIN_MCP_PROFILE=operator` is reserved as an explicit gate for
  future mutating tools.

Do not add broad Workspace-wide or domain-wide delegation mutations without
maintainer approval and dedicated tests for denial, redaction, and rollback
behavior.

## Least-Privilege Scopes

The generated ADC login command defaults to:

```text
https://www.googleapis.com/auth/analytics.readonly
https://www.googleapis.com/auth/cloud-platform
```

Use narrower scopes where the downstream Google API flow allows it.

## Redaction Guidance

Before sharing logs or issue reports, redact:

- credential file contents, tokens, client secrets, and private keys;
- local usernames, home paths, hostnames, IP addresses, and deployment-specific
  identifiers;
- project ids, GA account ids, or property ids unless they are intentionally
  required for the report.

See [../SECURITY.md](../SECURITY.md) for reporting guidance.
