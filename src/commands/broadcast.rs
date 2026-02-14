use anyhow::{Context, Result};
use ethers::{
    providers::{Http, Middleware, Provider},
    types::Bytes,
};
use std::fs;

use crate::types::sign_output::SignedTransaction;

pub async fn execute(signed_path: String) -> Result<()> {
    println!("Broadcasting transaction...");
    println!("Loading signed transaction from: {}", signed_path);

    // Load signed transaction
    let signed_json = fs::read_to_string(&signed_path)
        .context("Failed to read signed transaction file")?;

    let signed_tx: SignedTransaction = serde_json::from_str(&signed_json)
        .context("Failed to parse signed transaction JSON")?;

    // Validate transaction hash matches what's expected
    println!("Transaction hash: {}", signed_tx.transaction_hash);
    println!("From: {}", signed_tx.from);
    println!("Nonce: {}", signed_tx.nonce);

    // Use RPC URL from signed transaction
    let rpc_url = &signed_tx.rpc_url;
    println!("\nConnecting to RPC: {}", rpc_url);
    let provider = Provider::<Http>::try_from(rpc_url)
        .context("Failed to create provider")?;

    // Verify chain ID matches
    println!("Verifying chain ID...");
    let rpc_chain_id = provider.get_chainid()
        .await
        .context("Failed to fetch chain ID from RPC")?
        .as_u64();

    if rpc_chain_id != signed_tx.chain_id {
        anyhow::bail!(
            "Chain ID mismatch! Transaction signed for chain {} but RPC is on chain {}",
            signed_tx.chain_id,
            rpc_chain_id
        );
    }
    println!("Chain ID verified: {}", rpc_chain_id);

    // Decode raw transaction
    let raw_tx = signed_tx.raw_transaction.strip_prefix("0x")
        .unwrap_or(&signed_tx.raw_transaction);
    let tx_bytes = hex::decode(raw_tx)
        .context("Failed to decode raw transaction")?;

    // Send raw transaction
    println!("Broadcasting transaction to network...");
    let pending_tx = provider.send_raw_transaction(Bytes::from(tx_bytes))
        .await
        .context("Failed to send transaction to network")?;

    println!("\n✓ Transaction broadcast successfully!");
    println!("  Transaction hash: {:?}", pending_tx.tx_hash());

    // Wait for confirmation
    println!("\nWaiting for transaction confirmation...");
    match pending_tx.await {
        Ok(Some(receipt)) => {
            println!("\n✓ Transaction confirmed!");
            println!("  Block number: {}", receipt.block_number.unwrap());
            println!("  Gas used: {}", receipt.gas_used.unwrap());
            println!("  Status: {}", if receipt.status.unwrap().as_u64() == 1 { "Success" } else { "Failed" });

            // If contract deployment, show contract address
            if let Some(contract_address) = receipt.contract_address {
                println!("\n✓ Contract deployed!");
                println!("  Contract address: {:?}", contract_address);
            }
        }
        Ok(None) => {
            println!("\n⚠ Transaction was dropped from the mempool");
        }
        Err(e) => {
            println!("\n✗ Transaction failed: {:?}", e);
            anyhow::bail!("Transaction failed");
        }
    }

    Ok(())
}
