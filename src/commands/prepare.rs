use anyhow::{Context, Result};
use ethers::{
    abi::{Abi, Token},
    providers::{Http, Middleware, Provider},
    types::{H160, U256},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::str::FromStr;

use crate::types::prepare_output::UnsignedTransaction;
use crate::utils::contract;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareParams {
    pub contract: String,
    pub rpc_url: String,
    pub from: String,
    pub to: Option<String>,
    pub function_name: Option<String>,
    pub args: Option<String>,
    pub value: String,
    pub output: String,
    pub gas_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrepareResult {
    pub unsigned_tx: UnsignedTransaction,
    pub success: bool,
    pub message: String,
}

pub async fn run(params: PrepareParams) -> Result<PrepareResult> {
    let contract_path = params.contract;
    let rpc_url = params.rpc_url;
    let from = params.from;
    let to = params.to;
    let function_name = params.function_name;
    let args = params.args;
    let value = params.value;
    let output = params.output;
    let gas_limit = params.gas_limit;
    println!("Preparing unsigned transaction...");
    println!("Contract: {}", contract_path);
    println!("From: {}", from);

    // Parse contract JSON to extract bytecode and ABI
    let (bytecode, abi_value) = contract::parse_contract_json(&contract_path)
        .context("Failed to parse contract JSON")?;

    let abi: Abi = serde_json::from_value(abi_value)
        .context("Failed to parse ABI")?;

    // Determine call mode vs. deploy mode
    let is_call_mode = to.is_some() && function_name.is_some();

    // Build transaction data
    let (tx_to, tx_data) = if is_call_mode {
        // ── Call mode: encode a function call ──────────────────────────────
        let to_str = to.as_deref().unwrap();
        let func_name = function_name.as_deref().unwrap();

        println!("Mode: Function call");
        println!("To: {}", to_str);
        println!("Function: {}", func_name);

        // Validate the 'to' address
        let to_addr = H160::from_str(to_str)
            .with_context(|| format!("Invalid contract address: {}", to_str))?;

        // Look up the function in ABI
        let function = abi
            .function(func_name)
            .with_context(|| format!("Function '{}' not found in ABI", func_name))?;

        // Encode function call data
        let call_data = if let Some(args_str) = args {
            let args_vec: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

            if args_vec.len() != function.inputs.len() {
                anyhow::bail!(
                    "Function '{}' expects {} argument(s) but {} were provided",
                    func_name,
                    function.inputs.len(),
                    args_vec.len()
                );
            }

            let tokens: Vec<Token> = args_vec
                .iter()
                .zip(function.inputs.iter())
                .map(|(arg, param)| parse_arg_to_token(arg, &param.kind))
                .collect::<Result<Vec<_>>>()
                .context("Failed to parse function arguments")?;

            function
                .encode_input(&tokens)
                .context("Failed to encode function call")?
        } else {
            if !function.inputs.is_empty() {
                let param_list: Vec<String> = function
                    .inputs
                    .iter()
                    .map(|p| format!("{}: {}", p.name, p.kind))
                    .collect();
                anyhow::bail!(
                    "Function '{}' requires {} parameter(s) but none were provided: {}\n\
                     Use --args to supply function arguments.",
                    func_name,
                    function.inputs.len(),
                    param_list.join(", ")
                );
            }
            function
                .encode_input(&[])
                .context("Failed to encode function call")?
        };

        (Some(format!("{:?}", to_addr)), call_data)
    } else {
        // ── Deploy mode: bytecode + encoded constructor args ───────────────
        println!("Mode: Contract deployment");

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
                    .map(|(arg, param)| parse_arg_to_token(arg, &param.kind))
                    .collect::<Result<Vec<_>>>()
                    .context("Failed to parse constructor arguments")?;

                let bytecode_bytes = hex::decode(&bytecode)
                    .context("Failed to decode bytecode hex")?;
                constructor
                    .encode_input(bytecode_bytes, &tokens)
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

        (None, constructor_data)
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
    let chain_id = provider
        .get_chainid()
        .await
        .context("Failed to fetch chain ID from RPC")?
        .as_u64();
    println!("Chain ID: {}", chain_id);

    // Fetch nonce
    println!("Fetching nonce for address: {}", from);
    let nonce = provider
        .get_transaction_count(from_addr, None)
        .await
        .context("Failed to fetch nonce")?;

    // Fetch fee data (EIP-1559 or legacy)
    println!("Fetching gas price information...");
    let fee_data = provider
        .fee_history(1, ethers::types::BlockNumber::Latest, &[])
        .await;

    let (max_fee_per_gas, max_priority_fee_per_gas, gas_price) =
        if let Ok(fee_history) = fee_data {
            // EIP-1559
            let base_fee = fee_history
                .base_fee_per_gas
                .first()
                .copied()
                .unwrap_or(U256::from(1_000_000_000u64)); // 1 gwei default

            let priority_fee = U256::from(1_500_000_000u64); // 1.5 gwei
            let max_fee: U256 = base_fee * 2 + priority_fee;

            (Some(max_fee.as_u64()), Some(priority_fee.as_u64()), None)
        } else {
            // Legacy gas price
            let gas_price = provider
                .get_gas_price()
                .await
                .context("Failed to fetch gas price")?;
            (None, None, Some(gas_price.as_u64()))
        };

    // Gas limit
    let estimated_gas = gas_limit.unwrap_or(3_000_000u64);

    // Create unsigned transaction
    let unsigned_tx = UnsignedTransaction {
        to: tx_to,
        data: hex::encode(&tx_data),
        nonce: nonce.as_u64(),
        gas_limit: estimated_gas,
        gas_price,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        chain_id,
        value,
        rpc_url: rpc_url.clone(),
    };

    // Save to output file
    println!("Saving unsigned transaction to: {}", output);
    let json = serde_json::to_string_pretty(&unsigned_tx)
        .context("Failed to serialize transaction")?;

    fs::write(&output, json).context("Failed to write output file")?;

    println!("\n✓ Unsigned transaction prepared successfully!");
    println!("  Nonce: {}", unsigned_tx.nonce);
    println!("  Gas limit: {}", unsigned_tx.gas_limit);
    if let Some(gp) = unsigned_tx.gas_price {
        println!("  Gas price: {} gwei", gp / 1_000_000_000);
    } else {
        println!(
            "  Max fee per gas: {} gwei",
            unsigned_tx.max_fee_per_gas.unwrap() / 1_000_000_000
        );
        println!(
            "  Max priority fee per gas: {} gwei",
            unsigned_tx.max_priority_fee_per_gas.unwrap() / 1_000_000_000
        );
    }

    let message = "Unsigned transaction prepared successfully!".to_string();

    Ok(PrepareResult {
        unsigned_tx,
        success: true,
        message,
    })
}

pub async fn execute(
    contract_path: String,
    rpc_url: String,
    from: String,
    to: Option<String>,
    function_name: Option<String>,
    args: Option<String>,
    value: String,
    output: String,
    gas_limit: Option<u64>,
) -> Result<()> {
    let params = PrepareParams {
        contract: contract_path,
        rpc_url,
        from,
        to,
        function_name,
        args,
        value,
        output,
        gas_limit,
    };

    run(params).await?;
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
        ParamType::String => Ok(Token::String(arg.to_string())),
        ParamType::Bytes => {
            let bytes = hex::decode(arg.strip_prefix("0x").unwrap_or(arg))?;
            Ok(Token::Bytes(bytes))
        }
        _ => anyhow::bail!("Unsupported parameter type: {:?}", param_type),
    }
}
