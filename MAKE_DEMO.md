# 🎬 Guide de Démonstration Cyber-Kube

## Démarrage Rapide

### Option 1: Script Automatique (Recommandé)

**Windows:**
```powershell
.\scripts\demo\demo.ps1
```

**Linux/Mac:**
```bash
chmod +x scripts/demo/demo.sh
./scripts/demo/demo.sh
```

### Option 2: Manuel

```bash
# 1. Démarrer les services
docker-compose up -d

# 2. Attendre que les services soient prêts (10-15 secondes)

# 3. Vérifier la santé de l'API
curl http://localhost:8081/api/v1/health

# 4. Ouvrir le dashboard
# Naviguer vers http://localhost:3000
```

## Scénario de Démonstration

### 1. Dashboard Principal

1. Ouvrir http://localhost:3000
2. Observer les statistiques en temps réel
3. Vérifier les graphiques interactifs
4. Tester le mode Dark/Light

### 2. Gestion des Politiques

1. Aller dans "Policies"
2. Créer une nouvelle politique:
   - ID: `policy-demo-1`
   - Name: `Block Crypto Mining`
   - Description: `Detect and block crypto-mining activities`
3. Modifier une politique existante
4. Supprimer une politique

### 3. Alertes de Sécurité

1. Aller dans "Alerts"
2. Observer les alertes en temps réel
3. Filtrer par sévérité (high, medium, low)
4. Résoudre une alerte

### 4. Visualisation 3D du Cluster

1. Aller dans "Cluster"
2. Observer la visualisation 3D interactive
3. Cliquer sur un pod pour voir les détails
4. Observer les connexions entre les nodes

### 5. Rapports et Analytics

1. Aller dans "Reports"
2. Examiner les métriques de sécurité
3. Exporter un rapport (PDF/CSV)

### 6. Test de l'API

```bash
# Health check
curl http://localhost:8081/api/v1/health

# Liste des politiques
curl http://localhost:8081/api/v1/policies

# Créer une politique
curl -X POST http://localhost:8081/api/v1/policies \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-policy",
    "name": "Test Policy",
    "description": "Test Description"
  }'

# Liste des alertes
curl http://localhost:8081/api/v1/alerts

# Créer une alerte
curl -X POST http://localhost:8081/api/v1/alerts \
  -H "Content-Type: application/json" \
  -d '{
    "kind": "Container Escape",
    "severity": "high",
    "message": "Suspicious activity detected"
  }'
```

### 7. Authentification JWT

```bash
# Login
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "password"
  }'

# Utiliser le token reçu
TOKEN="<token-from-login>"
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8081/api/v1/policies
```

### 8. WebSocket (Temps Réel)

Ouvrir la console du navigateur sur le dashboard et observer les événements WebSocket en temps réel.

## Points Clés à Mettre en Avant

✅ **Architecture Zero-Trust**: Toutes les communications sont authentifiées
✅ **Détection en Temps Réel**: eBPF + ML pour détecter les menaces
✅ **Interface Professionnelle**: Dashboard moderne avec visualisation 3D
✅ **Sécurité**: JWT, rate limiting, CORS
✅ **Observabilité**: Métriques Prometheus, logs structurés
✅ **Production-Ready**: CI/CD, Docker, Kubernetes

## Arrêt des Services

```bash
docker-compose down
```

## Dépannage

### API ne démarre pas
- Vérifier que le port 8081 est libre
- Vérifier les logs: `docker-compose logs api-go`

### Dashboard ne se connecte pas
- Vérifier que l'API est accessible
- Vérifier la console du navigateur pour les erreurs CORS

### WebSocket ne fonctionne pas
- Vérifier que le firewall autorise les connexions WebSocket
- Vérifier les logs: `docker-compose logs api-go`
