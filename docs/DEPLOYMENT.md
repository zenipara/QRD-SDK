# Deployment

## Deployment Targets

- embedded devices
- edge gateways
- browser and WASM runtimes
- cloud services consuming telemetry files

## Native Deployment

The Rust core can be packaged as a static binary or library. It is suitable for Linux, macOS, and Windows native environments.

## Container Deployment

Containers should include the Rust runtime or the compiled native artifacts for the target platform.

## Browser Deployment

WASM builds should be distributed through standard package managers and loaded using the TypeScript bindings.

## Platform Notes

- keep row group sizes small in constrained environments
- use compression codecs appropriate for the target workload
- avoid full-file loads in low-memory deployments
