public/wasm/renderer.js: renderer/src/*.rs
	wasm-pack build --target web --out-dir ../public/wasm renderer

renderer: target/debug/build/renderer target/debug/build/renderer.rlib
	cargo build --lib renderer
	cargo build --bin renderer

server: target/debug/build/server
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