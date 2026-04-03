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
	"github.com/cyber-kube/api-go/internal/server"
	"github.com/gin-gonic/gin"
)

func main() {
	// Load configuration
	cfg := config.Load()
	database.Connect(cfg)

	// Initialize server
	srv := server.New(cfg)

	// Setup routes
	router := gin.Default()
	srv.SetupRoutes(router)

	httpServer := &http.Server{
		Addr:           cfg.Addr,
		Handler:        router,
		ReadTimeout:    5 * time.Second,
		WriteTimeout:   10 * time.Second,
		MaxHeaderBytes: 1 << 20,
	}

	// Start server
	go func() {
		log.Printf("Server starting on %s", httpServer.Addr)
		if err := httpServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("listen: %s\n", err)
		}
	}()

	// Wait for interrupt signal
	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	log.Println("Shutting down server...")
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	if err := httpServer.Shutdown(ctx); err != nil {
		log.Fatal("Server forced to shutdown:", err)
	}
	if err := database.Close(); err != nil {
		log.Printf("Error closing database: %v", err)
	}
	log.Println("Server exited")
}
