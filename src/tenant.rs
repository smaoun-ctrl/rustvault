use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use rand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub tenant_id: Option<i64>, // None pour superuser
    pub username: String,
    pub is_superuser: bool,
}

pub fn init_database(db_path: &PathBuf) -> Result<()> {
    if db_path.exists() {
        return Err(anyhow!(
            "Database already exists at {}. Use --force to delete and reinitialize, or delete the file manually.",
            db_path.display()
        ));
    }

    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let conn = Connection::open(db_path)?;

    // Table des tenants
    conn.execute(
        "CREATE TABLE tenants (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Table des utilisateurs (tenants + superuser)
    conn.execute(
        "CREATE TABLE users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tenant_id INTEGER,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            is_superuser INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Meta par tenant (pour le salt de chaque tenant)
    conn.execute(
        "CREATE TABLE tenant_meta (
            tenant_id INTEGER NOT NULL,
            key TEXT NOT NULL,
            value BLOB NOT NULL,
            PRIMARY KEY (tenant_id, key),
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Entrées par tenant
    conn.execute(
        "CREATE TABLE tenant_entries (
            tenant_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            nonce BLOB NOT NULL,
            ciphertext BLOB NOT NULL,
            PRIMARY KEY (tenant_id, name),
            FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Index pour améliorer les performances
    conn.execute("CREATE INDEX idx_tenant_entries_tenant ON tenant_entries(tenant_id)", [])?;
    conn.execute("CREATE INDEX idx_users_tenant ON users(tenant_id)", [])?;
    conn.execute("CREATE INDEX idx_users_username ON users(username)", [])?;

    // Version de la base de données
    conn.execute(
        "CREATE TABLE db_meta (key TEXT PRIMARY KEY, value TEXT)",
        [],
    )?;
    conn.execute(
        "INSERT INTO db_meta (key, value) VALUES ('version', '2.0')",
        [],
    )?;

    println!("Database initialized successfully.");
    Ok(())
}

pub fn create_superuser(db_path: &PathBuf, username: &str, password: &str) -> Result<()> {
    let conn = Connection::open(db_path)?;

    // Vérifier si le superuser existe déjà
    let mut stmt = conn.prepare("SELECT id FROM users WHERE username = ?1 AND is_superuser = 1")?;
    let exists: Result<i64, _> = stmt.query_row(params![username], |row| row.get(0));
    if exists.is_ok() {
        return Err(anyhow!("Superuser already exists"));
    }

    // Générer un hash du mot de passe avec Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| anyhow!("Password hashing failed"))?
        .to_string();

    // Créer le superuser (tenant_id = NULL pour superuser)
    conn.execute(
        "INSERT INTO users (username, password_hash, is_superuser, tenant_id) VALUES (?1, ?2, 1, NULL)",
        params![username, password_hash],
    )?;

    println!("Superuser '{}' created successfully.", username);
    Ok(())
}

pub fn create_tenant(db_path: &PathBuf, name: &str) -> Result<i64> {
    let conn = Connection::open(db_path)?;

    // Vérifier si le tenant existe déjà
    let mut stmt = conn.prepare("SELECT id FROM tenants WHERE name = ?1")?;
    let exists: Result<i64, _> = stmt.query_row(params![name], |row| row.get(0));
    if exists.is_ok() {
        return Err(anyhow!("Tenant '{}' already exists", name));
    }

    // Créer le tenant
    conn.execute(
        "INSERT INTO tenants (name) VALUES (?1)",
        params![name],
    )?;

    let tenant_id = conn.last_insert_rowid();

    // Générer un salt pour ce tenant
    let salt: [u8; 32] = rand::random();
    conn.execute(
        "INSERT INTO tenant_meta (tenant_id, key, value) VALUES (?1, 'salt', ?2)",
        params![tenant_id, &salt[..]],
    )?;

    println!("Tenant '{}' created with ID: {}", name, tenant_id);
    Ok(tenant_id)
}

pub fn create_tenant_user(
    db_path: &PathBuf,
    tenant_id: i64,
    username: &str,
    password: &str,
) -> Result<()> {
    let conn = Connection::open(db_path)?;

    // Vérifier que le tenant existe
    let mut stmt = conn.prepare("SELECT id FROM tenants WHERE id = ?1")?;
    stmt.query_row(params![tenant_id], |_| Ok(()))
        .map_err(|_| anyhow!("Tenant not found"))?;

    // Vérifier si l'utilisateur existe déjà
    let mut stmt = conn.prepare("SELECT id FROM users WHERE username = ?1")?;
    let exists: Result<i64, _> = stmt.query_row(params![username], |row| row.get(0));
    if exists.is_ok() {
        return Err(anyhow!("User '{}' already exists", username));
    }

    // Générer un hash du mot de passe avec Argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| anyhow!("Password hashing failed"))?
        .to_string();

    // Créer l'utilisateur du tenant
    conn.execute(
        "INSERT INTO users (tenant_id, username, password_hash, is_superuser) VALUES (?1, ?2, ?3, 0)",
        params![tenant_id, username, password_hash],
    )?;

    println!("User '{}' created for tenant {}", username, tenant_id);
    Ok(())
}

pub fn authenticate_user(
    db_path: &PathBuf,
    username: &str,
    password: &str,
) -> Result<User> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT id, tenant_id, username, password_hash, is_superuser FROM users WHERE username = ?1"
    )?;

    let user = stmt.query_row(params![username], |row| {
        Ok((
            row.get::<_, i64>(0)?,      // id
            row.get::<_, Option<i64>>(1)?, // tenant_id
            row.get::<_, String>(2)?,    // username
            row.get::<_, String>(3)?,  // password_hash (string maintenant)
            row.get::<_, i64>(4)?,      // is_superuser
        ))
    })
    .map_err(|_| anyhow!("Invalid username or password"))?;

    let (id, tenant_id, username, password_hash_str, is_superuser_int) = user;
    let is_superuser = is_superuser_int != 0;

    // Vérifier le mot de passe avec Argon2
    let parsed_hash = PasswordHash::new(&password_hash_str)
        .map_err(|_| anyhow!("Invalid password hash format"))?;
    
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| anyhow!("Invalid password"))?;
    
    Ok(User {
        id,
        tenant_id,
        username,
        is_superuser,
    })
}

pub fn get_tenant_salt(db_path: &PathBuf, tenant_id: i64) -> Result<[u8; 32]> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT value FROM tenant_meta WHERE tenant_id = ?1 AND key = 'salt'")?;
    let salt: Vec<u8> = stmt.query_row(params![tenant_id], |row| row.get(0))?;
    salt.try_into()
        .map_err(|_| anyhow!("Invalid salt length"))
}

pub fn list_tenants(db_path: &PathBuf) -> Result<Vec<Tenant>> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT id, name, created_at FROM tenants ORDER BY name")?;
    let rows = stmt.query_map([], |row| {
        Ok(Tenant {
            id: row.get(0)?,
            name: row.get(1)?,
            created_at: row.get(2)?,
        })
    })?;

    let mut tenants = Vec::new();
    for tenant in rows {
        tenants.push(tenant?);
    }
    Ok(tenants)
}

pub fn delete_tenant(db_path: &PathBuf, tenant_id: i64) -> Result<()> {
    let conn = Connection::open(db_path)?;
    conn.execute("DELETE FROM tenants WHERE id = ?1", params![tenant_id])?;
    println!("Tenant {} deleted (cascade will delete related data)", tenant_id);
    Ok(())
}

