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

        /// RPC endpoint URL (use this OR --network with --infura-key)
        #[arg(short, long, conflicts_with_all = ["network", "infura_key"])]
        rpc_url: Option<String>,

        /// Network name for Infura (mainnet, sepolia, polygon, arbitrum, optimism, base, avalanche)
        #[arg(short, long, requires = "infura_key")]
        network: Option<String>,

        /// Infura API key (required when using --network)
        #[arg(short, long, requires = "network")]
        infura_key: Option<String>,

        /// Deployer address
        #[arg(short, long)]
        from: String,

        /// Constructor arguments (comma-separated)
        #[arg(long)]
        args: Option<String>,

        /// Output file path for unsigned transaction
        #[arg(short, long, default_value = "unsigned.json")]
        output: String,

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

    /// Broadcast signed transaction to the network (uses RPC URL from signed.json)
    Broadcast {
        /// Path to signed transaction JSON
        #[arg(short, long)]
        signed: String,
    },

    /// Generate a new 24-word BIP39 mnemonic phrase (display only, nothing saved to disk)
    GenerateMnemonic,

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
            network,
            infura_key,
            from,
            args,
            output,
            gas_limit,
        } => {
            let resolved_rpc_url = utils::rpc::resolve_rpc_url(rpc_url, network, infura_key)?;
            commands::prepare::execute(contract, resolved_rpc_url, from, args, output, gas_limit)
                .await?;
        }
        Commands::Sign {
            unsigned,
            keystore,
            output,
        } => {
            commands::sign::execute(unsigned, keystore, output).await?;
        }
        Commands::Broadcast { signed } => {
            commands::broadcast::execute(signed).await?;
        }
        Commands::GenerateMnemonic => {
            commands::generate_mnemonic::execute().await?;
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
