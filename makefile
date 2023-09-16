public/wasm/renderer.js: $(shell find renderer)
	wasm-pack build --target web --out-dir ../public/wasm renderer

target/debug/build/renderer: $(shell find renderer)
	cargo build --lib renderer
	cargo build --bin renderer

target/debug/build/server: $(shell find server)
	cargo build --bin server

watch-renderer-wasm: 
	cargo watch -x 'wasm-pack build --target web --out-dir ../public/wasm renderer'

watch-renderer:
	cargo watch -x 'run --bin renderer'

watch-server:
	cargo watch -x 'run --bin server'

watch:
	parallel -j 2 -- \
	"wasm-pack build --target web --out-dir ../public/wasm renderer" \
	"cargo watch -x 'run --bin server'"

build: target/debug/build/server public/wasm/renderer.js

run: build
	cargo run --bin server