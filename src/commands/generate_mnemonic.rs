use anyhow::{Context, Result};
use ethers::{
    signers::{coins_bip39::{English, Mnemonic}, MnemonicBuilder, Signer},
};
use std::fs;

pub async fn execute(create_keystore: bool, output: Option<String>) -> Result<()> {
    println!("Generating new 24-word BIP39 mnemonic...\n");

    // Generate 24-word mnemonic using random number generator
    let mut rng = rand::thread_rng();
    let mnemonic = Mnemonic::<English>::new_with_count(&mut rng, 24)?;
    let phrase = mnemonic.to_phrase();

    // Display the mnemonic with clear warnings
    println!("═══════════════════════════════════════════════════════");
    println!("⚠️  IMPORTANT: Save these words in a secure location!");
    println!("⚠️  Anyone with these words can access your funds!");
    println!("⚠️  Keep them offline and never share them!");
    println!("═══════════════════════════════════════════════════════\n");

    println!("Your 24-word mnemonic phrase:\n");

    // Display words in a numbered, easy-to-read format
    let words: Vec<&str> = phrase.trim().split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        print!("{:2}. {:<12}", i + 1, word);
        if (i + 1) % 3 == 0 {
            println!();
        }
    }
    println!();

    println!("═══════════════════════════════════════════════════════\n");

    // If create-keystore flag is set, save the private key
    if create_keystore {
        println!("Saving private key to file...");

        // Derive wallet from mnemonic using default Ethereum path
        let wallet = MnemonicBuilder::<English>::default()
            .phrase(phrase.trim())
            .derivation_path("m/44'/60'/0'/0/0")?
            .build()?;

        println!("Derived address: {:?}", wallet.address());

        // Generate key file path
        let key_path = output.unwrap_or_else(|| {
            format!("private-key-{:?}.txt", wallet.address())
        });

        // Get private key in hex format
        let private_key_bytes = wallet.signer().to_bytes();
        let private_key_hex = format!("0x{}", hex::encode(private_key_bytes));

        // Write private key to file
        fs::write(&key_path, &private_key_hex)
            .context("Failed to write private key file")?;

        println!("\n⚠️  WARNING: Private key saved in PLAIN TEXT!");
        println!("⚠️  Keep this file EXTREMELY secure!");
        println!("⚠️  Anyone with this file can access your funds!");
        println!("\n✓ Private key saved successfully!");
        println!("  File: {}", key_path);
        println!("  Address: {:?}", wallet.address());
        println!("  Private key: {}", private_key_hex);
        println!("\n⚠️  Remember to save your mnemonic phrase separately!");
    } else {
        println!("To save the private key from this mnemonic later, use:");
        println!("  cold-deploy derive-key --output private-key.txt\n");
    }

    Ok(())
}
