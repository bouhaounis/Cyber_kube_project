package models

import (
	"time"
)

// Policy represents a security policy
type Policy struct {
	ID          string    `gorm:"primaryKey" json:"id"`
	Name        string    `gorm:"not null" json:"name"`
	Description string    `json:"description"`
	Enabled     bool      `gorm:"default:true" json:"enabled"`
	Rules       string    `gorm:"type:text" json:"rules"` // JSON string
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
}

// Alert represents a security alert
type Alert struct {
	ID        uint      `gorm:"primaryKey" json:"id"`
	Kind      string    `gorm:"not null;index" json:"kind"`
	Severity  string    `gorm:"not null;index" json:"severity"` // high, medium, low
	Message   string    `gorm:"type:text" json:"message"`
	Source    string    `json:"source"` // pod name, node, etc.
	Namespace string    `json:"namespace"`
	Resolved  bool      `gorm:"default:false" json:"resolved"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

// User represents an API user
type User struct {
	ID        uint      `gorm:"primaryKey" json:"id"`
	Username  string    `gorm:"uniqueIndex;not null" json:"username"`
	Email     string    `gorm:"uniqueIndex;not null" json:"email"`
	PasswordHash string `gorm:"not null" json:"-"`
	Role      string    `gorm:"default:user" json:"role"` // admin, user, viewer
	Active    bool      `gorm:"default:true" json:"active"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

// Event represents a security event
type Event struct {
	ID        uint      `gorm:"primaryKey" json:"id"`
	Type      string    `gorm:"not null;index" json:"type"`
	Source    string    `json:"source"`
	Payload   string    `gorm:"type:text" json:"payload"` // JSON string
	Processed bool      `gorm:"default:false" json:"processed"`
	CreatedAt time.Time `json:"created_at"`
}
