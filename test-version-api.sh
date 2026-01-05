#!/bin/bash
# Script pour tester l'endpoint de version

echo "Test de l'endpoint /api/version"
echo "================================"
echo ""

# Démarrer le serveur en arrière-plan (si pas déjà démarré)
# ./target/debug/rustvault server --port 8080 &

# Attendre un peu pour que le serveur démarre
# sleep 2

# Tester l'endpoint
echo "Requête vers http://localhost:8080/api/version"
curl -s http://localhost:8080/api/version | jq .

echo ""
echo "Si vous voyez une erreur de connexion, assurez-vous que le serveur est démarré :"
echo "  ./target/debug/rustvault server --port 8080"

