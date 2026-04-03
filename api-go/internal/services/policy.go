package services

import (
	"github.com/cyber-kube/api-go/internal/database"
)

type PolicyService struct {
}

func NewPolicyService() *PolicyService {
	return &PolicyService{}
}

func (s *PolicyService) List() []database.Policy {
	var policies []database.Policy
	database.DB.Order("created_at DESC").Find(&policies)
	return policies
}

func (s *PolicyService) Get(id string) *database.Policy {
	var policy database.Policy
	if err := database.DB.First(&policy, "id = ?", id).Error; err != nil {
		return nil
	}
	return &policy
}

func (s *PolicyService) Create(policy database.Policy) (*database.Policy, error) {
	if err := database.DB.Create(&policy).Error; err != nil {
		return nil, err
	}
	return &policy, nil
}

func (s *PolicyService) Update(id string, policy database.Policy) (*database.Policy, error) {
	policy.ID = id
	if err := database.DB.Model(&database.Policy{}).Where("id = ?", id).Updates(policy).Error; err != nil {
		return nil, err
	}

	return s.Get(id), nil
}

func (s *PolicyService) Delete(id string) {
	database.DB.Delete(&database.Policy{}, "id = ?", id)
}
