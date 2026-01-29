# Integration Testing Checklist - Backend Rig

Gunakan checklist ini untuk memverifikasi bahwa `backend-rig` berfungsi dengan benar dan kompatibel dengan Browser Extension.

## 1. Persiapan

- [ ] Hentikan backend lama jika masih berjalan (Ctrl+C di terminal backend).
- [ ] Pastikan file `.env` di dalam folder `backend-rig` sudah berisi `GOOGLE_API_KEY`.

## 2. Menjalankan Backend Rig

- [ ] Buka terminal baru.
- [ ] Masuk ke direktori: `cd backend-rig`
- [ ] Jalankan server: `cargo run`
- [ ] Verifikasi log menunjukkan: `Server running on http://0.0.0.0:3000`

## 3. Verifikasi Endpoint (Automated/CLI)

Buka terminal lain dan jalankan perintah berikut:

- [ ] **Health Check**:

  ```bash
  curl -s http://localhost:3000/health | jq
  ```

  _Ekspektasi: `{"status":"ok"}`_

- [ ] **Text Chat**:

  ```bash
  curl -s http://localhost:3000/api/chat \
    -H "Content-Type: application/json" \
    -d '{"message":"Halo"}' | jq
  ```

  _Ekspektasi: Response JSON berisi jawaban dari AI._

- [ ] **Chat dengan Custom Instruction**:
  ```bash
  curl -s http://localhost:3000/api/chat \
    -H "Content-Type: application/json" \
    -d '{"message":"Test","custom_instruction":"Reply in English"}' | jq
  ```
  _Ekspektasi: Response dalam Bahasa Inggris._

## 4. Verifikasi dengan Extension (Manual)

- [ ] Buka Google Chrome.
- [ ] Pastikan Extension sudah terpasang dan aktif.
- [ ] Buka **Side Panel** Chrome dan pilih "Browser AI Assistant".
- [ ] Kirim pesan teks: "Halo, apa kabar?"
  - [ ] Verifikasi: Muncul jawaban dalam Bahasa Indonesia.
- [ ] Ubah **Custom Instruction** di pengaturan extension (jika ada) atau test fitur instruksi khusus.
- [ ] Coba fitur **Screenshot/Image Upload**:
  - [ ] Ambil screenshot halaman atau upload gambar.
  - [ ] Kirim ke AI.
  - [ ] Verifikasi: AI dapat mendeskripsikan isi gambar tersebut.

## 5. Pengujian Error Handling

- [ ] Hentikan server `backend-rig` (Ctrl+C).
- [ ] Coba kirim pesan dari extension.
  - [ ] Verifikasi: Extension menampilkan pesan error yang sesuai (misal: "Connection failed" atau "Server unreachable").
