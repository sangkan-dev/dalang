# Built-in Skills

Dalang ships with **22 built-in security skills** covering network, web, cloud, and container security.

## Network Reconnaissance

| Skill            | Tool     | Description                                              |
| ---------------- | -------- | -------------------------------------------------------- |
| `nmap_scanner`   | nmap     | Port scanning with service version detection             |
| `masscan_fast`   | masscan  | Extremely fast full-port scanning (requires root)        |
| `rustscan_audit` | rustscan | Modern fast port scanning with nmap service handoff      |

## Web Application Security

| Skill             | Tool          | Description                                                   |
| ----------------- | ------------- | ------------------------------------------------------------- |
| `web-audit`       | Browser (CDP) | Client-side DOM analysis and vulnerability detection          |
| `ffuf_fuzzer`     | ffuf          | Directory and file fuzzing for hidden endpoints               |
| `gobuster_dir`    | gobuster      | Directory and file brute-forcing for content discovery        |
| `sqlmap_tester`   | sqlmap        | Automated SQL injection detection and validation              |
| `xss_strike`      | XSStrike      | Advanced cross-site scripting detection and fuzzing           |
| `nikto_scanner`   | nikto         | Comprehensive web server vulnerability scanner                |
| `header_analyzer` | curl          | HTTP security header analysis (HSTS, CSP, X-Frame-Options)   |
| `ssl_scan`        | sslscan       | TLS/SSL configuration audit (weak ciphers, protocol versions) |
| `jwt_analysis`    | Browser (JS)  | JWT token extraction and security analysis                    |

## CMS & Framework Specific

| Skill          | Tool   | Description                  |
| -------------- | ------ | ---------------------------- |
| `wpscan_audit` | wpscan | WordPress vulnerability scanning |

## Credential Testing

| Skill              | Tool  | Description                                          |
| ------------------ | ----- | ---------------------------------------------------- |
| `hydra_bruteforce` | hydra | Credential strength testing for network protocols    |

## Cloud & Infrastructure

| Skill                 | Tool    | Description                                        |
| --------------------- | ------- | -------------------------------------------------- |
| `kubectl_audit`       | kubectl | Kubernetes cluster permission and security review  |
| `aws_cli_enum`        | aws-cli | AWS resource enumeration (S3, IAM)                 |
| `docker_escape_check` | capsh   | Container escape vulnerability and capability check|

## Discovery & Enumeration

| Skill              | Tool      | Description                                      |
| ------------------ | --------- | ------------------------------------------------ |
| `nuclei_vuln_scan` | nuclei    | Template-based CVE and misconfiguration scanning |
| `subdomain_enum`   | subfinder | Passive subdomain enumeration for attack surface |
| `smbclient_enum`   | smbclient | SMB share and null session enumeration           |
| `snmpwalk_gather`  | snmpwalk  | SNMP misconfiguration information gathering      |

## Utility

| Skill           | Tool | Description                               |
| --------------- | ---- | ----------------------------------------- |
| `testing-skill` | —    | Dummy skill for development testing       |
