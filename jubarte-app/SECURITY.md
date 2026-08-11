# Security Policy

## Supported versions

Only the latest released Mac App Store / notarized desktop build of Jubarte
receives security fixes. Older store builds are superseded by the next
submission.

## Reporting a vulnerability

**Do not open a public GitHub issue** for security reports. This repository
is private, but reports still go through a private channel.

Email: **security@jandira.tech** (or the contact listed on
[jubarte.pro](https://jubarte.pro) if that address changes)

Please include:

1. Affected version(s) and platform (macOS version, App Store vs Developer ID)
2. Impact (data exposure, privilege escalation, purchase bypass, etc.)
3. Steps to reproduce or a proof of concept
4. Whether you plan any public disclosure timeline

We aim to acknowledge reports within **5 business days** and to provide a
status update within **14 days**. Coordinated disclosure is preferred.

## Secrets and credentials

Never commit:

- App Store Connect API keys (`.p8`), issuer IDs, or JWTs
- Provisioning profiles or distribution certificates
- `.env` files, webhook secrets, or Cloudflare worker secrets
- Notarytool / altool passwords or keychain profile dumps

Use local-only scripts and environment variables. See `.gitignore`.

## Dependency advisories

Rust dependencies are audited in CI (`cargo audit` when configured). Report
supply-chain issues the same way as product vulnerabilities if they affect
shipped builds.
