use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum BridgeError {
    UnsupportedChain(String),
    UnsupportedToken(String),
    RpcUnavailable(String),
    ConfigMissing(&'static str),
    SigningError(String),
    TxFailed(String),
    Unimplemented(&'static str),
    Other(String),
}

impl Display for BridgeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::UnsupportedChain(s) => write!(f, "Unsupported chain: {}", s),
            BridgeError::UnsupportedToken(s) => write!(f, "Unsupported token: {}", s),
            BridgeError::RpcUnavailable(s) => write!(f, "RPC unavailable: {}", s),
            BridgeError::ConfigMissing(k) => write!(f, "Missing config: {}", k),
            BridgeError::SigningError(s) => write!(f, "Signing error: {}", s),
            BridgeError::TxFailed(s) => write!(f, "Transaction failed: {}", s),
            BridgeError::Unimplemented(s) => write!(f, "Unimplemented: {}", s),
            BridgeError::Other(s) => write!(f, "{}", s),
        }
    }
}

impl Error for BridgeError {}

