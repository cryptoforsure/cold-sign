use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub raw_transaction: String,
    pub transaction_hash: String,
    pub from: String,
    pub to: Option<String>,
    pub nonce: u64,
    pub chain_id: u64,
    pub rpc_url: String,
}
