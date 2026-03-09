# Installation

## Prerequisites

- **Rust** (1.75+) and **Cargo** — [Install Rust](https://rustup.rs/)
- **Chromium** or **Google Chrome** — required for CDP browser features
- **Security Tools** — tools referenced by skills (e.g., `nmap`, `ffuf`, `sqlmap`)

## Installing with Docker Compose

This is the fastest path to a full Dalang runtime. The provided image bundles the Dalang binary,
web UI assets, and common offensive security tooling so you can start with a single command.

### Prerequisites

- Docker Engine 24+
- Docker Compose v2 (`docker compose` command)

### 1. Clone the repository

```bash
git clone https://github.com/sangkan-dev/dalang.git
cd dalang
```

### 2. Configure environment variables

Create a `.env` file beside `docker-compose.yml`:

```bash
LLM_PROVIDER=openai
LLM_API_KEY=your-api-key-here
# Optional for custom OpenAI-compatible endpoint
# LLM_BASE_URL=https://your-endpoint.example.com/v1
```

### 3. Build and start

```bash
docker compose up --build
```

The service starts Dalang web mode on `http://localhost:4000`.

### 4. Persistence and startup behavior

- Session data and runtime state are persisted through the `dalang_data` volume.
- Container startup runs `dalang init` automatically via `docker-entrypoint.sh`, so built-in
	skills are initialized before the web server starts.

::: tip
Use `docker compose up -d` for detached mode and `docker compose logs -f dalang` for live logs.
:::

## Building from Source

```bash
# Clone the repository
git clone https://github.com/sangkan-dev/dalang.git
cd dalang

# Build in release mode
cargo build --release

# The binary will be at:
./target/release/dalang
```

## Initializing the Environment

After building, initialize the skill library:

```bash
dalang init
```

This creates the `skills/` directory and installs all 22 built-in skill files. Skills that already exist are skipped, so running `dalang init` again after an update will only install new skills.

## Installing Security Tools

Dalang is a framework that orchestrates external tools. Install the tools you need:

::: code-group

```bash [Ubuntu/Debian]
sudo apt install nmap ffuf sqlmap wpscan masscan nikto sslscan hydra smbclient snmp gobuster
pip install xsstrike
# For nuclei: https://github.com/projectdiscovery/nuclei#install
# For subfinder: https://github.com/projectdiscovery/subfinder#install
# For rustscan: https://github.com/RustScan/RustScan#install
```

```bash [macOS (Homebrew)]
brew install nmap ffuf sqlmap wpscan masscan nikto sslscan hydra smbclient gobuster
brew install projectdiscovery/tap/nuclei projectdiscovery/tap/subfinder
# For rustscan: cargo install rustscan
```

```bash [Arch Linux]
sudo pacman -S nmap
yay -S ffuf sqlmap wpscan masscan
```

:::

::: tip
You don't need **all** tools installed — only the ones referenced by the skills you plan to use. Dalang gracefully handles missing tools with clear error messages.
:::

## Verifying Installation

```bash
# Check Dalang is working
dalang --help

# Check a specific tool
dalang scan --target 127.0.0.1 --skills nmap_scanner
```
