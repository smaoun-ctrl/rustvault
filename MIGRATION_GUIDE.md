# Guide de Migration vers Multi-Tenant

## Changements majeurs

L'architecture a été complètement refondue pour supporter un système multi-tenant avec un superuser.

## Nouvelle structure

### Initialisation

1. **Initialiser la base de données** :
```bash
./target/release/rustvault init-db
```

2. **Créer un superuser** :
```bash
./target/release/rustvault create-superuser admin
```

3. **Créer un tenant** :
```bash
./target/release/rustvault create-tenant "Company A"
```

4. **Créer un utilisateur pour un tenant** :
```bash
./target/release/rustvault create-user 1 user1
# (1 est l'ID du tenant)
```

## Migration des données existantes

Si vous avez une base de données existante avec l'ancienne structure :

1. Sauvegardez votre base de données actuelle
2. Exportez vos entrées avec l'ancienne version
3. Initialisez la nouvelle base de données
4. Créez un tenant
5. Réimportez vos entrées dans le nouveau tenant

## API Changes

L'API a été complètement refondue :

- Ancien : `POST /api/unlock` avec password
- Nouveau : `POST /api/login` avec username, password, tenant_id (optionnel)

Les endpoints d'entrées fonctionnent maintenant avec le tenant_id de la session.

## Interface Web

L'interface web doit être mise à jour pour :
- Afficher un écran de connexion avec sélection de tenant
- Gérer les sessions utilisateur
- Afficher une interface superuser pour gérer les tenants

