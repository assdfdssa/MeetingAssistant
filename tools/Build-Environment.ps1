$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path $PSScriptRoot -Parent
$env:MEETING_ASSISTANT_ROOT = $projectRoot
$env:CARGO_HOME = Join-Path $PSScriptRoot 'cargo'
$env:RUSTUP_HOME = Join-Path $PSScriptRoot 'rustup'
$env:CARGO_TARGET_DIR = Join-Path $projectRoot 'build/native'
$env:CARGO_INCREMENTAL = '0'
$toolchain = Join-Path $PSScriptRoot 'rustup/toolchains/stable-x86_64-pc-windows-msvc/bin'
$env:RUSTC = Join-Path $toolchain 'rustc.exe'
$env:RUSTDOC = Join-Path $toolchain 'rustdoc.exe'
$env:NEXT_TELEMETRY_DISABLED = '1'
$env:PATH = "$(Join-Path $PSScriptRoot 'node');$toolchain;$(Join-Path $PSScriptRoot 'ninja');$(Join-Path $PSScriptRoot 'python-packages/cmake/data/bin');$env:PATH"
$env:CMAKE_GENERATOR = 'Ninja'
# Whisper's upstream C++ includes UTF-8 punctuation. MSVC otherwise interprets
# it using the Chinese Windows legacy code page and fails on string literals.
$env:CXXFLAGS = '/utf-8'
# Generate ABI-correct Windows bindings. The upstream pregenerated bindings
# contain Linux libc layouts and must not be copied into this Windows build.
Remove-Item Env:WHISPER_DONT_GENERATE_BINDINGS -ErrorAction SilentlyContinue
$env:LIBCLANG_PATH = Join-Path $PSScriptRoot 'libclang-python/clang/native'
. (Join-Path $PSScriptRoot 'msvc/activate.ps1')
