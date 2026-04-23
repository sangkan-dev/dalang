# --- Stage 1: Frontend Build ---
FROM node:24-slim AS frontend-builder
WORKDIR /app/web2
COPY web2/package*.json ./
RUN npm install
COPY web2/ ./
RUN npm run build:dashboard

# --- Stage 2: Backend Build ---
FROM rust:1.94-slim-bookworm AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY skills/ ./skills/
COPY --from=frontend-builder /app/web2/build-dashboard ./web2/build-dashboard
RUN cargo build --release -p dalang-cli

# --- Stage 3: Final Runtime ---
FROM debian:bookworm-slim
WORKDIR /app

# Enable non-free repository (required for nikto, etc.)
RUN echo "deb http://deb.debian.org/debian bookworm main contrib non-free non-free-firmware" > /etc/apt/sources.list && \
    echo "deb http://deb.debian.org/debian bookworm-updates main contrib non-free non-free-firmware" >> /etc/apt/sources.list && \
    echo "deb http://security.debian.org/debian-security bookworm-security main contrib non-free non-free-firmware" >> /etc/apt/sources.list

# Install system dependencies and apt-available security tools
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    unzip \
    python3 \
    python3-pip \
    python3-venv \
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
    dirb \
    chromium \
    libssl3 \
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

# XSStrike - XSS scanner (Python, isolated venv to avoid PEP 668 conflict)
RUN git clone --depth 1 https://github.com/s0md3v/XSStrike.git /opt/xsstrike && \
    python3 -m venv /opt/xsstrike/venv && \
    /opt/xsstrike/venv/bin/pip install --no-cache-dir -r /opt/xsstrike/requirements.txt && \
    printf '#!/bin/sh\nexec /opt/xsstrike/venv/bin/python /opt/xsstrike/xsstrike.py "$@"\n' > /usr/local/bin/xsstrike && \
    chmod +x /usr/local/bin/xsstrike

# ── Port Scanners ──
# rustscan - ultra-fast port scanner
RUN wget -q https://github.com/bee-san/RustScan/releases/download/2.4.1/rustscan.deb.zip && \
    unzip rustscan.deb.zip && \
    dpkg -i rustscan_2.4.1-1_amd64.deb && rm rustscan_2.4.1-1_amd64.deb

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
RUN pip3 install --no-cache-dir --break-system-packages uv && \
    git clone --depth 1 https://github.com/laramies/theHarvester.git /opt/theHarvester && \
    cd /opt/theHarvester && uv sync && \
    printf '#!/bin/sh\nexec /opt/theHarvester/.venv/bin/python /opt/theHarvester/theHarvester.py "$@"\n' > /usr/local/bin/theHarvester && \
    chmod +x /usr/local/bin/theHarvester

# arjun - hidden HTTP parameter discovery (isolated venv)
RUN python3 -m venv /opt/arjun/venv && \
    /opt/arjun/venv/bin/pip install --no-cache-dir arjun && \
    printf '#!/bin/sh\nexec /opt/arjun/venv/bin/arjun "$@"\n' > /usr/local/bin/arjun && \
    chmod +x /usr/local/bin/arjun

# netexec (nxc) - CrackMapExec successor for network/AD enumeration
# PIPX_BIN_DIR=/usr/local/bin ensures binaries land directly in PATH during build
# (pipx ensurepath only modifies .bashrc which is not sourced during docker build)
RUN pip3 install --no-cache-dir --break-system-packages pipx && \
    PIPX_HOME=/opt/pipx PIPX_BIN_DIR=/usr/local/bin \
    pipx install git+https://github.com/Pennyw0rth/NetExec

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

# ── Misc Tools ──
# Add common wordlists for directory brute-forcing (used by dalang and others)
RUN mkdir -p /usr/share/wordlists/dirb
RUN cp /usr/share/dirb/wordlists/common.txt /usr/share/wordlists/dirb/common.txt

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