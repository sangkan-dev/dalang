# Docker Deployment

This guide covers running Dalang with Docker Compose for local operations and repeatable team setups.

## Quick Start

```bash
git clone https://github.com/sangkan-dev/dalang.git
cd dalang

cat > .env << 'EOF'
LLM_PROVIDER=openai
LLM_API_KEY=your-api-key-here
# Optional:
# LLM_BASE_URL=https://your-endpoint.example.com/v1
EOF

docker compose up --build
```

Dalang web UI is available at `http://localhost:4000`.

## Service Overview

The default `docker-compose.yml` runs one service:

- `dalang` container built from the repository `Dockerfile`
- startup command: `dalang web --port 4000`
- host networking enabled (`network_mode: host`)
- persistent data volume: `dalang_data:/root/.dalang`

## Environment Variables

Set values in `.env` or your shell before startup:

- `LLM_PROVIDER` (default: `openai`)
- `LLM_API_KEY` (required)
- `LLM_BASE_URL` (optional, for OpenAI-compatible endpoints)

Example:

```bash
LLM_PROVIDER=openai
LLM_API_KEY=sk-...
LLM_BASE_URL=
```

## Lifecycle Commands

```bash
# Build and start in foreground
docker compose up --build

# Start in background
docker compose up -d --build

# Stream logs
docker compose logs -f dalang

# Stop services
docker compose down

# Stop and remove named volumes
docker compose down -v
```

## Persistence

Dalang stores sessions, memory snapshots, and runtime artifacts under `/root/.dalang` in the container.
With the default compose file this path is backed by a named volume (`dalang_data`) so state survives
container restarts.

## Networking Notes

The default configuration uses `network_mode: host` for maximum compatibility with local/LAN scans.

Tradeoffs:

- Pros: simpler reachability to host-local services and private network targets
- Cons: no port publishing isolation; service binds directly to host namespace

If your environment requires stricter isolation, switch to bridge networking and publish only required
ports (for example `4000:4000`).

## Tooling Included in the Image

The Docker image includes Dalang plus a broad built-in tooling stack (for example `nmap`, `sqlmap`,
`nikto`, `hydra`, `masscan`, `rustscan`, `subfinder`, `httpx`, `nuclei`, `ffuf`, and more).

This avoids manual per-host tool installation for most common workflows.

## Customization

Common extension points:

- Add or pin extra tools in `Dockerfile`
- Override startup command in `docker-compose.yml`
- Mount additional host paths if your workflow requires custom wordlists or artifacts

## Troubleshooting

### Container starts but UI is unreachable

- Check logs: `docker compose logs -f dalang`
- Ensure nothing else is already using port `4000`
- Verify your firewall allows local access to the port

### LLM requests fail

- Confirm `.env` has correct `LLM_PROVIDER` and `LLM_API_KEY`
- If using custom endpoints, validate `LLM_BASE_URL`
- Restart after config changes: `docker compose up -d --build`

### Sessions disappear after restart

- Verify `dalang_data` volume exists: `docker volume ls | grep dalang_data`
- Confirm compose service still mounts `dalang_data:/root/.dalang`

### Missing tool error during execution

- Check tool presence inside container: `docker compose exec dalang which nmap`
- Rebuild image if Dockerfile changed: `docker compose up --build`
