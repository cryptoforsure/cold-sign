use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UnsignedTransaction {
    pub to: Option<String>,
    pub data: String,
    pub nonce: u64,
    pub gas_limit: u64,
    pub gas_price: Option<u64>,
    pub max_fee_per_gas: Option<u64>,
    pub max_priority_fee_per_gas: Option<u64>,
    pub chain_id: u64,
    pub value: String,
    pub rpc_url: String,
}
