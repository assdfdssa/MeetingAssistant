@echo off
chcp 65001 >nul
set "MEETING_ASSISTANT_ROOT=%~dp0"
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0tools\Start-Classroom.ps1"
if errorlevel 1 (
  echo 启动失败，请参阅 docs\开发进度与验证.md。
  pause
)
