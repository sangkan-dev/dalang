# Installation

## Prerequisites

- **Rust** (1.75+) and **Cargo** — [Install Rust](https://rustup.rs/)
- **Chromium** or **Google Chrome** — required for CDP browser features
- **Security Tools** — tools referenced by skills (e.g., `nmap`, `ffuf`, `sqlmap`)

## Building from Source

```bash
# Clone the repository
git clone https://github.com/HasanH47/dalang.git
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

This creates the `skills/` directory with an example skill file (`example-nmap.md`).

## Installing Security Tools

Dalang is a framework that orchestrates external tools. Install the tools you need:

::: code-group

```bash [Ubuntu/Debian]
sudo apt install nmap ffuf sqlmap wpscan masscan
```

```bash [macOS (Homebrew)]
brew install nmap ffuf sqlmap wpscan masscan
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
