package main

import (
	"net/http"
	"time"

	"github.com/cyber-kube/api-go/internal/database"
	"github.com/cyber-kube/api-go/internal/models"
	"github.com/cyber-kube/api-go/pkg/websocket"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

func registerRoutes(r *gin.RouterGroup, wsManager *websocket.Manager) {
	// Policies CRUD
	r.POST("/policies", createPolicy)
	r.GET("/policies", listPolicies)
	r.GET("/policies/:id", getPolicy)
	r.PUT("/policies/:id", updatePolicy)
	r.DELETE("/policies/:id", deletePolicy)

	// Alerts
	r.GET("/alerts", listAlerts)
	r.POST("/alerts", createAlert)
	r.PUT("/alerts/:id/resolve", resolveAlert)

	// Events (for WebSocket broadcasting)
	r.POST("/events", func(c *gin.Context) {
		var event models.Event
		if err := c.ShouldBindJSON(&event); err != nil {
			c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
			return
		}
		event.CreatedAt = time.Now()

		// Save to database if available
		if database.DB != nil {
			database.DB.Create(&event)
		}

		// Broadcast via WebSocket
		if wsManager != nil {
			wsManager.BroadcastEvent(event)
		}
		c.JSON(http.StatusCreated, event)
	})
}

func createPolicy(c *gin.Context) {
	var p Policy
	if err := c.ShouldBindJSON(&p); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if p.ID == "" {
		p.ID = uuid.New().String()
	}
	p.CreatedAt = time.Now()

	// Try database first, fallback to in-memory
	if database.DB != nil {
		policy := models.Policy{
			ID:          p.ID,
			Name:        p.Name,
			Description: p.Description,
			Enabled:     true,
			CreatedAt:   p.CreatedAt,
		}
		if err := database.DB.Create(&policy).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		c.JSON(http.StatusCreated, p)
		return
	}

	// In-memory fallback
	policies[p.ID] = p
	c.JSON(http.StatusCreated, p)
}

func listPolicies(c *gin.Context) {
	if database.DB != nil {
		var dbPolicies []models.Policy
		if err := database.DB.Find(&dbPolicies).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		result := make([]Policy, len(dbPolicies))
		for i, p := range dbPolicies {
			result[i] = Policy{
				ID:          p.ID,
				Name:        p.Name,
				Description: p.Description,
				CreatedAt:   p.CreatedAt,
			}
		}
		c.JSON(http.StatusOK, result)
		return
	}

	// In-memory fallback
	list := make([]Policy, 0, len(policies))
	for _, p := range policies {
		list = append(list, p)
	}
	c.JSON(http.StatusOK, list)
}

func getPolicy(c *gin.Context) {
	id := c.Param("id")

	if database.DB != nil {
		var policy models.Policy
		if err := database.DB.Where("id = ?", id).First(&policy).Error; err != nil {
			c.JSON(http.StatusNotFound, gin.H{"error": "not found"})
			return
		}
		c.JSON(http.StatusOK, Policy{
			ID:          policy.ID,
			Name:        policy.Name,
			Description: policy.Description,
			CreatedAt:   policy.CreatedAt,
		})
		return
	}

	// In-memory fallback
	if p, ok := policies[id]; ok {
		c.JSON(http.StatusOK, p)
		return
	}
	c.JSON(http.StatusNotFound, gin.H{"error": "not found"})
}

func updatePolicy(c *gin.Context) {
	id := c.Param("id")
	var p Policy
	if err := c.ShouldBindJSON(&p); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}
	p.ID = id
	p.CreatedAt = time.Now()

	if database.DB != nil {
		policy := models.Policy{
			ID:          p.ID,
			Name:        p.Name,
			Description: p.Description,
		}
		if err := database.DB.Model(&models.Policy{}).Where("id = ?", id).Updates(policy).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		c.JSON(http.StatusOK, p)
		return
	}

	// In-memory fallback
	policies[id] = p
	c.JSON(http.StatusOK, p)
}

func deletePolicy(c *gin.Context) {
	id := c.Param("id")

	if database.DB != nil {
		if err := database.DB.Where("id = ?", id).Delete(&models.Policy{}).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		c.JSON(http.StatusNoContent, gin.H{})
		return
	}

	// In-memory fallback
	delete(policies, id)
	c.JSON(http.StatusNoContent, gin.H{})
}

func listAlerts(c *gin.Context) {
	if database.DB != nil {
		var dbAlerts []models.Alert
		query := database.DB.Order("created_at DESC")

		// Filter by severity if provided
		if severity := c.Query("severity"); severity != "" {
			query = query.Where("severity = ?", severity)
		}

		// Filter by resolved status
		if resolved := c.Query("resolved"); resolved != "" {
			query = query.Where("resolved = ?", resolved == "true")
		}

		if err := query.Find(&dbAlerts).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}

		result := make([]Alert, len(dbAlerts))
		for i, a := range dbAlerts {
			result[i] = Alert{
				ID:        string(rune(a.ID)),
				Kind:      a.Kind,
				Severity:  a.Severity,
				Message:   a.Message,
				CreatedAt: a.CreatedAt,
			}
		}
		c.JSON(http.StatusOK, result)
		return
	}

	// In-memory fallback
	c.JSON(http.StatusOK, alerts)
}

func createAlert(c *gin.Context) {
	var a Alert
	if err := c.ShouldBindJSON(&a); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	if a.ID == "" {
		a.ID = uuid.New().String()
	}
	a.CreatedAt = time.Now()

	if database.DB != nil {
		alert := models.Alert{
			Kind:      a.Kind,
			Severity:  a.Severity,
			Message:   a.Message,
			CreatedAt: a.CreatedAt,
		}
		if err := database.DB.Create(&alert).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		a.ID = string(rune(alert.ID))

		// Broadcast via WebSocket
		if wsManager != nil {
			wsManager.BroadcastAlert(alert)
		}
		c.JSON(http.StatusCreated, a)
		return
	}

	// In-memory fallback
	alerts = append(alerts, a)
	if wsManager != nil {
		wsManager.BroadcastAlert(models.Alert{
			Kind:      a.Kind,
			Severity:  a.Severity,
			Message:   a.Message,
			CreatedAt: a.CreatedAt,
		})
	}
	c.JSON(http.StatusCreated, a)
}

func resolveAlert(c *gin.Context) {
	id := c.Param("id")

	if database.DB != nil {
		if err := database.DB.Model(&models.Alert{}).Where("id = ?", id).Update("resolved", true).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		c.JSON(http.StatusOK, gin.H{"message": "alert resolved"})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "alert resolved"})
}

// In-memory placeholder stores (fallback when DB not available)
var policies = make(map[string]Policy)
var alerts = make([]Alert, 0)

type Policy struct {
	ID          string    `json:"id"`
	Name        string    `json:"name"`
	Description string    `json:"description"`
	CreatedAt   time.Time `json:"created_at"`
}

type Alert struct {
	ID        string    `json:"id"`
	Kind      string    `json:"kind"`
	Severity  string    `json:"severity"`
	Message   string    `json:"message"`
	CreatedAt time.Time `json:"created_at"`
}
