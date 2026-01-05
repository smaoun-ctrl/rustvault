# Guide de Configuration - RustVault Multi-Tenant

## Configuration initiale

### 1. Initialiser la base de données

```bash
./target/release/rustvault init-db
```

Cette commande crée la structure de base de données avec les tables nécessaires.

**Note** : Si une base de données existe déjà, vous avez deux options :

1. **Supprimer manuellement** la base de données :
```bash
rm ~/.rustvault/vault.db
./target/release/rustvault init-db
```

2. **Utiliser l'option --force** pour supprimer et réinitialiser automatiquement :
```bash
./target/release/rustvault init-db --force
```

⚠️ **Attention** : L'option `--force` supprime définitivement toutes les données existantes !

### 2. Créer un superuser

Le superuser peut gérer tous les tenants mais n'a pas accès aux entrées cryptées (sécurité).

```bash
./target/release/rustvault create-superuser admin
```

Vous serez invité à saisir et confirmer le mot de passe.

**Exemple** :
- Username : `admin`
- Password : (vous choisissez, par exemple `admin123`)

### 3. Créer un tenant

Un tenant représente une organisation ou un groupe qui aura son propre coffre.

```bash
./target/release/rustvault create-tenant "Mon Entreprise"
```

Cette commande retourne l'ID du tenant créé (par exemple : `1`).

### 4. Créer un utilisateur pour le tenant

Chaque tenant peut avoir plusieurs utilisateurs. Chaque utilisateur peut accéder aux entrées de son tenant.

```bash
./target/release/rustvault create-user 1 utilisateur1
```

Vous serez invité à saisir et confirmer le mot de passe pour cet utilisateur.

**Exemple** :
- Tenant ID : `1` (celui créé à l'étape précédente)
- Username : `utilisateur1`
- Password : (vous choisissez, par exemple `user123`)

### 5. Lister les tenants

Pour voir tous les tenants créés :

```bash
./target/release/rustvault list-tenants
```

## Connexion via l'interface web

### Pour le superuser

1. Ouvrez `http://localhost:8080`
2. Username : `admin` (ou le nom que vous avez choisi)
3. Password : le mot de passe que vous avez défini
4. Tenant ID : laissez vide (pour superuser)

### Pour un utilisateur tenant

1. Ouvrez `http://localhost:8080`
2. Username : `utilisateur1` (ou le nom que vous avez choisi)
3. Password : le mot de passe que vous avez défini
4. Tenant ID : `1` (l'ID du tenant)

## Exemple de configuration complète

```bash
# 1. Initialiser
./target/release/rustvault init-db

# 2. Créer le superuser
./target/release/rustvault create-superuser admin
# Mot de passe : Admin123!

# 3. Créer un tenant
./target/release/rustvault create-tenant "Acme Corp"
# Retourne : Tenant 'Acme Corp' created with ID: 1

# 4. Créer un utilisateur pour ce tenant
./target/release/rustvault create-user 1 john
# Mot de passe : John2024!

# 5. Lancer le serveur
./target/release/rustvault server --port 8080
```

## Notes importantes

- **Superuser** : Peut créer/supprimer des tenants, mais ne peut pas voir les entrées cryptées
- **Utilisateur tenant** : Peut gérer les entrées de son tenant uniquement
- **Isolation** : Chaque tenant a son propre salt de chiffrement, les données sont complètement isolées
- **Sécurité** : Les mots de passe sont hashés avec Argon2

## Résolution de problèmes

### "Database already exists"
Si vous avez déjà une ancienne base de données, vous avez deux options :

**Option 1 : Utiliser --force (recommandé)**
```bash
./target/release/rustvault init-db --force
```
⚠️ Cela supprime définitivement toutes les données existantes !

**Option 2 : Supprimer manuellement**
```bash
# Sauvegarder si nécessaire
cp ~/.rustvault/vault.db ~/.rustvault/vault.db.backup

# Supprimer la base
rm ~/.rustvault/vault.db

# Réinitialiser
./target/release/rustvault init-db
```

### "User already exists"
Un username doit être unique dans tout le système. Choisissez un autre nom.

### "Tenant not found"
Vérifiez l'ID du tenant avec `list-tenants` avant de créer un utilisateur.

