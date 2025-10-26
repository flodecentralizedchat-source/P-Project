# P-Project - The Meme Coin Ecosystem

Welcome to P-Project, a comprehensive meme coin ecosystem built with Rust, WASM, and multiple database technologies.

## Project Overview

Based on the PROJECT-LAYOUT.MD specification, this project implements a full-featured meme coin ecosystem with:

- Token smart contracts with burn and reward mechanisms
- Staking and yield farming capabilities
- Airdrop distribution system
- DAO governance for community decision making
- Cross-chain bridge support
- Web interface with WASM components
- API server for external integrations

## Technology Stack

- **Language**: Rust
- **Web Assembly**: WASM for web components
- **Databases**:
  - MySQL for relational data
  - Redis for caching and session management
  - MongoDB for document storage (proposals, etc.)
- **Web Framework**: Axum for API server
- **Frontend**: WASM-based components

## Project Structure

```
p-project/
├── p-project-core/          # Shared models and utilities
├── p-project-contracts/     # Smart contract implementations
├── p-project-api/           # HTTP API server
├── p-project-dao/           # DAO governance system
├── p-project-staking/       # Staking contract services
├── p-project-airdrop/       # Airdrop distribution system
├── p-project-bridge/        # Cross-chain bridge functionality
├── p-project-web/           # Web interface with WASM components
└── src/main.rs             # Main application entry point
```

## Getting Started

### Prerequisites

- Rust and Cargo (latest stable version)
- MySQL, Redis, and MongoDB instances
- wasm-pack for building WASM components

### Building the Project

```bash
# Build the entire workspace
cargo build

# Build WASM components
wasm-pack build p-project-web --target web

# Run the API server
cargo run -p p-project-api
```

## Components

### Core Module
Contains shared models, utilities, and database connectors for MySQL, Redis, and MongoDB.

### Contracts Module
Implements the core token contract with:
- Token transfers with burn mechanism
- Holder reward distribution
- Token freezing for staking
- Balance management

### Staking Module
Provides staking functionality:
- Stake tokens for fixed periods
- Earn rewards based on staking duration
- Unstake tokens with accumulated rewards

### Airdrop Module
Handles airdrop distribution:
- Add eligible recipients
- Claim airdrop tokens
- Track claimed and unclaimed distributions

### DAO Module
Implements governance features:
- Create proposals
- Vote on proposals with token-weighted votes
- Tally votes and execute decisions

### Bridge Module
Enables cross-chain functionality:
- Bridge tokens between supported chains
- Track bridge transactions
- Verify transaction status

### Web Module
Provides web interface components using WASM:
- User profile management
- Token balance display
- Transaction history
- Staking interface

## Configuration

Database connections and other settings can be configured through environment variables or configuration files in each module.

## Development

To run tests:
```bash
cargo test
```

To check WASM build:
```bash
wasm-pack test p-project-web --headless --firefox
```

## Deployment

The API server can be deployed as a standalone service, and the WASM components can be served statically from any web server.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Thanks to the Rust community for excellent libraries and tools
- Inspired by various DeFi and meme coin projects