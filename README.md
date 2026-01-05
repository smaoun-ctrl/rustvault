# RustVault

A hyper-secure digital vault for storing sensitive information like passwords and secrets.

## Features

- **Strong Encryption**: Uses AES-256-GCM for authenticated encryption.
- **Secure Key Derivation**: Employs Argon2 for password-based key derivation with a random salt.
- **Database Storage**: Stores encrypted entries in a SQLite database for efficient access.
- **CLI Interface**: Simple command-line interface for managing entries.
- **GUI Interface**: Native graphical interface using egui.
- **Web Interface**: Modern web interface with Svelte for remote access.
- **Password Protection**: Master password required for all operations.

## Security Notes

- The vault is stored in `~/.rustvault/vault.db`.
- Each entry is encrypted separately with a unique nonce.
- Salt is stored in the database for key derivation.
- Argon2 parameters provide resistance against brute-force attacks.
- Data integrity is ensured via GCM authentication.

## Installation

Ensure you have Rust installed, then:

```bash
cargo build --release
```

## Usage

Check version:

```bash
./target/release/rustvault --version
```

Initialize the vault:

```bash
./target/release/rustvault init
```

Add an entry:

```bash
./target/release/rustvault add <name> <value>
```

Get an entry:

```bash
./target/release/rustvault get <name>
```

List entries:

```bash
./target/release/rustvault list
```

Delete an entry:

```bash
./target/release/rustvault delete <name>
```

Launch the GUI:

```bash
./target/release/rustvault gui
```

Launch the web server:

```bash
./target/release/rustvault server --port 8080
```

## Web Interface

The web interface allows you to manage your vault from anywhere through a browser.

### Prerequisites

- Node.js 20.19+ or 22.12+ (for building the frontend)
- npm

### Building the Frontend

1. Install dependencies:
```bash
cd web-frontend
npm install
```

2. Build the frontend:
```bash
npm run build
```

Or use the provided script:
```bash
./build-web.sh
```

### Running the Web Server

1. Make sure the frontend is built (see above)
2. Start the server:
```bash
./target/release/rustvault server --port 8080
```

3. Open your browser and navigate to `http://localhost:8080`

The web interface provides:
- Secure unlock with master password
- View all entries
- Add new entries
- Delete entries
- Lock/unlock functionality

### API Endpoints

The server exposes the following REST API endpoints:

- `POST /api/unlock` - Unlock the vault with password
- `POST /api/lock` - Lock the vault
- `GET /api/entries` - List all entries
- `POST /api/entries` - Add a new entry
- `POST /api/entries/get` - Get a specific entry
- `POST /api/entries/delete` - Delete an entry

## Warning

- Keep your master password secure.
- This tool does not protect against keyloggers or shoulder surfing.
- For maximum security, use on a trusted system.
- When using the web interface, ensure you're on a secure network or use HTTPS in production.