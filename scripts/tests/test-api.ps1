# Cyber-Kube API Test Script
# Tests all API endpoints automatically

$API_URL = "http://localhost:8081"
$TestResults = @()

function Test-Endpoint {
    param(
        [string]$Name,
        [string]$Method,
        [string]$Endpoint,
        [object]$Body = $null,
        [int]$ExpectedStatus = 200
    )
    
    Write-Host "`n🧪 Testing: $Name" -ForegroundColor Cyan
    Write-Host "   $Method $Endpoint" -ForegroundColor Gray
    
    try {
        $params = @{
            Uri = "$API_URL$Endpoint"
            Method = $Method
            ContentType = "application/json"
            ErrorAction = "Stop"
        }
        
        if ($Body) {
            $params.Body = ($Body | ConvertTo-Json)
        }
        
        $response = Invoke-RestMethod @params
        $statusCode = 200
        
        Write-Host "   ✅ SUCCESS" -ForegroundColor Green
        if ($response) {
            Write-Host "   Response: $($response | ConvertTo-Json -Compress)" -ForegroundColor Gray
        }
        
        $script:TestResults += [PSCustomObject]@{
            Test = $Name
            Status = "✅ PASS"
            Response = $response
        }
        
        return $response
    }
    catch {
        $statusCode = $_.Exception.Response.StatusCode.value__
        Write-Host "   ❌ FAILED: $($_.Exception.Message)" -ForegroundColor Red
        
        $script:TestResults += [PSCustomObject]@{
            Test = $Name
            Status = "❌ FAIL"
            Error = $_.Exception.Message
        }
        
        return $null
    }
}

Write-Host "========================================" -ForegroundColor Yellow
Write-Host "  Cyber-Kube API Test Suite" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "API URL: $API_URL`n" -ForegroundColor Gray

# Test 1: Health Check
Test-Endpoint -Name "Health Check" -Method "GET" -Endpoint "/api/v1/health"

# Test 2: Metrics
Test-Endpoint -Name "Metrics" -Method "GET" -Endpoint "/api/v1/metrics"

# Test 3: List Policies (should be empty)
$policies = Test-Endpoint -Name "List Policies (empty)" -Method "GET" -Endpoint "/api/v1/policies"

# Test 4: Create Policy
$newPolicy = @{
    id = "test-policy-1"
    name = "Test Policy 1"
    description = "This is a test policy created by automated tests"
}
$createdPolicy = Test-Endpoint -Name "Create Policy" -Method "POST" -Endpoint "/api/v1/policies" -Body $newPolicy

# Test 5: Get Policy by ID
if ($createdPolicy) {
    Test-Endpoint -Name "Get Policy by ID" -Method "GET" -Endpoint "/api/v1/policies/test-policy-1"
}

# Test 6: Update Policy
if ($createdPolicy) {
    $updatedPolicy = @{
        id = "test-policy-1"
        name = "Updated Test Policy"
        description = "This policy has been updated"
    }
    Test-Endpoint -Name "Update Policy" -Method "PUT" -Endpoint "/api/v1/policies/test-policy-1" -Body $updatedPolicy
}

# Test 7: List Policies (should have 1)
Test-Endpoint -Name "List Policies (with data)" -Method "GET" -Endpoint "/api/v1/policies"

# Test 8: List Alerts (should be empty)
Test-Endpoint -Name "List Alerts (empty)" -Method "GET" -Endpoint "/api/v1/alerts"

# Test 9: Create Alert
$newAlert = @{
    id = "test-alert-1"
    kind = "CONTAINER_ESCAPE"
    severity = "high"
    message = "Container escape attempt detected by automated test"
}
$createdAlert = Test-Endpoint -Name "Create Alert" -Method "POST" -Endpoint "/api/v1/alerts" -Body $newAlert

# Test 10: List Alerts (should have 1)
Test-Endpoint -Name "List Alerts (with data)" -Method "GET" -Endpoint "/api/v1/alerts"

# Test 11: Create Multiple Policies
Write-Host "`n🧪 Creating multiple policies..." -ForegroundColor Cyan
1..5 | ForEach-Object {
    $policy = @{
        id = "test-policy-$_"
        name = "Test Policy $_"
        description = "Automated test policy $_"
    }
    Test-Endpoint -Name "Create Policy $_" -Method "POST" -Endpoint "/api/v1/policies" -Body $policy | Out-Null
}

# Test 12: Final List Policies
$finalPolicies = Test-Endpoint -Name "Final List Policies" -Method "GET" -Endpoint "/api/v1/policies"

# Test 13: Delete Policy
Test-Endpoint -Name "Delete Policy" -Method "DELETE" -Endpoint "/api/v1/policies/test-policy-1"

# Summary
Write-Host "`n========================================" -ForegroundColor Yellow
Write-Host "  Test Summary" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow

$passed = ($TestResults | Where-Object { $_.Status -eq "✅ PASS" }).Count
$failed = ($TestResults | Where-Object { $_.Status -eq "❌ FAIL" }).Count
$total = $TestResults.Count

Write-Host "`nTotal Tests: $total" -ForegroundColor White
Write-Host "✅ Passed: $passed" -ForegroundColor Green
Write-Host "❌ Failed: $failed" -ForegroundColor $(if ($failed -gt 0) { "Red" } else { "Green" })

if ($failed -eq 0) {
    Write-Host "`n🎉 All tests passed!" -ForegroundColor Green
} else {
    Write-Host "`n⚠️  Some tests failed. Check the output above." -ForegroundColor Yellow
}

Write-Host "`n========================================" -ForegroundColor Yellow
