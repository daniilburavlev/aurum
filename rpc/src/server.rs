use crate::schema::{ErrorResponse, WalletInfo};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use block::block::Block;
use libp2p::PeerId;
use p2p::network::Client;
use serde::Serialize;
use std::sync::Arc;
use storage::storage::Storage;
use tx::tx::Tx;
use tx::tx_data::TxData;

struct P2pClientHolder {
    client: Arc<futures::lock::Mutex<Client>>,
}

impl P2pClientHolder {
    fn new(client: Client) -> Self {
        Self {
            client: Arc::new(futures::lock::Mutex::new(client)),
        }
    }

    async fn get_nonce(&self, wallet: String, peer_id: PeerId) -> u64 {
        let mut client = self.client.lock().await;
        client.get_nonce(wallet, peer_id).await
    }

    async fn add_tx(&self, data: TxData, peer_id: PeerId) -> Result<Tx, String> {
        let mut client = self.client.lock().await;
        client.add_tx(data, peer_id).await
    }
}

#[derive(Debug, Serialize)]
enum AppError {
    NotFound(String),
    BadRequest(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound(error) => {
                (StatusCode::NOT_FOUND, Json::from(ErrorResponse { error }))
            }
            AppError::BadRequest(error) => {
                (StatusCode::BAD_REQUEST, Json::from(ErrorResponse { error }))
            }
        };
        (status, body).into_response()
    }
}

#[derive(Clone)]
struct AppState {
    wallet: String,
    storage: Arc<Storage>,
    state: Arc<state::state::State>,
    client: Arc<P2pClientHolder>,
}

impl AppState {
    fn new(
        wallet: String,
        storage: &Arc<Storage>,
        state: &Arc<state::state::State>,
        client: Client,
    ) -> Self {
        Self {
            wallet,
            storage: Arc::clone(storage),
            state: Arc::clone(state),
            client: Arc::new(P2pClientHolder::new(client)),
        }
    }

    async fn find_block(&self, idx: u64) -> Option<Block> {
        self.storage.find_block_by_idx(idx).unwrap_or(None)
    }

    async fn get_nonce(&self, wallet: String) -> u64 {
        let current_validator = self.get_current_validator();
        if current_validator == self.wallet {
            self.state.get_nonce(wallet).await
        } else {
            if let Some(peer_id) = self.address_to_peer_id(current_validator) {
                self.client.get_nonce(wallet, peer_id).await
            } else {
                0
            }
        }
    }

    async fn add_tx(&self, tx: TxData) -> Result<Tx, String> {
        let current_validator = self.get_current_validator();
        if current_validator == self.wallet {
            self.state.add_tx(tx).await
        } else {
            if let Some(peer_id) = self.address_to_peer_id(current_validator) {
                self.client.add_tx(tx, peer_id).await
            } else {
                Err(String::from("Internal server error"))
            }
        }
    }

    fn address_to_peer_id(&self, address: String) -> Option<PeerId> {
        let Ok(public) = bs58::decode(address).into_vec() else {
            return None;
        };
        let Ok(public) = libp2p::identity::ecdsa::PublicKey::try_from_bytes(public.as_slice())
        else {
            return None;
        };
        let public = libp2p::identity::PublicKey::from(public);
        Some(PeerId::from_public_key(&public))
    }

    fn get_current_validator(&self) -> String {
        if let Ok(address) = self.storage.current_validator() {
            address
        } else {
            self.wallet.clone()
        }
    }
}

pub async fn run(
    port: i32,
    wallet: String,
    storage: &Arc<Storage>,
    state: &Arc<state::state::State>,
    client: Client,
) {
    let state = AppState::new(wallet, storage, state, client);
    let state = Arc::new(state);

    let app = Router::new()
        .route("/api/blocks/{idx}", get(find_block_by_idx))
        .route("/api/wallets/{wallet}", get(get_nonce))
        .route("/api/txs", post(add_tx))
        .with_state(state);

    println!("Listening http RPC on port: {}", port);

    let address = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn find_block_by_idx(
    Path(idx): Path<u64>,
    state: State<Arc<AppState>>,
) -> Result<Json<Block>, AppError> {
    if let Some(block) = state.find_block(idx).await {
        Ok(Json::from(block))
    } else {
        Err(AppError::NotFound(format!("Block #{}", idx)))
    }
}

#[axum::debug_handler]
async fn get_nonce(
    Path(wallet): Path<String>,
    state: State<Arc<AppState>>,
) -> Result<Json<WalletInfo>, AppError> {
    let nonce = state.get_nonce(wallet).await;
    Ok(Json(WalletInfo { nonce }))
}

#[axum::debug_handler]
async fn add_tx(
    state: State<Arc<AppState>>,
    Json(data): Json<TxData>,
) -> Result<Json<Tx>, AppError> {
    match state.add_tx(data).await {
        Ok(tx) => Ok(Json(tx)),
        Err(err) => Err(AppError::BadRequest(err)),
    }
}
