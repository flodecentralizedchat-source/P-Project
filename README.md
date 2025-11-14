# P-Project: The Social Impact Blockchain Ecosystem

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Build Status](https://img.shields.io/github/workflow/status/flodecentralizedchat-source/P-Project/CI)](https://github.com/flodecentralizedchat-source/P-Project/actions)

[![Static Web Image](https://img.shields.io/badge/GHCR-p--project--web--static-0A66?logo=github)](https://github.com/orgs/flodecentralizedchat-source/packages/container/package/p-project-web-static)
[![Web Static CI](https://github.com/flodecentralizedchat-source/P-Project/actions/workflows/web-static.yml/badge.svg)](https://github.com/flodecentralizedchat-source/P-Project/actions/workflows/web-static.yml)
[![API Health](https://github.com/flodecentralizedchat-source/P-Project/actions/workflows/api-health.yml/badge.svg)](https://github.com/flodecentralizedchat-source/P-Project/actions/workflows/api-health.yml)

Welcome to P-Project, a cutting-edge social impact ecosystem built with Rust, WebAssembly, and modern blockchain technologies. What started as a meme coin has evolved into a comprehensive platform for humanitarian aid, social verification, and community-driven governance.

## 🚀 Project Overview

P-Project transforms blockchain technology into a robust, scalable ecosystem with real-world utility. Built with performance and security in mind, our platform offers:

- **Token Contracts** with deflationary mechanisms and automatic rewards
- **DAO Governance** for community-driven decision making
- **Staking System** with time-based yield farming
- **Airdrop Distribution** for early supporters
- **Cross-Chain Bridge** for multi-network compatibility
- **Web Interface** powered by WebAssembly for a seamless user experience
- **AI Integration** for impact verification, NFT art generation, and fraud detection
- **IoT Solutions** for smart donation boxes, refugee camp wristbands, and food distribution systems
- **Web2 Integration** for social media donations, YouTube tips, and messaging platform bots

## 🏗️ Architecture

P-Project follows a modular monorepo architecture using Rust workspaces:

```
p-project/
├── p-project-core/          # Shared models, utilities, and database connectors
├── p-project-contracts/     # Token smart contracts with burn and reward mechanisms
├── p-project-api/           # RESTful API server for external integrations
├── p-project-dao/           # Decentralized Autonomous Organization governance
├── p-project-staking/       # Staking contract services with yield farming
├── p-project-airdrop/       # Airdrop distribution system
├── p-project-bridge/        # Cross-chain bridge functionality
└── p-project-web/           # WebAssembly frontend components
```

## 🛠️ Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| **Core** | Rust | High-performance, memory-safe systems programming |
| **Web** | WebAssembly | Frontend components with near-native performance |
| **Database** | MySQL | Relational data storage for users and transactions |
| **Cache** | Redis | Session management and caching layer |
| **Documents** | MongoDB | Flexible storage for proposals and governance data |
| **API** | Axum | High-performance web framework for API endpoints |
| **Build** | Cargo | Dependency management and build system |

## 📦 Installation

### Prerequisites

- Rust 1.70 or higher
- MySQL server
- Redis server
- MongoDB server

### Setup

1. Clone the repository:
```bash
git clone https://github.com/flodecentralizedchat-source/P-Project.git
cd P-Project
```

2. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your database credentials
```

3. Build the project:
```bash
cargo build
```

4. Build WebAssembly components:
```bash
wasm-pack build p-project-web --target web
```

## ▶️ Running the Application

### Start the API Server
```bash
cargo run -p p-project-api
```

The API will be available at `http://localhost:3000`

### Run Tests
```bash
cargo test
```

### Build Documentation
```bash
cargo doc --open
```

## Static Web Image (GHCR)

Prebuilt NGINX image (with Brotli and precompressed assets) is published to GHCR:

- Image: `ghcr.io/flodecentralizedchat-source/p-project-web-static:latest`
- Package page: https://github.com/orgs/flodecentralizedchat-source/packages/container/package/p-project-web-static

Pull and run locally:

```bash
docker pull ghcr.io/flodecentralizedchat-source/p-project-web-static:latest
docker run -p 8080:80 ghcr.io/flodecentralizedchat-source/p-project-web-static:latest
# open http://localhost:8080
```

Use in `docker-compose.yml` instead of building:

```yaml
services:
  nginx:
    image: ghcr.io/flodecentralizedchat-source/p-project-web-static:latest
    container_name: p_project_nginx
    ports:
      - "8080:80"
    depends_on:
      api:
        condition: service_healthy
    restart: unless-stopped
```

Notes:
- The image proxies API requests to `/api/` and serves the SPA from `/`.
- If the image is private, ensure the GHCR package visibility is set to public or log in: `echo $PAT | docker login ghcr.io -u USERNAME --password-stdin`.

## Docker Compose Profiles

- Full stack (API + DBs + NGINX):
  - `docker compose up --build -d`
  - NGINX at `http://localhost:8080`, API at `http://localhost:3000`

- Static web only (no API dependency):
  - `docker compose --profile web up --build -d web-static`
  - Opens `http://localhost:8080` and serves the SPA; `/api/` routes will 502 if called (API not started).

## 🧪 Core Components

### Token Contract
The heart of P-Project features:
- **Burn Mechanism**: 1-2% burn on each transaction to increase scarcity
- **Reflection Rewards**: Automatic redistribution of 1-3% to all holders
- **Anti-Whale Measures**: Transaction limits and maximum wallet restrictions

### Staking System
Earn rewards by staking your P tokens:
- Flexible staking periods (7-365 days)
- Annual reward rates up to 20%
- Automatic reward compounding

### DAO Governance
Community-driven decision making:
- Proposal creation and voting
- Token-weighted voting system
- Transparent execution of decisions

### Cross-Chain Bridge
Seamless token transfers between:
- Ethereum
- Binance Smart Chain
- Solana
- Polygon
- Base

### AI Services
Intelligent features powered by artificial intelligence:
- **Impact Verification**: AI chatbots that verify social impact activities
- **NFT Art Generation**: AI-powered creation of unique Peace NFTs
- **Fraud Detection**: Machine learning models to detect suspicious activities
- **Meme Generation**: AI-powered creation of viral memes for community engagement

### IoT Integration
Real-world applications through Internet of Things:
- **Smart Donation Boxes**: Hardware wallets for physical donations
- **NFC Wristbands**: Digital identity and payment solutions for refugee camps
- **QR-Code Food Distribution**: Efficient food aid distribution systems

### Web2 Integration
Bridging traditional platforms with blockchain:
- **Social Media Widgets**: Donation widgets for Facebook and Instagram
- **YouTube Tips**: "Tip in PeaceCoin" functionality for content creators
- **Messaging Bots**: Telegram and Discord bots for tipping and transactions

## 🌐 Web Interface

Our WebAssembly-powered frontend provides:
- Real-time token balance tracking
- Staking dashboard with yield calculators
- Governance interface for proposals
- Cross-chain bridge UI
- Mobile-responsive design

## 🔧 API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | API health check |
| `/users` | POST | Create new user |
| `/users/:id` | GET | Get user details |
| `/transfer` | POST | Transfer tokens between users |
| `/stake` | POST | Stake tokens for rewards |
| `/unstake` | POST | Unstake tokens with rewards |
| `/airdrop/claim` | POST | Claim airdrop tokens |
| `/ai/verify-impact` | POST | Verify social impact with AI chatbots |
| `/ai/generate-nft-art` | POST | Generate Peace NFT art with AI |
| `/ai/detect-fraud` | POST | Detect fraudulent activities |
| `/ai/generate-meme` | POST | Generate AI-powered memes |
| `/iot/register-donation-box` | POST | Register a new smart donation box |
| `/iot/record-donation` | POST | Record a donation to a box |
| `/iot/register-wristband` | POST | Register an NFC wristband |
| `/iot/add-funds-wristband` | POST | Add funds to a wristband |
| `/iot/create-food-qr` | POST | Create a food distribution QR code |
| `/iot/claim-food-qr` | POST | Claim food using a QR code |
| `/web2/create-donation-widget` | POST | Create a social media donation widget |
| `/web2/process-youtube-tip` | POST | Process a YouTube tip transaction |
| `/web2/register-messaging-bot` | POST | Register a messaging platform bot |

## 🤝 Community & Social

Join our vibrant community:
- **Twitter**: [@PProjectCoin](https://twitter.com/PProjectCoin)
- **Telegram**: [P-Project Community](https://t.me/PProject)
- **Discord**: [P-Project Server](https://discord.gg/PProject)
- **Reddit**: [r/PProject](https://reddit.com/r/PProject)

## 📈 Roadmap

### Phase 1: Foundation ✅
- [x] Core token contract development
- [x] API server implementation
- [x] Database integration
- [x] Basic web interface

### Phase 2: Expansion ✅
- [x] DAO governance launch
- [x] Staking platform release
- [x] Cross-chain bridge deployment
- [x] Mobile app development
- [x] Advanced Use Cases Implementation:
  - AI + P-Coin Integration (Impact verification chatbots, Peace NFT art generation, Fraud detection)
  - IoT + P-Coin Integration (Smart donation boxes, NFC wristbands, QR-code food distribution)
  - Web2 Integration (Social media widgets, YouTube tips, Messaging platform bots)

### Phase 3: Innovation 🚧
- [x] NFT marketplace integration
- [x] DeFi yield farming pools
- [x] AI-powered meme generator
- [ ] Layer 2 blockchain solution

## 🛡️ Security

P-Project prioritizes security through:
- **Audited Code**: Regular third-party security audits
- **Liquidity Lock**: LP tokens locked for 1-2 years
- **Renounced Ownership**: Optional for full decentralization
- **Transparency Dashboard**: Live token distribution and burn tracking

### Security and Auth Setup

- Set required environment variables (see `.env.example`) and copy to `.env`.
- JWT auth is required for sensitive endpoints. Generate a token offline:
  - `cargo run -p p-project-api --bin dev_token -- sub=<user_id_or_wallet> hours=24 role=admin`
  - Use the token with `Authorization: Bearer <token>`.
- Configure CORS with `CORS_ALLOWED_ORIGINS` and `CORS_ALLOW_CREDENTIALS`.
- Request guardrails (enabled by default): 128 concurrent requests, 30s timeout, 1MB body limit.

## 🤓 Meet Professor P

Our mascot, Professor P, is the genius social scientist who accidentally created the most sophisticated social impact ecosystem ever conceived. With his sarcastic wit and community-first approach, he guides P-Project's development while keeping the fun in DeFi and the purpose in profit.

Our mascot
## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙌 Contributing

We welcome contributions from the community! Please see our Contributing Guidelines for more information.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a pull request

## 📞 Support

For support, please open an issue on GitHub or contact us on our [Telegram](https://t.me/PProject) or [Discord](https://discord.gg/PProject) channels.

---

*Proof of Impact – Powered by the People*