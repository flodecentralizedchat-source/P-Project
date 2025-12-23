# Script to run p-project-contracts tests in Docker

Write-Host "Building and running p-project-contracts tests in Docker..."

# Build the Docker image
docker build -t p-project-contracts-tests -f Dockerfile ..

# Run the tests
docker run --rm p-project-contracts-tests

Write-Host "Tests completed."