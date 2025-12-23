.git - Git version control directory
.github - GitHub configuration
k8s - Kubernetes deployment files
nginx - Nginx web server configuration
p-docs - Project documentation
p-project-airdrop - Airdrop distribution system
p-project-api - REST API server
p-project-bridge - Cross-chain bridge functionality
p-project-contracts - Smart contracts
p-project-core - Shared models and utilities
p-project-dao - Decentralized Autonomous Organization governance
p-project-staking - Staking contract services
p-project-web - WebAssembly frontend components
scripts - Utility scripts
src - Main source directory
test_check - Test related files

```
bash

### run the project orderly 

Based on the P-Project structure and what we've learned from examining the Dockerfiles, here's the order in which you should run the folders/services:

## Running Order for P-Project Folders

1. **Database Services First** ([nginx](file:///d:/p-project/nginx) folder)
   - Start with the infrastructure that other services depend on
   - Run `docker compose up -d mysql redis mongodb` from the root directory
   - These are defined in the [docker-compose.yml](file:///d:/p-project/docker-compose.yml) file

2. **Core Services** ([p-project-core](file:///d:/p-project/p-project-core) folder)
   - This contains shared models and utilities used by other services
   - It's not run directly but is a dependency for other services

3. **API Service** ([p-project-api](file:///d:/p-project/p-project-api) folder)
   - Run this next as it provides the backend for the web interface
   - Command: `docker compose up -d api`
   - Or directly: `cargo run -p p-project-api`

4. **Bridge Service** ([p-project-bridge](file:///d:/p-project/p-project-bridge) folder)
   - Run after the API as it depends on the database
   - Command: `cargo run -p p-project-bridge --bin relayer_demo`

5. **Web Service** ([p-project-web](file:///d:/p-project/p-project-web) and [nginx](file:///d:/p-project/nginx) folders)
   - Run last as it depends on the API being available
   - Command: `docker compose up -d nginx`
   - Or for standalone web: `docker compose --profile web up -d web-static`

## Recommended Approach

The easiest way to run everything in the correct order is to use the full Docker Compose command from the root directory:

```bash
docker compose up --build -d
```

This command will automatically start services in the correct order based on their dependencies as defined in the [docker-compose.yml](file:///d:/p-project/docker-compose.yml) file:
- Databases start first (mysql, redis, mongodb)
- API starts next (depends on databases)
- Nginx starts last (depends on API being healthy)

The other folders ([p-project-contracts](file:///d:/p-project/p-project-contracts), [p-project-dao](file:///d:/p-project/p-project-dao), [p-project-staking](file:///d:/p-project/p-project-staking), [p-project-airdrop](file:///d:/p-project/p-project-airdrop), [p-docs](file:///d:/p-project/p-docs), etc.) contain libraries, documentation, or auxiliary services that are either compiled into the main services or run on scheduled jobs rather than continuously.

```