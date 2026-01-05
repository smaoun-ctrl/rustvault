# Configuration GitHub pour RustVault

Ce guide vous explique comment configurer votre projet RustVault avec GitHub.

## Prérequis

- Compte GitHub créé
- Git installé sur votre machine
- Accès en ligne de commande

## Étapes de configuration

### 1. Vérifier la configuration Git locale

```bash
git config --global user.name "Votre Nom"
git config --global user.email "votre.email@example.com"
```

### 2. Créer un dépôt sur GitHub

1. Connectez-vous à [GitHub](https://github.com)
2. Cliquez sur le bouton **"+"** en haut à droite, puis **"New repository"**
3. Remplissez les informations :
   - **Repository name** : `rustvault` (ou le nom de votre choix)
   - **Description** : "A hyper-secure digital vault with CLI, GUI, and web interface"
   - **Visibilité** : Public ou Private (selon votre préférence)
   - **NE PAS** cocher "Initialize with README" (le projet existe déjà)
4. Cliquez sur **"Create repository"**

### 3. Ajouter les fichiers au dépôt local

```bash
# Vérifier les fichiers à ajouter
git status

# Ajouter tous les fichiers (sauf ceux dans .gitignore)
git add .

# Créer le premier commit
git commit -m "Initial commit: RustVault - Secure vault with CLI, GUI, and web interface"
```

### 4. Connecter le dépôt local à GitHub

Après avoir créé le dépôt sur GitHub, vous verrez des instructions. Utilisez la commande pour un dépôt existant :

```bash
# Remplacer USERNAME par votre nom d'utilisateur GitHub
git remote add origin https://github.com/USERNAME/rustvault.git

# Ou avec SSH (si vous avez configuré une clé SSH)
git remote add origin git@github.com:USERNAME/rustvault.git
```

### 5. Pousser le code vers GitHub

```bash
# Renommer la branche principale en 'main' (si nécessaire)
git branch -M main

# Pousser le code vers GitHub
git push -u origin main
```

## Commandes Git utiles

### Vérifier l'état
```bash
git status
```

### Ajouter des fichiers
```bash
git add <fichier>
git add .  # Tous les fichiers modifiés
```

### Créer un commit
```bash
git commit -m "Description des modifications"
```

### Pousser vers GitHub
```bash
git push
```

### Récupérer les modifications
```bash
git pull
```

### Voir l'historique
```bash
git log --oneline
```

## Configuration SSH (optionnel mais recommandé)

Pour éviter de saisir votre mot de passe à chaque fois :

### 1. Générer une clé SSH
```bash
ssh-keygen -t ed25519 -C "votre.email@example.com"
```

### 2. Ajouter la clé à l'agent SSH
```bash
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519
```

### 3. Copier la clé publique
```bash
cat ~/.ssh/id_ed25519.pub
```

### 4. Ajouter la clé sur GitHub
1. Allez sur GitHub → Settings → SSH and GPG keys
2. Cliquez sur "New SSH key"
3. Collez le contenu de votre clé publique
4. Cliquez sur "Add SSH key"

### 5. Changer l'URL du remote (si vous avez utilisé HTTPS)
```bash
git remote set-url origin git@github.com:USERNAME/rustvault.git
```

## Structure recommandée du dépôt

Votre dépôt devrait contenir :
- `src/` - Code source Rust
- `web-frontend/` - Application Svelte
- `Cargo.toml` - Configuration Rust
- `README.md` - Documentation principale
- `.gitignore` - Fichiers à ignorer
- `build-web.sh` - Script de build

## Badges pour le README (optionnel)

Vous pouvez ajouter des badges dans votre README.md :

```markdown
![Rust](https://img.shields.io/badge/rust-1.0+-orange.svg)
![License](https://img.shields.io/badge/license-MIT-blue.svg)
```

## Workflow de développement recommandé

1. Créer une branche pour une nouvelle fonctionnalité :
```bash
git checkout -b feature/nom-de-la-fonctionnalite
```

2. Faire des commits réguliers :
```bash
git add .
git commit -m "Description claire"
```

3. Pousser la branche :
```bash
git push -u origin feature/nom-de-la-fonctionnalite
```

4. Créer une Pull Request sur GitHub pour fusionner dans `main`

## Protection de la branche main (recommandé)

Sur GitHub, allez dans Settings → Branches et configurez :
- Require pull request reviews before merging
- Require status checks to pass before merging

## Liens utiles

- [Documentation Git](https://git-scm.com/doc)
- [Guide GitHub](https://docs.github.com)
- [GitHub CLI](https://cli.github.com) (alternative en ligne de commande)

