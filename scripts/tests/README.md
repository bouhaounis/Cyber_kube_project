# Cyber-Kube Test Scripts

## Test API (PowerShell)

Teste automatiquement tous les endpoints de l'API :

```powershell
.\scripts\tests\test-api.ps1
```

Ce script teste :
- ✅ Health check
- ✅ Metrics
- ✅ CRUD Policies (Create, Read, Update, Delete)
- ✅ CRUD Alerts
- ✅ Multiple policies creation

## Test Dashboard (Browser)

Ouvre le fichier `test-dashboard.html` dans ton navigateur :

```powershell
# Option 1: Double-clic sur le fichier
# Option 2: Ouvrir depuis PowerShell
Start-Process "scripts\tests\test-dashboard.html"
```

Ce fichier HTML permet de tester l'API directement depuis le navigateur avec des boutons interactifs.

## Tests Manuels

### Test API avec curl

```powershell
# Health
curl.exe http://localhost:8081/api/v1/health

# Create Policy
curl.exe -X POST http://localhost:8081/api/v1/policies `
  -H "Content-Type: application/json" `
  -d '{\"id\":\"test\",\"name\":\"Test\",\"description\":\"Test\"}'

# List Policies
curl.exe http://localhost:8081/api/v1/policies
```

### Test depuis le navigateur console

Ouvre la console (F12) et teste :

```javascript
fetch('http://localhost:8081/api/v1/health')
  .then(r => r.json())
  .then(console.log);
```
