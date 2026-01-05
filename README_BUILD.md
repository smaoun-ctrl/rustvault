# Instructions pour reconstruire le frontend

Le frontend doit être reconstruit après chaque modification du code source pour que les changements soient visibles.

## Problème actuel

L'API `/api/version` fonctionne correctement (testée avec curl), mais la version n'apparaît pas sur la page web car le frontend n'a pas été reconstruit avec les dernières modifications.

## Solution

### Option 1 : Si vous avez Node.js 20.19+ ou 22.12+

```bash
cd web-frontend
npm install
npm run build
cd ..
```

### Option 2 : Si votre Node.js est trop ancien

Vous avez deux options :

1. **Mettre à jour Node.js** (recommandé)
   - Utilisez `nvm` (Node Version Manager) pour installer une version récente
   - Ou téléchargez depuis [nodejs.org](https://nodejs.org)

2. **Utiliser une version compatible de Vite**
   - Modifier `web-frontend/package.json` pour utiliser une version plus ancienne de Vite compatible avec Node.js 18

### Option 3 : Version statique (temporaire)

Si vous ne pouvez pas reconstruire le frontend, vous pouvez modifier directement le fichier HTML dans `web-frontend/dist/index.html` pour ajouter la version manuellement, mais ce n'est pas recommandé car les modifications seront perdues lors du prochain build.

## Vérification

Après avoir reconstruit le frontend :

1. Vérifiez que `web-frontend/dist/` contient les nouveaux fichiers
2. Redémarrez le serveur Rust :
   ```bash
   ./target/release/rustvault server --port 8080
   ```
3. Ouvrez `http://localhost:8080` dans votre navigateur
4. La version devrait apparaître dans un badge à côté du titre "🔐 RustVault"

## Debug

Si la version n'apparaît toujours pas :

1. Ouvrez la console du navigateur (F12)
2. Vérifiez les logs de chargement de la version
3. Vérifiez qu'il n'y a pas d'erreurs JavaScript
4. Testez l'endpoint directement : `curl http://localhost:8080/api/version`

