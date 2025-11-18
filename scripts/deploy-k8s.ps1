#!/usr/bin/env pwsh
[CmdletBinding()]
param(
  [string]$Namespace = 'default',
  [string]$ApiImage,
  [string]$WebImage,
  [string]$RelayerImage,
  [string]$AirdropImage,
  [switch]$DryRun,
  [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Show-Usage {
  @"
Usage: deploy-k8s.ps1 [options]

Options:
  -Namespace <name>       Kubernetes namespace (default: default)
  -ApiImage <ref>         Override API image
  -WebImage <ref>         Override Web image
  -RelayerImage <ref>     Override Bridge Relayer image
  -AirdropImage <ref>     Override Airdrop Cron image
  -DryRun                 Print actions without executing
  -Help                   Show this help
"@
}

if ($Help) { Show-Usage; exit 0 }

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")

function Invoke-Step([string]$Command) {
  if ($DryRun) { Write-Host "+ $Command" } else { Invoke-Expression $Command }
}
function Require-Command([string]$Name) {
  if ($DryRun) { return }
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) { throw "Required command not found: $Name" }
}

Require-Command kubectl

foreach ($f in @('k8s/api.yaml','k8s/web.yaml','k8s/bridge-relayer.yaml','k8s/airdrop-cronjob.yaml')) {
  $path = Join-Path $RepoRoot $f
  if (Test-Path -LiteralPath $path -PathType Leaf) {
    Invoke-Step "kubectl apply -n `"$Namespace`" -f `"$path`""
  } else {
    Write-Host "[deploy-k8s] Skipping missing manifest: $path"
  }
}

if ($ApiImage)     { Invoke-Step "kubectl -n `"$Namespace`" set image deploy/p-project-api api=`"$ApiImage`" --record=true" }
if ($WebImage)     { Invoke-Step "kubectl -n `"$Namespace`" set image deploy/p-project-web web=`"$WebImage`" --record=true" }
if ($RelayerImage) { Invoke-Step "kubectl -n `"$Namespace`" set image deploy/p-project-bridge-relayer relayer=`"$RelayerImage`" --record=true" }
if ($AirdropImage) { Invoke-Step "kubectl -n `"$Namespace`" set image cronjob/p-project-airdrop-cron airdrop-cron=`"$AirdropImage`" --record=true" }

Write-Host "[deploy-k8s] Done."

