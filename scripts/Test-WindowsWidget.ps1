[CmdletBinding()]
param(
    [string]$ExpectedVersion = '0.1.9.0',
    [ValidateRange(0, 600)][int]$WatchSeconds = 0,
    [switch]$ExpectUnsupported,
    [string]$ReportPath
)

# Read-only acceptance evidence. Does not install packages, import certificates,
# alter policy, delete apps, or read credentials/account labels.
$ErrorActionPreference = 'Stop'
$checks = New-Object 'System.Collections.Generic.List[object]'
function Add-Check([string]$Name, [bool]$Passed, [string]$Detail) {
    $checks.Add([pscustomobject]@{ check = $Name; passed = $Passed; detail = $Detail })
}
if ($env:OS -ne 'Windows_NT') { throw 'Run this script on Windows.' }
$build = [Environment]::OSVersion.Version.Build
Add-Check 'OS' ($build -ge 22621 -xor [bool]$ExpectUnsupported) "Windows build $build"
if (-not $ExpectUnsupported) {
    $package = Get-AppxPackage -Name 'BarryGe1214.AIUsageDock.Widget' | Select-Object -First 1
    Add-Check 'Package registered' ($null -ne $package) 'Current user package registration'
    if ($package) {
        Add-Check 'Package version' ($package.Version.ToString() -eq $ExpectedVersion) "Expected $ExpectedVersion; found $($package.Version)"
        Add-Check 'Publisher' ($package.Publisher -eq 'CN=AI Usage Dock') $package.Publisher
        Add-Check 'Package status' ($package.Status.ToString() -eq 'Ok') $package.Status.ToString()
    }
    $feedPath = Join-Path $env:LOCALAPPDATA 'com.barryge1214.aiusagedock\widget.json'
    Add-Check 'Feed exists' (Test-Path -LiteralPath $feedPath) 'Desktop application feed'
    if (Test-Path -LiteralPath $feedPath) {
        $feed = Get-Content -LiteralPath $feedPath -Raw | ConvertFrom-Json
        $parsed = [DateTimeOffset]::MinValue
        $validTime = [DateTimeOffset]::TryParse([string]$feed.updatedAt, [ref]$parsed)
        Add-Check 'Feed schema' ($feed.version -eq 1 -and $validTime) 'Schema 1 and parseable updatedAt'
        Add-Check 'Accounts available' ($feed.accountCount -gt 0) "Account count: $($feed.accountCount); error count: $($feed.errorCount)"
        if ($WatchSeconds -gt 0) {
            Write-Host 'Now refresh usage in the desktop app. This script watches the feed; inspect Win+W separately.'
            $previous = [string]$feed.updatedAt
            $deadline = [DateTime]::UtcNow.AddSeconds($WatchSeconds)
            $changed = $false
            do {
                Start-Sleep -Seconds 1
                $next = Get-Content -LiteralPath $feedPath -Raw | ConvertFrom-Json
                $changed = [string]$next.updatedAt -ne $previous
            } while (-not $changed -and [DateTime]::UtcNow -lt $deadline)
            Add-Check 'Feed refresh' $changed "Waited up to $WatchSeconds seconds for updatedAt to change"
        }
    }
}
$checks | Format-Table -AutoSize | Out-Host
if ($ReportPath) {
    $checks | ConvertTo-Json -Depth 3 | Set-Content -LiteralPath $ReportPath -Encoding UTF8
}
if (@($checks | Where-Object { -not $_.passed }).Count -gt 0) { throw 'Widget acceptance checks failed.' }
Write-Host 'Automated checks passed. Win+W rendering and installation transitions still require visual acceptance.'
