@echo off
echo ========================================
echo   DPolG Buchungssystem - Windows Build
echo ========================================
echo.

echo [1/3] Installing Node.js dependencies...
call npm install --legacy-peer-deps
if %errorlevel% neq 0 (
    echo ERROR: npm install failed!
    pause
    exit /b 1
)

echo.
echo [2/3] Cleaning Rust build cache...
cd src-tauri
call cargo clean
cd ..

echo.
echo [3/3] Building Windows application...
call npm run tauri build
if %errorlevel% neq 0 (
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo.
echo ========================================
echo   BUILD SUCCESSFUL!
echo ========================================
echo.
echo Find your installers at:
echo   src-tauri\target\release\bundle\msi\
echo   src-tauri\target\release\bundle\nsis\
echo.
pause
