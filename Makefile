wasm-build: ## ⚙️  Build wasm version
	RUST_LOG=off cargo build --release --target wasm32-unknown-unknown
	wasm-bindgen --out-dir ./www/public/out --target web ./target/wasm32-unknown-unknown/release/rusty-apple.wasm

wasm-opt: ## 🔩 Optimize wasm file size
	wasm-opt -Os -o ./www/public/out/rusty-apple_bg.wasm ./www/public/out/rusty-apple_bg.wasm

wasm-build-opt: ## ⚙️  Build wasm version with optimized file size
	$(MAKE) wasm-build
	$(MAKE) wasm-opt

copy-assets: ## 📂 Copy assets to the output directory
	cp -r ./assets ./www/public/out/

## Run cargo with info level logging
dev: ## 🚀 Run dev server
	RUST_LOG=info cargo run