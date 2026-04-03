package main

import (
	"context"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/cyber-kube/api-go/internal/config"
	"github.com/cyber-kube/api-go/internal/database"
	"github.com/cyber-kube/api-go/pkg/auth"
	"github.com/cyber-kube/api-go/pkg/ratelimit"
	"github.com/cyber-kube/api-go/pkg/websocket"
	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus/promhttp"
)

var wsManager *websocket.Manager

func main() {
	cfg := config.Load()

	// Initialize database (optional, will use in-memory if not available)
	database.Connect(cfg)

	// Initialize WebSocket manager
	wsManager = websocket.NewManager()
	go wsManager.Start()

	if os.Getenv("GIN_MODE") == "" {
		gin.SetMode(gin.ReleaseMode)
	}

	router := gin.Default()

	// CORS middleware
	router.Use(func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, accept, origin, Cache-Control, X-Requested-With")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS, GET, PUT, DELETE")

		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}
		c.Next()
	})

	// Public routes
	public := router.Group("/api/v1")
	{
		// Health check (no auth, no rate limit)
		public.GET("/health", func(c *gin.Context) {
			c.JSON(http.StatusOK, gin.H{
				"status":    "ok",
				"timestamp": time.Now().UTC(),
				"version":   "1.0.0",
			})
		})

		// Authentication
		public.POST("/auth/login", ratelimit.StrictRateLimit(), auth.LoginHandler)
	}

	// Protected routes with rate limiting
	protected := router.Group("/api/v1")
	protected.Use(ratelimit.StandardRateLimit())
	protected.Use(auth.AuthMiddleware())
	{
		registerRoutes(protected, wsManager)
		protected.GET("/metrics", gin.WrapH(promhttp.Handler()))
	}

	// WebSocket endpoint
	router.GET("/api/v1/events", func(c *gin.Context) {
		wsManager.HandleConnection(c.Writer, c.Request)
	})

	srv := &http.Server{
		Addr:           cfg.Addr,
		Handler:        router,
		ReadTimeout:    10 * time.Second,
		WriteTimeout:   10 * time.Second,
		MaxHeaderBytes: 1 << 20,
	}

	go func() {
		log.Printf("🚀 Cyber-Kube API Server starting on %s", srv.Addr)
		log.Printf("📊 Metrics available at http://localhost%s/api/v1/metrics", srv.Addr)
		log.Printf("🔌 WebSocket available at ws://localhost%s/api/v1/events", srv.Addr)
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("listen: %s\n", err)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	log.Println("Shutting down server...")
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := srv.Shutdown(ctx); err != nil {
		log.Fatal("Server forced to shutdown:", err)
	}

	if database.DB != nil {
		database.Close()
	}
	log.Println("Server exited gracefully")
}
