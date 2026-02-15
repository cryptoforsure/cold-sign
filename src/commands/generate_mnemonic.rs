use anyhow::Result;
use ethers::{
    signers::{coins_bip39::{English, Mnemonic}, MnemonicBuilder, Signer},
};

pub async fn execute() -> Result<()> {
    println!("Generating new 24-word BIP39 mnemonic...\n");

    // Generate 24-word mnemonic using random number generator
    let mut rng = rand::thread_rng();
    let mnemonic = Mnemonic::<English>::new_with_count(&mut rng, 24)?;
    let phrase = mnemonic.to_phrase();

    // Display the mnemonic with clear warnings
    println!("═══════════════════════════════════════════════════════");
    println!("⚠️  CRITICAL: Save these words IMMEDIATELY!");
    println!("⚠️  These words CANNOT be recovered if lost!");
    println!("⚠️  Anyone with these words can access your funds!");
    println!("⚠️  Write them down on paper and store securely!");
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

    // Derive wallet to show the address
    let wallet = MnemonicBuilder::<English>::default()
        .phrase(phrase.trim())
        .derivation_path("m/44'/60'/0'/0/0")?
        .build()?;

    println!("Derived address (m/44'/60'/0'/0/0): {:?}\n", wallet.address());

    println!("═══════════════════════════════════════════════════════");
    println!("⚠️  Write down ALL 24 words in order!");
    println!("⚠️  Test recovery with derive-key before funding!");
    println!("⚠️  Store backup in multiple secure locations!");
    println!("═══════════════════════════════════════════════════════\n");

    println!("Next steps:");
    println!("  1. Write down the 24 words above on paper");
    println!("  2. Store the paper securely (safe, safety deposit box, etc.)");
    println!("  3. To create a private key file or keystore, use:");
    println!("     cold-deploy derive-key --output private-key.txt\n");

    Ok(())
}
