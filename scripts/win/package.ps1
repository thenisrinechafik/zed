param(
    [Parameter(Mandatory = $true)][ValidateSet('Stable','Preview','Nightly')]
    [string]$Channel,
    [Parameter(Mandatory = $true)][string]$Version,
    [string]$ReleaseNotesPath = ""
)

$ErrorActionPreference = 'Stop'

$env:ZED_CHANNEL = $Channel.ToLowerInvariant()
$env:ZED_VERSION = $Version

Write-Host "Building Zed $Channel $Version"

pwsh -File "$PSScriptRoot/setup.ps1" | Out-Null

cargo build --release --features windows_port
cargo check -p zed_cli --features win-release

$staging = Join-Path $PSScriptRoot ".." | Join-Path -ChildPath "..\dist\windows\out\$Channel"
New-Item -ItemType Directory -Force -Path $staging | Out-Null

$installerName = "Zed-$Channel-$Version"
$iss = Join-Path $PSScriptRoot "..\..\dist\windows\inno.iss"

& "${env:INNOSETUP_COMPILER:-iscc}" /DAppVersion=$Version /DAppChannel=$($env:ZED_CHANNEL) $iss

Copy-Item "dist\windows\Output\$installerName.exe" $staging -Force

$appxManifest = Get-Content "dist/windows/appxmanifest.xml"
$appxManifest = $appxManifest.Replace('0.0.0.0', "$Version.0").Replace('Zed', "Zed $Channel")
$appxManifest | Set-Content "$staging/appxmanifest.xml"

Get-ChildItem $staging

if ($ReleaseNotesPath -and (Test-Path $ReleaseNotesPath)) {
    Copy-Item $ReleaseNotesPath "$staging\RELEASE_NOTES.md" -Force
}

(Get-FileHash "$staging\$installerName.exe" -Algorithm SHA256).Hash | \
    Set-Content "$staging\$installerName.exe.sha256"

@{
    channel = $Channel
    version = $Version
    artifacts = @{
        inno = "$installerName.exe"
    }
} | ConvertTo-Json -Depth 4 | Set-Content "$staging/RELEASE.json"
