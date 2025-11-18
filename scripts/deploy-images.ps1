#!/usr/bin/env pwsh
[CmdletBinding()]
param(
  [ValidateSet('api','web','relayer','airdrop','all')]
  [string]$Component = 'all',
  [switch]$Push,
  [string]$Tag,
  [string]$Owner,
  [switch]$DryRun,
  [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Show-Usage {
  @"
Usage: deploy-images.ps1 [options]

Options:
  -Component <name>   api|web|relayer|airdrop|all (default: all)
  -Push               Push to GHCR (requires docker login)
  -Tag <tag>          Image tag (default: sha-<git-sha>)
  -Owner <owner>      GHCR owner/org (default: repo owner)
  -DryRun             Print actions without executing
  -Help               Show this help
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

if (-not $Tag) {
  $sha = $env:GITHUB_SHA
  if (-not $sha) {
    try { $sha = (git rev-parse --short HEAD) } catch { $sha = 'local' }
  }
  $Tag = "sha-$sha"
}
if (-not $Owner) {
  $ownerEnv = $env:GITHUB_REPOSITORY_OWNER
  if (-not $ownerEnv) { $Owner = 'example' } else { $Owner = $ownerEnv.ToLowerInvariant() }
}

Require-Command docker

function Build-One([string]$Dockerfile, [string]$Image) {
  $context = $RepoRoot
  $base = "docker buildx build --platform linux/amd64 --file `"$Dockerfile`" `"$context`" --tag `"$Image:$Tag`""
  if ($Push) { Invoke-Step "$base --push" } else { Invoke-Step "$base --load" }
}

switch ($Component) {
  'api'     { Build-One (Join-Path $RepoRoot 'p-project-api/Dockerfile') "ghcr.io/$Owner/p-project-api" }
  'web'     { Build-One (Join-Path $RepoRoot 'p-project-web/Dockerfile') "ghcr.io/$Owner/p-project-web" }
  'relayer' { Build-One (Join-Path $RepoRoot 'p-project-bridge/Dockerfile') "ghcr.io/$Owner/p-project-bridge-relayer" }
  'airdrop' { Build-One (Join-Path $RepoRoot 'p-project-airdrop/Dockerfile') "ghcr.io/$Owner/p-project-airdrop-cron" }
  'all' {
    Build-One (Join-Path $RepoRoot 'p-project-api/Dockerfile') "ghcr.io/$Owner/p-project-api"
    Build-One (Join-Path $RepoRoot 'p-project-web/Dockerfile') "ghcr.io/$Owner/p-project-web"
    Build-One (Join-Path $RepoRoot 'p-project-bridge/Dockerfile') "ghcr.io/$Owner/p-project-bridge-relayer"
    Build-One (Join-Path $RepoRoot 'p-project-airdrop/Dockerfile') "ghcr.io/$Owner/p-project-airdrop-cron"
  }
  default { throw "Invalid component: $Component" }
}

Write-Host "[deploy-images] Done."

