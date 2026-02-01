# Backend Rig (Experimental)

Rust backend menggunakan Rig library untuk Browser AI Assistant. Ini adalah versi alternatif dari backend utama yang menggunakan abstraksi Rig untuk interaksi AI.

> **Note:** Proyek ini masih dalam tahap MVP/Experimental. Tidak mendukung WebSocket, fitur privasi, atau manajemen konteks halaman web yang kompleks saat ini.

## Persyaratan Sistem

- **Rust** (terbaru)
- **Google API Key** (Gemini API)

## Cara Instalasi

1. Masuk ke direktori backend-rig:

   ```bash
   cd backend-rig
   ```

2. Buat file `.env` dari contoh:

   ```bash
   cp .env.example .env
   ```

3. Edit file `.env` dan masukkan API Key Anda:

   ```env
   GEMINI_API_KEY=your_gemini_api_key_here
   ```

4. Jalankan server:

   ```bash
   cargo run
   ```

   Server akan berjalan di `http://localhost:3000`.

⚠️ **PENTING:** Pastikan untuk menghentikan backend lama jika sedang berjalan, karena keduanya menggunakan port **3000**.

## API Endpoints

### 1. Health Check

Memastikan server berjalan dengan baik.

- **URL:** `GET /health`
- **Response:**
  ```json
  {
    "status": "ok"
  }
  ```

### 2. Chat API

Mengirim pesan ke AI Gemini.

- **URL:** `POST /api/chat`
- **Request Body:**
  ```json
  {
    "message": "Halo, siapa kamu?",
    "custom_instruction": "Jawab dengan singkat",
    "image": null
  }
  ```
- **Response Body:**
  ```json
  {
    "response": "Saya adalah asisten AI..."
  }
  ```

## Pengujian dengan Curl

Anda dapat mengetes API secara manual menggunakan curl:

```bash
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "Tes koneksi", "custom_instruction": null, "image": null}'
```
