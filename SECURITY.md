# Security

## Reporting a Vulnerability

If you discover a security issue, please report it through the repository issue tracker with a private label or contact the maintainers directly.

## Supported Security Practices

- use strict parser validation for all external input
- verify CRC32 and footer checksums before decoding data
- reject malformed or truncated messages
- audit unsafe Rust and FFI boundaries carefully

## Responsible Disclosure

- avoid public disclosure until the issue is resolved
- allow maintainers a reasonable time to respond
- coordinate on a fix before wide publication

## Security Coverage

This repository documents security and audit guidance in `docs/SECURITY_AUDIT.md`, `docs/FUZZING.md`, and `docs/THREAT_MODEL.md`.
