param(
    [switch]$NoApi,
    [switch]$NoRelayer
)

$root = Split-Path -Parent $MyInvocation.MyCommand.Path

function Start-ServiceWindow($label, $command) {
    Write-Host "Launching $label..."
    Start-Process -FilePath "powershell.exe" -ArgumentList "-NoExit", "-Command", "cd `"$root`"; $command"
}

if (-not $NoApi) {
    Start-ServiceWindow "p-project-api" "cargo run -p p-project-api"
}

if (-not $NoRelayer) {
    Start-ServiceWindow "p-project-bridge relayer" "cargo run -p p-project-bridge --bin relayer_demo"
}
