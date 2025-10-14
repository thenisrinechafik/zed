param(
    [string]$Repo = "$PSScriptRoot/../../"
)

$ErrorActionPreference = 'Stop'

Set-Location $Repo
Write-Host "Running git smoke"

git status
