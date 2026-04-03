package main

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
	"github.com/stretchr/testify/assert"
)

func setupRouter() *gin.Engine {
	gin.SetMode(gin.TestMode)
	router := gin.New()
	api := router.Group("/api/v1")
	registerRoutes(api, nil) // No WebSocket manager for tests
	return router
}

func TestCreatePolicy(t *testing.T) {
	router := setupRouter()
	
	policy := Policy{
		ID:          "test-1",
		Name:        "Test Policy",
		Description: "Test Description",
	}
	
	body, _ := json.Marshal(policy)
	req, _ := http.NewRequest("POST", "/api/v1/policies", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusCreated, w.Code)
	
	var response Policy
	json.Unmarshal(w.Body.Bytes(), &response)
	assert.Equal(t, policy.ID, response.ID)
	assert.Equal(t, policy.Name, response.Name)
}

func TestListPolicies(t *testing.T) {
	router := setupRouter()
	
	// Create a policy first
	policy := Policy{ID: "test-1", Name: "Test"}
	body, _ := json.Marshal(policy)
	req, _ := http.NewRequest("POST", "/api/v1/policies", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	// List policies
	req, _ = http.NewRequest("GET", "/api/v1/policies", nil)
	w = httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusOK, w.Code)
	
	var policies []Policy
	json.Unmarshal(w.Body.Bytes(), &policies)
	assert.Greater(t, len(policies), 0)
}

func TestGetPolicy(t *testing.T) {
	router := setupRouter()
	
	// Create a policy
	policy := Policy{ID: "test-get", Name: "Test Get"}
	body, _ := json.Marshal(policy)
	req, _ := http.NewRequest("POST", "/api/v1/policies", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	// Get the policy
	req, _ = http.NewRequest("GET", "/api/v1/policies/test-get", nil)
	w = httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusOK, w.Code)
	
	var response Policy
	json.Unmarshal(w.Body.Bytes(), &response)
	assert.Equal(t, "test-get", response.ID)
}

func TestDeletePolicy(t *testing.T) {
	router := setupRouter()
	
	// Create a policy
	policy := Policy{ID: "test-delete", Name: "Test Delete"}
	body, _ := json.Marshal(policy)
	req, _ := http.NewRequest("POST", "/api/v1/policies", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	// Delete the policy
	req, _ = http.NewRequest("DELETE", "/api/v1/policies/test-delete", nil)
	w = httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusNoContent, w.Code)
	
	// Verify it's deleted
	req, _ = http.NewRequest("GET", "/api/v1/policies/test-delete", nil)
	w = httptest.NewRecorder()
	router.ServeHTTP(w, req)
	assert.Equal(t, http.StatusNotFound, w.Code)
}

func TestCreateAlert(t *testing.T) {
	router := setupRouter()
	
	alert := Alert{
		Kind:     "Test Alert",
		Severity: "high",
		Message:  "Test message",
	}
	
	body, _ := json.Marshal(alert)
	req, _ := http.NewRequest("POST", "/api/v1/alerts", bytes.NewBuffer(body))
	req.Header.Set("Content-Type", "application/json")
	
	w := httptest.NewRecorder()
	router.ServeHTTP(w, req)
	
	assert.Equal(t, http.StatusCreated, w.Code)
	
	var response Alert
	json.Unmarshal(w.Body.Bytes(), &response)
	assert.Equal(t, alert.Kind, response.Kind)
	assert.NotEmpty(t, response.ID)
}
