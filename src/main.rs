use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod types;
mod utils;

#[derive(Parser)]
#[command(name = "cold-deploy")]
#[command(about = "Offline signer for deploying Solidity contracts", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate unsigned transaction JSON for contract deployment
    Prepare {
        /// Path to compiled contract JSON (Solidity compiler output)
        #[arg(short, long)]
        contract: String,

        /// RPC endpoint URL to fetch nonce and gas price
        #[arg(short, long)]
        rpc_url: String,

        /// Deployer address
        #[arg(short, long)]
        from: String,

        /// Constructor arguments (comma-separated)
        #[arg(long)]
        args: Option<String>,

        /// Output file path for unsigned transaction
        #[arg(short, long, default_value = "unsigned.json")]
        output: String,

        /// Chain ID (e.g., 1 for Ethereum mainnet, 11155111 for Sepolia)
        #[arg(long)]
        chain_id: u64,

        /// Gas limit (optional, will estimate if not provided)
        #[arg(long)]
        gas_limit: Option<u64>,
    },

    /// Sign the unsigned transaction with encrypted keystore
    Sign {
        /// Path to unsigned transaction JSON
        #[arg(short, long)]
        unsigned: String,

        /// Path to encrypted keystore file
        #[arg(short, long)]
        keystore: String,

        /// Output file path for signed transaction
        #[arg(short, long, default_value = "signed.json")]
        output: String,
    },

    /// Broadcast signed transaction to the network
    Broadcast {
        /// Path to signed transaction JSON
        #[arg(short, long)]
        signed: String,

        /// RPC endpoint URL
        #[arg(short, long)]
        rpc_url: String,
    },

    /// Generate a new 24-word BIP39 mnemonic phrase
    GenerateMnemonic {
        /// Optional: Save private key to file immediately (plain text)
        #[arg(short, long)]
        create_keystore: bool,

        /// Output file path for private key (if --create-keystore is set)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Derive private key from mnemonic phrase and save to file
    DeriveKey {
        /// Path to file containing mnemonic phrase (optional, will prompt if not provided)
        #[arg(short, long)]
        mnemonic_file: Option<String>,

        /// Output file path for private key
        #[arg(short, long)]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Prepare {
            contract,
            rpc_url,
            from,
            args,
            output,
            chain_id,
            gas_limit,
        } => {
            commands::prepare::execute(contract, rpc_url, from, args, output, chain_id, gas_limit)
                .await?;
        }
        Commands::Sign {
            unsigned,
            keystore,
            output,
        } => {
            commands::sign::execute(unsigned, keystore, output).await?;
        }
        Commands::Broadcast { signed, rpc_url } => {
            commands::broadcast::execute(signed, rpc_url).await?;
        }
        Commands::GenerateMnemonic {
            create_keystore,
            output,
        } => {
            commands::generate_mnemonic::execute(create_keystore, output).await?;
        }
        Commands::DeriveKey {
            mnemonic_file,
            output,
        } => {
            commands::derive_key::execute(mnemonic_file, output).await?;
        }
    }

    Ok(())
}
