package main

import (
	"errors"
	"net/http"
	"strconv"
	"time"

	"github.com/cyber-kube/api-go/internal/config"
	"github.com/cyber-kube/api-go/internal/database"
	"github.com/cyber-kube/api-go/internal/models"
	"github.com/cyber-kube/api-go/pkg/notify"
	"github.com/cyber-kube/api-go/pkg/websocket"
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"gorm.io/gorm"
)

var appConfig *config.Config

type SettingsResponse struct {
	EmailNotifications bool   `json:"email_notifications"`
	NotificationEmail  string `json:"notification_email"`
	AutoRemediation    bool   `json:"auto_remediation"`
	LogRetentionDays   int    `json:"log_retention_days"`
}

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
	r.GET("/settings", getSettings)
	r.PUT("/settings", updateSettings)
	r.POST("/settings/test-email", sendTestEmail)

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
		if _, err := uuid.Parse(p.ID); err != nil {
			p.ID = uuid.New().String()
		}
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
				ID:        strconv.FormatUint(uint64(a.ID), 10),
				Kind:      a.Kind,
				Severity:  a.Severity,
				Message:   a.Message,
				Source:    a.Source,
				Namespace: a.Namespace,
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
			Source:    a.Source,
			Namespace: a.Namespace,
			CreatedAt: a.CreatedAt,
		}
		if err := database.DB.Create(&alert).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		a.ID = strconv.FormatUint(uint64(alert.ID), 10)

		// Broadcast via WebSocket
		if wsManager != nil {
			wsManager.BroadcastAlert(alert)
		}
		maybeSendAlertNotification(models.Alert{
			ID:        alert.ID,
			Kind:      alert.Kind,
			Severity:  alert.Severity,
			Message:   alert.Message,
			Source:    alert.Source,
			Namespace: alert.Namespace,
			CreatedAt: alert.CreatedAt,
			UpdatedAt: alert.UpdatedAt,
		})
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

func getSettings(c *gin.Context) {
	settings, err := loadSettings()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, SettingsResponse{
		EmailNotifications: settings.EmailNotifications,
		NotificationEmail:  settings.NotificationEmail,
		AutoRemediation:    settings.AutoRemediation,
		LogRetentionDays:   settings.LogRetentionDays,
	})
}

func updateSettings(c *gin.Context) {
	var payload SettingsResponse
	if err := c.ShouldBindJSON(&payload); err != nil {
		c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
		return
	}

	settings, err := loadSettings()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	settings.EmailNotifications = payload.EmailNotifications
	settings.NotificationEmail = payload.NotificationEmail
	settings.AutoRemediation = payload.AutoRemediation
	if payload.LogRetentionDays > 0 {
		settings.LogRetentionDays = payload.LogRetentionDays
	}

	if database.DB != nil {
		if err := database.DB.Save(&settings).Error; err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
	}

	c.JSON(http.StatusOK, SettingsResponse{
		EmailNotifications: settings.EmailNotifications,
		NotificationEmail:  settings.NotificationEmail,
		AutoRemediation:    settings.AutoRemediation,
		LogRetentionDays:   settings.LogRetentionDays,
	})
}

func sendTestEmail(c *gin.Context) {
	settings, err := loadSettings()
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	if !settings.EmailNotifications {
		c.JSON(http.StatusBadRequest, gin.H{"error": "email notifications are disabled"})
		return
	}

	recipient := settings.NotificationEmail
	if recipient == "" && appConfig != nil {
		recipient = appConfig.SMTPTo
	}
	if recipient == "" {
		c.JSON(http.StatusBadRequest, gin.H{"error": "notification email is required"})
		return
	}

	err = notify.SendEmail(
		appConfig,
		recipient,
		"Cyber-Kube test email",
		"Cyber-Kube email notifications are configured correctly.\r\n\r\nThis is a test message from the Settings page.",
	)
	if err != nil {
		c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
		return
	}

	c.JSON(http.StatusOK, gin.H{"message": "test email sent", "to": recipient})
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
	Source    string    `json:"source"`
	Namespace string    `json:"namespace"`
	CreatedAt time.Time `json:"created_at"`
}

func loadSettings() (database.AppSettings, error) {
	defaultSettings := database.AppSettings{
		ID:                 "default",
		EmailNotifications: false,
		NotificationEmail:  "",
		AutoRemediation:    true,
		LogRetentionDays:   30,
	}

	if database.DB == nil {
		return defaultSettings, nil
	}

	var settings database.AppSettings
	err := database.DB.First(&settings, "id = ?", "default").Error
	if err == nil {
		return settings, nil
	}
	if !errors.Is(err, gorm.ErrRecordNotFound) {
		return database.AppSettings{}, err
	}

	if createErr := database.DB.Create(&defaultSettings).Error; createErr != nil {
		return database.AppSettings{}, createErr
	}

	return defaultSettings, nil
}

func maybeSendAlertNotification(alert models.Alert) {
	settings, err := loadSettings()
	if err != nil || !settings.EmailNotifications {
		return
	}

	_ = notify.SendAlertEmail(appConfig, settings, notify.AlertPayload{
		ID:        strconv.FormatUint(uint64(alert.ID), 10),
		Kind:      alert.Kind,
		Severity:  alert.Severity,
		Message:   alert.Message,
		Source:    alert.Source,
		Namespace: alert.Namespace,
	})
}
