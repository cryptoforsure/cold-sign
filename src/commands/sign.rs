use anyhow::Result;

pub async fn execute(unsigned: String, keystore: String, output: String) -> Result<()> {
    println!("Signing transaction...");
    println!("Unsigned transaction: {}", unsigned);
    println!("Keystore: {}", keystore);
    println!("Output: {}", output);

    // TODO: Implement signing logic
    Ok(())
}
