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

## Sprint 7: Local Authentication & Multi-Provider OAuth Integration

**Goal:** Memudahkan _Developer Experience_ dalam menggunakan berbagai provider (Gemini, Anthropic, OpenAI) dengan memungkinkan login via OAuth (Browser Callback) atau membaca sesi CLI dari mesin host (misal `gemini-cli`, `gcloud`).

- **[DAL-701] - Feature - CLI Session Token Extractor**
  - Implementasikan rust _helper_ yang sanggup mengeksekusi command (seperti `gcloud auth print-access-token` atau membaca config `gemini-cli`) dan meretas sesi yang sedang hidup.
- **[DAL-702] - Feature - Universal OAuth Web Server Callback**
  - Buat handler `localhost` sederhana di port khusus (misal 38343) untuk menerima `code` oauth dari Google, Anthropic, atau provider lain dan melakukan token exchange.
- **[DAL-703] - Feature - Auth Persistence**
  - Buat mekanisme penyimpan _access_token_ dan _refresh_token_ secara aman (misal `keyring` rust) di `.dalang/credentials.json` untuk mencegah autentikasi manual yang berulang.

## Sprint 8: Autonomous Orchestrator (Auto-Pilot Mode)

**Goal:** Mengizinkan engine melakukan _chaining tool_ secara otomatis tanpa perlu di-skrip satu per satu oleh pengguna, mengubah fungsi command `dalang scan` dari eksekusi statis menjadi agen mandiri sepenuhnya.

- **[DAL-801] - Feature - Skill Library Cataloger**
  - Modifikasi _core engine_ agar memuat (load) seluruh direktori `skills/` di awal aplikasi dan mengabstraksi semuanya ke dalam _sistem prompt_ utama yang luas ("Berikut adalah Tool yang kamu miliki dan tujuannya: <List>").
- **[DAL-802] - Feature - High-Level ReAct Loop (Meta Orchestration)**
  - Bangun loop level-atas di mana LLM memutuskan _Path_ serangan secara runut.
  - LLM tidak hanya me-_return_ JSON untuk OS command, tapi JSON untuk "mengamankan observasi" dan "memilih skill berikutnya", e.g., `{"next_action": "use_nmap"}` -> _loop internal Nmap jalan_ -> kembali ke _loop utama_ -> `{"next_action": "use_ffuf_from_nmap_port80"}`.
- **[DAL-803] - Feature - Vulnerability Report Aggregation**
  - Sediakan mekanisme pengumpul fakta untuk akhir skenario, merepresentasikan hasil celah utuh secara tertata di Terminal.

## Sprint 9: Robust Skill Meta-Library

**Goal:** Memanfaatkan _framework_ Autonomous Dalang dengan memproduksi file `.md` (_skills_) secara masif dan mendalam untuk menangani target infrastruktur skala enterprise.

- **[DAL-901] - Documentation - Cloud & Container Auditing**
  - Tambahkan skill `.md` khusus infrastruktur modern: `kubectl_audit`, `aws_cli_enum`, `docker_escape_check`.
- **[DAL-902] - Documentation - Advanced Web Exploitation**
  - Tambahkan skill `.md` lanjutan: `wpscan`, `joomscan`, `nikto_scanner`, `xss_strike`.
- **[DAL-903] - Documentation - Network Protocols Deep Dive**
  - Tambahkan skill untuk service spesifik: `hydra_bruteforce`, `smbclient_enum`, `snmpwalk_gather`.
