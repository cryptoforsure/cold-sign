# cold-deploy

Offline signer for deploying Solidity contracts to Ethereum and EVM-compatible chains.

## Overview

`cold-deploy` is a secure CLI tool for deploying smart contracts using offline signing with encrypted keystores. It follows a three-step process:

1. **prepare** - Generate unsigned transaction JSON
2. **sign** - Sign the transaction offline with an encrypted keystore
3. **broadcast** - Send the signed transaction to the network

## Installation

```bash
cargo install --path .
```

## Usage

Coming soon...

## Security

- Never commit keystore files or private keys
- Always verify transaction details before signing
- Use this tool in an air-gapped environment for maximum security

## License

MIT
