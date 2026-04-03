@echo off
echo ========================================
echo    Cyber-Kube - Starting All Services
echo ========================================
echo.

cd /d "%~dp0"

echo [1/3] Starting API Go on port 8081...
start "Cyber-Kube API Go" cmd /k "cd api-go && echo API Go Server && go run ."
timeout /t 2 /nobreak >nul

echo [2/3] Starting Engine Rust on port 7000...
start "Cyber-Kube Engine Rust" cmd /k "cd .. && echo Engine Rust && cargo run --bin cyber-kube-engine"
timeout /t 2 /nobreak >nul

echo [3/3] Starting Dashboard on port 3000...
start "Cyber-Kube Dashboard" cmd /k "cd dashboard && echo Dashboard React && npm run dev"

echo.
echo ========================================
echo    All Services Started!
echo ========================================
echo.
echo 📊 Dashboard: http://localhost:3000
echo 🔵 API:       http://localhost:8081
echo 🦀 Engine:    http://localhost:7000
echo.
echo Press any key to close this window...
echo (Services will keep running in their own windows)
pause >nul
