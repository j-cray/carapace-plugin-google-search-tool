# Agent Instructions

## Project Overview

This project is the **Google Search Tool Plugin** for **Carapace** (`carapace-plugin-google-search-tool`).

It provides Carapace agents with real-time, global web search capabilities across the entire public internet by implementing the `google_search` tool over **Google Search Grounding** (Google AI Studio / Gemini API and Google Cloud Vertex AI).

### What is Carapace?

[Carapace](https://github.com/puremachinery/carapace) is a security-focused, open-source personal AI assistant written in Rust that runs locally on the user's machine. It connects to multiple messaging channels (such as Matrix, Signal, Telegram, Discord, Slack, webhooks, and console), supports multiple LLM providers, and provides an extensible, sandboxed WebAssembly (WASM) plugin runtime (supporting `tool-plugin`, `webhook-plugin`, `service-plugin`, and `channel-plugin`) with strict capability permissions and resource limits.

## Project Maintenance & Documentation

- **Keep Descriptions Updated**: As this project evolves or is fleshed out into a specific plugin implementation, agents **MUST** update the project description, purpose, and usage instructions in both [`GEMINI.md`](GEMINI.md) and [`README.md`](README.md) to accurately reflect the current state, WIT targets, and functionality of the plugin.

## Submodule Restriction

- **NEVER** make changes, create files, or modify anything inside the `carapace/` submodule directory.
- The `carapace/` submodule is strictly read-only and provided for reference only.
- All development and changes must take place exclusively within the main repository.

## Documentation & Development Guidelines

- Always refer to [`docs/plugin-development.md`](docs/plugin-development.md) for plugin development guidelines, WIT specifications, architecture details, and runtime workflows.

## Managing Nix Flake Dependencies

- The repository provides a Nix flake ([`flake.nix`](flake.nix)) and [`.envrc`](.envrc) that define and automatically load the development shell with all required tooling (`cargo-component`, `wasm-tools`, `wit-bindgen`, Rust toolchain with `wasm32-wasip1` / `wasm32-unknown-unknown` targets, etc.).
- When adding new tools, libraries, build dependencies, or compiler targets/extensions required by the project:
  - Update [`flake.nix`](flake.nix) in the workspace root by adding new packages to `devShells.default.buildInputs` or configuring `rustToolchain`.
  - Ensure any new or modified files are staged in Git (`git add flake.nix flake.lock`) so Nix can evaluate them in pure mode.
  - Verify the shell evaluates and builds cleanly with `nix develop` or `direnv reload`.
  - Always prefer maintaining dependencies declaratively in [`flake.nix`](flake.nix) over relying on unmanaged host-level installations.

## Architectural Constraints & Lessons Learned

### 1. No WASI Host Linker Support

- **Issue**: Carapace's WebAssembly host runtime (`wasmtime::component::Linker`) only provides host functions for the `carapace:plugin/host@1.0.0` interface (`config-get`, `credential-get`, `credential-set`, `http-fetch`, `media-fetch`, `log-debug`, `log-info`, `log-warn`, `log-error`). It does **NOT** link WASI Preview 2 modules (`wasi:cli/*`, `wasi:filesystem/*`, `wasi:clocks/*`, etc.).
- **Consequence**: Compiling for `wasm32-wasip1` attaches the `wasi_snapshot_preview1` adapter, which generates `wasi:cli/environment@0.2.3` and filesystem imports that fail at instantiation with:
  `Plugin instantiation failed: component imports instance wasi:cli/environment@0.2.3, but a matching implementation was not found in the linker`
- **Solution**: The plugin must always compile for the pure **`wasm32-unknown-unknown`** target.

### 2. Pure Rust & Sandboxed HTTP Access

- Plugins run in a sandboxed WASM environment without ambient host filesystem or socket syscalls.
- Remote network operations execute over HTTP/HTTPS via `carapace:plugin/host::http-fetch`.
- Dependencies must be pure Rust without OS/libc dependencies (`serde` and `serde_json` with `alloc`, custom URL percent encoder).

### 3. Plugin Manifest ID & Artifact Naming

- Carapace's plugin ID validator requires alphanumeric lowercase identifiers with dashes (`^[a-z0-9-]+$`).
- The plugin manifest ID is **`google-search-tool`**.
- The WASM component filename stem must match the plugin ID: **`google-search-tool.wasm`**.

### 4. Component Export Naming & Inline World Interfaces

- **Issue**: Carapace's plugin runtime (`carapace/src/plugins/runtime.rs`) looks up plugin exports by interface name using exact string matching on component export instances via `component.get_export_index(None, "tool")` and `component.get_export_index(None, "manifest")`.
- **Consequence**: When a WIT world defines exports using package references like `export manifest; export tool;` within `package carapace:plugin@1.0.0;`, WASM component tooling encodes them with fully qualified package names (`"carapace:plugin/manifest@1.0.0"` and `"carapace:plugin/tool@1.0.0"`). Carapace's exact instance lookup then fails with:
  `Binding error: Function call error: exported interface 'tool' not found`
- **Solution**: The `tool-plugin` world in `wit/plugin.wit` must export `manifest` and `tool` as inline named interfaces (`export manifest: interface { ... }` and `export tool: interface { ... }`). This ensures the final WASM component exports top-level instances named `"manifest"` and `"tool"`, matching Carapace's index lookup, and generates Rust bindings under `bindings::exports::manifest` and `bindings::exports::tool`.
