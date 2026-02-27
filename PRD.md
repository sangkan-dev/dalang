# Product Requirements Document (PRD): Dalang

## Executive Summary & Objectives

**Dalang** adalah framework cybersecurity AI Agent open-source yang beroperasi sebagai scanner kerentanan (vulnerability scanner) untuk aplikasi web dan jaringan. Mengadaptasi filosofi "Dalang" dalam pewayangan yang mengendalikan banyak karakter (wayang), framework ini berperan sebagai orchestrator yang mengontrol berbagai "Skills" (modul eksploitasi dan enumerasi) berbasis arsitektur ReAct (Reasoning and Acting).

Tujuan utama dari Dalang adalah memberikan layer automasi cerdas dalam penetration testing dan security auditing, di mana autonomous agent dapat memahami konteks keamanan, merencanakan path serangan/skenario audit, dan mengeksekusi scanning secara mandiri, presisi, dan aman.

## Core Features

1. **Markdown Skill Engine**
   - Mendefinisikan aksi atau metodologi penetration testing dalam bentuk file Markdown (`.md`).
   - Setiap file `.md` akan memiliki `Frontmatter` untuk metadata/konfigurasi operasi, System Prompt untuk memberikan instruksi kepada AI mengenai skill terkait, dan template aturan eksekusi command line OS.
2. **AI Tool Calling Bridge via JSON**
   - Menyediakan interface komunikasi standar berbasis JSON antara core engine (Rust) dan model AI (LLM).
   - Memastikan aksi spesifik yang diputuskan oleh AI lewat metode Tool Calling (function calling) dapat dipetakan dan diparsing dengan aman menjadi argumen sistem atau permintaan HTTP jaringan.
3. **CDP Web Crawler**
   - Integrasi headless browser secara fungsional menggunakan Chrome DevTools Protocol (CDP).
   - Memungkinkan crawler untuk melakukan bypass terhadap client-side rendering (SPA), menangkap traffic API/jaringan lewat interceptor, menyusun peta struktur DOM, dan mendeteksi injeksi kompleks (seperti DOM-based XSS) secara realistis seolah-olah dieksekusi oleh user asli.
4. **LLM Agnostic API**
   - Dirancang arsitektur modular yang mendukung berbagai provider LLM (seperti OpenAI (API & Oauth), Anthropic (API & Oauth), Gemini (API & Oauth), maupun open-source model lokal via Ollama/vLLM).
   - Menyediakan layer abstraksi komunikasi yang memungkinkan pengguna menukar otak AI backend tanpa harus mengubah core engine Dalang.

## Architecture & Tech Stack

- **Core Engine:** **Rust**
  - Dipilih karena memiliki performa pemrosesan paralel (multithreading) yang tinggi dan garansi memory safety yang ketat. Ini esensial untuk mencegah engine hacking/buffer overflows secara lokal dan mempercepat proses asynchronous saat scanning berjalan banyak sekaligus.
- **Web Interaction:** Library integrasi CDP berbasis Rust (misalnya `chromiumoxide` atau `headless_chrome`).
- **Skill Parser:** Parser Markdown & YAML/TOML (seperti `pulldown-cmark` atau penanganan regex, serta `serde`) untuk memecah dan memvalidasi file-file Skill.
- **Safe Execution Environment:** Pembungkusan (wrapping) eksekusi command OS menggunakan Rust `std::process::Command` dengan filter strict (mencegah shell/command injection) saat memanggil tool eksternal (seperti `nmap`, `ffuf`, dll).

## User Flow & CLI Design

Antarmuka utama untuk Dalang adalah berbasis Command Line Interface (CLI):

1. **Initialization:**

   ```bash
   dalang init
   ```

   _Menyiapkan environment awal, membuat folder `.dalang` di direktori lokal untuk menyimpan konfigurasi kredensial (keys LLM) dan local custom Skills._

2. **Run Automated Scan:**

   ```bash
   dalang scan --target https://example.com --skills web-basic,nmap-port
   ```

   _Mode utama. Agent membaca task, mencari file skill yang relevan, melakukan context reasoning, lalu memerintahkan crawler atau terminal OS untuk menggali kerentanan selangkah demi selangkah._

3. **Interactive / Copilot Mode:**
   ```bash
   dalang interact --target https://example.com
   ```
   _Membuka sesi interaktif (REPL) di mana pentester bisa berdiskusi dua arah dengan agent. Pengguna bisa memberikan instruksi secara natural language untuk mengeksekusi sub-task ad-hoc pada bagian target tertentu._
