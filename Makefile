.PHONY: all web-build cargo-build clean dev

# Build everything: frontend + Rust binary
all: web-build cargo-build

# Build the Svelte frontend
web-build:
	@echo "[*] Building Svelte frontend..."
	cd web && npm install && npm run build
	@echo "[+] Frontend built to web/dist/"

# Build the Rust binary (embeds web/dist/)
cargo-build:
	@echo "[*] Building Rust binary..."
	cargo build --release
	@echo "[+] Binary at target/release/dalang"

# Clean all build artifacts
clean:
	rm -rf web/dist web/node_modules target

# Dev mode: run Vite dev server + Rust backend concurrently
dev:
	@echo "[*] Starting dev mode..."
	@echo "    Run 'cd web && npm run dev' in one terminal"
	@echo "    Run 'cargo run -- web --port 8080' in another"
