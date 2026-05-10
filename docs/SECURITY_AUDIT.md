# Security Audit

## Audit Goals

- verify parser hardening
- validate encryption and ECC metadata handling
- confirm checksum and corruption detection
- review unsafe Rust usage and FFI boundaries

## Audit Process

- review format spec for attack surfaces
- inspect code paths for header/footer parsing
- evaluate compression and decoding paths
- verify memory safety in `core/qrd-core/`
- assess FFI boundary safety in `core/qrd-ffi/`

## Responsible Disclosure

Security issues should be reported through the repository's security contact or issue process. See `SECURITY.md`.

## Hardening

- instrument parsers with strict bounds checks
- avoid assumptions about external input length
- document any unsafe code with rationale and review status
