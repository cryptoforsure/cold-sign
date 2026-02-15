/// Default Ethereum BIP44 derivation path
///
/// Standard path for Ethereum wallets:
/// - m/44' (BIP44 purpose)
/// - /60' (Ethereum coin type)
/// - /0' (account 0)
/// - /0 (external chain)
/// - /0 (address index 0)
pub const DEFAULT_ETH_DERIVATION_PATH: &str = "m/44'/60'/0'/0/0";
