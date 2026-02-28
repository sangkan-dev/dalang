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

**Total: 18 Sprint — Semua ✅ Selesai**
