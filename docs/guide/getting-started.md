# Introduction

## What is Dalang?

**Dalang** (Indonesian for "puppet master") is a modern, extensible framework written in Rust that transforms Large Language Models into autonomous, context-aware cybersecurity orchestrators.

Instead of relying on rigid, pre-programmed scripts, Dalang gives AI the ability to:

- 🔍 **Interpret** targets and plan attack strategies
- 🛠️ **Select** appropriate security tools from a modular skill library
- ⚡ **Execute** them safely on the local operating system
- 🔗 **Chain** observations together to discover vulnerabilities
- 📝 **Report** findings in structured vulnerability reports

## Philosophy

The name "Dalang" perfectly captures the framework's core concept:

> Just like a traditional **Wayang Dalang** (shadow puppet master) orchestrates puppets to tell a story, the Dalang engine orchestrates AI agents and local tools to conduct comprehensive security assessments.

The AI acts as the storyteller, and Dalang provides the stage, the puppets (tools), and the strings (the ReAct orchestration loop).

## Key Features

| Feature                        | Description                                                 |
| ------------------------------ | ----------------------------------------------------------- |
| **Autonomous Auto-Pilot**      | AI orchestrates end-to-end pentesting with `--auto` mode    |
| **Universal Tool Integration** | Add any CLI tool via Markdown skill definition files        |
| **Defensive Prompting**        | Bypass AI safety filters with "Authorized Auditor" personas |
| **CDP Browser**                | Interact with SPAs via Chrome DevTools Protocol             |
| **Multi-Provider LLM**         | Supports Gemini, OpenAI, Anthropic, and local models        |
| **Secure Execution**           | OS command wrapper prevents shell injection attacks         |
| **OAuth & Keyring**            | Persistent, secure credential storage                       |

## Who is this for?

Dalang is designed for:

- **Penetration Testers** who want AI-augmented workflows
- **Security Researchers** exploring autonomous vulnerability discovery
- **DevSecOps Engineers** building automated security pipelines
- **Red Teams** that need extensible, scriptable pentesting frameworks
