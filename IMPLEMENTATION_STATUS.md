# État d'implémentation - Multi-Tenant

## ✅ Terminé

1. **Structure de base de données** - Tables créées (tenants, users, tenant_meta, tenant_entries)
2. **Module tenant** - Fonctions pour gérer les tenants et utilisateurs
3. **Module vault** - Fonctions pour gérer les entrées par tenant
4. **Module web_session** - Structure de session
5. **Commandes CLI** - init-db, create-superuser, create-tenant, create-user, list-tenants
6. **Handler login** - Authentification multi-tenant
7. **Handler add_entry** - Ajout d'entrées par tenant (partiellement)

## ⚠️ En cours / À compléter

1. **Handlers web restants** :
   - `get_entry_handler` - À réécrire pour utiliser la session
   - `list_entries_handler` - À réécrire pour utiliser la session
   - `delete_entry_handler` - À réécrire pour utiliser la session

2. **Endpoints superuser** :
   - `GET /api/tenants` - Liste des tenants
   - `POST /api/tenants` - Créer un tenant
   - `DELETE /api/tenants/:id` - Supprimer un tenant

3. **Interface web** :
   - Écran de connexion avec sélection de tenant
   - Interface superuser pour gérer les tenants
   - Mise à jour de l'interface tenant

## 🔧 Corrections nécessaires

1. Remplacer toutes les références à `session_key` par `session`
2. Utiliser `SessionStore` au lieu de `Arc<Mutex<Option<[u8; 32]>>>`
3. Vérifier l'authentification dans tous les handlers
4. Gérer les permissions (superuser vs tenant user)

## 📝 Notes

- La clé de chiffrement est stockée dans la session après login
- Le superuser n'a pas de clé de chiffrement (ne gère pas les entrées)
- Chaque tenant a son propre salt pour l'isolation

