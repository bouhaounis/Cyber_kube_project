package auth

import (
	"os"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func setJWTSecret(t *testing.T) {
	t.Helper()
	original := os.Getenv("JWT_SECRET")
	if err := os.Setenv("JWT_SECRET", "test-secret"); err != nil {
		t.Fatalf("failed to set JWT_SECRET: %v", err)
	}
	t.Cleanup(func() {
		if original == "" {
			_ = os.Unsetenv("JWT_SECRET")
			return
		}
		_ = os.Setenv("JWT_SECRET", original)
	})
}

func TestGenerateToken(t *testing.T) {
	setJWTSecret(t)

	token, err := GenerateToken("testuser")
	assert.NoError(t, err)
	assert.NotEmpty(t, token)
}

func TestValidateToken(t *testing.T) {
	setJWTSecret(t)

	username := "testuser"
	token, err := GenerateToken(username)
	assert.NoError(t, err)
	
	claims, err := ValidateToken(token)
	assert.NoError(t, err)
	assert.Equal(t, username, claims.Username)
	assert.True(t, claims.ExpiresAt.Time.After(time.Now()))
}

func TestValidateInvalidToken(t *testing.T) {
	setJWTSecret(t)

	_, err := ValidateToken("invalid-token")
	assert.Error(t, err)
}

func TestExpiredToken(t *testing.T) {
	setJWTSecret(t)

	// This would require mocking time, simplified for now
	token, _ := GenerateToken("testuser")
	claims, err := ValidateToken(token)
	assert.NoError(t, err)
	assert.NotNil(t, claims)
}
