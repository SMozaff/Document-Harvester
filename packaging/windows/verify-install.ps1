param(
    [Parameter(Mandatory = $true)]
    [string]$InstallRoot,
    [string]$Architecture = 'x64'
)

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$ownerPdf = Join-Path $repoRoot 'PDF_tests\01.pdf'
if (-not (Test-Path -LiteralPath $ownerPdf -PathType Leaf)) {
    throw "owner fixture is missing: $ownerPdf"
}

# The MSI is installed into an empty directory by the release workflow with
# `msiexec /a ... TARGETDIR=<InstallRoot>` before this script runs. Assert the
# installed file set, then run the built-in self-test with outbound networking
# blocked for the installed executable only.
$required = @(
    (Join-Path $InstallRoot 'dh-app.exe'),
    (Join-Path $InstallRoot 'pdfium.dll'),
    (Join-Path $InstallRoot 'SLINT-LICENSE.md'),
    (Join-Path $InstallRoot 'PDFIUM-LICENSE.txt'),
    (Join-Path $InstallRoot 'PDFIUM-LICENSES\pdfium.txt')
)
foreach ($path in $required) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "MSI install is missing file: $path"
    }
}

$executable = Join-Path $InstallRoot 'dh-app.exe'
$ruleName = "Document Harvester self-test $PID"
$rule = $null
try {
    $existing = Get-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue
    if ($existing) {
        Remove-NetFirewallRule -Name $existing.Name -ErrorAction Stop
    }
    $rule = New-NetFirewallRule -DisplayName $ruleName -Direction Outbound -Program $executable -Action Block -Profile Any -ErrorAction Stop
    Write-Host 'running installed self-test with outbound networking blocked for dh-app.exe'
    $output = & $executable '--pdfium-self-test' $ownerPdf 2>&1
    $exitCode = $LASTEXITCODE
    $output | ForEach-Object { Write-Host $_ }
    if ($exitCode -ne 0) {
        throw "pdfium self-test exited with $exitCode"
    }
}
finally {
    if ($null -ne $rule) {
        Remove-NetFirewallRule -Name $rule.Name -ErrorAction SilentlyContinue
    }
}

Write-Host 'MSI install smoke check passed'
