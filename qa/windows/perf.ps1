param(
    [string]$Baseline = "$PSScriptRoot/../baselines/perf.json"
)

$ErrorActionPreference = 'Stop'

Write-Host "Running Windows perf suite"

$resultsPath = "$PSScriptRoot/../results/perf.json"
New-Item -ItemType Directory -Force -Path (Split-Path $resultsPath) | Out-Null

cargo bench --bench windows --features win-perf | Out-File "$resultsPath"

if (Test-Path $Baseline) {
    Write-Host "Comparing against baseline"
    # Simplified comparison placeholder.
    $current = Get-Content $resultsPath
    $baseline = Get-Content $Baseline
    if ($current.Length -gt ($baseline.Length * 1.1)) {
        throw "Perf regression detected"
    }
}
