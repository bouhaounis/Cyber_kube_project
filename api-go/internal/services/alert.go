package services

import (
	"github.com/cyber-kube/api-go/internal/database"
)

type AlertService struct {
}

func NewAlertService() *AlertService {
	return &AlertService{}
}

func (s *AlertService) List() []database.Alert {
	var alerts []database.Alert
	database.DB.Order("created_at DESC").Find(&alerts)
	return alerts
}

func (s *AlertService) Get(id string) *database.Alert {
	var alert database.Alert
	if err := database.DB.First(&alert, "id = ?", id).Error; err != nil {
		return nil
	}
	return &alert
}

func (s *AlertService) Create(alert database.Alert) (*database.Alert, error) {
	if err := database.DB.Create(&alert).Error; err != nil {
		return nil, err
	}
	return &alert, nil
}
