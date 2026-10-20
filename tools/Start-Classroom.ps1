param([switch]$SelectOnly)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$env:MEETING_ASSISTANT_ROOT = $projectRoot
$candidates = @('build/native/release/meetily.exe', 'build/native/debug/meetily.exe')
$pendingPath = Join-Path $projectRoot 'build/native/debug/meetily-summary-fix.exe'
$readyPath = Join-Path $projectRoot 'build/native/debug/meetily-summary-fix.ready.json'
if ((Test-Path -LiteralPath $pendingPath -PathType Leaf) -and (Test-Path -LiteralPath $readyPath -PathType Leaf)) {
    try {
        $ready = Get-Content -LiteralPath $readyPath -Raw | ConvertFrom-Json
        if ((Get-Item -LiteralPath $pendingPath).Length -eq $ready.bytes) {
            $candidates += 'build/native/debug/meetily-summary-fix.exe'
        }
    } catch { Write-Warning 'Pending update is not ready; using a completed build.' }
}
$available = foreach ($relative in $candidates) {
    $candidatePath = Join-Path $projectRoot $relative
    if (Test-Path -LiteralPath $candidatePath -PathType Leaf) { Get-Item -LiteralPath $candidatePath }
}
$selected = $available | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1
if ($null -eq $selected) { throw 'No completed desktop build found. Run tools/Build-Classroom.ps1 first.' }
if ($SelectOnly) { Write-Output $selected.FullName; exit 0 }
Start-Process -FilePath $selected.FullName -WorkingDirectory $projectRoot
