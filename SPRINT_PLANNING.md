# Dalang Sprint Planning & Backlog

Berikut adalah rincian Sprint Planning awal (Sprint 1-3) untuk mengimplementasikan fungsionalitas inti framework "Dalang" berdasarkan PRD.

## Sprint 1: Core Foundation & Basic OS Execution

**Goal:** Membangun fondasi CLI, parser untuk file `.md` (Skill), dan eksekusi command OS sederhana (dummy tools) dengan security wrapper.

- **[DAL-101] - Feature - Setup Rust Project & CLI Structure**
  - Inisialisasi project dengan `cargo init`.
  - Setup CLI parsing menggunakan `clap` (perintah: `init`, `scan`, `interact`).
  - Buat struktur direktori sesuai `DEV_RULES.md` (`src/core`, `src/executor`, `skills/`).
- **[DAL-102] - Feature - Implement Markdown Frontmatter Parser**
  - Implementasi parser untuk membaca file `skills/*.md`.
  - Ekstrak konfigurasi dari YAML/TOML frontmatter menggunakan `serde` dan parser markdown ekstrak System Prompt.
- **[DAL-103] - Feature - Implement Safe OS Command Executor**
  - Buat wrapper untuk `std::process::Command` dengan pencegahan command injection (arg parsing terpisah).
  - Tambahkan timeout execution dan stdout/stderr capture.
- **[DAL-104] - Verification - Unit Tests for Parser & Executor**
  - Tulis unit test untuk memvalidasi parser Markdown.
  - Tulis test (dummy) untuk sistem OS execution (contoh: mengeksekusi `echo` atau `ls` dengan argumen aman).

## Sprint 2: LLM Integration & Tool Calling Bridge

**Goal:** Menghubungkan core engine dengan provider LLM dan mengonversi respons Tool Calling JSON menjadi aksi lokal.

- **[DAL-201] - Feature - LLM Provider Abstraction Layer**
  - Buat trait abstraction di `src/llm/` untuk provider AI.
  - Implementasi koneksi dasar HTTP client (menggunakan `reqwest`) ke satu provider awal (misal: OpenAI atau Ollama).
- **[DAL-202] - Feature - JSON Tool Calling Bridge**
  - Definisikan struct schema JSON untuk permintaan tool dari LLM.
  - Buat logika konversi dari JSON output LLM menuju parameter input eksekutor OS (`DAL-103`).
- **[DAL-203] - Feature - ReAct Orchestrator Loop**
  - Buat event loop di `src/core/` yang mengirim prompt (+ skill context), menerima respon Tool Call LLM, mengeksekusi aksi, dan menyuntikkan hasil kembali ke LLM (Reason -> Act -> Observe).

## Sprint 3: CDP Web Crawler Integration

**Goal:** Menambahkan kemampuan web crawling dan intercepting via Chrome DevTools Protocol untuk skenario pentest web tingkat lanjut.

- **[DAL-301] - Feature - Headless Browser Initialization**
  - Konfigurasi library CDP berbasis Rust (contoh: `chromiumoxide`) untuk meluncurkan headless Chrome/Chromium lokal.
- **[DAL-302] - Feature - DOM Navigation & Interceptor Tool**
  - Buat fungsi navigasi ke target URL.
  - Implementasikan Network Intercepting untuk menangkap traffic HTTP/XHR dan merender struktur DOM (mengatasi SPA).
- **[DAL-303] - Feature - CDP Tool Calling Registration**
  - Hubungkan fungsionalitas CDP ini sebagai "Tool" yang bisa di-_invoke_ oleh LLM di dalam ReAct loop (seperti tool click, typing, extract DOM).

## Sprint 4: Defensive Prompting Engine (Prompt Engineering)

**Goal:** Mengatasi strictness model (AI Safety Filters) agar agent dapat melakukan audit agresif tanpa diblokir oleh provider LLM.

- **[DAL-401] - Feature - System Prompt Injector**
  - Buat loader khusus untuk memuat template defensive & roleplaying prompt dari `.md` skills secara dinamis.
  - Paksakan role "Authorized Pentester" di awal semua pesan API.
- **[DAL-402] - Feature - Context & Violation Aggregator**
  - Deteksi dini bila LLM menolak melakukan instruksi (contoh: merespon "I cannot assist with...").
  - Auto-reprompt atau rotasi cara bertanya apabila terjadi pemblokiran (jailbreak loop mitigation).

## Sprint 5: Universal Tool ecosystem Integration

**Goal:** Mengimplementasi mekanisme eksekusi tool OS pihak ketiga secara otomatis berdasarkan deskripsi di file `.md`, mirip seperti pola framework OpenClaw.

- **[DAL-501] - Feature - Extended Frontmatter Parser**
  - Ubah parser YAML/TOML agar mendukung _parsing_ deklarasi eksekusi tool di file markdown meliputi field: `tool_path`, array `args` (dengan dukungan placeholder seperti `{{target}}`), dan `requires_root`.
- **[DAL-502] - Feature - Command Argument Interpolator**
  - Implementasikan fungsi interpolasi parameter LLM JSON ke dalam placeholder `args` dari frontmatter.
- **[DAL-503] - Feature - Defensive System Prompt Constructor**
  - Rangkai body markdown (Role, Task, Constraints) dan operkan sebagai injeksi ke System Prompt message LLM setiap tool dieksekusi, sehingga instruksi mitigasi dan role auditor melekat kuat.

## Sprint 6: Expanded Skill Library

**Goal:** Membuat modul `.md` sebanyak mungkin untuk berbagai macam tool security standar industri.

- **[DAL-601] - Documentation - Core Network Skills (.md)**
  - Tulis modul mapping beserta defensive system prompt untuk `nmap`, `masscan`, `rustscan`.
- **[DAL-602] - Documentation - Web Audit Skills (.md)**
  - Tulis modul untuk `sqlmap`, `ffuf`, `dirb`, `gobuster`.
