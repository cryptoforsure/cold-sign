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

The typical workflow involves five steps performed on different machines for maximum security:

1. **Offline machine**: Generate new mnemonic phrase (generate-mnemonic)
2. **Offline machine**: Derive private key from mnemonic (derive-key)
3. **Online machine**: Generate unsigned transaction with network configuration (prepare)
4. **Offline machine**: Sign transaction with keystore (sign) - preserves network config
5. **Online machine**: Broadcast signed transaction (broadcast) - uses stored network config

**Key Feature**: Network configuration (RPC URL, chain ID) flows through the entire workflow:
- `prepare` → saves to `unsigned.json`
- `sign` → copies to `signed.json`
- `broadcast` → reads from `signed.json` (no need to specify again!)

### 1. Generate-Mnemonic Command

Generate a new 24-word BIP39 mnemonic phrase for creating wallets. **This command only displays the mnemonic on screen and never saves anything to disk.**

```bash
cold-deploy generate-mnemonic
```

**Parameters:**
- None - command takes no parameters

**Output:**
- Displays 24-word mnemonic in numbered format
- Shows derived Ethereum address (m/44'/60'/0'/0/0 path)
- Displays prominent security warnings
- **Nothing is saved to disk** - you must write down the words

**⚠️ Critical Security Notes:**
- **WRITE DOWN ALL 24 WORDS IMMEDIATELY** - they cannot be recovered if lost
- Write the words on paper (never store digitally)
- Store in multiple secure locations (safe, safety deposit box, etc.)
- Test recovery with `derive-key` command before funding the wallet
- Anyone with these words can access ALL funds from derived accounts

**Example Output:**
```
Generating new 24-word BIP39 mnemonic...

═══════════════════════════════════════════════════════
⚠️  CRITICAL: Save these words IMMEDIATELY!
⚠️  These words CANNOT be recovered if lost!
⚠️  Anyone with these words can access your funds!
⚠️  Write them down on paper and store securely!
═══════════════════════════════════════════════════════

Your 24-word mnemonic phrase:

 1. abandon      2. ability      3. able
 4. about        5. above        6. absent
...
24. word

Derived address (m/44'/60'/0'/0/0): 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb

═══════════════════════════════════════════════════════
⚠️  Write down ALL 24 words in order!
⚠️  Test recovery with derive-key before funding!
⚠️  Store backup in multiple secure locations!
═══════════════════════════════════════════════════════

Next steps:
  1. Write down the 24 words above on paper
  2. Store the paper securely (safe, safety deposit box, etc.)
  3. To create a private key file or keystore, use:
     cold-deploy derive-key --output private-key.txt
```

### 2. Derive-Key Command

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

### 3. Prepare Command

Generate an unsigned transaction for contract deployment. The chain ID is automatically detected from the RPC endpoint.

**Using Infura (recommended for public networks):**

```bash
cold-deploy prepare \
  --contract examples/SimpleStorage.json \
  --network sepolia \
  --infura-key YOUR_INFURA_API_KEY \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --output unsigned.json
```

**Using custom RPC URL (for local chains or other providers):**

```bash
cold-deploy prepare \
  --contract examples/SimpleStorage.json \
  --rpc-url http://localhost:8545 \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --output unsigned.json
```

**With Constructor Arguments:**

```bash
cold-deploy prepare \
  --contract MyToken.json \
  --network mainnet \
  --infura-key YOUR_INFURA_API_KEY \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --args "1000000,MyToken,MTK" \
  --gas-limit 2000000 \
  --output unsigned.json
```

**Parameters:**
- `--contract` / `-c`: Path to compiled Solidity contract JSON (must have `bytecode` and `abi` fields)
- **Network configuration (choose one):**
  - `--network` / `-n` + `--infura-key` / `-i`: Network name and Infura API key (recommended for public networks)
  - `--rpc-url` / `-r`: Custom RPC endpoint URL (for local chains or other providers)
- `--from` / `-f`: Address that will deploy the contract
- `--args`: Comma-separated constructor arguments (optional)
- `--gas-limit`: Manual gas limit (optional, defaults to 3,000,000)
- `--output` / `-o`: Output file path (default: unsigned.json)

**Supported Networks (for --network):**
- **Ethereum:** `mainnet`, `sepolia`, `goerli`, `holesky`
- **Polygon:** `polygon`, `polygon-amoy`
- **Arbitrum:** `arbitrum`, `arbitrum-sepolia`
- **Optimism:** `optimism`, `optimism-sepolia`
- **Base:** `base`, `base-sepolia`
- **Avalanche:** `avalanche`, `avalanche-fuji`
- **Linea:** `linea`, `linea-sepolia`

**Chain ID Detection:**
The tool automatically fetches the chain ID from the RPC endpoint using the `eth_chainId` method. This ensures:
- No chain ID mismatches between RPC and transaction
- Simplified command-line usage
- Automatic replay protection (EIP-155)

**Output:** Creates `unsigned.json` containing:
- Unsigned transaction details
- Auto-detected chain ID
- RPC URL (preserved through sign → broadcast workflow)

### 4. Sign Command

Sign the transaction offline using an encrypted keystore. **This should be done on an air-gapped machine.**

```bash
cold-deploy sign \
  --unsigned unsigned.json \
  --keystore /path/to/keystore.json \
  --output signed.json
```

You will be prompted to enter your keystore password securely (input is hidden).

**Parameters:**
- `--unsigned` / `-u`: Path to unsigned transaction JSON
- `--keystore` / `-k`: Path to encrypted keystore file
- `--output` / `-o`: Output file path (default: signed.json)

**Output:** Creates `signed.json` containing:
- Signed transaction and transaction hash
- All parameters from `unsigned.json` (including RPC URL and chain ID)
- Signer address and nonce

### 5. Broadcast Command

Broadcast the signed transaction to the network. The RPC URL and chain ID are automatically read from `signed.json`.

```bash
cold-deploy broadcast --signed signed.json
```

**Parameters:**
- `--signed` / `-s`: Path to signed transaction JSON

**Automatic Network Configuration:**
The broadcast command uses the RPC URL that was specified during the `prepare` step and stored in the transaction files. It also automatically verifies that the chain ID in the signed transaction matches the chain ID of the RPC endpoint to prevent broadcasting to the wrong network.

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
- Review gas settings and chain ID (auto-detected during prepare, verified during broadcast)

## Supported Networks

Works with any EVM-compatible network:

- Ethereum (Mainnet, Sepolia, Goerli)
- Polygon
- Binance Smart Chain
- Arbitrum
- Optimism
- Avalanche C-Chain
- And more...

Just specify the correct `--rpc-url` and the chain ID will be automatically detected from the network.

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
