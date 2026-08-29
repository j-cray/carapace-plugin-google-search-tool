.PHONY: all build test clean wit

all: build

build:
	./build.sh

test:
	cargo test

wit: build
	wasm-tools component wit google-search-tool.wasm

clean:
	cargo clean
	rm -f google-search-tool.wasm
