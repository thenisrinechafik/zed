param(
    [Parameter(Mandatory = $true)][string]$ArtifactsPath,
    [Parameter()][string]$AzureKeyVaultName = $env:ZED_AZURE_KV,
    [Parameter()][string]$CertificateName = $env:ZED_CODESIGN_CERT
)

$ErrorActionPreference = 'Stop'

if (-not $AzureKeyVaultName -or -not $CertificateName) {
    throw "Azure signing configuration missing. Set ZED_AZURE_KV and ZED_CODESIGN_CERT."
}

Write-Host "Signing artifacts in $ArtifactsPath via Azure Key Vault $AzureKeyVaultName"

Get-ChildItem $ArtifactsPath -Filter *.exe -Recurse | ForEach-Object {
    $file = $_.FullName
    Write-Host "Signing $file"
    & "${env:AZURE_SIGNTOOL:-AzureSignTool}" sign `
        --azure-key-vault-url "https://$AzureKeyVaultName.vault.azure.net" `
        --azure-key-vault-certificate $CertificateName `
        --timestamp-rfc3161 "http://timestamp.digicert.com" `
        --input-file $file
}
