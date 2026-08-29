//! Integration test for Carapace Google Search Tool plugin.
//!
//! Tests that the compiled WASM component can be loaded by Carapace's
//! PluginLoader and PluginRuntime, registers its manifest and tools,
//! and properly exposes the `google_search` tool interface.

use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

use carapace::credentials::{CredentialStore, MockCredentialBackend};
use carapace::plugins::bindings::ToolContext;
use carapace::plugins::capabilities::{RateLimiterRegistry, SsrfConfig};
use carapace::plugins::loader::PluginLoader;
use carapace::plugins::runtime::PluginRuntime;
use carapace::plugins::sandbox::{CapabilityPolicy, SandboxConfig};
use carapace::plugins::PluginKind;

#[tokio::test]
async fn test_carapace_loads_google_search_tool_component() {
    let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("google-search-tool.wasm");
    if !wasm_path.exists() {
        eprintln!("google-search-tool.wasm not built yet, skipping test");
        return;
    }

    let temp_dir = tempdir().unwrap();
    let plugins_dir = temp_dir.path().join("plugins");
    std::fs::create_dir_all(&plugins_dir).unwrap();

    // Copy the compiled component into the test plugins directory
    let target_wasm = plugins_dir.join("google-search-tool.wasm");
    std::fs::copy(&wasm_path, &target_wasm).expect("Failed to copy WASM component");

    let loader = Arc::new(PluginLoader::new(plugins_dir).unwrap());
    let backend = MockCredentialBackend::new(true);
    let credential_store = Arc::new(
        CredentialStore::new(backend, temp_dir.path().to_path_buf())
            .await
            .unwrap(),
    );

    let sandbox_config = SandboxConfig {
        enabled: true,
        allow_tailscale: false,
        defaults: CapabilityPolicy {
            allow_http: true,
            allow_credentials: true,
            allow_media: false,
        },
        overrides: std::collections::HashMap::new(),
    };

    let runtime = PluginRuntime::with_full_config(
        loader.clone(),
        credential_store,
        Arc::new(RateLimiterRegistry::new()),
        SsrfConfig::default(),
        sandbox_config,
    )
    .unwrap();

    let plugin_id = loader.load_plugin(&target_wasm).expect("Failed to load plugin");
    assert_eq!(plugin_id, "google-search-tool");

    let loaded = loader.get_plugin("google-search-tool").expect("Plugin should be loaded");
    let manifest = &loaded.manifest;
    assert_eq!(manifest.id, "google-search-tool");
    assert_eq!(manifest.kind, PluginKind::Tool);
    assert!(!manifest.name.is_empty());
    assert!(!manifest.version.is_empty());

    // Instantiate plugin in runtime with sandbox permissions enabled
    runtime.instantiate_plugin("google-search-tool").await.expect("Plugin instantiation failed");

    // Verify tool definitions in registry
    let tools = runtime.registry().get_tools();
    let (_, tool_instance) = tools
        .into_iter()
        .find(|(id, _)| id == "google-search-tool")
        .expect("Tool plugin instance should be registered");

    let defs = tool_instance.get_definitions().expect("Should get tool definitions");
    let search_tool = defs.iter().find(|d| d.name == "google_search").expect("Tool 'google_search' should exist");

    assert!(search_tool.description.contains("Google Search"));
    assert!(search_tool.input_schema.contains("query"));
    assert!(search_tool.input_schema.contains("exact_terms"));
    assert!(search_tool.input_schema.contains("exclude_terms"));
    assert!(search_tool.input_schema.contains("sort"));

    let ctx = ToolContext {
        agent_id: None,
        session_key: None,
        message_channel: None,
        sandboxed: false,
    };

    // Verify invocation with empty arguments produces validation error as expected
    let tool_res = tool_instance
        .invoke("google_search", "{}", ctx.clone())
        .expect("Tool invocation call should succeed");

    assert!(!tool_res.success);
    assert!(tool_res.error.is_some());
    let err = tool_res.error.unwrap();
    assert!(
        err.contains("missing field `query`") || err.contains("Invalid arguments"),
        "Unexpected error message: {}",
        err
    );

    // Verify invocation with whitespace query produces validation error
    let whitespace_res = tool_instance
        .invoke("google_search", r#"{"query": "   "}"#, ctx.clone())
        .expect("Tool invocation call should succeed");
    assert!(!whitespace_res.success);
    let ws_err = whitespace_res.error.expect("Expected error for empty query");
    assert!(ws_err.contains("Argument 'query' must not be empty."));

    // Verify invocation with unknown tool name returns error
    let unknown_res = tool_instance
        .invoke("unknown_search_tool", "{}", ctx)
        .expect("Tool invocation call should succeed");
    assert!(!unknown_res.success);
    let unk_err = unknown_res.error.expect("Expected error for unknown tool");
    assert!(unk_err.contains("Unknown tool: unknown_search_tool"));
}
