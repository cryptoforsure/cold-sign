use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod constants;
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
    /// Generate a new 24-word BIP39 mnemonic phrase (display only, nothing saved to disk)
    GenerateMnemonic,

    /// Derive private key from mnemonic phrase (creates encrypted keystore by default)
    DeriveKey {
        /// Path to file containing mnemonic phrase (optional, will prompt if not provided)
        #[arg(short, long)]
        mnemonic_file: Option<String>,

        /// Output file path (default: keystore-<ADDRESS>.json for keystore, private-key-<ADDRESS>.txt for plain text)
        #[arg(short, long)]
        output: Option<String>,

        /// Save as plain text private key instead of encrypted keystore (NOT RECOMMENDED)
        #[arg(long)]
        plain_text: bool,
    },

    /// Generate unsigned transaction JSON for contract deployment or function call
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

        /// Sender address
        #[arg(short, long)]
        from: String,

        /// Deployed contract address to call (enables call mode, must be used with --function)
        #[arg(long, requires = "function_name")]
        to: Option<String>,

        /// Function name to call (enables call mode, must be used with --to)
        #[arg(long = "function", requires = "to")]
        function_name: Option<String>,

        /// Constructor or function arguments (comma-separated)
        #[arg(long)]
        args: Option<String>,

        /// ETH value to send in wei (default: 0, for payable constructors or functions)
        #[arg(long, default_value = "0")]
        value: String,

        /// Output file path for unsigned transaction
        #[arg(short, long, default_value = "unsigned.json")]
        output: String,

        /// Gas limit (optional, defaults to 3,000,000 if not provided)
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
            to,
            function_name,
            args,
            value,
            output,
            gas_limit,
        } => {
            let resolved_rpc_url = utils::rpc::resolve_rpc_url(rpc_url, network, infura_key)?;
            commands::prepare::execute(contract, resolved_rpc_url, from, to, function_name, args, value, output, gas_limit)
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
            plain_text,
        } => {
            commands::derive_key::execute(mnemonic_file, output, plain_text).await?;
        }
    }

    Ok(())
}
