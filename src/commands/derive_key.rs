use anyhow::{Context, Result};
use ethers::{
    signers::{coins_bip39::English, MnemonicBuilder, Signer},
};
use std::fs;
use std::io::{self, Write as _};
use std::path::Path;

use crate::constants::DEFAULT_ETH_DERIVATION_PATH;

pub async fn execute(mnemonic_file: Option<String>, output: Option<String>, plain_text: bool) -> Result<()> {
    if plain_text {
        println!("⚠️  WARNING: Creating PLAIN TEXT private key file!");
        println!("⚠️  Consider using encrypted keystore instead (default)\n");
        println!("Deriving private key from mnemonic...\n");
    } else {
        println!("Deriving encrypted keystore from mnemonic...\n");
    }

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
    println!("Deriving key using path: {}", DEFAULT_ETH_DERIVATION_PATH);

    let wallet = MnemonicBuilder::<English>::default()
        .phrase(mnemonic_phrase.as_str())
        .derivation_path(DEFAULT_ETH_DERIVATION_PATH)?
        .build()
        .context("Failed to derive wallet from mnemonic. Check that the mnemonic is valid.")?;

    let address = wallet.address();
    println!("\n✓ Key derived successfully!");
    println!("  Address: {:?}", address);

    if plain_text {
        // Plain text mode - save raw private key
        save_plain_text_key(&wallet, output, address).await?;
    } else {
        // Encrypted keystore mode (default)
        save_encrypted_keystore(&wallet, output, address).await?;
    }

    println!("\n⚠️  Keep your mnemonic phrase backed up in a safe location!");
    Ok(())
}

async fn save_plain_text_key(
    wallet: &ethers::signers::Wallet<ethers::core::k256::ecdsa::SigningKey>,
    output: Option<String>,
    address: ethers::types::Address,
) -> Result<()> {
    // Get private key in hex format
    let private_key_bytes = wallet.signer().to_bytes();
    let private_key_hex = format!("0x{}", hex::encode(private_key_bytes));

    // Generate default filename if not provided
    let file_path = output.unwrap_or_else(|| {
        format!("private-key-{:?}.txt", address)
    });

    // Save to output file
    println!("\nSaving private key to plain text file...");
    fs::write(&file_path, &private_key_hex)
        .context("Failed to write private key file")?;

    println!("\n⚠️  WARNING: Private key saved in PLAIN TEXT!");
    println!("⚠️  Keep this file EXTREMELY secure!");
    println!("⚠️  Anyone with this file can access your funds!");
    println!("\n✓ Private key saved successfully!");
    println!("  File: {}", file_path);
    println!("  Address: {:?}", address);
    println!("  Private key: {}", private_key_hex);

    Ok(())
}

async fn save_encrypted_keystore(
    wallet: &ethers::signers::Wallet<ethers::core::k256::ecdsa::SigningKey>,
    output: Option<String>,
    address: ethers::types::Address,
) -> Result<()> {
    // Prompt for password
    println!("\nCreate a strong password to encrypt your keystore:");
    println!("(Password must be at least 8 characters)");
    let password = read_password("Enter password: ")?;

    // Validate password length
    if password.len() < 8 {
        anyhow::bail!("Password must be at least 8 characters long");
    }

    // Confirm password
    let password_confirm = read_password("Confirm password: ")?;

    if password != password_confirm {
        anyhow::bail!("Passwords do not match!");
    }

    // Generate default filename if not provided
    let file_path = output.unwrap_or_else(|| {
        format!("keystore-{:?}.json", address)
    });

    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(&file_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .context("Failed to create output directory")?;
        }
    }

    // Encrypt and save keystore
    println!("\nEncrypting keystore...");
    let mut rng = rand::thread_rng();

    // Extract directory and filename from the file path
    let path = Path::new(&file_path);
    let dir = path.parent().unwrap_or(Path::new("."));
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .context("Invalid filename")?;

    // encrypt_key() writes the file to disk automatically
    eth_keystore::encrypt_key(
        dir,
        &mut rng,
        wallet.signer().to_bytes(),
        &password,
        Some(filename),
    ).context("Failed to encrypt keystore")?;

    println!("\n✓ Encrypted keystore saved successfully!");
    println!("  File: {}", file_path);
    println!("  Address: {:?}", address);
    println!("\n✓ Your private key is now protected with encryption!");
    println!("  Remember your password - it CANNOT be recovered if lost!");

    Ok(())
}

fn read_password(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    rpassword::read_password()
        .context("Failed to read password")
}
