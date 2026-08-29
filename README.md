# carapace-plugin-google-search-tool

A WebAssembly (WASM) component plugin for [Carapace](https://github.com/puremachinery/carapace) that provides agents with Google Web, Website, and Enterprise Search capabilities via the Google Cloud Vertex AI Search & Discovery Engine REST API (`discoveryengine.googleapis.com`).

---

## Features

- **Vertex AI Search & Enterprise Web Search**: Exposes the `google_search` tool to LLMs running in Carapace backed by Google Cloud Vertex AI Search.
- **Website & Data Store Indexing**: Connects to Vertex AI Website Data Stores (public and domain-verified websites) or Enterprise Search Apps.
- **Generative Summaries & Snippets**: Returns AI-generated summaries, extracted snippets with highlighted keywords, and spelling corrections alongside structured search results.
- **Rich Query Parameters**: Supports query customization including result count (`num`/`page_size`), pagination (`start`/`offset`), domain filtering (`site`), file types (`file_type`), exact phrase matching (`exact_terms`), exclusion terms (`exclude_terms`), structured filters (`filter`), and sorting (`order_by`/`sort`).
- **Flexible Credential Resolution**: Automatically resolves Google Cloud Project ID, Data Store / Engine ID, and authentication (API Key or OAuth2 Bearer Token) from Carapace Credential Store, gateway configuration (`carapace.json`), or per-call parameters.
- **Sandboxed & Pure WebAssembly**: Targets `wasm32-unknown-unknown` without ambient WASI preview dependencies, running fully within Carapace's memory and capability sandbox.
- **Granular Security Permissions**: Declares precise HTTP URL allowlists and credential scopes in the embedded plugin manifest.

---

## Authentication & Prerequisites

To use this plugin with Google Vertex AI Search:

1. **Google Cloud Project**: An active GCP project linked to billing with the **Discovery Engine API** (`discoveryengine.googleapis.com`) enabled.
2. **Vertex AI Search Data Store or App**: A website or unstructured data store configured in Vertex AI Agent Builder.
3. **Authentication**: An **API Key** or **OAuth2 Access Token** (`auth_token`).

### 1. Enable Discovery Engine API in Google Cloud

1. Go to the [Google Cloud Console](https://console.cloud.google.com/).
2. Select your project and navigate to **APIs & Services > Library**.
3. Search for and enable **Discovery Engine API** (or run `gcloud services enable discoveryengine.googleapis.com`).

### 2. Create a Website Data Store or Search App

1. In Google Cloud Console, open **Vertex AI Agent Builder** (or **Search & Conversation** / **Vertex AI Search**).
2. Navigate to **Data Stores** > **Create Data Store**.
3. Select **Website content**:
   - Enter the target sites/domains to index (e.g. `docs.rs/*`, `github.com/*`, or your documentation websites).
   - Choose location (default: `global`) and set a **Data Store ID** (e.g. `my-web-store`).
4. (Optional) Create a **Search App** attaching the data store if you wish to use an App / Engine ID.

### 3. Get Credentials

- **API Key (Quickest)**: Create an API key in **APIs & Services > Credentials** and optionally restrict it to the _Discovery Engine API_.
- **OAuth2 Token / Service Account (Enterprise)**: Use an access token generated from `gcloud auth print-access-token` or a service account with the _Discovery Engine Viewer_ role.

---

## Configuration in Carapace

You can supply credentials using either Carapace's credential store or the `carapace.json` configuration file:

### Option A: Carapace Credential Store (Recommended)

```bash
# Set your Google Cloud Project ID
carapace credentials set google-search-tool:project_id "<YOUR_GCP_PROJECT_ID>"

# Set your Vertex AI Data Store ID (or engine_id)
carapace credentials set google-search-tool:data_store_id "<YOUR_DATA_STORE_ID>"

# Set your API Key (or auth_token)
carapace credentials set google-search-tool:api_key "<YOUR_GOOGLE_API_KEY>"
```

_Note: If using OAuth2 tokens instead of an API key, set `auth_token` instead:_

```bash
carapace credentials set google-search-tool:auth_token "<YOUR_BEARER_TOKEN>"
```

### Option B: Carapace Configuration File (`carapace.json`)

```json
{
  "plugins": {
    "google-search-tool": {
      "project_id": "my-gcp-project-12345",
      "data_store_id": "my-web-store",
      "api_key": "AIzaSy...",
      "location": "global"
    }
  }
}
```

---

## Tool Specification: `google_search`

### Parameters

| Parameter           | Type      | Required | Default                | Description                                                          |
| :------------------ | :-------- | :------- | :--------------------- | :------------------------------------------------------------------- |
| `query`             | `string`  | **Yes**  | —                      | Search keywords or query string.                                     |
| `num` / `page_size` | `integer` | No       | `5`                    | Number of results to return (`1`–`100`).                             |
| `start`             | `integer` | No       | `1`                    | 1-based index of first result for pagination.                        |
| `offset`            | `integer` | No       | `0`                    | 0-based offset for pagination.                                       |
| `site`              | `string`  | No       | —                      | Restrict results to a domain (e.g. `docs.rs` or `github.com`).       |
| `file_type`         | `string`  | No       | —                      | Restrict results by file extension (e.g. `pdf`, `html`).             |
| `exact_terms`       | `string`  | No       | —                      | Phrase that all documents in search results must contain.            |
| `exclude_terms`     | `string`  | No       | —                      | Word or phrase that should not appear in search results.             |
| `filter`            | `string`  | No       | —                      | Filter expression for structured metadata attributes.                |
| `order_by` / `sort` | `string`  | No       | —                      | Order expression to apply to the results (e.g. `publish_time desc`). |
| `project_id`        | `string`  | No       | Config                 | Optional per-invocation Google Cloud Project ID override.            |
| `data_store_id`     | `string`  | No       | Config                 | Optional per-invocation Data Store ID override.                      |
| `engine_id`         | `string`  | No       | Config                 | Optional per-invocation Engine/App ID override.                      |
| `location`          | `string`  | No       | `"global"`             | Google Cloud region / location (e.g. `global`, `us`, `eu`).          |
| `collection_id`     | `string`  | No       | `"default_collection"` | Collection ID override.                                              |
| `serving_config_id` | `string`  | No       | `"default_search"`     | Serving configuration ID override.                                   |
| `api_key`           | `string`  | No       | Config                 | Optional per-invocation API key override.                            |
| `auth_token`        | `string`  | No       | Config                 | Optional per-invocation Bearer token override.                       |

### Example Tool Result

```json
{
  "query": "rust webassembly component model",
  "total_results": "120",
  "summary": "WebAssembly Components in Rust allow building modular, interoperable WASM modules using wit-bindgen.",
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
