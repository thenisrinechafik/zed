param(
    [string]$ProxyUrl = $env:HTTPS_PROXY,
    [string]$NodeVersion = "22.5.1"
)

Write-Host "==> Simulating managed Node download with proxy: $ProxyUrl"

if (-not $ProxyUrl) {
    Write-Host "No proxy specified – falling back to direct connection"
}

$env:HTTPS_PROXY = $ProxyUrl
$env:HTTP_PROXY = $ProxyUrl

$exe = "${PSScriptRoot}\..\..\target\debug\proxy-smoke.exe"
if (-not (Test-Path $exe)) {
    Write-Host "proxy smoke binary not present; run cargo test to build helpers" -ForegroundColor Yellow
}

Write-Host "Ensuring Node runtime $NodeVersion is available..."
Write-Host "Run the following from the repository root to exercise the managed runtime:" -ForegroundColor Cyan
Write-Host "  cargo run -p node_runtime --features win-shell-adapters --example ensure_node -- $NodeVersion"
