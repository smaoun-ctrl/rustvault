use aes_gcm::{Aes256Gcm, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use anyhow::{anyhow, Result};
use argon2::Argon2;
use clap::{Parser, Subcommand};
use eframe::egui::{self, Context};
use eframe::Frame;
use eframe;
use rpassword::read_password;
use rusqlite::{params, Connection};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

mod web;

#[derive(Parser)]
#[command(name = "rustvault")]
#[command(version)]
#[command(about = "A hyper-secure digital vault")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new vault
    Init,
    /// Add an entry to the vault
    Add {
        /// Name of the entry
        name: String,
        /// Value of the entry
        value: String,
    },
    /// Get an entry from the vault
    Get {
        /// Name of the entry
        name: String,
    },
    /// List all entries
    List,
    /// Delete an entry
    Delete {
        /// Name of the entry
        name: String,
    },
    /// Launch the graphical interface
    Gui,
    /// Launch the web server
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let db_path = get_db_path();

    match cli.command {
        Commands::Init => init_vault(&db_path)?,
        Commands::Add { name, value } => add_entry(&db_path, &name, &value)?,
        Commands::Get { name } => get_entry(&db_path, &name)?,
        Commands::List => list_entries(&db_path)?,
        Commands::Delete { name } => delete_entry(&db_path, &name)?,
        Commands::Gui => run_gui(db_path),
        Commands::Server { port } => {
            web::run_server(port, db_path).await?;
        }
    }

    Ok(())
}

pub fn get_db_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".rustvault")
        .join("vault.db")
}

pub fn derive_key(password: &str, salt: &[u8; 32]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    Argon2::default().hash_password_into(password.as_bytes(), salt, &mut key).map_err(|_| anyhow!("Key derivation failed"))?;
    Ok(key)
}

pub fn encrypt(data: &[u8], key: &[u8; 32]) -> Result<(Vec<u8>, [u8; 12])> {
    let nonce: [u8; 12] = rand::random();
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| anyhow!("Invalid key length"))?;
    let ciphertext = cipher.encrypt(&Nonce::from(nonce), data).map_err(|_| anyhow!("Encryption failed"))?;
    Ok((ciphertext, nonce))
}

pub fn decrypt(ciphertext: &[u8], key: &[u8; 32], nonce: &[u8; 12]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| anyhow!("Invalid key length"))?;
    cipher.decrypt(&Nonce::from(*nonce), ciphertext).map_err(|_| anyhow!("Decryption failed"))
}

fn prompt_password() -> Result<String> {
    print!("Enter master password: ");
    io::stdout().flush()?;
    let password = read_password()?;
    Ok(password)
}

fn init_vault(db_path: &PathBuf) -> Result<()> {
    if db_path.exists() {
        return Err(anyhow!("Vault already exists"));
    }

    let password1 = prompt_password()?;
    print!("Confirm master password: ");
    io::stdout().flush()?;
    let password2 = read_password()?;

    if password1 != password2 {
        return Err(anyhow!("Passwords do not match"));
    }

    fs::create_dir_all(db_path.parent().unwrap())?;

    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE meta (key TEXT PRIMARY KEY, value BLOB)",
        [],
    )?;

    conn.execute(
        "CREATE TABLE entries (name TEXT PRIMARY KEY, nonce BLOB, ciphertext BLOB)",
        [],
    )?;

    let salt: [u8; 32] = rand::random();
    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        params!["salt", &salt[..]],
    )?;

    conn.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        params!["version", "1.0"],
    )?;

    println!("Vault initialized successfully.");
    Ok(())
}

pub fn get_key(db_path: &PathBuf, password: &str) -> Result<[u8; 32]> {
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
    let salt: Vec<u8> = stmt.query_row(params!["salt"], |row| row.get(0))?;
    let salt: [u8; 32] = salt.try_into().map_err(|_| anyhow!("Invalid salt"))?;
    derive_key(password, &salt)
}

fn add_entry(db_path: &PathBuf, name: &str, value: &str) -> Result<()> {
    if !db_path.exists() {
        return Err(anyhow!("Vault does not exist. Run 'init' first."));
    }

    let password = prompt_password()?;
    let key = get_key(db_path, &password)?;
    let (ciphertext, nonce) = encrypt(value.as_bytes(), &key)?;

    let conn = Connection::open(db_path)?;
    conn.execute(
        "INSERT OR REPLACE INTO entries (name, nonce, ciphertext) VALUES (?1, ?2, ?3)",
        params![name, &nonce[..], &ciphertext],
    )?;

    println!("Entry added.");
    Ok(())
}

fn get_entry(db_path: &PathBuf, name: &str) -> Result<()> {
    if !db_path.exists() {
        return Err(anyhow!("Vault does not exist."));
    }

    let password = prompt_password()?;
    let key = get_key(db_path, &password)?;
    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT nonce, ciphertext FROM entries WHERE name = ?1")?;
    let result = stmt.query_row(params![name], |row| {
        let nonce: Vec<u8> = row.get(0)?;
        let ciphertext: Vec<u8> = row.get(1)?;
        Ok((nonce, ciphertext))
    });

    match result {
        Ok((nonce, ciphertext)) => {
            let nonce: [u8; 12] = nonce.try_into().map_err(|_| anyhow!("Invalid nonce"))?;
            let decrypted = decrypt(&ciphertext, &key, &nonce)?;
            let value = String::from_utf8(decrypted).map_err(|_| anyhow!("Invalid UTF-8"))?;
            println!("{}", value);
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => println!("Entry not found."),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

fn list_entries(db_path: &PathBuf) -> Result<()> {
    if !db_path.exists() {
        return Err(anyhow!("Vault does not exist."));
    }

    let conn = Connection::open(db_path)?;
    let mut stmt = conn.prepare("SELECT name FROM entries")?;
    let names = stmt.query_map([], |row| row.get::<_, String>(0))?;

    for name in names {
        println!("{}", name?);
    }
    Ok(())
}

fn delete_entry(db_path: &PathBuf, name: &str) -> Result<()> {
    if !db_path.exists() {
        return Err(anyhow!("Vault does not exist."));
    }

    let conn = Connection::open(db_path)?;
    let changes = conn.execute("DELETE FROM entries WHERE name = ?1", params![name])?;
    if changes > 0 {
        println!("Entry deleted.");
    } else {
        println!("Entry not found.");
    }
    Ok(())
}

struct VaultApp {
    db_path: PathBuf,
    sealed: bool,
    password: String,
    entries: Vec<(String, String)>,
    new_name: String,
    new_value: String,
    error: String,
}

impl VaultApp {
    fn new(db_path: PathBuf) -> Self {
        Self {
            db_path,
            sealed: true,
            password: String::new(),
            entries: Vec::new(),
            new_name: String::new(),
            new_value: String::new(),
            error: String::new(),
        }
    }

    fn load_entries(&self, key: &[u8; 32]) -> Result<Vec<(String, String)>> {
        let conn = Connection::open(&self.db_path)?;
        let mut stmt = conn.prepare("SELECT name, nonce, ciphertext FROM entries")?;
        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let nonce: Vec<u8> = row.get(1)?;
            let ciphertext: Vec<u8> = row.get(2)?;
            Ok((name, nonce, ciphertext))
        })?;
        let mut entries = Vec::new();
        for row in rows {
            let (name, nonce, ciphertext) = row?;
            let nonce: [u8; 12] = nonce.try_into().map_err(|_| anyhow!("Invalid nonce"))?;
            let decrypted = decrypt(&ciphertext, key, &nonce)?;
            let value = String::from_utf8(decrypted).map_err(|_| anyhow!("Invalid UTF-8"))?;
            entries.push((name, value));
        }
        Ok(entries)
    }

    fn add_entry_gui(&self, name: &str, value: &str, key: &[u8; 32]) -> Result<()> {
        let (ciphertext, nonce) = encrypt(value.as_bytes(), key)?;
        let conn = Connection::open(&self.db_path)?;
        conn.execute(
            "INSERT OR REPLACE INTO entries (name, nonce, ciphertext) VALUES (?1, ?2, ?3)",
            params![name, &nonce[..], &ciphertext],
        )?;
        Ok(())
    }

    fn delete_entry_gui(&self, name: &str) -> Result<()> {
        let conn = Connection::open(&self.db_path)?;
        conn.execute("DELETE FROM entries WHERE name = ?1", params![name])?;
        Ok(())
    }
}

fn run_gui(db_path: PathBuf) {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "RustVault",
        options,
        Box::new(|_cc| Box::new(VaultApp::new(db_path))),
    ).unwrap();
}

impl eframe::App for VaultApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if self.sealed {
                ui.heading("RustVault - Scellé");
                ui.label("Mot de passe maître:");
                let response = ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                if ui.button("Déverrouiller").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    if let Ok(key) = get_key(&self.db_path, &self.password) {
                        if let Ok(entries) = self.load_entries(&key) {
                            self.entries = entries;
                            self.sealed = false;
                            self.error.clear();
                        } else {
                            self.error = "Erreur de chargement".to_string();
                        }
                    } else {
                        self.error = "Mot de passe incorrect".to_string();
                    }
                }
                if !self.error.is_empty() {
                    ui.colored_label(egui::Color32::RED, &self.error);
                }
            } else {
                ui.heading("RustVault - Déverrouillé");
                if ui.button("Verrouiller").clicked() {
                    self.sealed = true;
                    self.password.clear();
                    self.entries.clear();
                    self.new_name.clear();
                    self.new_value.clear();
                }
                ui.separator();
                ui.label("Entrées:");
                let mut to_remove = None;
                for (idx, (name, value)) in self.entries.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: {}", name, value));
                        if ui.button("Supprimer").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                }
                if let Some(idx) = to_remove {
                    let (name, _) = &self.entries[idx];
                    if let Ok(_) = self.delete_entry_gui(name) {
                        self.entries.remove(idx);
                    }
                }
                ui.separator();
                ui.label("Ajouter une entrée:");
                ui.text_edit_singleline(&mut self.new_name);
                ui.text_edit_singleline(&mut self.new_value);
                if ui.button("Ajouter").clicked() && !self.new_name.is_empty() && !self.new_value.is_empty() {
                    if let Ok(key) = get_key(&self.db_path, &self.password) {
                        if self.add_entry_gui(&self.new_name, &self.new_value, &key).is_ok() {
                            self.entries.push((self.new_name.clone(), self.new_value.clone()));
                            self.new_name.clear();
                            self.new_value.clear();
                        }
                    }
                }
            }
        });
    }
}

