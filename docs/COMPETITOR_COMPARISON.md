# Competitor Comparison

This document compares QRD to common storage and analytics formats.

| Property | QRD | Parquet | Arrow IPC | CSV | SQLite |
|---|---|---|---|---|---|
| Format type | Columnar binary container | Columnar binary file | In-memory/binary IPC | Text table | Embedded relational file |
| Streaming write | native row-group stream | requires buffering | not designed for streaming | yes (no schema) | limited |
| Offline-first | yes | yes, but ecosystem-heavy | no | yes | yes |
| Partial column read | yes | yes | yes | no | query-bound |
| Schema embedded | yes | yes | yes | no | yes |
| Compression built-in | yes | yes | yes | no | optional |
| Encryption built-in | metadata-aware | external | external | no | optional |
| Error correction | optional ECC | none | none | none | none |
| Browser/WASM | supported | limited | supported via Arrow JS | yes | no |
| Cross-language fidelity | single engine | multiple implementations | reference implementation | trivial | single engine |

## Notes

- QRD is positioned as an edge-native analytical container rather than a general OLTP store.
- Parquet is a mature ecosystem for analytics, but it is not optimized for bounded-memory streaming writes in constrained hosts.
- Arrow is primarily an in-memory analytic representation; Arrow IPC is not a streaming-first format.
- CSV is useful for interchange, but lacks schema, compression, and binary efficiency.
- SQLite is a general-purpose embedded database, not a streaming columnar container.

## Use Cases

QRD is intended for:

- local telemetry capture and analytics
- browser-based feature stores
- embedded inference caches
- reproducible cross-language data exchange
