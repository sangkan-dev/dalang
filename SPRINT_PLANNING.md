# Dalang Sprint Planning & Backlog

Berikut adalah rincian Sprint Planning untuk mengimplementasikan fungsionalitas framework "Dalang" berdasarkan PRD.

> **Legenda Status:**
> - ✅ = Selesai (Done)
> - 🔄 = Sedang Dikerjakan (In Progress)
> - ⬜ = Belum Dimulai (Not Started)

---

## Sprint 1: Core Foundation & Basic OS Execution ✅

**Goal:** Membangun fondasi CLI, parser untuk file `.md` (Skill), dan eksekusi command OS sederhana (dummy tools) dengan security wrapper.

- ✅ **[DAL-101] - Feature - Setup Rust Project & CLI Structure**
  - Inisialisasi project dengan `cargo init`.
  - Setup CLI parsing menggunakan `clap` (perintah: `init`, `scan`, `interact`).
  - Buat struktur direktori sesuai `DEV_RULES.md` (`src/core`, `src/executor`, `skills/`).
- ✅ **[DAL-102] - Feature - Implement Markdown Frontmatter Parser**
  - Implementasi parser untuk membaca file `skills/*.md`.
  - Ekstrak konfigurasi dari YAML/TOML frontmatter menggunakan `serde` dan parser markdown ekstrak System Prompt.
- ✅ **[DAL-103] - Feature - Implement Safe OS Command Executor**
  - Buat wrapper untuk `std::process::Command` dengan pencegahan command injection (arg parsing terpisah).
  - Tambahkan timeout execution dan stdout/stderr capture.
- ✅ **[DAL-104] - Verification - Unit Tests for Parser & Executor**
  - Tulis unit test untuk memvalidasi parser Markdown.
  - Tulis test (dummy) untuk sistem OS execution (contoh: mengeksekusi `echo` atau `ls` dengan argumen aman).

## Sprint 2: LLM Integration & Tool Calling Bridge ✅

**Goal:** Menghubungkan core engine dengan provider LLM dan mengonversi respons Tool Calling JSON menjadi aksi lokal.

- ✅ **[DAL-201] - Feature - LLM Provider Abstraction Layer**
  - Buat trait abstraction di `src/llm/` untuk provider AI.
  - Implementasi koneksi dasar HTTP client (menggunakan `reqwest`) ke satu provider awal (misal: OpenAI atau Ollama).
- ✅ **[DAL-202] - Feature - JSON Tool Calling Bridge**
  - Definisikan struct schema JSON untuk permintaan tool dari LLM.
  - Buat logika konversi dari JSON output LLM menuju parameter input eksekutor OS (`DAL-103`).
- ✅ **[DAL-203] - Feature - ReAct Orchestrator Loop**
  - Buat event loop di `src/core/` yang mengirim prompt (+ skill context), menerima respon Tool Call LLM, mengeksekusi aksi, dan menyuntikkan hasil kembali ke LLM (Reason -> Act -> Observe).

## Sprint 3: CDP Web Crawler Integration ✅

**Goal:** Menambahkan kemampuan web crawling dan intercepting via Chrome DevTools Protocol untuk skenario pentest web tingkat lanjut.

- ✅ **[DAL-301] - Feature - Headless Browser Initialization**
  - Konfigurasi library CDP berbasis Rust (`chromiumoxide`) untuk meluncurkan headless Chrome/Chromium lokal.
  - Diimplementasi sebagai `LazyBrowser` — browser hanya diluncurkan saat pertama kali dibutuhkan oleh tool call.
- ✅ **[DAL-302] - Feature - DOM Navigation & Interceptor Tool**
  - Buat fungsi navigasi ke target URL.
  - Implementasikan Network Intercepting untuk menangkap traffic HTTP/XHR dan merender struktur DOM (mengatasi SPA).
- ✅ **[DAL-303] - Feature - CDP Tool Calling Registration**
  - Hubungkan fungsionalitas CDP ini sebagai "Tool" yang bisa di-_invoke_ oleh LLM di dalam ReAct loop (seperti tool click, typing, extract DOM).

## Sprint 4: Defensive Prompting Engine (Prompt Engineering) ✅

**Goal:** Mengatasi strictness model (AI Safety Filters) agar agent dapat melakukan audit agresif tanpa diblokir oleh provider LLM.

- ✅ **[DAL-401] - Feature - System Prompt Injector**
  - Buat loader khusus untuk memuat template defensive & roleplaying prompt dari `.md` skills secara dinamis.
  - Paksakan role "Authorized Pentester" di awal semua pesan API.
- ✅ **[DAL-402] - Feature - Context & Violation Aggregator**
  - Deteksi dini bila LLM menolak melakukan instruksi (contoh: merespon "I cannot assist with...").
  - Auto-reprompt atau rotasi cara bertanya apabila terjadi pemblokiran (jailbreak loop mitigation).

## Sprint 5: Universal Tool Ecosystem Integration ✅

**Goal:** Mengimplementasi mekanisme eksekusi tool OS pihak ketiga secara otomatis berdasarkan deskripsi di file `.md`, mirip seperti pola framework OpenClaw.

- ✅ **[DAL-501] - Feature - Extended Frontmatter Parser**
  - Ubah parser YAML/TOML agar mendukung _parsing_ deklarasi eksekusi tool di file markdown meliputi field: `tool_path`, array `args` (dengan dukungan placeholder seperti `{{target}}`), dan `requires_root`.
- ✅ **[DAL-502] - Feature - Command Argument Interpolator**
  - Implementasikan fungsi interpolasi parameter LLM JSON ke dalam placeholder `args` dari frontmatter.
- ✅ **[DAL-503] - Feature - Defensive System Prompt Constructor**
  - Rangkai body markdown (Role, Task, Constraints) dan operkan sebagai injeksi ke System Prompt message LLM setiap tool dieksekusi, sehingga instruksi mitigasi dan role auditor melekat kuat.

## Sprint 6: Expanded Skill Library ✅

**Goal:** Membuat modul `.md` sebanyak mungkin untuk berbagai macam tool security standar industri.

- ✅ **[DAL-601] - Documentation - Core Network Skills (.md)**
  - Tulis modul mapping beserta defensive system prompt untuk `nmap`, `masscan`, `rustscan`.
- ✅ **[DAL-602] - Documentation - Web Audit Skills (.md)**
  - Tulis modul untuk `sqlmap`, `ffuf`, `gobuster`.

## Sprint 7: Local Authentication & Multi-Provider OAuth Integration ✅

**Goal:** Memudahkan _Developer Experience_ dalam menggunakan berbagai provider (Gemini, Anthropic, OpenAI) dengan memungkinkan login via OAuth (Browser Callback) atau membaca sesi CLI dari mesin host (misal `gemini-cli`, `gcloud`).

- ✅ **[DAL-701] - Feature - CLI Session Token Extractor**
  - Implementasikan rust _helper_ yang sanggup mengeksekusi command (seperti `gcloud auth print-access-token` atau membaca config `gemini-cli`) dan meretas sesi yang sedang hidup.
- ✅ **[DAL-702] - Feature - Universal OAuth Web Server Callback**
  - Buat handler `localhost` sederhana di port khusus (misal 38343) untuk menerima `code` oauth dari Google, Anthropic, atau provider lain dan melakukan token exchange.
- ✅ **[DAL-703] - Feature - Auth Persistence**
  - Buat mekanisme penyimpan _access_token_ dan _refresh_token_ secara aman (misal `keyring` rust) di `.dalang/credentials.json` untuk mencegah autentikasi manual yang berulang.

## Sprint 8: Autonomous Orchestrator (Auto-Pilot Mode) ✅

**Goal:** Mengizinkan engine melakukan _chaining tool_ secara otomatis tanpa perlu di-skrip satu per satu oleh pengguna, mengubah fungsi command `dalang scan` dari eksekusi statis menjadi agen mandiri sepenuhnya.

- ✅ **[DAL-801] - Feature - Skill Library Cataloger**
  - Modifikasi _core engine_ agar memuat (load) seluruh direktori `skills/` di awal aplikasi dan mengabstraksi semuanya ke dalam _sistem prompt_ utama yang luas ("Berikut adalah Tool yang kamu miliki dan tujuannya: <List>").
- ✅ **[DAL-802] - Feature - High-Level ReAct Loop (Meta Orchestration)**
  - Bangun loop level-atas di mana LLM memutuskan _Path_ serangan secara runut.
  - LLM tidak hanya me-_return_ JSON untuk OS command, tapi JSON untuk "mengamankan observasi" dan "memilih skill berikutnya", e.g., `{"next_action": "use_nmap"}` -> _loop internal Nmap jalan_ -> kembali ke _loop utama_ -> `{"next_action": "use_ffuf_from_nmap_port80"}`.
- ✅ **[DAL-803] - Feature - Vulnerability Report Aggregation**
  - Sediakan mekanisme pengumpul fakta untuk akhir skenario, merepresentasikan hasil celah utuh secara tertata di Terminal.

## Sprint 9: Robust Skill Meta-Library ✅

**Goal:** Memanfaatkan _framework_ Autonomous Dalang dengan memproduksi file `.md` (_skills_) secara masif dan mendalam untuk menangani target infrastruktur skala enterprise.

- ✅ **[DAL-901] - Documentation - Cloud & Container Auditing**
  - Tambahkan skill `.md` khusus infrastruktur modern: `kubectl_audit`, `aws_cli_enum`, `docker_escape_check`.
- ✅ **[DAL-902] - Documentation - Advanced Web Exploitation**
  - Tambahkan skill `.md` lanjutan: `wpscan`, `nikto_scanner`, `xss_strike`.
- ✅ **[DAL-903] - Documentation - Network Protocols Deep Dive**
  - Tambahkan skill untuk service spesifik: `hydra_bruteforce`, `smbclient_enum`, `snmpwalk_gather`.

## Sprint 10: Advanced Auto-Pilot & Persistent Context Memory ✅

**Goal:** Memaksimalkan fleksibilitas agen otonom dengan mengizinkan AI membentuk argumen command sendiri di atas path tool dasar, serta mengimplementasikan _Persistent Context_ (memori jangka panjang) agar agen tidak kehilangan orientasi logis di tengah operasi multi-tahap.

- ✅ **[DAL-1001] - Feature - Dynamic Argument Injection (Free-form Args)**
  - Modifikasi skema _Meta-Tool_ `execute_skill` agar selain menerima `skill_name`, ia juga menerima array string tambahan opsional `custom_args`.
  - Di layer _executor_, gabungkan argumen statis dari `.md` (misal wajib `xsstrike`) dengan argumen bentukan AI (misal `--crawl -l 3 --skip-dom`).
  - Terapkan validasi ketat/blacklist (misal blokir arg `&&` atau `;` untuk mencegah AI melakukan command injection pada OS host).
- ✅ **[DAL-1002] - Feature - Persistent Context Memory Engine**
  - Buat representasi memori (seperti `ContextManager`) di `src/core/memory.rs` yang menyimpan jejak observasi ringkas (bukan seluruh raw JSON/HTML) yang bisa bertahan antar-sesi atau antar-loop.
  - LLM tidak hanya menyuap ulang seluruh _history_ chat, melainkan meringkas observasi sebelumnya ("Saya baru saja port scan, menemukan X dan Y") dan menaruhnya di slot memori _System Prompt_.
- ✅ **[DAL-1003] - Feature - Project README & Documentation Polish**
  - Buat `README.md` utama berbahasa Inggris ringkas dan formal yang menjelaskan apa itu Dalang, cara instalasi, dependensi, dan showcase perintah `dalang scan --auto`.

## Sprint 11: Addressing TODOs & Code Cleanup ✅

**Goal:** Membersihkan dan melengkapi fungsionalitas yang tertunda (_technical debt_) berupa komentar `// TODO` di dalam _codebase_, sehingga framework menjadi lebih solid dan lengkap sebelum rilis 1.0.

- ✅ **[DAL-1101] - Cleanup - Implement Explicit Tools Definition (`src/llm/openai.rs`)**
  - Saat ini _tool calling_ dikelola murni via JSON di _System Prompt_.
  - Ekstrak abstrak `SkillDefinition` menjadi skema _Native Tool Calling_ API milik OpenAI/Anthropic/Gemini (menggunakan parameter `tools` pada JSON payload HTTP request) agar deteksi _tool_ oleh LLM lebih akurat dan mengurangi beban token di _System Prompt_.
- ✅ **[DAL-1102] - Feature - Implement `init` Command (`src/main.rs`)**
  - Gantikan `// TODO: Implement init logic` dengan logika pembuatan _scaffolding_ direktori awal.
  - Perintah `dalang init` akan secara otomatis membuat folder `skills/` dan men-_generate_ seluruh 22 skill bawaan dari `bundled.rs`, sehingga pengguna baru punya titik awal (_starting point_) yang terstandardisasi.
- ✅ **[DAL-1103] - Feature - Implement `interact` Command (`src/main.rs`)**
  - Gantikan `// TODO: Implement interactive logic` dengan sebuah REPL (_Read-Eval-Print Loop_) interaktif.
  - Alih-alih mengeksekusi satu command `scan` lalu selesai, mode `interact` memungkinkan user me-maintain sebuah sesi obrolan (chat) dengan _DalangEngine_. Pengguna bisa bertanya, _"Coba kamu check port 80"_, lalu Dalang akan merespons dengan tool, dan pengguna bisa merespons lagi. Ini mirip dengan _Auto-Pilot_ tetapi bersifat _Human-in-the-Loop_ (HITL).

## Sprint 12: Interactive Model Selection ✅

**Goal:** Memberikan pengalaman interaktif bagi pengguna untuk memilih model AI secara dinamis setelah mereka berhasil melakukan autentikasi (login), baik melalui API Key maupun OAuth.

- ✅ **[DAL-1201] - Feature - Implement Provider Model Fetching (`src/llm/` & Provider API)**
  - Implementasikan endpoint/metode untuk mengambil daftar model yang didukung secara langsung dari API Provider (misalnya endpoint `/v1/models` untuk list model) menggunakan token/auth yang baru saja didapatkan.
- ✅ **[DAL-1202] - UX - Interactive CLI Prompt (`src/main.rs`)**
  - Integrasikan _crate_ CLI interaktif (seperti `dialoguer` atau `inquire`) ke dalam alur `dalang login`.
  - Setelah token berhasil diperoleh dan divalidasi, program tidak langsung _exit_, melainkan menampilkan _loading spinner_ saat menarik daftar model, lalu menampilkan antarmuka pilihan _dropdown_ di terminal (contoh: `gemini-1.5-pro`, `gemini-1.5-flash`, dll) untuk dipilih oleh _user_.
- ✅ **[DAL-1203] - Feature - Persist User Model Preference (`src/auth/persistence.rs` atau Config)**
  - Setelah pengguna memilih model, simpan preferensi ini secara lokal (misalnya di _keyring_ atau _config file_ default di dalam direktori `~/.dalang/`).
  - Ubah perilaku default eksekusi CLI (seperti command `scan` dan `interact`) agar membaca dari preferensi tersimpan ini jika _environment variable_ `LLM_MODEL` tidak disediakan.

## Sprint 13: Dynamic Provider Configuration ✅

**Goal:** Menggantikan konfigurasi statis/hardcoded `LLM_BASE_URL` dan `LLM_MODEL` dengan resolusi dinamis berdasarkan _provider_ yang sedang aktif.

- ✅ **[DAL-1301] - Refactor - Provider-Aware Defaults (`src/llm/mod.rs` & `src/main.rs`)**
  - Buat mekanisme resolusi URL default secara dinamis. Jika provider adalah `openai`, maka default `LLM_BASE_URL` haruslah `https://api.openai.com/v1` dan `LLM_MODEL` haruslah `gpt-4o` (buka lagi `gemini-1.5-pro`).
  - Lakukan hal yang sama untuk provider lain (misal Anthropic, local Ollama).
- ✅ **[DAL-1302] - Feature - Persist Active Provider (`src/auth/persistence.rs` & `src/main.rs`)**
  - Saat `dalang login --provider <NAME>`, simpan nama provider ke dalam `keyring` (atau config lokal) bersamaan dengan token.
  - Saat mengeksekusi `dalang scan` atau `dalang interact`, CLI harus mencari tahu provider mana yang aktif dari persistensi, lalu menerapkan resolusi dinamis untuk `LLM_BASE_URL` jika _environment variable_ tidak disediakan.

## Sprint 14: Gemini CloudCode Native Provider & OAuth ✅

**Goal:** Mengimplementasi koneksi langsung ke Google Cloud Code Assist endpoint (`cloudcode-pa.googleapis.com`) menggunakan format native `generateContent` — bukan via OpenAI-compatible wrapper — sesuai cara kerja Gemini CLI resmi.

- ✅ **[DAL-1401] - Feature - CloudCode Native generateContent Provider (`src/llm/gemini_codeassist.rs`)**
  - Buat LLM provider baru `GeminiCodeAssistProvider` yang mengirim request ke `cloudcode-pa.googleapis.com/v1internal:generateContent` menggunakan format Google-native (bukan OpenAI).
  - Konversi internal `Message` list ke Google `Content` list + `system_instruction`.
  - Konversi tool definitions ke format Google `FunctionDeclaration`.
  - Parse response dari `candidates[0].content.parts` (text atau `function_call`).
- ✅ **[DAL-1402] - Feature - Full Gemini CLI OAuth Flow (`src/auth/gemini_codeassist.rs`)**
  - Implementasi full OAuth2 authorization code flow:
    - Generate code_verifier + code_challenge (PKCE S256).
    - Buka browser ke Google OAuth consent URL.
    - Jalankan `tiny_http` localhost server untuk menerima callback code.
    - Exchange code → access_token + refresh_token via Google token endpoint.
  - Simpan semua token ke keyring via `persistence.rs`.
  - Auto-detect GCP project via `loadCodeAssist` metadata endpoint.
- ✅ **[DAL-1403] - Feature - 429 Rate Limit Retry & Model Fallback Chain**
  - Bedakan `RATE_LIMIT_EXCEEDED` (tunggu retry-after lalu coba ulang model yang sama) vs `MODEL_CAPACITY_EXHAUSTED` (langsung fallback ke model lain).
  - Implementasi fallback chain 6 model: `gemini-3.1-pro-preview` → `gemini-3-pro-preview` → `gemini-3-flash-preview` → `gemini-2.5-pro` → `gemini-2.5-flash` → `gemini-2.5-flash-lite`.
  - Parse `retry-after` dari response body ("Your quota will reset after Xs.").
- ✅ **[DAL-1404] - Feature - OAuth Token Auto-Refresh on 401 (`src/llm/gemini_codeassist.rs`)**
  - Wrap `access_token` di `Arc<tokio::sync::Mutex<String>>` untuk interior mutability.
  - Pada response 401 Unauthorized, otomatis panggil `refresh_access_token()` (baca refresh_token dari keyring, POST ke Google token endpoint, simpan token baru).
  - Retry request sekali dengan token baru. Jika refresh gagal, tampilkan pesan error yang jelas untuk re-login.

## Sprint 15: Internationalization & Skill Library Expansion ✅

**Goal:** Menerjemahkan seluruh skill dan prompt ke bahasa Inggris agar kualitas output LLM optimal, serta menambah 6 skill baru untuk memperkaya kapabilitas framework.

- ✅ **[DAL-1501] - Refactor - Translate All Skills to English**
  - Terjemahkan 7 file skill dari bahasa Indonesia ke Inggris: `nmap_scanner`, `masscan_fast`, `rustscan_audit`, `ffuf_fuzzer`, `sqlmap_tester`, `web-audit`, `testing`.
  - Setiap skill kini memiliki bagian Role, Task, dan Constraints yang detail dalam bahasa Inggris dengan referensi CWE.
- ✅ **[DAL-1502] - Feature - Create 6 New Skills**
  - `header_analyzer.md` — HTTP security header analysis via `curl -sI` (CWE-693, CWE-1021, CWE-16).
  - `ssl_scan.md` — TLS/SSL configuration audit via `sslscan` (CWE-326, CWE-327).
  - `jwt_analysis.md` — JWT token extraction & analysis via browser CDP (CWE-345, CWE-347).
  - `nuclei_vuln_scan.md` — Template-based vulnerability scanning via `nuclei` (CWE-200).
  - `subdomain_enum.md` — Subdomain enumeration via `subfinder` (CWE-200).
  - `gobuster_dir.md` — Directory and file brute-forcing via `gobuster` (CWE-538, CWE-548).
  - Total skill library: **22 skills** (16 existing + 6 new), semua terdaftar di `bundled.rs`.
- ✅ **[DAL-1503] - Bugfix - Fix hydra_bruteforce Skill Arguments**
  - Ganti path wordlist yang tidak ada (`users.txt`/`pass.txt`) dengan path standar SecLists (`/usr/share/seclists/...`).
  - Tambahkan flag `-t 4 -f` dan gunakan placeholder `{{target}}` dengan benar.

## Sprint 16: Enhanced Report Quality & Prompt Engineering ✅

**Goal:** Meningkatkan kualitas output laporan vulnerability dari format generik menjadi format bug-bounty-grade dengan PoC, URL spesifik, dan klasifikasi CWE/CVSS.

- ✅ **[DAL-1601] - Feature - Bug-Bounty-Style Report Prompt (Autonomous Mode)**
  - Rewrite system prompt untuk `run_autonomous_loop()` dengan template laporan komprehensif yang mewajibkan:
    - Exact affected URL + parameter
    - CWE classification + CVSS 3.1 score
    - Step-by-step PoC dengan payload & curl command
    - Raw evidence (request/response)
    - Impact analysis + remediation
- ✅ **[DAL-1602] - Feature - Enhanced Interactive Mode Prompt**
  - Upgrade system prompt `run_interactive_loop()` dengan format laporan dan instruksi tool calling yang lebih detail.
- ✅ **[DAL-1603] - Feature - Enhanced Scan Mode Tool Description**
  - Tambahkan requirement PoC/URL/parameter tracking di `tool_description` pada `run_scan_loop()`.
- ✅ **[DAL-1604] - Feature - Enhanced Memory Context Prompt**
  - Modifikasi `get_summary_prompt()` di `memory.rs` agar menginstruksikan LLM untuk mereferensikan URL, parameter, dan temuan spesifik dari observasi sebelumnya.

## Sprint 17: Robustness & Developer Experience ✅

**Goal:** Meningkatkan ketangguhan eksekusi dan pengalaman developer dengan fitur keamanan runtime, kontrol iterasi, dan mode debug.

- ✅ **[DAL-1701] - Feature - `requires_root` Enforcement (`src/core/engine.rs`)**
  - Tambahkan pemeriksaan `libc::geteuid() == 0` di `execute_skill_native()` sebelum menjalankan skill yang memerlukan root.
  - Jika tidak root: skip skill dengan warning, informasikan LLM untuk memilih skill alternatif.
  - Tambahkan dependency `libc = "0.2"` di `Cargo.toml`.
- ✅ **[DAL-1702] - Feature - Dynamic Iteration Limit (`--max-iter` / `-n`)**
  - Tambahkan flag `--max-iter` (`-n`) pada command `scan` untuk mengontrol jumlah iterasi auto-pilot.
  - Default: 15 iterasi. Nilai `0` = unlimited.
  - `run_autonomous_loop()` menerima parameter `max_iter: u32`.
- ✅ **[DAL-1703] - Feature - Dynamic Command Timeout (`--cmd-timeout`)**
  - Tambahkan flag `--cmd-timeout` pada command `scan` dan `interact` untuk mengontrol timeout eksekusi command.
  - Default: 300 detik. Nilai `0` = unlimited (`u64::MAX`).
  - `DalangEngine` menyimpan field `cmd_timeout: u64` + helper `effective_timeout()`.
- ✅ **[DAL-1704] - Feature - Verbose Debug Mode (`--verbose` / `-v`)**
  - Tambahkan global flag `-v` / `--verbose` pada CLI (`DalangArgs`).
  - Pass ke `DalangEngine` sebagai field `verbose: bool`.
  - Ketika aktif, cetak output `[VERBOSE]` di ketiga call site LLM (scan, autonomous, interactive):
    - Jumlah dan ukuran message sebelum dikirim.
    - Full response text + karakter setelah diterima.

## Sprint 18: Documentation Site Overhaul ✅

**Goal:** Memperbarui seluruh dokumentasi VitePress agar akurat mencerminkan state terkini dari codebase, termasuk 22 skill, flag CLI baru, dan arsitektur terbaru.

- ✅ **[DAL-1801] - Documentation - Update Built-in Skills Reference (`docs/skills/built-in.md`)**
  - Ganti daftar skill lama (termasuk phantom skills yang tidak ada) dengan daftar lengkap 22 skill aktual.
  - Fix: `wpscan_enumeration` → `wpscan_audit` (sesuai nama file asli).
  - Tambahkan 6 skill baru: header_analyzer, ssl_scan, jwt_analysis, nuclei_vuln_scan, subdomain_enum, gobuster_dir.
- ✅ **[DAL-1802] - Documentation - Update Installation Guide (`docs/guide/installation.md`)**
  - Fix deskripsi `dalang init` (bukan lagi "generate example-nmap.md", tapi "install 22 bundled skills").
  - Perluas daftar command instalasi tool pihak ketiga (sslscan, subfinder, nuclei, gobuster, dll).
- ✅ **[DAL-1803] - Documentation - Update Architecture Docs**
  - `docs/architecture/core-engine.md`: Tambahkan field `cmd_timeout`, method `effective_timeout()`, update tabel method.
  - `docs/architecture/llm-providers.md`: Fix default model ke `gemini-2.5-flash`, update deskripsi CloudCode endpoint.
- ✅ **[DAL-1804] - Documentation - Update Guide Docs**
  - `docs/guide/authentication.md`: Fix urutan resolusi auth: Keyring → Env → CLI Extractor.
  - `docs/guide/auto-pilot.md`: Dokumentasi flag `--max-iter` dan `--cmd-timeout`, update format laporan.
  - `docs/guide/scan-mode.md`: Tambahkan parameter `--max-iter`, `--cmd-timeout`, contoh usage.

---

## Sprint 19: Web UI — Backend Foundation (axum + WebSocket) ✅

**Goal:** Membangun backend web server menggunakan axum yang menyediakan REST API dan WebSocket endpoint untuk komunikasi real-time antara browser dan DalangEngine.

- ✅ **[DAL-1901] - Feature - Add `dalang web` CLI Command (`src/cli.rs` & `src/main.rs`)**
  - Tambahkan variant `Web` pada `Commands` enum dengan flag `--port` (default: 8080) dan `--open` (auto-buka browser).
  - Tambahkan dependency baru: `axum 0.8`, `tower-http 0.6` (CORS, static files), `rust-embed 8`, `dashmap 6`, `uuid 1` (v4).
- ✅ **[DAL-1902] - Feature - Engine Event System (`src/web/events.rs`)**
  - Buat `EngineEvent` enum sebagai abstraksi output dari DalangEngine:
    - `Thinking { iteration }` — LLM sedang reasoning
    - `AssistantMessage { content, done }` — Respons teks dari LLM (streaming-ready)
    - `ToolExecution { skill, command }` — Skill/tool sedang dieksekusi
    - `Observation { content, bytes }` — Hasil output dari tool execution
    - `SafetyRefusal { retry }` — LLM menolak, sedang auto-reprompt
    - `Report { markdown }` — Final vulnerability report
    - `Error { message }` — Error yang terjadi
  - Semua event di-serialize ke JSON via `serde::Serialize`.
- ✅ **[DAL-1903] - Refactor - Channel-Based Engine Output (`src/core/engine.rs`)**
  - Tambahkan method baru `run_interactive_ws()` dan `run_autonomous_ws()` yang menerima `tokio::sync::mpsc::Sender<EngineEvent>`.
  - Method ini identik dengan versi CLI (`run_interactive_loop`, `run_autonomous_loop`) namun mengirim `EngineEvent` ke channel alih-alih `println!()`.
  - Method CLI lama tetap berfungsi (backward compatible).
- ✅ **[DAL-1904] - Feature - axum Web Server (`src/web/mod.rs`)**
  - Setup axum router dengan:
    - Static file serving via `rust-embed` (serve Svelte dist/ files)
    - WebSocket upgrade endpoint di `/api/ws/{session_id}`
    - REST API routes di `/api/sessions`, `/api/skills`, `/api/reports`, `/api/settings`
    - CORS middleware via `tower-http`
  - Startup log: `[*] Dalang Web UI running at http://localhost:{port}`
- ✅ **[DAL-1905] - Feature - Session State Management (`src/web/state.rs`)**
  - Buat `AppState` struct yang di-share via `axum::Extension`:
    - `sessions: DashMap<Uuid, Session>` — active chat sessions
    - `Session` struct: target, messages history (`Vec<Message>`), mode (interactive/scan), created_at
  - Session lifecycle: create → chat via WS → persist messages → delete

## Sprint 20: Web UI — Svelte Frontend, WebSocket Chat & REST API ✅

**Goal:** Implementasi handler WebSocket untuk real-time chat dan REST API untuk manajemen session, skill, report, dan settings.

- ✅ **[DAL-2001] - Feature - WebSocket Chat Handler (`src/web/handlers/chat.rs`)**
  - Accept WebSocket upgrade di `/api/ws/{session_id}`
  - Terima JSON message dari client (`{ "type": "chat", "message": "..." }`)
  - Spawn tokio task: jalankan `DalangEngine::run_interactive_ws()` dengan `mpsc::Sender`
  - Forward setiap `EngineEvent` dari channel ke WebSocket sebagai JSON frame
  - Handle disconnect gracefully (abort engine task)
- ✅ **[DAL-2002] - Feature - Auto-Pilot Scan via WebSocket**
  - Terima `{ "type": "start_scan", "target": "...", "max_iter": 15 }` dari client
  - Spawn `run_autonomous_ws()` dengan event channel
  - Stream progress (Thinking, ToolExecution, Observation) dan final Report ke client
- ✅ **[DAL-2003] - Feature - Session REST API (`src/web/handlers/sessions.rs`)**
  - `POST /api/sessions` → create session (body: `{ target, mode }`) → return `{ id, target, mode, created_at }`
  - `GET /api/sessions` → list all sessions
  - `GET /api/sessions/{id}/messages` → return chat history
  - `DELETE /api/sessions/{id}` → remove session + cleanup
- ✅ **[DAL-2004] - Feature - Skills REST API (`src/web/handlers/skills.rs`)**
  - `GET /api/skills` → list semua skill (name, description, tool_path, requires_root)
  - `GET /api/skills/{name}` → detail skill termasuk system prompt
  - `PUT /api/skills/{name}` → update/toggle skill (enable/disable)
- ✅ **[DAL-2005] - Feature - Reports REST API (`src/web/handlers/reports.rs`)**
  - `GET /api/reports` → list saved report files (`dalang_report_*.md`)
  - `GET /api/reports/{filename}` → return report content (markdown)
  - `GET /api/reports/{filename}?format=html` → return rendered HTML report (via `pulldown-cmark`)
- ✅ **[DAL-2006] - Feature - Settings REST API (`src/web/handlers/settings.rs`)**
  - `GET /api/settings` → return current provider, model, auth status, endpoint mode
  - `PUT /api/settings` → update model preference, provider config
  - Gunakan existing `auth::persistence` untuk baca/tulis config

## Sprint 21: Web UI — Hardening, Polish & Testing ✅

**Goal:** Memperbaiki bug kritis, menambahkan fitur UX polish (toast, theme, command palette, mobile responsive), dan membangun infrastruktur testing untuk frontend dan backend web.

- ✅ **[DAL-2101] - Bugfix - Fix SkillDetail Type Mismatch (`types.ts` & `SkillsView.svelte`)**
  - Backend mengirim `system_prompt`, `role`, `task`, `constraints` tapi frontend mengharapkan `raw_prompt`.
  - Update `SkillDetail` type di `types.ts` dan render semua section di `SkillsView.svelte`.
- ✅ **[DAL-2102] - Bugfix - Fix Settings Persistence (`settings.rs` & `SettingsView.svelte`)**
  - Backend hanya menyimpan `model`, sekarang juga menyimpan `provider` dan `endpoint_mode`.
  - Frontend menampilkan auth status banner, `auth_method` read-only.
- ✅ **[DAL-2103] - Feature - Toast Notification System (`toast.ts` & `Toast.svelte`)**
  - Global toast store dengan `subscribe()` pattern, auto-dismiss, color-coded (success/error/warning/info).
  - Terintegrasi di `App.svelte`, `SettingsView`, `ReportsView`, `ChatView`.
- ✅ **[DAL-2104] - Feature - Dark/Light Theme Toggle (`theme.ts` & `app.css`)**
  - Theme store dengan localStorage persistence, `initTheme()` di `main.ts`.
  - Light theme CSS variables, toggle button di Sidebar footer.
  - Print media query untuk report printing.
- ✅ **[DAL-2105] - Feature - Session List in Sidebar (`Sidebar.svelte`)**
  - Complete rewrite Sidebar: session list dari API, delete sessions, periodic refresh setiap 10 detik.
  - Mobile hamburger menu dengan slide-in overlay dan backdrop.
- ✅ **[DAL-2106] - Feature - WebSocket Auto-Reconnect (`api.ts`)**
  - Exponential backoff (1s-16s), max 5 attempts, `intentionalClose` flag.
  - Reconnecting banner di `ChatView`, toast notifications on reconnect/failure.
- ✅ **[DAL-2107] - Feature - Configurable Command Timeout (`ChatView.svelte`)**
  - `cmdTimeout` state variable dengan input field di setup form.
  - Menggantikan hardcoded `300` di dua tempat.
- ✅ **[DAL-2108] - Feature - Report Download & Export (`ReportsView.svelte`)**
  - Complete rewrite: download Markdown, download HTML, print button.
  - Loading spinner saat fetch, file size in KB, toast notifications.
- ✅ **[DAL-2109] - Feature - Keyboard Shortcuts & Command Palette (`CommandPalette.svelte`)**
  - `Ctrl+K` command palette dengan fuzzy filtering, arrow key navigation.
  - `Ctrl+N` new session, `Escape` close palette.
  - 4 page entries dengan "Active" badge untuk halaman aktif.
- ✅ **[DAL-2110] - Testing - Frontend Unit Tests (vitest + jsdom)**
  - Setup vitest 3.2 + jsdom 26 environment.
  - 4 test files: `api.test.ts` (4 tests), `markdown.test.ts` (7 tests), `toast.test.ts` (3 tests), `theme.test.ts` (3 tests).
  - Total: 17 frontend tests, semua pass.
- ✅ **[DAL-2111] - Testing - Rust Web Handler Tests (`src/web/tests.rs`)**
  - 8 integration tests menggunakan `tower::ServiceExt::oneshot`.
  - Tests: list_skills, create+delete session, list_sessions, get_settings, get nonexistent skill 404, list_reports, static fallback, update_settings.
  - Total: 15 Rust tests (including existing), semua pass.

## Sprint 22: Web UI — Chat & Skills Polish ✅

**Goal:** Extract reusable ChatInput component, add skill search/filter + grid/list toggle, and fix remaining a11y warnings.

- ✅ **[DAL-2201] - Feature - ChatInput Component (`ChatInput.svelte`)**
  - Extracted reusable textarea+send button from ChatView.
  - Auto-resize via `$effect` tracking scrollHeight (max 6 rows / 144px).
  - Enter = send, Shift+Enter = newline, `items-end` flex alignment.
- ✅ **[DAL-2202] - Refactor - ChatView Uses ChatInput**
  - Replaced inline textarea block with `<ChatInput>` component.
  - Removed `handleKeydown` function, simplified ChatView by ~20 lines.
- ✅ **[DAL-2203] - Feature - Skill Search & Filter (`SkillsView.svelte`)**
  - Added `searchQuery` state with text input, case-insensitive filter on name+description.
  - `filteredSkills` derived state for reactive filtering.
- ✅ **[DAL-2204] - Feature - Grid/List View Toggle (`SkillsView.svelte`)**
  - `viewMode` toggle ('list' | 'grid'), grid layout with `grid-cols-1 md:grid-cols-2 lg:grid-cols-3`.
  - `{#snippet skillDetail(skill)}` for reuse in both views.
  - Enable/disable button in detail header with toast integration.
- ✅ **[DAL-2205] - Bugfix - Accessibility Warnings**
  - ChatView: replaced `<div>/<label>` with `<fieldset>/<legend>` for Mode selector.
  - CommandPalette: added `tabindex="-1"` to dialog div.
  - Result: `svelte-check` returns 0 errors AND 0 warnings.
- ✅ **[DAL-2206] - Verification - Build & Tests**
  - `npm run build` ✅ (0 warnings), `npm run check` ✅ (0 errors, 0 warnings), `npm run test` ✅ (17/17 pass).

## Sprint 23: Settings Enhancement & Skill Toggle Backend ✅

**Goal:** Add API key management, model presets, test connection, verbose toggle, and skill enable/disable backend.

- ✅ **[DAL-2301] - Feature - Skill Toggle Backend (`skills.rs` + `state.rs`)**
  - Added `disabled_skills: Arc<DashMap<String, bool>>` to `AppState`.
  - `list_skills` now includes `enabled: bool` field per skill.
  - `PUT /api/skills/{name}` handler to toggle skill enabled/disabled.
- ✅ **[DAL-2302] - Feature - Settings Persistence (`persistence.rs`)**
  - Added `save_api_key`/`get_api_key`/`save_verbose`/`get_verbose` functions.
  - All use OS keyring via `keyring` crate for secure storage.
- ✅ **[DAL-2303] - Feature - Settings REST API Expansion (`settings.rs`)**
  - `GET /api/settings` now returns `has_api_key: bool` and `verbose: bool`.
  - `PUT /api/settings` accepts optional `api_key` and `verbose` fields.
  - API key saved via keyring, verbose saved via keyring.
- ✅ **[DAL-2304] - Feature - Test Connection Endpoint (`settings.rs`)**
  - `POST /api/settings/test-connection` sends minimal LLM request, measures latency.
  - Returns `{ success, message, latency_ms }`.
- ✅ **[DAL-2305] - Feature - SettingsView Frontend Rewrite (`SettingsView.svelte`)**
  - Model selector with `PROVIDER_MODELS` presets + "Custom" free-text toggle.
  - Masked API key input (`type="password"`) with show/hide toggle.
  - Test connection button with spinner and inline success/failure indicator.
  - Verbose mode checkbox.
  - Auth status banner reflects `has_api_key` state.
  - Ollama added as provider option.
- ✅ **[DAL-2306] - Testing - New Rust Tests**
  - 4 new tests: `test_update_skill_toggle`, `test_update_nonexistent_skill_returns_404`,
    `test_settings_has_api_key_and_verbose_fields`, `test_test_connection_endpoint_exists`.
  - Total: 19 Rust tests, all pass. 17 frontend tests, all pass.

## Sprint 24: Documentation & Final Polish ✅

**Goal:** Update documentation to cover the web UI, update sprint planning records.

- ✅ **[DAL-2401] - Documentation - Web UI Guide (`docs/guide/web-ui.md`)**
  - Full usage guide: starting the server, pages (Chat, Skills, Reports, Settings), keyboard shortcuts.
- ✅ **[DAL-2402] - Documentation - Web Server Architecture (`docs/architecture/web-server.md`)**
  - Architecture diagram, REST API table, WebSocket protocol spec, AppState design, frontend stack.
- ✅ **[DAL-2403] - Documentation - VitePress Sidebar Update**
  - Added "Web UI" to Guide sidebar, "Web Server" to Architecture sidebar.
- ✅ **[DAL-2404] - Documentation - README Update**
  - Added `dalang web` Quick Start section with usage example.
- ✅ **[DAL-2405] - Documentation - Sprint Planning Update**
  - Updated Sprints 22-24 with actual implementation details and marked ✅.

---

## Sprint 25: GitHub Copilot Provider Integration

**Goal:** Add GitHub Copilot as a new LLM provider using reverse-engineered Copilot CLI endpoints, with full authentication flow and GitHub Models API fallback.

### Completed Tasks

- ✅ **[DAL-2501] - Auth Module - `src/auth/copilot.rs`**
  - Full Copilot authentication: device flow OAuth, keychain extraction, env var, gh CLI extraction
  - Token exchange for short-lived Copilot session tokens (`api.github.com/copilot_internal/v2/token`)
  - Token validation via `api.github.com/copilot_internal/user`
  - Classic PAT (ghp_) rejection per Copilot CLI behavior
  - Persist login with `auth_method=copilot_oauth`, `endpoint_mode=copilot`

- ✅ **[DAL-2502] - Auth Provider Variant**
  - Added `Copilot` variant to `AuthProvider` enum in `src/auth/mod.rs`
  - Accepts "copilot", "github", "github-copilot" as provider strings

- ✅ **[DAL-2503] - LLM Provider - `src/llm/copilot.rs`**
  - `CopilotProvider` with auto-refreshing Copilot session tokens (5-min buffer)
  - Primary: `api.githubcopilot.com/chat/completions` (OpenAI-compatible)
  - Fallback: `models.github.ai/inference` (GitHub Models API with raw PAT)
  - Custom headers: `User-Agent: GithubCopilot/1.155.0`, `editor-version: dalang/0.1.0`
  - Curated model list: claude-sonnet-4.6, claude-opus-4.6, gpt-5.2, gpt-4.1, gemini-3-pro-preview

- ✅ **[DAL-2504] - LLM Factory Integration**
  - Updated `src/llm/mod.rs`: new `copilot` module, default URL/model, `create_provider` dispatch

- ✅ **[DAL-2505] - CLI Login Flow**
  - 4 auth methods: Device Flow OAuth (recommended), Copilot CLI keychain, Env var, Manual PAT
  - Risk disclaimer displayed before login
  - Interactive model selection from curated Copilot model list

- ✅ **[DAL-2506] - Web UI Integration**
  - Added `copilot` to `PROVIDER_MODELS` in `web/src/lib/types.ts`
  - Added "GitHub Copilot" option to provider dropdown in `SettingsView.svelte`
  - Updated `src/web/state.rs` to handle `copilot_oauth` auth method

---

## Sprint 26: Skill Tool Availability Validation & Bug Fixes ✅

**Goal:** Auto-detect and disable skills whose `tool_path` binary is not installed on the system, plus fix critical frontend bugs (chat session navigation, WebSocket scan start).

- ✅ **[DAL-2601] - Feature - Tool Binary Availability Check**
  - Added `tool_available: bool` field to `SkillDefinition` (serde-skipped)
  - Implemented `check_tool_available(tool_path)`: checks absolute path existence, then falls back to `which` command
  - Browser-based skills (`tool_path: null`) always marked as available

- ✅ **[DAL-2602] - Feature - Auto-Disable Unavailable Skills in Engine**
  - Added `load_available_skills()` that filters out skills with missing tool binaries
  - Returns list of unavailable skill names for warning messages
  - Updated all 4 engine call sites (autonomous CLI, interactive CLI, autonomous WS, interactive WS)
  - CLI modes print `[!] N skills disabled (tool not found)` warning
  - WebSocket modes send `EngineEvent::Status` with disabled skill names

- ✅ **[DAL-2603] - Feature - Web API Tool Availability**
  - Added `tool_available` field to `SkillSummary` and `SkillDetail` REST responses
  - `list_skills` handler auto-disables skills where tool binary is missing
  - `get_skill` handler checks tool availability on detail fetch

- ✅ **[DAL-2604] - Feature - Frontend Tool Availability UI**
  - Added `tool_available` to `SkillSummary` and `SkillDetail` TypeScript interfaces
  - Skills grid: "not installed" orange badge (distinct from manual "disabled" red badge)
  - Skill detail: warning banner with install instructions when tool is missing
  - Skill detail: "found"/"missing" status next to tool path
  - Enable/Disable button replaced with "Not Installed" label for unavailable skills

- ✅ **[DAL-2605] - Bugfix - Chat Session Navigation**
  - Fixed clicking existing sessions in sidebar showing setup screen instead of chat history
  - Added `$effect` in ChatView to detect `sessionId` changes and load existing messages
  - Reconnects WebSocket when switching sessions

- ✅ **[DAL-2606] - Bugfix - WebSocket Scan Start Race Condition**
  - Fixed auto-pilot scan not starting from frontend when `max_iter=0` (unlimited)
  - Root cause: `setTimeout(500ms)` — WebSocket might not be OPEN yet when `startScan()` called
  - Added `waitForOpen()` promise, made `startScan()`/`startInteractive()` async

---

## Ringkasan Status

| Sprint | Nama | Status |
|--------|------|--------|
| 1 | Core Foundation & Basic OS Execution | ✅ Done |
| 2 | LLM Integration & Tool Calling Bridge | ✅ Done |
| 3 | CDP Web Crawler Integration | ✅ Done |
| 4 | Defensive Prompting Engine | ✅ Done |
| 5 | Universal Tool Ecosystem Integration | ✅ Done |
| 6 | Expanded Skill Library | ✅ Done |
| 7 | Local Auth & Multi-Provider OAuth | ✅ Done |
| 8 | Autonomous Orchestrator (Auto-Pilot) | ✅ Done |
| 9 | Robust Skill Meta-Library | ✅ Done |
| 10 | Advanced Auto-Pilot & Context Memory | ✅ Done |
| 11 | Addressing TODOs & Code Cleanup | ✅ Done |
| 12 | Interactive Model Selection | ✅ Done |
| 13 | Dynamic Provider Configuration | ✅ Done |
| 14 | Gemini CloudCode Native Provider & OAuth | ✅ Done |
| 15 | Internationalization & Skill Library Expansion | ✅ Done |
| 16 | Enhanced Report Quality & Prompt Engineering | ✅ Done |
| 17 | Robustness & Developer Experience | ✅ Done |
| 18 | Documentation Site Overhaul | ✅ Done |
| 19 | Web UI — Backend Foundation (axum + WebSocket) | ✅ Done |
| 20 | Web UI — Svelte Frontend, WebSocket Chat & REST API | ✅ Done |
| 21 | Web UI — Hardening, Polish & Testing | ✅ Done |
| 22 | Web UI — Chat & Skills Polish | ✅ Done |
| 23 | Settings Enhancement & Skill Toggle Backend | ✅ Done |
| 24 | Documentation & Final Polish | ✅ Done |
| 25 | GitHub Copilot Provider Integration | ✅ Done |
| 26 | Skill Tool Availability Validation & Bug Fixes | ✅ Done |

**Total: 26 Sprint — 26 ✅ Selesai**
