use std::env;

#[derive(Clone, Debug, Default)]
pub struct EthConfig {
    pub rpc_url: String,
    pub bridge_address: String,
    pub token_address: String,
    pub private_key_env: String,
    pub confirmations: u32,
}

#[derive(Clone, Debug, Default)]
pub struct SolanaConfig {
    pub rpc_url: String,
    pub bridge_program: String,
    pub keypair_path_env: String,
    pub confirmations: u32,
}

#[derive(Clone, Debug, Default)]
pub struct SuiConfig {
    pub rpc_url: String,
    pub bridge_package: String,
    pub key_env: String,
    pub confirmations: u32,
}

#[derive(Clone, Debug, Default)]
pub struct BridgeConfig {
    pub eth: Option<EthConfig>,
    pub solana: Option<SolanaConfig>,
    pub sui: Option<SuiConfig>,
}

impl BridgeConfig {
    pub fn from_env() -> Self {
        let eth = match (
            env::var("ETH_RPC_URL").ok(),
            env::var("ETH_BRIDGE_ADDRESS").ok(),
            env::var("ETH_TOKEN_ADDRESS").ok(),
            env::var("ETH_PRIVATE_KEY_ENV").ok(),
            env::var("ETH_CONFIRMATIONS").ok(),
        ) {
            (Some(rpc), Some(addr), Some(token), pk_env, conf) => Some(EthConfig {
                rpc_url: rpc,
                bridge_address: addr,
                token_address: token,
                private_key_env: pk_env.unwrap_or_else(|| "ETH_PRIVATE_KEY".to_string()),
                confirmations: conf.and_then(|v| v.parse().ok()).unwrap_or(3),
            }),
            _ => None,
        };

        let solana = match (
            env::var("SOLANA_RPC_URL").ok(),
            env::var("SOLANA_BRIDGE_PROGRAM").ok(),
            env::var("SOLANA_KEYPAIR_PATH_ENV").ok(),
            env::var("SOLANA_CONFIRMATIONS").ok(),
        ) {
            (Some(rpc), Some(prog), kp_env, conf) => Some(SolanaConfig {
                rpc_url: rpc,
                bridge_program: prog,
                keypair_path_env: kp_env.unwrap_or_else(|| "SOLANA_KEYPAIR_PATH".to_string()),
                confirmations: conf.and_then(|v| v.parse().ok()).unwrap_or(10),
            }),
            _ => None,
        };

        let sui = match (
            env::var("SUI_RPC_URL").ok(),
            env::var("SUI_BRIDGE_PACKAGE").ok(),
            env::var("SUI_KEY_ENV").ok(),
            env::var("SUI_CONFIRMATIONS").ok(),
        ) {
            (Some(rpc), Some(pkg), key_env, conf) => Some(SuiConfig {
                rpc_url: rpc,
                bridge_package: pkg,
                key_env: key_env.unwrap_or_else(|| "SUI_KEY".to_string()),
                confirmations: conf.and_then(|v| v.parse().ok()).unwrap_or(2),
            }),
            _ => None,
        };

        Self { eth, solana, sui }
    }
}
