param([switch]$SelectOnly)
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$env:MEETING_ASSISTANT_ROOT = $projectRoot
function Get-ClassroomFileHash([string]$Path) {
    $stream = [System.IO.File]::OpenRead($Path)
    $algorithm = [System.Security.Cryptography.SHA256]::Create()
    try {
        return [System.BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-', '').ToLowerInvariant()
    } finally {
        $algorithm.Dispose()
        $stream.Dispose()
    }
}
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
$generationPath = Join-Path $projectRoot 'build/native/debug/meetily-generation-fix.exe'
$generationReadyPath = Join-Path $projectRoot 'build/native/debug/meetily-generation-fix.ready.json'
if ((Test-Path -LiteralPath $generationPath -PathType Leaf) -and (Test-Path -LiteralPath $generationReadyPath -PathType Leaf)) {
    try {
        $generationReady = Get-Content -LiteralPath $generationReadyPath -Raw | ConvertFrom-Json
        $generationItem = Get-Item -LiteralPath $generationPath
        $generationHashOk = $true
        if ($generationReady.sha256) {
            $generationHashOk = ((Get-ClassroomFileHash $generationPath) -eq ([string]$generationReady.sha256).ToLowerInvariant())
        }
        if ($generationItem.Length -eq [int64]$generationReady.bytes -and $generationHashOk) {
            $candidates += 'build/native/debug/meetily-generation-fix.exe'
        }
    } catch { Write-Warning ("Generation fix is not ready: " + $_.Exception.Message) }
}
$noCitationsPath = Join-Path $projectRoot 'build/native/debug/meetily-no-citations.exe'
$noCitationsReadyPath = Join-Path $projectRoot 'build/native/debug/meetily-no-citations.ready.json'
if ((Test-Path -LiteralPath $noCitationsPath -PathType Leaf) -and (Test-Path -LiteralPath $noCitationsReadyPath -PathType Leaf)) {
    try {
        $ready = Get-Content -LiteralPath $noCitationsReadyPath -Raw | ConvertFrom-Json
        if ((Get-Item -LiteralPath $noCitationsPath).Length -eq [int64]$ready.bytes -and
            (Get-ClassroomFileHash $noCitationsPath) -eq $ready.sha256) {
            $candidates += 'build/native/debug/meetily-no-citations.exe'
        }
    } catch { Write-Warning ("Citation-free update is not ready: " + $_.Exception.Message) }
}
$promptSettingsPath = Join-Path $projectRoot 'build/native/debug/meetily-prompt-settings.exe'
$promptSettingsReadyPath = Join-Path $projectRoot 'build/native/debug/meetily-prompt-settings.ready.json'
if ((Test-Path -LiteralPath $promptSettingsPath -PathType Leaf) -and (Test-Path -LiteralPath $promptSettingsReadyPath -PathType Leaf)) {
    try {
        $ready = Get-Content -LiteralPath $promptSettingsReadyPath -Raw | ConvertFrom-Json
        if ((Get-Item -LiteralPath $promptSettingsPath).Length -eq [int64]$ready.bytes -and
            (Get-ClassroomFileHash $promptSettingsPath) -eq $ready.sha256) {
            $candidates += 'build/native/debug/meetily-prompt-settings.exe'
        }
    } catch { Write-Warning ("Prompt settings update is not ready: " + $_.Exception.Message) }
}
$brandPath = Join-Path $projectRoot 'build/native/debug/meetily-brand.exe'
$brandReadyPath = Join-Path $projectRoot 'build/native/debug/meetily-brand.ready.json'
if ((Test-Path -LiteralPath $brandPath -PathType Leaf) -and (Test-Path -LiteralPath $brandReadyPath -PathType Leaf)) {
    try {
        $ready = Get-Content -LiteralPath $brandReadyPath -Raw | ConvertFrom-Json
        if ((Get-Item -LiteralPath $brandPath).Length -eq [int64]$ready.bytes -and
            (Get-ClassroomFileHash $brandPath) -eq $ready.sha256) {
            $candidates += 'build/native/debug/meetily-brand.exe'
        }
    } catch { Write-Warning ("Brand update is not ready: " + $_.Exception.Message) }
}
$available = foreach ($relative in $candidates) {
    $candidatePath = Join-Path $projectRoot $relative
    if (Test-Path -LiteralPath $candidatePath -PathType Leaf) { Get-Item -LiteralPath $candidatePath }
}
$selected = $available | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1
if ($null -eq $selected) { throw 'No completed desktop build found. Run tools/Build-Classroom.ps1 first.' }
if ($SelectOnly) { Write-Output $selected.FullName; exit 0 }
Start-Process -FilePath $selected.FullName -WorkingDirectory $projectRoot
