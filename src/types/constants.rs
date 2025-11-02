//! Common constants for networks and schemes

/// Common network configurations
pub mod networks {
    /// Base mainnet configuration
    pub const BASE_MAINNET: &str = "base";
    /// Base Sepolia testnet configuration
    pub const BASE_SEPOLIA: &str = "base-sepolia";
    /// Avalanche mainnet configuration
    pub const AVALANCHE_MAINNET: &str = "avalanche";
    /// Avalanche Fuji testnet configuration
    pub const AVALANCHE_FUJI: &str = "avalanche-fuji";

    /// Get USDC contract address for a network
    pub fn get_usdc_address(network: &str) -> Option<&'static str> {
        match network {
            BASE_MAINNET => Some("0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"),
            BASE_SEPOLIA => Some("0x036CbD53842c5426634e7929541eC2318f3dCF7e"),
            AVALANCHE_MAINNET => Some("0xB97EF9Ef8734C71904D8002F8b6Bc66Dd9c48a6E"),
            AVALANCHE_FUJI => Some("0x5425890298aed601595a70AB815c96711a31Bc65"),
            _ => None,
        }
    }

    /// Check if a network is supported
    pub fn is_supported(network: &str) -> bool {
        matches!(
            network,
            BASE_MAINNET | BASE_SEPOLIA | AVALANCHE_MAINNET | AVALANCHE_FUJI
        )
    }

    /// Get all supported networks
    pub fn all_supported() -> Vec<&'static str> {
        vec![
            BASE_MAINNET,
            BASE_SEPOLIA,
            AVALANCHE_MAINNET,
            AVALANCHE_FUJI,
        ]
    }
}

/// Common payment schemes
pub mod schemes {
    /// Exact payment scheme (EIP-3009)
    pub const EXACT: &str = "exact";
}
