param([switch]$Release)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'Build-Environment.ps1')
$projectRoot=Split-Path $PSScriptRoot -Parent
Push-Location (Join-Path $projectRoot 'meetily/frontend')
try {
    & node scripts/build-classroom.cjs
    if($LASTEXITCODE -ne 0){throw '前端构建失败'}
    Push-Location (Join-Path $projectRoot 'meetily')
    try {
        # The helper must use the release CRT; debug builds need unavailable debug DLLs.
        & cargo build -p llama-helper --release --locked -j 4
        if($LASTEXITCODE -ne 0){throw '本地摘要辅助程序构建失败'}
        $helperSource=Join-Path $env:CARGO_TARGET_DIR 'release/llama-helper.exe'
        $flavor=if($Release){'release'}else{'debug'}
        $nativeDir=Join-Path $env:CARGO_TARGET_DIR $flavor
        New-Item -ItemType Directory -Path $nativeDir -Force | Out-Null
        Copy-Item -LiteralPath $helperSource -Destination (Join-Path $projectRoot 'meetily/frontend/src-tauri/binaries/llama-helper-x86_64-pc-windows-msvc.exe') -Force
        Copy-Item -LiteralPath $helperSource -Destination (Join-Path $nativeDir 'llama-helper-x86_64-pc-windows-msvc.exe') -Force
        if(-not $Release){Copy-Item -LiteralPath $helperSource -Destination (Join-Path $nativeDir 'llama-helper.exe') -Force}
        $cargoArgs=@('build','-p','meetily','--features','custom-protocol','--locked','-j','6')
        if($Release){$cargoArgs+='--release'}
        & cargo @cargoArgs
        if($LASTEXITCODE -ne 0){throw 'Windows 桌面程序构建失败'}
    } finally {Pop-Location}
} finally {Pop-Location}
