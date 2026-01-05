<script>
  import { onMount } from 'svelte';

  let username = '';
  let password = '';
  let tenantId = '';
  let isLocked = true;
  let error = '';
  let entries = [];
  let newName = '';
  let newValue = '';
  let loading = false;
  let version = 'v0.1.0'; // Version par défaut
  let currentUser = null;
  let currentTenant = null;
  let isSuperuser = false;

  const API_BASE = '/api';

  // Charger la version au démarrage
  async function loadVersion() {
    try {
      console.log('Chargement de la version depuis:', `${API_BASE}/version`);
      const response = await fetch(`${API_BASE}/version`);
      console.log('Réponse reçue:', response.status, response.statusText);
      const data = await response.json();
      console.log('Données reçues:', data);
      if (data.success && data.data) {
        version = `v${data.data.version}`;
        console.log('Version définie à:', version);
      } else {
        console.error('Erreur API version:', data);
        version = 'v0.1.0';
      }
    } catch (e) {
      console.error('Erreur lors du chargement de la version:', e);
      // En cas d'erreur, on peut afficher une version par défaut
      version = 'v0.1.0';
    }
  }

  // Charger la version au montage du composant
  onMount(() => {
    loadVersion();
  });

  async function login() {
    if (!username || !password) {
      error = 'Veuillez entrer un nom d\'utilisateur et un mot de passe';
      return;
    }

    loading = true;
    error = '';

    try {
      const loginData = {
        username: username,
        password: password,
        tenant_id: tenantId ? parseInt(tenantId) : null
      };

      const response = await fetch(`${API_BASE}/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(loginData),
      });

      const data = await response.json();

      if (data.success && data.data) {
        isLocked = false;
        currentUser = data.data.user;
        currentTenant = data.data.tenant;
        isSuperuser = data.data.is_superuser;
        username = '';
        password = '';
        tenantId = '';
        
        if (!isSuperuser) {
          await loadEntries();
        }
      } else {
        error = data.error || 'Identifiants incorrects';
      }
    } catch (e) {
      error = 'Erreur de connexion au serveur';
      console.error(e);
    } finally {
      loading = false;
    }
  }

  async function logout() {
    loading = true;
    try {
      await fetch(`${API_BASE}/logout`, {
        method: 'POST',
      });
      isLocked = true;
      entries = [];
      username = '';
      password = '';
      tenantId = '';
      currentUser = null;
      currentTenant = null;
      isSuperuser = false;
      error = '';
    } catch (e) {
      console.error(e);
    } finally {
      loading = false;
    }
  }

  async function loadEntries() {
    loading = true;
    error = '';

    try {
      const response = await fetch(`${API_BASE}/entries`);
      const data = await response.json();

      if (data.success) {
        entries = data.data || [];
      } else {
        error = data.error || 'Erreur lors du chargement des entrées';
      }
    } catch (e) {
      error = 'Erreur de connexion au serveur';
      console.error(e);
    } finally {
      loading = false;
    }
  }

  async function addEntry() {
    if (!newName || !newValue) {
      error = 'Veuillez remplir tous les champs';
      return;
    }

    loading = true;
    error = '';

    try {
      const response = await fetch(`${API_BASE}/entries`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          name: newName,
          value: newValue,
        }),
      });

      const data = await response.json();

      if (data.success) {
        newName = '';
        newValue = '';
        await loadEntries();
      } else {
        error = data.error || 'Erreur lors de l\'ajout';
      }
    } catch (e) {
      error = 'Erreur de connexion au serveur';
      console.error(e);
    } finally {
      loading = false;
    }
  }

  async function deleteEntry(name) {
    if (!confirm(`Êtes-vous sûr de vouloir supprimer l'entrée "${name}" ?`)) {
      return;
    }

    loading = true;
    error = '';

    try {
      const response = await fetch(`${API_BASE}/entries/delete`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ name }),
      });

      const data = await response.json();

      if (data.success) {
        await loadEntries();
      } else {
        error = data.error || 'Erreur lors de la suppression';
      }
    } catch (e) {
      error = 'Erreur de connexion au serveur';
      console.error(e);
    } finally {
      loading = false;
    }
  }

  function handleKeyPress(event, action) {
    if (event.key === 'Enter') {
      action();
    }
  }
</script>

<main>
  <div class="container">
    <div class="header-title">
      <h1>🔐 RustVault</h1>
      <span class="version-badge">{version || 'v0.1.0'}</span>
    </div>

    {#if isLocked}
      <div class="lock-screen">
        <h2>Connexion</h2>
        <p>Connectez-vous pour accéder à votre coffre</p>
        
        <div class="login-form">
          <input
            type="text"
            bind:value={username}
            placeholder="Nom d'utilisateur"
            on:keypress={(e) => handleKeyPress(e, login)}
            disabled={loading}
          />
          <input
            type="password"
            bind:value={password}
            placeholder="Mot de passe"
            on:keypress={(e) => handleKeyPress(e, login)}
            disabled={loading}
          />
          <input
            type="number"
            bind:value={tenantId}
            placeholder="ID Tenant (optionnel pour superuser)"
            on:keypress={(e) => handleKeyPress(e, login)}
            disabled={loading}
          />
          <button on:click={login} disabled={loading || !username || !password}>
            {loading ? 'Connexion...' : 'Se connecter'}
          </button>
        </div>

        {#if error}
          <div class="error">{error}</div>
        {/if}
      </div>
    {:else}
      <div class="unlock-screen">
        <div class="header">
          <h2>
            {#if isSuperuser}
              Panel Superuser
            {:else}
              Coffre déverrouillé
              {#if currentTenant}
                - {currentTenant.name}
              {/if}
            {/if}
          </h2>
          <div class="user-info">
            {#if currentUser}
              <span class="user-badge">{currentUser.username}</span>
            {/if}
            <button on:click={logout} class="lock-btn" disabled={loading}>
              🔒 Déconnexion
            </button>
          </div>
        </div>

        {#if error}
          <div class="error">{error}</div>
        {/if}

        {#if isSuperuser}
          <div class="section">
            <h3>Panel Superuser</h3>
            <p>Vous êtes connecté en tant que superuser. Vous pouvez gérer les tenants mais ne pouvez pas accéder aux entrées cryptées.</p>
            <p><em>Les fonctionnalités de gestion des tenants seront disponibles prochainement.</em></p>
          </div>
        {:else}
          <div class="section">
          <h3>Entrées ({entries.length})</h3>
          {#if loading && entries.length === 0}
            <div class="loading">Chargement...</div>
          {:else if entries.length === 0}
            <div class="empty">Aucune entrée</div>
          {:else}
            <div class="entries-list">
              {#each entries as entry}
                <div class="entry">
                  <div class="entry-content">
                    <strong>{entry.name}</strong>
                    <span class="entry-value">{entry.value}</span>
                  </div>
                  <button
                    on:click={() => deleteEntry(entry.name)}
                    class="delete-btn"
                    disabled={loading}
                  >
                    🗑️
                  </button>
                </div>
              {/each}
            </div>
          {/if}
          </div>

          <div class="section">
            <h3>Ajouter une entrée</h3>
            <div class="add-form">
              <input
                type="text"
                bind:value={newName}
                placeholder="Nom de l'entrée"
                disabled={loading}
              />
              <input
                type="text"
                bind:value={newValue}
                placeholder="Valeur"
                on:keypress={(e) => handleKeyPress(e, addEntry)}
                disabled={loading}
              />
              <button on:click={addEntry} disabled={loading || !newName || !newValue}>
                {loading ? 'Ajout...' : '➕ Ajouter'}
              </button>
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  main {
    width: 100%;
    max-width: 800px;
    padding: 20px;
  }

  .container {
    background: white;
    border-radius: 16px;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
    padding: 40px;
  }

  .header-title {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 15px;
    margin-bottom: 30px;
    flex-wrap: wrap;
  }

  h1 {
    text-align: center;
    color: #333;
    margin: 0;
    font-size: 2.5em;
  }

  .version-badge {
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    padding: 6px 12px;
    border-radius: 20px;
    font-size: 0.9em;
    font-weight: 600;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  h2 {
    color: #333;
    margin-bottom: 10px;
  }

  h3 {
    color: #555;
    margin-bottom: 15px;
    font-size: 1.2em;
  }

  .lock-screen {
    text-align: center;
  }

  .lock-screen p {
    color: #666;
    margin-bottom: 30px;
  }

  .login-form {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin-bottom: 20px;
  }

  .login-form input {
    padding: 12px;
    border: 2px solid #ddd;
    border-radius: 8px;
    font-size: 16px;
    transition: border-color 0.3s;
  }

  .login-form input:focus {
    outline: none;
    border-color: #667eea;
  }

  .login-form input:disabled {
    background: #f5f5f5;
    cursor: not-allowed;
  }

  .login-form button {
    margin-top: 10px;
  }

  .input-group {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
  }

  .input-group input {
    flex: 1;
    padding: 12px;
    border: 2px solid #ddd;
    border-radius: 8px;
    font-size: 16px;
    transition: border-color 0.3s;
  }

  .input-group input:focus {
    outline: none;
    border-color: #667eea;
  }

  .input-group input:disabled {
    background: #f5f5f5;
    cursor: not-allowed;
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .user-badge {
    background: #667eea;
    color: white;
    padding: 6px 12px;
    border-radius: 20px;
    font-size: 0.9em;
    font-weight: 600;
  }

  button {
    padding: 12px 24px;
    background: #667eea;
    color: white;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    cursor: pointer;
    transition: background 0.3s;
    font-weight: 600;
  }

  button:hover:not(:disabled) {
    background: #5568d3;
  }

  button:disabled {
    background: #ccc;
    cursor: not-allowed;
  }

  .lock-btn {
    background: #e74c3c;
  }

  .lock-btn:hover:not(:disabled) {
    background: #c0392b;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 30px;
  }

  .section {
    margin-bottom: 30px;
    padding: 20px;
    background: #f8f9fa;
    border-radius: 8px;
  }

  .entries-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .entry {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px;
    background: white;
    border-radius: 8px;
    border: 1px solid #e0e0e0;
  }

  .entry-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }

  .entry-content strong {
    color: #333;
    font-size: 1.1em;
  }

  .entry-value {
    color: #666;
    font-family: monospace;
    word-break: break-all;
  }

  .delete-btn {
    padding: 8px 12px;
    background: #e74c3c;
    font-size: 18px;
    min-width: 50px;
  }

  .delete-btn:hover:not(:disabled) {
    background: #c0392b;
  }

  .add-form {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .add-form input {
    padding: 12px;
    border: 2px solid #ddd;
    border-radius: 8px;
    font-size: 16px;
    transition: border-color 0.3s;
  }

  .add-form input:focus {
    outline: none;
    border-color: #667eea;
  }

  .add-form input:disabled {
    background: #f5f5f5;
    cursor: not-allowed;
  }

  .add-form button {
    align-self: flex-start;
  }

  .error {
    background: #fee;
    color: #c33;
    padding: 12px;
    border-radius: 8px;
    margin-bottom: 20px;
    border: 1px solid #fcc;
  }

  .loading {
    text-align: center;
    color: #666;
    padding: 20px;
  }

  .empty {
    text-align: center;
    color: #999;
    padding: 20px;
    font-style: italic;
  }
</style>
