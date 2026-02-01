# cold-deploy

Offline signer for deploying Solidity contracts to Ethereum and EVM-compatible chains.

## Overview

`cold-deploy` is a secure CLI tool for deploying smart contracts using offline signing with encrypted keystores. It follows a three-step process:

1. **prepare** - Generate unsigned transaction JSON
2. **sign** - Sign the transaction offline with an encrypted keystore
3. **broadcast** - Send the signed transaction to the network

## Features

- **Offline Signing**: Sign transactions on an air-gapped machine for maximum security
- **Encrypted Keystore Support**: Uses standard Ethereum encrypted JSON keystores
- **EIP-1559 Support**: Automatic detection and support for both legacy and EIP-1559 transactions
- **Multi-Chain**: Works with Ethereum and all EVM-compatible chains
- **Contract Deployment**: Specialized for deploying Solidity contracts with constructor arguments
- **Transaction Tracking**: Monitor transaction confirmation and retrieve deployed contract addresses

## Installation

### From Source

```bash
git clone https://github.com/yourusername/cold-deploy
cd cold-deploy
cargo build --release
cargo install --path .
```

The binary will be available as `cold-deploy` in your PATH.

## Usage

### Complete Workflow

The typical workflow involves three steps performed on different machines for maximum security:

1. **Online machine**: Generate unsigned transaction (prepare)
2. **Offline machine**: Sign transaction with keystore (sign)
3. **Online machine**: Broadcast signed transaction (broadcast)

### 1. Prepare Command

Generate an unsigned transaction for contract deployment.

```bash
cold-deploy prepare \
  --contract examples/SimpleStorage.json \
  --rpc-url https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --chain-id 11155111 \
  --output unsigned.json
```

**With Constructor Arguments:**

```bash
cold-deploy prepare \
  --contract MyToken.json \
  --rpc-url https://mainnet.infura.io/v3/YOUR_API_KEY \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --chain-id 1 \
  --args "1000000,MyToken,MTK" \
  --gas-limit 2000000 \
  --output unsigned.json
```

**Parameters:**
- `--contract`: Path to compiled Solidity contract JSON (must have `bytecode` and `abi` fields)
- `--rpc-url`: Ethereum RPC endpoint URL
- `--from`: Address that will deploy the contract
- `--chain-id`: Network chain ID (1 = Ethereum mainnet, 11155111 = Sepolia, etc.)
- `--args`: Comma-separated constructor arguments (optional)
- `--gas-limit`: Manual gas limit (optional, defaults to 3,000,000)
- `--output`: Output file path (default: unsigned.json)

**Output:** Creates `unsigned.json` containing the unsigned transaction details.

### 2. Sign Command

Sign the transaction offline using an encrypted keystore. **This should be done on an air-gapped machine.**

```bash
cold-deploy sign \
  --unsigned unsigned.json \
  --keystore /path/to/keystore.json \
  --output signed.json
```

You will be prompted to enter your keystore password securely (input is hidden).

**Parameters:**
- `--unsigned`: Path to unsigned transaction JSON
- `--keystore`: Path to encrypted keystore file
- `--output`: Output file path (default: signed.json)

**Output:** Creates `signed.json` containing the signed transaction and transaction hash.

### 3. Broadcast Command

Broadcast the signed transaction to the network.

```bash
cold-deploy broadcast \
  --signed signed.json \
  --rpc-url https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
```

**Parameters:**
- `--signed`: Path to signed transaction JSON
- `--rpc-url`: Ethereum RPC endpoint URL

**Output:**
- Transaction hash
- Confirmation status
- Contract address (for deployments)
- Gas used
- Block number

### Example Contract JSON

The contract JSON should follow Solidity compiler output format:

```json
{
  "bytecode": "0x608060405234801561000f575f80fd5b50...",
  "abi": [
    {
      "inputs": [],
      "name": "get",
      "outputs": [{"internalType": "uint256", "name": "", "type": "uint256"}],
      "stateMutability": "view",
      "type": "function"
    }
  ]
}
```

See `examples/SimpleStorage.json` for a complete example.

## Security Best Practices

### Air-Gapped Signing

For maximum security:

1. **Online Machine**: Run `prepare` command to generate unsigned transaction
2. **Transfer**: Move `unsigned.json` to air-gapped machine via USB or QR code
3. **Offline Machine**: Run `sign` command on air-gapped machine with your keystore
4. **Transfer**: Move `signed.json` back to online machine
5. **Online Machine**: Run `broadcast` command

### Keystore Security

- Never commit keystore files to version control
- Store keystores in encrypted volumes
- Use strong passwords for keystore encryption
- Keep backups of keystores in secure locations
- Never share keystore files or passwords

### Transaction Verification

Before broadcasting:
- Verify the transaction hash in `signed.json`
- Check the `from` address matches your keystore
- Verify `nonce` is correct for your address
- Review gas settings and chain ID

## Supported Networks

Works with any EVM-compatible network:

- Ethereum (Mainnet, Sepolia, Goerli)
- Polygon
- Binance Smart Chain
- Arbitrum
- Optimism
- Avalanche C-Chain
- And more...

Just specify the correct `--chain-id` and `--rpc-url`.

## Troubleshooting

### "Failed to decrypt keystore"
- Verify password is correct
- Check keystore file is valid JSON
- Ensure keystore follows standard Ethereum format

### "Failed to fetch nonce"
- Verify RPC URL is accessible
- Check the `--from` address is valid
- Ensure network connectivity

### "Transaction failed"
- Check you have sufficient balance for gas
- Verify nonce hasn't been used
- Review contract bytecode is valid
- Check constructor arguments match ABI

### "No bytecode found in contract JSON"
- Ensure contract JSON has `bytecode` field
- Verify JSON is from Solidity compiler output
- Check file isn't corrupted

## Building from Source

```bash
# Clone repository
git clone https://github.com/yourusername/cold-deploy
cd cold-deploy

# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## License

MIT
