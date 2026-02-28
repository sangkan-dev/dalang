# Auto-Pilot Mode

Auto-Pilot mode gives the AI full control over the penetration testing workflow. The AI acts as a **Meta-Orchestrator** that decides which tools to use and when.

## Usage

```bash
# Default: 15 iterations, 300s command timeout
dalang scan --target <TARGET> --auto

# Custom iteration limit
dalang scan --target <TARGET> --auto --max-iter 50

# Unlimited iterations (runs until AI produces final report)
dalang scan --target <TARGET> --auto --max-iter 0

# Unlimited command timeout (tools run until they finish)
dalang scan --target <TARGET> --auto --cmd-timeout 0

# Full unlimited mode
dalang scan --target <TARGET> --auto --max-iter 0 --cmd-timeout 0
```

### Parameters

| Parameter           | Short | Default | Description                                          |
| ------------------- | ----- | ------- | ---------------------------------------------------- |
| `--max-iter`        | `-n`  | 15      | Maximum iterations (0 = unlimited)                   |
| `--cmd-timeout`     | —     | 300     | Command execution timeout in seconds (0 = unlimited) |

## How It Works

1. **All skills** are loaded into a catalog and presented to the LLM
2. The AI receives a **Meta-Orchestrator** persona prompt
3. The AI plans and executes a multi-stage assessment:
   - Stage 1: Reconnaissance (port scanning, service discovery)
   - Stage 2: Enumeration (web fuzzing, directory brute-forcing)
   - Stage 3: Vulnerability Assessment (injection testing, misconfig checks)
   - Stage 4: Reporting
4. After gathering enough data (or reaching the iteration limit), the AI generates a **VULNERABILITY REPORT**
5. The report follows a structured bug-bounty format with:
   - Exact affected URLs and parameters
   - CWE classifications and CVSS scores
   - Step-by-step Proof of Concept (PoC) with payloads and curl commands
   - Raw tool output as evidence
   - Impact analysis and remediation recommendations
6. The report is automatically saved to `dalang_report_<target>_<timestamp>.md`

## Persistent Memory

During auto-pilot, Dalang maintains a **Context Memory** that tracks the last 20 observations. This allows the AI to:

- Reference previous scan results
- Avoid redundant tool calls
- Build upon earlier findings

## Dynamic Argument Injection

The AI can inject custom arguments into skill executions. For example, if the base `nmap_scanner` skill scans with `-sV`, the AI might add `--script vuln` based on initial findings.

All injected arguments are **sanitized** against shell metacharacters before execution.

## Example Output

```
[*] Initializing Autonomous Auto-Pilot Mode...
[*] Loaded 22 skills into catalog.

[...] Strategic Reasoning (Iteration 1/15)...
[>] Orchestrator decided to use skill: nmap_scanner
    $ nmap -sV -T4 192.168.1.1
[<] Observation received (2340 bytes)

[...] Strategic Reasoning (Iteration 2/15)...
[>] Orchestrator decided to use skill: ffuf_fuzzer
    [+] AI injected 2 custom arguments
    $ ffuf -w /usr/share/wordlists/common.txt -u http://192.168.1.1/FUZZ
[<] Observation received (5120 bytes)

[...] Strategic Reasoning (Iteration 3/15)...
[✓] Final Vulnerability Report Generated!
[+] Report saved to: dalang_report_192_168_1_1_20250227_103045.md
```
