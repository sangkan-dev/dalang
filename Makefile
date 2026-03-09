.PHONY: all web-build cargo-build clean dev

# Build everything: frontend + Rust binary
all: web-build cargo-build

# Build the SvelteKit frontend
web-build:
	@echo "[*] Building SvelteKit frontend..."
	cd web2 && npm install && npm run build
	@echo "[+] Frontend built to web2/build/"

# Build the Rust binary (embeds web2/build/)
cargo-build:
	@echo "[*] Building Rust binary..."
	cargo build --release
	@echo "[+] Binary at target/release/dalang"

# Clean all build artifacts
clean:
	rm -rf web2/build web2/node_modules target

# Dev mode: run Vite dev server + Rust backend concurrently
dev:
	@echo "[*] Starting dev mode..."
	@echo "    Run 'cd web2 && npm run dev' in one terminal"
	@echo "    Run 'cargo run -- web --port 8080' in another"
