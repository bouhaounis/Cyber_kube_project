package config

import (
	"os"
)

type Config struct {
	Addr           string
	EngineURL      string
	DatabaseURL    string
	RedisURL       string
	KubeconfigPath string
	JWTSecret      string
	LogLevel       string
}

func Load() *Config {
	return &Config{
		Addr:           getEnv("ADDR", ":8081"),
		EngineURL:      getEnv("ENGINE_URL", "http://localhost:7000"),
		DatabaseURL:    getEnv("DATABASE_URL", "postgres://localhost/cyberkube"),
		RedisURL:       getEnv("REDIS_URL", "redis://localhost:6379"),
		KubeconfigPath: getEnv("KUBECONFIG", ""),
		JWTSecret:      getEnv("JWT_SECRET", "change-me-in-production"),
		LogLevel:       getEnv("LOG_LEVEL", "info"),
	}
}

func getEnv(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}
