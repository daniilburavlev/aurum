use crate::schema::{ErrorResponse, WalletInfo};
use block::block::Block;
use httpclient::{Client, InMemoryBody, ResponseExt};
use tx::tx_data::TxData;

pub struct RpcClient {
    client: Client,
}

impl RpcClient {
    pub fn new(address: String) -> Self {
        let client = Client::new().base_url(&address);
        Self { client }
    }

    pub async fn find_block(&self, idx: u64) -> Option<Block> {
        if let Ok(response) = self.client.get(format!("/api/blocks/{}", idx)).send().await {
            if response.status().as_u16() == 200 {
                let body = response.text().await;
                let block = serde_json::from_str(&body.unwrap()).unwrap();
                Some(block)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn get_nonce(&self, wallet: String) -> u64 {
        if let Ok(response) = self
            .client
            .get(format!("/api/wallets/{}", wallet))
            .send()
            .await
        {
            if response.status().as_u16() == 200 {
                let body = response.text().await;
                let body: WalletInfo = serde_json::from_str(&body.unwrap()).unwrap();
                body.nonce
            } else {
                0
            }
        } else {
            0
        }
    }

    pub async fn add_tx(&self, tx: TxData) -> Option<String> {
        if let Ok(response) = self
            .client
            .post("/api/txs")
            .header("Content-Type", "application/json")
            .body(InMemoryBody::Json(serde_json::to_value(tx).unwrap()))
            .send()
            .await
        {
            if response.status().as_u16() == 200 {
                return None;
            } else if response.status().as_u16() == 400 {
                let error = response.text().await.unwrap();
                let error: ErrorResponse = serde_json::from_str(&error).unwrap();
                return Some(error.error);
            }
        }
        Some(String::from("Internal error"))
    }
}
