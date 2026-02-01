# Browser AI Assistant

![Browser AI Assistant Preview](preview.png)

Browser AI Assistant adalah proyek integrasi AI dengan browser yang memungkinkan Anda untuk berinteraksi dengan asisten cerdas yang sadar akan konteks halaman web yang sedang Anda buka.

Sistem ini terdiri dari dua komponen utama:

1. **Backend (Rust)**: Server yang menangani logika AI, memproses konteks halaman, dan berkomunikasi dengan API Google Gemini.
2. **Browser Extension**: Ekstensi Chrome yang menangkap konten halaman dan screenshot untuk dikirim ke backend.

## Fitur Utama

- 🧠 **Konteks Real-time**: Asisten mengetahui URL, judul, dan isi teks dari halaman yang sedang Anda baca.
- 📸 **Analisis Visual**: Mengirimkan screenshot halaman ke AI untuk analisis visual (layout, gambar, grafik).
- 💬 **Chat Interaktif**: Antarmuka chat melalui Side Panel browser.
- 🔒 **Privasi**: Data diproses secara lokal di server backend Anda sendiri sebelum dikirim ke API AI.
- 🔎 **Google Search**: Kemampuan built-in untuk mencari informasi terkini dari internet.

## Persyaratan Sistem

- **Rust** (terbaru) untuk menjalankan backend.
- **Google Chrome** atau browser berbasis Chromium (Edge, Brave, dll).
- **Google API Key** (Gemini API).

## Cara Instalasi

### 1. Setup Backend (Pilih salah satu)

#### A. Backend Standar (Recommended)

Backend utama dengan fitur lengkap (WebSocket, Privacy, dll).

1. Masuk ke direktori backend: `cd backend`
2. Ikuti instruksi di folder tersebut.

#### B. Backend Rig (Experimental/MVP)

Backend alternatif menggunakan library Rig untuk integrasi AI yang lebih simpel.

1. Masuk ke direktori: `cd backend-rig`
2. Salin `.env.example` menjadi `.env` dan isi `GEMINI_API_KEY`.
3. Jalankan server: `cargo run`.
4. Lihat `backend-rig/TESTING_CHECKLIST.md` untuk panduan pengujian.

Server akan berjalan di `http://localhost:3000`. Pastikan hanya satu backend yang berjalan di port ini.

### 2. Setup Extension

1. Buka Google Chrome dan navigasi ke `chrome://extensions`.
2. Aktifkan **Developer mode** di pojok kanan atas.
3. Klik tombol **Load unpacked**.
4. Pilih folder `extension` dari proyek ini.
5. Ekstensi akan muncul di browser Anda.

## Cara Penggunaan

1. Pastikan backend server sedang berjalan (`cargo run`).
2. Buka halaman web apapun yang ingin Anda analisis.
3. Klik ikon ekstensi atau buka **Side Panel** Chrome dan pilih "Browser AI Assistant".
4. Mulai percakapan! Anda bisa bertanya tentang ringkasan artikel, penjelasan gambar, atau info terkait lainnya.

## Struktur Proyek

```
.
├── backend/           # Source code server Rust (Standard)
│   ├── src/
│   │   ├── ai.rs      # Logika integrasi Gemini API
│   │   ├── privacy.rs # Sanitasi data pengguna
│   │   ├── ws.rs      # Handler WebSocket
│   │   └── main.rs    # Entry point & server config
│   └── Cargo.toml     # Dependencies Rust
│
├── backend-rig/       # Alternatif backend menggunakan Rig (Experimental)
│   ├── src/
│   │   ├── main.rs    # Entry point & handlers
│   │   └── models.rs  # Data models
│   └── Cargo.toml     # Dependencies Rig & Axum
│
└── extension/         # Source code ekstensi browser
    ├── manifest.json  # Konfigurasi ekstensi V3
    ├── background.js  # Service worker
    ├── content.js     # Script yang berjalan di halaman web
    ├── sidepanel.html # UI Chat
    ├── offscreen.html # Background processing (DOM parser)
    └── popup.html     # Extension popup UI
```

## Teknologi

- **Backend**: Rust, Axum, Tokio, Reqwest, Serde.
- **AI Model**: Google Gemini 2.5 Flash.
- **Frontend**: HTML, JavaScript (Vanilla), Chrome Extension API (Manifest V3).

## Panduan Kontribusi

Terima kasih telah tertarik untuk berkontribusi pada **Browser AI Assistant**! Kami menyambut segala bentuk kontribusi, mulai dari laporan bug, saran fitur, hingga perbaikan kode.

### Alur Kerja Kontribusi

1.  **Fork** repositori ini ke akun GitHub Anda.
2.  **Clone** hasil fork tersebut ke mesin lokal:
    ```bash
    git clone https://github.com/USERNAME/browser-ai-integrations.git
    ```
3.  **Buat Branch** baru untuk fitur atau perbaikan Anda:
    ```bash
    git checkout -b feat/nama-fitur-anda
    # atau
    git checkout -b fix/deskripsi-perbaikan
    ```
4.  **Lakukan Perubahan** dan pastikan kode Anda mengikuti standar kualitas kami.
5.  **Commit** perubahan Anda dengan pesan yang deskriptif. Kami menyarankan format [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
6.  **Push** ke repositori fork Anda:
    ```bash
    git push origin HEAD
    ```
7.  **Buka Pull Request** dari branch Anda ke branch `master` kami.

### Standar Pengembangan

Untuk menjaga konsistensi dan kualitas kode, kami menerapkan standar berikut:

#### 🛠️ Alat Pengembangan

Kami menggunakan **Node.js** untuk mengelola alat pemformatan dan git hooks.

- Pastikan Anda telah menjalankan `npm install` di root direktori untuk menginstal **Husky**, **Lint-staged**, dan **Prettier**.

#### 🎨 Pemformatan Kode

Kami mewajibkan semua file diformat dengan benar sebelum dikirimkan.

- **Frontend (Extension)**: Menggunakan Prettier (indentasi 2 spasi).
- **Backend (Rust)**: Menggunakan Rustfmt (indentasi 4 spasi, Edisi 2024).
- **Otomatisasi**: Git hooks (Husky) akan secara otomatis memformat file yang di-stage saat Anda melakukan `git commit`.

Jika Anda ingin menjalankan pemformatan secara manual:

```bash
npm run format       # Format file Frontend (JS, HTML, CSS, JSON)
npm run format:rust  # Format file Backend (Rust)
```

#### 💻 Pengaturan Editor (VS Code)

Jika Anda menggunakan VS Code, proyek ini sudah dilengkapi dengan pengaturan workspace:

- **Format on Save**: Aktif secara otomatis.
- **Ekstensi Rekomendasi**: Pastikan untuk menginstal ekstensi yang disarankan (Prettier, Rust Analyzer, EditorConfig) saat pertama kali membuka folder ini.

### Melaporkan Masalah

Jika Anda menemukan bug atau memiliki ide fitur, silakan buka **Issue** baru dengan informasi berikut:

- Deskripsi singkat tentang masalah/fitur.
- Langkah-langkah untuk mereproduksi (untuk bug).
- Ekspektasi perilaku.
- Screenshot (jika ada).

---

## Lisensi

Proyek ini dibuat untuk tujuan pembelajaran dan pengembangan. Silakan gunakan dan modifikasi sesuai kebutuhan.
