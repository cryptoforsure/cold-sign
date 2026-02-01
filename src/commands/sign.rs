use anyhow::{Context, Result};
use ethers::{
    prelude::*,
    signers::Signer,
    types::transaction::eip2718::TypedTransaction,
};
use std::fs;

use crate::types::prepare_output::UnsignedTransaction;
use crate::types::sign_output::SignedTransaction;

pub async fn execute(unsigned_path: String, keystore_path: String, output: String) -> Result<()> {
    println!("Signing transaction...");
    println!("Loading unsigned transaction from: {}", unsigned_path);

    // Load unsigned transaction
    let unsigned_json = fs::read_to_string(&unsigned_path)
        .context("Failed to read unsigned transaction file")?;

    let unsigned_tx: UnsignedTransaction = serde_json::from_str(&unsigned_json)
        .context("Failed to parse unsigned transaction JSON")?;

    // Prompt for password
    println!("Enter keystore password:");
    let password = rpassword::read_password()
        .context("Failed to read password")?;

    // Load and decrypt keystore
    println!("Loading keystore from: {}", keystore_path);
    let wallet = LocalWallet::decrypt_keystore(&keystore_path, &password)
        .context("Failed to decrypt keystore. Check password and keystore file")?;

    println!("Keystore loaded successfully!");
    println!("Address: {:?}", wallet.address());

    // Build transaction
    let mut tx: TypedTransaction = if unsigned_tx.max_fee_per_gas.is_some() {
        // EIP-1559 transaction
        let mut eip1559 = Eip1559TransactionRequest::new();
        eip1559 = eip1559.chain_id(unsigned_tx.chain_id);
        eip1559 = eip1559.nonce(unsigned_tx.nonce);
        eip1559 = eip1559.gas(unsigned_tx.gas_limit);
        eip1559 = eip1559.max_fee_per_gas(unsigned_tx.max_fee_per_gas.unwrap());
        eip1559 = eip1559.max_priority_fee_per_gas(unsigned_tx.max_priority_fee_per_gas.unwrap());

        if let Some(ref to) = unsigned_tx.to {
            let to_addr: Address = to.parse()
                .context("Invalid to address")?;
            eip1559 = eip1559.to(to_addr);
        }

        let data = hex::decode(&unsigned_tx.data)
            .context("Failed to decode transaction data")?;
        eip1559 = eip1559.data(data);

        let value: U256 = unsigned_tx.value.parse()
            .context("Failed to parse value")?;
        eip1559 = eip1559.value(value);

        TypedTransaction::Eip1559(eip1559)
    } else {
        // Legacy transaction
        let mut legacy = TransactionRequest::new();
        legacy = legacy.chain_id(unsigned_tx.chain_id);
        legacy = legacy.nonce(unsigned_tx.nonce);
        legacy = legacy.gas(unsigned_tx.gas_limit);
        legacy = legacy.gas_price(unsigned_tx.gas_price.unwrap());

        if let Some(ref to) = unsigned_tx.to {
            let to_addr: Address = to.parse()
                .context("Invalid to address")?;
            legacy = legacy.to(to_addr);
        }

        let data = hex::decode(&unsigned_tx.data)
            .context("Failed to decode transaction data")?;
        legacy = legacy.data(data);

        let value: U256 = unsigned_tx.value.parse()
            .context("Failed to parse value")?;
        legacy = legacy.value(value);

        TypedTransaction::Legacy(legacy)
    };

    // Sign transaction
    println!("Signing transaction...");
    let signature = wallet.sign_transaction(&tx)
        .await
        .context("Failed to sign transaction")?;

    // Set the signature on the transaction
    tx.set_from(wallet.address());

    // Encode the signed transaction
    let rlp_signed = tx.rlp_signed(&signature);
    let raw_transaction = hex::encode(&rlp_signed);

    // Calculate transaction hash
    let tx_hash = ethers::utils::keccak256(&rlp_signed);
    let transaction_hash = format!("0x{}", hex::encode(tx_hash));

    // Create signed transaction output
    let signed_tx = SignedTransaction {
        raw_transaction: format!("0x{}", raw_transaction),
        transaction_hash: transaction_hash.clone(),
        from: format!("{:?}", wallet.address()),
        to: unsigned_tx.to.clone(),
        nonce: unsigned_tx.nonce,
        chain_id: unsigned_tx.chain_id,
    };

    // Save to output file
    println!("Saving signed transaction to: {}", output);
    let json = serde_json::to_string_pretty(&signed_tx)
        .context("Failed to serialize signed transaction")?;

    fs::write(&output, json)
        .context("Failed to write output file")?;

    println!("\n✓ Transaction signed successfully!");
    println!("  Transaction hash: {}", transaction_hash);
    println!("  From: {}", signed_tx.from);
    println!("  Nonce: {}", signed_tx.nonce);

    Ok(())
}
