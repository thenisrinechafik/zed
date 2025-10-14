[CmdletBinding()]
param(
    [switch]$SkipTests,
    [string[]]$Exclude = @('gpui'),
    [switch]$StaticCRT,
    [switch]$RunClippy
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot '..' '..')
Set-Location $RepoRoot

$LogDir = Join-Path $RepoRoot 'target/win-ci-logs'
New-Item -ItemType Directory -Path $LogDir -Force | Out-Null

$env:ZED_WIN_BOOTSTRAP = '1'
if ($StaticCRT) {
    $env:ZED_STATIC_CRT = '1'
}

function Invoke-Cargo {
    param(
        [string[]]$Args,
        [string]$LogName
    )

    Write-Host "==> cargo $($Args -join ' ')" -ForegroundColor Cyan
    $logPath = Join-Path $LogDir $LogName
    & cargo @Args 2>&1 | Tee-Object -FilePath $logPath
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $($Args[0]) failed"
    }
}

if ($RunClippy) {
    Invoke-Cargo -Args @('clippy', '--workspace', '--all-targets', '--all-features', '--', '-D', 'warnings') -LogName 'cargo-clippy.log'
}

Invoke-Cargo -Args @('check', '--workspace') -LogName 'cargo-check.log'
Invoke-Cargo -Args @('build', '-p', 'gpui', '--example', 'win_smoke', '--features', 'win-smoke') -LogName 'cargo-build-win-smoke.log'

if (-not $SkipTests) {
    $testArgs = @('test', '--workspace', '--no-fail-fast')
    foreach ($crate in $Exclude) {
        $testArgs += @('--exclude', $crate)
    }
    Invoke-Cargo -Args $testArgs -LogName 'cargo-test.log'
}

$gpuLog = Join-Path $env:LOCALAPPDATA 'Zed\logs\gpu'
if (Test-Path $gpuLog) {
    Copy-Item -Path $gpuLog -Destination (Join-Path $LogDir 'gpu') -Recurse -Force
}

Write-Host "Windows build pipeline finished" -ForegroundColor Green
