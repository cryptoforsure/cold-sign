use anyhow::Result;

/// Build Infura RPC URL from network name and API key
pub fn build_infura_url(network: &str, api_key: &str) -> Result<String> {
    let url = match network.to_lowercase().as_str() {
        "mainnet" | "ethereum" => format!("https://mainnet.infura.io/v3/{}", api_key),
        "sepolia" => format!("https://sepolia.infura.io/v3/{}", api_key),
        "goerli" => format!("https://goerli.infura.io/v3/{}", api_key),
        "holesky" => format!("https://holesky.infura.io/v3/{}", api_key),
        "polygon" | "polygon-mainnet" => format!("https://polygon-mainnet.infura.io/v3/{}", api_key),
        "polygon-amoy" => format!("https://polygon-amoy.infura.io/v3/{}", api_key),
        "arbitrum" | "arbitrum-mainnet" => format!("https://arbitrum-mainnet.infura.io/v3/{}", api_key),
        "arbitrum-sepolia" => format!("https://arbitrum-sepolia.infura.io/v3/{}", api_key),
        "optimism" | "optimism-mainnet" => format!("https://optimism-mainnet.infura.io/v3/{}", api_key),
        "optimism-sepolia" => format!("https://optimism-sepolia.infura.io/v3/{}", api_key),
        "base" | "base-mainnet" => format!("https://base-mainnet.infura.io/v3/{}", api_key),
        "base-sepolia" => format!("https://base-sepolia.infura.io/v3/{}", api_key),
        "avalanche" | "avalanche-mainnet" | "avalanche-c-chain" => {
            format!("https://avalanche-mainnet.infura.io/v3/{}", api_key)
        }
        "avalanche-fuji" => format!("https://avalanche-fuji.infura.io/v3/{}", api_key),
        "linea" | "linea-mainnet" => format!("https://linea-mainnet.infura.io/v3/{}", api_key),
        "linea-sepolia" => format!("https://linea-sepolia.infura.io/v3/{}", api_key),
        _ => {
            anyhow::bail!(
                "Unsupported network: '{}'. Supported networks: mainnet, sepolia, goerli, holesky, polygon, polygon-amoy, arbitrum, arbitrum-sepolia, optimism, optimism-sepolia, base, base-sepolia, avalanche, avalanche-fuji, linea, linea-sepolia",
                network
            );
        }
    };
    Ok(url)
}

/// Resolve RPC URL from either direct URL or network name + Infura key
pub fn resolve_rpc_url(
    rpc_url: Option<String>,
    network: Option<String>,
    infura_key: Option<String>,
) -> Result<String> {
    if let Some(url) = rpc_url {
        Ok(url)
    } else if let (Some(net), Some(key)) = (network, infura_key) {
        build_infura_url(&net, &key)
    } else {
        anyhow::bail!("Must specify either --rpc-url OR (--network and --infura-key)")
    }
}
