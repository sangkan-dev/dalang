---
name: tooling-setup
description: Setup tooling untuk AI Agent
---

# Dalang AI Agent Skills & Tooling Setup

Untuk mendevelop "Dalang" secara efektif dan terstruktur sesuai PRD dan Dev Rules, berikut adalah konfigurasi Skill dan Tooling utama yang saya (AI Agent) perlukan selama fase eksekusi:

## 1. File System Access (Code Generation & Refactoring)

- **Kemampuan:** Melakukan operasi read, write, dan replace pada file lokal (`src/`, `skills/`, `.github/`, dll).
- **Tooling Integrasi:**
  - File reader (`view_file`, `list_dir`, `grep_search`, `find_by_name`) untuk memetakan arsitektur dan mencari dependensi.
  - File writer (`write_to_file`, `replace_file_content`, `multi_replace_file_content`) untuk merealisasikan pembuatan modul core, llm interface, cdp interop, dan parser.

## 2. Terminal Execution (Linting, Building & Testing)

- **Kemampuan:** Menjalankan perintah OS di background untuk memverifikasi kode Rust yang telah ditulis tanpa harus menunggu pengguna.
- **Tooling Integrasi (`run_command`, `command_status`, `send_command_input`):**
  - **Kompilasi:** `cargo check` dan `cargo build` untuk memastikan tidak ada strict type error dan lifetime issue.
  - **Linting:** `cargo clippy -- -D warnings` untuk menegakkan idiomatic Rust code sesuai `DEV_RULES.md`.
  - **Testing:** `cargo test` (unit tests dan integration tests) untuk memvalidasi parser Markdown dan interaksi module execution.
  - **Formatting:** `cargo fmt` memastikan format kode terstandarisasi.

## 3. Web Research & Documentation Reading

- **Kemampuan:** Membaca dokumentasi dari external library/crate terbaru yang mungkin berubah secara dinamis, untuk menghindari halusinasi API (misalnya perubahan versi pada framework CDP atau LLM).
- **Tooling Integrasi (`search_web`, `read_url_content`):**
  - Mengakses documentation.rs (docs.rs).
  - Meresearch issue/bug di repository GitHub (seperti pada library `chromiumoxide`, `reqwest`, atau `pulldown-cmark`).

## 4. Environment & Dependency Management

- **Kemampuan:** Menavigasi ekosistem Rust.
- **Tugas Spesifik:**
  - Melakukan instalasi package / crate (contoh: `cargo add tokio -F full`, `cargo add anyhow`, `cargo add serde -F derive`).
  - Mengeksekusi command utilitas pendukung seperti `nmap` dummy local untuk menguji integrasi Modul Executor pada fase akhir.
