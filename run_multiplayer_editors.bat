@echo off
title Code Blue - Multiplayer Collaborative Editor Demo (2 Editors + 1 Server)
echo =========================================================================
echo   Starting Code Blue Multiplayer Collaborative Session
echo   - 1 Dedicated Server
echo   - 2 Independent Editor Instances (Auto-Connecting with Live Sync)
echo =========================================================================

echo 1. Starting Dedicated Server in background...
start "Code Blue Server" cmd /k "cargo run --bin cb_server"

echo Waiting 4 seconds for server initialization...
timeout /t 4 /nobreak >nul

echo 2. Launching Editor Client #1...
start "Code Blue Editor 1" cmd /k "cargo run --bin cb_editor"

echo 3. Launching Editor Client #2...
start "Code Blue Editor 2" cmd /k "cargo run --bin cb_editor"

echo All processes launched! You can now collaborate across both editor windows in real-time.
