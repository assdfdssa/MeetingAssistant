@echo off
chcp 65001 >nul
cd /d "%~dp0"
set NEXT_TELEMETRY_DISABLED=1
echo 浏览器预览不录音、不调用 API。请点击“查看课堂演示”体验。
start "" "http://localhost:3118"
"%~dp0tools\node\node.exe" "%~dp0tools\preview-server.cjs"
pause
