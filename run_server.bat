@echo off
title Code Blue - Dedicated Server
echo ===================================================
echo   Starting Code Blue Dedicated Multiplayer Server
echo ===================================================
echo Running cb_server on port 5000 (120Hz UDP)...
cargo run --bin cb_server
pause
