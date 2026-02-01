use anyhow::{Context, Result};
use ethers::{
    signers::{coins_bip39::English, MnemonicBuilder, Signer},
};
use std::fs;
use std::io::{self, Write as _};

pub async fn execute(mnemonic_file: Option<String>, output: String) -> Result<()> {
    println!("Deriving private key from mnemonic...\n");

    // Get mnemonic phrase either from file or user input
    let mnemonic_phrase = if let Some(file_path) = mnemonic_file {
        println!("Reading mnemonic from file: {}", file_path);
        fs::read_to_string(&file_path)
            .context("Failed to read mnemonic file")?
            .trim()
            .to_string()
    } else {
        println!("Enter your 24-word mnemonic phrase:");
        println!("(paste all words separated by spaces, then press Enter)\n");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read mnemonic from input")?;

        input.trim().to_string()
    };

    // Validate mnemonic has 24 words
    let word_count = mnemonic_phrase.split_whitespace().count();
    if word_count != 24 {
        anyhow::bail!(
            "Invalid mnemonic: expected 24 words, got {}. Please check your mnemonic phrase.",
            word_count
        );
    }

    println!("Mnemonic validated: {} words", word_count);

    // Derive wallet from mnemonic using default Ethereum path (m/44'/60'/0'/0/0)
    println!("Deriving key using path: m/44'/60'/0'/0/0");

    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic_phrase.as_str())
        .derivation_path("m/44'/60'/0'/0/0")?
        .build()
        .context("Failed to derive wallet from mnemonic. Check that the mnemonic is valid.")?;

    println!("\n✓ Key derived successfully!");
    println!("  Address: {:?}", wallet.address());

    // Get private key in hex format
    let private_key_bytes = wallet.signer().to_bytes();
    let private_key_hex = format!("0x{}", hex::encode(private_key_bytes));

    // Save to output file
    println!("\nSaving private key to file...");
    fs::write(&output, &private_key_hex)
        .context("Failed to write private key file")?;

    println!("\n⚠️  WARNING: Private key saved in PLAIN TEXT!");
    println!("⚠️  Keep this file EXTREMELY secure!");
    println!("⚠️  Anyone with this file can access your funds!");
    println!("\n✓ Private key saved successfully!");
    println!("  File: {}", output);
    println!("  Address: {:?}", wallet.address());
    println!("  Private key: {}", private_key_hex);
    println!("\n⚠️  Keep your mnemonic phrase backed up in a safe location!");

    Ok(())
}
