# cold-deploy

Offline signer for deploying Solidity contracts to Ethereum and EVM-compatible chains.

## Overview

`cold-deploy` is a secure CLI tool for deploying smart contracts using offline signing. It provides:

**Contract Deployment Workflow:**
1. **prepare** - Generate unsigned transaction JSON
2. **sign** - Sign the transaction offline with a keystore
3. **broadcast** - Send the signed transaction to the network

**Mnemonic & Key Management:**
- **generate-mnemonic** - Generate new 24-word BIP39 mnemonic phrases
- **derive-key** - Derive private keys from mnemonic phrases

## Features

- **Offline Signing**: Sign transactions on an air-gapped machine for maximum security
- **Mnemonic Management**: Generate BIP39 mnemonics and derive private keys
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

### 4. Generate-Mnemonic Command

Generate a new 24-word BIP39 mnemonic phrase for creating wallets.

```bash
# Generate mnemonic only (display on screen)
cold-deploy generate-mnemonic

# Generate mnemonic and save private key to file
cold-deploy generate-mnemonic --create-keystore --output private-key.txt
```

**Parameters:**
- `--create-keystore` / `-c`: Optional flag to save the derived private key to file
- `--output` / `-o`: Output file path for private key (default: `private-key-<ADDRESS>.txt`)

**Output:**
- Displays 24-word mnemonic in numbered format
- With `--create-keystore`: saves private key in plain text hex format
- Shows derived address
- Displays security warnings

**⚠️ Security Notes:**
- **SAVE YOUR MNEMONIC WORDS IMMEDIATELY** - they cannot be recovered
- Private keys are saved in **PLAIN TEXT** - keep files extremely secure
- Never share your mnemonic or private key files
- Back up mnemonic in multiple secure locations

**Example Output:**
```
Generating new 24-word BIP39 mnemonic...

═══════════════════════════════════════════════════════
⚠️  IMPORTANT: Save these words in a secure location!
⚠️  Anyone with these words can access your funds!
⚠️  Keep them offline and never share them!
═══════════════════════════════════════════════════════

Your 24-word mnemonic phrase:

 1. abandon      2. ability      3. able
 4. about        5. above        6. absent
...

Derived address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
Private key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

### 5. Derive-Key Command

Derive a private key from an existing 24-word mnemonic phrase.

```bash
# Derive from interactive input
cold-deploy derive-key --output private-key.txt

# Derive from mnemonic file
cold-deploy derive-key --mnemonic-file mnemonic.txt --output private-key.txt
```

**Parameters:**
- `--mnemonic-file` / `-m`: Optional path to file containing mnemonic phrase
- `--output` / `-o`: Output file path for private key (required)

**Derivation Path:** Uses standard Ethereum path `m/44'/60'/0'/0/0`

**Output:**
- Validates mnemonic has 24 words
- Derives private key using BIP44 path
- Saves private key in plain text hex format
- Displays address and private key

**⚠️ Security Notes:**
- Private keys are saved in **PLAIN TEXT**
- Store mnemonic files securely or delete after use
- Never commit mnemonic or private key files to version control

**Example Usage:**
```bash
# Interactive input
$ cold-deploy derive-key --output my-key.txt
Enter your 24-word mnemonic phrase:
(paste all words separated by spaces, then press Enter)

> abandon ability able about above absent absorb abstract ...

Mnemonic validated: 24 words
Deriving key using path: m/44'/60'/0'/0/0

✓ Key derived successfully!
  Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb

✓ Private key saved successfully!
  File: my-key.txt
  Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb
  Private key: 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

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

### Keystore & Private Key Security

- **Never commit** keystore files, private keys, or mnemonics to version control
- **Private key files** are stored in **PLAIN TEXT** - treat them as extremely sensitive
- Store keystores and private keys in encrypted volumes
- Use strong passwords for keystore encryption
- Keep backups of keystores and mnemonics in secure, offline locations
- Never share keystore files, private keys, mnemonics, or passwords
- Delete plain text private key files after importing to secure storage

### Mnemonic Security

- **Write down your 24-word mnemonic** immediately when generated
- Store mnemonic backups in multiple secure, offline locations (e.g., safe, safety deposit box)
- **Never store mnemonics digitally** (no photos, no cloud storage, no email)
- Consider using metal backup solutions for fire/water resistance
- Anyone with your mnemonic can access ALL derived accounts
- Test your mnemonic backup by attempting recovery before funding

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

### "Invalid mnemonic: expected 24 words"
- Verify you're entering exactly 24 words
- Check for extra spaces or line breaks
- Ensure all words are from the BIP39 word list
- Try saving mnemonic to file and using `--mnemonic-file`

### "Failed to derive wallet from mnemonic"
- Verify mnemonic phrase is correct and complete
- Check that words are in correct order
- Ensure using valid BIP39 English words
- Test mnemonic in another wallet to verify it's valid

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
