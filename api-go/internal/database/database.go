package database

import (
	"fmt"
	"time"

	"github.com/cyber-kube/api-go/internal/config"
	"github.com/google/uuid"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
	"gorm.io/gorm/logger"
)

type Policy struct {
	ID          string    `gorm:"type:uuid;primaryKey" json:"id"`
	Name        string    `gorm:"not null" json:"name"`
	Description string    `json:"description"`
	Enabled     bool      `gorm:"default:true" json:"enabled"`
	Rules       string    `gorm:"type:text" json:"rules"`
	CreatedAt   time.Time `json:"created_at"`
	UpdatedAt   time.Time `json:"updated_at"`
}

type Alert struct {
	ID        string    `gorm:"type:uuid;primaryKey" json:"id"`
	Kind      string    `gorm:"not null;index" json:"kind"`
	Severity  string    `gorm:"not null;index" json:"severity"`
	Message   string    `gorm:"type:text" json:"message"`
	Source    string    `json:"source"`
	Namespace string    `json:"namespace"`
	Resolved  bool      `gorm:"default:false" json:"resolved"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
}

type AppSettings struct {
	ID                 string    `gorm:"primaryKey" json:"id"`
	EmailNotifications bool      `gorm:"default:false" json:"email_notifications"`
	NotificationEmail  string    `json:"notification_email"`
	AutoRemediation    bool      `gorm:"default:true" json:"auto_remediation"`
	LogRetentionDays   int       `gorm:"default:30" json:"log_retention_days"`
	CreatedAt          time.Time `json:"created_at"`
	UpdatedAt          time.Time `json:"updated_at"`
}

var DB *gorm.DB

func (p *Policy) BeforeCreate(_ *gorm.DB) error {
	if _, err := uuid.Parse(p.ID); err != nil {
		p.ID = uuid.NewString()
	}
	return nil
}

func (a *Alert) BeforeCreate(_ *gorm.DB) error {
	if a.ID == "" {
		a.ID = uuid.NewString()
	}
	return nil
}

func (s *AppSettings) BeforeCreate(_ *gorm.DB) error {
	if s.ID == "" {
		s.ID = "default"
	}
	return nil
}

func Connect(cfg *config.Config) {
	db, err := gorm.Open(postgres.Open(cfg.DatabaseURL), &gorm.Config{
		Logger: logger.Default.LogMode(logger.Warn),
		NowFunc: func() time.Time {
			return time.Now().UTC()
		},
	})
	if err != nil {
		panic(fmt.Sprintf("failed to connect to database: %v", err))
	}

	if err := db.AutoMigrate(&Policy{}, &Alert{}, &AppSettings{}); err != nil {
		panic(fmt.Sprintf("failed to migrate database: %v", err))
	}

	DB = db
}

func Close() error {
	if DB == nil {
		return nil
	}

	sqlDB, err := DB.DB()
	if err != nil {
		return err
	}

	return sqlDB.Close()
}
