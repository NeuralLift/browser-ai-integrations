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

### 1. Setup Backend

Backend ditulis menggunakan Rust dan framework Axum.

1. Masuk ke direktori backend:
   ```bash
   cd backend
   ```

2. Buat file `.env` di dalam folder `backend` dan isi dengan API Key Anda:
   ```env
   GOOGLE_API_KEY=api_key_gemini_anda_disini
   ```

3. Jalankan server:
   
   Mode Development:
   ```bash
   cargo run
   ```
   
   Mode Release (Lebih Cepat & Recommended):
   ```bash
   ./start.sh
   # atau
   cd backend && cargo run --release
   ```
   Server akan berjalan di `http://localhost:3000`.

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
├── backend/           # Source code server Rust
│   ├── src/
│   │   ├── ai.rs      # Logika integrasi Gemini API
│   │   ├── privacy.rs # Sanitasi data pengguna
│   │   ├── ws.rs      # Handler WebSocket
│   │   └── main.rs    # Entry point & server config
│   └── Cargo.toml     # Dependencies Rust
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

## Lisensi

Proyek ini dibuat untuk tujuan pembelajaran dan pengembangan. Silakan gunakan dan modifikasi sesuai kebutuhan.
