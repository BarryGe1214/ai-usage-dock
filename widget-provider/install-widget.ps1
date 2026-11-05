[CmdletBinding()]
param(
    [string]$MsixPath = (Join-Path $PSScriptRoot "AIUsageDock.Widget.msix"),
    [string]$CertPath = (Join-Path $PSScriptRoot "AIUsageDock.Widget.cer")
)

$ErrorActionPreference = "Stop"

if ([Environment]::OSVersion.Version.Build -lt 22621) {
    throw "The AI Usage Dock widget requires Windows 11 22H2 (build 22621) or later."
}

if (-not (Test-Path $MsixPath)) { throw "Package not found: $MsixPath" }
if (-not (Test-Path $CertPath)) { throw "Certificate not found: $CertPath" }

Unblock-File -Path $MsixPath -ErrorAction SilentlyContinue
Unblock-File -Path $CertPath -ErrorAction SilentlyContinue

Write-Host "Trusting the AI Usage Dock widget certificate..."
Import-Certificate -FilePath $CertPath -CertStoreLocation "Cert:\CurrentUser\TrustedPeople" | Out-Null

Write-Host "Installing the AI Usage Dock widget package..."
try {
    Add-AppxPackage -Path $MsixPath -ForceUpdateFromAnyVersion
}
catch {
    Write-Host ""
    Write-Host "Installation failed: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host "If the error mentions certificate trust (0x800B0109), open PowerShell as administrator and run:"
    Write-Host "  Import-Certificate -FilePath `"$CertPath`" -CertStoreLocation `"Cert:\LocalMachine\TrustedPeople`""
    throw
}

Write-Host ""
Write-Host "Installed successfully." -ForegroundColor Green
Write-Host "Open the widgets board with Win+W, click '+ Add widgets', and choose 'AI Usage Dock'."
Write-Host "The widget reads data written by the AI Usage Dock desktop app, so keep that app running."
