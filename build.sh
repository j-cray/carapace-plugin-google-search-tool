#!/usr/bin/env bash
set -euo pipefail

echo "========================================="
echo "Building Carapace Google Search Tool WASM"
echo "========================================="

TARGET="wasm32-unknown-unknown"
CRATE_NAME="carapace_plugin_google_search_tool"
OUT_COMPONENT="google-search-tool.wasm"

echo "1. Compiling Rust crate for ${TARGET}..."
cargo build --target "${TARGET}" --release

CORE_WASM="target/${TARGET}/release/${CRATE_NAME}.wasm"
if [ ! -f "${CORE_WASM}" ]; then
    echo "Error: Expected core wasm not found at ${CORE_WASM}" >&2
    exit 1
fi

echo "2. Packaging WebAssembly Component with wasm-tools..."
wasm-tools component new "${CORE_WASM}" -o "${OUT_COMPONENT}"

echo "3. Validating component..."
wasm-tools validate "${OUT_COMPONENT}"

SIZE=$(du -h "${OUT_COMPONENT}" | cut -f1)
SHA=$(sha256sum "${OUT_COMPONENT}" | cut -d' ' -f1)

echo "========================================="
echo "Build Successful!"
echo "Artifact:    ${OUT_COMPONENT}"
echo "Size:        ${SIZE}"
echo "SHA256:      ${SHA}"
echo "========================================="
