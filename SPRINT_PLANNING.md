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
- [DAL-903] - Documentation - Network Protocols Deep Dive
  - Tambahkan skill untuk service spesifik: `hydra_bruteforce`, `smbclient_enum`, `snmpwalk_gather`.

## Sprint 10: Advanced Auto-Pilot & Persistent Context Memory

**Goal:** Memaksimalkan fleksibilitas agen otonom dengan mengizinkan AI membentuk argumen command sendiri di atas path tool dasar, serta mengimplementasikan _Persistent Context_ (memori jangka panjang) agar agen tidak kehilangan orientasi logis di tengah operasi multi-tahap.

- **[DAL-1001] - Feature - Dynamic Argument Injection (Free-form Args)**
  - Modifikasi skema _Meta-Tool_ `execute_skill` agar selain menerima `skill_name`, ia juga menerima array string tambahan opsional `custom_args`.
  - Di layer _executor_, gabungkan argumen statis dari `.md` (misal wajib `xsstrike`) dengan argumen bentukan AI (misal `--crawl -l 3 --skip-dom`).
  - Terapkan validasi ketat/blacklist (misal blokir arg `&&` atau `;` untuk mencegah AI melakukan command injection pada OS host).
- **[DAL-1002] - Feature - Persistent Context Memory Engine**
  - Buat representasi memori (seperti `ContextManager`) di `src/core/memory.rs` yang menyimpan jejak observasi ringkas (bukan seluruh raw JSON/HTML) yang bisa bertahan antar-sesi atau antar-loop.
  - LLM tidak hanya menyuap ulang seluruh _history_ chat, melainkan meringkas observasi sebelumnya ("Saya baru saja port scan, menemukan X dan Y") dan menaruhnya di slot memori _System Prompt_.
- **[DAL-1003] - Feature - Project README & Documentation Polish**
  - Buat `README.md` utama berbahasa Inggris ringkas dan formal yang menjelaskan apa itu Dalang, cara instalasi, dependensi, dan showcase perintah `dalang scan --auto`.

## Sprint 11: Addressing TODOs & Code Cleanup

**Goal:** Membersihkan dan melengkapi fungsionalitas yang tertunda (_technical debt_) berupa komentar `// TODO` di dalam _codebase_, sehingga framework menjadi lebih solid dan lengkap sebelum rilis 1.0.

- **[DAL-1101] - Cleanup - Implement Explicit Tools Definition (`src/llm/openai.rs`)**
  - Saat ini _tool calling_ dikelola murni via JSON di _System Prompt_.
  - Ekstrak abstrak `SkillDefinition` menjadi skema _Native Tool Calling_ API milik OpenAI/Anthropic/Gemini (menggunakan parameter `tools` pada JSON payload HTTP request) agar deteksi _tool_ oleh LLM lebih akurat dan mengurangi beban token di _System Prompt_.
- **[DAL-1102] - Feature - Implement `init` Command (`src/main.rs`)**
  - Gantikan `// TODO: Implement init logic` dengan logika pembuatan _scaffolding_ direktori awal.
  - Perintah `dalang init` akan secara otomatis membuat folder `skills/` dan men-_generate_ file `skills/example-nmap.md` berisi template dasar, sehingga pengguna baru punya titik awal (_starting point_) yang terstandardisasi.
- **[DAL-1103] - Feature - Implement `interact` Command (`src/main.rs`)**
  - Gantikan `// TODO: Implement interactive logic` dengan sebuah REPL (_Read-Eval-Print Loop_) interaktif.
  - Alih-alih mengeksekusi satu command `scan` lalu selesai, mode `interact` memungkinkan user me-maintain sebuah sesi obrolan (chat) dengan _DalangEngine_. Pengguna bisa bertanya, _"Coba kamu check port 80"_, lalu Dalang akan merespons dengan tool, dan pengguna bisa merespons lagi. Ini mirip dengan _Auto-Pilot_ tetapi bersifat _Human-in-the-Loop_ (HITL).

## Sprint 12: Interactive Model Selection

**Goal:** Memberikan pengalaman interaktif bagi pengguna untuk memilih model AI secara dinamis setelah mereka berhasil melakukan autentikasi (login), baik melalui API Key maupun OAuth.

- **[DAL-1201] - Feature - Implement Provider Model Fetching (`src/llm/` & Provider API)**
  - Implementasikan endpoint/metode untuk mengambil daftar model yang didukung secara langsung dari API Provider (misalnya endpoint `/v1/models` untuk list model) menggunakan token/auth yang baru saja didapatkan.
- **[DAL-1202] - UX - Interactive CLI Prompt (`src/main.rs`)**
  - Integrasikan _crate_ CLI interaktif (seperti `dialoguer` atau `inquire`) ke dalam alur `dalang login`.
  - Setelah token berhasil diperoleh dan divalidasi, program tidak langsung _exit_, melainkan menampilkan _loading spinner_ saat menarik daftar model, lalu menampilkan antarmuka pilihan _dropdown_ di terminal (contoh: `gemini-1.5-pro`, `gemini-1.5-flash`, dll) untuk dipilih oleh _user_.
- **[DAL-1203] - Feature - Persist User Model Preference (`src/auth/persistence.rs` atau Config)**
  - Setelah pengguna memilih model, simpan preferensi ini secara lokal (misalnya di _keyring_ atau _config file_ default di dalam direktori `~/.dalang/`).
  - Ubah perilaku default eksekusi CLI (seperti command `scan` dan `interact`) agar membaca dari preferensi tersimpan ini jika _environment variable_ `LLM_MODEL` tidak disediakan.
