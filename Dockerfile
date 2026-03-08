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
# The build will include embedded frontend assets if the code is configured for it
COPY --from=frontend-builder /app/web/dist ./web/dist
RUN touch src/main.rs && cargo build --release

# --- Stage 3: Final Runtime ---
FROM debian:bullseye-slim
WORKDIR /app

# Enable non-free repository (required for nikto, etc.)
RUN echo "deb http://deb.debian.org/debian bullseye main contrib non-free" > /etc/apt/sources.list && \
    echo "deb http://deb.debian.org/debian bullseye-updates main contrib non-free" >> /etc/apt/sources.list && \
    echo "deb http://security.debian.org/debian-security bullseye-security main contrib non-free" >> /etc/apt/sources.list

# Install system dependencies and security tools available in apt
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    git \
    unzip \
    python3 \
    python3-pip \
    ruby \
    ruby-dev \
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
    && rm -rf /var/lib/apt/lists/*

# Install Go-based tools (Nuclei, Gobuster, FFUF) from binaries
RUN wget -q https://github.com/projectdiscovery/nuclei/releases/download/v3.3.0/nuclei_3.3.0_linux_amd64.zip && \
    unzip nuclei_3.3.0_linux_amd64.zip nuclei && \
    mv nuclei /usr/local/bin/ && \
    rm nuclei_3.3.0_linux_amd64.zip

RUN wget -q https://github.com/OJ/gobuster/releases/download/v3.6.0/gobuster_Linux_x86_64.tar.gz && \
    tar -xzf gobuster_Linux_x86_64.tar.gz gobuster && \
    mv gobuster /usr/local/bin/ && \
    rm gobuster_Linux_x86_64.tar.gz

RUN wget -q https://github.com/ffuf/ffuf/releases/download/v2.1.0/ffuf_2.1.0_linux_amd64.tar.gz && \
    tar -xzf ffuf_2.1.0_linux_amd64.tar.gz ffuf && \
    mv ffuf /usr/local/bin/ && \
    rm ffuf_2.1.0_linux_amd64.tar.gz

# Install Subfinder (subdomain enumeration) - binary from GitHub
RUN wget -q https://github.com/projectdiscovery/subfinder/releases/download/v2.6.6/subfinder_2.6.6_linux_amd64.zip && \
    unzip subfinder_2.6.6_linux_amd64.zip subfinder && \
    mv subfinder /usr/local/bin/ && \
    rm subfinder_2.6.6_linux_amd64.zip

# Install RustScan via .deb package from GitHub
RUN wget -q https://github.com/RustScan/RustScan/releases/download/2.2.3/rustscan_2.2.3_amd64.deb && \
    dpkg -i rustscan_2.2.3_amd64.deb && \
    rm rustscan_2.2.3_amd64.deb

# Install XSStrike (XSS scanner - Python tool)
RUN git clone --depth 1 https://github.com/s0md3v/XSStrike.git /opt/xsstrike && \
    pip3 install --no-cache-dir -r /opt/xsstrike/requirements.txt && \
    printf '#!/bin/sh\nexec python3 /opt/xsstrike/xsstrike.py "$@"\n' > /usr/local/bin/xsstrike && \
    chmod +x /usr/local/bin/xsstrike

# Install WPScan via RubyGems
RUN gem install wpscan

# Install Kubectl
RUN curl -fsSL -o /usr/local/bin/kubectl \
    "https://dl.k8s.io/release/$(curl -fsSL https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl" && \
    chmod +x /usr/local/bin/kubectl

# Install AWS CLI
RUN curl -fsSL "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip" && \
    unzip awscliv2.zip && \
    ./aws/install && \
    rm -rf aws awscliv2.zip

# Environment variables for Dalang
ENV CHROME_PATH=/usr/bin/chromium
ENV DALANG_DOCKER=true

# Copy binary from builder
COPY --from=backend-builder /app/target/release/dalang /usr/local/bin/dalang

# Prepare data directory
RUN mkdir -p /root/.dalang

# Copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

EXPOSE 4000
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["dalang", "web", "--port", "4000"]