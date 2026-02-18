use anyhow::{Context, Result};
use ethers::{
    abi::{Abi, Token},
    providers::{Http, Middleware, Provider},
    types::{H160, U256},
};
use std::fs;
use std::str::FromStr;

use crate::types::prepare_output::UnsignedTransaction;
use crate::utils::contract;

pub async fn execute(
    contract_path: String,
    rpc_url: String,
    from: String,
    args: Option<String>,
    output: String,
    gas_limit: Option<u64>,
) -> Result<()> {
    println!("Preparing unsigned transaction...");
    println!("Contract: {}", contract_path);
    println!("From: {}", from);

    // Parse contract JSON to extract bytecode and ABI
    let (bytecode, abi_value) = contract::parse_contract_json(&contract_path)
        .context("Failed to parse contract JSON")?;

    let abi: Abi = serde_json::from_value(abi_value)
        .context("Failed to parse ABI")?;

    // Encode constructor arguments if provided
    let constructor_data = if let Some(args_str) = args {
        if let Some(constructor) = abi.constructor() {
            let args_vec: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

            if args_vec.len() != constructor.inputs.len() {
                anyhow::bail!(
                    "Constructor expects {} argument(s) but {} were provided",
                    constructor.inputs.len(),
                    args_vec.len()
                );
            }

            let tokens: Vec<Token> = args_vec
                .iter()
                .zip(constructor.inputs.iter())
                .map(|(arg, param)| {
                    parse_arg_to_token(arg, &param.kind)
                })
                .collect::<Result<Vec<_>>>()
                .context("Failed to parse constructor arguments")?;

            let bytecode_bytes = hex::decode(&bytecode)
                .context("Failed to decode bytecode hex")?;
            constructor.encode_input(bytecode_bytes, &tokens)
                .context("Failed to encode constructor")?
        } else {
            anyhow::bail!("Contract has no constructor but arguments were provided");
        }
    } else {
        // Validate that the constructor does not require parameters
        if let Some(constructor) = abi.constructor() {
            if !constructor.inputs.is_empty() {
                let param_list: Vec<String> = constructor
                    .inputs
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.kind))
                    .collect();
                anyhow::bail!(
                    "Constructor requires {} parameter(s) but none were provided: {}\n\
                     Use --args to supply constructor arguments.",
                    constructor.inputs.len(),
                    param_list.join(", ")
                );
            }
        }
        hex::decode(&bytecode).context("Failed to decode bytecode")?
    };

    // Connect to RPC provider
    println!("Connecting to RPC: {}", rpc_url);
    let provider = Provider::<Http>::try_from(&rpc_url)
        .context("Failed to create provider")?;

    // Parse from address
    let from_addr = H160::from_str(&from)
        .context("Invalid from address")?;

    // Fetch chain ID from RPC
    println!("Fetching chain ID from RPC...");
    let chain_id = provider.get_chainid()
        .await
        .context("Failed to fetch chain ID from RPC")?
        .as_u64();
    println!("Chain ID: {}", chain_id);

    // Fetch nonce
    println!("Fetching nonce for address: {}", from);
    let nonce = provider.get_transaction_count(from_addr, None)
        .await
        .context("Failed to fetch nonce")?;

    // Fetch fee data (EIP-1559 or legacy)
    println!("Fetching gas price information...");
    let fee_data = provider.fee_history(1, ethers::types::BlockNumber::Latest, &[])
        .await;

    let (max_fee_per_gas, max_priority_fee_per_gas, gas_price) = if let Ok(fee_history) = fee_data {
        // EIP-1559
        let base_fee = fee_history.base_fee_per_gas.first()
            .copied()
            .unwrap_or(U256::from(1_000_000_000u64)); // 1 gwei default

        let priority_fee = U256::from(1_500_000_000u64); // 1.5 gwei
        let max_fee: U256 = base_fee * 2 + priority_fee;

        (Some(max_fee.as_u64()), Some(priority_fee.as_u64()), None)
    } else {
        // Legacy gas price
        let gas_price = provider.get_gas_price()
            .await
            .context("Failed to fetch gas price")?;
        (None, None, Some(gas_price.as_u64()))
    };

    // Estimate gas limit if not provided
    let estimated_gas = gas_limit.unwrap_or_else(|| {
        // Default gas limit for contract deployment
        3_000_000u64
    });

    // Create unsigned transaction
    let unsigned_tx = UnsignedTransaction {
        to: None, // Contract deployment has no recipient
        data: hex::encode(&constructor_data),
        nonce: nonce.as_u64(),
        gas_limit: estimated_gas,
        gas_price,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        chain_id,
        value: "0".to_string(),
        rpc_url: rpc_url.clone(),
    };

    // Save to output file
    println!("Saving unsigned transaction to: {}", output);
    let json = serde_json::to_string_pretty(&unsigned_tx)
        .context("Failed to serialize transaction")?;

    fs::write(&output, json)
        .context("Failed to write output file")?;

    println!("\n✓ Unsigned transaction prepared successfully!");
    println!("  Nonce: {}", unsigned_tx.nonce);
    println!("  Gas limit: {}", unsigned_tx.gas_limit);
    if let Some(gp) = unsigned_tx.gas_price {
        println!("  Gas price: {} gwei", gp / 1_000_000_000);
    } else {
        println!("  Max fee per gas: {} gwei", unsigned_tx.max_fee_per_gas.unwrap() / 1_000_000_000);
        println!("  Max priority fee per gas: {} gwei", unsigned_tx.max_priority_fee_per_gas.unwrap() / 1_000_000_000);
    }

    Ok(())
}

fn parse_arg_to_token(arg: &str, param_type: &ethers::abi::ParamType) -> Result<Token> {
    use ethers::abi::ParamType;

    match param_type {
        ParamType::Address => {
            let addr = H160::from_str(arg)?;
            Ok(Token::Address(addr))
        }
        ParamType::Uint(_) => {
            let value = U256::from_dec_str(arg)?;
            Ok(Token::Uint(value))
        }
        ParamType::Int(_) => {
            let value = U256::from_dec_str(arg)?;
            Ok(Token::Int(value))
        }
        ParamType::Bool => {
            let value = arg.parse::<bool>()?;
            Ok(Token::Bool(value))
        }
        ParamType::String => {
            Ok(Token::String(arg.to_string()))
        }
        ParamType::Bytes => {
            let bytes = hex::decode(arg.strip_prefix("0x").unwrap_or(arg))?;
            Ok(Token::Bytes(bytes))
        }
        _ => anyhow::bail!("Unsupported parameter type: {:?}", param_type),
    }
}
