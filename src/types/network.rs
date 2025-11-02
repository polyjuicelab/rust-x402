//! Network configuration types

/// Network configuration for x402 payments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

/// Network configuration with chain-specific details
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Chain ID for the network
    pub chain_id: u64,
    /// USDC contract address
    pub usdc_contract: String,
    /// Network name
    pub name: String,
    /// Whether this is a testnet
    pub is_testnet: bool,
}

impl NetworkConfig {
    /// Base mainnet configuration
    pub fn base_mainnet() -> Self {
        Self {
            chain_id: 8453,
            usdc_contract: "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string(),
            name: "base".to_string(),
            is_testnet: false,
        }
    }

    /// Base Sepolia testnet configuration
    pub fn base_sepolia() -> Self {
        Self {
            chain_id: 84532,
            usdc_contract: "0x036CbD53842c5426634e7929541eC2318f3dCF7e".to_string(),
            name: "base-sepolia".to_string(),
            is_testnet: true,
        }
    }

    /// Get network config by name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "base" => Some(Self::base_mainnet()),
            "base-sepolia" => Some(Self::base_sepolia()),
            _ => None,
        }
    }
}

impl Network {
    /// Get the network identifier string
    pub fn as_str(&self) -> &'static str {
        match self {
            Network::Mainnet => "base",
            Network::Testnet => "base-sepolia",
        }
    }

    /// Get the USDC contract address for this network
    pub fn usdc_address(&self) -> &'static str {
        match self {
            Network::Mainnet => "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
            Network::Testnet => "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
        }
    }

    /// Get the USDC token name for this network
    pub fn usdc_name(&self) -> &'static str {
        match self {
            Network::Mainnet => "USD Coin",
            Network::Testnet => "USDC",
        }
    }
}
