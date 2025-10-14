param(
    [Parameter(Mandatory = $true)][string]$Artifacts
)

$ErrorActionPreference = 'Stop'

Get-ChildItem $Artifacts -Filter *.exe -Recurse | ForEach-Object {
    $file = $_.FullName
    Write-Host "Verifying signature for $file"
    $sig = Get-AuthenticodeSignature $file
    if ($sig.Status -ne 'Valid') {
        throw "Signature verification failed for $file: $($sig.StatusMessage)"
    }
}

Get-ChildItem $Artifacts -Filter *.sha256 -Recurse | ForEach-Object {
    $hashFile = $_.FullName
    $target = Join-Path $_.DirectoryName ($_.BaseName)
    if (-not (Test-Path $target)) { return }
    $expected = Get-Content $hashFile
    $actual = (Get-FileHash $target -Algorithm SHA256).Hash
    if ($expected.Trim().ToLowerInvariant() -ne $actual.ToLowerInvariant()) {
        throw "Checksum mismatch for $target"
    }
}

Write-Host "Artifacts verified"
