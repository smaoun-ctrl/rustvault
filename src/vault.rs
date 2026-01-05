use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use std::path::PathBuf;
use crate::{encrypt, decrypt, derive_key};
use crate::tenant::get_tenant_salt;

pub fn add_entry_for_tenant(
    db_path: &PathBuf,
    tenant_id: i64,
    name: &str,
    value: &str,
    password: &str,
) -> Result<()> {
    let salt = get_tenant_salt(db_path, tenant_id)?;
    let key = derive_key(password, &salt)?;
    let (ciphertext, nonce) = encrypt(value.as_bytes(), &key)?;

    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT OR REPLACE INTO tenant_entries (tenant_id, name, nonce, ciphertext) VALUES (?1, ?2, ?3, ?4)",
        params![tenant_id, name, &nonce[..], &ciphertext],
    )?;

    Ok(())
}

pub fn get_entry_for_tenant(
    db_path: &PathBuf,
    tenant_id: i64,
    name: &str,
    password: &str,
) -> Result<String> {
    let salt = get_tenant_salt(db_path, tenant_id)?;
    let key = derive_key(password, &salt)?;
    
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT nonce, ciphertext FROM tenant_entries WHERE tenant_id = ?1 AND name = ?2"
    )?;
    
    let result = stmt.query_row(params![tenant_id, name], |row| {
        let nonce: Vec<u8> = row.get(0)?;
        let ciphertext: Vec<u8> = row.get(1)?;
        Ok((nonce, ciphertext))
    });

    match result {
        Ok((nonce, ciphertext)) => {
            let nonce: [u8; 12] = nonce.try_into().map_err(|_| anyhow!("Invalid nonce"))?;
            let decrypted = decrypt(&ciphertext, &key, &nonce)?;
            String::from_utf8(decrypted).map_err(|_| anyhow!("Invalid UTF-8"))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Err(anyhow!("Entry not found")),
        Err(e) => Err(e.into()),
    }
}

pub fn list_entries_for_tenant(
    db_path: &PathBuf,
    tenant_id: i64,
    password: &str,
) -> Result<Vec<(String, String)>> {
    let salt = get_tenant_salt(db_path, tenant_id)?;
    let key = derive_key(password, &salt)?;
    
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare(
        "SELECT name, nonce, ciphertext FROM tenant_entries WHERE tenant_id = ?1"
    )?;
    
    let rows = stmt.query_map(params![tenant_id], |row| {
        let name: String = row.get(0)?;
        let nonce: Vec<u8> = row.get(1)?;
        let ciphertext: Vec<u8> = row.get(2)?;
        Ok((name, nonce, ciphertext))
    })?;

    let mut entries = Vec::new();
    for row in rows {
        let (name, nonce, ciphertext) = row?;
        let nonce: [u8; 12] = nonce.try_into().map_err(|_| anyhow!("Invalid nonce"))?;
        let decrypted = decrypt(&ciphertext, &key, &nonce)?;
        let value = String::from_utf8(decrypted).map_err(|_| anyhow!("Invalid UTF-8"))?;
        entries.push((name, value));
    }
    Ok(entries)
}

pub fn delete_entry_for_tenant(
    db_path: &PathBuf,
    tenant_id: i64,
    name: &str,
) -> Result<()> {
    let conn = Connection::open(db_path)?;
    let changes = conn.execute(
        "DELETE FROM tenant_entries WHERE tenant_id = ?1 AND name = ?2",
        params![tenant_id, name],
    )?;
    
    if changes == 0 {
        return Err(anyhow!("Entry not found"));
    }
    Ok(())
}

