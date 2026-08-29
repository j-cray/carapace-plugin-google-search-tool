# carapace-plugin-google-search-tool

A WebAssembly (WASM) component plugin for [Carapace](https://github.com/puremachinery/carapace) that provides agents with Google Web & Image Search capabilities via the Google Custom Search JSON API.

---

## Features

- **Google Search Integration**: Exposes the `google_search` tool to LLMs running in Carapace.
- **Rich Query Parameters**: Supports query customization including result count (`num`), pagination (`start`), site filtering (`site`), file types (`file_type`), date ranges (`date_restrict`), SafeSearch (`safe`), language (`language`), country (`country`), and image search (`search_type`).
- **Flexible Credential Resolution**: Automatically resolves Google Cloud API Key and Search Engine ID (`cx`) from Carapace Credential Store, gateway configuration (`carapace.json`), or per-call parameters.
- **Sandboxed & Pure WebAssembly**: Targets `wasm32-unknown-unknown` without ambient WASI preview dependencies, running fully within Carapace's memory and capability sandbox.
- **Granular Security Permissions**: Declares precise HTTP URL allowlists and credential scopes in the embedded plugin manifest.

---

## Authentication & Prerequisites

To use this plugin, you will need:

1. **Google Cloud API Key** with the **Custom Search API** enabled.
2. **Search Engine ID (`cx`)** configured to search the web (or specific sites).

### 1. Get a Google Cloud API Key

1. Go to the [Google Cloud Console](https://console.cloud.google.com/).
2. Create or select a project.
3. Enable the **Custom Search API** in **APIs & Services > Library**.
4. Create an API Key in **APIs & Services > Credentials**.

### 2. Get a Search Engine ID (`cx`)

1. Open the [Programmable Search Engine](https://programmablesearchengine.google.com/) control panel.
2. Create a new search engine and enable **Search the entire web**.
3. Copy the **Search engine ID** (`cx`).

---

## Configuration in Carapace

You can supply credentials using either Carapace's credential store or the `carapace.json` configuration file:

### Option A: Carapace Credential Store (Recommended)

```bash
carapace credentials set google-search-tool:api_key "<YOUR_GOOGLE_API_KEY>"
carapace credentials set google-search-tool:cx "<YOUR_SEARCH_ENGINE_ID>"
```

### Option B: Carapace Configuration File (`carapace.json`)

```json
{
  "plugins": {
    "google-search-tool": {
      "api_key": "AIzaSy...",
      "cx": "0123456789abcdef:..."
    }
  }
}
```

---

## Tool Specification: `google_search`

### Parameters

| Parameter       | Type      | Required | Default    | Description                                                                           |
| :-------------- | :-------- | :------- | :--------- | :------------------------------------------------------------------------------------ |
| `query`         | `string`  | **Yes**  | —          | Search keywords or query string.                                                      |
| `num`           | `integer` | No       | `5`        | Number of results to return (`1`–`10`).                                               |
| `start`         | `integer` | No       | `1`        | 1-based index of first result for pagination (`1`–`91`).                              |
| `site`          | `string`  | No       | —          | Restrict results to a domain (e.g. `docs.rs` or `github.com`).                        |
| `file_type`     | `string`  | No       | —          | Restrict results by file extension (e.g. `pdf`, `json`).                              |
| `date_restrict` | `string`  | No       | —          | Date range filter (`d1` for 24h, `w1` for 1 week, `m1` for 1 month, `y1` for 1 year). |
| `safe`          | `string`  | No       | `"active"` | SafeSearch level (`"active"` or `"off"`).                                             |
| `language`      | `string`  | No       | —          | Language code filter (e.g. `lang_en`).                                                |
| `country`       | `string`  | No       | —          | Geolocation country code (e.g. `us`, `uk`).                                           |
| `search_type`   | `string`  | No       | `"web"`    | Search mode (`"web"` or `"image"`).                                                   |
| `api_key`       | `string`  | No       | —          | Optional per-invocation API key override.                                             |
| `cx`            | `string`  | No       | —          | Optional per-invocation Custom Search Engine ID override.                             |

### Example Tool Result

```json
{
  "query": "rust webassembly component model",
  "total_results": "248,000",
  "search_time": 0.24,
  "spelling_suggestion": null,
  "count": 2,
  "results": [
    {
      "title": "WebAssembly Components in Rust",
      "link": "https://component-model.bytecodealliance.org/language-support/rust.html",
      "snippet": "Learn how to build WebAssembly components in Rust using wit-bindgen and cargo-component.",
      "display_link": "component-model.bytecodealliance.org"
    },
    {
      "title": "Rust and WebAssembly",
      "link": "https://rustwasm.github.io/docs/book/",
      "snippet": "This book is for anyone interested in compiling Rust to WebAssembly.",
      "display_link": "rustwasm.github.io"
    }
  ]
}
```

---

## Building and Testing

### Prerequisites

- [Nix](https://nixos.org/) with Flakes enabled (provides Rust, `wasm-tools`, `wit-bindgen`, `cargo-component`).

### Build the WASM Plugin Component

Enter the Nix environment and run the build script:

```bash
nix develop
./build.sh
```

Or run via make:

```bash
make build
```

This generates `google-search-tool.wasm` in the root directory.

### Run Unit and Integration Tests

```bash
nix develop --command cargo test
```

### Inspect WIT Interface

```bash
make wit
```

---

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE)).
