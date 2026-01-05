use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{get_key, encrypt, decrypt};

#[derive(Clone)]
pub struct AppState {
    pub db_path: PathBuf,
    pub session_key: Arc<Mutex<Option<[u8; 32]>>>,
}

#[derive(Deserialize)]
pub struct UnlockRequest {
    password: String,
}

#[derive(Deserialize)]
pub struct AddEntryRequest {
    name: String,
    value: String,
}

#[derive(Deserialize)]
pub struct GetEntryRequest {
    name: String,
}

#[derive(Deserialize)]
pub struct DeleteEntryRequest {
    name: String,
}

#[derive(Serialize)]
pub struct Entry {
    name: String,
    value: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

pub async fn unlock(
    state: web::Data<AppState>,
    req: web::Json<UnlockRequest>,
) -> ActixResult<HttpResponse> {
    match get_key(&state.db_path, &req.password) {
        Ok(key) => {
            *state.session_key.lock().await = Some(key);
            Ok(HttpResponse::Ok().json(ApiResponse::success("Unlocked")))
        }
        Err(e) => Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            format!("Invalid password: {}", e),
        ))),
    }
}

pub async fn lock(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    *state.session_key.lock().await = None;
    Ok(HttpResponse::Ok().json(ApiResponse::success("Locked")))
}

pub async fn add_entry_handler(
    state: web::Data<AppState>,
    req: web::Json<AddEntryRequest>,
) -> ActixResult<HttpResponse> {
    let key_guard = state.session_key.lock().await;
    let key = match *key_guard {
        Some(k) => k,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Vault is locked".to_string(),
            )));
        }
    };
    drop(key_guard);

    match encrypt(req.value.as_bytes(), &key) {
        Ok((ciphertext, nonce)) => {
            let conn = rusqlite::Connection::open(&state.db_path)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            conn.execute(
                "INSERT OR REPLACE INTO entries (name, nonce, ciphertext) VALUES (?1, ?2, ?3)",
                rusqlite::params![req.name, &nonce[..], &ciphertext],
            )
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            Ok(HttpResponse::Ok().json(ApiResponse::success("Entry added")))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            format!("Encryption failed: {}", e),
        ))),
    }
}

pub async fn get_entry_handler(
    state: web::Data<AppState>,
    req: web::Json<GetEntryRequest>,
) -> ActixResult<HttpResponse> {
    let key_guard = state.session_key.lock().await;
    let key = match *key_guard {
        Some(k) => k,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Vault is locked".to_string(),
            )));
        }
    };
    drop(key_guard);

    let conn = rusqlite::Connection::open(&state.db_path)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let mut stmt = conn
        .prepare("SELECT nonce, ciphertext FROM entries WHERE name = ?1")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    match stmt.query_row(rusqlite::params![req.name], |row| {
        let nonce: Vec<u8> = row.get(0)?;
        let ciphertext: Vec<u8> = row.get(1)?;
        Ok((nonce, ciphertext))
    }) {
        Ok((nonce, ciphertext)) => {
            let nonce: [u8; 12] = nonce
                .try_into()
                .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid nonce"))?;
            match decrypt(&ciphertext, &key, &nonce) {
                Ok(decrypted) => {
                    let value = String::from_utf8(decrypted)
                        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
                    Ok(HttpResponse::Ok().json(ApiResponse::success(Entry {
                        name: req.name.clone(),
                        value,
                    })))
                }
                Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
                    format!("Decryption failed: {}", e),
                ))),
            }
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Entry not found".to_string(),
            )))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            format!("Database error: {}", e),
        ))),
    }
}

pub async fn list_entries_handler(
    state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let key_guard = state.session_key.lock().await;
    let key = match *key_guard {
        Some(k) => k,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Vault is locked".to_string(),
            )));
        }
    };
    drop(key_guard);

    let conn = rusqlite::Connection::open(&state.db_path)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let mut stmt = conn
        .prepare("SELECT name, nonce, ciphertext FROM entries")
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let rows = stmt
        .query_map([], |row| {
            let name: String = row.get(0)?;
            let nonce: Vec<u8> = row.get(1)?;
            let ciphertext: Vec<u8> = row.get(2)?;
            Ok((name, nonce, ciphertext))
        })
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    let mut entries = Vec::new();
    for row in rows {
        let (name, nonce, ciphertext) = row.map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
        let nonce: [u8; 12] = nonce
            .try_into()
            .map_err(|_| actix_web::error::ErrorInternalServerError("Invalid nonce"))?;
        match decrypt(&ciphertext, &key, &nonce) {
            Ok(decrypted) => {
                if let Ok(value) = String::from_utf8(decrypted) {
                    entries.push(Entry { name, value });
                }
            }
            Err(_) => continue,
        }
    }

    Ok(HttpResponse::Ok().json(ApiResponse::success(entries)))
}

pub async fn delete_entry_handler(
    state: web::Data<AppState>,
    req: web::Json<DeleteEntryRequest>,
) -> ActixResult<HttpResponse> {
    let key_guard = state.session_key.lock().await;
    if key_guard.is_none() {
        return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            "Vault is locked".to_string(),
        )));
    }
    drop(key_guard);

    let conn = rusqlite::Connection::open(&state.db_path)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    let changes = conn
        .execute("DELETE FROM entries WHERE name = ?1", rusqlite::params![req.name])
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    if changes > 0 {
        Ok(HttpResponse::Ok().json(ApiResponse::success("Entry deleted")))
    } else {
        Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
            "Entry not found".to_string(),
        )))
    }
}

pub async fn run_server(port: u16, db_path: PathBuf) -> anyhow::Result<()> {
    let state = web::Data::new(AppState {
        db_path,
        session_key: Arc::new(Mutex::new(None)),
    });

    println!("Starting web server on http://0.0.0.0:{}", port);
    println!("Access the web interface at http://localhost:{}", port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(state.clone())
            .wrap(cors)
            .route("/api/unlock", web::post().to(unlock))
            .route("/api/lock", web::post().to(lock))
            .route("/api/entries", web::get().to(list_entries_handler))
            .route("/api/entries", web::post().to(add_entry_handler))
            .route("/api/entries/get", web::post().to(get_entry_handler))
            .route("/api/entries/delete", web::post().to(delete_entry_handler))
            .service(
                actix_files::Files::new("/", "./web-frontend/dist")
                    .index_file("index.html")
                    .prefer_utf8(true),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await?;

    Ok(())
}


