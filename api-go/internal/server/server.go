package server

import (
	"log"

	"github.com/cyber-kube/api-go/internal/config"
	"github.com/cyber-kube/api-go/internal/handlers"
	"github.com/cyber-kube/api-go/internal/services"
	"github.com/cyber-kube/api-go/pkg/auth"
	"github.com/gin-gonic/gin"
)

type Server struct {
	config   *config.Config
	handlers *handlers.Handlers
}

func New(cfg *config.Config) *Server {
	// Initialize services
	k8sService, err := services.NewK8sService(cfg)
	if err != nil {
		log.Printf("warning: kubernetes client unavailable: %v", err)
	}
	policyService := services.NewPolicyService()
	alertService := services.NewAlertService()

	// Initialize handlers
	h := handlers.New(k8sService, policyService, alertService)

	return &Server{
		config:   cfg,
		handlers: h,
	}
}

func (s *Server) SetupRoutes(router *gin.Engine) {
	api := router.Group("/api/v1")

	// Health
	api.GET("/health", s.handlers.Health)

	// Authentication
	api.POST("/auth/login", auth.LoginHandler)

	protected := router.Group("/api/v1")
	protected.Use(auth.AuthMiddleware())

	// Policies
	protected.POST("/policies", s.handlers.CreatePolicy)
	protected.GET("/policies", s.handlers.ListPolicies)
	protected.GET("/policies/:id", s.handlers.GetPolicy)
	protected.PUT("/policies/:id", s.handlers.UpdatePolicy)
	protected.DELETE("/policies/:id", s.handlers.DeletePolicy)

	// Alerts
	protected.GET("/alerts", s.handlers.ListAlerts)
	protected.POST("/alerts", s.handlers.CreateAlert)
	protected.GET("/alerts/:id", s.handlers.GetAlert)

	// Events (WebSocket)
	api.GET("/events", s.handlers.EventsWS)

	// Metrics
	protected.GET("/metrics", s.handlers.Metrics)
}
