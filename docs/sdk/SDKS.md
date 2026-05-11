# SDKs

## Supported SDKs

The QRD SDK family is centered on one Rust core engine and thin language-specific bindings.

| SDK | Path | Current Status |
|---|---|---|
| Rust | `core/qrd-core/` | reference implementation |
| Python | `sdk/python/` | in-progress, PyO3 binding |
| TypeScript / WASM | `sdk/typescript/` | in-progress, browser/runtime support |
| Go | `sdk/go/` | in-progress, CGO wrapper |
| Java | `sdk/java/` | in-progress, JNA/JNI wrapper |
| C/C++ | `core/qrd-ffi/`, `sdk/go/qrd.c` | low-level C-compatible interface |

## Installation

### Rust

Build the core engine from the repository root:

```bash
cargo build --workspace --release
```

### Python

The Python binding is packaged under `sdk/python/`.

```bash
cd sdk/python
python -m pip install --user .
```

If you are developing the Python SDK, use an editable install:

```bash
cd sdk/python
python -m pip install --user -e .
```

### TypeScript / WASM

The TypeScript/WASM package is managed in `sdk/typescript/`.

```bash
cd sdk/typescript
npm install
npm run build
```

WASM packaging is intended to produce a browser-compatible module and a Node.js-compatible runtime wrapper.

### Go

The Go binding is implemented in `sdk/go/` and depends on the C-compatible FFI header.

```bash
cd sdk/go
go test ./...
```

### Java

The Java binding lives in `sdk/java/`.

```bash
cd sdk/java
mvn package
```

### C/C++

The C-compatible layer is defined by `core/qrd-ffi/` and can be used by C/C++ systems.

Use the header and library produced by the Rust build to link native applications.

## Example Usage

### Python (example)

```python
from qrd import Reader, Schema

schema = Schema([
    ("id", "INT64", False),
    ("value", "FLOAT64", True),
])

with open("output.qrd", "rb") as f:
    reader = Reader(f)
    print(reader.schema)
```

### Go (example)

```go
package main

import (
    "fmt"
    "os"
    "github.com/zenipara/QRD-SDK/sdk/go"
)

func main() {
    file, _ := os.Open("output.qrd")
    reader := qrd.NewReader(file)
    fmt.Println(reader.Schema())
}
```

## Platform Notes

- Native Rust is the authoritative implementation.
- Bindings are thin wrappers around the Rust core engine.
- WASM builds are intended for browser and lightweight runtime environments.
- Platform-specific packaging and runtime requirements are documented in each SDK directory.

## Stability Status

- `core/qrd-core/` is the reference engine and the source of truth for format semantics.
- Language bindings are under active development and may not be production-ready.
- Users should verify binding compatibility and runtime support before relying on a SDK for production.

## Documentation and Support

- Consult `docs/FORMAT_SPEC.md` for format-level guarantees.
- Consult `docs/sdk/SDKS.md` for binding status and install notes.
- Report issues or feature requests through the repository issue templates.
