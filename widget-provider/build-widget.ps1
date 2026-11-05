[CmdletBinding()]
param(
    [string]$Configuration = "Release",
    [string]$Runtime = "win-x64",
    [string]$OutputDir = "",
    [string]$Version = "",
    [string]$PfxPath = $env:WIDGET_SIGNING_PFX_PATH,
    [string]$PfxPassword = $env:WIDGET_SIGNING_PFX_PASSWORD
)

$ErrorActionPreference = "Stop"
$root = $PSScriptRoot
$project = Join-Path $root "AiUsageDock.WidgetProvider.csproj"
$publishDir = Join-Path $root "obj\publish"
$stageDir = Join-Path $root "obj\stage"

if (-not $OutputDir) {
    $OutputDir = Join-Path $root "..\artifacts\widget"
}

if (-not $Version) {
    $config = Get-Content (Join-Path $root "..\src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json
    $Version = "$($config.version).0"
}

Write-Host "Building AI Usage Dock widget provider $Version ($Runtime)"

foreach ($dir in @($publishDir, $stageDir)) {
    if (Test-Path $dir) { Remove-Item $dir -Recurse -Force }
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
}
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

function Find-SdkTool {
    param([Parameter(Mandatory)][string]$Name)

    $roots = @()
    if (${env:ProgramFiles(x86)}) {
        $roots += Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"
    }
    if ($env:ProgramFiles) {
        $roots += Join-Path $env:ProgramFiles "Windows Kits\10\bin"
    }
    $roots = $roots | Where-Object { Test-Path $_ }

    foreach ($sdkRoot in $roots) {
        $candidate = Get-ChildItem $sdkRoot -Directory |
            Where-Object { $_.Name -match '^\d+\.\d+\.\d+\.\d+$' } |
            Sort-Object { [version]$_.Name } -Descending |
            ForEach-Object { Join-Path $_.FullName "x64\$Name.exe" } |
            Where-Object { Test-Path $_ } |
            Select-Object -First 1

        if ($candidate) { return $candidate }
    }

    $command = Get-Command "$Name.exe" -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }

    throw "$Name.exe was not found. Install the Windows 11 SDK (10.0.22621 or later)."
}

function Find-VisualStudio {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $vswhere)) {
        throw "vswhere.exe was not found. Install Visual Studio 2022 with the .NET desktop and Windows application development workloads."
    }

    $installPath = & $vswhere -latest -products * -requires Microsoft.Component.MSBuild -property installationPath | Select-Object -First 1
    if (-not $installPath) {
        throw "Visual Studio with MSBuild was not found."
    }

    $msbuild = Join-Path $installPath "MSBuild\Current\Bin\MSBuild.exe"
    if (-not (Test-Path $msbuild)) {
        throw "MSBuild.exe was not found under $installPath."
    }

    $appxTools = Get-ChildItem (Join-Path $installPath "MSBuild\Microsoft\VisualStudio") -Directory -ErrorAction SilentlyContinue |
        ForEach-Object { Join-Path $_.FullName "AppxPackage" } |
        Where-Object { Test-Path (Join-Path $_ "Microsoft.Build.AppxPackage.dll") } |
        Select-Object -First 1

    if (-not $appxTools) {
        throw "The MSIX/Appx MSBuild tasks were not found under $installPath. Install the 'Windows application development' workload."
    }

    return @{ MSBuild = $msbuild; AppxTools = $appxTools }
}

Write-Host "Publishing provider executable..."
$visualStudio = Find-VisualStudio
Push-Location $root
try {
    & $visualStudio.MSBuild $project /t:Restore,Publish `
        /p:Configuration=$Configuration `
        /p:Platform=x64 `
        /p:RuntimeIdentifier=$Runtime `
        /p:SelfContained=true `
        /p:WindowsAppSDKSelfContained=true `
        /p:WindowsPackageType=None `
        /p:AppxMSBuildToolsPath="$($visualStudio.AppxTools)\" `
        /p:PublishDir="$publishDir\" `
        /p:DebugType=None `
        /p:DebugSymbols=false `
        /nologo `
        /v:minimal
}
finally {
    Pop-Location
}

if ($LASTEXITCODE -ne 0) { throw "MSBuild publish failed with exit code $LASTEXITCODE" }

$exePath = Join-Path $publishDir "AiUsageDock.WidgetProvider.exe"
if (-not (Test-Path $exePath)) { throw "Expected provider executable was not produced: $exePath" }

Write-Host "Assembling package payload..."
Copy-Item -Path (Join-Path $publishDir "*") -Destination $stageDir -Recurse -Force
Copy-Item -Path (Join-Path $root "Assets") -Destination $stageDir -Recurse -Force
Copy-Item -Path (Join-Path $root "ProviderAssets") -Destination $stageDir -Recurse -Force

$manifest = Get-Content (Join-Path $root "Package.appxmanifest") -Raw
$manifest = [regex]::Replace($manifest, 'Version="\d+\.\d+\.\d+\.\d+"', "Version=`"$Version`"")
Set-Content -Path (Join-Path $stageDir "AppxManifest.xml") -Value $manifest -Encoding UTF8

$makeappx = Find-SdkTool "makeappx"
$signtool = Find-SdkTool "signtool"
$msixPath = Join-Path $OutputDir "AIUsageDock.Widget.msix"
$cerPath = Join-Path $OutputDir "AIUsageDock.Widget.cer"

Write-Host "Packing MSIX..."
& $makeappx pack /d $stageDir /p $msixPath /o
if ($LASTEXITCODE -ne 0) { throw "makeappx failed with exit code $LASTEXITCODE" }

Write-Host "Preparing signing certificate..."
if ($PfxPath) {
    if (-not (Test-Path $PfxPath)) { throw "Signing certificate not found: $PfxPath" }
    if ($PfxPassword) {
        $securePassword = ConvertTo-SecureString -String $PfxPassword -Force -AsPlainText
        $certificate = Import-PfxCertificate -FilePath $PfxPath -CertStoreLocation "Cert:\CurrentUser\My" -Password $securePassword
    }
    else {
        $certificate = Import-PfxCertificate -FilePath $PfxPath -CertStoreLocation "Cert:\CurrentUser\My"
    }

    $pfxToUse = $PfxPath
    $passwordToUse = $PfxPassword
}
else {
    $existing = Get-ChildItem "Cert:\CurrentUser\My" |
        Where-Object { $_.Subject -eq "CN=AI Usage Dock" -and $_.HasPrivateKey } |
        Sort-Object NotAfter -Descending |
        Select-Object -First 1

    if ($existing) {
        $certificate = $existing
    }
    else {
        $certificate = New-SelfSignedCertificate -Type Custom `
            -Subject "CN=AI Usage Dock" `
            -KeyUsage DigitalSignature `
            -KeySpec Signature `
            -FriendlyName "AI Usage Dock Widget Signing" `
            -CertStoreLocation "Cert:\CurrentUser\My" `
            -KeyExportPolicy Exportable `
            -NotAfter (Get-Date).AddYears(5) `
            -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3")
    }

    $passwordToUse = [guid]::NewGuid().ToString("N")
    $pfxToUse = Join-Path $OutputDir "AIUsageDock.Widget.pfx"
    $securePassword = ConvertTo-SecureString -String $passwordToUse -Force -AsPlainText
    Export-PfxCertificate -Cert $certificate -FilePath $pfxToUse -Password $securePassword | Out-Null
}

if ($certificate.Subject -ne "CN=AI Usage Dock") {
    throw "Signing certificate subject must be 'CN=AI Usage Dock', but it is '$($certificate.Subject)'."
}

Export-Certificate -Cert $certificate -FilePath $cerPath | Out-Null

Write-Host "Signing package..."
if ($passwordToUse) {
    & $signtool sign /fd SHA256 /f $pfxToUse /p $passwordToUse /d "AI Usage Dock Widget" $msixPath
}
else {
    & $signtool sign /fd SHA256 /f $pfxToUse /d "AI Usage Dock Widget" $msixPath
}
if ($LASTEXITCODE -ne 0) { throw "signtool failed with exit code $LASTEXITCODE" }

if (-not $PfxPath) {
    # Never ship the throwaway private key with the release artifacts.
    Remove-Item $pfxToUse -Force -ErrorAction SilentlyContinue
}

Copy-Item (Join-Path $root "install-widget.ps1") (Join-Path $OutputDir "install-widget.ps1") -Force

Write-Host ""
Write-Host "Widget package ready:" -ForegroundColor Green
Get-ChildItem $OutputDir | Select-Object Name, Length | Format-Table -AutoSize
