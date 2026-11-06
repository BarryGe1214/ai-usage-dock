$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot '../widget-provider/install-widget.ps1')

function Assert([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
    Write-Host "PASS: $Message"
}
function Assert-Throws($Action, [string]$Pattern) {
    $message = $null
    try { & $Action } catch { $message = $_.Exception.Message }
    Assert ($null -ne $message -and $message -like "*$Pattern*") "Expected failure: $Pattern"
}

# Exercise the real pin validation with a different certificate. Only the public
# certificate is written; no machine certificate store or credentials are used.
$temp = Join-Path ([IO.Path]::GetTempPath()) ([Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Path $temp | Out-Null
try {
    $rsa = [Security.Cryptography.RSA]::Create(2048)
    $request = [Security.Cryptography.X509Certificates.CertificateRequest]::new(
        'CN=Test only', $rsa, [Security.Cryptography.HashAlgorithmName]::SHA256,
        [Security.Cryptography.RSASignaturePadding]::Pkcs1)
    $wrong = $request.CreateSelfSigned([DateTimeOffset]::UtcNow.AddMinutes(-1), [DateTimeOffset]::UtcNow.AddDays(1))
    $certPath = Join-Path $temp 'wrong.cer'
    [IO.File]::WriteAllBytes($certPath, $wrong.Export([Security.Cryptography.X509Certificates.X509ContentType]::Cert))
    Assert-Throws { Get-PinnedWidgetCertificate $certPath } 'Unexpected widget signing certificate'
    $encoded = New-WidgetTrustCommand $wrong
    $decoded = [Text.Encoding]::Unicode.GetString([Convert]::FromBase64String($encoded))
    Assert ($decoded.Contains($script:WidgetCertificateThumbprint) -and $decoded.Contains('LocalMachine')) 'Elevated command retains pin and machine trust scope'
    $tokens = $null; $parseErrors = $null
    [Management.Automation.Language.Parser]::ParseInput($decoded, [ref]$tokens, [ref]$parseErrors) | Out-Null
    Assert ($parseErrors.Count -eq 0) 'Encoded elevation script parses'
    $wrong.Dispose(); $rsa.Dispose()

    # Replace OS boundaries only. Test the actual orchestration function.
    function Get-WidgetWindowsBuild { return $script:build }
    function Get-PinnedWidgetCertificate { return [pscustomobject]@{ Thumbprint = $script:WidgetCertificateThumbprint } }
    function Test-WidgetCertificateTrusted { return $script:trusted }
    function Test-WidgetAdministrator { return $false }
    function Request-WidgetCertificateTrust {
        $script:prompts++
        if ($script:cancel) { throw 'UAC cancelled' }
        $script:trusted = $true
    }
    function Add-AppxPackage {
        param($Path, [switch]$ForceUpdateFromAnyVersion, [switch]$ForceApplicationShutdown, $ErrorAction)
        Assert ($ForceUpdateFromAnyVersion -and $ForceApplicationShutdown) 'Package upgrade closes the old provider'
        $script:installs++
    }
    $package = Join-Path $temp 'test.msix'
    [IO.File]::WriteAllText($package, 'test boundary only')
    $script:build = 22621
    $script:trusted = $false
    $script:cancel = $false
    $script:prompts = 0
    $script:installs = 0
    Assert-Throws { Install-WidgetPackage $package $certPath -NoTrustPrompt } 'WIDGET_TRUST_REQUIRED'
    Assert ($script:prompts -eq 0 -and $script:installs -eq 0) 'Automatic installation does not prompt or install without trust'
    Install-WidgetPackage $package $certPath
    Assert ($script:prompts -eq 1 -and $script:installs -eq 1) 'Manual installation authorizes trust then registers package'
    Install-WidgetPackage $package $certPath -NoTrustPrompt
    Assert ($script:prompts -eq 1 -and $script:installs -eq 2) 'Trusted automatic update needs no further elevation'
    $script:trusted = $false
    $script:cancel = $true
    Assert-Throws { Install-WidgetPackage $package $certPath } 'UAC cancelled'
    Assert ($script:installs -eq 2) 'Cancelled authorization does not install the package'
    $script:build = 19045
    Assert-Throws { Install-WidgetPackage $package $certPath } 'requires Windows 11'
    Assert ($script:installs -eq 2) 'Unsupported Windows does not install'
}
finally { Remove-Item -LiteralPath $temp -Recurse -Force }
