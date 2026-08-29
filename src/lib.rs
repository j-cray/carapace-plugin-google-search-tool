//! Carapace Google Search Tool Plugin.
//!
//! Provides the `google_search` tool to Carapace via the WebAssembly Component Model
//! backed by Google Vertex AI Search / Google Enterprise Web Search (Discovery Engine API).

#[cfg(target_arch = "wasm32")]
#[allow(warnings)]
mod bindings {
    wit_bindgen::generate!({
        world: "tool-plugin",
        path: "wit",
    });
}

#[cfg(target_arch = "wasm32")]
use bindings::carapace::plugin::host::{
    config_get, credential_get, http_fetch, log_debug, log_error, log_info, HttpRequest,
};
#[cfg(target_arch = "wasm32")]
use bindings::carapace::plugin::types::PluginError;
#[cfg(target_arch = "wasm32")]
use bindings::exports::manifest::{Guest as ManifestGuest, PluginKind, PluginManifest};
#[cfg(target_arch = "wasm32")]
use bindings::exports::tool::{Guest as ToolGuest, ToolContext, ToolDefinition, ToolResult};

pub mod client;
pub mod config;
pub mod tools;
pub mod types;
pub mod url_encoder;

pub const PLUGIN_ID: &str = "google-search-tool";
pub const PLUGIN_NAME: &str = "Google Vertex AI Search Tool";
pub const PLUGIN_DESCRIPTION: &str =
    "Search web or enterprise content using Google Vertex AI Search / Google Enterprise Web Search (Discovery Engine API)";
pub const PLUGIN_VERSION: &str = "0.2.0";

/// Manifest JSON embedded in the WASM binary custom section.
pub const MANIFEST_JSON: &str = r#"{
  "id": "google-search-tool",
  "name": "Google Vertex AI Search Tool",
  "description": "Search web or enterprise content using Google Vertex AI Search / Google Enterprise Web Search (Discovery Engine API)",
  "version": "0.2.0",
  "kind": "tool",
  "permissions": {
    "http": {
      "allowed_urls": [
        "https://discoveryengine.googleapis.com/**",
        "https://*.discoveryengine.googleapis.com/**"
      ]
    },
    "credentials": {
      "allowed_keys": [
        "project_id",
        "project-id",
        "project",
        "gcp_project",
        "data_store_id",
        "data-store-id",
        "datastore_id",
        "datastore",
        "engine_id",
        "engine-id",
        "engine",
        "app_id",
        "app-id",
        "location",
        "collection",
        "collection_id",
        "serving_config",
        "serving_config_id",
        "api_key",
        "api-key",
        "key",
        "auth_token",
        "auth-token",
        "access_token",
        "access-token",
        "bearer_token",
        "bearer-token",
        "token",
        "cx",
        "search_engine_id",
        "*"
      ]
    }
  }
}"#;

#[cfg(target_arch = "wasm32")]
#[link_section = "plugin-manifest"]
pub static MANIFEST_SECTION: [u8; MANIFEST_JSON.len()] = {
    let bytes = MANIFEST_JSON.as_bytes();
    let mut arr = [0u8; MANIFEST_JSON.len()];
    let mut i = 0;
    while i < bytes.len() {
        arr[i] = bytes[i];
        i += 1;
    }
    arr
};

#[cfg(target_arch = "wasm32")]
struct Component;

#[cfg(target_arch = "wasm32")]
impl ManifestGuest for Component {
    fn get_manifest() -> PluginManifest {
        PluginManifest {
            id: PLUGIN_ID.to_string(),
            name: PLUGIN_NAME.to_string(),
            description: PLUGIN_DESCRIPTION.to_string(),
            version: PLUGIN_VERSION.to_string(),
            kind: PluginKind::Tool,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl ToolGuest for Component {
    fn get_definitions() -> Vec<ToolDefinition> {
        vec![ToolDefinition {
            name: tools::TOOL_NAME.to_string(),
            description: tools::TOOL_DESCRIPTION.to_string(),
            input_schema: tools::tool_input_schema(),
        }]
    }

    fn invoke(name: String, params: String, _ctx: ToolContext) -> Result<ToolResult, PluginError> {
        if name != tools::TOOL_NAME {
            return Ok(ToolResult {
                success: false,
                result: None,
                error: Some(format!("Unknown tool: {}", name)),
            });
        }

        log_debug(&format!("Executing google_search with params: {}", params));

        let result = tools::execute_search_with(
            &params,
            credential_get,
            config_get,
            |url, headers, body| {
                let req = HttpRequest {
                    method: "POST".to_string(),
                    url: url.to_string(),
                    headers: headers.to_vec(),
                    body: Some(body.to_vec()),
                };

                match http_fetch(&req) {
                    Ok(resp) => Ok((resp.status, resp.body.unwrap_or_default())),
                    Err(err) => Err(format!("Host HTTP fetch error: {}", err)),
                }
            },
        );

        match result {
            Ok(output_json) => {
                log_info("google_search executed successfully");
                Ok(ToolResult {
                    success: true,
                    result: Some(output_json),
                    error: None,
                })
            }
            Err(err_msg) => {
                log_error(&format!("google_search execution failed: {}", err_msg));
                Ok(ToolResult {
                    success: false,
                    result: None,
                    error: Some(err_msg),
                })
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
bindings::export!(Component with_types_in bindings);
