#!/usr/bin/env pwsh
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Assert-Success([int]$Code, [string]$Message) {
  if ($Code -ne 0) { throw "FAIL: $Message (exit $Code)" } else { Write-Host "PASS: $Message" }
}

$root = Resolve-Path (Join-Path $PSScriptRoot "..\..")

# test-all.ps1 -Help
$help = & pwsh -NoProfile -File (Join-Path $root 'scripts\test-all.ps1') -Help
if ($help -notmatch 'Usage:') { throw 'FAIL: test-all.ps1 -Help did not print usage' } else { Write-Host 'PASS: test-all.ps1 -Help' }

# test-all.ps1 -DryRun
& pwsh -NoProfile -File (Join-Path $root 'scripts\test-all.ps1') -DryRun -StartInfra -NoContracts -NoWeb
Assert-Success $LASTEXITCODE 'test-all.ps1 -DryRun'

# deploy-images.ps1 -Help
$help2 = & pwsh -NoProfile -File (Join-Path $root 'scripts\deploy-images.ps1') -Help
if ($help2 -notmatch 'Usage:') { throw 'FAIL: deploy-images.ps1 -Help did not print usage' } else { Write-Host 'PASS: deploy-images.ps1 -Help' }

# deploy-images.ps1 -DryRun
& pwsh -NoProfile -File (Join-Path $root 'scripts\deploy-images.ps1') -DryRun -Component api -Tag ci-test
Assert-Success $LASTEXITCODE 'deploy-images.ps1 -DryRun'

# deploy-k8s.ps1 -Help
$help3 = & pwsh -NoProfile -File (Join-Path $root 'scripts\deploy-k8s.ps1') -Help
if ($help3 -notmatch 'Usage:') { throw 'FAIL: deploy-k8s.ps1 -Help did not print usage' } else { Write-Host 'PASS: deploy-k8s.ps1 -Help' }

# deploy-k8s.ps1 -DryRun
& pwsh -NoProfile -File (Join-Path $root 'scripts\deploy-k8s.ps1') -DryRun -Namespace ci -ApiImage ghcr.io/example/p-project-api:test
Assert-Success $LASTEXITCODE 'deploy-k8s.ps1 -DryRun'

Write-Host 'All script tests passed.'

