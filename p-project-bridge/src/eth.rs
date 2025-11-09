use crate::adapter::{AdapterTxStatus, ChainAdapter};
use crate::config::EthConfig;
use crate::error::BridgeError;
use async_trait::async_trait;
use ethers::contract::abigen;
use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Signer};
use ethers::types::{Address, H256, U256};
use std::sync::Arc;

abigen!(Bridge, "abi/Bridge.json");
abigen!(Erc20, "abi/ERC20.json");

pub struct EthereumAdapter {
    provider: Option<Provider<Http>>,
    signer: Option<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    bridge_address: Option<Address>,
    token_address: Option<Address>,
    confirmations: u32,
}

impl EthereumAdapter {
    pub fn new(cfg: Option<&EthConfig>) -> Self {
        if let Some(c) = cfg {
            let provider = Provider::<Http>::try_from(c.rpc_url.as_str()).ok();
            let bridge_address = c.bridge_address.parse::<Address>().ok();
            let token_address = c.token_address.parse::<Address>().ok();
            let signer = match (provider.clone(), std::env::var(&c.private_key_env).ok()) {
                (Some(p), Some(pk)) => {
                    let wallet: LocalWallet = match pk.parse() {
                        Ok(w) => w,
                        Err(_) => {
                            return Self {
                                provider,
                                signer: None,
                                bridge_address,
                                confirmations: c.confirmations,
                            }
                        }
                    };
                    Some(Arc::new(SignerMiddleware::new(p.clone(), wallet)))
                }
                _ => None,
            };
            return Self {
                provider,
                signer,
                bridge_address,
                token_address,
                confirmations: c.confirmations,
            };
        }
        Self {
            provider: None,
            signer: None,
            bridge_address: None,
            token_address: None,
            confirmations: 0,
        }
    }
}

#[async_trait]
impl ChainAdapter for EthereumAdapter {
    fn name(&self) -> &'static str {
        "Ethereum"
    }

    async fn lock(
        &self,
        recipient: &str,
        _token: &str,
        amount: f64,
        _to_chain: &str,
    ) -> Result<String, BridgeError> {
        let signer = self
            .signer
            .as_ref()
            .ok_or(BridgeError::ConfigMissing("ETH_PRIVATE_KEY_ENV or signer"))?;
        let bridge_addr = self
            .bridge_address
            .ok_or(BridgeError::ConfigMissing("ETH_BRIDGE_ADDRESS"))?;
        let token_addr = self
            .token_address
            .ok_or(BridgeError::ConfigMissing("ETH_TOKEN_ADDRESS"))?;

        let recipient_addr: Address = recipient
            .parse()
            .map_err(|_| BridgeError::Other("Invalid recipient address".into()))?;

        let erc20 = Erc20::new(token_addr, signer.clone());
        let decimals: u8 = erc20
            .decimals()
            .call()
            .await
            .map_err(|e| BridgeError::RpcUnavailable(e.to_string()))?;
        let scale = 10u128.pow(decimals as u32);
        let scaled = U256::from(((amount * scale as f64).round() as u128));

        // Ensure allowance
        let owner = signer.address();
        let allowance: U256 = erc20
            .allowance(owner, bridge_addr)
            .call()
            .await
            .map_err(|e| BridgeError::RpcUnavailable(e.to_string()))?;
        if allowance < scaled {
            let _ = erc20
                .approve(bridge_addr, scaled)
                .send()
                .await
                .map_err(|e| BridgeError::TxFailed(e.to_string()))?;
        }

        let bridge = Bridge::new(bridge_addr, signer.clone());
        let pending = bridge
            .lock(token_addr, scaled, recipient_addr)
            .send()
            .await
            .map_err(|e| BridgeError::TxFailed(e.to_string()))?;
        let tx_hash = *pending;
        Ok(format!("0x{:x}", tx_hash))
    }

    async fn mint_or_release(
        &self,
        recipient: &str,
        _token: &str,
        amount: f64,
        _from_chain: &str,
        _source_tx: &str,
        lock_id: Option<&str>,
    ) -> Result<String, BridgeError> {
        let signer = self
            .signer
            .as_ref()
            .ok_or(BridgeError::ConfigMissing("ETH_PRIVATE_KEY_ENV or signer"))?;
        let bridge_addr = self
            .bridge_address
            .ok_or(BridgeError::ConfigMissing("ETH_BRIDGE_ADDRESS"))?;
        let token_addr = self
            .token_address
            .ok_or(BridgeError::ConfigMissing("ETH_TOKEN_ADDRESS"))?;

        let recipient_addr: Address = recipient
            .parse()
            .map_err(|_| BridgeError::Other("Invalid recipient address".into()))?;
        let erc20 = Erc20::new(token_addr, signer.clone());
        let decimals: u8 = erc20
            .decimals()
            .call()
            .await
            .map_err(|e| BridgeError::RpcUnavailable(e.to_string()))?;
        let scale = 10u128.pow(decimals as u32);
        let scaled = U256::from(((amount * scale as f64).round() as u128));

        let bridge = Bridge::new(bridge_addr, signer.clone());
        let h = if let Some(l) = lock_id {
            l.parse::<H256>()
                .map_err(|_| BridgeError::Other("Invalid lockId".into()))?
        } else {
            H256::zero()
        };
        let pending = bridge
            .mint(token_addr, recipient_addr, scaled, h.into())
            .send()
            .await
            .map_err(|e| BridgeError::TxFailed(e.to_string()))?;
        let tx_hash = *pending;
        Ok(format!("0x{:x}", tx_hash))
    }

    async fn get_tx_status(&self, tx_id: &str) -> Result<AdapterTxStatus, BridgeError> {
        let provider = self
            .provider
            .as_ref()
            .ok_or(BridgeError::ConfigMissing("ETH_RPC_URL"))?;
        let tx_hash: H256 = tx_id
            .parse()
            .map_err(|_| BridgeError::Other("Invalid tx hash".into()))?;
        let receipt = provider
            .get_transaction_receipt(tx_hash)
            .await
            .map_err(|e| BridgeError::RpcUnavailable(e.to_string()))?;

        let mut status = "Pending".to_string();
        let mut confirmations = 0u32;
        if let Some(r) = receipt {
            status = if r.status.unwrap_or_default().as_u64() == 1 {
                "Success"
            } else {
                "Failed"
            }
            .to_string();
            if let Some(block) = r.block_number {
                let latest = provider
                    .get_block_number()
                    .await
                    .map_err(|e| BridgeError::RpcUnavailable(e.to_string()))?;
                if latest > block {
                    confirmations = (latest - block).as_u32();
                }
            }
        }
        Ok(AdapterTxStatus {
            tx_id: tx_id.to_string(),
            status,
            confirmations,
        })
    }

    async fn extract_lock_id(&self, tx_hash: &str) -> Result<Option<String>, BridgeError> {
        use ethers::abi::RawLog;
        use ethers::contract::EthEvent;

        let provider = self
            .provider
            .as_ref()
            .ok_or(BridgeError::ConfigMissing("ETH_RPC_URL"))?;
        let bridge_addr = self
            .bridge_address
            .ok_or(BridgeError::ConfigMissing("ETH_BRIDGE_ADDRESS"))?;
        let h: H256 = tx_hash
            .parse()
            .map_err(|_| BridgeError::Other("Invalid tx hash".into()))?;
        let receipt = provider
            .get_transaction_receipt(h)
            .await
            .map_err(|e| BridgeError::RpcUnavailable(e.to_string()))?;
        if let Some(r) = receipt {
            for lg in r.logs {
                if lg.address == bridge_addr {
                    let raw = RawLog {
                        topics: lg.topics.clone(),
                        data: lg.data.to_vec(),
                    };
                    if let Ok(parsed) = LockedFilter::decode_log(&raw) {
                        let id: H256 = H256::from(parsed.lock_id);
                        return Ok(Some(format!("0x{:x}", id)));
                    }
                }
            }
        }
        Ok(None)
    }
}
