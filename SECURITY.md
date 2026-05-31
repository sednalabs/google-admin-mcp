# Security Policy

## Supported Versions

Security fixes are accepted against the current `main` branch.

## Reporting a Vulnerability

Please report suspected vulnerabilities through a private maintainer channel
when available. If public reporting is unavoidable, redact sensitive details and
provide a minimal reproduction that does not expose real credentials.

Do not include:

- access tokens, refresh tokens, OAuth client secrets, service-account JSON, ADC
  files, or private keys;
- local usernames, hostnames, IP addresses, absolute home paths, or deployment
  names;
- project ids, GA account ids, or property ids unless required for the report.

## Project Security Posture

- The default profile is `read_only`.
- V1 tools inspect, plan, validate, or smoke-test local Google auth state.
- V1 does not install credentials on remote hosts.
- Tokens are used only in memory for the live GA4 smoke call and are not
  returned in tool responses.

See [docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md) for details.
