[CmdletBinding()]
param(
    [switch]$CI,
    [switch]$StaticCRT
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Write-Step {
    param([string]$Message)
    Write-Host "==> $Message" -ForegroundColor Cyan
}

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot '..' '..')
Write-Step "Repository root: $RepoRoot"

# Ensure cargo bin directory is available for child shells.
$CargoBin = Join-Path $env:USERPROFILE '.cargo\\bin'
if (-not ($env:PATH -split ';' | Where-Object { $_ -eq $CargoBin })) {
    Write-Step "Adding $CargoBin to PATH"
    $env:PATH = "$CargoBin;$env:PATH"
}

Write-Step "Ensuring required Rust target is installed"
& rustup target add x86_64-pc-windows-msvc

if ($StaticCRT -or $CI) {
    Write-Step "Enabling static CRT linkage via ZED_STATIC_CRT"
    $env:ZED_STATIC_CRT = '1'
}

# Visual Studio Build Tools check
if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
    Write-Warning "MSVC compiler (cl.exe) not detected. Install the Visual Studio Build Tools with the Desktop C++ workload."
    Write-Warning "winget install --id Microsoft.VisualStudio.2022.BuildTools --override '--add Microsoft.VisualStudio.Workload.VCTools --quiet --includeRecommended'"
}

# Ensure CMake is present – required by various crates during build.
if (-not (Get-Command cmake.exe -ErrorAction SilentlyContinue)) {
    if (-not $CI -and (Get-Command winget -ErrorAction SilentlyContinue)) {
        Write-Step "Installing CMake via winget"
        winget install --id Kitware.CMake --exact --silent
    } else {
        Write-Warning "CMake not found on PATH. Install from https://cmake.org/download/ or via winget (Kitware.CMake)."
    }
}

if (-not $CI) {
    Write-Host "• Install PostgreSQL if you plan to run collaboration services locally."
    Write-Host "  Recommended installer: https://www.postgresql.org/download/windows/"
}

Write-Step "Windows bootstrap environment prepared"
