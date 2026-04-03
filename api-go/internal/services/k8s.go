package services

import (
	"context"
	"errors"
	"fmt"

	"github.com/cyber-kube/api-go/internal/config"
	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/client-go/kubernetes"
	"k8s.io/client-go/rest"
	"k8s.io/client-go/tools/clientcmd"
)

type K8sService struct {
	config    *config.Config
	clientset *kubernetes.Clientset
}

func NewK8sService(cfg *config.Config) (*K8sService, error) {
	service := &K8sService{
		config: cfg,
	}

	var kubeconfig *rest.Config
	var err error

	if cfg.KubeconfigPath != "" {
		kubeconfig, err = clientcmd.BuildConfigFromFlags("", cfg.KubeconfigPath)
	} else {
		kubeconfig, err = rest.InClusterConfig()
	}

	if err != nil {
		return service, fmt.Errorf("failed to build kubernetes config: %w", err)
	}

	clientset, err := kubernetes.NewForConfig(kubeconfig)
	if err != nil {
		return service, fmt.Errorf("failed to create kubernetes clientset: %w", err)
	}

	service.clientset = clientset
	return service, nil
}

func (s *K8sService) ListPods(namespace string) ([]corev1.Pod, error) {
	if s == nil || s.clientset == nil {
		return nil, errors.New("kubernetes clientset is not initialized")
	}

	podList, err := s.clientset.CoreV1().Pods(namespace).List(context.Background(), metav1.ListOptions{})
	if err != nil {
		return nil, err
	}

	return podList.Items, nil
}

func (s *K8sService) GetPod(namespace, name string) (*corev1.Pod, error) {
	if s == nil || s.clientset == nil {
		return nil, errors.New("kubernetes clientset is not initialized")
	}

	return s.clientset.CoreV1().Pods(namespace).Get(context.Background(), name, metav1.GetOptions{})
}

func (s *K8sService) ListNamespaces() ([]corev1.Namespace, error) {
	if s == nil || s.clientset == nil {
		return nil, errors.New("kubernetes clientset is not initialized")
	}

	namespaceList, err := s.clientset.CoreV1().Namespaces().List(context.Background(), metav1.ListOptions{})
	if err != nil {
		return nil, err
	}

	return namespaceList.Items, nil
}
