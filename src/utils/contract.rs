use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

#[derive(serde::Deserialize)]
struct SolcOutput {
    bytecode: Option<Value>,
    abi: Option<Value>,
}

pub fn parse_contract_json(path: &str) -> Result<(String, Value)> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read contract file: {}", path))?;

    let parsed: SolcOutput = serde_json::from_str(&content)
        .with_context(|| "Failed to parse contract JSON. Expected Solidity compiler output format")?;

    let bytecode_value = parsed.bytecode
        .ok_or_else(|| anyhow::anyhow!("No bytecode found in contract JSON"))?;

    // Support both a plain string and an object with an "object" field (e.g. Hardhat artifacts)
    let bytecode = match &bytecode_value {
        Value::String(s) => s.clone(),
        Value::Object(map) => map
            .get("object")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("bytecode object has no \"object\" string field"))?
            .to_string(),
        _ => anyhow::bail!("Unexpected bytecode format in contract JSON"),
    };

    let abi = parsed.abi
        .ok_or_else(|| anyhow::anyhow!("No ABI found in contract JSON"))?;

    // Remove 0x prefix if present
    let bytecode = bytecode.strip_prefix("0x").unwrap_or(&bytecode).to_string();

    Ok((bytecode, abi))
}
