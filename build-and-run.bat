@echo off
REM Horizon Offline Launcher - Windows Build Script
REM Run this after installing Rust (https://rustup.rs/)

echo ============================================
echo Horizon Offline Launcher - Build & Run
echo ============================================
echo.

REM Check if Rust is installed
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo ERROR: Rust/Cargo not found in PATH
    echo Please install Rust from https://rustup.rs/
    echo Then restart this terminal and try again.
    pause
    exit /b 1
)

echo Rust found:
cargo --version
echo.

REM Set paths
set HORIZON_PATH=..\horizon\horizon-main
set LAUNCHER_PATH=%~dp0

echo Building offline launcher...
cd /d "%LAUNCHER_PATH%"
cargo build --release
if %errorlevel% neq 0 (
    echo Build failed!
    pause
    exit /b 1
)

echo.
echo Offline launcher built successfully!
echo Binary: %LAUNCHER_PATH%target\release\horizon-offline-launcher.exe
echo.

echo ============================================
echo Next steps:
echo ============================================
echo.
echo 1. Clone Horizon repository (if not done):
echo    git clone https://github.com/horizon-rust/horizon.git
echo.
echo 2. Build Horizon client:
echo    %LAUNCHER_PATH%target\release\horizon-offline-launcher.exe build --horizon-path %HORIZON_PATH%
echo.
echo 3. Prepare resources (requires JDK 25 and server.jar):
echo    %LAUNCHER_PATH%target\release\horizon-offline-launcher.exe prepare --horizon-path %HORIZON_PATH% --server-jar "C:\path\to\server.jar" --sounds
echo.
echo 4. Launch offline:
echo    %LAUNCHER_PATH%target\release\horizon-offline-launcher.exe launch --horizon-path %HORIZON_PATH%
echo.
echo 5. Check status:
echo    %LAUNCHER_PATH%target\release\horizon-offline-launcher.exe check --horizon-path %HORIZON_PATH%
echo.

pause