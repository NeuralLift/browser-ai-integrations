#!/bin/bash
set -e

echo ""
echo "============================================"
echo "  Browser AI Assistant - Build Script"
echo "============================================"
echo ""

# Configuration
VERSION="1.0.0"
DIST_NAME="browser-ai-assistant"

# Detect OS
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
    BINARY_NAME="server"
    ZIP_NAME="${DIST_NAME}-v${VERSION}-macos.zip"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
    BINARY_NAME="server"
    ZIP_NAME="${DIST_NAME}-v${VERSION}-linux.zip"
else
    OS="windows"
    BINARY_NAME="server.exe"
    ZIP_NAME="${DIST_NAME}-v${VERSION}-windows.zip"
fi

echo "[INFO] Building for: $OS"
echo ""

# Check cargo
if ! command -v cargo &> /dev/null; then
    echo "[ERROR] Rust/Cargo not found!"
    echo "Install from: https://rustup.rs"
    exit 1
fi

echo "[1/6] Building backend (release mode)..."
cd backend
cargo build --release
cd ..
echo "[OK] Backend built successfully"

echo "[2/6] Creating dist folder..."
rm -rf "dist/${DIST_NAME}"
mkdir -p "dist/${DIST_NAME}"
echo "[OK] Dist folder created"

echo "[3/6] Copying extension files..."
cp -r extension "dist/${DIST_NAME}/"
echo "[OK] Extension copied"

echo "[4/6] Copying server binary..."
if [[ "$OS" == "windows" ]]; then
    cp "backend/target/release/backend-rig.exe" "dist/${DIST_NAME}/${BINARY_NAME}"
else
    cp "backend/target/release/backend-rig" "dist/${DIST_NAME}/${BINARY_NAME}"
    chmod +x "dist/${DIST_NAME}/${BINARY_NAME}"
fi
cp "backend/.env.example" "dist/${DIST_NAME}/.env.example"
echo "[OK] Server binary copied"

echo "[5/6] Creating helper scripts..."

# Create start-server.bat (Windows)
cat > "dist/${DIST_NAME}/start-server.bat" << 'BATEOF'
@echo off
title Browser AI Assistant - Server
echo ============================================
echo   Browser AI Assistant - Local Server
echo ============================================
echo.
if not exist ".env" (
    echo [ERROR] File .env tidak ditemukan!
    echo.
    echo 1. Rename ".env.example" menjadi ".env"
    echo 2. Edit dan isi GEMINI_API_KEY
    echo 3. API Key: https://aistudio.google.com/apikey
    echo.
    pause
    exit /b 1
)
echo [OK] File .env ditemukan
echo [INFO] Server running at http://localhost:3000
echo.
server.exe
pause
BATEOF

# Create start-server.sh (Unix)
cat > "dist/${DIST_NAME}/start-server.sh" << 'SHEOF'
#!/bin/bash
echo "============================================"
echo "  Browser AI Assistant - Local Server"
echo "============================================"
echo ""
if [ ! -f ".env" ]; then
    echo "[ERROR] File .env tidak ditemukan!"
    echo ""
    echo "1. Rename '.env.example' menjadi '.env'"
    echo "2. Edit dan isi GEMINI_API_KEY"
    echo "3. API Key: https://aistudio.google.com/apikey"
    exit 1
fi
echo "[OK] File .env ditemukan"
echo "[INFO] Server running at http://localhost:3000"
echo ""
./server
SHEOF
chmod +x "dist/${DIST_NAME}/start-server.sh"

# Create README
cat > "dist/${DIST_NAME}/README.txt" << 'READMEEOF'
============================================
  BROWSER AI ASSISTANT - QUICK START
============================================

LANGKAH 1: SETUP API KEY
------------------------
1. Buka https://aistudio.google.com/apikey
2. Buat API Key dan copy
3. Rename ".env.example" menjadi ".env"
4. Edit ".env" dan paste API key

LANGKAH 2: JALANKAN SERVER
--------------------------
Windows: Double-click "start-server.bat"
Mac/Linux: ./start-server.sh

LANGKAH 3: INSTALL EXTENSION
----------------------------
1. Buka chrome://extensions
2. Aktifkan "Developer mode"
3. Klik "Load unpacked"
4. Pilih folder "extension"

LANGKAH 4: GUNAKAN
------------------
1. Buka halaman web apapun
2. Klik icon extension / buka Side Panel
3. Mulai chat dengan AI!

============================================
  TROUBLESHOOTING
============================================

Titik merah (disconnected):
- Pastikan server berjalan
- Cek error di terminal

API Key Error:
- Cek file .env sudah benar
- Pastikan API key valid

============================================
READMEEOF

echo "[OK] Helper scripts created"

echo "[6/6] Creating ZIP archive..."
rm -f "dist/${ZIP_NAME}"
cd dist
zip -r "${ZIP_NAME}" "${DIST_NAME}" -q
cd ..
echo "[OK] ZIP created"

echo ""
echo "============================================"
echo "  BUILD COMPLETE!"
echo "============================================"
echo ""
echo "Output: dist/${ZIP_NAME}"
echo "Size: $(du -h "dist/${ZIP_NAME}" | cut -f1)"
echo ""
