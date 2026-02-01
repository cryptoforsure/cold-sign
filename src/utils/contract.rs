use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

#[derive(serde::Deserialize)]
struct SolcOutput {
    bytecode: Option<String>,
    abi: Option<Value>,
}

pub fn parse_contract_json(path: &str) -> Result<(String, Value)> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read contract file: {}", path))?;

    let parsed: SolcOutput = serde_json::from_str(&content)
        .with_context(|| "Failed to parse contract JSON. Expected Solidity compiler output format")?;

    let bytecode = parsed.bytecode
        .ok_or_else(|| anyhow::anyhow!("No bytecode found in contract JSON"))?;

    let abi = parsed.abi
        .ok_or_else(|| anyhow::anyhow!("No ABI found in contract JSON"))?;

    // Remove 0x prefix if present
    let bytecode = bytecode.strip_prefix("0x").unwrap_or(&bytecode).to_string();

    Ok((bytecode, abi))
}
