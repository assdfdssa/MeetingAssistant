@echo off
setlocal
set "MEETING_ASSISTANT_ROOT=%~dp0"
set "CLASSROOM_LAUNCHER=%~dp0tools\Start-Classroom.ps1"
if not exist "%CLASSROOM_LAUNCHER%" (
  echo Startup script not found: "%CLASSROOM_LAUNCHER%"
  pause
  exit /b 1
)
"%SystemRoot%\System32\WindowsPowerShell\v1.0\powershell.exe" -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%CLASSROOM_LAUNCHER%" %*
set "CLASSROOM_EXIT_CODE=%ERRORLEVEL%"
if not "%CLASSROOM_EXIT_CODE%"=="0" (
  echo.
  echo MeetingAssistant could not start. Error code: %CLASSROOM_EXIT_CODE%
  echo Please keep this window open to read the error above.
  pause
)
exit /b %CLASSROOM_EXIT_CODE%
