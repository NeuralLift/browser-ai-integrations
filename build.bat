@echo off
setlocal enabledelayedexpansion

echo.
echo ============================================
echo   Browser AI Assistant - Build Script
echo ============================================
echo.

REM Configuration
set VERSION=1.0.0
set DIST_NAME=browser-ai-assistant
set ZIP_NAME=%DIST_NAME%-v%VERSION%-windows.zip

REM Check if cargo is available
where cargo >nul 2>nul
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust/Cargo tidak ditemukan!
    echo Install Rust dari: https://rustup.rs
    pause
    exit /b 1
)

echo [1/6] Building backend (release mode)...
cd backend
cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Build gagal!
    cd ..
    pause
    exit /b 1
)
cd ..
echo [OK] Backend built successfully

echo [2/6] Creating dist folder...
if exist "dist\%DIST_NAME%" rmdir /s /q "dist\%DIST_NAME%"
mkdir "dist\%DIST_NAME%"
echo [OK] Dist folder created

echo [3/6] Copying extension files...
xcopy /E /I /Q "extension" "dist\%DIST_NAME%\extension" >nul
echo [OK] Extension copied

echo [4/6] Copying server binary...
copy "backend\target\release\backend-rig.exe" "dist\%DIST_NAME%\server.exe" >nul
copy "backend\.env.example" "dist\%DIST_NAME%\.env.example" >nul
echo [OK] Server binary copied

echo [5/6] Creating helper scripts...

REM Create start-server.bat
(
echo @echo off
echo title Browser AI Assistant - Server
echo echo ============================================
echo echo   Browser AI Assistant - Local Server
echo echo ============================================
echo echo.
echo if not exist ".env" ^(
echo     echo [ERROR] File .env tidak ditemukan!
echo     echo.
echo     echo Langkah setup:
echo     echo 1. Rename file ".env.example" menjadi ".env"
echo     echo 2. Edit file ".env" dan isi GEMINI_API_KEY dengan API key Anda
echo     echo 3. Dapatkan API key di: https://aistudio.google.com/apikey
echo     echo.
echo     pause
echo     exit /b 1
echo ^)
echo echo [OK] File .env ditemukan
echo echo [INFO] Menjalankan server di http://localhost:3000
echo echo.
echo echo Tekan Ctrl+C untuk menghentikan server
echo echo ============================================
echo echo.
echo server.exe
echo pause
) > "dist\%DIST_NAME%\start-server.bat"

REM Create start-server.sh
(
echo #!/bin/bash
echo echo "============================================"
echo echo "  Browser AI Assistant - Local Server"
echo echo "============================================"
echo echo ""
echo if [ ! -f ".env" ]; then
echo     echo "[ERROR] File .env tidak ditemukan!"
echo     echo "1. Rename '.env.example' menjadi '.env'"
echo     echo "2. Edit dan isi GEMINI_API_KEY"
echo     exit 1
echo fi
echo echo "[OK] File .env ditemukan"
echo echo "[INFO] Server running at http://localhost:3000"
echo ./server
) > "dist\%DIST_NAME%\start-server.sh"

REM Create README.txt
(
echo ============================================
echo   BROWSER AI ASSISTANT - QUICK START
echo ============================================
echo.
echo LANGKAH 1: SETUP API KEY
echo ------------------------
echo 1. Buka https://aistudio.google.com/apikey
echo 2. Buat API Key dan copy
echo 3. Rename ".env.example" menjadi ".env"
echo 4. Edit ".env" dan paste API key
echo.
echo LANGKAH 2: JALANKAN SERVER
echo --------------------------
echo Windows: Double-click "start-server.bat"
echo Mac/Linux: ./start-server.sh
echo.
echo LANGKAH 3: INSTALL EXTENSION
echo ----------------------------
echo 1. Buka chrome://extensions
echo 2. Aktifkan "Developer mode"
echo 3. Klik "Load unpacked"
echo 4. Pilih folder "extension"
echo.
echo LANGKAH 4: GUNAKAN
echo ------------------
echo 1. Buka halaman web apapun
echo 2. Klik icon extension / buka Side Panel
echo 3. Mulai chat dengan AI!
echo.
echo ============================================
echo   TROUBLESHOOTING
echo ============================================
echo.
echo Titik merah ^(disconnected^):
echo - Pastikan server berjalan
echo - Cek error di terminal
echo.
echo API Key Error:
echo - Cek file .env sudah benar
echo - Pastikan API key valid
echo.
echo ============================================
) > "dist\%DIST_NAME%\README.txt"

echo [OK] Helper scripts created

echo [6/6] Creating ZIP archive...
if exist "dist\%ZIP_NAME%" del "dist\%ZIP_NAME%"
powershell -Command "Compress-Archive -Path 'dist\%DIST_NAME%' -DestinationPath 'dist\%ZIP_NAME%' -Force"
echo [OK] ZIP created

echo.
echo ============================================
echo   BUILD COMPLETE!
echo ============================================
echo.
echo Output: dist\%ZIP_NAME%
echo.
for %%A in ("dist\%ZIP_NAME%") do echo Size: %%~zA bytes
echo.
pause
