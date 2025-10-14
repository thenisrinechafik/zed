param(
    [string]$Output = "zed-logs.zip"
)

$ErrorActionPreference = 'Stop'

$root = Join-Path $env:LOCALAPPDATA 'Zed'
if (-not (Test-Path $root)) {
    throw "Zed log root not found: $root"
}

$items = @(
    Join-Path $root 'logs'
    Join-Path $root 'settings.json'
    Join-Path $root 'telemetry.json'
)

$existing = $items | Where-Object { Test-Path $_ }
if (-not $existing) {
    throw "No logs to collect"
}

Compress-Archive -Path $existing -DestinationPath $Output -Force
Write-Host "Logs bundle created at $Output"
