# Product Requirements Document (PRD): Dalang

## Executive Summary & Objectives

**Dalang** adalah sebuah framework dan tool keamanan siber (_Cybersecurity_) berbasis AI Agent yang dirancang untuk melakukan scanning kerentanan pada aplikasi web dan jaringan. Berbeda dengan scanner tradisional yang terpaku pada database CVE statis (seperti Nuclei) dan heuristik deterministik, tool ini menggunakan arsitektur ReAct (Reasoning and Acting) untuk menganalisis logika aplikasi secara dinamis, mengontrol browser via CDP, dan mengeksekusi tool keamanan lokal melalui modul berbasis Markdown (`.md`).

Tujuan utama Dalang bukan sekadar menjalankan tool otomasi, namun meniru jalan pikiran dan proses coba-coba (trial & error) seorang pentester manusia, mulai dari enumerasi awal hingga mendeteksi celah secara mandiri.

## Core Features

1. **Markdown Skill Engine**
   - Mendefinisikan aksi atau metodologi penetration testing dalam bentuk file Markdown (`.md`).
   - Setiap file `.md` akan memiliki `Frontmatter` (seperti `tool_path`, variabel `args`, dan parameter `requires_root`) untuk mengontrol eksekusi _local binary/CLI tools_.
   - Body Markdown berisi System Prompt (Role "Authorized Auditor", Task analisis, dan Constraints mitigasi) untuk mengatur cara AI menganalisis hasil output (_Defensive Prompting_).
2. **AI Tool Calling Bridge via JSON**
   - Menyediakan interface komunikasi standar berbasis JSON antara core engine (Rust) dan model AI (LLM).
   - Memastikan aksi spesifik yang diputuskan oleh AI lewat metode Tool Calling (function calling) dapat dipetakan dan diparsing dengan aman menjadi argumen sistem atau permintaan HTTP jaringan.
3. **CDP Web Crawler**
   - Integrasi headless browser secara fungsional menggunakan Chrome DevTools Protocol (CDP).
   - Memungkinkan crawler untuk melakukan bypass terhadap client-side rendering (SPA), menangkap traffic API/jaringan lewat interceptor, menyusun peta struktur DOM, dan mendeteksi injeksi kompleks (seperti DOM-based XSS) secara realistis seolah-olah dieksekusi oleh user asli.
4. **LLM Agnostic API**
   - Dirancang arsitektur modular yang mendukung berbagai provider LLM (seperti OpenAI (API & Oauth), Anthropic (API & Oauth), Gemini (API & Oauth), maupun open-source model lokal via Ollama/vLLM).
   - Menyediakan layer abstraksi komunikasi yang memungkinkan pengguna menukar otak AI backend tanpa harus mengubah core engine Dalang.
5. **Defensive Prompting Layer (Bypass Strictness AI)**
   - Karena standar LLM (terutama model terkelola komersial) sangat berhati-hati merespon prompt yang berkaitan dengan "finding vulnerabilities" (Safety Filters), Dalang memiliki mekanisme internal _Defensive Prompting & Roleplay Framing_.
   - Core engine menyuntikkan template _system prompt_ khusus (diambil dari `.md`) untuk memosisikan AI sebagai _Authorized Security Auditor_ sehingga engine terhindar dari pemblokiran _AI strictness_ tanpa harus memodifikasi core model.
6. **Universal Native Tool Execution**
   - Dalang dapat memerintahkan sistem OS secara langsung untuk mengeksekusi utilitas eksternal yang sudah terinstal di komputer pengguna (seperti `nmap`, `ffuf`, `hydra`, dsb).
   - Aturan cara menggunakan sebuah external tool, output formatnya, dan maksud tool tersebut secara keseluruhan digambarkan ke dalam bentuk deskripsi Markdown (_Skill_), sehingga mirip dengan platform automasi security universal yang mampu di-extend tanpa mendevelop kode analyzer tambahan.

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
