#!/bin/bash
# Script pour construire le frontend Svelte

cd web-frontend
npm run build
cd ..

echo "Frontend construit dans web-frontend/dist"

