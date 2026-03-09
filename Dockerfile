# --- Stage 1: Frontend Build ---
FROM node:24-slim AS frontend-builder
WORKDIR /app/web
COPY web/package*.json ./
RUN npm install
COPY web/ ./
RUN npm run build

# --- Stage 2: Backend Build ---
FROM rust:1.94-slim-bullseye AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
# Pre-build dependencies for caching
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY src/ ./src/
COPY skills/ ./skills/
COPY --from=frontend-builder /app/web/dist ./web/dist
RUN touch src/main.rs && cargo build --release

# --- Stage 3: Final Runtime ---
FROM debian:bullseye-slim
WORKDIR /app

# Enable non-free repository (required for nikto, etc.)
RUN echo "deb http://deb.debian.org/debian bullseye main contrib non-free" > /etc/apt/sources.list && \
    echo "deb http://deb.debian.org/debian bullseye-updates main contrib non-free" >> /etc/apt/sources.list && \
    echo "deb http://security.debian.org/debian-security bullseye-security main contrib non-free" >> /etc/apt/sources.list

# Install system dependencies and apt-available security tools
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    unzip \
    python3 \
    python3-pip \
    ruby \
    ruby-dev \
    build-essential \
    libpcap-dev \
    libssl-dev \
    libffi-dev \
    nmap \
    sqlmap \
    nikto \
    hydra \
    smbclient \
    snmp \
    chromium \
    libssl1.1 \
    ca-certificates \
    masscan \
    sslscan \
    dnsutils \
    && rm -rf /var/lib/apt/lists/*

# ── ProjectDiscovery suite (Nuclei, Subfinder, httpx, dnsx, naabu, katana) ──
RUN wget -q https://github.com/projectdiscovery/nuclei/releases/download/v3.7.1/nuclei_3.7.1_linux_amd64.zip && \
    unzip nuclei_3.7.1_linux_amd64.zip nuclei && mv nuclei /usr/local/bin/ && \
    rm nuclei_3.7.1_linux_amd64.zip

RUN wget -q https://github.com/projectdiscovery/subfinder/releases/download/v2.12.0/subfinder_2.12.0_linux_amd64.zip && \
    unzip subfinder_2.12.0_linux_amd64.zip subfinder && mv subfinder /usr/local/bin/ && \
    rm subfinder_2.12.0_linux_amd64.zip

RUN wget -q https://github.com/projectdiscovery/httpx/releases/download/v1.8.1/httpx_1.8.1_linux_amd64.zip && \
    unzip httpx_1.8.1_linux_amd64.zip httpx && mv httpx /usr/local/bin/ && \
    rm httpx_1.8.1_linux_amd64.zip

RUN wget -q https://github.com/projectdiscovery/dnsx/releases/download/v1.2.3/dnsx_1.2.3_linux_amd64.zip && \
    unzip dnsx_1.2.3_linux_amd64.zip dnsx && mv dnsx /usr/local/bin/ && \
    rm dnsx_1.2.3_linux_amd64.zip

RUN wget -q https://github.com/projectdiscovery/naabu/releases/download/v2.4.0/naabu_2.4.0_linux_amd64.zip && \
    unzip naabu_2.4.0_linux_amd64.zip naabu && mv naabu /usr/local/bin/ && \
    rm naabu_2.4.0_linux_amd64.zip

RUN wget -q https://github.com/projectdiscovery/katana/releases/download/v1.4.0/katana_1.4.0_linux_amd64.zip && \
    unzip katana_1.4.0_linux_amd64.zip katana && mv katana /usr/local/bin/ && \
    rm katana_1.4.0_linux_amd64.zip

# ── Directory / Content Discovery ──
RUN wget -q https://github.com/OJ/gobuster/releases/download/v3.8.2/gobuster_Linux_x86_64.tar.gz && \
    tar -xzf gobuster_Linux_x86_64.tar.gz gobuster && mv gobuster /usr/local/bin/ && \
    rm gobuster_Linux_x86_64.tar.gz

RUN wget -q https://github.com/ffuf/ffuf/releases/download/v2.1.0/ffuf_2.1.0_linux_amd64.tar.gz && \
    tar -xzf ffuf_2.1.0_linux_amd64.tar.gz ffuf && mv ffuf /usr/local/bin/ && \
    rm ffuf_2.1.0_linux_amd64.tar.gz

# feroxbuster - fast recursive content discovery
RUN wget -q https://github.com/epi052/feroxbuster/releases/latest/download/feroxbuster_amd64.deb.zip && \
    unzip feroxbuster_amd64.deb.zip && \
    dpkg -i feroxbuster_*_amd64.deb && \
    rm -f feroxbuster_amd64.deb.zip feroxbuster_*_amd64.deb

# ── XSS Scanners ──
# dalfox - modern XSS scanner
RUN wget -q https://github.com/hahwul/dalfox/releases/download/v2.12.0/dalfox-linux-amd64.tar.gz && \
    tar -xzf dalfox-linux-amd64.tar.gz dalfox-linux-amd64 && mv dalfox-linux-amd64 /usr/local/bin/ && \
    rm dalfox-linux-amd64.tar.gz

# XSStrike - XSS scanner (Python)
RUN git clone --depth 1 https://github.com/s0md3v/XSStrike.git /opt/xsstrike && \
    pip3 install --no-cache-dir -r /opt/xsstrike/requirements.txt && \
    printf '#!/bin/sh\nexec python3 /opt/xsstrike/xsstrike.py "$@"\n' > /usr/local/bin/xsstrike && \
    chmod +x /usr/local/bin/xsstrike

# ── Port Scanners ──
# rustscan - ultra-fast port scanner
RUN wget -q https://github.com/RustScan/RustScan/releases/download/2.4.1/rustscan_2.4.1_amd64.deb && \
    dpkg -i rustscan_2.4.1_amd64.deb && rm rustscan_2.4.1_amd64.deb

# ── OSINT / Recon ──
# OWASP Amass - attack surface mapping
RUN wget -q https://github.com/owasp-amass/amass/releases/download/v5.0.1/amass_linux_amd64.tar.gz && \
    tar -xzf amass_linux_amd64.tar.gz amass_linux_amd64 && mv amass_linux_amd64/amass /usr/local/bin/ && \
    rm -rf amass_linux_amd64 amass_linux_amd64.tar.gz

# trufflehog - secrets scanner
RUN wget -q https://github.com/trufflesecurity/trufflehog/releases/download/v3.93.7/trufflehog_3.93.7_linux_amd64.tar.gz && \
    tar -xzf trufflehog_3.93.7_linux_amd64.tar.gz trufflehog && mv trufflehog /usr/local/bin/ && \
    rm trufflehog_3.93.7_linux_amd64.tar.gz

# theHarvester - OSINT tool (uses uv as package manager, no requirements.txt)
RUN pip3 install --no-cache-dir uv && \
    git clone --depth 1 https://github.com/laramies/theHarvester.git /opt/theHarvester && \
    cd /opt/theHarvester && uv sync && \
    printf '#!/bin/sh\nexec /opt/theHarvester/.venv/bin/python /opt/theHarvester/theHarvester.py "$@"\n' > /usr/local/bin/theHarvester && \
    chmod +x /usr/local/bin/theHarvester

# arjun - hidden HTTP parameter discovery
RUN pip3 install --no-cache-dir arjun

# netexec (nxc) - CrackMapExec successor for network enumeration (not in Debian apt)
RUN pip3 install --no-cache-dir pipx && \
    pipx install git+https://github.com/Pennyw0rth/NetExec --pip-args="--no-cache-dir" && \
    ln -s /root/.local/bin/nxc /usr/local/bin/nxc && \
    ln -s /root/.local/bin/netexec /usr/local/bin/netexec || true

# ── Cloud Security ──
# trivy - container/IaC/cloud scanner
RUN curl -sfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh -s -- -b /usr/local/bin

# ── WordPress ──
RUN gem install wpscan

# ── Cloud CLI tools ──
RUN curl -fsSL -o /usr/local/bin/kubectl \
    "https://dl.k8s.io/release/$(curl -fsSL https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" && \
    chmod +x /usr/local/bin/kubectl

RUN curl -fsSL "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip" && \
    unzip awscliv2.zip && ./aws/install && rm -rf aws awscliv2.zip

# ── Environment ──
ENV CHROME_PATH=/usr/bin/chromium
ENV DALANG_DOCKER=true
# Ensure pipx-installed binaries are in PATH
ENV PATH="/root/.local/bin:${PATH}"

COPY --from=backend-builder /app/target/release/dalang /usr/local/bin/dalang

RUN mkdir -p /root/.dalang

COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

EXPOSE 4000
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["dalang", "web", "--port", "4000"]