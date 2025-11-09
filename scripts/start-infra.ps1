param(
    [switch]$NoMutex
)

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$compose = Join-Path $root "..\\docker-compose.yml"

if (-not (Test-Path $compose)) {
    Write-Error "docker-compose.yml not found at $compose"
    exit 1
}

$services = @("mysql", "redis", "mongodb")

function Invoke-Compose($command, $services) {
    $cmd = "docker compose -f `"$compose`" $command $($services -join ' ')"
    Write-Host $cmd
    & docker compose -f $compose $command $services
}

if ($NoMutex) {
    Invoke-Compose "up -d" $services
} else {
    Invoke-Compose "up -d" $services
}

Write-Host "MySQL, Redis, and MongoDB are starting via Docker Compose."
Write-Host "Use `docker compose -f $compose ps` to check status."
