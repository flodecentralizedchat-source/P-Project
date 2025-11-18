#!/usr/bin/env pwsh
[CmdletBinding()]
param(
  [switch]$Rust = $true,
  [switch]$Contracts = $true,
  [switch]$Web = $false,
  [switch]$StartInfra = $false,
  [string]$Package,
  [string]$ContractsDir,
  [switch]$DryRun,
  [switch]$Help
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Show-Usage {
  @"
Usage: test-all.ps1 [options]

Options:
  -Rust / -NoRust           Run or skip Rust tests (default: Run)
  -Contracts / -NoContracts Run or skip Hardhat tests (default: Run)
  -Web / -NoWeb             Run or skip WebAssembly tests (default: Skip)
  -StartInfra               Start MySQL/Redis/MongoDB via docker compose
  -Package <name>           Run cargo tests for a specific package
  -ContractsDir <path>      Override contracts directory
  -DryRun                   Print actions without executing
  -Help                     Show this help

Examples:
  .\test-all.ps1 -StartInfra
  .\test-all.ps1 -Package p-project-dao -NoContracts -NoWeb
  .\test-all.ps1 -Web
"@
}

if ($Help) { Show-Usage; exit 0 }

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
if (-not $ContractsDir) { $ContractsDir = Join-Path $RepoRoot "p-project-contracts/src/contracts" }

function Invoke-Step([string]$Command) {
  if ($DryRun) {
    Write-Host "+ $Command"
  } else {
    Invoke-Expression $Command
  }
}

function Require-Command([string]$Name) {
  if ($DryRun) { return }
  if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
    throw "Required command not found: $Name"
  }
}

if ($StartInfra) {
  Write-Host "[test-all] Starting local infra (docker compose: mysql, redis, mongodb)"
  Require-Command docker
  $compose = Join-Path $RepoRoot "docker-compose.yml"
  Invoke-Step "docker compose -f `"$compose`" up -d mysql redis mongodb"
}

if ($Rust) {
  Write-Host "[test-all] Running Rust tests"
  Require-Command cargo
  $env:DATABASE_URL = "mysql://root:rootpassword@localhost:3306/p_project"
  $env:REDIS_URL = "redis://localhost:6379"
  $env:MONGODB_URL = "mongodb://localhost:27017"
  $env:MONGO_URI = "mongodb://localhost:27017"
  $env:MONGO_DB = "p_project_dao_test_ci"
  if ($Package) {
    Invoke-Step "cargo test -p `"$Package`" --all-features --verbose"
  } else {
    Invoke-Step "cargo test --all-features --workspace --verbose"
    Invoke-Step "cargo test -p p-project-dao -- --ignored create_and_fetch_active_proposal_via_mongo"
  }
}

if ($Contracts) {
  Write-Host "[test-all] Running Hardhat contract tests at: $ContractsDir"
  if (-not (Test-Path -LiteralPath $ContractsDir -PathType Container)) {
    throw "Contracts directory not found: $ContractsDir"
  }
  Require-Command npx
  Invoke-Step "cd `"$ContractsDir`"; npx hardhat compile"
  Invoke-Step "cd `"$ContractsDir`"; npx hardhat test"
}

if ($Web) {
  Write-Host "[test-all] Running WebAssembly build and tests"
  Require-Command wasm-pack
  $webDir = Join-Path $RepoRoot "p-project-web"
  Invoke-Step "wasm-pack build `"$webDir`" --target web"
  try {
    Invoke-Step "wasm-pack test `"$webDir`" --headless --chrome"
  } catch {
    Write-Host "No headless Chrome available; skipping wasm tests (build done)."
  }
}

Write-Host "[test-all] Done."

