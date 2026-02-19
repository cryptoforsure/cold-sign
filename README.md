# cold-sign

Offline signer for EVM-compatible transactions to Ethereum and EVM-compatible chains.

## Overview

`cold-sign` is a secure CLI tool for deploying smart contracts and calling contract functions using offline signing. It provides:

**Transaction Workflow (deployment or function call):**
1. **prepare** - Generate unsigned transaction JSON (deploy a contract or call a function)
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
- **Contract Deployment**: Deploy Solidity contracts with ABI-encoded constructor arguments
- **Function Calls**: Call any function on an already-deployed contract with ABI-encoded arguments
- **Transaction Tracking**: Monitor transaction confirmation and retrieve deployed contract addresses

## Installation

### From Source

```bash
git clone https://github.com/yourusername/cold-sign
cd cold-sign
cargo build --release
cargo install --path .
```

The binary will be available as `cold-sign` in your PATH.

## Usage

### Complete Workflow

The typical workflow involves five steps performed on different machines for maximum security:

1. **Offline machine**: Generate new mnemonic phrase (generate-mnemonic)
2. **Offline machine**: Derive private key from mnemonic (derive-key)
3. **Online machine**: Generate unsigned transaction with network configuration (prepare) — for contract deployment or a function call
4. **Offline machine**: Sign transaction with keystore (sign) - preserves network config
5. **Online machine**: Broadcast signed transaction (broadcast) - uses stored network config

**Key Feature**: Network configuration (RPC URL, chain ID) flows through the entire workflow:
- `prepare` → saves to `unsigned.json`
- `sign` → copies to `signed.json`
- `broadcast` → reads from `signed.json` (no need to specify again!)

### 1. Generate-Mnemonic Command

Generate a new 24-word BIP39 mnemonic phrase for creating wallets. **This command only displays the mnemonic on screen and never saves anything to disk.**

```bash
cold-sign generate-mnemonic
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
  3. To create an encrypted keystore, use:
     cold-sign derive-key
```

### 2. Derive-Key Command

Derive an encrypted keystore from your 24-word mnemonic phrase. **Creates an encrypted JSON keystore by default** (recommended).

```bash
# Create encrypted keystore (recommended, default behavior)
cold-sign derive-key

# Create encrypted keystore with custom filename
cold-sign derive-key --output my-keystore.json

# Read mnemonic from file
cold-sign derive-key --mnemonic-file mnemonic.txt

# Plain text private key (NOT RECOMMENDED - requires explicit flag)
cold-sign derive-key --plain-text --output private-key.txt
```

**Parameters:**
- `--mnemonic-file` / `-m`: Optional path to file containing mnemonic phrase (will prompt if not provided)
- `--output` / `-o`: Optional output file path (default: `keystore-<ADDRESS>.json` for encrypted, `private-key-<ADDRESS>.txt` for plain text)
- `--plain-text`: Save as plain text private key instead of encrypted keystore (NOT RECOMMENDED)

**Derivation Path:** Uses standard Ethereum path `m/44'/60'/0'/0/0`

**Encrypted Keystore Mode (Default):**
- Prompts for password (hidden input)
- Password confirmation required
- Minimum 8 characters
- Creates standard Ethereum encrypted JSON keystore
- Compatible with all Ethereum tools (geth, MetaMask, etc.)

**⚠️ Security Notes:**
- **RECOMMENDED**: Use encrypted keystore (default) instead of plain text
- Choose a strong password for keystore encryption
- **Password cannot be recovered** - keep it safe!
- Store mnemonic files securely or delete after use
- Never commit keystores, mnemonics, or private keys to version control

**Example Usage (Encrypted Keystore):**
```bash
$ cold-sign derive-key
Deriving encrypted keystore from mnemonic...

Enter your 24-word mnemonic phrase:
(paste all words separated by spaces, then press Enter)

> abandon ability able about above absent absorb abstract ...

Mnemonic validated: 24 words
Deriving key using path: m/44'/60'/0'/0/0

✓ Key derived successfully!
  Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb

Create a strong password to encrypt your keystore:
(Password must be at least 8 characters)
Enter password: ********
Confirm password: ********

Encrypting keystore...

✓ Encrypted keystore saved successfully!
  File: keystore-0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb.json
  Address: 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb

✓ Your private key is now protected with encryption!
  Remember your password - it CANNOT be recovered if lost!
```

**Example Usage (Plain Text - Not Recommended):**
```bash
$ cold-sign derive-key --plain-text --output my-key.txt
⚠️  WARNING: Creating PLAIN TEXT private key file!
⚠️  Consider using encrypted keystore instead (default)

[... mnemonic entry ...]

⚠️  WARNING: Private key saved in PLAIN TEXT!
⚠️  Keep this file EXTREMELY secure!
⚠️  Anyone with this file can access your funds!
```

### 3. Prepare Command

Generate an unsigned transaction for **contract deployment** or **function calls**. The chain ID is automatically detected from the RPC endpoint.

The mode is selected by the flags provided:
- **Deploy mode** (default): omit `--to` and `--function`; `data` is the contract bytecode + ABI-encoded constructor arguments
- **Call mode**: provide both `--to` and `--function`; `data` is the ABI-encoded function call (selector + arguments)

---

#### Deploy Mode

**Using Infura (recommended for public networks):**

```bash
cold-sign prepare \
  --contract examples/SimpleStorage.json \
  --network sepolia \
  --infura-key YOUR_INFURA_API_KEY \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --output unsigned.json
```

**Using custom RPC URL (for local chains or other providers):**

```bash
cold-sign prepare \
  --contract examples/SimpleStorage.json \
  --rpc-url http://localhost:8545 \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --output unsigned.json
```

**With constructor arguments:**

```bash
cold-sign prepare \
  --contract MyToken.json \
  --network mainnet \
  --infura-key YOUR_INFURA_API_KEY \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --args "1000000,MyToken,MTK" \
  --gas-limit 2000000 \
  --output unsigned.json
```

---

#### Call Mode

Call an already-deployed contract function by providing `--to` (contract address) and `--function` (function name). Both flags must always be used together.

**Calling a function with arguments:**

```bash
cold-sign prepare \
  --contract examples/SimpleStorage.json \
  --rpc-url http://localhost:8545 \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --to 0xDeployedContractAddress \
  --function store \
  --args "42" \
  --output unsigned.json
```

**Calling a no-argument function:**

```bash
cold-sign prepare \
  --contract examples/SimpleStorage.json \
  --network sepolia \
  --infura-key YOUR_INFURA_API_KEY \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --to 0xDeployedContractAddress \
  --function reset \
  --output unsigned.json
```

**Calling a payable function (sending ETH):**

```bash
cold-sign prepare \
  --contract examples/SimpleStorage.json \
  --rpc-url http://localhost:8545 \
  --from 0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb \
  --to 0xDeployedContractAddress \
  --function deposit \
  --value 1000000000000000000 \
  --output unsigned.json
```

> **Note:** The compiled contract JSON must contain the ABI of the target contract. Only the ABI is used in call mode; the `bytecode` field is ignored.

---

**Parameters:**
- `--contract` / `-c`: Path to compiled Solidity contract JSON (must have `bytecode` and `abi` fields for deploy; only `abi` is required for call mode)
- **Network configuration (choose one):**
  - `--network` / `-n` + `--infura-key` / `-i`: Network name and Infura API key (recommended for public networks)
  - `--rpc-url` / `-r`: Custom RPC endpoint URL (for local chains or other providers)
- `--from` / `-f`: Sender address
- `--to`: Deployed contract address to call *(call mode only, requires `--function`)*
- `--function`: Function name to call *(call mode only, requires `--to`)*
- `--args`: Comma-separated constructor or function arguments (optional)
- `--value`: ETH value to send in wei (optional, default: `0`; for payable constructors or functions)
- `--gas-limit`: Manual gas limit (optional, defaults to 3,000,000)
- `--output` / `-o`: Output file path (default: `unsigned.json`)

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
cold-sign sign \
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
cold-sign broadcast --signed signed.json
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

### "Function 'X' not found in ABI"
- Verify the function name matches exactly (case-sensitive) the name in the Solidity source
- Confirm the correct contract JSON file is being passed via `--contract`
- Check the ABI array in the JSON includes the target function

### "Function 'X' expects N argument(s) but M were provided"
- Count the comma-separated values in `--args` to ensure they match the function signature
- Check for extra or missing commas in the argument list

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
git clone https://github.com/yourusername/cold-sign
cd cold-sign

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
