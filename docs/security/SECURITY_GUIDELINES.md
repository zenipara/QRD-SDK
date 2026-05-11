# QRD-SDK Security Guidelines

## Overview

QRD-SDK is designed with security-first principles for production use. This guide covers:
- Threat model and vulnerabilities
- Secure configuration
- Encryption and key management
- Vulnerability disclosure
- Audit and compliance

---

## Threat Model

### Assumed Threats

1. **Unauthorized Data Access**
   - Attackers reading QRD files without authorization
   - **Mitigations:** File permissions, encryption at rest

2. **Data Corruption**
   - Accidental or malicious modification of QRD files
   - **Mitigations:** CRC32 checksums, SHA256 hashing, ECC recovery

3. **Malformed Input**
   - Attackers providing corrupted headers, row groups, or footers
   - **Mitigations:** Input validation,  bounds checking, format versioning

4. **Resource Exhaustion**
   - Memory bombs, parsing loops, allocations from untrusted data
   - **Mitigations:** Size limits, safe integer arithmetic, timeouts

5. **Cryptographic Weakness**
   - Weak keys, nonce reuse, side-channel attacks
   - **Mitigations:** AES-256-GCM, Argon2, constant-time operations

### Out of Scope

- Network security (use TLS for remote access)
- Authentication (use system user/role from host)
- Denial of service (resource quotas are host responsibility)
- Physical security (use OS file permissions)

---

## Secure Configuration

### 1. File Permissions

```bash
# Restrict to owner only
chmod 600 /data/qrd/*.qrd

# Restrict directory access
chmod 750 /data/qrd/

# Set ownership to dedicated user
sudo chown qrd:qrd /data/qrd/
```

### 2. Encryption at Rest

**Generate secure key:**
```bash
openssl rand -hex 32 > /etc/qrd/master.key.hex
chmod 600 /etc/qrd/master.key.hex
```

**Enable in config:**
```toml
[security]
encryption_enabled = true
key_provider = "file"  # or "env", "vault", "aws-kms"
key_path = "/etc/qrd/master.key.hex"
```

### 3. Key Management

**Never:**
```bash
❌ export KEY=abc123              # Visible in process list
❌ key="hardcoded" in code         # In version control
❌ Use same key for multiple apps  # Shared key risk
```

**Instead:**
```bash
✅ Use HashiCorp Vault
✅ Use AWS Secrets Manager
✅ Use Azure Key Vault
✅ Use file with restricted permissions
✅ Use environment variable injection at runtime
```

### 4. Access Control

```bash
# Create dedicated, unprivileged user
useradd -r -s /bin/false -m -d /var/lib/qrd qrd

# Set up systemd service with restrictions
[Service]
User=qrd
Group=qrd
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/data/qrd
```

---

## Encryption and Key Derivation

### Algorithm Details

**Encryption:** AES-256-GCM
- Key: 256 bits (32 bytes)
- Nonce: 96 bits (12 bytes), unique per encryption
- Authentication: GCM tag (16 bytes)
- No padding needed (GCM mode)

**Key Derivation:** Argon2id
- Time: 2 iterations
- Memory: 65536 KB (64 MB)
- Parallelism: 4 threads
- Output: 32 bytes (256-bit key)

```rust
use qrd_core::encryption::EncryptionConfig;

// Generate 256-bit key
let key = EncryptionConfig::generate_key();

// Or derive from password
let config = EncryptionConfig::derive_from_password(
    "user_password",
    &salt  // 32 bytes
)?;

// Encrypt data
let ciphertext = encrypt(&plaintext, &config)?;

// Decrypt data
let plaintext = decrypt(&ciphertext, &config)?;
```

### Key Rotation

```bash
# 1. Read all data with current key
qrd-convert --input master.qrd --output temp.csv --key $OLD_KEY

# 2. Re-encrypt with new key
qrd-convert --input temp.csv --output master_new.qrd --key $NEW_KEY

# 3. Verify integrity
qrd-validator master_new.qrd

# 4. Rotate keys securely
rm temp.csv
```

---

## Error Correction and Integrity

### CRC32 Checksums

Every file includes CRC32 checksums for:
- Header validation
- Row group integrity
- Footer integrity

```rust
use qrd_core::validation::calculate_crc32;

let crc = calculate_crc32(&data);
// Stored in file footer for verification
```

### Reed-Solomon ECC

For high-reliability scenarios:
```rust
use qrd_core::ecc::{EccCodec, EccConfig};

let config = EccConfig::with_chunk_size(2, 1024)?;
let mut codec = EccCodec::new(config)?;

// Encode with 2 parity chunks (recover from 2 lost chunks)
let encoded = codec.encode(&data)?;

// Later, recover from corruption
let recovered = recovery_attempt(&damaged_shards, &config)?;
```

---

## Input Validation

### Format Validation

All file reading implements strict validation:

```rust
// Header validation
if header.magic != QRD_MAGIC {
    return Err("Invalid magic bytes");
}
if header.version > LATEST_VERSION {
    return Err("Unsupported version");
}

// Schema validation
schema.validate_consistency()?;

// Row group validation
for row_group in &row_groups {
    validate_rowgroup_offsets(&offsets)?;
    validate_column_count(row_group)?;
    validate_encoding(row_group)?;
}
```

### Bounds Checking

```rust
// Safe integer arithmetic
let offset = chunk_offset.checked_add(chunk_size)
    .ok_or("Offset overflow")?;

// Size limits
const MAX_FILE_SIZE: u64 = 1 << 40;  // 1 TB
const MAX_ROW_SIZE: usize = 1 << 20; // 1 MB

if parsed_size > MAX_FILE_SIZE {
    return Err("File too large");
}
```

---

## Vulnerability Disclosure

### Report Security Issues

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, email: **security@qrd.dev**

Include:
- Description of vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Disclosure Timeline

1. **Day 1:** Vulnerability reported to security@qrd.dev
2. **Day 2:** Acknowledgment and initial assessment
3. **Day 3-7:** Investigation and fix development
4. **Day 8-10:** Fix testing and validation
5. **Day 14:** Security release published (30-day embargo from report)
6. **Day 14+:** Public disclosure allowed

---

## Compliance & Standards

### OWASP Top 10

QRD-SDK addresses OWASP risks:

| Risk | QRD Mitigation |
|---|---|
| Injection | Input validation, type safety (Rust) |
| Broken Auth | File permissions, OS-level auth |
| Sensitive Data Exposure | AES-256-GCM encryption, TLS for transport |
| XML External Entities | N/A (no XML parsing) |
| Broken Access Control | File permissions, user isolation |
| Security Misconfiguration | Documentation, defaults reviewed |
| XSS | N/A (binary format, no HTML) |
| Insecure Deserialization | Safe bincode/protobuf, validation |
| Using Known Vulnerable Components | Dependency audits, SBOM |
| Insufficient Logging | Audit trail support in roadmap |

### CWE Coverage

Rust language properties prevent many CWEs:

- ❌ CWE-119: Buffer overflow (memory safe)
- ❌ CWE-125: Out-of-bounds read (bounds checking)
- ❌ CWE-415: Double free (ownership system)
- ✅ CWE-476: Null pointer dereference (Option<T>)
- ✅ CWE-190: Integer overflow (checked arithmetic)
- ✅ CWE-400: Uncontrolled resource consumption (limits)

### GDPR Compliance

QRD supports data lifecycle requirements:

- **Data minimization:** Selective column reads
- **Right to deletion:** Selective row group deletion tools
- **Data portability:** Export to standard formats
- **Personal data protection:** Encryption at rest
- **Audit trails:** Checksums and version tracking

---

## Runtime Security

### Resource Limits

```toml
[security]
max_file_size = 1_000_000_000_000    # 1 TB
max_row_group_size = 1_000_000_000   # 1 GB
max_string_length = 10_000_000       # 10 MB
max_allocation_size = 1_000_000_000  # 1 GB
```

### Audit Logging

```bash
# Enable extended logging
export RUST_LOG=qrd_core=debug

# Log to file
export RUST_LOG_FILE=/var/log/qrd/audit.log

# Structured logging (JSON format)
export RUST_LOG_FORMAT=json
```

### Process Isolation

```dockerfile
# Run as non-root in container
RUN useradd -r -u 1000 qrd
USER qrd

# Restrict capabilities
RUN setcap -r /usr/local/bin/qrd-tool
```

---

## Security Testing

### Fuzz Testing

```bash
cargo fuzz run format_parser
cargo fuzz run encryption_roundtrip
```

### Sanitizers

```bash
# Memory sanitizer
RUSTFLAGS="-Z sanitizer=memory" cargo test

# Address sanitizer
RUSTFLAGS="-Z sanitizer=address" cargo test
```

### Penetration Testing

Suggested areas:
- Format parsing with craft malformed inputs
- Resource consumption with large row groups
- Encryption with weak keys/repeated nonces
- Concurrent access patterns

---

## Security Checklist for Operators

Before production deployment:

- [ ] File permissions correctly set (600 for files, 750 for directories)
- [ ] Encryption keys backed up securely
- [ ] Key rotation schedule established
- [ ] Access control implemented (dedicated user)
- [ ] Monitoring/logging enabled
- [ ] Backups tested and verified
- [ ] Disaster recovery procedure documented
- [ ] Team trained on security policies
- [ ] Audit logs configured
- [ ] Vulnerability disclosure process understood

---

## Additional Resources

- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)
- [CWE/SANS Top 25](https://cwe.mitre.org/top25/)
- [Rust Security](https://www.rust-lang.org/what/wg-security/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)

---

## Support

- **Security Questions:** security@qrd.dev
- **Documentation:** https://docs.qrd.dev
- **Issues:** https://github.com/zenipara/QRD-SDK/issues
