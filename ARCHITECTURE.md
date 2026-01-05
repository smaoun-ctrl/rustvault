# Architecture Multi-Tenant RustVault

## Structure de la base de données

### Tables principales

1. **tenants** - Liste des tenants
   - id (INTEGER PRIMARY KEY)
   - name (TEXT UNIQUE)
   - created_at (TEXT)

2. **users** - Utilisateurs (tenants + superuser)
   - id (INTEGER PRIMARY KEY)
   - tenant_id (INTEGER, NULL pour superuser)
   - username (TEXT UNIQUE)
   - password_hash (TEXT)
   - is_superuser (INTEGER, 0 ou 1)
   - created_at (TEXT)

3. **tenant_meta** - Métadonnées par tenant (salt, etc.)
   - tenant_id (INTEGER)
   - key (TEXT)
   - value (BLOB)
   - PRIMARY KEY (tenant_id, key)

4. **tenant_entries** - Entrées cryptées par tenant
   - tenant_id (INTEGER)
   - name (TEXT)
   - nonce (BLOB)
   - ciphertext (BLOB)
   - PRIMARY KEY (tenant_id, name)

## Flux d'authentification

1. **Superuser** :
   - Se connecte avec username/password
   - Peut gérer tous les tenants
   - Peut créer/supprimer des tenants
   - N'a pas accès aux entrées des tenants (sécurité)

2. **Utilisateur tenant** :
   - Se connecte avec username/password + tenant_id
   - Accès uniquement aux entrées de son tenant
   - Chaque tenant a son propre salt pour le chiffrement

## API Endpoints

### Authentification
- `POST /api/login` - Connexion (username, password, tenant_id optionnel)
- `POST /api/logout` - Déconnexion

### Gestion des tenants (superuser uniquement)
- `GET /api/tenants` - Liste des tenants
- `POST /api/tenants` - Créer un tenant
- `DELETE /api/tenants/:id` - Supprimer un tenant

### Gestion des entrées (utilisateur tenant)
- `GET /api/entries` - Lister les entrées du tenant
- `POST /api/entries` - Ajouter une entrée
- `GET /api/entries/:name` - Obtenir une entrée
- `DELETE /api/entries/:name` - Supprimer une entrée

## Sécurité

- Chaque tenant a son propre salt pour le chiffrement
- Les mots de passe sont hashés avec Argon2
- Les sessions sont stockées en mémoire (à améliorer avec tokens JWT)
- Isolation complète des données entre tenants

