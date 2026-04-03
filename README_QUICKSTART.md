# 🚀 Cyber-Kube - Quick Start Guide

## Méthode 1 : VS Code (Recommandé) ⭐

### Option A : Utiliser les Tâches
1. Ouvre le projet dans VS Code
2. Appuie sur `Ctrl+Shift+P` (ou `F1`)
3. Tape : `Tasks: Run Task`
4. Sélectionne : **🚀 Start All Services**
5. Ouvre ton navigateur : `http://localhost:3000`

### Option B : Utiliser le Débogage
1. Ouvre le projet dans VS Code
2. Appuie sur `F5` (ou va dans Run and Debug)
3. Sélectionne : **🚀 Launch All Services**
4. Clique sur le bouton Play ▶️
5. Ouvre ton navigateur : `http://localhost:3000`

## Méthode 2 : Double-Clic (Le Plus Simple) 🖱️

1. **Double-clique sur `START.bat`** à la racine du projet
2. Trois fenêtres s'ouvrent automatiquement
3. Ouvre ton navigateur : `http://localhost:3000`

C'est tout ! 🎉

## Méthode 3 : PowerShell

```powershell
.\START.bat
```

## 📋 Vérification

Une fois lancé, teste :

### 1. Health Check API
```powershell
curl.exe http://localhost:8081/api/v1/health
```
Ou avec PowerShell :
```powershell
Invoke-RestMethod -Uri http://localhost:8081/api/v1/health
```
Devrait retourner : `{"status":"ok"}`

### 2. Dashboard
Ouvre : `http://localhost:3000`

### 3. Créer une politique
```powershell
curl.exe -X POST http://localhost:8081/api/v1/policies `
  -H "Content-Type: application/json" `
  -d '{\"id\":\"test\",\"name\":\"Test Policy\",\"description\":\"Test\"}'
```

## 🛠️ Extensions VS Code Recommandées

VS Code te proposera automatiquement d'installer :
- **Go** (golang.go)
- **Rust Analyzer** (rust-lang.rust-analyzer)
- **ESLint** (dbaeumer.vscode-eslint)
- **Prettier** (esbenp.prettier-vscode)

## ⚠️ Dépannage

### Port 8081 déjà utilisé
```powershell
netstat -ano | findstr :8081
taskkill /PID <PID> /F
```

### Erreur "go mod"
```powershell
cd api-go
go mod tidy
```

### Dashboard ne démarre pas
```powershell
cd dashboard
npm install
```

## 🎯 Prochaines Étapes

1. ✅ Lancer les services (choisis une méthode ci-dessus)
2. ✅ Tester le dashboard : `http://localhost:3000`
3. ✅ Tester l'API : `http://localhost:8081/api/v1/health`
4. ✅ Créer des politiques via l'API
5. ✅ Explorer le code dans VS Code

---

**Astuce** : La méthode la plus simple est de **double-cliquer sur `START.bat`** ! 🚀
