package ratelimit

import (
	"net/http"
	"sync"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/didip/tollbooth/v7"
	"github.com/didip/tollbooth/v7/limiter"
)

var (
	rateLimiters = make(map[string]*limiter.Limiter)
	mu          sync.RWMutex
)

// CreateRateLimiter creates a rate limiter for a specific route
func CreateRateLimiter(requestsPerSecond float64, burst int) *limiter.Limiter {
	limiter := tollbooth.NewLimiter(requestsPerSecond, &limiter.ExpirableOptions{
		DefaultExpirationTTL: time.Hour,
	})
	limiter.SetBurst(burst)
	return limiter
}

// RateLimitMiddleware applies rate limiting
func RateLimitMiddleware(requestsPerSecond float64, burst int) gin.HandlerFunc {
	key := "default"
	mu.RLock()
	limiter, exists := rateLimiters[key]
	mu.RUnlock()

	if !exists {
		limiter = CreateRateLimiter(requestsPerSecond, burst)
		mu.Lock()
		rateLimiters[key] = limiter
		mu.Unlock()
	}

	return func(c *gin.Context) {
		httpError := tollbooth.LimitByRequest(limiter, c.Writer, c.Request)
		if httpError != nil {
			c.JSON(http.StatusTooManyRequests, gin.H{
				"error": "Rate limit exceeded",
			})
			c.Abort()
			return
		}
		c.Next()
	}
}

// StrictRateLimit for sensitive endpoints
func StrictRateLimit() gin.HandlerFunc {
	return RateLimitMiddleware(1.0, 5) // 1 req/sec, burst of 5
}

// StandardRateLimit for normal endpoints
func StandardRateLimit() gin.HandlerFunc {
	return RateLimitMiddleware(10.0, 20) // 10 req/sec, burst of 20
}
