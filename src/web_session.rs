use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::tenant::{User, Tenant};

#[derive(Clone, Debug)]
pub struct Session {
    pub user: User,
    pub tenant: Option<Tenant>, // None pour superuser
    pub encryption_key: Option<[u8; 32]>, // Clé de déchiffrement pour le tenant
}

pub type SessionStore = Arc<Mutex<Option<Session>>>;

pub fn new_session_store() -> SessionStore {
    Arc::new(Mutex::new(None))
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub tenant_id: Option<i64>, // Optionnel pour superuser
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user: User,
    pub tenant: Option<Tenant>,
    pub is_superuser: bool,
}

