param(
    [string]$Workspace = "$PSScriptRoot/../../"
)

$ErrorActionPreference = 'Stop'

Write-Host "Running Windows sanity script"

Start-Process -FilePath "$Workspace/target/release/zed.exe" -ArgumentList "--win-smoke" -PassThru | Out-Null
Start-Sleep -Seconds 5

Write-Host "Simulating Git workflow"
Set-Location $Workspace
if (-not (Test-Path .git)) { git init | Out-Null }
Get-ChildItem -Filter *.rs -Recurse | Select-Object -First 1 | ForEach-Object { Add-Content $_ '// sanity touch' }
git add .
git commit -am "sanity" --allow-empty | Out-Null

Write-Host "Sanity complete"
