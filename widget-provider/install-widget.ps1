[CmdletBinding()]
param(
    [string]$MsixPath = '',
    [string]$CertPath = '',
    [switch]$NoTrustPrompt
)

$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
$OutputEncoding = [Console]::OutputEncoding
$script:WidgetCertificateThumbprint = 'C2A01C2B80A834302C5982151E0AA28F93A68911'

function Get-PinnedWidgetCertificate([string]$Path) {
    $certificate = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($Path)
    if ($certificate.Thumbprint -ne $script:WidgetCertificateThumbprint) {
        throw "Unexpected widget signing certificate: $($certificate.Thumbprint)"
    }
    return $certificate
}

function Test-WidgetCertificateTrusted {
    return Test-Path -LiteralPath "Cert:\LocalMachine\TrustedPeople\$script:WidgetCertificateThumbprint"
}

function Test-WidgetAdministrator {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Add-WidgetCertificateTrust($Certificate) {
    $store = New-Object System.Security.Cryptography.X509Certificates.X509Store('TrustedPeople', 'LocalMachine')
    try {
        $store.Open([Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
        $store.Add($Certificate)
    }
    finally { $store.Close() }
}

function New-WidgetTrustCommand($Certificate) {
    # Embed the already verified public certificate, rather than asking the
    # elevated process to re-read a user-writable path. Recheck the pin there.
    $raw = [Convert]::ToBase64String($Certificate.RawData)
    $command = @'
$ErrorActionPreference = 'Stop'
try {
    $certificate = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2 -ArgumentList (,[Convert]::FromBase64String('__CERTIFICATE__'))
    if ($certificate.Thumbprint -ne '__THUMBPRINT__') { throw 'Unexpected widget signing certificate' }
    $store = New-Object System.Security.Cryptography.X509Certificates.X509Store('TrustedPeople', 'LocalMachine')
    try {
        $store.Open([Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
        $store.Add($certificate)
    } finally { $store.Close() }
    exit 0
} catch { exit 1 }
'@
    $command = $command.Replace('__CERTIFICATE__', $raw).Replace('__THUMBPRINT__', $script:WidgetCertificateThumbprint)
    return [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($command))
}

function Request-WidgetCertificateTrust($Certificate) {
    $encoded = New-WidgetTrustCommand $Certificate
    Write-Host 'Windows permission is required to trust the pinned widget publisher on this computer.'
    try {
        $process = Start-Process -FilePath (Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe') -Verb RunAs `
            -ArgumentList @('-NoProfile', '-NonInteractive', '-EncodedCommand', $encoded) -Wait -PassThru
    }
    catch {
        throw "Widget certificate authorization was cancelled or could not start. Retry Install and approve the Windows prompt. $($_.Exception.Message)"
    }
    if ($process.ExitCode -ne 0) {
        throw 'Could not trust the widget certificate. Administrator authorization or machine policy prevented the change.'
    }
}

function Get-WidgetWindowsBuild { return [Environment]::OSVersion.Version.Build }

function Install-WidgetPackage([string]$PackagePath, [string]$CertificatePath, [switch]$NoTrustPrompt) {
    if ((Get-WidgetWindowsBuild) -lt 22621) {
        throw 'The AI Usage Dock widget requires Windows 11 22H2 (build 22621) or later.'
    }
    if (-not (Test-Path -LiteralPath $PackagePath)) { throw "Package not found: $PackagePath" }
    if (-not (Test-Path -LiteralPath $CertificatePath)) { throw "Certificate not found: $CertificatePath" }
    $certificate = Get-PinnedWidgetCertificate $CertificatePath
    if (-not (Test-WidgetCertificateTrusted)) {
        if (Test-WidgetAdministrator) {
            Add-WidgetCertificateTrust $certificate
        }
        elseif ($NoTrustPrompt) {
            throw 'WIDGET_TRUST_REQUIRED: The widget certificate must be trusted for this computer (0x800B0109). Click Install in Settings and approve the Windows administrator prompt.'
        }
        else {
            Request-WidgetCertificateTrust $certificate
        }
        if (-not (Test-WidgetCertificateTrusted)) {
            throw 'The pinned widget certificate is still not trusted for this computer.'
        }
    }
    # Keep package registration in the original user's process. Elevating the
    # entire installer could register it for a different administrator account.
    Add-AppxPackage -Path $PackagePath -ForceUpdateFromAnyVersion -ForceApplicationShutdown -ErrorAction Stop
    Write-Host 'Installed successfully. Open Win+W and add AI Usage Dock.'
}

# Dot-sourcing exposes functions for regression tests without changing trust.
if ($MyInvocation.InvocationName -ne '.') {
    try {
        # Resolve defaults only after parameter binding and UTF-8 initialization.
        # The app passes absolute paths; the fallback supports manual -File use.
        if ([string]::IsNullOrEmpty($MsixPath) -or [string]::IsNullOrEmpty($CertPath)) {
            $installerDirectory = $PSScriptRoot
            if ([string]::IsNullOrEmpty($installerDirectory)) {
                $installerDirectory = Split-Path -Parent $MyInvocation.MyCommand.Path
            }
            if ([string]::IsNullOrEmpty($MsixPath)) {
                $MsixPath = Join-Path $installerDirectory 'AIUsageDock.Widget.msix'
            }
            if ([string]::IsNullOrEmpty($CertPath)) {
                $CertPath = Join-Path $installerDirectory 'AIUsageDock.Widget.cer'
            }
        }
        Install-WidgetPackage -PackagePath $MsixPath -CertificatePath $CertPath -NoTrustPrompt:$NoTrustPrompt
    }
    catch {
        [Console]::Error.WriteLine($_.Exception.Message)
        exit 1
    }
}
