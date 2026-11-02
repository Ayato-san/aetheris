#!/usr/bin/env bash

set -euo pipefail

case "${1:---release}" in
    --debug) build_profile=debug ;;
    --release) build_profile=release ;;
    *)
        echo "Usage : $0 [--debug|--release]" >&2
        exit 1
        ;;
esac
if (( $# > 1 )); then
    echo "Usage : $0 [--debug|--release]" >&2
    exit 1
fi

project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

if ! command -v powershell.exe >/dev/null 2>&1; then
    echo "Erreur : powershell.exe est introuvable." >&2
    echo "Ce script doit être lancé depuis WSL avec l’intégration Windows active." >&2
    exit 1
fi

if ! command -v wslpath >/dev/null 2>&1; then
    echo "Erreur : wslpath est introuvable." >&2
    exit 1
fi

windows_project_dir="$(wslpath -w "$project_dir")"
export AETHERIS_WINDOWS_PROJECT_DIR="$windows_project_dir"
export AETHERIS_WINDOWS_BUILD_PROFILE="$build_profile"
export WSLENV="${WSLENV:+$WSLENV:}AETHERIS_WINDOWS_PROJECT_DIR:AETHERIS_WINDOWS_BUILD_PROFILE"

powershell.exe -NoProfile -NonInteractive -ExecutionPolicy Bypass -Command '
    $ErrorActionPreference = "Stop"
    [Console]::OutputEncoding = [System.Text.UTF8Encoding]::new()
    $OutputEncoding = [Console]::OutputEncoding
    $ProjectDir = $env:AETHERIS_WINDOWS_PROJECT_DIR
    Set-Location -LiteralPath $ProjectDir

    if (-not (Get-Command cargo.exe -ErrorAction SilentlyContinue)) {
        Write-Error "cargo.exe est introuvable. Installez Rust sur Windows avec rustup-init.exe."
        exit 1
    }

    $env:CARGO_TARGET_DIR = Join-Path $ProjectDir "target\windows"
    # Rust incremental-session locks are unsupported on WSL UNC shares.
    $env:CARGO_INCREMENTAL = "0"
    $BuildProfile = $env:AETHERIS_WINDOWS_BUILD_PROFILE
    $BuildArgs = @("build", "--locked")
    if ($BuildProfile -eq "release") {
        $BuildArgs += "--release"
    }
    cargo.exe @BuildArgs

    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }

    $Executable = Join-Path $env:CARGO_TARGET_DIR "$BuildProfile\aetheris.exe"
    Write-Host "Build Windows terminé : $Executable"
'
