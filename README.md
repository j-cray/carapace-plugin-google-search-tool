# carapace-plugin-google-search-tool

A WebAssembly (WASM) component plugin for [Carapace](https://github.com/puremachinery/carapace) that provides agents with **real-time Google Web Search across the entire public internet** via Google Search Grounding (Google AI Studio & Google Cloud Vertex AI).

---

## Features

- **Whole-Web Search**: Searches Google's entire global public index in real time with **zero domain restrictions** or manual indexing required.
- **Synthesized Answers & Live Citations**: Returns comprehensive answers grounded in live web results, search queries, page titles, URLs, and supported snippets.
- **Simple 1-Minute Setup**: Works out of the box with a standard Gemini API Key from Google AI Studio (`aistudio.google.com`). No complex GCP cloud projects or custom search engines required.
- **Enterprise Vertex AI Support**: Also supports enterprise Google Cloud Vertex AI projects using GCP Project ID and OAuth2 Bearer tokens or API keys.
- **Rich Query Options**: Supports citation limits (`num`/`page_size`), domain filtering (`site`), exact phrase matching (`exact_terms`), exclusion terms (`exclude_terms`), and model selection (`model`, default: `gemini-2.5-flash`).
- **Sandboxed WebAssembly**: Built for `wasm32-unknown-unknown` running under Carapace's capability-based sandbox with strict HTTP allowlisting and credential isolation.

---

## Quick Setup (1-Minute)

### 1. Get a Gemini API Key

1. Visit [Google AI Studio](https://aistudio.google.com/app/apikey).
2. Click **Create API Key**.

### 2. Configure Credentials in Carapace

Set your API key using the Carapace CLI:

```bash
carapace config set google-search-tool:api_key "<YOUR_GEMINI_API_KEY>"
```

_(Alternatively, add it to your `carapace.json`)_:

```json
{
  "plugins": {
    "google-search-tool": {
      "api_key": "AIzaSy..."
    }
  }
}
```

That's it! Your Carapace agents can now search the entire web using `google_search`.

---

## Enterprise Setup (Google Cloud Vertex AI)

If you prefer to route requests through an enterprise Google Cloud Vertex AI project:

```bash
# Set your Google Cloud Project ID
carapace credentials set google-search-tool:project_id "<YOUR_GCP_PROJECT_ID>"

# Set your OAuth2 Bearer token (or Vertex-enabled API key)
carapace credentials set google-search-tool:auth_token "<YOUR_ACCESS_TOKEN>"
```

---

## Tool Specification: `google_search`

### Parameters

| Parameter           | Type      | Required | Default              | Description                                                    |
| :------------------ | :-------- | :------- | :------------------- | :------------------------------------------------------------- |
| `query`             | `string`  | **Yes**  | —                    | Search keywords or question to search across the entire web.   |
| `num` / `page_size` | `integer` | No       | `5`                  | Maximum number of web citation sources to return (`1`–`20`).   |
| `model`             | `string`  | No       | `"gemini-2.5-flash"` | Gemini model used for grounded search retrieval.               |
| `site`              | `string`  | No       | —                    | Optional domain filter (e.g. `docs.rs` or `github.com`).       |
| `exact_terms`       | `string`  | No       | —                    | Phrase that all search results must contain.                   |
| `exclude_terms`     | `string`  | No       | —                    | Word or phrase that should not appear in search results.       |
| `api_key`           | `string`  | No       | Config               | Optional per-invocation API key override.                      |
| `auth_token`        | `string`  | No       | Config               | Optional per-invocation Bearer token override for Vertex AI.   |
| `project_id`        | `string`  | No       | Config               | Optional per-invocation Google Cloud Project ID for Vertex AI. |
| `location`          | `string`  | No       | `"us-central1"`      | Google Cloud region for Vertex AI.                             |

### Example Tool Output

```json
{
  "query": "latest rust releases and webassembly component model updates",
  "summary": "Rust 1.85 was recently released with async closures, standard library stabilization, and enhanced WebAssembly Component Model support with wit-bindgen 0.36.",
  "search_queries": ["rust 1.85 release notes webassembly component model"],
  "count": 2,
  "results": [
    {
      "title": "Announcing Rust 1.85.0 | Rust Blog",
      "link": "https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html",
      "snippet": "Rust 1.85.0 includes stabilization of async closures and updated wasm targets.",
      "display_link": "blog.rust-lang.org"
    },
    {
      "title": "WebAssembly Component Model in Rust",
      "link": "https://component-model.bytecodealliance.org/language-support/rust.html",
      "snippet": "Learn how to build WebAssembly components in Rust with cargo-component.",
      "display_link": "component-model.bytecodealliance.org"
    }
  ]
}
```

---

## Building and Testing

### Prerequisites

- [Nix](https://nixos.org/) with Flakes enabled (provides Rust, `wasm-tools`, `wit-bindgen`, `cargo-component`).

### Build the WASM Plugin Component

```bash
nix develop
./build.sh
# or
make build
```

This compiles and packages `google-search-tool.wasm`.

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
