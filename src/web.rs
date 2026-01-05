use actix_cors::Cors;
use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{encrypt, decrypt, derive_key};
use crate::tenant::{authenticate_user, list_tenants, create_tenant, delete_tenant, get_tenant_salt, Tenant};
use crate::vault::{add_entry_for_tenant, get_entry_for_tenant, list_entries_for_tenant, delete_entry_for_tenant};
use crate::web_session::{Session, SessionStore, LoginRequest, LoginResponse, new_session_store};

#[derive(Clone)]
pub struct AppState {
    pub db_path: PathBuf,
    pub session: SessionStore,
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
pub struct VersionInfo {
    version: String,
    name: String,
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

// Nouveau handler de login
pub async fn login(
    state: web::Data<AppState>,
    req: web::Json<LoginRequest>,
) -> ActixResult<HttpResponse> {
    match authenticate_user(&state.db_path, &req.username, &req.password) {
        Ok(user) => {
            // Vérifier que le tenant_id correspond si fourni
            if let Some(tenant_id) = req.tenant_id {
                if user.tenant_id != Some(tenant_id) {
                    return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                        "Tenant ID mismatch".to_string(),
                    )));
                }
            }

            // Si c'est un superuser, pas besoin de tenant
            if user.is_superuser {
                let session = Session {
                    user: user.clone(),
                    tenant: None,
                    encryption_key: None, // Superuser n'a pas de clé de chiffrement
                };
                *state.session.lock().await = Some(session);
                return Ok(HttpResponse::Ok().json(ApiResponse::success(LoginResponse {
                    user,
                    tenant: None,
                    is_superuser: true,
                })));
            }

            // Pour un utilisateur tenant, récupérer le tenant et générer la clé
            if let Some(tenant_id) = user.tenant_id {
                // Récupérer les informations du tenant
                let tenants = list_tenants(&state.db_path)
                    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
                let tenant = tenants.iter().find(|t| t.id == tenant_id)
                    .ok_or_else(|| actix_web::error::ErrorInternalServerError("Tenant not found"))?;

                // Générer la clé de chiffrement avec le password et le salt du tenant
                let salt = get_tenant_salt(&state.db_path, tenant_id)
                    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
                let encryption_key = derive_key(&req.password, &salt)
                    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

                let session = Session {
                    user: user.clone(),
                    tenant: Some(tenant.clone()),
                    encryption_key: Some(encryption_key),
                };
                *state.session.lock().await = Some(session);
                Ok(HttpResponse::Ok().json(ApiResponse::success(LoginResponse {
                    user,
                    tenant: Some(tenant.clone()),
                    is_superuser: false,
                })))
            } else {
                Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                    "Invalid user configuration".to_string(),
                )))
            }
        }
        Err(e) => Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
            format!("Invalid credentials: {}", e),
        ))),
    }
}

pub async fn logout(state: web::Data<AppState>) -> ActixResult<HttpResponse> {
    *state.session.lock().await = None;
    Ok(HttpResponse::Ok().json(ApiResponse::success("Logged out")))
}

pub async fn add_entry_handler(
    state: web::Data<AppState>,
    req: web::Json<AddEntryRequest>,
) -> ActixResult<HttpResponse> {
    let session_guard = state.session.lock().await;
    let session = match session_guard.as_ref() {
        Some(s) => s,
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Not authenticated".to_string(),
            )));
        }
    };

    // Vérifier que c'est un utilisateur tenant (pas superuser)
    if session.user.is_superuser {
        return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
            "Superuser cannot manage entries".to_string(),
        )));
    }

    let tenant_id = session.user.tenant_id
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("No tenant ID"))?;

    let encryption_key = session.encryption_key
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("No encryption key"))?;

    drop(session_guard);

    // Utiliser le password de la session pour ajouter l'entrée
    // Note: On devrait stocker le password dans la session ou le redemander
    // Pour l'instant, on utilise la clé déjà dérivée
    match add_entry_for_tenant(&state.db_path, tenant_id, &req.name, &req.value, "") {
        Ok(_) => {
            // TODO: Il faut passer le password, pas une chaîne vide
            // Pour l'instant, on utilise directement la clé
            let (ciphertext, nonce) = encrypt(req.value.as_bytes(), &encryption_key)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            
            let conn = rusqlite::Connection::open(&state.db_path)
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            conn.execute(
                "INSERT OR REPLACE INTO tenant_entries (tenant_id, name, nonce, ciphertext) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![tenant_id, req.name, &nonce[..], &ciphertext],
            )
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
            
            Ok(HttpResponse::Ok().json(ApiResponse::success("Entry added")))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            format!("Failed to add entry: {}", e),
        ))),
    }
}

pub async fn get_entry_handler(
    state: web::Data<AppState>,
    req: web::Json<GetEntryRequest>,
) -> ActixResult<HttpResponse> {
    let session_guard = state.session.lock().await;
    let session = match session_guard.as_ref() {
        Some(s) if !s.user.is_superuser => s,
        Some(_) => {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "Superuser cannot access entries".to_string(),
            )));
        }
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Not authenticated".to_string(),
            )));
        }
    };

    let tenant_id = session.user.tenant_id
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("No tenant ID"))?;
    drop(session_guard);

    match get_entry_for_tenant(&state.db_path, tenant_id, &req.name, "") {
        Ok(value) => Ok(HttpResponse::Ok().json(ApiResponse::success(Entry {
            name: req.name.clone(),
            value,
        }))),
        Err(e) if e.to_string().contains("not found") => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Entry not found".to_string(),
            )))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            format!("Error: {}", e),
        ))),
    }
}

pub async fn list_entries_handler(
    state: web::Data<AppState>,
) -> ActixResult<HttpResponse> {
    let session_guard = state.session.lock().await;
    let session = match session_guard.as_ref() {
        Some(s) if !s.user.is_superuser => s,
        Some(_) => {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "Superuser cannot access entries".to_string(),
            )));
        }
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Not authenticated".to_string(),
            )));
        }
    };

    let tenant_id = session.user.tenant_id
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("No tenant ID"))?;
    drop(session_guard);

    match list_entries_for_tenant(&state.db_path, tenant_id, "") {
        Ok(entries_vec) => {
            let entries: Vec<Entry> = entries_vec
                .into_iter()
                .map(|(name, value)| Entry { name, value })
                .collect();
            Ok(HttpResponse::Ok().json(ApiResponse::success(entries)))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            format!("Error: {}", e),
        ))),
    }
}

pub async fn version_handler() -> ActixResult<HttpResponse> {
    let version_info = VersionInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: env!("CARGO_PKG_NAME").to_string(),
    };
    Ok(HttpResponse::Ok().json(ApiResponse::success(version_info)))
}

pub async fn delete_entry_handler(
    state: web::Data<AppState>,
    req: web::Json<DeleteEntryRequest>,
) -> ActixResult<HttpResponse> {
    let session_guard = state.session.lock().await;
    let session = match session_guard.as_ref() {
        Some(s) if !s.user.is_superuser => s,
        Some(_) => {
            return Ok(HttpResponse::Forbidden().json(ApiResponse::<()>::error(
                "Superuser cannot delete entries".to_string(),
            )));
        }
        None => {
            return Ok(HttpResponse::Unauthorized().json(ApiResponse::<()>::error(
                "Not authenticated".to_string(),
            )));
        }
    };

    let tenant_id = session.user.tenant_id
        .ok_or_else(|| actix_web::error::ErrorInternalServerError("No tenant ID"))?;
    drop(session_guard);

    match delete_entry_for_tenant(&state.db_path, tenant_id, &req.name) {
        Ok(_) => Ok(HttpResponse::Ok().json(ApiResponse::success("Entry deleted"))),
        Err(e) if e.to_string().contains("not found") => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::error(
                "Entry not found".to_string(),
            )))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::error(
            format!("Error: {}", e),
        ))),
    }
}

pub async fn run_server(port: u16, db_path: PathBuf) -> anyhow::Result<()> {
    let state = web::Data::new(AppState {
        db_path,
        session: new_session_store(),
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
            // Routes API en premier pour éviter les conflits avec les fichiers statiques
            .route("/api/version", web::get().to(version_handler))
            .route("/api/login", web::post().to(login))
            .route("/api/logout", web::post().to(logout))
            .route("/api/entries", web::get().to(list_entries_handler))
            .route("/api/entries", web::post().to(add_entry_handler))
            .route("/api/entries/get", web::post().to(get_entry_handler))
            .route("/api/entries/delete", web::post().to(delete_entry_handler))
            // Fichiers statiques en dernier
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


