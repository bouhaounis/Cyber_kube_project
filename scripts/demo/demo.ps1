# Cyber-Kube Demo Script for Windows
Write-Host "🚀 Cyber-Kube Demo Script" -ForegroundColor Green
Write-Host "==========================" -ForegroundColor Green
Write-Host ""

# Check dependencies
Write-Host "Checking dependencies..." -ForegroundColor Blue
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "Docker is required but not installed." -ForegroundColor Red
    exit 1
}
if (-not (Get-Command docker-compose -ErrorAction SilentlyContinue)) {
    Write-Host "docker-compose is required but not installed." -ForegroundColor Red
    exit 1
}

# Start services
Write-Host "Starting services..." -ForegroundColor Green
docker-compose up -d

# Wait for services
Write-Host "Waiting for services to be ready..." -ForegroundColor Blue
Start-Sleep -Seconds 10

# Check API health
Write-Host "Checking API health..." -ForegroundColor Blue
$maxRetries = 30
$retryCount = 0
$apiReady = $false

while ($retryCount -lt $maxRetries) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8081/api/v1/health" -UseBasicParsing -ErrorAction Stop
        if ($response.StatusCode -eq 200) {
            Write-Host "✓ API is ready" -ForegroundColor Green
            $apiReady = $true
            break
        }
    } catch {
        # Continue retrying
    }
    $retryCount++
    Start-Sleep -Seconds 1
}

if (-not $apiReady) {
    Write-Host "✗ API failed to start" -ForegroundColor Red
    exit 1
}

# Create test policies
Write-Host "Creating test policies..." -ForegroundColor Blue
$policy1 = @{
    id = "policy-1"
    name = "Block Container Escape"
    description = "Detect and block container escape attempts"
} | ConvertTo-Json

$policy2 = @{
    id = "policy-2"
    name = "Network Isolation"
    description = "Enforce network policies between namespaces"
} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "http://localhost:8081/api/v1/policies" -Method Post -Body $policy1 -ContentType "application/json" | Out-Null
    Invoke-RestMethod -Uri "http://localhost:8081/api/v1/policies" -Method Post -Body $policy2 -ContentType "application/json" | Out-Null
    Write-Host "✓ Policies created" -ForegroundColor Green
} catch {
    Write-Host "Warning: Failed to create policies" -ForegroundColor Yellow
}

# Create test alerts
Write-Host "Creating test alerts..." -ForegroundColor Blue
$alert1 = @{
    kind = "Container Escape"
    severity = "high"
    message = "Suspicious process detected attempting to escape container"
} | ConvertTo-Json

$alert2 = @{
    kind = "Network Scan"
    severity = "medium"
    message = "Multiple connection attempts detected"
} | ConvertTo-Json

try {
    Invoke-RestMethod -Uri "http://localhost:8081/api/v1/alerts" -Method Post -Body $alert1 -ContentType "application/json" | Out-Null
    Invoke-RestMethod -Uri "http://localhost:8081/api/v1/alerts" -Method Post -Body $alert2 -ContentType "application/json" | Out-Null
    Write-Host "✓ Alerts created" -ForegroundColor Green
} catch {
    Write-Host "Warning: Failed to create alerts" -ForegroundColor Yellow
}

# Display summary
Write-Host ""
Write-Host "==========================" -ForegroundColor Green
Write-Host "Demo Setup Complete!" -ForegroundColor Green
Write-Host "==========================" -ForegroundColor Green
Write-Host ""
Write-Host "Services:" -ForegroundColor Blue
Write-Host "  • API: http://localhost:8081"
Write-Host "  • Dashboard: http://localhost:3000"
Write-Host "  • Metrics: http://localhost:8081/api/v1/metrics"
Write-Host ""
Write-Host "Test Endpoints:" -ForegroundColor Blue
Write-Host "  • Health: Invoke-WebRequest http://localhost:8081/api/v1/health"
Write-Host "  • Policies: Invoke-RestMethod http://localhost:8081/api/v1/policies"
Write-Host "  • Alerts: Invoke-RestMethod http://localhost:8081/api/v1/alerts"
Write-Host ""
Write-Host "To stop services: docker-compose down" -ForegroundColor Yellow
