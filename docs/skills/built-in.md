# Built-in Skills

Dalang ships with a comprehensive library of security skills covering network, web, cloud, and container security.

## Network Reconnaissance

| Skill            | Tool     | Description                                 |
| ---------------- | -------- | ------------------------------------------- |
| `nmap_scanner`   | nmap     | Port scanning and service version detection |
| `masscan_fast`   | masscan  | High-speed port scanning for large networks |
| `rustscan_audit` | rustscan | Fast Rust-based port scanner                |

## Web Application Security

| Skill             | Tool          | Description                                          |
| ----------------- | ------------- | ---------------------------------------------------- |
| `web-audit`       | Browser (CDP) | DOM analysis and client-side vulnerability detection |
| `ffuf_fuzzer`     | ffuf          | Directory and parameter fuzzing                      |
| `sqlmap_tester`   | sqlmap        | SQL injection validation                             |
| `xss_strike`      | XSStrike      | Cross-site scripting detection                       |
| `header_analyzer` | Browser       | HTTP security header analysis                        |
| `ssl_scan`        | sslscan       | SSL/TLS configuration audit                          |

## CMS & Framework Specific

| Skill                | Tool         | Description                       |
| -------------------- | ------------ | --------------------------------- |
| `wpscan_enumeration` | wpscan       | WordPress vulnerability scanning  |
| `jwt_analysis`       | Browser (JS) | JWT token analysis and validation |

## Cloud & Infrastructure

| Skill                 | Tool    | Description                                      |
| --------------------- | ------- | ------------------------------------------------ |
| `kubectl_audit`       | kubectl | Kubernetes cluster security assessment           |
| `aws_cli_enum`        | aws-cli | AWS resource enumeration and misconfig detection |
| `docker_escape_check` | docker  | Container escape vulnerability checks            |

## Utility

| Skill              | Tool      | Description                           |
| ------------------ | --------- | ------------------------------------- |
| `nuclei_vuln_scan` | nuclei    | Template-based vulnerability scanning |
| `subdomain_enum`   | subfinder | Subdomain enumeration and discovery   |
